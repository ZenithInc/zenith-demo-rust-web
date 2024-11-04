use crate::cron::check_lamp_status::handle as check_lamp_status;
use crate::cron::lamp_check_offline::handle as check_offline;
use crate::cron::cron_task_manager::CronTaskManager;
use crate::tasks::mqtt_tasks;
use crate::tasks::task_manager::TaskManager;
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::{event, Level};
use crate::tasks;

pub async fn init_tasks(notify: Arc<Notify>) {
    let task_manager = TaskManager::new(notify);
    task_manager.register_task(mqtt_tasks::notify).await;
    task_manager.register_task(tasks::mqtt_status_tasks::notify).await;
    task_manager.start_tasks().await;

    event!(Level::INFO, "tasks initialized");
}

pub async fn init_cron_tasks() {
    let task_manager = CronTaskManager::new();

    task_manager.register_task(
        "UV device network status check".to_string(),
        "*/1 * * * * *",
        Arc::new(|| check_lamp_status()),
    ).await;

    task_manager.register_task(
        "Lamp offline check".to_string(),
        "*/1 * * * * *",
        Arc::new(|| check_offline()),
    ).await;

    task_manager.start().await
}
