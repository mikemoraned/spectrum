use std::path::PathBuf;

use async_compression::tokio::bufread::GzipDecoder;
use clap::Parser;
use geo::{Coord, GeometryCollection, LineString, Polygon};
use geojson::FeatureCollection;
use mvt_reader::Reader;
use pmtiles::{async_reader::AsyncPmTilesReader, cache::HashMapCache, HttpBackend};
use serde_json::Value;
use tokio::io::AsyncReadExt;
use url::Url;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// PMTiles URL
    #[arg(long, short)]
    url: Url,

    /// file to dump GeoJSON in
    #[arg(long, short)]
    geojson: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let client = reqwest::Client::new();
    let backend = HttpBackend::try_from(client, args.url)?;
    let cache = HashMapCache::default();
    let reader = AsyncPmTilesReader::try_from_cached_source(backend, cache).await?;

    let metadata = reader.get_metadata().await?;
    let metadata_json: Value = serde_json::from_str(&metadata)?;
    println!("{}", serde_json::to_string_pretty(&metadata_json)?);

    let tile = reader.get_tile(1, 10, 10).await?;
    if let Some(bytes) = tile {
        println!("Tile byte size: {}", bytes.len());
        let data = bytes.to_vec();
        let guessed_mime_type = tree_magic::from_u8(&data);
        println!("Guessed MIME type of bytes: {}", guessed_mime_type);
        if guessed_mime_type == "application/gzip" {
            let mut gzip_reader = GzipDecoder::new(&data[..]);
            let mut decompressed_data = vec![];
            gzip_reader.read_to_end(&mut decompressed_data).await?;
            println!(
                "Guessed MIME type of decompressed: {}",
                tree_magic::from_u8(&decompressed_data)
            );
            let reader = Reader::new(decompressed_data)?;
            let layer_names = reader.get_layer_names()?;
            let mut layer_id = 0;
            for (id, name) in layer_names.iter().enumerate() {
                println!("Layer: {}", name);
                if name == "landcover" {
                    layer_id = id;
                }
            }
            let geometry = reader
                .get_features(layer_id)?
                .into_iter()
                .flat_map(|f| match f.geometry {
                    geo::Geometry::Polygon(p_32) => {
                        let coords: Vec<Coord<f64>> = p_32
                            .exterior()
                            .coords()
                            .map(|c_32| Coord {
                                x: c_32.x as f64,
                                y: c_32.y as f64,
                            })
                            .collect();
                        let poly = Polygon::new(LineString::from(coords), vec![]);
                        Some(geo_types::Geometry::from(poly))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            let geometry_collection = GeometryCollection::from_iter(geometry);
            let feature_collection = FeatureCollection::from(&geometry_collection);
            let geojson_string = feature_collection.to_string();
            println!("{}", geojson_string);
        }
    } else {
        println!("Tile not found");
    }

    Ok(())
}
