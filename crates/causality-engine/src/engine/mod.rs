// Engine implementation for Causality
//
// This module provides the main Engine struct for Causality.

use std::sync::Arc;
use anyhow::Result;

use crate::log::LogStorage;
use crate::log::memory_storage::MemoryLogStorage;
use crate::config::EngineConfig;

/// The main Causality Engine
pub struct Engine {
    config: EngineConfig,
    log_storage: Arc<dyn LogStorage>,
}

impl Engine {
    /// Create a new Engine with default configuration
    pub fn new(log_storage: Arc<dyn LogStorage>) -> Result<Self> {
        let config = EngineConfig::default();
        Ok(Self {
            config,
            log_storage,
        })
    }
    
    /// Create a new Engine with custom configuration
    pub fn with_config(
        config: EngineConfig, 
        _storage: Arc<dyn std::marker::Send + std::marker::Sync>, 
        log_storage: Arc<dyn LogStorage>
    ) -> Result<Self> {
        Ok(Self {
            config,
            log_storage,
        })
    }
    
    /// Get the log storage
    pub fn log(&self) -> Arc<dyn LogStorage> {
        self.log_storage.clone()
    }
    
    /// Get the engine configuration
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }
    
    /// Initialize the engine (placeholder for future implementation)
    pub async fn initialize(&self) -> Result<()> {
        // Placeholder for initialization logic
        Ok(())
    }
    
    /// Shut down the engine (placeholder for future implementation)
    pub async fn shutdown(&self) -> Result<()> {
        // Placeholder for shutdown logic
        Ok(())
    }
    
    /// Execute a value in the given context (placeholder for testing)
    pub fn execute_sync<T, C>(&self, _value: &T, _context: &C) -> Result<T> 
    where T: Clone
    {
        // This is just a placeholder implementation for testing
        Ok(_value.clone())
    }
}

impl Default for Engine {
    fn default() -> Self {
        // Create a memory log storage for the default engine
        let log_storage = Arc::new(MemoryLogStorage::new());
        Self::new(log_storage).expect("Failed to create default engine")
    }
}

// Tests for the Engine
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_engine_creation() {
        let log_storage = Arc::new(MemoryLogStorage::new());
        let engine = Engine::new(log_storage).unwrap();
        assert!(engine.config().invocation_timeout_ms > 0);
    }
    
    #[test]
    fn test_engine_default() {
        let engine = Engine::default();
        assert!(engine.config().invocation_timeout_ms > 0);
    }
} 