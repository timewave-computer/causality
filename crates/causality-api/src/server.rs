//! HTTP server for the Causality API

use anyhow::Result;
use crate::config::ApiConfig;

pub struct Server {
    config: ApiConfig,
}

impl Server {
    pub fn new(config: ApiConfig) -> Self {
        Self { config }
    }
    
    pub async fn start(&self) -> Result<()> {
        println!("Starting Causality API server on {}:{}", self.config.host, self.config.port);
        // Minimal implementation for now
        Ok(())
    }
}
