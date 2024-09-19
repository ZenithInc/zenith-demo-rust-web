use axum::{
    routing::post,
    Router,
};

use crate::handles::users::login;

pub fn register_user_routes() -> Router {
    Router::new().route("/login", post(login))
}
