use crate::params::requests::user::LoginParams;
use crate::utils::password::verify_password;
use crate::repositories::user::UserRepository;
use sqlx::Row;

pub struct LoginService;

impl LoginService {

    pub async fn login(params: LoginParams) -> Result<String, anyhow::Error> {
        let user = UserRepository::get_user_by_username(&params.username).await?;
        if !verify_password(&params.password, &user.try_get::<String, &str>("password")?)? {
            return Err(anyhow::anyhow!("Invalid password"));
        }

        Ok("Logged in".to_string())
    }
}

