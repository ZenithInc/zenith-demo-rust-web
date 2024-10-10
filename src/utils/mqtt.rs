use std::sync::Arc;
use std::time::Duration;
use anyhow::anyhow;
use once_cell::sync::OnceCell;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use tokio::sync::{mpsc};
use tracing::{debug, error, info};

pub struct MqttHandler {
    sender: mpsc::Sender<(String, String)>,
}

impl MqttHandler {
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
        client.subscribe("87855294541367dab3e244c2441c5f22/+/oc/c", QoS::AtLeastOnce).await?;
        client.subscribe("87855294541367dab3e244c2441c5f22/+/up/c", QoS::AtLeastOnce).await?;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    event = event_loop.poll() => {
                        match event {
                            Ok(Event::Incoming(Incoming::Publish(publish))) => {
                                let topic = publish.topic;
                                let payload = publish.payload.to_vec();
                                match String::from_utf8(payload) {
                                    Ok(payload) => info!("Received message: topic [{}], payload: {}", topic, payload),
                                    Err(_) => return
                                }
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

    pub async fn send(&self, topic: &str, message: String) -> Result<(), anyhow::Error> {
        self.sender.send((topic.to_string(), message)).await.map_err(|e| anyhow!(e))
    }
}

static MQTT_HANDLER: OnceCell<Arc<MqttHandler>> = OnceCell::new();

pub async fn init_mqtt_handler() -> Result<(), anyhow::Error> {
    let handler = MqttHandler::new().await?;
    MQTT_HANDLER.set(Arc::new(handler)).map_err(|_| anyhow!("Failed to set mqtt handler"))?;
    Ok(())
}

pub fn instance() -> Option<Arc<MqttHandler>> {
    MQTT_HANDLER.get().cloned()
}