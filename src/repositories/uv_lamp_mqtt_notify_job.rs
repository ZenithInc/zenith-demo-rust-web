use crate::utils::mysql::MySql;

pub struct UVLampMqttNotifyJob;

impl UVLampMqttNotifyJob {

    pub async fn create(device_number: String, notify_contents: String) -> Result<u64, anyhow::Error> {
        let db = MySql::new().await?;
        let sql = "INSERT INTO `uv_lamp_mqtt_notify_jobs` (`device_number`, `notify_contents`) value (?, ?)";
        let result = sqlx::query(sql).bind(device_number).bind(notify_contents).execute(&db.pool).await?;
        Ok(result.last_insert_id())
    }

}