use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Position, Value};
use osmpbf::{Element, IndexedReader};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

fn main() -> Result<(), ()> {
    let mut reader = IndexedReader::from_path("data/edinburgh_scotland.osm.pbf").unwrap();

    // let mut pending_refs = vec![];
    let mut pending_refs: HashMap<i64, _> = HashMap::new();

    reader
        .read_ways_and_deps(
            |way| way.tags().any(|key_value| key_value == ("leisure", "park")),
            |element| {
                // Increment counter
                match element {
                    Element::Way(way) => {
                        println!("way: {:?}", way.id());
                        let mut pending_refs_for_way = vec![];
                        way.refs().for_each(|r| {
                            pending_refs_for_way.push(r);
                        });
                        pending_refs.insert(way.id(), pending_refs_for_way);
                    }
                    _ => (),
                }
            },
        )
        .unwrap();

    println!("pending refs: {:?}", pending_refs);

    // let mut positions: Vec<Position> = Vec::new();
    let mut positions_for_way: HashMap<i64, Vec<Position>> = HashMap::new();
    for (way_id, pending_refs) in pending_refs.iter() {
        let mut positions: Vec<Position> = Vec::new();
        positions.resize(pending_refs.len(), Vec::new());
        positions_for_way.insert(*way_id, positions);
    }

    reader
        .read_ways_and_deps(
            |way| way.tags().any(|key_value| key_value == ("leisure", "park")),
            |element| match element {
                Element::DenseNode(dense_node) => {
                    println!(
                        "dense_node: lat: {}, lon: {}",
                        dense_node.lat(),
                        dense_node.lon()
                    );
                    let position = vec![dense_node.lon(), dense_node.lat()];
                    for (way_id, pending_refs) in pending_refs.iter() {
                        let positions = positions_for_way.get_mut(way_id).unwrap();
                        for i in 0..pending_refs.len() {
                            if pending_refs[i] == dense_node.id() {
                                positions[i] = position.clone();
                                println!(
                                    "slotting in {} at position {} in way {}",
                                    dense_node.id(),
                                    i,
                                    way_id
                                );
                            }
                        }
                    }
                }
                _ => (),
            },
        )
        .unwrap();

    let mut features = vec![];
    for (_way_id, positions) in positions_for_way.iter() {
        let geometry = Geometry::new(Value::Polygon(vec![positions.clone()]));
        features.push(Feature {
            bbox: None,
            geometry: Some(geometry),
            id: None,
            properties: None,
            foreign_members: None,
        });
    }
    let geojson = GeoJson::FeatureCollection(FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    });

    let geojson_json = geojson.to_json_value();

    let geojson_string = serde_json::to_string_pretty(&geojson_json).unwrap();

    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("data/landuse.json")
        .unwrap();
    f.write_all(geojson_string.as_bytes()).unwrap();

    Ok(())
}
