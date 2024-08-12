use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use std::{
    fmt::{Display, Formatter},
    path::Path,
};

use geo::geometry::{Coord, Geometry, GeometryCollection, LineString, Polygon};
use osmpbf::{Element, ElementReader, Relation, Way};
use tracing::{debug, instrument};

use crate::filter::GreenTags;
use crate::progress::progress_bar;

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
                false
            }) {
                self.ways.insert(WayId(outer_way.member_id));
                self.ways_via_relation_count += 1;
            }
        }
    }

    fn into_pending_stage(self) -> PendingStage {
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
    refs_for_ways: Vec<Vec<RefId>>,
}

impl PendingStage {
    fn new(allowed_ways: HashSet<WayId>) -> Self {
        PendingStage {
            allowed_ways,
            refs_for_ways: Vec::default(),
        }
    }
}

impl Display for PendingStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PendingStage, saw #ways: {}", self.refs_for_ways.len())
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
            });
            self.refs_for_ways.push(refs_for_way);
        }
    }

    fn into_assign_stage(self) -> AssignStage {
        AssignStage {
            refs_for_ways: self.refs_for_ways,
            coords_for_refs: HashMap::default(),
        }
    }
}

struct AssignStage {
    refs_for_ways: Vec<Vec<RefId>>,
    coords_for_refs: HashMap<RefId, Coord>,
}

impl AssignStage {
    fn add_coord_for_ref_id(&mut self, ref_id: RefId, coord: &Coord) {
        self.coords_for_refs.insert(ref_id, *coord);
    }

    fn into_geometry(self) -> Vec<Geometry<f64>> {
        let mut geometry = vec![];
        let bar = progress_bar(self.refs_for_ways.len() as u64);
        for ref_ids in self.refs_for_ways.into_iter() {
            let coords = ref_ids
                .into_iter()
                .map(|ref_id| self.coords_for_refs.get(&ref_id).unwrap())
                .cloned()
                .collect::<Vec<Coord>>();
            let polygon = Polygon::new(LineString::from(coords), vec![]);
            geometry.push(Geometry::Polygon(polygon));
            bar.inc(1);
        }
        bar.finish();
        geometry
    }
}

impl Display for AssignStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AssignStage {{ ways: {} }}", self.refs_for_ways.len())
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
    let mut pending_stage = filter_stage.into_pending_stage();
    let pending_stage_bar = progress_bar(total_elements);
    let element_reader = ElementReader::from_path(osmpbf_path)?;
    element_reader.for_each(|element| {
        if let Element::Way(way) = element {
            pending_stage.append_way(&way);
        }
        pending_stage_bar.inc(1);
    })?;
    pending_stage_bar.finish();
    debug!("Collected: {}", pending_stage);

    debug!("Assigning Coords");
    let mut assign_stage = pending_stage.into_assign_stage();
    debug!("Created stage");
    let assign_stage_bar = progress_bar(total_elements);
    let element_reader = ElementReader::from_path(osmpbf_path)?;
    element_reader.for_each(|element| {
        match element {
            Element::DenseNode(dense_node) => {
                let coord = Coord::from((dense_node.lon(), dense_node.lat()));
                assign_stage.add_coord_for_ref_id(RefId(dense_node.id()), &coord);
            }
            Element::Node(node) => {
                let coord = Coord::from((node.lon(), node.lat()));
                assign_stage.add_coord_for_ref_id(RefId(node.id()), &coord);
            }
            _ => (),
        };
        assign_stage_bar.inc(1);
    })?;
    assign_stage_bar.finish();

    debug!("Found positions for ways: {}", assign_stage);

    debug!("Creating polygons");
    let geometry = assign_stage.into_geometry();
    debug!("Created {} polygons", geometry.len());

    Ok(GeometryCollection::from_iter(geometry))
}
