use std::{fs::File, io::BufWriter};

use builder::builder::build;
use clap::Parser;
use flatgeobuf::{FgbWriter, GeometryType};
use geozero::{geojson::GeoJsonWriter, GeozeroGeometry};

/// Extract features from Openstreetmap and convert into single output file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input Openstreetmap `.pbf` path
    #[arg(short, long)]
    pbf: String,

    /// output GeoJSON `.geojson` file
    #[arg(short, long)]
    geojson: Option<String>,

    /// output flatgeobuf `.fgb` file
    #[arg(short, long)]
    fgb: Option<String>,
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let geojson = build(args.pbf).unwrap();

    let geom: geo_types::Geometry<f64> = geojson.try_into().unwrap();
    match &geom {
        geo_types::Geometry::GeometryCollection(c) => {
            println!("GeometryCollection with {} elements", c.0.len());
        }
        _ => {
            println!("some other type");
        }
    }

    if let Some(s) = args.geojson {
        let fout = BufWriter::new(File::create(s)?);
        let mut gout = GeoJsonWriter::new(fout);
        geom.process_geom(&mut gout)?;
    }

    if let Some(s) = args.fgb {
        let mut fgb = FgbWriter::create("all", GeometryType::GeometryCollection)?;
        fgb.add_feature_geom(geom, |_| {})?;
        let mut fout = BufWriter::new(File::create(s)?);
        fgb.write(&mut fout)?;
    }

    Ok(())
}
