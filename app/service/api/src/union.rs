use std::collections::HashSet;

use geo::{BooleanOps, Geometry, Intersects, MultiPolygon, Polygon};
use tracing::debug;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
struct PolygonId(usize);

struct Partitioned {
    // sets of polygons which do not overlap, where polygons are represented by their index
    disjunctive_groups: Vec<HashSet<PolygonId>>,
}

#[derive(Clone)]
enum GroupId {
    Some(usize),
    None,
}

fn partition(polygons: &Vec<Polygon>) -> Partitioned {
    let polygon_ids = polygons
        .iter()
        .enumerate()
        .map(|(i, _)| PolygonId(i))
        .collect::<Vec<_>>();
    let mut disjunctive_groups: Vec<HashSet<PolygonId>> = vec![];
    let mut group_ids: Vec<GroupId> = vec![];
    group_ids.resize(polygon_ids.len(), GroupId::None);

    for from_id in polygon_ids.iter() {
        for to_id in polygon_ids.iter() {
            if from_id.0 < to_id.0 {
                if polygons[from_id.0].intersects(&polygons[to_id.0]) {
                    if let GroupId::Some(from_group_id) = group_ids[from_id.0] {
                        if let GroupId::Some(to_group_id) = group_ids[to_id.0] {
                            // both belong already to a group, so need to combine
                            let to_group = disjunctive_groups[to_group_id].clone();
                            disjunctive_groups[from_group_id].extend(&to_group);
                            for id in to_group.iter() {
                                group_ids[id.0] = GroupId::Some(from_group_id);
                            }
                            disjunctive_groups[to_group_id].clear();
                        } else {
                            // from_id belongs to a group, to_id does not, so add to from_id's group
                            let group = &mut disjunctive_groups[from_group_id];
                            group.insert(*to_id);
                            group_ids[to_id.0] = GroupId::Some(from_group_id);
                        }
                    } else {
                        // neither belong to a group, so create a new group for them
                        let group_id = disjunctive_groups.len();
                        let mut group = HashSet::new();
                        group.insert(*from_id);
                        group.insert(*to_id);
                        disjunctive_groups.push(group);
                        group_ids[from_id.0] = GroupId::Some(group_id);
                        group_ids[to_id.0] = GroupId::Some(group_id);
                    }
                }
            }
        }
    }

    let disjunctive_groups = disjunctive_groups
        .into_iter()
        .filter(|g| !g.is_empty())
        .collect();

    Partitioned { disjunctive_groups }
}

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

    debug!(
        "Num groups needing unioned: {}",
        partitioned.disjunctive_groups.len()
    );

    let mut unioned_polygons: Vec<Polygon<f64>> = vec![];
    for group in partitioned.disjunctive_groups {
        debug!("group: {:?}", group);
        let multi: Vec<MultiPolygon> = group
            .into_iter()
            .map(|p| MultiPolygon::new(vec![polygons[p.0].clone()]))
            .collect();

        let unioned = multi
            .iter()
            .skip(1)
            .fold(multi[0].clone(), |acc, p| acc.union(p));

        unioned_polygons.append(unioned.into_iter().collect::<Vec<Polygon<f64>>>().as_mut());
    }

    let unioned = unioned_polygons
        .into_iter()
        .map(|p| Geometry::Polygon(p))
        .collect();

    Ok(unioned)
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

    #[test]
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

    #[test]
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
