use axum::extract::State;
use axum::{extract::Query, Json};
use geo_types::polygon;
use geo_types::Geometry;
use geo_types::GeometryCollection;
use geojson::FeatureCollection;
use geojson::GeoJson;
use serde::Deserialize;
use std::iter::FromIterator;
use tracing::instrument;

use crate::state::AppState;

#[derive(Deserialize, Debug)]
pub struct Bounds {
    sw_lat: f64,
    sw_lon: f64,
    ne_lat: f64,
    ne_lon: f64,
}

pub struct Regions {}

impl Regions {
    #[instrument(skip(self))]
    pub fn regions(&self, bounds: Bounds) -> GeometryCollection {
        let bounds_as_poly: Geometry<f64> = polygon![
            (x: bounds.sw_lon, y: bounds.sw_lat),
            (x: bounds.ne_lon, y: bounds.sw_lat),
            (x: bounds.ne_lon, y: bounds.ne_lat),
            (x: bounds.sw_lon, y: bounds.ne_lat),
            (x: bounds.sw_lon, y: bounds.sw_lat),
        ]
        .into();

        GeometryCollection::from_iter(vec![bounds_as_poly])
    }
}

#[instrument(skip(state))]
pub async fn regions(state: State<AppState>, Query(bounds): Query<Bounds>) -> Json<GeoJson> {
    let regions = state.regions.clone();
    let geometry_collection = regions.regions(bounds);
    let feature_collection = FeatureCollection::from(&geometry_collection);
    Json(GeoJson::FeatureCollection(feature_collection))
}
