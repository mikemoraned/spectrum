use std::{fs::File, io::BufWriter, path::PathBuf};

use builder::builder::extract_regions;
use clap::Parser;
use flatgeobuf::{FgbWriter, GeometryType};
use geozero::{geojson::GeoJsonWriter, GeozeroGeometry};
use tracing::{debug, info, trace};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// Extract features from Openstreetmap and convert into single output file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input `.osm.pbf` file(s)
    #[arg(short, long)]
    pbf: Vec<PathBuf>,

    /// output GeoJSON `.geojson` file
    #[arg(short, long)]
    geojson: Option<PathBuf>,

    /// output flatgeobuf `.fgb` file
    #[arg(short, long)]
    fgb: Option<PathBuf>,
}

fn setup_tracing_and_logging(fmt_filter: EnvFilter) {
    let fmt_layer = fmt::layer().with_filter(fmt_filter);
    tracing_subscriber::registry()
        .with(fmt_layer)
        .try_init()
        .unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_tracing_and_logging(EnvFilter::from_default_env());

    let args = Args::parse();
    debug!("{:?}", args);

    let mut geoms = vec![];

    info!("processing input files: {:?}", args.pbf);
    for input in args.pbf {
        let collection = extract_regions(&input).expect("failed when extracting regions");
        geoms.push(geo::geometry::Geometry::GeometryCollection(collection));
    }

    if let Some(s) = args.geojson {
        info!("writing geojson to {:?}", s);
        let fout = BufWriter::new(File::create(s)?);
        let mut gout = GeoJsonWriter::new(fout);
        for geom in geoms.iter() {
            geom.process_geom(&mut gout)?;
        }
    }

    if let Some(s) = args.fgb {
        info!("writing flatgeobuf to {:?}", s);
        let mut fgb = FgbWriter::create("all", GeometryType::Polygon)?;
        let mut geom_added_count = 0;
        debug!("adding geoms");
        for geom in geoms.iter() {
            match geom {
                geo::geometry::Geometry::GeometryCollection(c) => {
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
        }
        debug!("added {} geoms", geom_added_count);

        let mut fout = BufWriter::new(File::create(s)?);
        fgb.write(&mut fout)?;
    }

    Ok(())
}
