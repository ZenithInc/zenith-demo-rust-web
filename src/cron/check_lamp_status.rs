use crate::cron::DEVICE_NUMBERS;
use crate::utils;
use futures::future::BoxFuture;
use futures::FutureExt;
use rand::Rng;
use rand_core::SeedableRng;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::error;
use crate::utils::mqtt::get_device_manager;

pub fn handle() -> BoxFuture<'static, ()> {
    async move {
        let index = get_device_number();
        let device_number = DEVICE_NUMBERS[index];
        let topic = get_topic(&device_number);

        let mut rng = rand::rngs::StdRng::from_entropy();
        let random_number: u32 = rng.gen_range(100_000..1_000_000);
        let message = json!({
            "id": random_number.to_string(),
        })
        .to_string();
        if let Some(mqtt_handler) = utils::mqtt::instance() {
            mqtt_handler
                .send(topic.as_str(), message.clone())
                .await
                .unwrap();

            // 在内存中维护设备的在线状态
            let manager = get_device_manager();
            let mut manager = manager.lock().await;
            manager.record_query_time(&device_number.to_string());
        } else {
            error!("MQTT Handler not initialized");
        }
    }
    .boxed()
}

fn get_device_number() -> usize {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let timestamp = since_the_epoch.as_secs();

    (timestamp % DEVICE_NUMBERS.len() as u64) as usize
}

fn get_topic(device_number: &str) -> String {
    let topic = format!("87855294541367dab3e244c2441c5f22/{}/nI/s", device_number);
    topic
}
