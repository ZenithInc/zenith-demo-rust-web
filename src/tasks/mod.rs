use std::fmt::{Display, Formatter};
use chrono::Utc;
use reqwest::Response;
use tracing::{error, info};
use crate::repositories::uv_lamp_mqtt_notify_job::{Job, UVLampMqttNotifyJob};

pub mod mqtt_tasks;
pub mod task_manager;
pub mod mqtt_status_tasks;

pub enum TaskType {
    LightSwitchTask,
    LightStatusTask,
}

impl Display for TaskType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskType::LightSwitchTask => write!(f, "{}", "LIGHT_SWITCH_TASK"),
            TaskType::LightStatusTask => write!(f, "{}", "LIGHT_STATUS_TASK"),
        }
    }
}

enum NextRetryDuration {
    OneMinute,
    ThreeMinutes,
    FifteenMinutes,
    OneHour,
    SixHours,
    TwelveHours,
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

async fn handle_error(job: &Job) {
    let retry_count = job.retry_count + 1;
    let option_seconds = NextRetryDuration::from_retry_count(retry_count);

    if let Some(seconds) = option_seconds {
        let current_timestamp = Utc::now().timestamp() as u64;
        let next_timestamp  = current_timestamp + seconds.as_seconds();

        if let Err(err) = UVLampMqttNotifyJob::update_retry_count(
            job.id,
            retry_count,
            job.retry_count,
            next_timestamp
        ).await {
            error!("Failed to update retry count: {}", err);
        };
    } else if let Err(err) = UVLampMqttNotifyJob::update_failed(job.id).await {
        error!("Update job failed: {}", err);
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
        error!(
            "Request endpoint failed, status is {}",
            response.status().as_str()
        );
        handle_error(&job).await;
    }
}