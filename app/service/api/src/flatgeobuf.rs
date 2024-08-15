use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use core_geo::Bounds;
use flatgeobuf::{
    geozero::ToGeo, AsyncFeatureIter, FallibleStreamingIterator, FgbReader, HttpFgbReader,
};
use geo::Geometry;
use std::fmt::Display;
use tracing::{instrument, trace};
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
        trace!("Opening reader for FlatGeobuf file: {:?}", self.path);
        let reader = FgbReader::open(filein)?;
        trace!("Opened reader");

        trace!("Selecting bbox, {:?}", bounds);
        let mut features =
            reader.select_bbox(bounds.sw_lon, bounds.sw_lat, bounds.ne_lon, bounds.ne_lat)?;
        trace!("Selected bbox");

        trace!("Iterating over features");
        let mut geoms: Vec<Geometry<f64>> = vec![];
        while let Some(feature) = features.next()? {
            let geom: Geometry<f64> = feature.to_geo()?;
            geoms.push(geom);
        }
        trace!(
            "Finished iterating over features, found {} geoms",
            geoms.len()
        );

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
        trace!("Opening reader for FlatGeobuf URL: {:?}", self.url);
        let reader = HttpFgbReader::open(&self.url.to_string()).await?;
        trace!("Opened reader");

        let mut features = select_bbox(reader, &bounds).await?;
        let geoms = load_geoms(&mut features).await?;

        Ok(geoms)
    }
}

#[instrument(skip(reader, bounds))]
async fn select_bbox(
    reader: HttpFgbReader,
    bounds: &Bounds,
) -> Result<AsyncFeatureIter, Box<dyn std::error::Error>> {
    Ok(reader
        .select_bbox(bounds.sw_lon, bounds.sw_lat, bounds.ne_lon, bounds.ne_lat)
        .await?)
}

#[instrument(skip(features))]
async fn load_geoms(
    features: &mut AsyncFeatureIter,
) -> Result<Vec<Geometry>, Box<dyn std::error::Error>> {
    trace!("Iterating over features");
    let mut geoms: Vec<Geometry<f64>> = vec![];
    let mut report_at = 1;
    while let Some(feature) = features.next().await? {
        let geom: Geometry<f64> = feature.to_geo()?;
        geoms.push(geom);
        if geoms.len() == report_at {
            trace!("Loaded {} geoms", geoms.len());
            report_at *= 2;
        }
    }
    trace!(
        "Finished iterating over features, found {} geoms",
        geoms.len()
    );
    Ok(geoms)
}
