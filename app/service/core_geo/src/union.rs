use std::collections::HashSet;

use geo::{BooleanOps, Geometry, Intersects, MultiPolygon, Polygon};
use rstar::{
    primitives::{CachedEnvelope, GeomWithData},
    RTree,
};
use tracing::{debug, instrument, trace, warn};

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
struct PolygonId(usize);

struct Partitioned {
    // sets of polygons which do not overlap, where polygons are represented by their index
    disjunctive_groups: Vec<HashSet<PolygonId>>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct GroupId(usize);

#[instrument(skip(polygons))]
fn intersection_candidates(polygons: &[Polygon]) -> Vec<(PolygonId, PolygonId)> {
    let entries = polygons
        .iter()
        .enumerate()
        .map(|(i, polygon)| CachedEnvelope::new(GeomWithData::new(polygon.clone(), PolygonId(i))))
        .collect::<Vec<_>>();

    debug!("Building RTree");
    let rtree = RTree::bulk_load(entries);
    debug!("RTree built");

    debug!("Finding intersection candidates");
    let mut candidates: Vec<(PolygonId, PolygonId)> = vec![];
    for (p1, p2) in rtree.intersection_candidates_with_other_tree(&rtree) {
        if p1.data != p2.data {
            candidates.push((p1.data, p2.data));
        }
    }
    debug!("Found {} candidates", candidates.len());

    candidates
}

#[instrument(skip(polygons))]
fn partition(polygons: &[Polygon]) -> Partitioned {
    let polygon_ids = polygons
        .iter()
        .enumerate()
        .map(|(i, _)| PolygonId(i))
        .collect::<Vec<_>>();
    let mut disjunctive_groups: Vec<HashSet<PolygonId>> = polygon_ids
        .iter()
        .map(|id| HashSet::from_iter(vec![*id]))
        .collect();
    let mut group_ids: Vec<GroupId> = polygon_ids
        .iter()
        .enumerate()
        .map(|(i, _)| GroupId(i))
        .collect::<Vec<_>>();

    let candidates = intersection_candidates(polygons);

    debug!("Finding disjunctive groups");
    for (p1_id, p2_id) in candidates {
        // don't bother checking for intersection if already in the same group
        if group_ids[p1_id.0] != group_ids[p2_id.0] {
            if polygons[p1_id.0].intersects(&polygons[p2_id.0]) {
                trace!("move, {:?} <- {:?}", group_ids[p1_id.0], group_ids[p2_id.0]);
                trace!("A: disjunctive_groups: {:?}", disjunctive_groups);
                // they intersect, so merge the groups by moving
                // everything from p2_group into p1_group
                let p2_group = disjunctive_groups[group_ids[p2_id.0].0].clone();
                trace!(
                    "{:?} + {:?}",
                    disjunctive_groups[group_ids[p1_id.0].0],
                    p2_group,
                );
                disjunctive_groups[group_ids[p1_id.0].0].extend(&p2_group);
                trace!("= {:?}", disjunctive_groups[group_ids[p1_id.0].0]);
                trace!("B: disjunctive_groups: {:?}", disjunctive_groups);
                disjunctive_groups[group_ids[p2_id.0].0].clear();
                for id in p2_group.iter() {
                    group_ids[id.0] = group_ids[p1_id.0];
                }
                trace!("C: disjunctive_groups: {:?}", disjunctive_groups);
            }
        } else {
            trace!(
                "already in same group, {:?}, {:?}",
                group_ids[p1_id.0],
                group_ids[p2_id.0]
            );
        }
    }
    debug!("Found {} disjunctive groups", disjunctive_groups.len());

    trace!("disjunctive_groups: {:?}", disjunctive_groups);

    Partitioned { disjunctive_groups }
}

#[instrument(skip(geometry))]
pub fn union(
    geometry: Vec<Geometry<f64>>,
) -> Result<Vec<Geometry<f64>>, Box<dyn std::error::Error>> {
    let polygons = geometry
        .iter()
        .filter_map(|g| match g {
            Geometry::Polygon(p) => Some(p.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if polygons.is_empty() {
        return Err("No polygons found".into());
    };

    let partitioned = partition(&polygons);

    trace!(
        "Num groups needing unioned: {}",
        partitioned.disjunctive_groups.len()
    );

    debug!("Unioning polygons");
    let mut unioned_polygons: Vec<Polygon<f64>> = vec![];
    for group in partitioned.disjunctive_groups {
        if group.is_empty() {
            continue;
        }
        if group.len() == 1 {
            let id = group.iter().next().unwrap().0;
            unioned_polygons.push(polygons[id].clone());
        } else {
            let multi: Vec<MultiPolygon> = group
                .into_iter()
                .map(|p| MultiPolygon::new(vec![polygons[p.0].clone()]))
                .collect();

            let unioned = multi
                .iter()
                .skip(1)
                .fold(multi[0].clone(), panic_safe_union);

            unioned_polygons.append(unioned.into_iter().collect::<Vec<Polygon<f64>>>().as_mut());
        }
    }
    debug!("Unioned {} polygons", unioned_polygons.len());

    debug!("converting to Geometry");
    let unioned = unioned_polygons
        .into_iter()
        .map(Geometry::Polygon)
        .collect();
    debug!("converted to Geometry");

    Ok(unioned)
}

fn panic_safe_union(lhs: MultiPolygon, rhs: &MultiPolygon) -> MultiPolygon {
    use std::panic;

    let result = panic::catch_unwind(|| lhs.union(rhs));

    match result {
        Ok(unioned) => unioned,
        Err(_) => {
            warn!("Panic detected in union, falling back to lhs of attempted union");
            lhs
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::{collections::HashSet, hash::Hash};

    use super::*;

    fn polygon(geometry: &Geometry<f64>) -> Option<&Polygon<f64>> {
        match geometry {
            Geometry::Polygon(p) => Some(p),
            _ => None,
        }
    }

    #[derive(Debug, PartialEq, Eq, Hash)]
    struct Edge {
        from: String,
        to: String,
    }

    fn as_edgeset(poly: &Polygon<f64>) -> Vec<Edge> {
        let mut edges: HashSet<Edge> = HashSet::new();
        let coords: Vec<String> = poly
            .exterior()
            .coords()
            .map(|c| format!("{:?}", c.x_y()))
            .collect();
        for i in 0..coords.len() - 1 {
            edges.insert(Edge {
                from: coords[i].clone(),
                to: coords[(i + 1) % coords.len()].clone(),
            });
        }
        let mut sorted: Vec<Edge> = edges.into_iter().collect();
        sorted.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));
        sorted
    }

    fn pretty_print_edgeset(edgeset: &Vec<Edge>) -> String {
        let edges: Vec<String> = edgeset
            .iter()
            .map(|e| format!("{}->{}", e.from, e.to))
            .collect();
        format!("[{}]", edges.join(","))
    }

    #[test_log::test]
    fn union_of_two_overlapping_polygons() {
        let p1 = Geometry::Polygon(Polygon::new(
            vec![(1.0, 1.0), (2.0, 1.0), (2.0, 2.0), (1.0, 2.0), (1.0, 1.0)].into(),
            vec![],
        ));
        let p2 = Geometry::Polygon(Polygon::new(
            vec![(1.5, 1.5), (2.5, 1.5), (2.5, 2.5), (1.5, 2.5), (1.5, 1.5)].into(),
            vec![],
        ));
        let expected_p = Geometry::Polygon(Polygon::new(
            vec![
                (1.0, 1.0),
                (2.0, 1.0),
                (2.0, 1.5),
                (2.5, 1.5),
                (2.5, 2.5),
                (1.5, 2.5),
                (1.5, 2.0),
                (1.0, 2.0),
                (1.0, 1.0),
            ]
            .into(),
            vec![],
        ));
        let actual = union(vec![p1, p2]).unwrap();
        let expected = vec![expected_p];
        assert_eq!(actual.len(), 1);
        assert_equivalent_polygons(polygon(&actual[0]).unwrap(), polygon(&expected[0]).unwrap());
    }

    #[test_log::test]
    fn union_where_one_polygon_contains_one_other() {
        let p1 = Geometry::Polygon(Polygon::new(
            vec![(1.0, 1.0), (2.0, 1.0), (2.0, 2.0), (1.0, 2.0), (1.0, 1.0)].into(),
            vec![],
        ));
        let outer = Geometry::Polygon(Polygon::new(
            vec![(0.5, 0.5), (3.0, 0.5), (3.0, 3.0), (0.5, 3.0), (0.5, 0.5)].into(),
            vec![],
        ));
        let actual = union(vec![outer.clone(), p1]).unwrap();
        let expected = vec![outer];
        assert_eq!(actual.len(), 1);
        assert_equivalent_polygons(polygon(&actual[0]).unwrap(), polygon(&expected[0]).unwrap());
    }

    #[test]
    fn union_where_one_polygon_contains_multple_others() {
        let p1 = Geometry::Polygon(Polygon::new(
            vec![(1.0, 1.0), (2.0, 1.0), (2.0, 2.0), (1.0, 2.0), (1.0, 1.0)].into(),
            vec![],
        ));
        let p2 = Geometry::Polygon(Polygon::new(
            vec![(1.5, 1.5), (2.5, 1.5), (2.5, 2.5), (1.5, 2.5), (1.5, 1.5)].into(),
            vec![],
        ));
        let outer = Geometry::Polygon(Polygon::new(
            vec![(0.5, 0.5), (3.0, 0.5), (3.0, 3.0), (0.5, 3.0), (0.5, 0.5)].into(),
            vec![],
        ));
        let actual = union(vec![outer.clone(), p1, p2]).unwrap();
        let expected = vec![outer];
        for a in &actual {
            println!(
                "{:?}",
                pretty_print_edgeset(&as_edgeset(polygon(a).unwrap()))
            );
        }
        assert_eq!(actual.len(), 1);
        assert_equivalent_polygons(polygon(&actual[0]).unwrap(), polygon(&expected[0]).unwrap());
    }

    fn assert_equivalent_polygons(actual: &Polygon<f64>, expected: &Polygon<f64>) {
        let actual_edges = pretty_print_edgeset(&as_edgeset(actual));
        let expected_edges = pretty_print_edgeset(&as_edgeset(expected));
        assert_eq!(actual_edges, expected_edges);
    }

    #[test]
    fn test_as_edgeset() {
        let p = Polygon::new(
            vec![(1.0, 1.0), (2.0, 1.0), (2.0, 2.0), (1.0, 2.0), (1.0, 1.0)].into(),
            vec![],
        );
        let actual_edgeset = as_edgeset(&p);
        let expected_edgeset = vec![
            Edge {
                from: "(1.0, 1.0)".to_string(),
                to: "(2.0, 1.0)".to_string(),
            },
            Edge {
                from: "(1.0, 2.0)".to_string(),
                to: "(1.0, 1.0)".to_string(),
            },
            Edge {
                from: "(2.0, 1.0)".to_string(),
                to: "(2.0, 2.0)".to_string(),
            },
            Edge {
                from: "(2.0, 2.0)".to_string(),
                to: "(1.0, 2.0)".to_string(),
            },
        ];
        assert_eq!(
            pretty_print_edgeset(&actual_edgeset),
            pretty_print_edgeset(&expected_edgeset)
        );
    }
}
