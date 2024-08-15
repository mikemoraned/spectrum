use clap::Parser;
use pmtiles::{async_reader::AsyncPmTilesReader, cache::HashMapCache, HttpBackend};
use serde_json::Value;
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
    } else {
        println!("Tile not found");
    }

    Ok(())
}
