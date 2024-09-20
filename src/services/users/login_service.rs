use crate::params::requests::user::LoginParams;
use crate::utils::password::verify_password;
use crate::repositories::user::UserRepository;
use crate::utils::jwt::create_token;

pub struct LoginService;

impl LoginService {

    pub async fn login(params: LoginParams) -> Result<String, anyhow::Error> {
        dotenv::dotenv().ok();

        let user = UserRepository::get_user_by_username(&params.username).await?;
        if !verify_password(&params.password, &user.password)? {
            return Err(anyhow::anyhow!("Invalid password"));
        }

        let sercet_key = std::env::var("SECRET_KEY").unwrap();
        let token = create_token(&user.id.to_string(), &sercet_key)?;

        Ok(token)
    }
}

