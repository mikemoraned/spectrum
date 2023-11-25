use axum::{http::Method, routing::get, Json, Router};
use geojson::{Feature, GeoJson, Geometry, Value};
use service::find;
use tower_http::cors::{Any, CorsLayer};

// async fn layers() -> Json<GeoJson> {
//     let geometry = Geometry::new(Value::Polygon(vec![vec![
//         vec![-67.13734, 45.13745],
//         vec![-66.96466, 44.8097],
//         vec![-68.03252, 44.3252],
//         vec![-69.06, 43.98],
//         vec![-70.11617, 43.68405],
//         vec![-70.64573, 43.09008],
//         vec![-70.75102, 43.08003],
//         vec![-70.79761, 43.21973],
//         vec![-70.98176, 43.36789],
//         vec![-70.94416, 43.46633],
//         vec![-71.08482, 45.30524],
//         vec![-70.66002, 45.46022],
//         vec![-70.30495, 45.91479],
//         vec![-70.00014, 46.69317],
//         vec![-69.23708, 47.44777],
//         vec![-68.90478, 47.18479],
//         vec![-68.2343, 47.35462],
//         vec![-67.79035, 47.06624],
//         vec![-67.79141, 45.70258],
//         vec![-67.13734, 45.13745],
//     ]]));
//     let geojson = GeoJson::Feature(Feature {
//         bbox: None,
//         geometry: Some(geometry),
//         id: None,
//         properties: None,
//         foreign_members: None,
//     });
//     Json(geojson)
// }

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
