use geojson::{Feature, GeoJson, Geometry, Position, Value};
use osmpbf::{Element, IndexedReader};
use std::fs::OpenOptions;
use std::io::Write;

fn main() -> Result<(), ()> {
    let mut reader = IndexedReader::from_path("data/edinburgh_scotland.osm.pbf").unwrap();

    let mut pending_refs = vec![];

    reader
        .read_ways_and_deps(
            |way| {
                way.tags()
                    .any(|key_value| key_value == ("name", "The Meadows"))
            },
            |element| {
                // Increment counter
                match element {
                    Element::Way(way) => {
                        println!("way: {:?}", way.id());
                        way.refs().for_each(|r| {
                            pending_refs.push(r);
                        })
                    }
                    _ => (),
                }
            },
        )
        .unwrap();

    println!("pending refs: {:?}", pending_refs);

    let mut positions: Vec<Position> = Vec::new();
    positions.resize(pending_refs.len(), Vec::new());

    reader
        .read_ways_and_deps(
            |way| {
                way.tags()
                    .any(|key_value| key_value == ("name", "The Meadows"))
            },
            |element| {
                // Increment counter
                match element {
                    Element::DenseNode(dense_node) => {
                        println!(
                            "dense_node: lat: {}, lon: {}",
                            dense_node.lat(),
                            dense_node.lon()
                        );
                        let position = vec![dense_node.lon(), dense_node.lat()];
                        for i in 0..pending_refs.len() {
                            if pending_refs[i] == dense_node.id() {
                                positions[i] = position.clone();
                                println!("slotting in {} at position {}", dense_node.id(), i);
                            }
                        }
                    }
                    _ => (),
                }
            },
        )
        .unwrap();

    // // add first position to end to make it a ring
    // positions.push(positions[0].clone());

    let geometry = Geometry::new(Value::Polygon(vec![positions]));

    let geojson = GeoJson::Feature(Feature {
        bbox: None,
        geometry: Some(geometry),
        id: None,
        properties: None,
        foreign_members: None,
    });

    let geojson_json = geojson.to_json_value();

    let geojson_string = serde_json::to_string_pretty(&geojson_json).unwrap();

    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("data/meadows.json")
        .unwrap();
    f.write_all(geojson_string.as_bytes()).unwrap();

    Ok(())
}
