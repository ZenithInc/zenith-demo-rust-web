use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;
use crate::params::requests::uv_lamp::TurnParams;
use crate::repositories::uv_lamp_mqtt_message::UVLampMqttMessage;
use crate::utils;

pub struct ControlService;

impl ControlService {

    pub async fn turn(params: TurnParams) -> Result<i32, anyhow::Error> {
        let topic = Self::get_topic(&params.device_number)?;
        info!("Topic is {}", topic);

        let message = json!({
            "id": params.message_id,
            "s": if params.status { 1 } else { 0 },
            // 我也不知道 d 什么意思，有什么作用，如果开灯没有这个参数，数据包解析错误
            "d": "",
        }).to_string();

        if let Some(mqtt_handler) = utils::mqtt::instance() {
            mqtt_handler.send(topic.as_str(), message.clone()).await?;
            UVLampMqttMessage::create(params.message_id.to_string(), Uuid::new_v4().to_string(), message).await?;
        } else {
            error!("MQTT Handler not initialized!")
        }

        Ok(params.message_id)
    }

    fn get_topic(device_number: &str) -> Result<String, anyhow::Error> {
        let topic = format!("87855294541367dab3e244c2441c5f22/{}/oc/s", device_number);
        Ok(topic)
    }

}