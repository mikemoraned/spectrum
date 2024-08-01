use axum::{extract::Query, Json};
use geo_types::polygon;
use geo_types::Geometry;
use geo_types::GeometryCollection;
use geojson::FeatureCollection;
use geojson::GeoJson;
use serde::Deserialize;
use std::iter::FromIterator;
use tracing::instrument;

#[derive(Deserialize, Debug)]
pub struct Bounds {
    sw_lat: f64,
    sw_lon: f64,
    ne_lat: f64,
    ne_lon: f64,
}

#[instrument()]
pub async fn regions(Query(bounds): Query<Bounds>) -> Json<GeoJson> {
    let bounds_as_poly: Geometry<f64> = polygon![
        (x: bounds.sw_lon, y: bounds.sw_lat),
        (x: bounds.ne_lon, y: bounds.sw_lat),
        (x: bounds.ne_lon, y: bounds.ne_lat),
        (x: bounds.sw_lon, y: bounds.ne_lat),
        (x: bounds.sw_lon, y: bounds.sw_lat),
    ]
    .into();

    let geometry_collection = GeometryCollection::from_iter(vec![bounds_as_poly]);
    let feature_collection = FeatureCollection::from(&geometry_collection);
    Json(GeoJson::FeatureCollection(feature_collection))
}
