use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::Duration;
use chrono::Local;
use futures::future::BoxFuture;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use reqwest::{Client};
use serde::{Deserialize, Serialize};
use tokio::sync::{Notify, Semaphore};
use tracing::{debug, error, info};
use crate::repositories::uv_lamp_mqtt_notify_job::{Job, UVLampMqttNotifyJob};
use crate::tasks::{handle_error, handle_received_response, TaskType};

#[derive(Debug, Clone)]
struct Config {
    max_retry_count: u8,
    timeout_seconds: u8,
    notify_url: String,
}

#[derive(Debug)]
enum ConfigError {
    MissingNotifyUrl,
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingNotifyUrl => write!(f, "Missing notify URL in environment variables"),
        }
    }
}

impl Config {
    fn load() -> Result<Self, ConfigError> {
        let max_retry_count = std::env::var("UV_LAMP_MQTT_TASK_RETRY_MAX_COUNT")
            .unwrap_or_else(|_| "6".to_string())
            .parse::<u8>()
            .unwrap_or(6);

        let timeout_seconds = std::env::var("UV_LAMP_MQTT_TASK_TIMEOUT)")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u8>()
            .unwrap_or(5);

        let notify_url = std::env::var("UV_LAMP_MQTT_DEVICE_STATUS_NOTIFY_URL")
            .map_err(|_| ConfigError::MissingNotifyUrl)?;

        Ok(Config {
            max_retry_count,
            timeout_seconds,
            notify_url,
        })
    }
}

#[derive(Debug, Deserialize)]
struct Payload {
    // 下面这些字段因为暂时不用，所以注释

    // 查询请求的随机数
    // id: u32,

    // 0 表示成功
    // code: i32,

    // 客户端 IP 地址
    // ip: String,

    // 信号质量（1-5，最大值为 5）
    // rssi: i32,

    // 时间戳（Format: YYYY-MM-DD hh:mm:ss)
    ts: String,
}

#[derive(Debug, Serialize)]
struct NotifyBody {
    // 设备编号
    device_number: String,
    // 是否在线
    is_online: bool,
    // 时间戳
    timestamp: String,
}

impl NotifyBody {
    fn from_payload(payload: Payload, device_number: String) -> Self {
        NotifyBody {
            device_number,
            is_online: true,
            timestamp: payload.ts,
        }
    }
}

pub fn notify(notify: Arc<Notify>) -> BoxFuture<'static, ()> {
    Box::pin(async move {
        loop {
            info!("MQTT device status modify notify task start running...");
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    handle_notify().await;
                },
                _ = notify.notified() => {
                    info!("MQTT notify received stop signal!");
                    break;
                }
            }
            info!("MQTT device status modify notify task stop stopped!");
        }
    })
}

fn handle_notify() -> BoxFuture<'static, ()> {
    Box::pin(async move {
        let config = match Config::load() {
            Ok(config) => config,
            Err(e) => {
                error!("Error loading config: {}", e);
                return;
            }
        };
        let job_type = TaskType::LightStatusTask.to_string();
        match UVLampMqttNotifyJob::get_incomplete_jobs(config.max_retry_count, job_type).await {
            Ok(jobs) => {
                info!("Find jobs: {}", jobs.len());
                send_requests(jobs, &config).await;
            }
            Err(err) => error!("Failed to get incomplete jobs: {}", err),
        }
    })
}

async fn send_requests(jobs: Vec<Job>, config: &Config) {
    let result = Client::builder()
        .timeout(Duration::from_secs(config.timeout_seconds.into()))
        .build();
    let client = match result {
        Ok(client) => client,
        Err(_) => {
            error!("Failed to create HTTP client");
            return;
        }
    };
    let concurrency_limit = 10;
    let semaphore = Arc::new(Semaphore::new(concurrency_limit));
    let mut futures = FuturesUnordered::new();

    for job in jobs {
        let client = client.clone();
        let semaphore = semaphore.clone();
        let config = config.clone();

        futures.push(tokio::spawn(async move {
            send_request(&job, &semaphore, &client, config).await;
        }));
    }

    while let Some(_) = futures.next().await {}
}

async fn send_request(job: &Job, semaphore: &Semaphore, client: &Client, config: Config) {
    let _permit = semaphore.acquire().await;
    let body = build_notify_body(&job);
    debug!("Sending notification: {:?}", body);
    let request_result = client.post(&config.notify_url).json(&body).send().await;
    if let Err(e) = request_result {
        error!("Failed to send notification: {}", e);
        handle_error(&job).await;
    } else if let Ok(response) = request_result {
        handle_received_response(&job, response).await;
    }
}

fn build_notify_body(job: &Job) -> NotifyBody {
    if job.notify_contents.is_empty() {
        // 离线消息
        info!("Send offline notify....");
        let now = Local::now();
        let current_time = now.format("%Y-%m-%d %H:%M:%S").to_string();
        NotifyBody {
            device_number: job.device_number.clone(),
            is_online: false,
            timestamp: current_time,
        }
    } else {
        // 在线消息
        notify_contents_2_payload(&job.notify_contents, &job.device_number)
    }
}

fn notify_contents_2_payload(notify_contents: &String, device_number: &str) -> NotifyBody {
    let payload: Payload = serde_json::from_str(&notify_contents).map_err(|_| {
        error!("Failed to parse notify contents!");
    }).ok().expect("Failed to parse notify contents!");

    NotifyBody::from_payload(payload, device_number.to_string())
}