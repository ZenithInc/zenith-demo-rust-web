use axum::routing::{post, Router};
use crate::handles::uv_lamp::turn;

pub fn register_uv_lamp_routes() -> Router {
    Router::new().route("/uv_lamp/turn", post(turn))
}