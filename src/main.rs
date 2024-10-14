use axum::response::IntoResponse;
use axum::{
    extract::Request as ExtractRequest, http::StatusCode, middleware, response::Response,
    Extension, Json as AxumJson, Router,
};
use chrono_tz::Tz;
use rust_demo::routes::uv_lamp::register_uv_lamp_routes;
use rust_demo::tasks::mqtt_tasks;
use rust_demo::tasks::task_manager::TaskManager;
use rust_demo::utils;
use rust_demo::utils::config;
use serde_json::json;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::Notify;
use tracing::{event, Level};
use tracing_subscriber::fmt::SubscriberBuilder;

#[tokio::main]
async fn main() {
    config::init();

    let timezone: Tz = std::env::var("TIMEZONE")
        .unwrap_or_else(|_| "Asia/Shanghai".to_string())
        .parse()
        .expect("Invalid timezone");
    let shared_timezone = Arc::new(timezone);

    let log_path = std::env::var("LOG_PATH").unwrap_or_else(|_| "./logs".to_string());
    let log_filename_prefix =
        std::env::var("LOG_FILEPATH_PREFIX").unwrap_or_else(|_| "log.json".to_string());

    let file_appender = tracing_appender::rolling::daily(log_path, log_filename_prefix);

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    SubscriberBuilder::default()
        .with_max_level(Level::INFO)
        .with_writer(non_blocking)
        .init();

    event!(Level::INFO, "config initialized");

    utils::mqtt::init_mqtt_handler().await.unwrap();
    event!(Level::INFO, "mqtt handler initialized");

    let notify = Arc::new(Notify::new());
    let _notify_clone = notify.clone();

    let task_manager = TaskManager::new();

    task_manager.register_task(mqtt_tasks::notify).await;

    task_manager.start_tasks().await;

    let app = Router::new()
        .merge(register_uv_lamp_routes())
        .layer(Extension(shared_timezone))
        .layer(middleware::from_fn(error_handler));

    let bind = std::env::var("BIND").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    event!(Level::INFO, "server started at {}", bind);

    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(notify))
        .await
        .unwrap();
}

async fn shutdown_signal(notify: Arc<Notify>) {
    signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    println!("Shutdown signal received");

    // Notify task to stop
    notify.notify_one();
}

async fn error_handler(req: ExtractRequest, next: middleware::Next) -> Response {
    match req.extensions().get::<anyhow::Error>() {
        Some(err) => {
            let status_code = StatusCode::INTERNAL_SERVER_ERROR;
            let error_message = err.to_string();
            (
                status_code,
                AxumJson(json!({
                    "message": error_message,
                })),
            )
                .into_response()
        }
        None => next.run(req).await,
    }
}
