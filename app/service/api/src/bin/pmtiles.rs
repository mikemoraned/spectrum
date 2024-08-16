use async_compression::tokio::bufread::GzipDecoder;
use clap::Parser;
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
            for name in layer_names {
                println!("Layer: {}", name);
            }
        }
    } else {
        println!("Tile not found");
    }

    Ok(())
}
