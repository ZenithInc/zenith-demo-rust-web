use crate::repositories::uv_lamp_mqtt_notify_job::{Job, UVLampMqttNotifyJob};
use chrono::Utc;
use futures::future::BoxFuture;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use reqwest::{Client, Error};
use std::sync::Arc;
use tokio::sync::{Notify, Semaphore};
use tracing::{error, info};

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
                send_requests(jobs).await;
            },
            Err(err) => error!("Failed to get incomplete jobs: {}", err),
        }
    })
}

async fn send_requests(jobs: Vec<Job>) {
    let client = Client::new();
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
}

async fn send_request(job: &Job, semaphore: &Arc<Semaphore>, client: &Client) {
    let _permit = semaphore.acquire().await;
    let result = client
        .post("https://api.example.com/submit")
        .json(&job.notify_contents)
        .send()
        .await;
    match result {
        Ok(_) => {}
        Err(err) => handle_error(&job, err).await,
    }
}

async fn handle_error(job: &Job, err: Error) {
    error!("Request endpoint failed: {}", err);
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
                Ok(_) => {}
                Err(err) => error!("Update retry count failed: {}", err),
            }
        }
        None => error!("Failed to send request to next retry interval"),
    }
}
