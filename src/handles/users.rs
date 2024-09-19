use axum::extract::Json;
use crate::params::requests::user::LoginParams;
use validator::Validate;
use crate::services::users::login_service::LoginService;

pub async fn login(Json(params): Json<LoginParams>) -> String {
    if let Err(e) = params.validate() {
        return format!("Validation failed: {:?}", e);
    }

    LoginService::login(params.into());
    String::from("Hello")
}
