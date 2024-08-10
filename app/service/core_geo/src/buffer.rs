use geo::MultiPolygon;

pub fn buffer_multi_polygon(poly: &MultiPolygon<f64>, distance: f64) -> MultiPolygon<f64> {
    poly.clone()
}
