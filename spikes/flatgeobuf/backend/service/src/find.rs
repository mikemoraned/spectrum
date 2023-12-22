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
}
