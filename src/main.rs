use chrono_tz::Tz;
use connect_x::utils;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::Notify;
use tracing::{event, Level};
use connect_x::init::{init_config, init_logging, init_routes, init_tasks};

#[tokio::main]
async fn main() {
    init_config();

    let timezone: Tz = std::env::var("TIMEZONE")
        .unwrap_or_else(|_| "Asia/Shanghai".to_string())
        .parse()
        .expect("Invalid timezone");
    let shared_timezone = Arc::new(timezone);

    // It is crucial to store the log guard returned by `init_logging` to ensure that the
    // logging subsystem remains active for the lifetime of the application. The guard
    // prevents premature flushing of the log buffer, which could result in incomplete
    // log entries if the guard is dropped too soon. By maintaining the guard's lifecycle
    // in the `main` function, we ensure that all logs are properly written before the
    // application exits.
    let log_guard = init_logging(shared_timezone.clone());

    utils::mqtt::init_mqtt_handler().await.unwrap();
    event!(Level::INFO, "mqtt handler initialized");

    let notify = Arc::new(Notify::new());
    init_tasks(notify.clone()).await;

    let app = init_routes(shared_timezone);

    let bind = std::env::var("BIND").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    event!(Level::INFO, "server started at {}", bind);

    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(notify))
        .await
        .unwrap();

    drop(log_guard);
}

async fn shutdown_signal(notify: Arc<Notify>) {
    signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    println!("Shutdown signal received");

    // Notify task to stop
    notify.notify_one();
}