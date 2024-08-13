use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use core_geo::Bounds;
use flatgeobuf::{geozero::ToGeo, FallibleStreamingIterator, FgbReader};
use geo::Geometry;
use tracing::{debug, instrument};

pub trait FgbSource {
    fn load(&self, bounds: &Bounds) -> Result<Vec<Geometry<f64>>, Box<dyn std::error::Error>>;
}

pub struct FgbFileSource {
    path: PathBuf,
}

impl FgbFileSource {
    pub fn from_path(path: &Path) -> Self {
        FgbFileSource {
            path: path.to_path_buf(),
        }
    }
}

impl FgbSource for FgbFileSource {
    #[instrument(skip(self))]
    fn load(&self, bounds: &Bounds) -> Result<Vec<Geometry<f64>>, Box<dyn std::error::Error>> {
        let filein = BufReader::new(File::open(self.path.clone())?);
        let reader = FgbReader::open(filein)?;
        debug!("Opened FlatGeobuf file: {:?}", self.path);

        let mut features =
            reader.select_bbox(bounds.sw_lon, bounds.sw_lat, bounds.ne_lon, bounds.ne_lat)?;

        let mut geoms: Vec<Geometry<f64>> = vec![];
        while let Some(feature) = features.next()? {
            let geom: Geometry<f64> = feature.to_geo()?;
            geoms.push(geom);
        }

        Ok(geoms)
    }
}
