use sqlx::MySqlPool;

pub struct MySql {
    pub pool: MySqlPool,
}


impl MySql {
    pub async fn new() -> Result<Self, sqlx::Error> {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").unwrap();
        let pool = MySqlPool::connect(&database_url).await.unwrap();
        Ok(Self { pool })
    }

}

