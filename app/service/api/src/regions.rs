use axum::extract::State;
use axum::{extract::Query, Json};
use core_geo::union::union;
use flatgeobuf::geozero::ToGeo;
use flatgeobuf::{FallibleStreamingIterator, FgbReader};
use geo::geometry::{Geometry, GeometryCollection};
use geo::{coord, Rect};
use geojson::feature::Id;
use geojson::FeatureCollection;
use geojson::GeoJson;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use tracing::{debug, instrument};

use crate::state::AppState;

#[derive(Deserialize, Debug)]
pub struct Bounds {
    sw_lat: f64,
    sw_lon: f64,
    ne_lat: f64,
    ne_lon: f64,
}
pub struct Regions {
    fgb_path: PathBuf,
}

impl Regions {
    pub fn from_flatgeobuf(fgb_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Regions {
            fgb_path: fgb_path.to_path_buf(),
        })
    }
}

impl Regions {
    #[instrument(skip(self))]
    pub async fn regions(
        &self,
        bounds: Bounds,
    ) -> Result<GeometryCollection<f64>, Box<dyn std::error::Error>> {
        let geoms = self.load_area(&bounds).await?;

        let unioned: Vec<Geometry<f64>> = union(geoms)?;

        Ok(GeometryCollection::from_iter(unioned))
    }

    #[instrument(skip(self))]
    pub async fn overlaps(
        &self,
        bounds: Bounds,
    ) -> Result<GeometryCollection<f64>, Box<dyn std::error::Error>> {
        let mut geoms = self.load_area(&bounds).await?;

        let bounds_width = (bounds.ne_lon - bounds.sw_lon).abs();
        let bounds_height = (bounds.ne_lat - bounds.sw_lat).abs();
        let route_rect = Rect::new(
            coord! {
            x: bounds.sw_lon + (bounds_width / 5.0),
            y: bounds.ne_lat - (bounds_height / 2.0) + (0.02 * bounds_height / 2.0) },
            coord! {
            x: bounds.ne_lon - (bounds_width / 5.0),
            y: bounds.sw_lat + (bounds_height / 2.0) - (0.02 * bounds_height / 2.0)},
        );

        geoms.push(Geometry::Rect(route_rect));

        Ok(GeometryCollection::from_iter(geoms))
    }

    #[instrument(skip(self))]
    pub async fn load_area(
        &self,
        bounds: &Bounds,
    ) -> Result<Vec<Geometry<f64>>, Box<dyn std::error::Error>> {
        let filein = BufReader::new(File::open(self.fgb_path.clone())?);
        let reader = FgbReader::open(filein)?;
        debug!("Opened FlatGeobuf file: {:?}", self.fgb_path);

        let mut features =
            reader.select_bbox(bounds.sw_lon, bounds.sw_lat, bounds.ne_lon, bounds.ne_lat)?;

        let mut geoms: Vec<Geometry<f64>> = vec![];
        while let Some(feature) = features.next()? {
            let geom: Geometry<f64> = feature.to_geo()?;
            geoms.push(geom);
        }

        Ok(geoms)
    }
}

#[instrument(skip(state))]
pub async fn regions(state: State<AppState>, Query(bounds): Query<Bounds>) -> Json<GeoJson> {
    let regions = state.regions.clone();
    let geometry_collection = regions.regions(bounds).await.unwrap();
    Json(as_geojson(&geometry_collection))
}

#[instrument(skip(state))]
pub async fn overlaps(state: State<AppState>, Query(bounds): Query<Bounds>) -> Json<GeoJson> {
    let regions = state.regions.clone();
    let geometry_collection = regions.overlaps(bounds).await.unwrap();
    Json(as_geojson(&geometry_collection))
}

fn as_geojson(geometry_collection: &GeometryCollection<f64>) -> GeoJson {
    let mut feature_collection = FeatureCollection::from(geometry_collection);
    for (id, feature) in feature_collection.features.iter_mut().enumerate() {
        feature.id = Some(Id::Number(serde_json::Number::from(id)));
    }
    GeoJson::FeatureCollection(feature_collection)
}
