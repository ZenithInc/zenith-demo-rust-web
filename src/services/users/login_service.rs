use crate::params::requests::user::LoginParams;
use crate::utils::mysql::MySql;

pub struct LoginService;

impl LoginService {

    pub async fn login(params: LoginParams) -> Result<String, anyhow::Error> {
        println!("Hello, {}!", params.username);
        let db = MySql::new().await.unwrap();
        let sql = format!("SELECT * FROM users WHERE username = '{}'", params.username); 
        let result = sqlx::query(&sql).bind(&params.username).fetch_one(&db.pool).await
            .map_err(|_e| anyhow::anyhow!("Failed to fetch user"))?;

        println!("{:?}", result);
        Ok(params.username)
    }
}

