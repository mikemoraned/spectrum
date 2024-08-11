use std::{path::PathBuf, sync::Arc};

use api::{
    env::{load_public, load_secret},
    regions::{regions, route, Regions},
    state::AppState,
    tracing::{init_opentelemetry_from_environment, init_safe_default_from_environment},
};
use axum::{
    http::{Method, StatusCode},
    routing::get,
    Router,
};
use clap::Parser;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
};
use tracing::info;
use url::Url;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to FlatGeobuf file
    #[arg(long, short)]
    fgb: PathBuf,

    /// enable opentelemetry
    #[arg(long)]
    opentelemetry: bool,
}

#[tracing::instrument()]
async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.opentelemetry {
        match init_opentelemetry_from_environment("spectrum") {
            Ok(_) => {
                info!("Opentelemetry initialised")
            }
            Err(e) => {
                println!(
                    "Failed to initialise Opentelemetry ('{:?}'), falling back to default",
                    e
                );
                init_safe_default_from_environment()?;
            }
        }
    } else {
        init_safe_default_from_environment()?;
    }

    let stadia_maps_api_key = load_secret("STADIA_MAPS_API_KEY")?;
    let stadia_maps_endpoint_base = Url::parse(&load_public("STADIA_MAPS_ENDPOINT_BASE")?)?;

    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        // allow requests from any origin
        .allow_origin(Any);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/v2/regions", get(regions))
        .route("/v2/route", get(route))
        .route("/health", get(health))
        .layer(cors)
        .layer(CompressionLayer::new())
        .with_state(AppState {
            regions: Arc::new(Regions::from_flatgeobuf(
                &args.fgb,
                &stadia_maps_api_key,
                &stadia_maps_endpoint_base,
            )?),
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
