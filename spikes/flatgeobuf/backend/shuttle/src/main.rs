use std::str::FromStr;

use axum::{
    extract::{Query, State},
    http::Method,
    routing::get,
    Json, Router,
};
use geojson::GeoJson;
use opentelemetry_sdk::{trace::Config, Resource};
use shared::find::{find_remote, Bounds};
use shuttle_secrets::SecretStore;
use tower_http::cors::{Any, CorsLayer};
use tracing::instrument;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

#[derive(Clone, Debug)]
struct AppState {
    flatgeobuf_url: String,
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[instrument(skip(state, bounds))]
async fn layers(State(state): State<AppState>, bounds: Query<Bounds>) -> Json<GeoJson> {
    Json(find_remote(bounds.0, state.flatgeobuf_url).await.unwrap())
}

fn setup_tracing_and_logging(service_name: &str, fmt_filter: EnvFilter) {
    use opentelemetry_semantic_conventions as semconv;

    let otlp_exporter = opentelemetry_otlp::new_exporter().tonic();
    let resource = Resource::new(vec![
        semconv::resource::SERVICE_NAME.string(service_name.to_string())
    ]);
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(Config::default().with_resource(resource))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap();

    let opentelemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let fmt_layer = fmt::layer().with_filter(fmt_filter);
    tracing_subscriber::registry()
        .with(opentelemetry_layer)
        .with(fmt_layer)
        .try_init()
        .unwrap();
}

#[shuttle_runtime::main]
async fn main(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
    let log_level = secret_store.get("RUST_LOG").unwrap();
    setup_tracing_and_logging("shuttle", EnvFilter::from_str(&log_level).unwrap());

    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        // allow requests from any origin
        .allow_origin(Any);

    let state = AppState {
        flatgeobuf_url: secret_store.get("FLATGEOBUF_URL").unwrap(),
    };

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/layers", get(layers))
        .layer(cors)
        .with_state(state);

    Ok(router.into())
}
