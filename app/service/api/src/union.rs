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
