use axum::{routing::get, Router};
use opentelemetry::global;
use tracing::{debug, info, instrument, trace};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[instrument]
async fn single_level() -> &'static str {
    trace!("single_level");
    "Hello, World!"
}

#[instrument]
async fn multi_level() -> String {
    trace!("multi_level");
    format!("the answer is {}", some_number().await)
}

#[instrument]
async fn some_number() -> u8 {
    trace!("some_number");
    42
}

#[tokio::main]
async fn main() {
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("opentelemetry-example")
        .install_simple()
        .unwrap();
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::registry()
        .with(opentelemetry)
        .with(fmt::Layer::default())
        .try_init()
        .unwrap();

    info!("Hello, world!");

    let app = Router::new()
        .route("/", get(single_level))
        .route("/multi", get(multi_level));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:10000")
        .await
        .unwrap();
    debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
