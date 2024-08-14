use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use core_geo::Bounds;
use flatgeobuf::{geozero::ToGeo, FallibleStreamingIterator, FgbReader, HttpFgbReader};
use geo::Geometry;
use std::fmt::Display;
use tracing::{debug, instrument};
use url::Url;

pub enum FgbSource {
    File(FgbFileSource),
    Url(FgbUrlSource),
}

impl Display for FgbSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FgbSource::File(source) => write!(f, "File: {:?}", source.path),
            FgbSource::Url(source) => write!(f, "URL: {:?}", source.url.to_string()),
        }
    }
}

impl FgbSource {
    pub async fn load(
        &self,
        bounds: &Bounds,
    ) -> Result<Vec<Geometry<f64>>, Box<dyn std::error::Error>> {
        match self {
            FgbSource::File(source) => source.load(bounds),
            FgbSource::Url(source) => source.load(bounds).await,
        }
    }

    pub fn from_path(path: &Path) -> Self {
        FgbSource::File(FgbFileSource {
            path: path.to_path_buf(),
        })
    }

    pub fn from_url(url: &Url) -> Self {
        FgbSource::Url(FgbUrlSource { url: url.clone() })
    }
}

pub struct FgbFileSource {
    path: PathBuf,
}

impl FgbFileSource {
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

pub struct FgbUrlSource {
    url: Url,
}

impl FgbUrlSource {
    #[instrument(skip(self))]
    async fn load(
        &self,
        bounds: &Bounds,
    ) -> Result<Vec<Geometry<f64>>, Box<dyn std::error::Error>> {
        let mut features = HttpFgbReader::open(&self.url.to_string())
            .await?
            .select_bbox(bounds.sw_lon, bounds.sw_lat, bounds.ne_lon, bounds.ne_lat)
            .await?;

        let mut geoms: Vec<Geometry<f64>> = vec![];
        while let Some(feature) = features.next().await? {
            let geom: Geometry<f64> = feature.to_geo()?;
            geoms.push(geom);
        }

        Ok(geoms)
    }
}
