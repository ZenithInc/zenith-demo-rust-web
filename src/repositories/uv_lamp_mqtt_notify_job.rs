use crate::utils::mysql::MySql;

pub struct UVLampMqttNotifyJob;

impl UVLampMqttNotifyJob {

    pub async fn create(message_id: String, uuid: String, notify_contents: String) -> Result<(), anyhow::Error> {
        let db = MySql::new().await?;
        let sql = "INSERT INTO `uv_lamp_mqtt_notify_jobs` (`message_id`, `uuid`, `notify_contents`) value (?, ?, ?)";
        sqlx::query(sql).bind(message_id).bind(uuid).bind(notify_contents).execute(&db.pool).await?;
        Ok(())
    }

}