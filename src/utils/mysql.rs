use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

pub struct MySql {
    pub pool: MySqlPool,
}


impl MySql {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let database_url = std::env::var("DATABASE_URL")?;
        let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "15".to_string());
        let pool = MySqlPoolOptions::new()
            .max_connections(max_connections.parse()?)
            .connect(&database_url)
            .await?;
        Ok(Self { pool })
    }

}

