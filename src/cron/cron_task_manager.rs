use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use cron::Schedule;
use tokio::sync::Mutex;
use tokio::time;
use tracing::{info};
use tracing::log::error;

type Task = Arc<dyn Fn() + Send + Sync>;

#[derive(Clone)]
struct CronTask {
    schedule: Schedule,
    task: Task,
}

pub struct CronTaskManager {
    tasks: Arc<Mutex<HashMap<String, CronTask>>>,
}

impl CronTaskManager {
    pub fn new() -> Self {
        CronTaskManager {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register_task(&self, name: String, cron_expression: &str, task: Task) -> Result<(), anyhow::Error> {
        let schedule = Schedule::from_str(cron_expression)
            .map_err(|e| {
                error!("Failed to parse cron expression: {}", e);
                e
            })?;
        let mut tasks = self.tasks.lock().await;
        info!("Registering task '{}'", name);
        tasks.insert(name, CronTask { schedule, task });
        Ok(())
    }

    pub async fn start(&self) {
        let tasks = self.tasks.clone();
        let tasks = tasks.lock().await.clone();

        for (name, cron_task) in tasks.iter() {
            let name = name.clone();
            let schedule = cron_task.schedule.clone();
            let task = cron_task.clone();

            tokio::spawn(async move {
                let mut upcoming = schedule.upcoming(Utc);
                while let Some(next_time) = upcoming.next() {
                    let now = Utc::now();
                    if next_time > now {
                        let duration = (next_time - now).to_std()
                            .unwrap_or_else(|_| Duration::from_secs(0));
                        time::sleep(duration).await;
                    }
                    info!("Executing task: {}", name);
                    (task.task)();
                }
            });
        }
    }
}
