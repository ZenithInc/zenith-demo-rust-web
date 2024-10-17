use crate::repositories::uv_lamp_mqtt_notify_job::{Job, UVLampMqttNotifyJob};
use chrono::Utc;
use futures::future::BoxFuture;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use reqwest::{Client, Response};
use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tokio::sync::{Notify, Semaphore};
use tracing::{debug, error, info};

enum NextRetryDuration {
    OneMinute,
    ThreeMinutes,
    FifteenMinutes,
    OneHour,
    SixHours,
    TwelveHours,
}

#[derive(Debug)]
enum LampStatus {
    Free,
    Off,
    Check,
    Running,
}

impl LampStatus {
    fn as_int(&self) -> u8 {
        match self {
            LampStatus::Free => 0,
            LampStatus::Off => 1,
            LampStatus::Check => 2,
            LampStatus::Running => 3,
        }
    }
}

impl Serialize for LampStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.as_int())
    }
}

impl<'de> Deserialize<'de> for LampStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(LampStatus::Free),
            1 => Ok(LampStatus::Off),
            2 => Ok(LampStatus::Check),
            3 => Ok(LampStatus::Running),
            _ => Err(serde::de::Error::custom("Invalid value for LampStatus")),
        }
    }
}

#[derive(Debug)]
enum Reason {
    // 状态更新
    StatusModified,
    // 定时打开
    TimedOpen,
    // 定时关闭
    TimedOff,
    // 平台打开
    PlatformOpen,
    // 平台关闭
    PlatformOff,
    // 发生红外报警
    InfraredAlarmActivated,
    // 红外报警解除
    InfraredAlarmDeactivated,
    // 检测正常
    DetectionNormal,
    // 非法灯管
    IllegalLamp,
}

impl Serialize for Reason {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.as_int())
    }
}

impl<'de> Deserialize<'de> for Reason {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            1 => Ok(Reason::StatusModified),
            2 => Ok(Reason::TimedOpen),
            3 => Ok(Reason::TimedOff),
            4 => Ok(Reason::PlatformOpen),
            5 => Ok(Reason::PlatformOff),
            6 => Ok(Reason::InfraredAlarmActivated),
            7 => Ok(Reason::InfraredAlarmDeactivated),
            8 => Ok(Reason::DetectionNormal),
            9 => Ok(Reason::IllegalLamp),
            _ => Err(serde::de::Error::custom("Invalid value for Reason")),
        }
    }
}

impl Reason {
    fn as_int(&self) -> u8 {
        match self {
            Reason::StatusModified => 1,
            Reason::TimedOpen => 2,
            Reason::TimedOff => 3,
            Reason::PlatformOpen => 4,
            Reason::PlatformOff => 5,
            Reason::InfraredAlarmActivated => 6,
            Reason::InfraredAlarmDeactivated => 7,
            Reason::DetectionNormal => 8,
            Reason::IllegalLamp => 9,
        }
    }
}

#[derive(Deserialize, Debug)]
struct Payload {
    // 灯的状态
    #[serde(rename = "s")]
    status: LampStatus,

    // 紫外线强度，最大为 200
    #[serde(rename = "u")]
    strength: i8,

    // 消毒开启时间，单位分钟
    #[serde(rename = "d")]
    duration: i32,

    // 时间: YYYY-mm-dd HH:mm:ss
    #[serde(rename = "ts")]
    timestamp: String,

    // 切换到当前状态的原因
    #[serde(rename = "c")]
    reason: Reason,
}

#[derive(Serialize, Debug)]
struct NotifyBody {
    status: LampStatus,
    device_number: String,
    strength: i8,
    duration: i32,
    timestamp: String,
    reason: Reason,
}

impl NotifyBody {
    fn from_payload(payload: Payload, device_number: String) -> Self {
        NotifyBody {
            device_number,
            status: payload.status,
            strength: payload.strength,
            duration: payload.duration,
            timestamp: payload.timestamp,
            reason: payload.reason,
        }
    }
}


impl NextRetryDuration {
    fn as_seconds(&self) -> u64 {
        match self {
            NextRetryDuration::OneMinute => 60,
            NextRetryDuration::ThreeMinutes => 3 * 60,
            NextRetryDuration::FifteenMinutes => 15 * 60,
            NextRetryDuration::OneHour => 60 * 60,
            NextRetryDuration::SixHours => 6 * 60 * 60,
            NextRetryDuration::TwelveHours => 12 * 60 * 60,
        }
    }

    fn from_retry_count(count: u8) -> Option<NextRetryDuration> {
        match count {
            1 => Some(NextRetryDuration::OneMinute),
            2 => Some(NextRetryDuration::ThreeMinutes),
            3 => Some(NextRetryDuration::FifteenMinutes),
            4 => Some(NextRetryDuration::OneHour),
            5 => Some(NextRetryDuration::SixHours),
            6 => Some(NextRetryDuration::TwelveHours),
            _ => None,
        }
    }
}

pub fn notify(notify: Arc<Notify>) -> BoxFuture<'static, ()> {
    Box::pin(async move {
        loop {
            info!("MQTT notify task start running...");
            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(15)) => {
                     handle_notify().await;
                },
                _ = notify.notified() => {
                    info!("MQTT notify received stop signal!");
                    break;
                }
            }
            info!("MQTT notify task stopped!");
        }
    })
}

fn handle_notify() -> BoxFuture<'static, ()> {
    Box::pin(async move {
        let max_retry_count = std::env::var("UV_LAMP_MQTT_TASK_RETRY_MAX_COUNT")
            .unwrap_or_else(|_| "6".to_string())
            .parse::<u8>()
            .unwrap_or(6);

        match UVLampMqttNotifyJob::get_incomplete_jobs(max_retry_count).await {
            Ok(jobs) => {
                info!("Find jobs: {}", jobs.len());
                send_requests(jobs).await;
            }
            Err(err) => error!("Failed to get incomplete jobs: {}", err),
        }
    })
}

async fn send_requests(jobs: Vec<Job>) {
    let timeout_seconds = std::env::var("UV_LAMP_MQTT_TASK_TIMEOUT")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .unwrap_or_else(|_| 5);
    let result = Client::builder()
        .timeout(Duration::from_secs(timeout_seconds))
        .build();
    match result {
        Ok(client) => {
            let concurrency_limit = 10;
            let semaphore = Arc::new(Semaphore::new(concurrency_limit));

            let mut futures = FuturesUnordered::new();

            for job in jobs {
                let client = client.clone();
                let semaphore = semaphore.clone();

                futures.push(tokio::spawn(async move {
                    send_request(&job, &semaphore, &client).await;
                }));
            }

            while let Some(_) = futures.next().await {}
        },
        Err(_) => error!("Failed to create client"),
    }
}

async fn send_request(job: &Job, semaphore: &Arc<Semaphore>, client: &Client) {
    let _permit = semaphore.acquire().await;
    let result = std::env::var("UV_LAMP_MQTT_TASK_NOTIFY_URL");
    match result {
        Ok(url) => {
            if let Some(body) = notify_contents_2_payload(&job.notify_contents, &job.device_number) {
                let request_result = client
                    .post(url)
                    .json(&body)
                    .send()
                    .await;
                match request_result {
                    Ok(response) => handle_received_response(&job, response).await,
                    Err(err) => {
                        error!("Request endpoint failed: {}", err);
                        handle_error(&job).await
                    },
                }
            } else {
                error!("Failed to notify contents");
            }
        }
        Err(_) => error!("Get notify url of task failed!")
    }
}

async fn handle_received_response(job: &&Job, response: Response) {
    if response.status().is_success() {
        let result = UVLampMqttNotifyJob::update_success(job.id).await;
        match result {
            Ok(_) => info!("Job notify task has completed successfully!"),
            Err(err) => error!("Failed update to notify job: {}", err),
        }
    } else {
        error!("Request endpoint failed, status is {}", response.status().as_str());
        handle_error(&job).await;
    }
}

fn notify_contents_2_payload(notify_contents: &String, device_number: &String) -> Option<String> {
    let payload: Option<Payload> = match serde_json::from_str(&notify_contents) {
        Ok(body) => Some(body),
        Err(_) => {
            error!("Failed to parse notify contents!");
            return None;
        }
    };
    if let Some(payload) = payload {
        let body = NotifyBody::from_payload(payload, device_number.clone());
        return match serde_json::to_string(&body) {
            Ok(body) => {
                debug!("Sending notification body: {}", body);
                Some(body)
            },
            Err(_) => {
                error!("Failed to serialize notify contents!");
                None
            },
        };
    }
    None
}

async fn handle_error(job: &Job) {
    let retry_count = job.retry_count + 1;
    let option_seconds = NextRetryDuration::from_retry_count(retry_count);
    match option_seconds {
        Some(seconds) => {
            let current_timestamp = Utc::now().timestamp() as u64;
            let next_timestamp = current_timestamp + seconds.as_seconds();

            let result = UVLampMqttNotifyJob::update_retry_count(
                job.id,
                retry_count,
                job.retry_count,
                next_timestamp,
            )
                .await;
            match result {
                Ok(_) => {},
                Err(err) => error!("Update retry count failed: {}", err),
            }
        }
        None => {
            let result = UVLampMqttNotifyJob::update_failed(job.id).await;
            match result {
                Ok(_) => {},
                Err(err)  => error!("Update failed failed: {}", err),
            }
        },
    }
}
