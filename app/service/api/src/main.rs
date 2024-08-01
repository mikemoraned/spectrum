use api::{
    regions::regions,
    tracing::{init_opentelemetry_from_environment, init_safe_default_from_environment},
};
use axum::{http::StatusCode, routing::get, Router};
use clap::Parser;
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
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

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/v1/regions", get(regions))
        .route("/health", get(health));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
