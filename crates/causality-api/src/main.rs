// Causality API server for cross-chain deployment coordination
use anyhow::Result;
use causality_api::{ApiConfig, ExecutionSession, server::start_server};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    // Create configuration
    let config = ApiConfig::default();
    
    // Create session storage
    let sessions = Arc::new(RwLock::new(HashMap::<String, ExecutionSession>::new()));
    
    // Start the API server
    start_server(config, sessions).await?;
    
    Ok(())
} 