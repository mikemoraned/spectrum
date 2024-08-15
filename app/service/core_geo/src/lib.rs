use geo::Rect;
use serde::Deserialize;

pub mod buffer;
pub mod union;

#[derive(Deserialize, Debug)]
pub struct Bounds {
    pub sw_lat: f64,
    pub sw_lon: f64,
    pub ne_lat: f64,
    pub ne_lon: f64,
}

impl From<Rect<f64>> for Bounds {
    fn from(rect: Rect<f64>) -> Self {
        // in Rect, x axis: corresponds to longitude, y axis: corresponds to latitude
        // so, min of x is the most western value, max of x is the most eastern value
        // and, min of y is the most southern value, max of y is the most northern value

        // so, min is the south-western corner, max is the north-eastern corner
        let sw = rect.min();
        let ne = rect.max();

        Bounds {
            sw_lat: sw.y,
            sw_lon: sw.x,
            ne_lat: ne.y,
            ne_lon: ne.x,
        }
    }
}

#[cfg(test)]
mod test {
    use geo::coord;

    use super::*;

    #[test]
    fn test_bounds_from_rect() {
        let rect = Rect::new(coord! { x: 5., y: 7. }, coord! { x: 15., y: 17. });
        let bounds = Bounds::from(rect);
        assert_eq!(bounds.sw_lat, 7.);
        assert_eq!(bounds.sw_lon, 5.);
        assert_eq!(bounds.ne_lat, 17.);
        assert_eq!(bounds.ne_lon, 15.);
    }
}
