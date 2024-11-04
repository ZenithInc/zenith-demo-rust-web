use crate::routes::uv_lamp::register_uv_lamp_routes;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{extract::Request as ExtractRequest, Json as AxumJson};
use axum::{middleware, Extension, Router};
use chrono_tz::Tz;
use serde_json::json;
use std::sync::Arc;

pub fn init_routes(shared_timezone: Arc<Tz>) -> Router {
    Router::new()
        .merge(register_uv_lamp_routes())
        .layer(Extension(shared_timezone))
        .layer(middleware::from_fn(error_handler))
}

async fn error_handler(req: ExtractRequest, next: middleware::Next) -> Response {
    match req.extensions().get::<anyhow::Error>() {
        Some(err) => {
            let status_code = StatusCode::INTERNAL_SERVER_ERROR;
            let error_message = err.to_string();
            (
                status_code,
                AxumJson(json!({
                    "message": error_message,
                })),
            )
                .into_response()
        }
        None => next.run(req).await,
    }
}
