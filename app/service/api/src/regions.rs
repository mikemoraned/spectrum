use axum::extract::State;
use axum::{extract::Query, Json};
use core_geo::union::union;
use flatgeobuf::geozero::ToGeo;
use flatgeobuf::{FallibleStreamingIterator, FgbReader};
use geo::geometry::{Geometry, GeometryCollection};
use geo::{BooleanOps, BoundingRect, LineString, MultiLineString, MultiPolygon, Polygon};
use geojson::feature::Id;
use geojson::FeatureCollection;
use geojson::GeoJson;
use rstar::RTree;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use tracing::{debug, instrument};

use crate::state::AppState;

#[derive(Deserialize, Debug)]
pub struct Bounds {
    pub(crate) sw_lat: f64,
    pub(crate) sw_lon: f64,
    pub(crate) ne_lat: f64,
    pub(crate) ne_lon: f64,
}
pub struct Regions {
    fgb_path: PathBuf,
}

impl Regions {
    pub fn from_flatgeobuf(fgb_path: &Path) -> Self {
        Regions {
            fgb_path: fgb_path.to_path_buf(),
        }
    }
}

struct LabelledRoute {
    route: LineString<f64>,
    green: MultiLineString<f64>,
}

impl Regions {
    #[instrument(skip(self, bounds))]
    pub async fn regions(
        &self,
        bounds: Bounds,
    ) -> Result<GeometryCollection<f64>, Box<dyn std::error::Error>> {
        let geoms = self.load_regions(&bounds).await?;

        let unioned: Vec<Geometry<f64>> = union(geoms)?;

        Ok(GeometryCollection::from_iter(unioned))
    }

    #[instrument(skip(self, route, bounds))]
    pub async fn label_route(
        &self,
        route: &LineString<f64>,
        bounds: &Bounds,
    ) -> Result<LabelledRoute, Box<dyn std::error::Error>> {
        let regions = self.load_regions(bounds).await?;

        let route_bounding_rect = route
            .bounding_rect()
            .expect("some bounding rect")
            .to_polygon();

        let possible = Regions::find_possibly_overlapping_regions(&regions, &route_bounding_rect)?;
        let overlaps = possible.clip(&MultiLineString::new(vec![route.clone()]), false);

        Ok(LabelledRoute {
            route: route.clone(),
            green: overlaps,
        })
    }

    fn find_possibly_overlapping_regions(
        regions: &[Geometry],
        route: &Polygon,
    ) -> Result<MultiPolygon, Box<dyn std::error::Error>> {
        let polygons = regions
            .iter()
            .filter_map(|g| match g {
                Geometry::Polygon(p) => Some(p.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        let route_rtree = RTree::bulk_load(vec![route.clone()]);
        let region_rtree = RTree::bulk_load(polygons);
        let mut overlap_candidates = vec![];
        for (poly, _) in region_rtree.intersection_candidates_with_other_tree(&route_rtree) {
            overlap_candidates.push(Geometry::Polygon(poly.clone()))
        }
        let unioned = union(overlap_candidates)?;

        let union_polygons = unioned
            .iter()
            .filter_map(|g| match g {
                Geometry::Polygon(p) => Some(p.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        Ok(MultiPolygon::new(union_polygons))
    }

    #[instrument(skip(self))]
    pub async fn load_regions(
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
pub async fn route(
    state: State<AppState>,
    Query(bounds): Query<Bounds>,
) -> Json<serde_json::Value> {
    let regions = state.regions.clone();
    let routing = state.routing.clone();
    let route = routing.find_route(&bounds).await.unwrap();
    let labelled_route = regions.label_route(&route, &bounds).await.unwrap();
    let route_geojson = as_geojson(&GeometryCollection::from(vec![Geometry::LineString(
        labelled_route.route,
    )]));
    let green_json = as_geojson(&GeometryCollection::from(vec![Geometry::MultiLineString(
        labelled_route.green,
    )]));

    let route_json = serde_json::to_value(route_geojson).unwrap();
    let green_json = serde_json::to_value(green_json).unwrap();

    let parts: serde_json::Map<String, serde_json::Value> = serde_json::Map::from_iter(vec![
        ("route".to_string(), route_json),
        ("green".to_string(), green_json),
    ]);
    let json = serde_json::Value::Object(parts);
    Json(json)
}

fn as_geojson(geometry_collection: &GeometryCollection<f64>) -> GeoJson {
    let mut feature_collection = FeatureCollection::from(geometry_collection);
    for (id, feature) in feature_collection.features.iter_mut().enumerate() {
        feature.id = Some(Id::Number(serde_json::Number::from(id)));
    }
    GeoJson::FeatureCollection(feature_collection)
}
