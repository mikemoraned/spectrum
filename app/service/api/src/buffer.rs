use cavalier_contours::polyline::{PlineCreation, PlineSource, PlineVertex, Polyline};
use geo::{Coord, LineString, MultiPolygon, Polygon};

pub fn buffer_linestring(linestring: &LineString<f64>, distance: f64) -> MultiPolygon {
    let coords_iter = linestring.coords().into_iter();
    let vertex_iter = coords_iter.map(|c| PlineVertex::new(c.x, c.y, 0.0));
    let open_polyline = Polyline::from_iter(vertex_iter, false);

    let offsetted = open_polyline.parallel_offset(-1.0 * distance);

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
