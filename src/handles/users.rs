use axum::extract::Json;
use crate::params::requests::user::LoginParams;
use validator::Validate;
use crate::services::users::login_service::LoginService;
use crate::utils::error::AppError;

pub async fn login(Json(params): Json<LoginParams>) -> Result<String, AppError> {
    if let Err(e) = params.validate() {
        return Err(AppError::new(format!("Validation failed: {:?}", e)));
    }

    LoginService::login(params.into()).await.map_err(|e| AppError::new(e.to_string()))
}
