use std::sync::Arc;
use tokio::sync::Notify;
use tracing::{event,Level};
use crate::cron::check_lamp_status::handle as check_lamp_status;
use crate::cron::cron_task_manager::CronTaskManager;
use crate::tasks::mqtt_tasks;
use crate::tasks::task_manager::TaskManager;

pub async fn init_tasks(notify: Arc<Notify>) {
    let task_manager = TaskManager::new(notify);
    task_manager.register_task(mqtt_tasks::notify).await;
    task_manager.start_tasks().await;

    event!(Level::INFO, "tasks initialized");
}

pub async fn init_cron_tasks() {
    let task_manager = CronTaskManager::new();
    task_manager.register_task(
        "task1".to_string(),
        "*/5 * * * * *",
        Arc::new(check_lamp_status),
    ).await.unwrap();

    task_manager.start().await
}