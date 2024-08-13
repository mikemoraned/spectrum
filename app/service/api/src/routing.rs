use std::time::SystemTime;

use axum::http::HeaderMap;
use core_geo::Bounds;
use ferrostar::{
    models::{GeographicCoordinate, UserLocation, Waypoint, WaypointKind},
    routing_adapters::{
        osrm::OsrmResponseParser, valhalla::ValhallaHttpRequestGenerator, RouteRequest,
        RouteRequestGenerator, RouteResponseParser,
    },
};
use geo::{coord, LineString};
use tracing::debug;
use url::Url;

pub struct StadiaMapsRouting {
    route_url: Url,
}

impl StadiaMapsRouting {
    pub fn new(api_key: &str, endpoint_base: &Url) -> Result<Self, Box<dyn std::error::Error>> {
        let mut route_url = endpoint_base.join("/route/v1")?;
        let authenticated_route_url = route_url
            .query_pairs_mut()
            .append_pair("api_key", api_key)
            .finish();
        Ok(StadiaMapsRouting {
            route_url: authenticated_route_url.clone(),
        })
    }

    pub async fn find_route(
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
        debug!("Converting {:?} routes", routes.len());

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
}
