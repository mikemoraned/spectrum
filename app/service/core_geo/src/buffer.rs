use cavalier_contours::polyline::{
    PlineCreation, PlineOffsetOptions, PlineSource, PlineVertex, Polyline,
};
use geo::{Coord, CoordsIter, LineString, MultiPolygon, Polygon};
use geo_validity_check::Valid;
use tracing::{debug, trace};

pub fn buffer_polygon(poly: &Polygon<f64>, distance: f64) -> MultiPolygon<f64> {
    let coords_iter = poly.exterior().coords_iter();
    let vertex_iter = coords_iter.map(|c| PlineVertex::new(c.x, c.y, 0.0));
    let polyline = Polyline::from_iter(vertex_iter, true);

    let offsetted = polyline.parallel_offset_opt(
        -1.0 * distance,
        &PlineOffsetOptions {
            handle_self_intersects: true,
            ..Default::default()
        },
    );
    // let offsetted = vec![polyline.clone()];

    trace!("Number of offsetted polylines: {}", offsetted.len());

    fn from_polyline(polyline: Polyline) -> Polygon {
        let mut coords: Vec<Coord> = polyline
            .iter_vertexes()
            .map(|v| Coord::from((v.x, v.y)))
            .collect();
        coords.push(coords[0]);
        let poly = Polygon::new(LineString::from(coords), vec![]);
        if poly.is_valid() {
            debug!("poly is valid");
        } else {
            debug!("poly is invalid {:?}", poly.explain_invalidity());
        }
        poly
    }

    let polygons = offsetted.into_iter().map(from_polyline).collect::<Vec<_>>();
    MultiPolygon::new(polygons)
}

pub fn buffer_multi_polygon(poly: &MultiPolygon<f64>, distance: f64) -> MultiPolygon<f64> {
    let buffered = poly
        .iter()
        .flat_map(|p| {
            let buffered = buffer_polygon(p, distance);
            let buffered_polygons: Vec<Polygon> = buffered.into_iter().collect();
            buffered_polygons
        })
        .collect::<Vec<_>>();
    MultiPolygon::new(buffered)
}
