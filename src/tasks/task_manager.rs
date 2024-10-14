use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

type TaskLogic = Arc<dyn Fn(Arc<Notify>) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> + Send + Sync>;

pub struct TaskManager {
    notify: Arc<Notify>,
    tasks: Arc<Mutex<Vec<TaskLogic>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager {
            notify: Arc::new(Notify::new()),
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn register_task<F, Fut>(&self, task_logic: F)
    where
        F: Fn(Arc<Notify>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let boxed_task: TaskLogic = Arc::new(move |notify| {
            let fut = task_logic(notify);
            Box::pin(fut) as Pin<Box<dyn Future<Output = ()> + Send + Sync>>
        });
        let mut tasks = self.tasks.lock().await;
        tasks.push(boxed_task);
    }

    pub async fn start_tasks(&self) {
        let tasks = self.tasks.lock().await.clone();
        for task in tasks {
            let task = task.clone();
            let notify_clone = self.notify.clone();
            tokio::spawn(async move {
                (task)(notify_clone).await;
            });
        }
    }

}