use axum::extract::Json;
use crate::params::requests::user::LoginParams;
use validator::Validate;
use crate::services::users::login_service::LoginService;
use crate::utils::error::AppError;
use crate::params::responses::common::ApiResponse;
use crate::params::responses::user::LoginSuccess;

pub async fn login(Json(params): Json<LoginParams>) -> Result<ApiResponse<LoginSuccess>, AppError> {
    if let Err(e) = params.validate() {
        return Err(AppError::new(format!("Validation failed: {:?}", e)));
    }

    let result = LoginService::login(params.into()).await.map_err(|e| AppError::new(e.to_string()));
    match result {
        Ok(token) => Ok(ApiResponse::new(LoginSuccess { token })),
        Err(e) => Err(e),
    }
}
