// Provider module
//
// This module provides data providers for external observation sources.

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

use crate::log::{LogStorage, LogStorageError};

/// Error type for provider operations
#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Data error: {0}")]
    Data(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Storage error: {0}")]
    Storage(#[from] LogStorageError),
}

/// Type alias for provider results
pub type Result<T> = std::result::Result<T, ProviderError>;

/// Configuration for an observation provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider type (e.g., "http", "websocket", etc.)
    pub provider_type: String,
    
    /// Unique provider ID
    pub provider_id: String,
    
    /// Provider URL
    pub url: String,
    
    /// Authentication configuration (optional)
    pub auth: Option<ProviderAuth>,
    
    /// Polling interval in seconds (for poll-based providers)
    pub polling_interval: Option<u64>,
    
    /// Whether to validate data
    pub validate_data: bool,
    
    /// Additional configuration options
    pub options: HashMap<String, String>,
}

/// Authentication details for providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAuth {
    /// Authentication type (e.g., "basic", "oauth", "apikey", etc.)
    pub auth_type: String,
    
    /// API key (for API key authentication)
    pub api_key: Option<String>,
    
    /// Username (for basic authentication)
    pub username: Option<String>,
    
    /// Password (for basic authentication)
    pub password: Option<String>,
    
    /// Authentication token (for token-based authentication)
    pub token: Option<String>,
}

/// Status of a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    /// Provider ID
    pub provider_id: String,
    
    /// Whether the provider is connected
    pub is_connected: bool,
    
    /// Connection time
    pub connected_since: Option<u64>,
    
    /// Last error
    pub last_error: Option<String>,
    
    /// Number of successful requests
    pub successful_requests: u64,
    
    /// Number of failed requests
    pub failed_requests: u64,
    
    /// Last update timestamp
    pub last_updated: u64,
}

/// Data returned from a provider
#[derive(Debug, Clone)]
pub struct ProviderData {
    /// The provider ID
    pub provider_id: String,
    /// The timestamp
    pub timestamp: u64,
    /// The data format
    pub format: String,
    /// The data content
    pub content: serde_json::Value,
    /// The data metadata
    pub metadata: HashMap<String, String>,
}

/// Trait for data providers
#[async_trait]
pub trait DataProvider: Send + Sync {
    /// Get the provider ID
    fn get_id(&self) -> &str;
    
    /// Initialize the provider
    fn initialize(&self) -> Result<()>;
    
    /// Start providing data
    fn start(&self) -> Result<()>;
    
    /// Stop providing data
    fn stop(&self) -> Result<()>;
    
    /// Get provider status
    fn get_status(&self) -> Result<ProviderStatus>;
    
    /// Set the target log
    fn set_target_log(&self, log: Arc<dyn LogStorage>) -> Result<()>;
    
    /// Retrieve data from the provider
    async fn retrieve_data(&self, query: &str) -> Result<ProviderData>;
}

/// Trait for provider factories
pub trait ProviderFactory: Send + Sync {
    /// Create a provider from configuration
    fn create_provider(&self, config: ProviderConfig) -> Result<Arc<dyn DataProvider>>;
}

/// Configuration for the observation provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationProviderConfig {
    /// Providers to use
    pub providers: Vec<ProviderConfig>,
}

/// The main observation provider that manages multiple data providers
pub struct ObservationProvider {
    /// Configuration
    config: ObservationProviderConfig,
    
    /// Provider factory
    factory: Arc<dyn ProviderFactory>,
    
    /// Active providers
    providers: RwLock<HashMap<String, Arc<dyn DataProvider>>>,
    
    /// Log storage
    log: Option<Arc<dyn LogStorage>>,
}

impl ObservationProvider {
    /// Create a new observation provider
    pub fn new(config: ObservationProviderConfig, factory: Arc<dyn ProviderFactory>) -> Self {
        ObservationProvider {
            config,
            factory,
            providers: RwLock::new(HashMap::new()),
            log: None,
        }
    }
    
    /// Initialize the provider
    pub fn initialize(&mut self) -> Result<()> {
        // Set up providers
        let mut providers = HashMap::new();
        
        for provider_config in &self.config.providers {
            let provider = self.factory.create_provider(provider_config.clone())?;
            
            // Initialize the provider
            provider.initialize()?;
            
            // Set log storage if available
            if let Some(log) = &self.log {
                provider.set_target_log(log.clone())?;
            }
            
            providers.insert(provider_config.provider_id.clone(), provider);
        }
        
        // Store providers
        let mut providers_lock = self.providers.write().map_err(|e| 
            ProviderError::Internal(format!("Failed to lock providers: {}", e)))?;
            
        *providers_lock = providers;
        
        Ok(())
    }
    
    /// Start all providers
    pub fn start(&self) -> Result<()> {
        let providers = self.providers.read().map_err(|e| 
            ProviderError::Internal(format!("Failed to lock providers: {}", e)))?;
            
        for provider in providers.values() {
            provider.start()?;
        }
        
        Ok(())
    }
    
    /// Stop all providers
    pub fn stop(&self) -> Result<()> {
        let providers = self.providers.read().map_err(|e| 
            ProviderError::Internal(format!("Failed to lock providers: {}", e)))?;
            
        for provider in providers.values() {
            provider.stop()?;
        }
        
        Ok(())
    }
    
    /// Set the log storage
    pub fn set_log_storage(&mut self, log: Arc<dyn LogStorage>) -> Result<()> {
        self.log = Some(log.clone());
        
        let providers = self.providers.read().map_err(|e| 
            ProviderError::Internal(format!("Failed to lock providers: {}", e)))?;
            
        for provider in providers.values() {
            provider.set_target_log(log.clone())?;
        }
        
        Ok(())
    }
    
    /// Get a provider by ID
    pub fn get_provider(&self, provider_id: &str) -> Result<Arc<dyn DataProvider>> {
        let providers = self.providers.read().map_err(|e| 
            ProviderError::Internal(format!("Failed to lock providers: {}", e)))?;
            
        providers.get(provider_id)
            .cloned()
            .ok_or_else(|| ProviderError::Data(format!("Provider not found: {}", provider_id)))
    }
    
    /// Get data from a provider
    pub async fn get_data(&self, provider_id: &str, query: &str) -> Result<ProviderData> {
        let provider = self.get_provider(provider_id)?;
        provider.retrieve_data(query).await
    }
} 