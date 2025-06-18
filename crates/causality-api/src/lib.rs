//! Causality API
//!
//! This crate provides the API interface for the Causality system, including
//! HTTP server endpoints, client libraries, and blockchain integrations.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

// Core modules
pub mod client;
pub mod config;
pub mod coprocessor;
pub mod handlers;
pub mod server;
pub mod session;
pub mod traits;
pub mod types;

// Blockchain integration modules
pub mod blockchain;
pub mod valence_integration;

// Re-exports for convenience
pub use client::CausalityClient;
pub use config::ApiConfig;
pub use coprocessor::{CoprocessorClient, CoprocessorService};
pub use session::{ExecutionSession};
pub use types::*;

// Re-export blockchain clients
pub use blockchain::{EthereumClientWrapper, NeutronClientWrapper, CoprocessorClientWrapper};

/// Main API service
#[derive(Debug)]
pub struct CausalityApi {
    /// Configuration
    config: ApiConfig,
    
    /// Session manager
    sessions: Arc<RwLock<HashMap<String, ExecutionSession>>>,
}

impl CausalityApi {
    /// Create a new API instance
    pub fn new(config: ApiConfig) -> Result<Self> {
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        
        Ok(Self {
            config,
            sessions,
        })
    }
    
    /// Get the configuration
    pub fn config(&self) -> &ApiConfig {
        &self.config
    }
    
    /// Get the session manager
    pub fn sessions(&self) -> Arc<RwLock<HashMap<String, ExecutionSession>>> {
        Arc::clone(&self.sessions)
    }
} 