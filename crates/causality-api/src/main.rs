//! Causality API Server
//!
//! HTTP API server for the Causality system

use anyhow::Result;
use causality_api::{config::ApiConfig, server::Server};

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = ApiConfig::default();
    
    // Create and start server
    let server = Server::new(config);
    server.start().await?;
    
    Ok(())
}
