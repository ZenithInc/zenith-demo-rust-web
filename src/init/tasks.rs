use std::sync::Arc;
use tokio::sync::Notify;
use tracing::{event, Level};
use crate::tasks::mqtt_tasks;
use crate::tasks::task_manager::TaskManager;

pub async fn init_tasks(notify: Arc<Notify>) {
    let task_manager = TaskManager::new(notify);
    task_manager.register_task(mqtt_tasks::notify).await;
    task_manager.start_tasks().await;

    event!(Level::INFO, "tasks initialized");
}