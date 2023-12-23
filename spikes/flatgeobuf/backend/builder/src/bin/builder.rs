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
    pbf: Vec<String>,

    /// output GeoJSON `.geojson` file
    #[arg(short, long)]
    geojson: Option<String>,

    /// output flatgeobuf `.fgb` file
    #[arg(short, long)]
    fgb: Option<String>,
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut geoms = vec![];

    for input in args.pbf {
        let geojson = build(input).unwrap();

        let geom: geo_types::Geometry<f64> = geojson.try_into().unwrap();
        match &geom {
            geo_types::Geometry::GeometryCollection(c) => {
                println!("GeometryCollection with {} elements", c.0.len());
            }
            _ => {
                println!("some other type");
            }
        }
        geoms.push(geom);
    }

    if let Some(s) = args.geojson {
        let fout = BufWriter::new(File::create(s)?);
        let mut gout = GeoJsonWriter::new(fout);
        for geom in geoms.iter() {
            geom.process_geom(&mut gout)?;
        }
    }

    if let Some(s) = args.fgb {
        let mut fgb = FgbWriter::create("all", GeometryType::GeometryCollection)?;
        for geom in geoms.iter() {
            fgb.add_feature_geom(geom.clone(), |_| {})?;
        }
        let mut fout = BufWriter::new(File::create(s)?);
        fgb.write(&mut fout)?;
    }

    Ok(())
}
