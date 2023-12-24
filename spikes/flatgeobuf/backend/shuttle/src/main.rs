use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::Method,
    routing::get,
    Json, Router,
};
use geojson::GeoJson;
use opentelemetry::trace::{Tracer, TracerProvider as _};
use opentelemetry_sdk::trace::TracerProvider;
use shared::find::{find_remote, Bounds, Finder};
use shuttle_secrets::SecretStore;
use tower_http::cors::{Any, CorsLayer};
use tracing::instrument;
use tracing::{error, span};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

#[derive(Clone, Debug)]
struct AppState {
    finder: Arc<Finder>,
    flatgeobuf_url: String,
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[instrument]
async fn layers(State(state): State<AppState>, bounds: Query<Bounds>) -> Json<GeoJson> {
    Json(find_remote(bounds.0, state.flatgeobuf_url).await.unwrap())
}

#[shuttle_runtime::main]
async fn main(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    let log_level = "DEBUG";
    let fmt_layer = tracing_subscriber::fmt::layer();
    let filter_layer =
        tracing_subscriber::EnvFilter::try_new(log_level).expect("failed to set log level");
    let provider = TracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    let tracer = provider.tracer("shuttle");
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(telemetry_layer)
        .init();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        // allow requests from any origin
        .allow_origin(Any);

    let state = AppState {
        finder: Arc::new(Finder::new()),
        flatgeobuf_url: secret_store.get("FLATGEOBUF_URL").unwrap(),
    };

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/layers", get(layers))
        .layer(cors)
        .with_state(state);

    Ok(router.into())
}
