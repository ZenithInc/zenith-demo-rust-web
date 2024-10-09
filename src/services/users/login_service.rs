use crate::params::requests::user::LoginParams;
use crate::utils::password::verify_password;
use crate::repositories::user::UserRepository;
use crate::utils::jwt::create_token;

pub struct LoginService;

impl LoginService {

    pub async fn login(params: LoginParams) -> Result<String, anyhow::Error> {
        let user = UserRepository::get_user_by_username(&params.username).await?;
        if !verify_password(&params.password, &user.password)? {
            return Err(anyhow::anyhow!("Invalid password"));
        }

        let secret_key = std::env::var("SECRET_KEY")?;
        let token = create_token(&user.id.to_string(), &secret_key)?;

        Ok(token)
    }
}

