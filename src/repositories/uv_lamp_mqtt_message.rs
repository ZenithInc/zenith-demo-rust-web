use crate::utils::mysql::MySql;

pub struct UVLampMqttMessage;

impl UVLampMqttMessage {

    pub async fn create(message_id: String, uuid: String, payload: String) -> Result<(), anyhow::Error> {
        let db = MySql::new().await?;
        let sql = "INSERT INTO `uv_lamp_mqtt_messages` (`message_id`, `uuid`, `payload`) value (?, ?, ?);" ;
        sqlx::query(sql).bind(message_id).bind(uuid).bind(payload).execute(&db.pool).await?;
        Ok(())
    }
}