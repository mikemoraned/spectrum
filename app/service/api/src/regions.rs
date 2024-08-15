use axum::extract::State;
use axum::{extract::Query, Json};
use core_geo::union::union;
use core_geo::Bounds;
use geo::geometry::{Geometry, GeometryCollection};
use geo::{BooleanOps, BoundingRect, LineString, MultiLineString, MultiPolygon, Polygon};
use geojson::feature::Id;
use geojson::FeatureCollection;
use geojson::GeoJson;
use rstar::RTree;
use std::iter::FromIterator;
use tracing::instrument;

use crate::flatgeobuf::FgbSource;
use crate::state::AppState;

#[derive(Default)]
pub struct Regions;

struct LabelledRoute {
    route: LineString<f64>,
    green: MultiLineString<f64>,
}

impl Regions {
    #[instrument(skip(self, fgb, bounds))]
    pub async fn regions(
        &self,
        fgb: &FgbSource,
        bounds: Bounds,
    ) -> Result<GeometryCollection<f64>, Box<dyn std::error::Error>> {
        let geoms = fgb.load(&bounds).await?;

        let unioned: Vec<Geometry<f64>> = union(geoms)?;

        Ok(GeometryCollection::from_iter(unioned))
    }

    #[instrument(skip(self, fgb, route))]
    pub async fn label_route(
        &self,
        fgb: &FgbSource,
        route: &LineString<f64>,
    ) -> Result<LabelledRoute, Box<dyn std::error::Error>> {
        let route_bounding_rect = route.bounding_rect().expect("some bounding rect");

        let regions = fgb.load(&route_bounding_rect.into()).await?;

        let possible = Regions::find_possibly_overlapping_regions(
            &regions,
            &route_bounding_rect.to_polygon(),
        )?;
        let overlaps = possible.clip(&MultiLineString::new(vec![route.clone()]), false);

        Ok(LabelledRoute {
            route: route.clone(),
            green: overlaps,
        })
    }

    #[instrument(skip(regions, route))]
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
}

#[instrument(skip(state))]
pub async fn regions(state: State<AppState>, Query(bounds): Query<Bounds>) -> Json<GeoJson> {
    let regions = state.regions.clone();
    let fgb = state.flatgeobuf.clone();
    let geometry_collection = regions.regions(&fgb, bounds).await.unwrap();
    Json(as_geojson(&geometry_collection))
}

#[instrument(skip(state))]
pub async fn route(
    state: State<AppState>,
    Query(bounds): Query<Bounds>,
) -> Json<serde_json::Value> {
    let regions = state.regions.clone();
    let fgb = state.flatgeobuf.clone();
    let routing = state.routing.clone();
    let route = routing.find_route(&bounds).await.unwrap();
    let labelled_route = regions.label_route(&fgb, &route).await.unwrap();
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
