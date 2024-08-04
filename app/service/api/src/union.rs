use geo::{BooleanOps, Geometry, MultiPolygon};

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

    let multi: Vec<MultiPolygon> = polygons
        .into_iter()
        .map(|p| MultiPolygon::new(vec![p]))
        .collect();

    let unioned = multi
        .iter()
        .skip(1)
        .fold(multi[0].clone(), |acc, p| acc.union(p));

    let unioned = unioned.into_iter().map(|p| Geometry::Polygon(p)).collect();

    Ok(unioned)
}
