use crate::utils::mysql::MySql;

pub struct UVLampMqttReceivedMessages;

impl UVLampMqttReceivedMessages {
    pub async fn create(
        topic: String,
        device_number: String,
        payload: String,
    ) -> Result<u64, anyhow::Error> {
        let db = MySql::new().await?;
        let sql = "INSERT INTO `uv_lamp_mqtt_received_messages` (`topic`, `device_number`, `payload`) VALUES (?, ?, ?);";
        let result = sqlx::query(sql)
            .bind(topic)
            .bind(device_number)
            .bind(payload)
            .execute(&db.pool)
            .await?;
        let last_insert_id = result.last_insert_id();
        Ok(last_insert_id)
    }
}
