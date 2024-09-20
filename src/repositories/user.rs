use crate::utils::mysql::MySql;
use sqlx::mysql::MySqlRow;

pub struct UserRepository;

impl UserRepository {
    pub async fn get_user_by_username(username: &String) -> Result<MySqlRow, anyhow::Error> {
        let db = MySql::new().await.unwrap();
        let sql = format!("SELECT * FROM users WHERE username = '{}'", username); 
        sqlx::query(&sql).bind(&username).fetch_one(&db.pool).await
            .map_err(|_e| anyhow::anyhow!("Failed to fetch user"))
    }
}


