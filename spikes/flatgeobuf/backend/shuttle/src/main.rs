use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::Method,
    routing::get,
    Json, Router,
};
use geojson::GeoJson;
use shared::find::{find_remote, Bounds, Finder};
use shuttle_secrets::SecretStore;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct AppState {
    finder: Arc<Finder>,
    flatgeobuf_url: String,
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn layers(State(state): State<AppState>, bounds: Query<Bounds>) -> Json<GeoJson> {
    Json(find_remote(bounds.0, state.flatgeobuf_url).await.unwrap())
}

#[shuttle_runtime::main]
async fn main(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> shuttle_axum::ShuttleAxum {
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
