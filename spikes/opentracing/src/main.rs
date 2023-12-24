use axum::{routing::get, Router};
use tracing::{debug, info};

async fn root() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Hello, world!");

    let app = Router::new().route("/", get(root));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:10000")
        .await
        .unwrap();
    debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
