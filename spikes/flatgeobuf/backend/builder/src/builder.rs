use std::collections::{HashMap, HashSet};

use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Position, Value};
use osmpbf::{Element, IndexedReader};

pub fn build() -> Result<GeoJson, ()> {
    let mut reader = IndexedReader::from_path("data/edinburgh_scotland.osm.pbf").unwrap();

    fn way_filter(way: &osmpbf::Way<'_>) -> bool {
        let generic: Vec<(&str, &str)> = vec![
            ("leisure", "common"),
            ("leisure", "dog_park"),
            ("leisure", "golf_course"),
            ("leisure", "horse_riding"),
            ("leisure", "nature_reserve"),
            ("leisure", "park"),
            ("leisure", "pitch"),
            ("leisure", "wildlife_hide"),
            ("natural", "fell"),
            ("natural", "grassland"),
            ("natural", "heath"),
            ("natural", "moor"),
            ("natural", "scrub"),
            ("natural", "shrubbery"),
            ("natural", "tree"),
            ("natural", "tree_row"),
            ("natural", "tree_stump"),
            ("natural", "tundra"),
            ("natural", "wood"),
            ("amenity", "grave_yard"),
            ("landuse", "farmland"),
            ("landuse", "farmyard"),
            ("landuse", "forest"),
            ("landuse", "meadow"),
            ("landuse", "orchard"),
            ("landuse", "vineyard"),
            ("landuse", "cemetery"),
            ("landuse", "grass"),
            ("landuse", "recreation_ground"),
            ("landuse", "village_green"),
        ];
        let generic_tag_set: HashSet<(&str, &str)> = generic.into_iter().collect();
        let tag_set: HashSet<(&str, &str)> = way.tags().collect();
        if tag_set.contains(&("leisure", "garden")) {
            tag_set.contains(&("access", "yes")) || tag_set.contains(&("garden:type", "community"))
        } else {
            tag_set.intersection(&generic_tag_set).count() > 0
        }
    }

    let mut pending_refs_for_ways: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut pending_ways_for_refs: HashMap<i64, Vec<i64>> = HashMap::new();

    println!("Finding pending ways");
    reader
        .read_ways_and_deps(way_filter, |element| match element {
            Element::Way(way) => {
                let mut pending_refs_for_way = vec![];
                way.refs().for_each(|r| {
                    pending_refs_for_way.push(r);
                    let pending_ways = pending_ways_for_refs.entry(r).or_default();
                    pending_ways.push(way.id());
                });
                pending_refs_for_ways.insert(way.id(), pending_refs_for_way);
            }
            _ => (),
        })
        .unwrap();

    println!("Found {} pending ways", pending_refs_for_ways.len());

    println!("Finding positions for ways");
    let mut positions_for_way: HashMap<i64, Vec<Position>> = HashMap::new();
    for (way_id, pending_refs) in pending_refs_for_ways.iter() {
        let mut positions: Vec<Position> = Vec::new();
        positions.resize(pending_refs.len(), Vec::new());
        positions_for_way.insert(*way_id, positions);
    }

    fn insert_position_into_way(
        node_id: i64,
        position: &Position,
        pending_refs_for_ways: &HashMap<i64, Vec<i64>>,
        pending_ways_for_refs: &HashMap<i64, Vec<i64>>,
        positions_for_way: &mut HashMap<i64, Vec<Vec<f64>>>,
    ) {
        let pending_ways = pending_ways_for_refs.get(&node_id).unwrap();
        for way_id in pending_ways {
            let pending_refs = pending_refs_for_ways.get(way_id).unwrap();
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
                    &pending_refs_for_ways,
                    &pending_ways_for_refs,
                    &mut positions_for_way,
                );
            }
            Element::Node(node) => {
                let position = vec![node.lon(), node.lat()];
                insert_position_into_way(
                    node.id(),
                    &position,
                    &pending_refs_for_ways,
                    &pending_ways_for_refs,
                    &mut positions_for_way,
                );
            }
            _ => (),
        })
        .unwrap();

    println!("Found positions for ways");

    println!("Creating features");

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
