use std::io::Cursor;

use flatgeobuf::{AsyncFeatureIter, HttpFgbReader};
use geojson::GeoJson;
use geozero::geojson::GeoJsonWriter;
use serde::Deserialize;

use crate::geo_assets::GeoAssets;
use tracing::{debug, instrument, span, Level};

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
    let reader = open_reader(flatgeobuf_url).await;
    let fgb: AsyncFeatureIter = select_bbox(reader, bounds).await;
    let buf = convert_to_geojson_string(fgb).await;
    convert_to_geojson_object(buf)
}

#[instrument]
async fn open_reader(flatgeobuf_url: String) -> HttpFgbReader {
    HttpFgbReader::open(&flatgeobuf_url).await.unwrap()
}

async fn select_bbox(reader: HttpFgbReader, bounds: Bounds) -> AsyncFeatureIter {
    let span = span!(Level::TRACE, "select_bbox", bounds = ?bounds);
    let _enter = span.enter();
    reader
        .select_bbox(bounds.sw_lon, bounds.sw_lat, bounds.ne_lon, bounds.ne_lat)
        .await
        .unwrap()
}

async fn convert_to_geojson_string(mut fgb: AsyncFeatureIter) -> Vec<u8> {
    let span = span!(Level::TRACE, "convert_to_geojson_string");
    let _enter = span.enter();
    let mut buf = vec![];
    let cursor = Cursor::new(&mut buf);
    let mut gout = GeoJsonWriter::new(cursor);
    fgb.process_features(&mut gout).await.unwrap();
    buf
}

#[instrument]
fn convert_to_geojson_object(buf: Vec<u8>) -> Result<GeoJson, ()> {
    match String::from_utf8(buf) {
        Ok(s) => match s.parse::<GeoJson>() {
            Ok(geojson) => Ok(geojson),
            Err(_) => Err(()),
        },
        Err(_) => Err(()),
    }
}
