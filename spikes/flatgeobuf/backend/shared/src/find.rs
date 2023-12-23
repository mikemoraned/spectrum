use geojson::GeoJson;
use serde::Deserialize;

use crate::geo_assets::GeoAssets;

#[derive(Deserialize, Debug)]
pub struct Bounds {
    sw_lat: f64,
    sw_lon: f64,
    ne_lat: f64,
    ne_lon: f64,
}

pub struct Finder {}

impl Finder {
    pub fn new() -> Finder {
        Finder {}
    }

    pub fn find(&self) -> Result<GeoJson, ()> {
        match GeoAssets::get("find.json") {
            Some(f) => match String::from_utf8(f.data.to_vec()) {
                Ok(s) => match s.parse::<GeoJson>() {
                    Ok(geojson) => Ok(geojson),
                    Err(_) => Err(()),
                },
                Err(_) => Err(()),
            },
            None => Err(()),
        }
    }

    pub fn find_flatgeobuf(&self, bounds: Bounds) -> Result<GeoJson, ()> {
        use flatgeobuf::*;
        use geozero::ProcessToJson;
        use std::io::Cursor;

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
