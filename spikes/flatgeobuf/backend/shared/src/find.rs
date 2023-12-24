use std::io::Cursor;

use geojson::GeoJson;
use geozero::geojson::GeoJsonWriter;
use serde::Deserialize;

use crate::geo_assets::GeoAssets;
use tracing::{debug, instrument};

#[derive(Deserialize, Debug)]
pub struct Bounds {
    sw_lat: f64,
    sw_lon: f64,
    ne_lat: f64,
    ne_lon: f64,
}

#[derive(Debug)]
pub struct Finder {}

impl Finder {
    pub fn new() -> Finder {
        Finder {}
    }

    pub fn find(&self, bounds: Bounds) -> Result<GeoJson, ()> {
        use flatgeobuf::*;
        use geozero::ProcessToJson;

        println!("bounds: {:?}", bounds);

        match GeoAssets::get("all.fgb") {
            Some(f) => {
                let reader = Cursor::new(f.data);
                match FgbReader::open(reader) {
                    Ok(fgb) => {
                        match fgb.select_bbox(
                            bounds.sw_lon,
                            bounds.sw_lat,
                            bounds.ne_lon,
                            bounds.ne_lat,
                        ) {
                            Ok(mut fgb) => match fgb.to_json().unwrap().parse::<GeoJson>() {
                                Ok(geojson) => Ok(geojson),
                                Err(_) => Err(()),
                            },
                            Err(_) => Err(()),
                        }
                    }
                    Err(_) => Err(()),
                }
            }
            None => Err(()),
        }
    }
}

#[instrument]
pub async fn find_remote(bounds: Bounds, flatgeobuf_url: String) -> Result<GeoJson, ()> {
    use flatgeobuf::*;

    debug!("getting from url {}", flatgeobuf_url);
    let mut fgb = HttpFgbReader::open(&flatgeobuf_url)
        .await
        .unwrap()
        .select_bbox(bounds.sw_lon, bounds.sw_lat, bounds.ne_lon, bounds.ne_lat)
        .await
        .unwrap();

    debug!("converting to geojson string");

    let mut buf = vec![];
    let cursor = Cursor::new(&mut buf);
    let mut gout = GeoJsonWriter::new(cursor);
    fgb.process_features(&mut gout).await.unwrap();

    debug!("converting to geojson object");
    match String::from_utf8(buf) {
        Ok(s) => match s.parse::<GeoJson>() {
            Ok(geojson) => Ok(geojson),
            Err(_) => Err(()),
        },
        Err(_) => Err(()),
    }
}
