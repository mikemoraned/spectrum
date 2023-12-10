use axum::{http::Method, routing::get, Json, Router};
use geojson::GeoJson;
use shuttle_spike::find;
use tower_http::cors::{Any, CorsLayer};

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn layers() -> Json<GeoJson> {
    Json(find::find().unwrap())
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        // allow requests from any origin
        .allow_origin(Any);

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/layers", get(layers))
        .layer(cors);

    Ok(router.into())
}
