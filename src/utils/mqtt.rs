use crate::repositories::uv_lamp_mqtt_notify_job::UVLampMqttNotifyJob;
use crate::repositories::uv_lamp_mqtt_received_messages::UVLampMqttReceivedMessages;
use anyhow::anyhow;
use once_cell::sync::OnceCell;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, Publish, QoS};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

pub struct MqttHandler {
    sender: mpsc::Sender<(String, String)>,
}

impl MqttHandler {
    pub async fn send(&self, topic: &str, message: String) -> Result<(), anyhow::Error> {
        self.sender
            .send((topic.to_string(), message))
            .await
            .map_err(|e| anyhow!(e))
    }

    async fn new() -> Result<Self, anyhow::Error> {
        let client_id = "tuo_tu";
        let host = std::env::var("UV_LAMP_MQTT_HOST")?;
        let port: u16 = std::env::var("UV_LAMP_MQTT_PORT")
            .unwrap_or_else(|_| "8883".to_string())
            .parse()?;

        info!("MQTT connecting {}:{}...", host, port);

        let username = std::env::var("UV_LAMP_MQTT_USER")?;
        let password = std::env::var("UV_LAMP_MQTT_PASSWORD")?;

        let mut mqtt_options = MqttOptions::new(client_id, host, port);
        mqtt_options.set_credentials(username, password);
        mqtt_options.set_keep_alive(Duration::from_secs(5));

        let (client, mut event_loop) = AsyncClient::new(mqtt_options, 10);

        let (sender, mut receiver) = mpsc::channel(100);

        // Subscribe topic
        client
            .subscribe("87855294541367dab3e244c2441c5f22/+/oc/c", QoS::AtLeastOnce)
            .await?;
        client
            .subscribe("87855294541367dab3e244c2441c5f22/+/up/c", QoS::AtLeastOnce)
            .await?;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    event = event_loop.poll() => {
                        match event {
                            Ok(Event::Incoming(Incoming::Publish(publish))) => {
                                Self::handle_received_message(publish).await;
                            },
                            Ok(notif) => debug!("MQTT Event: {:?}", notif),
                            Err(e) => {
                                eprintln!("MQTT Event: {:?}", e);
                            }
                        }
                    },
                    Some((topic, message)) = receiver.recv() => {
                        match client.publish(topic, QoS::AtLeastOnce, false, message).await {
                            Ok(_) => info!("Topic message sent successfully!"),
                            Err(e) => error!("Topic message error: {}", e.to_string()),
                        }
                    }
                }
            }
        });

        Ok(MqttHandler { sender })
    }

    async fn handle_received_message(publish: Publish) {
        let topic = publish.topic;
        let payload = publish.payload.to_vec();
        match String::from_utf8(payload) {
            Ok(payload) => {
                if payload.contains("id") {
                    return;
                }
                info!("Received message: topic [{}], payload: {}", topic, payload);
                if let Some(device_number) = Self::get_device_number_from_topic(topic.as_str()) {
                    Self::save_received_message(&topic, &device_number, &payload).await;
                    Self::create_notify_job(device_number, payload).await;
                }
            }
            Err(e) => error!("An error occurred: {}", e),
        }
    }

    async fn save_received_message(topic: &String, device_number: &String, payload: &String) {
        let result = UVLampMqttReceivedMessages::create(
            topic.clone(),
            device_number.to_string(),
            payload.clone(),
        )
        .await;
        match result {
            Ok(id) => info!("Saved message, id {}", id),
            Err(e) => error!("An error occurred: {}", e),
        }
    }

    async fn create_notify_job(device_number: String, payload: String) {
        let result = UVLampMqttNotifyJob::create(device_number, payload).await;
        match result {
            Ok(id) => info!("Created notification job, id {}", id),
            Err(e) => error!("An error occurred: {}", e),
        }
    }

    fn get_device_number_from_topic(topic: &str) -> Option<String> {
        let parts: Vec<String> = topic.split("/").map(|s| s.to_string()).collect();
        if let Some(device_number) = parts.get(1) {
            Some(device_number.clone())
        } else {
            error!("Received message: not found device number!");
            None
        }
    }
}

static MQTT_HANDLER: OnceCell<Arc<MqttHandler>> = OnceCell::new();

pub async fn init_mqtt_handler() -> Result<(), anyhow::Error> {
    let handler = MqttHandler::new().await?;
    MQTT_HANDLER
        .set(Arc::new(handler))
        .map_err(|_| anyhow!("Failed to set mqtt handler"))?;
    Ok(())
}

pub fn instance() -> Option<Arc<MqttHandler>> {
    MQTT_HANDLER.get().cloned()
}
