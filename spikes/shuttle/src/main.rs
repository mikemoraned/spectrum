use axum::{extract::State, http::Method, routing::get, Json, Router};
use geojson::GeoJson;
use shuttle_spike::find::Finder;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct AppState {
    finder: Finder,
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn layers(State(state): State<AppState>) -> Json<GeoJson> {
    Json(state.finder.find().unwrap())
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        // allow requests from any origin
        .allow_origin(Any);

    let state = AppState {
        finder: Finder::new(),
    };

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/layers", get(layers))
        .layer(cors)
        .with_state(state);

    Ok(router.into())
}
