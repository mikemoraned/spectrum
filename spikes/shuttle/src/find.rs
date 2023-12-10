use geojson::{FeatureCollection, GeoJson};

pub fn find() -> Result<GeoJson, ()> {
    println!("Creating features");
    let mut features = vec![];
    println!("Created {} features", features.len());

    println!("Creating feature collection");
    let geojson = GeoJson::FeatureCollection(FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    });
    println!("Created feature collection");

    Ok(geojson)
}
