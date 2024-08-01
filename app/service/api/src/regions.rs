use axum::{extract::Query, Json};
use geojson::{FeatureCollection, GeoJson};
use serde::Deserialize;
use tracing::instrument;

#[derive(Deserialize, Debug)]
pub struct Bounds {
    sw_lat: f64,
    sw_lon: f64,
    ne_lat: f64,
    ne_lon: f64,
}

#[instrument()]
pub async fn regions(bounds: Query<Bounds>) -> Json<GeoJson> {
    Json(GeoJson::FeatureCollection(FeatureCollection {
        bbox: None,
        features: vec![],
        foreign_members: None,
    }))
}
