use std::sync::Arc;

use axum::{extract::State, http::Method, routing::get, Json, Router};
use geojson::GeoJson;
use service::find::Finder;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct AppState {
    finder: Arc<Finder>,
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn layers(State(state): State<AppState>) -> Json<GeoJson> {
    Json(state.finder.find_flatgeobuf().unwrap())
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        // allow requests from any origin
        .allow_origin(Any);

    let state = AppState {
        finder: Arc::new(Finder::new()),
    };

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/layers", get(layers))
        .layer(cors)
        .with_state(state);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}
