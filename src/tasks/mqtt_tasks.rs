use std::sync::Arc;
use tokio::sync::Notify;
use tracing::info;

pub async fn notify(notify: Arc<Notify>) {
   loop {
       info!("MQTT notify task start running...");
       tokio::select! {
           _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {
               info!("MQTT notify todo....")
           },
           _ = notify.notified() => {
               info!("MQTT notify received stop signal!");
               break;
           }
       }
       info!("MQTT notify task stopped!");
   }
}