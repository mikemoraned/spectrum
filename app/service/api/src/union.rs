use geo::{BooleanOps, Geometry, Intersects, MultiPolygon, Polygon};
use tracing::debug;

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

    let mut groups: Vec<Vec<&Polygon>> = vec![];
    let mut group_indexes: Vec<Option<usize>> = vec![];
    group_indexes.resize(polygons.len(), None);

    for from_p in 0..polygons.len() {
        for to_p in 0..polygons.len() {
            if from_p < to_p {
                if polygons[from_p].intersects(&polygons[to_p]) {
                    if let Some(group_index) = group_indexes[from_p] {
                        let group = &mut groups[group_index];
                        group.push(&polygons[to_p]);
                    } else {
                        let group_index = groups.len();
                        let group = vec![&polygons[from_p], &polygons[to_p]];
                        groups.push(group);
                        group_indexes[from_p] = Some(group_index);
                    }
                }
            }
        }
    }

    debug!("Num groups needing unioned: {}", groups.len());
    debug!(
        "Num Polygons which intersect something else: {}, total: {}",
        group_indexes.iter().filter(|i| i.is_some()).count(),
        polygons.len()
    );

    let mut unioned_polygons: Vec<Polygon<f64>> = vec![];
    debug!("unioning {} groups", groups.len());
    for group in groups {
        // debug!("unioning {} polygons", group.len());
        let multi: Vec<MultiPolygon> = group
            .into_iter()
            .map(|p| MultiPolygon::new(vec![p.clone()]))
            .collect();

        let unioned = multi
            .iter()
            .skip(1)
            .fold(multi[0].clone(), |acc, p| acc.union(p));

        unioned_polygons.append(unioned.into_iter().collect::<Vec<Polygon<f64>>>().as_mut());
    }
    debug!("unioned groups");

    let unioned = unioned_polygons
        .into_iter()
        .map(|p| Geometry::Polygon(p))
        .collect();

    Ok(unioned)
}

#[cfg(test)]
mod tests {
    use geo::{coord, Coord, LineString};

    use super::*;

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

    // #[test]
    // fn _union_of_where_one_polygon_contains_the_other() {
    //     let inner = Geometry::Polygon(Polygon::new(
    //         vec![(1.0, 1.0), (1.0, 3.0), (3.0, 2.0), (3.0, 1.0), (1.0, 1.0)].into(),
    //         vec![],
    //     ));
    //     let outer = Geometry::Polygon(Polygon::new(
    //         vec![(0.0, 0.0), (0.0, 4.0), (4.0, 4.0), (4.0, 0.0), (0.0, 0.0)].into(),
    //         vec![],
    //     ));
    //     let actual = union(vec![outer.clone(), inner]).unwrap();
    //     let expected = vec![outer];
    //     assert_eq!(actual.len(), 1);
    //     assert_equivalent_polygons(polygon(&actual[0]).unwrap(), polygon(&expected[0]).unwrap());
    // }

    fn polygon(geometry: &Geometry<f64>) -> Option<&Polygon<f64>> {
        match geometry {
            Geometry::Polygon(p) => Some(p),
            _ => None,
        }
    }

    // compare the actual and expected polygons, allowing for the coordinates to be in the same order
    // but starting at a different coord
    fn assert_equivalent_polygons(actual: &Polygon<f64>, expected: &Polygon<f64>) {
        let actual_coords: Vec<Coord> = actual.exterior().coords().map(|c| c.clone()).collect();
        for rotation in 0..actual_coords.len() {
            let mut rotated_coords: Vec<Coord> = actual_coords.clone();
            rotated_coords.rotate_right(rotation);
            let rotated_polygon = Polygon::new(LineString::from(rotated_coords.clone()), vec![]);
            if &rotated_polygon == expected {
                return;
            }
        }
        assert!(
            false,
            "Polygons are not equivalent: actual: {:?}, expected: {:?}",
            pretty_print_poly(&actual),
            pretty_print_poly(&expected)
        );
    }

    fn pretty_print_poly(poly: &Polygon<f64>) -> String {
        let coords: Vec<String> = poly
            .exterior()
            .coords()
            .map(|c| format!("{:?}", c.x_y()))
            .collect();
        format!("[{}]", coords.join(","))
    }

    fn pretty_print(geometry: &Vec<Geometry<f64>>) -> String {
        fn poly(poly: &Polygon<f64>) -> String {
            let coords: Vec<String> = poly
                .exterior()
                .coords()
                .map(|c| format!("{:?}", c.x_y()))
                .collect();
            format!("[{}]", coords.join(","))
        }
        let polys: Vec<String> = geometry
            .iter()
            .flat_map(|g| match g {
                Geometry::Polygon(p) => Some(poly(p)),
                _ => None,
            })
            .collect();
        polys.join(",")
    }
}
