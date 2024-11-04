use std::collections::HashMap;
use crate::repositories::uv_lamp_mqtt_notify_job::UVLampMqttNotifyJob;
use crate::repositories::uv_lamp_mqtt_received_messages::UVLampMqttReceivedMessages;
use crate::tasks::TaskType;
use anyhow::anyhow;
use once_cell::sync::OnceCell;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, Publish, QoS};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info};

pub struct MqttHandler {
    sender: mpsc::Sender<(String, String)>,
}

const SUBSCRIBE_TOPIC: [&str; 3] = [
    "87855294541367dab3e244c2441c5f22/+/oc/c",
    "87855294541367dab3e244c2441c5f22/+/up/c",
    "87855294541367dab3e244c2441c5f22/+/nI/c",
];

enum Message {
    LightSwitchResponse(String),
    LightNetworkResponse(String),
}

trait MessageHandle {
    async fn handle(&self, topic: &String, device_number: String, payload: String);
}

struct LightSwitchMessageHandler;

impl MessageHandle for LightSwitchMessageHandler {
    async fn handle(&self, topic: &String, device_number: String, payload: String) {
        let result = UVLampMqttNotifyJob::create(
            device_number,
            payload,
            TaskType::LightSwitchTask.to_string(),
        )
            .await;
        match result {
            Ok(id) => info!(
                "Created light switch notification job, id {}, topic {}",
                id, topic
            ),
            Err(e) => error!("An error occurred: {}", e),
        }
    }
}

struct LightNetworkMessageHandler;

impl MessageHandle for LightNetworkMessageHandler {
    async fn handle(&self, _topic: &String, device_number: String, payload: String) {
        // 更新在线状态
        let manager = get_device_manager();
        let mut manager = manager.lock().await;
        manager.update_status(&device_number, true);
        info!("The device {} is online!", device_number);

        // 创建任务
        let result = UVLampMqttNotifyJob::create(
            device_number,
            payload,
            TaskType::LightStatusTask.to_string(),
        )
            .await;
        match result {
            Ok(id) => info!("Created light network notification job, id {}", id),
            Err(e) => error!("An error occurred: {}", e),
        }
    }
}

impl MqttHandler {
    pub async fn send(&self, topic: &str, message: String) -> Result<(), anyhow::Error> {
        self.sender
            .send((topic.to_string(), message))
            .await
            .map_err(|e| anyhow!(e))
    }

    async fn new() -> Result<Self, anyhow::Error> {
        let client_id = "tuo_tu_client";
        let host = std::env::var("UV_LAMP_MQTT_HOST")?;
        let port: u16 = std::env::var("UV_LAMP_MQTT_PORT")
            .unwrap_or_else(|_| "8883".to_string())
            .parse()?;

        info!("MQTT connecting {}:{}...", host, port);

        let username = std::env::var("UV_LAMP_MQTT_USER")?;
        let password = std::env::var("UV_LAMP_MQTT_PASSWORD")?;

        let mut mqtt_options = MqttOptions::new(client_id, host, port);
        mqtt_options.set_credentials(username, password);
        mqtt_options.set_keep_alive(Duration::from_secs(60));

        let (client, mut event_loop) = AsyncClient::new(mqtt_options, 30);

        let (sender, mut receiver) = mpsc::channel(100);

        for topic in SUBSCRIBE_TOPIC {
            client.subscribe(topic, QoS::AtLeastOnce).await?;
        }

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

    fn parse_topic(topic: &String) -> Option<Message> {
        if topic.contains("up/c") {
            Some(Message::LightSwitchResponse(topic.to_string()))
        } else if topic.contains("nI/c") {
            Some(Message::LightNetworkResponse(topic.to_string()))
        } else {
            None
        }
    }

    async fn handle_received_message(publish: Publish) {
        let topic = publish.topic;
        let payload = publish.payload.to_vec();
        let payload = match String::from_utf8(payload) {
            Ok(payload) => payload,
            Err(_) => {
                error!("MQTT payload is not valid UTF-8");
                return;
            }
        };
        info!("Received message: topic [{}], payload: {}", topic, payload);
        let device_number = match get_device_number_from_topic(topic.as_str()) {
            Some(device_number) => device_number,
            None => return,
        };
        save_received_message(&topic, &device_number, &payload).await;
        if let Some(message) = Self::parse_topic(&topic) {
            match message {
                Message::LightSwitchResponse(message) if message.contains("up/c") => {
                    LightSwitchMessageHandler
                        .handle(&topic, device_number, payload)
                        .await;
                }
                Message::LightNetworkResponse(message) if message.contains("nI/c") => {
                    LightNetworkMessageHandler
                        .handle(&topic, device_number, payload)
                        .await;
                }
                _ => {}
            }
        }
    }
}

async fn save_received_message(topic: &String, device_number: &String, payload: &String) {
    let result =
        UVLampMqttReceivedMessages::create(topic, device_number.to_string(), payload).await;
    match result {
        Ok(id) => info!("Saved message, id {}", id),
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

pub struct DeviceInfo {
    is_online: bool,
    last_response_time: Option<u64>,
    last_query_time: Option<u64>,
}

pub struct DeviceManager {
    devices: HashMap<String, DeviceInfo>,
}

impl DeviceManager {
    fn new() -> Self {
        DeviceManager {
            devices: HashMap::new(),
        }
    }

    pub fn record_query_time(&mut self, device_number: &String) {
        let current_time = Self::get_current_time();
        if let Some(device_info) = self.devices.get_mut(device_number) {
            device_info.last_query_time = Some(current_time);
        } else {
            self.devices.insert(
                device_number.clone(),
                DeviceInfo {
                    is_online: false,
                    last_response_time: None,
                    last_query_time: Some(current_time),
                });
        }
    }

    pub fn update_status(&mut self, device_number: &String, is_online: bool) {
        let current_time = Self::get_current_time();
        if let Some(device_info) = self.devices.get_mut(device_number) {
            device_info.is_online = is_online;
            device_info.last_response_time = Some(current_time);
        } else {
            self.devices.insert(
                device_number.clone(),
                DeviceInfo {
                    is_online,
                    last_response_time: Some(current_time),
                    last_query_time: None,
                });
        }
    }

    pub fn find_all_offline_devices(&self) -> Vec<String> {
        let current_time = Self::get_current_time();
        self.devices.iter().filter_map(|(device_number, device_info)| {
            if device_info.last_response_time.is_none() || current_time - device_info.last_response_time.unwrap() > 60 {
                Some(device_number.clone())
            } else {
                None
            }
        }).collect()
    }

    fn get_current_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }
}

static DEVICE_MANAGER: OnceCell<Arc<Mutex<DeviceManager>>> = OnceCell::new();

pub fn get_device_manager() -> &'static Arc<Mutex<DeviceManager>> {
    DEVICE_MANAGER.get_or_init(|| Arc::new(Mutex::new(DeviceManager::new())))
}