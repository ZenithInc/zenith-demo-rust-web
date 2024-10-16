use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use tokio::sync::OnceCell;
use std::env;
use anyhow::Error;

pub struct MySql {
    pub pool: MySqlPool,
}

impl MySql {
    pub async fn get_instance() -> Result<&'static MySql, Error> {
        static INSTANCE: OnceCell<MySql> = OnceCell::const_new();

        INSTANCE.get_or_try_init(|| async {
            let database_url = env::var("DATABASE_URL")?;
            let max_connections = env::var("DATABASE_MAX_CONNECTIONS").unwrap_or_else(|_| "15".to_string());
            let pool = MySqlPoolOptions::new()
                .max_connections(max_connections.parse()?)
                .connect(&database_url)
                .await?;
            Ok(MySql { pool })
        }).await
    }
}
