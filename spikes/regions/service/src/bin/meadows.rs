use geojson::{Feature, GeoJson, Geometry, Position, Value};
use osmpbf::{Element, IndexedReader};
use std::fs::OpenOptions;
use std::io::Write;

fn main() -> Result<(), ()> {
    let mut reader = IndexedReader::from_path("data/edinburgh_scotland.osm.pbf").unwrap();

    let mut positions: Vec<Position> = vec![];

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
                    }
                    Element::Node(node) => {
                        println!("node: lat: {}, lon: {}", node.lat(), node.lon());
                    }
                    Element::DenseNode(dense_node) => {
                        println!(
                            "dense_node: lat: {}, lon: {}",
                            dense_node.lat(),
                            dense_node.lon()
                        );
                        let position = vec![dense_node.lon(), dense_node.lat()];
                        positions.push(position);
                    }
                    _ => (),
                }
            },
        )
        .unwrap();

    // add first position to end to make it a ring
    positions.push(positions[0].clone());

    let geometry = Geometry::new(Value::Polygon(vec![positions]));

    let geojson = GeoJson::Feature(Feature {
        bbox: None,
        geometry: Some(geometry),
        id: None,
        properties: None,
        foreign_members: None,
    });

    let geojson_string = geojson.to_string();

    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("data/meadows.json")
        .unwrap();
    f.write_all(geojson_string.as_bytes()).unwrap();

    Ok(())
}
