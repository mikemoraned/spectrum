use geojson::GeoJson;

use crate::geo_assets::GeoAssets;

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

    pub fn find_flatgeobuf(&self) -> Result<GeoJson, ()> {
        use flatgeobuf::*;
        use geozero::ProcessToJson;
        use std::io::Cursor;

        match GeoAssets::get("find.fgb") {
            Some(f) => {
                let reader = Cursor::new(f.data);
                match FgbReader::open(reader) {
                    Ok(fgb) => match fgb.select_all() {
                        Ok(mut fgb) => match fgb.to_json().unwrap().parse::<GeoJson>() {
                            Ok(geojson) => Ok(geojson),
                            Err(_) => Err(()),
                        },
                        Err(_) => Err(()),
                    },
                    Err(_) => Err(()),
                }
            }
            None => Err(()),
        }
    }
}
