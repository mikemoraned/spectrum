use axum::{http::Method, routing::get, Json, Router};
use geojson::GeoJson;
use service::find;
use tower_http::cors::{Any, CorsLayer};

async fn layers() -> Json<GeoJson> {
    Json(find::find().unwrap())
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        // allow requests from any origin
        .allow_origin(Any);

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/layers", get(layers))
        .layer(cors);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
