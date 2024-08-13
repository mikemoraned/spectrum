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
