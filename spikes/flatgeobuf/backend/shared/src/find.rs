use std::io::Cursor;

use flatgeobuf::{AsyncFeatureIter, HttpFgbReader};
use geojson::GeoJson;
use geozero::geojson::GeoJsonWriter;
use serde::Deserialize;

use tracing::{debug, instrument, trace};

#[derive(Deserialize, Debug)]
pub struct Bounds {
    sw_lat: f64,
    sw_lon: f64,
    ne_lat: f64,
    ne_lon: f64,
}

#[instrument]
pub async fn find_remote(bounds: Bounds, flatgeobuf_url: String) -> Result<GeoJson, ()> {
    use flatgeobuf::*;

    debug!("starting");
    let reader = open_reader(flatgeobuf_url).await;
    let fgb: AsyncFeatureIter = select_bbox(reader, bounds).await;
    let buf = convert_to_geojson_string(fgb).await;
    let result = convert_to_geojson_object(buf);
    debug!("done");
    result
}

#[instrument]
async fn open_reader(flatgeobuf_url: String) -> HttpFgbReader {
    HttpFgbReader::open(&flatgeobuf_url).await.unwrap()
}

#[instrument(skip(reader))]
async fn select_bbox(reader: HttpFgbReader, bounds: Bounds) -> AsyncFeatureIter {
    trace!("select_bbox");
    reader
        .select_bbox(bounds.sw_lon, bounds.sw_lat, bounds.ne_lon, bounds.ne_lat)
        .await
        .unwrap()
}

#[instrument(skip(fgb))]
async fn convert_to_geojson_string(mut fgb: AsyncFeatureIter) -> Vec<u8> {
    trace!("convert_to_geojson_string");
    let mut buf = vec![];
    let cursor = Cursor::new(&mut buf);
    let mut gout = GeoJsonWriter::new(cursor);
    fgb.process_features(&mut gout).await.unwrap();
    buf
}

#[instrument(skip(buf))]
fn convert_to_geojson_object(buf: Vec<u8>) -> Result<GeoJson, ()> {
    trace!("convert_to_geojson_object");
    match String::from_utf8(buf) {
        Ok(s) => match s.parse::<GeoJson>() {
            Ok(geojson) => Ok(geojson),
            Err(_) => Err(()),
        },
        Err(_) => Err(()),
    }
}
