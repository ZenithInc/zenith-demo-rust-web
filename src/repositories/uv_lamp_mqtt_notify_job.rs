use crate::utils::mysql::MySql;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use tracing::debug;

pub struct UVLampMqttNotifyJob;

#[derive(FromRow)]
pub struct Job {
    pub id: u64,
    pub device_number: String,
    pub notify_contents: String,
    pub is_completed: u8,
    pub retry_count: u8,
    pub next_retry_time: u64,
    pub deleted_at: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum IsCompleted {
    Incomplete = 0,
    Complete = 1,
}

impl IsCompleted {
    fn as_i32(&self) -> i32 {
        match self {
            IsCompleted::Incomplete => 0,
            IsCompleted::Complete => 1,
        }
    }
}

impl UVLampMqttNotifyJob {
    pub async fn create(
        device_number: String,
        notify_contents: String,
    ) -> Result<u64, anyhow::Error> {
        let db = MySql::new().await?;
        let sql = "INSERT INTO `uv_lamp_mqtt_notify_jobs` (`device_number`, `notify_contents`) value (?, ?)";
        let result = sqlx::query(sql)
            .bind(device_number)
            .bind(notify_contents)
            .execute(&db.pool)
            .await?;
        Ok(result.last_insert_id())
    }

    pub async fn get_incomplete_jobs(max_retry_count: u8) -> Result<Vec<Job>, anyhow::Error> {
        let db = MySql::new().await?;
        let current_time = Utc::now().timestamp() as u64;

        let sql = "SELECT * from `uv_lamp_mqtt_notify_jobs` where `retry_count` <= ? and `is_completed` = ? and `next_retry_time` <= ? limit 10;";

        let jobs = sqlx::query_as::<_, Job>(sql)
            .bind(max_retry_count)
            .bind(IsCompleted::Incomplete.as_i32())
            .bind(current_time)
            .fetch_all(&db.pool)
            .await?;
        Ok(jobs)
    }

    pub async fn update_retry_count(
        id: u64,
        updated_value: u8,
        before_value: u8,
        next_try_time: u64,
    ) -> Result<(), anyhow::Error> {
        let db = MySql::new().await?;

        let sql = "UPDATE `uv_lamp_mqtt_notify_jobs` SET `retry_count` = ?, `next_retry_time` = ? where `id` = ? and `retry_count` = ?;";
        debug!(
            "sql of update retry_count field: {}, binds: {}, {}, {}, {}",
            sql, updated_value, next_try_time, id, before_value
        );

        let result = sqlx::query(sql)
            .bind(updated_value)
            .bind(next_try_time)
            .bind(id)
            .bind(before_value)
            .execute(&db.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!(
                "Update failed: retry_count does not match the expected value."
            ));
        }
        Ok(())
    }
}
