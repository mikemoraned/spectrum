use std::collections::HashMap;

use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Position, Value};
use osmpbf::{Element, IndexedReader};

pub fn find() -> Result<GeoJson, ()> {
    let mut reader = IndexedReader::from_path("data/edinburgh_scotland.osm.pbf").unwrap();

    fn way_filter(way: &osmpbf::Way<'_>) -> bool {
        let pairs = vec![
            ("leisure", "park"),
            // ("leisure", "garden")
        ];
        way.tags().any(|key_value| pairs.contains(&key_value))
    }

    let mut pending_refs = HashMap::new();

    reader
        .read_ways_and_deps(way_filter, |element| match element {
            Element::Way(way) => {
                let mut pending_refs_for_way = vec![];
                way.refs().for_each(|r| {
                    pending_refs_for_way.push(r);
                });
                pending_refs.insert(way.id(), pending_refs_for_way);
            }
            _ => (),
        })
        .unwrap();

    let mut positions_for_way: HashMap<i64, Vec<Position>> = HashMap::new();
    for (way_id, pending_refs) in pending_refs.iter() {
        let mut positions: Vec<Position> = Vec::new();
        positions.resize(pending_refs.len(), Vec::new());
        positions_for_way.insert(*way_id, positions);
    }

    fn insert_position_into_way(
        node_id: i64,
        position: &Position,
        pending_refs: &HashMap<i64, Vec<i64>>,
        positions_for_way: &mut HashMap<i64, Vec<Vec<f64>>>,
    ) {
        for (way_id, pending_refs) in pending_refs.iter() {
            let positions = positions_for_way.get_mut(way_id).unwrap();
            for i in 0..pending_refs.len() {
                if pending_refs[i] == node_id {
                    positions[i] = position.clone();
                }
            }
        }
    }

    reader
        .read_ways_and_deps(way_filter, |element| match element {
            Element::DenseNode(dense_node) => {
                let position = vec![dense_node.lon(), dense_node.lat()];
                insert_position_into_way(
                    dense_node.id(),
                    &position,
                    &pending_refs,
                    &mut positions_for_way,
                );
            }
            Element::Node(node) => {
                let position = vec![node.lon(), node.lat()];
                insert_position_into_way(
                    node.id(),
                    &position,
                    &pending_refs,
                    &mut positions_for_way,
                );
            }
            _ => (),
        })
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

    Ok(geojson)
}
