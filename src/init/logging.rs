use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use std::sync::Arc;
use tracing::{event, Level};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::EnvFilter;

struct LocalTimeFormatter {
    timezone: Arc<Tz>,
}

impl tracing_subscriber::fmt::time::FormatTime for LocalTimeFormatter {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let now_utc: DateTime<Utc> = Utc::now();
        let now_local = now_utc.with_timezone(&*self.timezone);
        write!(w, "{}", now_local.format("%Y-%m-%d %H:%M:%S"))
    }
}

pub fn init_logging(timezone: Arc<Tz>) -> WorkerGuard {
    let log_path = std::env::var("LOG_PATH").unwrap_or_else(|_| "./logs".to_string());
    let log_filename_prefix =
        std::env::var("LOG_FILEPATH_PREFIX").unwrap_or_else(|_| "log.json".to_string());

    let file_appender = tracing_appender::rolling::daily(log_path, log_filename_prefix);

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    println!("Current log level: {}", log_level);

    tracing_subscriber::fmt()
        .with_timer(LocalTimeFormatter { timezone })
        .with_env_filter(EnvFilter::new(log_level))
        .with_writer(non_blocking)
        .init();

    event!(Level::INFO, "Logging initialized!");

    guard
}
