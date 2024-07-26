use std::fs::File;

use clap::{command, Parser};
use flatgeobuf::FgbReader;

/// show info about a Flatgeobuf file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input flatgeobuf `.fgb` file
    #[arg(short, long)]
    fgb: String,
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut filein = File::open(args.fgb)?;
    let fgb = FgbReader::open(&mut filein)?;

    let header = fgb.header();
    println!("{:?}", header);
    Ok(())
}
