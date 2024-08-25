use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use async_compression::tokio::bufread::GzipDecoder;
use clap::Parser;
use geo::{Coord, GeometryCollection, LineString, MultiLineString, MultiPolygon, Polygon};
use geojson::FeatureCollection;
use mvt_reader::Reader;
use pmtiles::{async_reader::AsyncPmTilesReader, cache::HashMapCache, HttpBackend};
use serde_json::Value;
use tile_grid::{tms, Xyz};
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

    // let metadata = reader.get_metadata().await?;
    // let metadata_json: Value = serde_json::from_str(&metadata)?;
    // println!("{}", serde_json::to_string_pretty(&metadata_json)?);

    let tile_json = reader.parse_tilejson(vec![]).await?;
    println!("{:?}", tile_json);

    // let (lon, lat) = (-3.188267, 55.953251); // edinburgh
    let tms = tms().lookup("WebMercatorQuad")?;
    // let tile = tms.tile(lon, lat, 4)?;
    let tile = Xyz::new(251, 159, 9); // should cover Edinburgh
    let tile_bounds = tms.bounds(&tile)?;
    println!("Tile for Edinburgh: {:?}, bbox: {:?}", tile, tile_bounds);
    let tile_extent = 4096.0f32;

    let tile = reader.get_tile(tile.z, tile.x, tile.y).await?;
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
            let mvt_reader = Reader::new(decompressed_data)?;
            let layer_names = mvt_reader.get_layer_names()?;
            let mut layer_id = 0;
            for (id, name) in layer_names.iter().enumerate() {
                println!("Layer: {}", name);
                if name == "roads" {
                    layer_id = id;
                }
            }
            fn convert_linestring_f32_to_f64(
                l_32: &LineString<f32>,
                tile_extent: f32,
                bbox: &tile_grid::BoundingBox,
            ) -> geo_types::LineString<f64> {
                let coords: Vec<Coord<f64>> = l_32
                    .coords()
                    .map(|c_32| {
                        // let before = c_32;
                        let width = (bbox.right - bbox.left).abs();
                        let height = (bbox.bottom - bbox.top).abs();
                        let after = Coord {
                            x: (((1.0f32 * (c_32.x / tile_extent)) as f64) * width) + bbox.left,
                            y: bbox.top - (((1.0f32 * (c_32.y / tile_extent)) as f64) * height),
                        };
                        // println!("{:?} -> {:?}", before, after);
                        after
                    })
                    .collect();
                LineString::from(coords)
            }
            fn convert_poly_f32_to_f64(
                p_32: Polygon<f32>,
                tile_extent: f32,
                bbox: &tile_grid::BoundingBox,
            ) -> geo_types::Polygon<f64> {
                let linestring =
                    convert_linestring_f32_to_f64(&p_32.exterior(), tile_extent, &bbox);
                Polygon::new(linestring, vec![])
            }
            let geometry = mvt_reader
                .get_features(layer_id)?
                .into_iter()
                .flat_map(|f| match f.geometry {
                    geo::Geometry::Point(_) => {
                        println!("Point");
                        None
                    }
                    geo::Geometry::Line(_) => {
                        println!("Line");
                        None
                    }
                    geo::Geometry::LineString(_) => {
                        println!("LineString");
                        None
                    }
                    geo::Geometry::Polygon(p_32) => Some(geo_types::Geometry::from(
                        convert_poly_f32_to_f64(p_32, tile_extent, &tile_bounds),
                    )),
                    geo::Geometry::MultiPoint(_) => {
                        println!("MultiPoint");
                        None
                    }
                    geo::Geometry::MultiLineString(l_32) => {
                        let lines: Vec<_> = l_32
                            .into_iter()
                            .map(|l| convert_linestring_f32_to_f64(&l, tile_extent, &tile_bounds))
                            .collect();
                        Some(geo_types::Geometry::from(MultiLineString::from_iter(lines)))
                    }
                    geo::Geometry::MultiPolygon(p_32) => {
                        let polys: Vec<_> = p_32
                            .into_iter()
                            .map(|p| convert_poly_f32_to_f64(p, tile_extent, &tile_bounds))
                            .collect();
                        Some(geo_types::Geometry::from(MultiPolygon::from_iter(polys)))
                    }
                    geo::Geometry::GeometryCollection(_) => {
                        println!("GeometryCollection");
                        None
                    }
                    geo::Geometry::Rect(_) => {
                        println!("Rect");
                        None
                    }
                    geo::Geometry::Triangle(_) => {
                        println!("Triangle");
                        None
                    }
                })
                .collect::<Vec<_>>();
            let geometry_collection = GeometryCollection::from_iter(geometry);
            let feature_collection = FeatureCollection::from(&geometry_collection);
            let file = File::create(args.geojson)?;
            let mut writer = BufWriter::new(file);
            serde_json::to_writer(&mut writer, &feature_collection)?;
            writer.flush()?;
        }
    } else {
        println!("Tile not found");
    }

    Ok(())
}
