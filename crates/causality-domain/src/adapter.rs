// Domain adapter interface for working with different blockchain domains
//
// This module defines interfaces and utilities for working with domain adapters.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

use crate::{
    BlockHeight, BlockHash, Timestamp, 
    DomainType, DomainStatus, TransactionId,
    TransactionReceipt, Transaction,
    FactType, 
};
use crate::selection::DomainId;

/// Error type for domain adapter operations
#[derive(Debug, thiserror::Error)]
pub enum DomainAdapterError {
    /// Domain not found
    #[error("Domain not found: {0}")]
    DomainNotFound(String),
    
    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    
    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),
    
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for domain adapter operations
pub type DomainAdapterResult<T> = Result<T, DomainAdapterError>;

/// Metadata from a fact observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactObservationMeta {
    /// Metadata about the observation
    pub metadata: HashMap<String, String>,
}

/// Query for facts in a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactQuery {
    /// Fact type to query
    pub fact_type: String,
    
    /// Query parameters
    pub parameters: HashMap<String, String>,
    
    /// Whether verification is required
    pub requires_verification: bool,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl FactQuery {
    /// Create a new fact query
    pub fn new(fact_type: impl Into<String>) -> Self {
        Self {
            fact_type: fact_type.into(),
            parameters: HashMap::new(),
            requires_verification: false,
            metadata: HashMap::new(),
        }
    }
    
    /// Add a parameter to the query
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Set verification requirements
    pub fn with_verification(mut self, required: bool) -> Self {
        self.requires_verification = required;
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Interface for domain adapters
pub trait DomainAdapter: Send + Sync + fmt::Debug {
    /// Get the domain ID
    fn domain_id(&self) -> &DomainId;
    
    /// Get the domain type
    fn domain_type(&self) -> DomainType;
    
    /// Get the domain status
    fn status(&self) -> Result<DomainStatus, DomainAdapterError>;
    
    /// Get the latest block height
    fn get_latest_block(&self) -> Result<BlockHeight, DomainAdapterError>;
    
    /// Get a time map entry for a specific block height
    fn get_time_map_entry(&self, height: BlockHeight) -> Result<TimeMapEntry, DomainAdapterError>;
    
    /// Observe a fact from the domain
    fn observe_fact(&self, query: &FactQuery) -> Result<(FactType, FactObservationMeta), DomainAdapterError>;
    
    /// Submit a transaction to the domain
    fn submit_transaction(&self, tx: Transaction) -> DomainAdapterResult<TransactionId>;
    
    /// Get a transaction receipt
    fn transaction_receipt(&self, tx_id: &TransactionId) -> DomainAdapterResult<TransactionReceipt>;
    
    /// Check if a transaction has been confirmed
    fn transaction_confirmed(&self, tx_id: &TransactionId) -> DomainAdapterResult<bool>;
    
    /// Wait for a transaction to be confirmed, with an optional timeout
    fn wait_for_confirmation(
        &self,
        tx_id: &TransactionId,
        max_wait_ms: Option<u64>,
    ) -> DomainAdapterResult<TransactionReceipt>;
    
    /// Get capabilities supported by this domain
    fn capabilities(&self) -> Vec<String> {
        Vec::new()
    }
    
    /// Check if the domain is connected and responsive
    fn check_connectivity(&self) -> DomainAdapterResult<bool> {
        Ok(true) // Default implementation assumes connectivity
    }
}

/// Time map entry for a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeMapEntry {
    /// Block height
    pub height: BlockHeight,
    /// Block hash
    pub hash: BlockHash,
    /// Timestamp
    pub timestamp: Timestamp,
}

/// Information about a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInfo {
    /// Unique domain identifier
    pub domain_id: DomainId,
    /// Human-readable name for the domain
    pub name: String,
    /// Domain type (e.g., EVM, CosmWasm)
    pub domain_type: DomainType,
    /// Current domain status
    pub status: DomainStatus,
    /// Additional domain metadata
    pub metadata: HashMap<String, String>,
}

/// Factory for creating domain adapters
pub trait DomainAdapterFactory: Send + Sync + fmt::Debug {
    /// Create a domain adapter
    fn create_adapter(&self, domain_id: DomainId, config: HashMap<String, String>) -> Result<Arc<dyn DomainAdapter>, DomainAdapterError>;
    
    /// Get supported domain types
    fn supported_types(&self) -> Vec<DomainType>;
    
    /// Get the factory name
    fn name(&self) -> &str;
}

/// Registry for domain adapters
#[derive(Debug, Default)]
pub struct DomainAdapterRegistry {
    /// Factories for creating adapters
    factories: Mutex<Vec<Box<dyn DomainAdapterFactory>>>,
    /// Registered adapters
    adapters: Mutex<HashMap<DomainId, Arc<dyn DomainAdapter>>>,
}

impl DomainAdapterRegistry {
    /// Create a new domain adapter registry
    pub fn new() -> Self {
        Self {
            factories: Mutex::new(Vec::new()),
            adapters: Mutex::new(HashMap::new()),
        }
    }
    
    /// Register a factory
    pub fn register_factory(&self, factory: Box<dyn DomainAdapterFactory>) {
        let mut factories = self.factories.lock().unwrap();
        factories.push(factory);
    }
    
    /// Register an adapter
    pub fn register_adapter(&self, adapter: Arc<dyn DomainAdapter>) {
        let domain_id = adapter.domain_id().clone();
        let mut adapters = self.adapters.lock().unwrap();
        adapters.insert(domain_id, adapter);
    }
    
    /// Get a domain adapter by ID
    pub fn get_adapter(&self, domain_id: &DomainId) -> Option<Arc<dyn DomainAdapter>> {
        let adapters = self.adapters.lock().unwrap();
        adapters.get(domain_id).cloned()
    }
    
    /// List all registered domain IDs
    pub fn list_domains(&self) -> Vec<DomainId> {
        let adapters = self.adapters.lock().unwrap();
        adapters.keys().cloned().collect()
    }
    
    /// Remove a domain adapter
    pub fn remove_adapter(&self, domain_id: &DomainId) -> bool {
        let mut adapters = self.adapters.lock().unwrap();
        adapters.remove(domain_id).is_some()
    }
    
    /// Get all registered adapters
    pub fn get_all_adapters(&self) -> Vec<Arc<dyn DomainAdapter>> {
        let adapters = self.adapters.lock().unwrap();
        adapters.values().cloned().collect()
    }
    
    /// Create an adapter for a domain
    pub fn create_adapter(&self, domain_id: DomainId, domain_type: DomainType, config: HashMap<String, String>) -> Result<Arc<dyn DomainAdapter>, DomainAdapterError> {
        // Check if already registered
        if let Some(adapter) = self.get_adapter(&domain_id) {
            return Ok(adapter);
        }
        
        // Find a factory that supports this domain type
        let factories = self.factories.lock().unwrap();
        for factory in factories.iter() {
            if factory.supported_types().contains(&domain_type) {
                let adapter = factory.create_adapter(domain_id, config)?;
                self.register_adapter(adapter.clone());
                return Ok(adapter);
            }
        }
        
        Err(DomainAdapterError::DomainNotFound(format!("No factory found for domain type {:?}", domain_type)))
    }
} 

