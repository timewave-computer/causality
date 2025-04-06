// Purpose: Provides the entry point for the causality-simulation CLI.

mod agent;
mod cli;
mod controller;
mod observer;
mod replay;
mod runner;
mod scenario;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing with a reasonable default configuration
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "causality_simulation=info,causality_engine=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Run the CLI
    cli::run().await
} 