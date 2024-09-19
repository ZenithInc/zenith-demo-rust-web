use crate::params::requests::user::LoginParams;

pub struct LoginService;

impl LoginService {
    pub fn login(params: LoginParams) -> String {
        println!("Hello, {}!", params.username);
        params.username.clone()
    }
}

