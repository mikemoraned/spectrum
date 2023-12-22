use std::fs::File;
use std::io::{BufReader, BufWriter};

use flatgeobuf::*;
use geozero::geojson::GeoJsonReader;
use geozero::GeozeroDatasource;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut fgb = FgbWriter::create("find", GeometryType::MultiPolygon)?;
    let mut fin = BufReader::new(File::open("data/find.json")?);
    let mut reader = GeoJsonReader(&mut fin);
    reader.process(&mut fgb)?;
    let mut fout = BufWriter::new(File::create("data/find.fgb")?);
    fgb.write(&mut fout)?;

    Ok(())
}
