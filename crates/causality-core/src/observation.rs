// Observation system for consensus and coordination
//
// This module provides functionality for observations to index external chains,
// extract facts, and share them with other system components.

use std::fmt;
use std::sync::Arc;
use thiserror::Error;
use serde::{Serialize, Deserialize};

pub mod indexer;
pub mod extraction;
pub mod proxy;
pub mod reconstruction;
pub mod provider;

/// Result type for observation operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during observation operations
#[derive(Error, Debug)]
pub enum Error {
    /// Error connecting to an external source
    #[error("Connection error: {0}")]
    Connection(String),
    
    /// Error extracting facts
    #[error("Extraction error: {0}")]
    Extraction(String),
    
    /// Error with observation configuration
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Error processing data
    #[error("Data error: {0}")]
    Data(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// Log system error
    #[error("Log error: {0}")]
    Log(String),
}

/// Configuration for an observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationConfig {
    /// Unique identifier for the observation
    pub id: String,
    /// Configuration for the proxy
    pub proxy: proxy::ProxyConfig,
    /// Configuration for reconstruction
    pub reconstruction: reconstruction::ReconstructionConfig,
    /// Configuration for data providers
    pub providers: Vec<provider::ProviderConfig>,
}

impl Default for ObservationConfig {
    fn default() -> Self {
        ObservationConfig {
            id: "default".to_string(),
            proxy: proxy::ProxyConfig::default(),
            reconstruction: reconstruction::ReconstructionConfig::default(),
            providers: Vec::new(),
        }
    }
}

/// An observation system that can index, extract, and reconstruct data
pub struct Observation {
    /// Configuration for the observation
    config: ObservationConfig,
    /// The proxy for interacting with external chains
    proxy: Arc<proxy::ObservationProxy>,
    /// Factory for creating reconstructors
    reconstructor_factory: Arc<reconstruction::ReconstructorFactory>,
    /// Registry for managing reconstructors
    reconstructor_registry: Arc<reconstruction::ReconstructorRegistry>,
    /// Factory for creating data providers
    provider_factory: Arc<provider::ProviderFactory>,
    /// Registry for managing data providers
    provider_registry: Arc<provider::ProviderRegistry>,
}

impl Observation {
    /// Create a new observation
    pub fn new(config: ObservationConfig) -> Result<Self> {
        // Create proxy
        let proxy = Arc::new(proxy::ObservationProxy::new(config.proxy.clone())?);
        
        // Create reconstructor components
        let reconstructor_factory = Arc::new(
            reconstruction::ReconstructorFactory::new(
                config.reconstruction.clone()
            )
        );
        
        let reconstructor_registry = Arc::new(
            reconstruction::ReconstructorRegistry::new(
                reconstructor_factory.clone()
            )
        );
        
        // Create provider components
        let provider_factory = Arc::new(provider::ProviderFactory::new());
        
        // Register HTTP provider constructor
        provider_factory.register_constructor("http", |config| {
            let provider = provider::HttpProvider::new(config)?;
            Ok(Arc::new(provider) as Arc<dyn provider::DataProvider>)
        })?;
        
        let provider_registry = Arc::new(
            provider::ProviderRegistry::new(provider_factory.clone())
        );
        
        Ok(Observation {
            config,
            proxy,
            reconstructor_factory,
            reconstructor_registry,
            provider_factory,
            provider_registry,
        })
    }
    
    /// Initialize the observation
    pub async fn initialize(&self) -> Result<()> {
        // Initialize proxy
        self.proxy.initialize().await?;
        
        // Register event handler for proxy events
        self.proxy.add_event_handler(Arc::new(proxy::LoggingEventHandler))?;
        
        // Initialize providers
        for provider_config in &self.config.providers {
            self.provider_registry.create_and_register(provider_config.clone())?;
        }
        
        // Initialize all providers
        self.provider_registry.initialize_all().await?;
        
        // Set up fact channel and connect to reconstructor
        let fact_receiver = self.proxy.take_fact_receiver()?;
        
        // Create reconstructor for the domain
        self.reconstructor_registry.create_and_register(
            &self.config.reconstruction.domain,
            fact_receiver,
        )?;
        
        Ok(())
    }
    
    /// Start the observation
    pub async fn start(&self) -> Result<()> {
        // Connect all providers
        self.provider_registry.connect_all().await?;
        
        // Start proxy
        self.proxy.start().await?;
        
        // Start all reconstructors
        self.reconstructor_registry.start_all().await?;
        
        Ok(())
    }
    
    /// Stop the observation
    pub async fn stop(&self) -> Result<()> {
        // Stop proxy
        self.proxy.stop().await?;
        
        // Stop all reconstructors
        self.reconstructor_registry.stop_all().await?;
        
        // Disconnect all providers
        self.provider_registry.disconnect_all().await?;
        
        Ok(())
    }
    
    /// Get the proxy
    pub fn proxy(&self) -> Arc<proxy::ObservationProxy> {
        self.proxy.clone()
    }
    
    /// Get the reconstructor registry
    pub fn reconstructor_registry(&self) -> Arc<reconstruction::ReconstructorRegistry> {
        self.reconstructor_registry.clone()
    }
    
    /// Get the provider registry
    pub fn provider_registry(&self) -> Arc<provider::ProviderRegistry> {
        self.provider_registry.clone()
    }
    
    /// Add a rule to extract facts from external data
    pub fn add_extraction_rule(&self, rule: extraction::ExtractionRule) -> Result<()> {
        self.proxy.add_rule(rule)
    }
    
    /// Load extraction rules from a TOML string
    pub fn load_extraction_rules_from_toml(&self, toml_str: &str) -> Result<()> {
        self.proxy.load_rules_from_toml(toml_str)
    }
}

/// Factory function to create an observation
pub fn create_observation(config: ObservationConfig) -> Result<Observation> {
    Observation::new(config)
}

// Backward compatibility layer
#[deprecated(since = "0.9.0", note = "Use Observation instead of Committee")]
pub mod committee_compat {
    use super::*;
    
    /// Alias for ObservationConfig for backward compatibility
    pub type CommitteeConfig = ObservationConfig;
    
    /// Alias for Observation for backward compatibility
    pub type Committee = Observation;
    
    /// Factory function for backward compatibility
    pub fn create_committee(config: CommitteeConfig) -> Result<Committee> {
        create_observation(config)
    }
} 