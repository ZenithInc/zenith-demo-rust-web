use serde::Serialize;
use serde_json::json;
use axum::{
    response::{IntoResponse, Response},
    Json,
    http::StatusCode,
};

#[derive(Serialize)]
pub struct ApiResponse<T> {
    code: i32,
    message: String,
    data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        ApiResponse {
            code: 0,
            message: "Success".to_string(),
            data: Some(data),
        }
    }
}

impl<T> IntoResponse for ApiResponse<T> where T: Serialize,{
    fn into_response(self) -> Response {
        let body = Json(json!({
            "code": self.code,
            "message": self.message,
            "data": self.data,
        }));
        (StatusCode::OK, body).into_response()
    }
}
