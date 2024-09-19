mod handles;
mod routes;
mod params;
mod services;
mod utils;

use axum::Router;

use routes::users::register_user_routes;


#[tokio::main]
async fn main() {
    let app = Router::new().merge(register_user_routes());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

