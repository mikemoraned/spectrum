use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};

use crate::geo_assets::GeoAssets;

pub struct Finder {}

impl Finder {
    pub fn new() -> Finder {
        Finder {}
    }

    // pub fn find(&self) -> Result<GeoJson, ()> {
    //     let geometry = Geometry::new(Value::Polygon(vec![vec![
    //         vec![-67.13734, 45.13745],
    //         vec![-66.96466, 44.8097],
    //         vec![-68.03252, 44.3252],
    //         vec![-69.06, 43.98],
    //         vec![-70.11617, 43.68405],
    //         vec![-70.64573, 43.09008],
    //         vec![-70.75102, 43.08003],
    //         vec![-70.79761, 43.21973],
    //         vec![-70.98176, 43.36789],
    //         vec![-70.94416, 43.46633],
    //         vec![-71.08482, 45.30524],
    //         vec![-70.66002, 45.46022],
    //         vec![-70.30495, 45.91479],
    //         vec![-70.00014, 46.69317],
    //         vec![-69.23708, 47.44777],
    //         vec![-68.90478, 47.18479],
    //         vec![-68.2343, 47.35462],
    //         vec![-67.79035, 47.06624],
    //         vec![-67.79141, 45.70258],
    //         vec![-67.13734, 45.13745],
    //     ]]));
    //     let meadows = Feature {
    //         bbox: None,
    //         geometry: Some(geometry),
    //         id: None,
    //         properties: None,
    //         foreign_members: None,
    //     };

    //     println!("Creating features");
    //     let features = vec![meadows];
    //     println!("Created {} features", features.len());

    //     println!("Creating feature collection");
    //     let geojson = GeoJson::FeatureCollection(FeatureCollection {
    //         bbox: None,
    //         features,
    //         foreign_members: None,
    //     });
    //     println!("Created feature collection");

    //     Ok(geojson)
    // }

    pub fn find(&self) -> Result<GeoJson, ()> {
        match GeoAssets::get("edinburgh.geojson") {
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
