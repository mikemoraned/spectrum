use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Formatter},
    path::Path,
};

use geo::geometry::{Coord, Geometry, GeometryCollection, LineString, Polygon};
use indicatif::ProgressBar;
use osmpbf::{Element, ElementReader, Relation, Way};
use tracing::{debug, instrument};

use crate::filter::GreenTags;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct WayId(i64);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct RefId(i64);

#[derive(Default)]
struct FilterStage {
    ways: HashSet<WayId>,
    direct_ways_count: usize,
    ways_via_relation_count: usize,
}

impl FilterStage {
    fn append_way(&mut self, way: &Way) {
        self.ways.insert(WayId(way.id()));
        self.direct_ways_count += 1;
    }

    fn append_relation(&mut self, relation: &Relation) {
        let tag_set: HashSet<(&str, &str)> = relation.tags().collect();
        if tag_set.contains(&("type", "multipolygon")) {
            if let Some(outer_way) = relation.members().find(|m| {
                if m.member_type == osmpbf::RelMemberType::Way {
                    if let Ok("outer") = m.role() {
                        return true;
                    }
                }
                return false;
            }) {
                self.ways.insert(WayId(outer_way.member_id));
                self.ways_via_relation_count += 1;
            }
        }
    }

    fn to_pending_stage(self) -> PendingStage {
        PendingStage::new(self.ways)
    }
}

impl Display for FilterStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FilterStage, #direct: {}, #via_relation: {}, #total: {}",
            self.direct_ways_count,
            self.ways_via_relation_count,
            self.ways.len()
        )
    }
}

struct PendingStage {
    allowed_ways: HashSet<WayId>,
    refs_for_ways: HashMap<WayId, Vec<RefId>>,
    ways_for_refs: HashMap<RefId, Vec<WayId>>,
}

impl PendingStage {
    fn new(allowed_ways: HashSet<WayId>) -> Self {
        PendingStage {
            allowed_ways,
            refs_for_ways: HashMap::new(),
            ways_for_refs: HashMap::new(),
        }
    }
}

impl Display for PendingStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PendingStage, saw #ways: {}, #refs: {}",
            self.refs_for_ways.len(),
            self.ways_for_refs.len()
        )
    }
}

impl PendingStage {
    fn append_way(&mut self, way: &Way) {
        let way_id = WayId(way.id());
        if self.allowed_ways.contains(&way_id) {
            let mut refs_for_way: Vec<RefId> = vec![];
            way.refs().for_each(|r| {
                let ref_id = RefId(r);
                refs_for_way.push(ref_id);
                let ways = self.ways_for_refs.entry(ref_id).or_default();
                ways.push(way_id);
            });
            self.refs_for_ways.insert(way_id, refs_for_way);
        }
    }

    fn to_assignment(self) -> AssignStage {
        let mut coords_for_way: HashMap<WayId, Vec<Coord>> = HashMap::new();
        for (way_id, pending_refs) in self.refs_for_ways.iter() {
            let mut coords: Vec<Coord> = Vec::new();
            coords.resize(pending_refs.len(), Coord::default());
            coords_for_way.insert(*way_id, coords);
        }
        AssignStage {
            coords_for_way,
            refs_for_ways: self.refs_for_ways,
            ways_for_refs: self.ways_for_refs,
        }
    }
}

struct AssignStage {
    coords_for_way: HashMap<WayId, Vec<Coord>>,
    refs_for_ways: HashMap<WayId, Vec<RefId>>,
    ways_for_refs: HashMap<RefId, Vec<WayId>>,
}

impl AssignStage {
    fn insert_coord_into_way(&mut self, ref_id: RefId, coord: &Coord) {
        if let Some(pending_ways) = self.ways_for_refs.get(&ref_id) {
            for way_id in pending_ways {
                let refs = self.refs_for_ways.get(way_id).unwrap();
                let coords = self.coords_for_way.get_mut(way_id).unwrap();
                for i in 0..refs.len() {
                    if refs[i] == ref_id {
                        coords[i] = *coord;
                    }
                }
            }
        }
    }

    fn to_geometry(self) -> Vec<Geometry<f64>> {
        let mut geometry = vec![];
        for (_, coords) in self.coords_for_way.into_iter() {
            let polygon = Polygon::new(LineString::from(coords), vec![]);
            geometry.push(Geometry::Polygon(polygon));
        }
        geometry
    }
}

impl Display for AssignStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AssignStage {{ ways: {} }}", self.coords_for_way.len())
    }
}

#[instrument]
pub fn extract_regions(
    osmpbf_path: &Path,
) -> Result<GeometryCollection<f64>, Box<dyn std::error::Error>> {
    debug!("Filtering Ways");
    let mut filter_stage = FilterStage::default();

    let green_tags = GreenTags::default();
    let way_filter = |way: &Way| {
        let tag_set: HashSet<(&str, &str)> = way.tags().collect();
        green_tags.filter(tag_set)
    };
    let element_reader = ElementReader::from_path(osmpbf_path)?;
    let mut total_elements = 0u64;
    element_reader.for_each(|element| {
        if let Element::Way(way) = element {
            if way_filter(&way) {
                filter_stage.append_way(&way);
            }
        } else if let Element::Relation(relation) = element {
            let tag_set: HashSet<(&str, &str)> = relation.tags().collect();
            if green_tags.filter(tag_set) {
                filter_stage.append_relation(&relation);
            }
        }
        total_elements += 1;
    })?;
    debug!("Filtered: {}", filter_stage);

    debug!("Collecting");
    let mut pending_stage = filter_stage.to_pending_stage();
    let pending_stage_bar = ProgressBar::new(total_elements);
    let element_reader = ElementReader::from_path(osmpbf_path)?;
    element_reader.for_each(|element| {
        if let Element::Way(way) = element {
            pending_stage.append_way(&way);
        }
        pending_stage_bar.inc(1);
    })?;
    pending_stage_bar.finish_and_clear();
    debug!("Collected: {}", pending_stage);

    debug!("Assigning Coords");
    let mut assign_stage = pending_stage.to_assignment();
    debug!("Created stage");
    let assign_stage_bar = ProgressBar::new(total_elements);
    let element_reader = ElementReader::from_path(osmpbf_path)?;
    element_reader.for_each(|element| {
        match element {
            Element::DenseNode(dense_node) => {
                let coord = Coord::from((dense_node.lon(), dense_node.lat()));
                assign_stage.insert_coord_into_way(RefId(dense_node.id()), &coord);
            }
            Element::Node(node) => {
                let coord = Coord::from((node.lon(), node.lat()));
                assign_stage.insert_coord_into_way(RefId(node.id()), &coord);
            }
            _ => (),
        };
        assign_stage_bar.inc(1);
    })?;
    assign_stage_bar.finish_and_clear();

    debug!("Found positions for ways: {}", assign_stage);

    debug!("Creating polygons");
    let geometry = assign_stage.to_geometry();
    debug!("Created {} polygons", geometry.len());

    Ok(GeometryCollection::from_iter(geometry))
}
