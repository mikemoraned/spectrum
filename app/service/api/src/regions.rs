use axum::extract::State;
use axum::{extract::Query, Json};
use flatgeobuf::geozero::ToGeo;
use flatgeobuf::{FallibleStreamingIterator, FgbReader};
use geo_types::Geometry;
use geo_types::GeometryCollection;
use geojson::FeatureCollection;
use geojson::GeoJson;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::iter::FromIterator;
use std::path::PathBuf;
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
    pub fn from_flatgeobuf(fgb_path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Regions {
            fgb_path: fgb_path.clone(),
        })
    }
}

impl Regions {
    #[instrument(skip(self))]
    pub async fn regions(
        &self,
        bounds: Bounds,
    ) -> Result<GeometryCollection<f64>, Box<dyn std::error::Error>> {
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

        Ok(GeometryCollection::from_iter(geoms))
    }
}

#[instrument(skip(state))]
pub async fn regions(state: State<AppState>, Query(bounds): Query<Bounds>) -> Json<GeoJson> {
    let regions = state.regions.clone();
    let geometry_collection = regions.regions(bounds).await.unwrap();
    let feature_collection = FeatureCollection::from(&geometry_collection);
    Json(GeoJson::FeatureCollection(feature_collection))
}
