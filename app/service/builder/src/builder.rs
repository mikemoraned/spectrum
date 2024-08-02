use std::path::Path;

use geo_types::{Geometry, GeometryCollection};

pub fn extract_regions(osmpbf_path: &Path) -> Result<GeometryCollection<f64>, ()> {
    Ok(GeometryCollection::from_iter(Vec::<Geometry<f64>>::new()))
}
