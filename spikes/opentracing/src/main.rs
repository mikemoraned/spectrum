use axum::{routing::get, Router};
use opentelemetry::KeyValue;
use opentelemetry_sdk::trace::Config;
use opentelemetry_sdk::Resource;
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

fn setup_tracing_and_logging(service_name: &str) {
    let otlp_exporter = opentelemetry_otlp::new_exporter().tonic();
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(
            Config::default()
                .with_resource(Resource::new(vec![KeyValue::new("service.name", "foop")])),
        )
        .install_simple()
        .unwrap();

    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::registry()
        .with(opentelemetry)
        .with(fmt::Layer::default())
        .try_init()
        .unwrap();
}

#[tokio::main]
async fn main() {
    setup_tracing_and_logging("opentelemetry-example");

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
