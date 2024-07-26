use std::{fs::File, io::BufWriter};

use builder::builder::build;
use clap::Parser;
use flatgeobuf::{FgbWriter, GeometryType};
use geozero::{geojson::GeoJsonWriter, GeozeroGeometry};
use tracing::{debug, info, trace, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// Extract features from Openstreetmap and convert into single output file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input Openstreetmap `.pbf` path
    #[arg(short, long)]
    pbf: Vec<String>,

    /// output GeoJSON `.geojson` file
    #[arg(short, long)]
    geojson: Option<String>,

    /// output flatgeobuf `.fgb` file
    #[arg(short, long)]
    fgb: Option<String>,
}

fn setup_tracing_and_logging(fmt_filter: EnvFilter) {
    let fmt_layer = fmt::layer().with_filter(fmt_filter);
    tracing_subscriber::registry()
        .with(fmt_layer)
        .try_init()
        .unwrap();
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    setup_tracing_and_logging(EnvFilter::from_default_env());

    let args = Args::parse();
    let mut geoms = vec![];

    info!("processing input files");
    for input in args.pbf {
        let geojson = build(input).unwrap();

        let geom: geo_types::Geometry<f64> = geojson.try_into().unwrap();
        match &geom {
            geo_types::Geometry::GeometryCollection(c) => {
                debug!("GeometryCollection with {} elements", c.0.len());
            }
            _ => {
                warn!("some other type");
            }
        }
        geoms.push(geom);
    }

    if let Some(s) = args.geojson {
        info!("writing geojson to {}", s);
        let fout = BufWriter::new(File::create(s)?);
        let mut gout = GeoJsonWriter::new(fout);
        for geom in geoms.iter() {
            geom.process_geom(&mut gout)?;
        }
    }

    if let Some(s) = args.fgb {
        info!("writing flatgeobuf to {}", s);
        let mut fgb = FgbWriter::create("all", GeometryType::Polygon)?;
        let mut geom_added_count = 0;
        debug!("adding geoms");
        for geom in geoms.iter() {
            match geom {
                geo_types::Geometry::GeometryCollection(c) => {
                    for geom in c.into_iter() {
                        trace!("adding geom, {:?}", geom);
                        fgb.add_feature_geom(geom.clone(), |_| {})?;
                        geom_added_count += 1;
                    }
                }
                _ => {
                    todo!("only handling GeometryCollection for now");
                }
            }
            // trace!("adding geom, {:?}", geom);
            // fgb.add_feature_geom(geom.clone(), |_| {})?;
            // geom_added_count += 1;
        }
        debug!("added {} geoms", geom_added_count);

        let mut fout = BufWriter::new(File::create(s)?);
        fgb.write(&mut fout)?;
    }

    Ok(())
}
