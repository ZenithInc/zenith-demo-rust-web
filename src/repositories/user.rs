use crate::utils::mysql::MySql;
use crate::models::user::User;

pub struct UserRepository;

impl UserRepository {
    pub async fn get_user_by_username(username: &String) -> Result<User, anyhow::Error> {
        let db = MySql::new().await.unwrap();
        let sql = format!("SELECT * FROM users WHERE username = '{}'", username); 
        sqlx::query_as::<_, User>(&sql).bind(&username).fetch_one(&db.pool).await
            .map_err(|_e| anyhow::anyhow!("Failed to fetch user"))
    }
}


