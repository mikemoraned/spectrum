use cavalier_contours::polyline::{PlineCreation, PlineSource, PlineVertex, Polyline};
use geo::{Coord, CoordsIter, LineString, MultiPolygon, Polygon};

pub fn buffer_polygon(poly: &Polygon<f64>, distance: f64) -> MultiPolygon<f64> {
    let coords_iter = poly.exterior().coords_iter();
    let vertex_iter = coords_iter.map(|c| PlineVertex::new(c.x, c.y, 0.0));
    let polyline = Polyline::from_iter(vertex_iter, true);

    let offsetted = polyline.parallel_offset(-1.0 * distance);
    // let offsetted = vec![polyline.clone()];

    fn from_polyline(polyline: Polyline) -> Polygon {
        let coords: Vec<Coord> = polyline
            .iter_vertexes()
            .map(|v| Coord::from((v.x, v.y)))
            .collect();
        Polygon::new(LineString::from(coords), vec![])
    }

    let polygons = offsetted.into_iter().map(from_polyline).collect::<Vec<_>>();
    MultiPolygon::new(polygons)
}
