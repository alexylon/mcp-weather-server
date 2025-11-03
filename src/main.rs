mod constants;
mod formatters;
mod models;
mod service;

use anyhow::Result;
use rmcp::ServiceExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use service::Weather;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mcp_rust_weather=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting MCP weather server");

    let weather = Weather::new()?;
    let server = weather.serve(rmcp::transport::stdio()).await?;
    server.waiting().await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}
