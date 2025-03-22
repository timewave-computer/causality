//! Data Provider Interface
//!
//! This module provides interfaces for accessing data from external sources.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::committee::{Result, Error};

/// A provider ID that uniquely identifies a data provider
pub type ProviderId = String;

/// Configuration for a data provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Unique identifier for the provider
    pub id: ProviderId,
    /// Type of the provider
    pub provider_type: String,
    /// Connection details
    pub connection: HashMap<String, String>,
    /// Authentication details
    pub auth: Option<HashMap<String, String>>,
    /// Timeout for requests (in seconds)
    pub timeout_secs: u64,
    /// Maximum number of retries for requests
    pub max_retries: u32,
    /// Rate limiting (requests per second)
    pub rate_limit: Option<f64>,
    /// Additional parameters
    pub parameters: HashMap<String, String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        ProviderConfig {
            id: "default".to_string(),
            provider_type: "generic".to_string(),
            connection: HashMap::new(),
            auth: None,
            timeout_secs: 30,
            max_retries: 3,
            rate_limit: None,
            parameters: HashMap::new(),
        }
    }
}

/// Status of a data provider
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderStatus {
    /// The provider is disconnected
    Disconnected,
    /// The provider is connected
    Connected,
    /// The provider is in an error state
    Error,
}

/// A query parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParameter {
    /// Name of the parameter
    pub name: String,
    /// Value of the parameter
    pub value: String,
}

/// A query for data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQuery {
    /// Resource path or identifier
    pub resource: String,
    /// Query parameters
    pub parameters: Vec<QueryParameter>,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Body of the query
    pub body: Option<String>,
    /// Timeout for the query (in seconds)
    pub timeout_secs: Option<u64>,
}

impl DataQuery {
    /// Create a new data query
    pub fn new(resource: &str) -> Self {
        DataQuery {
            resource: resource.to_string(),
            parameters: Vec::new(),
            headers: HashMap::new(),
            body: None,
            timeout_secs: None,
        }
    }
    
    /// Add a parameter to the query
    pub fn with_parameter(mut self, name: &str, value: &str) -> Self {
        self.parameters.push(QueryParameter {
            name: name.to_string(),
            value: value.to_string(),
        });
        self
    }
    
    /// Add a header to the query
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }
    
    /// Set the body of the query
    pub fn with_body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }
    
    /// Set the timeout for the query
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = Some(timeout_secs);
        self
    }
}

/// A data response
#[derive(Debug, Clone)]
pub struct DataResponse {
    /// Status code of the response
    pub status_code: u16,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Body of the response
    pub body: Vec<u8>,
    /// Error message if any
    pub error: Option<String>,
}

impl DataResponse {
    /// Check if the response is successful
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }
    
    /// Get the response body as a string
    pub fn body_as_string(&self) -> Result<String> {
        String::from_utf8(self.body.clone()).map_err(|e| {
            Error::Data(format!("Failed to convert response body to string: {}", e))
        })
    }
    
    /// Get the response body as JSON
    pub fn body_as_json(&self) -> Result<serde_json::Value> {
        let body_str = self.body_as_string()?;
        serde_json::from_str(&body_str).map_err(|e| {
            Error::Data(format!("Failed to parse response body as JSON: {}", e))
        })
    }
}

/// A data provider that can retrieve data from external sources
#[async_trait]
pub trait DataProvider: Send + Sync {
    /// Get the provider ID
    fn id(&self) -> &ProviderId;
    
    /// Get the provider type
    fn provider_type(&self) -> &str;
    
    /// Initialize the provider
    async fn initialize(&self) -> Result<()>;
    
    /// Connect to the data source
    async fn connect(&self) -> Result<()>;
    
    /// Disconnect from the data source
    async fn disconnect(&self) -> Result<()>;
    
    /// Get the current status of the provider
    async fn status(&self) -> Result<ProviderStatus>;
    
    /// Query data from the provider
    async fn query(&self, query: &DataQuery) -> Result<DataResponse>;
    
    /// Check if the provider is available
    async fn is_available(&self) -> Result<bool> {
        match self.status().await? {
            ProviderStatus::Connected => Ok(true),
            _ => Ok(false),
        }
    }
}

/// A factory for creating data providers
pub struct ProviderFactory {
    /// Constructors for different provider types
    constructors: Mutex<HashMap<String, Box<dyn Fn(ProviderConfig) -> Result<Arc<dyn DataProvider>> + Send + Sync>>>,
}

impl ProviderFactory {
    /// Create a new provider factory
    pub fn new() -> Self {
        ProviderFactory {
            constructors: Mutex::new(HashMap::new()),
        }
    }
    
    /// Register a constructor for a provider type
    pub fn register_constructor<F>(&self, provider_type: &str, constructor: F) -> Result<()>
    where
        F: Fn(ProviderConfig) -> Result<Arc<dyn DataProvider>> + Send + Sync + 'static,
    {
        let mut constructors = self.constructors.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on constructors".to_string())
        })?;
        
        constructors.insert(provider_type.to_string(), Box::new(constructor));
        
        Ok(())
    }
    
    /// Create a provider with the given configuration
    pub fn create_provider(&self, config: ProviderConfig) -> Result<Arc<dyn DataProvider>> {
        let constructors = self.constructors.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on constructors".to_string())
        })?;
        
        let constructor = constructors.get(&config.provider_type).ok_or_else(|| {
            Error::Configuration(format!(
                "No constructor for provider type '{}'", config.provider_type
            ))
        })?;
        
        constructor(config)
    }
}

/// A registry for managing data providers
pub struct ProviderRegistry {
    /// Map of provider IDs to providers
    providers: Mutex<HashMap<ProviderId, Arc<dyn DataProvider>>>,
    /// Factory for creating providers
    factory: Arc<ProviderFactory>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new(factory: Arc<ProviderFactory>) -> Self {
        ProviderRegistry {
            providers: Mutex::new(HashMap::new()),
            factory,
        }
    }
    
    /// Register a provider
    pub fn register(&self, provider: Arc<dyn DataProvider>) -> Result<()> {
        let mut providers = self.providers.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on providers".to_string())
        })?;
        
        providers.insert(provider.id().clone(), provider);
        
        Ok(())
    }
    
    /// Create and register a provider with the given configuration
    pub fn create_and_register(&self, config: ProviderConfig) -> Result<Arc<dyn DataProvider>> {
        let provider = self.factory.create_provider(config)?;
        self.register(provider.clone())?;
        Ok(provider)
    }
    
    /// Get a provider by ID
    pub fn get(&self, id: &str) -> Result<Arc<dyn DataProvider>> {
        let providers = self.providers.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on providers".to_string())
        })?;
        
        providers.get(id).cloned().ok_or_else(|| {
            Error::Configuration(format!("No provider with ID '{}'", id))
        })
    }
    
    /// Remove a provider by ID
    pub fn remove(&self, id: &str) -> Result<()> {
        let mut providers = self.providers.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on providers".to_string())
        })?;
        
        if providers.remove(id).is_none() {
            return Err(Error::Configuration(format!(
                "No provider with ID '{}'", id
            )));
        }
        
        Ok(())
    }
    
    /// Get all registered provider IDs
    pub fn get_provider_ids(&self) -> Result<Vec<ProviderId>> {
        let providers = self.providers.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on providers".to_string())
        })?;
        
        Ok(providers.keys().cloned().collect())
    }
    
    /// Initialize all providers
    pub async fn initialize_all(&self) -> Result<()> {
        let provider_ids = self.get_provider_ids()?;
        
        for id in provider_ids {
            let provider = self.get(&id)?;
            provider.initialize().await?;
        }
        
        Ok(())
    }
    
    /// Connect all providers
    pub async fn connect_all(&self) -> Result<()> {
        let provider_ids = self.get_provider_ids()?;
        
        for id in provider_ids {
            let provider = self.get(&id)?;
            provider.connect().await?;
        }
        
        Ok(())
    }
    
    /// Disconnect all providers
    pub async fn disconnect_all(&self) -> Result<()> {
        let provider_ids = self.get_provider_ids()?;
        
        for id in provider_ids {
            let provider = self.get(&id)?;
            provider.disconnect().await?;
        }
        
        Ok(())
    }
}

/// A basic HTTP data provider
pub struct HttpProvider {
    /// Configuration for the provider
    config: ProviderConfig,
    /// HTTP client
    client: reqwest::Client,
    /// Current status
    status: Mutex<ProviderStatus>,
}

impl HttpProvider {
    /// Create a new HTTP provider
    pub fn new(config: ProviderConfig) -> Result<Self> {
        // Validate configuration
        if !config.connection.contains_key("base_url") {
            return Err(Error::Configuration(
                "HTTP provider requires 'base_url' in connection".to_string()
            ));
        }
        
        // Create HTTP client with proper configuration
        let mut client_builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs));
        
        // Add authentication if provided
        if let Some(auth) = &config.auth {
            if let (Some(username), Some(password)) = (auth.get("username"), auth.get("password")) {
                client_builder = client_builder.basic_auth(username, Some(password));
            } else if let Some(token) = auth.get("token") {
                client_builder = client_builder.bearer_auth(token);
            }
        }
        
        let client = client_builder.build().map_err(|e| {
            Error::Configuration(format!("Failed to create HTTP client: {}", e))
        })?;
        
        Ok(HttpProvider {
            config,
            client,
            status: Mutex::new(ProviderStatus::Disconnected),
        })
    }
    
    /// Get the base URL
    fn base_url(&self) -> &str {
        self.config.connection.get("base_url").unwrap()
    }
}

#[async_trait]
impl DataProvider for HttpProvider {
    fn id(&self) -> &ProviderId {
        &self.config.id
    }
    
    fn provider_type(&self) -> &str {
        "http"
    }
    
    async fn initialize(&self) -> Result<()> {
        // Nothing special to initialize for HTTP
        Ok(())
    }
    
    async fn connect(&self) -> Result<()> {
        // Check connection by making a simple request to the base URL
        let response = self.client.get(self.base_url())
            .send()
            .await
            .map_err(|e| {
                Error::Connection(format!("Failed to connect to {}: {}", self.base_url(), e))
            })?;
        
        // Update status based on response
        let mut status = self.status.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on status".to_string())
        })?;
        
        if response.status().is_success() {
            *status = ProviderStatus::Connected;
        } else {
            *status = ProviderStatus::Error;
            return Err(Error::Connection(format!(
                "Failed to connect to {}: HTTP {}", 
                self.base_url(), 
                response.status()
            )));
        }
        
        Ok(())
    }
    
    async fn disconnect(&self) -> Result<()> {
        // Update status
        let mut status = self.status.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on status".to_string())
        })?;
        
        *status = ProviderStatus::Disconnected;
        
        Ok(())
    }
    
    async fn status(&self) -> Result<ProviderStatus> {
        let status = self.status.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on status".to_string())
        })?;
        
        Ok(status.clone())
    }
    
    async fn query(&self, query: &DataQuery) -> Result<DataResponse> {
        // Check if connected
        let status = self.status().await?;
        if status != ProviderStatus::Connected {
            return Err(Error::Connection(
                "Provider is not connected".to_string()
            ));
        }
        
        // Build URL
        let url = format!("{}/{}", self.base_url(), query.resource);
        
        // Build request
        let mut request_builder = self.client.get(&url);
        
        // Add query parameters
        for param in &query.parameters {
            request_builder = request_builder.query(&[(param.name.as_str(), param.value.as_str())]);
        }
        
        // Add headers
        for (name, value) in &query.headers {
            request_builder = request_builder.header(name, value);
        }
        
        // Add body if provided
        if let Some(body) = &query.body {
            request_builder = request_builder.body(body.clone());
        }
        
        // Add timeout if provided
        if let Some(timeout_secs) = query.timeout_secs {
            request_builder = request_builder.timeout(Duration::from_secs(timeout_secs));
        }
        
        // Execute request with retries
        let mut last_error = None;
        for attempt in 0..=self.config.max_retries {
            match request_builder.try_clone().unwrap().send().await {
                Ok(response) => {
                    // Convert response
                    let status_code = response.status().as_u16();
                    
                    let headers = response.headers().iter()
                        .map(|(name, value)| {
                            (name.to_string(), value.to_str().unwrap_or("").to_string())
                        })
                        .collect();
                    
                    let body = response.bytes().await.map_err(|e| {
                        Error::Data(format!("Failed to read response body: {}", e))
                    })?.to_vec();
                    
                    let error = if !response.status().is_success() {
                        Some(format!("HTTP error: {}", response.status()))
                    } else {
                        None
                    };
                    
                    return Ok(DataResponse {
                        status_code,
                        headers,
                        body,
                        error,
                    });
                }
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempt < self.config.max_retries {
                        // Exponential backoff
                        let backoff = Duration::from_millis(100 * 2u64.pow(attempt));
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }
        
        Err(Error::Data(format!(
            "Failed to execute query after {} retries: {}",
            self.config.max_retries,
            last_error.unwrap()
        )))
    }
} 