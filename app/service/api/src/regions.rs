use axum::extract::State;
use axum::http::HeaderMap;
use axum::{extract::Query, Json};
use core_geo::buffer::{buffer_multi_polygon, buffer_polygon};
use core_geo::union::union;
use ferrostar::models::{GeographicCoordinate, UserLocation, Waypoint, WaypointKind};
use ferrostar::routing_adapters::osrm::OsrmResponseParser;
use ferrostar::routing_adapters::valhalla::ValhallaHttpRequestGenerator;
use ferrostar::routing_adapters::{RouteRequest, RouteRequestGenerator, RouteResponseParser};
use flatgeobuf::geozero::ToGeo;
use flatgeobuf::{FallibleStreamingIterator, FgbReader};
use geo::geometry::{Geometry, GeometryCollection};
use geo::{coord, Area, BoundingRect, LineString, MultiPolygon, Polygon};
use geo_validity_check::Valid;
use geojson::feature::Id;
use geojson::FeatureCollection;
use geojson::GeoJson;
use rstar::RTree;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, instrument};
use url::Url;

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
    route_url: Url,
}

impl Regions {
    pub fn from_flatgeobuf(
        fgb_path: &Path,
        stadia_maps_api_key: &str,
        stadia_maps_endpoint_base: &Url,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut route_url = stadia_maps_endpoint_base.join("/route/v1")?;
        let authenticated_route_url = route_url
            .query_pairs_mut()
            .append_pair("api_key", stadia_maps_api_key)
            .finish();
        Ok(Regions {
            fgb_path: fgb_path.to_path_buf(),
            route_url: authenticated_route_url.clone(),
        })
    }
}

impl Regions {
    #[instrument(skip(self))]
    pub async fn regions(
        &self,
        bounds: Bounds,
    ) -> Result<GeometryCollection<f64>, Box<dyn std::error::Error>> {
        let geoms = self.load_regions(&bounds).await?;

        let unioned: Vec<Geometry<f64>> = union(geoms)?;

        Ok(GeometryCollection::from_iter(unioned))
    }

    #[instrument(skip(self))]
    pub async fn overlaps(
        &self,
        bounds: Bounds,
    ) -> Result<GeometryCollection<f64>, Box<dyn std::error::Error>> {
        let regions = self.load_regions(&bounds).await?;

        let stadiamaps_route = self.find_stadiamaps_route(&bounds).await?;
        let route_bounding_rect = stadiamaps_route
            .bounding_rect()
            .expect("some bounding rect")
            .to_polygon();

        let possible = Regions::find_possibly_overlapping_regions(&regions, &route_bounding_rect)?;
        let possible_example = select_largest_polygon(&possible)?;
        let buffer_distance = 0.001;
        if possible_example.is_valid() {
            debug!("example is valid");
        } else {
            debug!(
                "example is invalid {:?}",
                possible_example.explain_invalidity()
            );
        }
        // let buffered = buffer_multi_polygon(&possible, buffer_distance);
        let buffered = buffer_polygon(&possible_example.clone(), buffer_distance);

        let mut overlaps = vec![];

        overlaps.push(Geometry::LineString(stadiamaps_route.clone()));
        overlaps.push(Geometry::Polygon(route_bounding_rect.clone()));
        // overlaps.push(Geometry::MultiPolygon(possible.clone()));
        overlaps.push(Geometry::Polygon(possible_example.clone()));
        overlaps.push(Geometry::MultiPolygon(buffered.clone()));

        Ok(GeometryCollection::from_iter(overlaps))
    }

    async fn find_stadiamaps_route(
        &self,
        bounds: &Bounds,
    ) -> Result<LineString, Box<dyn std::error::Error>> {
        let bounds_width = (bounds.ne_lon - bounds.sw_lon).abs();
        let bounds_height = (bounds.ne_lat - bounds.sw_lat).abs();
        let corner1 = coord! {
        x: bounds.sw_lon + (bounds_width / 5.0),
        y: bounds.ne_lat - (bounds_height / 2.0) + (0.02 * bounds_height / 2.0) };
        let corner2 = coord! {
        x: bounds.ne_lon - (bounds_width / 5.0),
        y: bounds.sw_lat + (bounds_height / 2.0) - (0.02 * bounds_height / 2.0)};

        let generator = ValhallaHttpRequestGenerator::new(
            self.route_url.to_string().clone(),
            "pedestrian".into(),
            None,
        );

        let user_location = UserLocation {
            coordinates: GeographicCoordinate {
                lat: corner1.y,
                lng: corner1.x,
            },
            horizontal_accuracy: 1.0,
            course_over_ground: None,
            timestamp: SystemTime::now(),
            speed: None,
        };
        let waypoints: Vec<Waypoint> = vec![Waypoint {
            coordinate: GeographicCoordinate {
                lat: corner2.y,
                lng: corner2.x,
            },
            kind: WaypointKind::Break,
        }];

        let RouteRequest::HttpPost { url, body, headers } =
            generator.generate_request(user_location, waypoints)?;

        debug!("Route body: {:?}", String::from_utf8(body.clone()));

        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .body(body)
            .headers(HeaderMap::try_from(&headers)?)
            .send()
            .await?;

        debug!("Route response: {:?}", response);

        let content = response.bytes().await?;
        let routes = OsrmResponseParser::new(6).parse_response(content.to_vec())?;

        debug!("Parsed routes: {:?}", routes);

        let route = routes.first().unwrap();
        let route_line = LineString::new(
            route
                .geometry
                .iter()
                .map(|c| coord!(x: c.lng, y: c.lat))
                .collect(),
        );

        Ok(route_line)
    }

    fn find_possibly_overlapping_regions(
        regions: &Vec<Geometry>,
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

fn select_largest_polygon(multi: &MultiPolygon) -> Result<Polygon, Box<dyn std::error::Error>> {
    let mut largest = None;
    let mut largest_area = 0.0;
    for p in multi.iter() {
        let area = p.signed_area();
        if area > largest_area {
            largest = Some(p.clone());
            largest_area = area;
        }
    }
    largest.ok_or("No smallest polygon found".into())
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
