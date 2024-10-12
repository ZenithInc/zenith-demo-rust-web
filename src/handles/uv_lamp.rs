use axum::Json;
use tracing::info;
use validator::Validate;
use crate::params::requests::uv_lamp::TurnParams;
use crate::params::responses::common::{ApiResponse, Empty};
use crate::utils::error::AppError;
use crate::services::uv_lamp::control_service::ControlService;

pub async fn turn(Json(params): Json<TurnParams>) -> Result<ApiResponse<Empty>, AppError> {
    if let Err(e) = params.validate() {
        return Err(AppError::new(format!("Invalid ID parameters: {:?}", e)));
    }
    info!("Turn Light: {:?}", params);

    match ControlService::turn(params).await {
        Ok(_) => Ok(ApiResponse::new(Empty { })),
        Err(e) => Err(AppError::new(e.to_string())),
    }
}