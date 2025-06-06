//! Causality API
//!
//! High-level API server for interacting with the Causality system.
//! Provides REST endpoints for compilation, execution, debugging, and session management.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod config;
pub mod traits;
pub mod types;
pub mod session;
pub mod handlers;
pub mod client;
pub mod server;
pub mod coprocessor;
// pub mod effects;

// Re-export key types
pub use server::*;
pub use types::*;
pub use client::*;
pub use session::*;
pub use config::*;
pub use coprocessor::*;

/// Main API service for Causality
#[derive(Debug)]
pub struct CausalityApi {
    /// Execution sessions
    sessions: Arc<RwLock<HashMap<String, ExecutionSession>>>,
    
    /// Server configuration
    config: ApiConfig,
    
    /// Coprocessor service
    coprocessor: Option<Arc<tokio::sync::Mutex<CoprocessorService>>>,
}

impl CausalityApi {
    /// Create a new API instance
    pub fn new(config: ApiConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
            coprocessor: None,
        }
    }
    
    /// Create a new API instance with coprocessor integration
    pub fn with_coprocessor(config: ApiConfig, coprocessor_config: CoprocessorConfig) -> Result<Self> {
        let coprocessor = CoprocessorService::new(coprocessor_config)?;
        
        Ok(Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
            coprocessor: Some(Arc::new(tokio::sync::Mutex::new(coprocessor))),
        })
    }
    
    /// Start the API server
    pub async fn start(&self) -> Result<()> {
        server::start_server(self.config.clone(), self.sessions.clone()).await
    }
} 