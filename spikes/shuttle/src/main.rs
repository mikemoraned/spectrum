use axum::{http::Method, routing::get, Router};
use tower_http::cors::{Any, CorsLayer};

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        // allow requests from any origin
        .allow_origin(Any);

    let router = Router::new().route("/", get(hello_world)).layer(cors);

    Ok(router.into())
}
