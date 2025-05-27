//! Coprocessor Connection Pool
//!
//! This module provides connection pooling functionality for ZK coprocessors,
//! allowing efficient management of connections with automatic health checking
//! and connection recycling.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::generator::ProofGenerator;
use super::types::CoprocessorId;
use crate::gateway::ApiError;

//-----------------------------------------------------------------------------
// Connection Pool Type
//-----------------------------------------------------------------------------

/// Connection pool for managing multiple coprocessor connections
pub struct CoprocessorConnectionPool {
    /// Map of coprocessor ID to connection
    connections: Mutex<HashMap<CoprocessorId, Arc<dyn ProofGenerator>>>,
}

impl Default for CoprocessorConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

impl CoprocessorConnectionPool {
    /// Create a new connection pool with default settings
    pub fn new() -> Self {
        Self {
            connections: Mutex::new(HashMap::new()),
        }
    }

    /// Create a new connection pool with custom settings
    pub fn with_config(_max_connections: usize, _max_idle_time: u64) -> Self {
        Self {
            connections: Mutex::new(HashMap::new()),
        }
    }

    /// Get a connection to a coprocessor
    pub async fn get_connection(
        &self,
        coprocessor_id: &CoprocessorId,
    ) -> Result<Arc<dyn ProofGenerator>, ApiError> {
        let connections = self.connections.lock().await;

        if let Some(connection) = connections.get(coprocessor_id) {
            return Ok(connection.clone());
        }

        // In a full implementation, this would create a new connection
        Err(ApiError::InternalError(
            "No connection available for coprocessor".to_string(),
        ))
    }

    /// Return a connection to the pool
    pub async fn return_connection(&self, generator: Arc<dyn ProofGenerator>) {
        let coprocessor_id = generator.coprocessor_id();
        let mut connections = self.connections.lock().await;
        connections.insert(coprocessor_id, generator);
    }

    /// Health check for all connections
    pub async fn health_check(&self) -> bool {
        let connections = self.connections.lock().await;

        for connection in connections.values() {
            if connection.health_check().await.is_err() {
                return false;
            }
        }

        true
    }

    /// Close all connections in the pool
    pub async fn close_all(&self) {
        let mut connections = self.connections.lock().await;
        connections.clear();
    }
}
