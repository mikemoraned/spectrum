use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use geo::geometry::{Coord, Geometry, GeometryCollection, LineString, Polygon};
use osmpbf::{Element, IndexedReader};
use tracing::{debug, instrument};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct WayId(i64);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct RefId(i64);

#[instrument]
pub fn extract_regions(
    osmpbf_path: &Path,
) -> Result<GeometryCollection<f64>, Box<dyn std::error::Error>> {
    let mut reader = IndexedReader::from_path(osmpbf_path).expect("Failed to open file");

    let mut pending_refs_for_ways: HashMap<WayId, Vec<RefId>> = HashMap::new();
    let mut pending_ways_for_refs: HashMap<RefId, Vec<WayId>> = HashMap::new();

    debug!("Finding pending ways");
    reader
        .read_ways_and_deps(way_filter, |element| {
            if let Element::Way(way) = element {
                let way_id = WayId(way.id());
                let mut pending_refs_for_way: Vec<RefId> = vec![];
                way.refs().for_each(|r| {
                    let ref_id = RefId(r);
                    pending_refs_for_way.push(ref_id);
                    let pending_ways = pending_ways_for_refs.entry(ref_id).or_default();
                    pending_ways.push(way_id);
                });
                pending_refs_for_ways.insert(way_id, pending_refs_for_way);
            }
        })
        .unwrap();

    debug!("Found {} pending ways", pending_refs_for_ways.len());

    debug!("Finding coords for ways");
    let mut coords_for_way: HashMap<WayId, Vec<Coord>> = HashMap::new();
    for (way_id, pending_refs) in pending_refs_for_ways.iter() {
        let mut coords: Vec<Coord> = Vec::new();
        coords.resize(pending_refs.len(), Coord::default());
        coords_for_way.insert(*way_id, coords);
    }

    reader
        .read_ways_and_deps(way_filter, |element| match element {
            Element::DenseNode(dense_node) => {
                let coord = Coord::from((dense_node.lon(), dense_node.lat()));
                insert_coord_into_way(
                    RefId(dense_node.id()),
                    &coord,
                    &pending_refs_for_ways,
                    &pending_ways_for_refs,
                    &mut coords_for_way,
                );
            }
            Element::Node(node) => {
                let coord = Coord::from((node.lon(), node.lat()));
                insert_coord_into_way(
                    RefId(node.id()),
                    &coord,
                    &pending_refs_for_ways,
                    &pending_ways_for_refs,
                    &mut coords_for_way,
                );
            }
            _ => (),
        })
        .unwrap();

    debug!("Found positions for ways");

    debug!("Creating polygons");
    let mut geometry = vec![];
    for (_, coords) in coords_for_way.iter() {
        let polygon = Polygon::new(LineString::from(coords.clone()), vec![]);
        geometry.push(Geometry::Polygon(polygon));
    }
    debug!("Created {} polygons", geometry.len());

    Ok(GeometryCollection::from_iter(geometry))
}

fn way_filter(way: &osmpbf::Way<'_>) -> bool {
    let tag_set: HashSet<(&str, &str)> = way.tags().collect();
    tag_filter(tag_set)
}

fn tag_filter(tag_set: HashSet<(&str, &str)>) -> bool {
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
    if tag_set.contains(&("leisure", "garden")) {
        tag_set.contains(&("access", "yes")) || tag_set.contains(&("garden:type", "community"))
    } else {
        tag_set.intersection(&generic_tag_set).count() > 0
    }
}

fn insert_coord_into_way(
    ref_id: RefId,
    coord: &Coord,
    pending_refs_for_ways: &HashMap<WayId, Vec<RefId>>,
    pending_ways_for_refs: &HashMap<RefId, Vec<WayId>>,
    coords_for_way: &mut HashMap<WayId, Vec<Coord>>,
) {
    let pending_ways = pending_ways_for_refs.get(&ref_id).unwrap();
    for way_id in pending_ways {
        let pending_refs = pending_refs_for_ways.get(way_id).unwrap();
        let coords = coords_for_way.get_mut(way_id).unwrap();
        for i in 0..pending_refs.len() {
            if pending_refs[i] == ref_id {
                coords[i] = *coord;
            }
        }
    }
}
