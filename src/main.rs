mod handles;
mod repositories;
mod routes;
mod params;
mod services;
mod utils;

use axum::{
    Router,
    Json as AxumJson,
    middleware,
    http::StatusCode,
    extract::Request as ExtractRequest,
    response::Response,
};
use axum::response::IntoResponse;
use serde_json::json;

use routes::users::register_user_routes;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .merge(register_user_routes())
        .layer(middleware::from_fn(error_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
           ).into_response()
       }
       None => next.run(req).await,
   }
}
