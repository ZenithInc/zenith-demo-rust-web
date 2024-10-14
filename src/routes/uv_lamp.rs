use crate::handles::uv_lamp::turn;
use axum::routing::{post, Router};

pub fn register_uv_lamp_routes() -> Router {
    Router::new().route("/uv_lamp/turn", post(turn))
}
