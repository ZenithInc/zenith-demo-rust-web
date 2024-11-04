use futures::future::BoxFuture;
use futures::FutureExt;
use tracing::{error, info};
use crate::repositories::uv_lamp_mqtt_notify_job::UVLampMqttNotifyJob;
use crate::tasks::TaskType;
use crate::utils::mqtt::get_device_manager;

pub fn handle() -> BoxFuture<'static, ()> {
    async move {
        let manager = get_device_manager();
        let mut manager = manager.lock().await;

        let offline_devices = manager.find_all_offline_devices();
        for device_number in offline_devices {
            info!("The device {} is offline!", device_number);
            manager.update_status(&device_number, false);
            create_job(device_number).await;
        }
    }.boxed()
}

async fn create_job(device_number: String) {
    let result = UVLampMqttNotifyJob::create(device_number, "".to_string(), TaskType::LightStatusTask.to_string()).await;
    match result {
        Ok(id) => info!("Created light switch notification job, id {}", id),
        Err(e) => error!("An error occurred: {}", e),
    }
}