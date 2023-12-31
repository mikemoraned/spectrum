use std::{fs::OpenOptions, io::Write};

use service::find::find;

fn main() -> Result<(), ()> {
    let geojson = find().unwrap();

    let geojson_json = geojson.to_json_value();

    let geojson_string = serde_json::to_string_pretty(&geojson_json).unwrap();

    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("data/find.json")
        .unwrap();
    f.write_all(geojson_string.as_bytes()).unwrap();

    Ok(())
}
