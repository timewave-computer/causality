// Observation provider functionality
//
// This module provides data provider functionality for observations.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};

use causality_types::{Error, Result};
use crate::log::{LogEntry, LogStorage};

/// Configuration for a data provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Unique identifier for the provider
    pub id: String,
    /// Provider type
    pub provider_type: String,
    /// Provider-specific configuration
    pub config: serde_json::Value,
}

/// A fact provided by an external data source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidedData {
    /// Unique identifier for the data
    pub id: String,
    /// The provider ID that produced this data
    pub provider_id: String,
    /// The type of data
    pub data_type: String,
    /// The data content
    pub content: serde_json::Value,
    /// Metadata associated with the data
    pub metadata: HashMap<String, String>,
    /// Timestamp when the data was provided
    pub timestamp: u64,
}

/// Factory for creating data providers
pub struct ProviderFactory {
    /// Registered provider types
    provider_types: RwLock<HashMap<String, Box<dyn ProviderCreator>>>,
}

impl ProviderFactory {
    /// Create a new provider factory
    pub fn new() -> Self {
        ProviderFactory {
            provider_types: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a provider creator
    pub fn register_provider<C>(&self, provider_type: &str, creator: C) -> Result<()>
    where
        C: ProviderCreator + 'static,
    {
        let mut types = self.provider_types.write().map_err(|_| 
            Error::Internal("Failed to lock provider types".to_string()))?;
            
        types.insert(provider_type.to_string(), Box::new(creator));
        
        Ok(())
    }
    
    /// Create a provider from configuration
    pub fn create_provider(&self, config: ProviderConfig) -> Result<Arc<dyn DataProvider>> {
        let types = self.provider_types.read().map_err(|_| 
            Error::Internal("Failed to lock provider types".to_string()))?;
            
        let creator = types.get(&config.provider_type)
            .ok_or_else(|| Error::Configuration(
                format!("Unknown provider type: {}", config.provider_type)
            ))?;
            
        creator.create_provider(config)
    }
}

/// An interface for creating data providers
pub trait ProviderCreator: Send + Sync {
    /// Create a provider from configuration
    fn create_provider(&self, config: ProviderConfig) -> Result<Arc<dyn DataProvider>>;
}

/// An interface for data providers
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
}

/// Status of a data provider
#[derive(Debug, Clone)]
pub struct ProviderStatus {
    /// The provider ID
    pub id: String,
    /// Whether the provider is running
    pub running: bool,
    /// The number of items provided
    pub items_provided: u64,
    /// The latest timestamp
    pub latest_timestamp: Option<u64>,
}

/// A basic HTTP data provider
pub struct HttpProvider {
    /// Provider ID
    id: String,
    /// Configuration
    config: HttpProviderConfig,
    /// Target log for storing provided data
    target_log: RwLock<Option<Arc<dyn LogStorage>>>,
    /// Provider status
    status: RwLock<ProviderStatus>,
    /// HTTP client
    client: reqwest::Client,
}

/// Configuration for an HTTP provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpProviderConfig {
    /// Base URL for the API
    pub base_url: String,
    /// Endpoint path
    pub endpoint: String,
    /// HTTP method
    pub method: String,
    /// Headers to include
    pub headers: HashMap<String, String>,
    /// Query parameters
    pub query_params: HashMap<String, String>,
    /// How often to poll (in seconds)
    pub poll_interval_secs: u64,
    /// Data type to assign to provided items
    pub data_type: String,
}

impl HttpProvider {
    /// Create a new HTTP provider
    pub fn new(id: String, config: HttpProviderConfig) -> Result<Self> {
        // Initialize status
        let status = ProviderStatus {
            id: id.clone(),
            running: false,
            items_provided: 0,
            latest_timestamp: None,
        };
        
        // Create HTTP client
        let client = reqwest::Client::new();
        
        Ok(HttpProvider {
            id,
            config,
            target_log: RwLock::new(None),
            status: RwLock::new(status),
            client,
        })
    }
    
    /// Start the polling loop
    fn start_polling(self: Arc<Self>) -> Result<()> {
        let poll_interval = std::time::Duration::from_secs(self.config.poll_interval_secs);
        
        // Update status
        {
            let mut status = self.status.write().map_err(|_| 
                Error::Internal("Failed to lock status".to_string()))?;
                
            status.running = true;
        }
        
        // Start polling task
        tokio::spawn(async move {
            loop {
                // Check if still running
                let running = {
                    let status = self.status.read().expect("Failed to lock status");
                    status.running
                };
                
                if !running {
                    break;
                }
                
                // Poll for data
                match self.poll_data().await {
                    Ok(items) => {
                        // Process items
                        for item in items {
                            if let Err(e) = self.process_item(item).await {
                                log::error!("Error processing item: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        log::error!("Error polling for data: {}", e);
                    }
                }
                
                // Wait before polling again
                tokio::time::sleep(poll_interval).await;
            }
        });
        
        Ok(())
    }
    
    /// Poll for data
    async fn poll_data(&self) -> Result<Vec<ProvidedData>> {
        // Build URL
        let url = format!("{}{}", self.config.base_url, self.config.endpoint);
        
        // Build request
        let mut request = match self.config.method.to_uppercase().as_str() {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            method => return Err(Error::Configuration(format!("Unsupported HTTP method: {}", method))),
        };
        
        // Add headers
        for (key, value) in &self.config.headers {
            request = request.header(key, value);
        }
        
        // Add query parameters
        for (key, value) in &self.config.query_params {
            request = request.query(&[(key, value)]);
        }
        
        // Send request
        let response = request.send().await.map_err(|e| 
            Error::External(format!("HTTP request failed: {}", e)))?;
            
        // Check status
        if !response.status().is_success() {
            return Err(Error::External(format!(
                "HTTP request failed with status: {}", response.status()
            )));
        }
        
        // Parse response
        let json = response.json::<serde_json::Value>().await.map_err(|e| 
            Error::Serialization(format!("Failed to parse response: {}", e)))?;
            
        // Convert to provided data
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let item = ProvidedData {
            id: format!("{}:{}", self.id, timestamp),
            provider_id: self.id.clone(),
            data_type: self.config.data_type.clone(),
            content: json,
            metadata: HashMap::new(),
            timestamp,
        };
        
        Ok(vec![item])
    }
    
    /// Process a provided data item
    async fn process_item(&self, item: ProvidedData) -> Result<()> {
        // Get target log
        let target_log = {
            let log = self.target_log.read().map_err(|_| 
                Error::Internal("Failed to lock target log".to_string()))?;
                
            log.clone().ok_or_else(|| 
                Error::Configuration("No target log configured".to_string())
            )?
        };
        
        // Create log entry
        let entry = LogEntry {
            log_id: format!("provider:{}", self.id),
            sequence: 0, // Will be assigned by storage
            timestamp: item.timestamp,
            data: serde_json::to_value(item.clone()).map_err(|e| 
                Error::Serialization(format!("Failed to serialize item: {}", e)))?,
            metadata: item.metadata.clone(),
        };
        
        // Append to log
        target_log.append(entry).map_err(|e| 
            Error::Storage(format!("Failed to append to log: {}", e)))?;
            
        // Update status
        {
            let mut status = self.status.write().map_err(|_| 
                Error::Internal("Failed to lock status".to_string()))?;
                
            status.items_provided += 1;
            status.latest_timestamp = Some(item.timestamp);
        }
        
        Ok(())
    }
}

impl DataProvider for HttpProvider {
    /// Get the provider ID
    fn get_id(&self) -> &str {
        &self.id
    }
    
    /// Initialize the provider
    fn initialize(&self) -> Result<()> {
        // Nothing to do for HTTP provider
        Ok(())
    }
    
    /// Start providing data
    fn start(&self) -> Result<()> {
        // Start polling
        let provider = Arc::new(self.clone());
        provider.start_polling()?;
        
        Ok(())
    }
    
    /// Stop providing data
    fn stop(&self) -> Result<()> {
        // Update status
        let mut status = self.status.write().map_err(|_| 
            Error::Internal("Failed to lock status".to_string()))?;
            
        status.running = false;
        
        Ok(())
    }
    
    /// Get provider status
    fn get_status(&self) -> Result<ProviderStatus> {
        let status = self.status.read().map_err(|_| 
            Error::Internal("Failed to lock status".to_string()))?;
            
        Ok(status.clone())
    }
    
    /// Set the target log
    fn set_target_log(&self, log: Arc<dyn LogStorage>) -> Result<()> {
        let mut target = self.target_log.write().map_err(|_| 
            Error::Internal("Failed to lock target log".to_string()))?;
            
        *target = Some(log);
        
        Ok(())
    }
}

impl Clone for HttpProvider {
    fn clone(&self) -> Self {
        // Create a new client for the clone
        let client = reqwest::Client::new();
        
        // Clone the status
        let status = {
            let status = self.status.read().expect("Failed to lock status");
            status.clone()
        };
        
        HttpProvider {
            id: self.id.clone(),
            config: self.config.clone(),
            target_log: RwLock::new(None),
            status: RwLock::new(status),
            client,
        }
    }
}

/// Creator for HTTP providers
pub struct HttpProviderCreator;

impl ProviderCreator for HttpProviderCreator {
    fn create_provider(&self, config: ProviderConfig) -> Result<Arc<dyn DataProvider>> {
        // Parse HTTP-specific config
        let http_config: HttpProviderConfig = serde_json::from_value(config.config.clone())
            .map_err(|e| Error::Configuration(format!(
                "Invalid HTTP provider configuration: {}", e
            )))?;
            
        // Create provider
        let provider = HttpProvider::new(config.id, http_config)?;
        
        Ok(Arc::new(provider))
    }
} 