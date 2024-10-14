use crate::utils::mysql::MySql;

pub struct UVLampMqttMessage;

impl UVLampMqttMessage {
    pub async fn create(
        message_id: String,
        device_number: String,
        payload: String,
    ) -> Result<(), anyhow::Error> {
        let db = MySql::new().await?;
        let sql = "INSERT INTO `uv_lamp_mqtt_messages` (`message_id`, `device_number`, `payload`) value (?, ?, ?);" ;
        sqlx::query(sql)
            .bind(message_id)
            .bind(device_number)
            .bind(payload)
            .execute(&db.pool)
            .await?;
        Ok(())
    }
}
