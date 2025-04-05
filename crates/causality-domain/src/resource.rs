// Resource integration for domains
// Original file: src/domain/resource_integration.rs

// Domain Resource Integration
//
// This module provides the integration between domain adapters and the resource register system,
// enabling cross-domain resource operations and storage.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::error::{Result as DomainResult, Error};
use crate::selection::{DomainSelectionStrategy, SelectionCriteria, DomainId};
use crate::{BlockHeight, Timestamp, DomainAdapter, Transaction, DomainInfo};
use crate::adapter::FactQuery;
use crate::fact::types::FactType;

// Implement the trait to make domain adapters compatible with selection system
struct DomainAdapterWrapper {
    inner: Arc<dyn DomainAdapter>
}

impl std::fmt::Debug for DomainAdapterWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomainAdapterWrapper")
            .field("domain_id", &self.inner.domain_id())
            .finish()
    }
}

impl crate::selection::DomainAdapter for DomainAdapterWrapper {
    fn domain_id(&self) -> &DomainId {
        self.inner.domain_id()
    }
    
    fn info(&self) -> crate::selection::DomainInfo {
        // Create a selection-compatible DomainInfo
        crate::selection::DomainInfo {
            domain_id: self.inner.domain_id().clone(),
            domain_type: self.inner.domain_type().to_string(),
            capabilities: self.inner.capabilities(),
            avg_latency: 0, // Default values, could be retrieved from adapter
            cost: 0,
            reliability: 1.0,
        }
    }
}

/// Content ID for resources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ContentId(pub String);

impl ContentId {
    /// Create a new content ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Convert to string
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl std::fmt::Display for ContentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Status of a resource lock operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockStatus {
    /// Lock was successfully acquired
    Acquired,
    /// Lock is already held by the same entity
    AlreadyHeld,
    /// Lock is unavailable (held by another entity)
    Unavailable,
}

/// Operation type for cross-domain resource operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CrossDomainResourceOperation {
    /// Store a resource in a domain
    Store,
    /// Retrieve a resource from a domain
    Retrieve,
    /// Verify a resource exists in a domain
    Verify,
    /// Transfer a resource from one domain to another
    Transfer {
        /// Source domain ID
        source_domain_id: String,
        /// Target domain ID
        target_domain_id: String,
        /// Additional metadata for the transfer
        additional_metadata: HashMap<String, String>,
    },
}

/// Result of a cross-domain resource operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossDomainResourceResult {
    /// Resource was stored successfully
    Stored {
        /// Resource ID
        resource_id: ContentId,
        /// Domain ID
        domain_id: String,
        /// Transaction ID
        transaction_id: String,
        /// Block height where stored
        block_height: Option<BlockHeight>,
        /// Timestamp when stored
        timestamp: Option<Timestamp>,
    },
    /// Resource was retrieved successfully
    Retrieved {
        /// Resource ID
        resource_id: ContentId,
        /// Domain ID
        domain_id: String,
        /// Resource contents
        contents: Vec<u8>,
        /// Resource metadata
        metadata: HashMap<String, String>,
    },
    /// Resource verification result
    Verified {
        /// Resource ID
        resource_id: ContentId,
        /// Domain ID
        domain_id: String,
        /// Whether the resource exists
        exists: bool,
        /// Resource metadata
        metadata: HashMap<String, String>,
    },
    /// Resource was transferred successfully
    Transferred {
        /// Resource ID
        resource_id: ContentId,
        /// Source domain ID
        source_domain_id: String,
        /// Target domain ID
        target_domain_id: String,
        /// Transaction ID
        transaction_id: String,
        /// Block height where stored
        block_height: Option<BlockHeight>,
        /// Timestamp when stored
        timestamp: Option<Timestamp>,
    },
}

/// Interface for domain resource adapters
#[async_trait]
pub trait DomainResourceAdapter: Send + Sync + std::fmt::Debug {
    /// Get the domain ID
    fn domain_id(&self) -> &DomainId;
    
    /// Store a resource in the domain
    async fn store_resource(
        &self, 
        resource_id: &ContentId, 
        contents: &[u8], 
        metadata: &HashMap<String, String>
    ) -> DomainResult<CrossDomainResourceResult>;
    
    /// Retrieve a resource from the domain
    async fn retrieve_resource(
        &self, 
        resource_id: &ContentId
    ) -> DomainResult<CrossDomainResourceResult>;
    
    /// Verify a resource exists in the domain
    async fn verify_resource(
        &self, 
        resource_id: &ContentId
    ) -> DomainResult<CrossDomainResourceResult>;
    
    /// Handle a cross-domain resource operation
    async fn handle_resource_operation(
        &self, 
        resource_id: &ContentId, 
        operation: &CrossDomainResourceOperation
    ) -> DomainResult<CrossDomainResourceResult> {
        match operation {
            CrossDomainResourceOperation::Store => {
                // This requires content, so return an error
                Err(Box::new(Error::InvalidArgument("Store operation requires content".to_string())))
            },
            CrossDomainResourceOperation::Retrieve => {
                self.retrieve_resource(resource_id).await
            },
            CrossDomainResourceOperation::Verify => {
                self.verify_resource(resource_id).await
            },
            CrossDomainResourceOperation::Transfer { .. } => {
                // Transfer operations are managed by the resource manager, not individual adapters
                Err(Box::new(Error::UnsupportedOperation("Transfer operations must be handled by the resource manager".to_string())))
            }
        }
    }
}

/// CosmWasm-based resource adapter
pub struct CosmWasmResourceAdapter {
    /// Domain adapter
    domain_adapter: Arc<dyn DomainAdapter>,
}

impl std::fmt::Debug for CosmWasmResourceAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CosmWasmResourceAdapter")
            .field("domain_id", &self.domain_adapter.domain_id())
            .finish()
    }
}

impl CosmWasmResourceAdapter {
    /// Create a new CosmWasm resource adapter
    pub fn new(domain_adapter: Arc<dyn DomainAdapter>) -> Self {
        Self { domain_adapter }
    }
}

#[async_trait]
impl DomainResourceAdapter for CosmWasmResourceAdapter {
    fn domain_id(&self) -> &DomainId {
        self.domain_adapter.domain_id()
    }
    
    async fn store_resource(
        &self, 
        resource_id: &ContentId, 
        contents: &[u8], 
        metadata: &HashMap<String, String>
    ) -> DomainResult<CrossDomainResourceResult> {
        // In a full implementation, this would create and submit a CosmWasm contract transaction
        // to store the resource data
        
        // Create resource storage transaction
        let tx = Transaction {
            data: contents.to_vec(),
            transaction_type: "store_resource".to_string(),
            metadata: metadata.clone(),
            domain_id: self.domain_adapter.domain_id().clone(),
        };
        
        // Submit transaction
        let tx_id = match self.domain_adapter.submit_transaction(tx) {
            Ok(id) => id,
            Err(e) => return Err(Box::new(Error::TransactionError(e.to_string()))),
        };
        
        // Wait for transaction to be confirmed
        let receipt = match self.domain_adapter.wait_for_confirmation(&tx_id, Some(30000)) {
            Ok(receipt) => receipt,
            Err(e) => return Err(Box::new(Error::TransactionError(e.to_string()))),
        };
        
        // Return result
        Ok(CrossDomainResourceResult::Stored {
            resource_id: resource_id.clone(),
            domain_id: self.domain_adapter.domain_id().clone(),
            transaction_id: tx_id.to_string(),
            block_height: Some(receipt.block_height),
            timestamp: None, // Would extract from receipt in a full implementation
        })
    }
    
    async fn retrieve_resource(
        &self, 
        resource_id: &ContentId
    ) -> DomainResult<CrossDomainResourceResult> {
        // For now, we create a fact query to retrieve the resource data
        let fact_query = FactQuery::new("resource_data")
            .with_parameter("resource_id", resource_id.to_string())
            .with_parameter("operation", "store")
            .with_verification(false);
        
        // Query the domain
        let (fact_type, fact_meta) = match self.domain_adapter.observe_fact(&fact_query) {
            Ok(result) => result,
            Err(e) => return Err(Box::new(Error::FactError(e.to_string()))),
        };
        
        // Extract resource data from the fact type
        match fact_type {
            FactType::Binary(data) => {
                // Extract metadata from the observation metadata
                let metadata = fact_meta.metadata.clone();
                
                Ok(CrossDomainResourceResult::Retrieved {
                    resource_id: resource_id.clone(),
                    domain_id: self.domain_adapter.domain_id().clone(),
                    contents: data,
                    metadata,
                })
            },
            _ => Err(Box::new(Error::DomainDataError(format!("Unexpected fact type returned for resource {}", resource_id))))
        }
    }
    
    async fn verify_resource(
        &self, 
        resource_id: &ContentId
    ) -> DomainResult<CrossDomainResourceResult> {
        // Similar to retrieve, but just checks existence
        let fact_query = FactQuery::new("resource_exists")
            .with_parameter("resource_id", resource_id.to_string())
            .with_parameter("query_type", "contract");
        
        // Query the domain
        let result = self.domain_adapter.observe_fact(&fact_query);
        
        // Check if the result is Ok (resource exists) or an error (resource doesn't exist)
        let exists = result.is_ok();
        
        // Extract metadata if the resource exists
        let metadata = if let Ok((_, fact_meta)) = result {
            fact_meta.metadata
        } else {
            HashMap::new()
        };
        
        Ok(CrossDomainResourceResult::Verified {
            resource_id: resource_id.clone(),
            domain_id: self.domain_adapter.domain_id().clone(),
            exists,
            metadata,
        })
    }
}

/// EVM-based resource adapter
pub struct EvmResourceAdapter {
    /// Domain adapter
    domain_adapter: Arc<dyn DomainAdapter>,
}

impl std::fmt::Debug for EvmResourceAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvmResourceAdapter")
            .field("domain_id", &self.domain_adapter.domain_id())
            .finish()
    }
}

impl EvmResourceAdapter {
    /// Create a new EVM resource adapter
    pub fn new(domain_adapter: Arc<dyn DomainAdapter>) -> Self {
        Self { domain_adapter }
    }
}

#[async_trait]
impl DomainResourceAdapter for EvmResourceAdapter {
    fn domain_id(&self) -> &DomainId {
        self.domain_adapter.domain_id()
    }
    
    async fn store_resource(
        &self, 
        resource_id: &ContentId, 
        contents: &[u8], 
        metadata: &HashMap<String, String>
    ) -> DomainResult<CrossDomainResourceResult> {
        // In a full implementation, this would create and submit an Ethereum transaction
        // to store the resource data in a smart contract
        
        // Create resource storage transaction
        let tx = Transaction {
            data: contents.to_vec(),
            transaction_type: "store_resource".to_string(),
            metadata: metadata.clone(),
            domain_id: self.domain_adapter.domain_id().clone(),
        };
        
        // Submit transaction
        let tx_id = match self.domain_adapter.submit_transaction(tx) {
            Ok(id) => id,
            Err(e) => return Err(Box::new(Error::TransactionError(e.to_string()))),
        };
        
        // Wait for transaction to be confirmed
        let receipt = match self.domain_adapter.wait_for_confirmation(&tx_id, Some(30000)) {
            Ok(receipt) => receipt,
            Err(e) => return Err(Box::new(Error::TransactionError(e.to_string()))),
        };
        
        // Return result
        Ok(CrossDomainResourceResult::Stored {
            resource_id: resource_id.clone(),
            domain_id: self.domain_adapter.domain_id().clone(),
            transaction_id: tx_id.to_string(),
            block_height: Some(receipt.block_height),
            timestamp: None, // Would extract from receipt in a full implementation
        })
    }
    
    async fn retrieve_resource(
        &self, 
        resource_id: &ContentId
    ) -> DomainResult<CrossDomainResourceResult> {
        // For now, we create a fact query to retrieve the resource data
        let fact_query = FactQuery::new("resource_data")
            .with_parameter("resource_id", resource_id.to_string())
            .with_parameter("operation", "store")
            .with_verification(false);
        
        // Query the domain
        let (fact_type, fact_meta) = match self.domain_adapter.observe_fact(&fact_query) {
            Ok(result) => result,
            Err(e) => return Err(Box::new(Error::FactError(e.to_string()))),
        };
        
        // Extract resource data from the fact type
        match fact_type {
            FactType::Binary(data) => {
                // Extract metadata from the observation metadata
                let metadata = fact_meta.metadata.clone();
                
                Ok(CrossDomainResourceResult::Retrieved {
                    resource_id: resource_id.clone(),
                    domain_id: self.domain_adapter.domain_id().clone(),
                    contents: data,
                    metadata,
                })
            },
            _ => Err(Box::new(Error::DomainDataError(format!("Unexpected fact type returned for resource {}", resource_id))))
        }
    }
    
    async fn verify_resource(
        &self, 
        resource_id: &ContentId
    ) -> DomainResult<CrossDomainResourceResult> {
        // Similar to retrieve, but just checks existence
        let fact_query = FactQuery::new("resource_exists")
            .with_parameter("resource_id", resource_id.to_string())
            .with_parameter("query_type", "contract");
        
        // Query the domain
        let result = self.domain_adapter.observe_fact(&fact_query);
        
        // Check if the result is Ok (resource exists) or an error (resource doesn't exist)
        let exists = result.is_ok();
        
        // Extract metadata if the resource exists
        let metadata = if let Ok((_, fact_meta)) = result {
            fact_meta.metadata
        } else {
            HashMap::new()
        };
        
        Ok(CrossDomainResourceResult::Verified {
            resource_id: resource_id.clone(),
            domain_id: self.domain_adapter.domain_id().clone(),
            exists,
            metadata,
        })
    }
}

/// Interface for a domain registry
pub trait DomainRegistry: Send + Sync + std::fmt::Debug {
    /// Get a domain adapter by ID
    fn get_adapter(&self, domain_id: &DomainId) -> Option<Arc<dyn DomainAdapter>>;
    
    /// List all registered domain IDs
    fn list_domains(&self) -> Vec<DomainId>;
    
    /// Get all registered adapters
    fn get_all_adapters(&self) -> DomainResult<Vec<Arc<dyn DomainAdapter>>>;
    
    /// Get domain information
    fn get_domain_info(&self, domain_id: &DomainId) -> Option<DomainInfo>;
}

/// Factory for creating domain resource adapters
pub struct DomainResourceAdapterFactory {
    domain_registry: Arc<dyn DomainRegistry>,
}

impl DomainResourceAdapterFactory {
    /// Create a new domain resource adapter factory
    pub fn new(domain_registry: Arc<dyn DomainRegistry>) -> Self {
        Self { domain_registry }
    }
    
    /// Create an adapter for the specified domain
    pub async fn create_adapter(&self, domain_id: &DomainId) -> DomainResult<Box<dyn DomainResourceAdapter>> {
        // Get domain adapter from registry
        let adapter = self.domain_registry.get_adapter(domain_id)
            .ok_or_else(|| Box::new(Error::DomainAdapterNotFound(domain_id.clone())))?;
        
        // Get domain info
        let domain_info = self.domain_registry.get_domain_info(domain_id)
            .ok_or_else(|| Box::new(Error::DomainNotFound(domain_id.clone())))?;
        
        // Create the appropriate adapter based on domain type
        let adapter: Box<dyn DomainResourceAdapter> = match domain_info.domain_type.as_str() {
            // For CosmWasm domains, we use the CosmWasm adapter
            #[cfg(feature = "cosmwasm_zk")]
            "cosmwasm" => Ok(Box::new(CosmWasmResourceAdapter::new(adapter))),
            
            #[cfg(not(feature = "cosmwasm_zk"))]
            "cosmwasm" => Err(Box::new(Error::UnsupportedDomainType(format!("{} (requires 'cosmwasm_zk' feature)", domain_info.domain_type)))),
            
            // For EVM domains, we use the EVM adapter
            "evm" | "ethereum" => Ok(Box::new(EvmResourceAdapter::new(adapter))),
            
            // Other domain types are not supported yet
            _ => Err(Box::new(Error::UnsupportedDomainType(domain_info.domain_type.clone())))
        }?;
        
        Ok(adapter)
    }
    
    /// Create the most appropriate domain resource adapter based on selection strategy
    pub async fn create_adapter_by_strategy(
        &self,
        required_capabilities: &std::collections::HashSet<String>,
        preferences: &HashMap<String, String>,
        strategy: &dyn DomainSelectionStrategy,
    ) -> DomainResult<Box<dyn DomainResourceAdapter>> {
        // Get all adapters
        let adapters = self.domain_registry.get_all_adapters()?;
        
        // Convert to SelectionCriteria
        let criteria = SelectionCriteria {
            required_capabilities: required_capabilities.iter().cloned().collect(),
            constraints: preferences.clone(),
            ..Default::default()
        };
        
        // Convert adapter types to match selection strategy's expected type
        let selection_adapters: Vec<Arc<dyn crate::selection::DomainAdapter>> = 
            adapters.iter()
                .map(|a| Arc::new(DomainAdapterWrapper { inner: a.clone() }) as Arc<dyn crate::selection::DomainAdapter>)
                .collect();
        
        // Use the strategy to select a domain
        let selection_result = strategy.select_domain(&selection_adapters, &criteria)
            .ok_or_else(|| Error::DomainNotFound("No suitable domain found for the given criteria".to_string()))?;
        
        // Create the adapter for the selected domain
        self.create_adapter(&selection_result.domain_id).await
    }
}

/// Manager for cross-domain resource operations
pub struct CrossDomainResourceManager {
    adapter_factory: Arc<DomainResourceAdapterFactory>,
    #[allow(dead_code)]
    default_strategy: Box<dyn DomainSelectionStrategy>,
}

impl std::fmt::Debug for CrossDomainResourceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CrossDomainResourceManager")
            .field("adapter_factory", &"DomainResourceAdapterFactory")
            .finish_non_exhaustive()
    }
}

impl CrossDomainResourceManager {
    /// Create a new cross-domain resource manager
    pub fn new(
        domain_registry: Arc<dyn DomainRegistry>,
        default_strategy: Box<dyn DomainSelectionStrategy>,
    ) -> Self {
        let adapter_factory = Arc::new(DomainResourceAdapterFactory::new(domain_registry));
        Self {
            adapter_factory,
            default_strategy,
        }
    }
    
    /// Store a resource in a domain
    pub async fn store_resource(
        &self,
        resource_id: &ContentId,
        contents: &[u8],
        metadata: &HashMap<String, String>,
        target_domain_id: &DomainId,
    ) -> DomainResult<CrossDomainResourceResult> {
        // Create the resource adapter
        let adapter = self.adapter_factory.create_adapter(target_domain_id).await?;
        
        // Store the resource
        adapter.store_resource(resource_id, contents, metadata).await
    }
    
    /// Retrieve a resource from a domain
    pub async fn retrieve_resource(
        &self,
        resource_id: &ContentId,
        source_domain_id: &DomainId,
    ) -> DomainResult<CrossDomainResourceResult> {
        // Create the resource adapter
        let adapter = self.adapter_factory.create_adapter(source_domain_id).await?;
        
        // Retrieve the resource
        adapter.retrieve_resource(resource_id).await
    }
    
    /// Verify a resource exists in a domain
    pub async fn verify_resource(
        &self,
        resource_id: &ContentId,
        domain_id: &DomainId,
    ) -> DomainResult<CrossDomainResourceResult> {
        // Create the resource adapter
        let adapter = self.adapter_factory.create_adapter(domain_id).await?;
        
        // Verify the resource
        adapter.verify_resource(resource_id).await
    }
    
    /// Transfer a resource from one domain to another
    pub async fn transfer_resource(
        &self,
        resource_id: &ContentId,
        source_domain_id: &DomainId,
        target_domain_id: &DomainId,
        _additional_metadata: &HashMap<String, String>,
    ) -> DomainResult<CrossDomainResourceResult> {
        // Retrieve the resource from the source domain
        let source_adapter = self.adapter_factory.create_adapter(source_domain_id).await?;
        let result = source_adapter.retrieve_resource(resource_id).await?;
        
        // Extract content and metadata
        let (contents, metadata) = match result {
            CrossDomainResourceResult::Retrieved { contents, metadata, .. } => {
                (contents, metadata)
            },
            _ => return Err(Box::new(Error::DomainDataError(format!(
                "Unexpected result type from retrieve operation"
            )))),
        };
        
        // Store the resource in the target domain
        let _target_adapter = self.adapter_factory.create_adapter(target_domain_id).await?;
        let result = _target_adapter.store_resource(resource_id, &contents, &metadata).await?;
        
        // Create the transfer result
        match result {
            CrossDomainResourceResult::Stored { transaction_id, block_height, timestamp, .. } => {
                Ok(CrossDomainResourceResult::Transferred {
                    resource_id: resource_id.clone(),
                    source_domain_id: source_domain_id.to_string(),
                    target_domain_id: target_domain_id.to_string(),
                    transaction_id,
                    block_height,
                    timestamp,
                })
            },
            _ => Err(Box::new(Error::DomainDataError("Unexpected result type from store operation".to_string())))
        }
    }
    
    /// Transfer a register from one domain to another
    pub async fn transfer_register(
        &self,
        register: &ResourceRegister,
        source_domain_id: &DomainId,
        target_domain_id: &DomainId,
        additional_metadata: &HashMap<String, String>,
    ) -> DomainResult<RegisterTransferResult> {
        // Create source adapter
        let source_adapter = self.adapter_factory.create_adapter(source_domain_id).await?;
        
        // Create target adapter
        let _target_adapter = self.adapter_factory.create_adapter(target_domain_id).await?;
        
        // Verify register exists in source domain
        let result = source_adapter.verify_resource(&register.id).await?;
        
        let exists = match result {
            CrossDomainResourceResult::Verified { exists, .. } => exists,
            _ => return Err(Box::new(Error::DomainDataError(format!(
                "Register {} not found in source domain {}", register.id, source_domain_id
            )))),
        };
        
        if !exists {
            return Err(Box::new(Error::DomainDataError(format!(
                "Register {} not found in source domain {}", register.id, source_domain_id
            ))));
        }
        
        // Transfer the register data
        let transfer_result = self.transfer_resource(
            &register.id,
            source_domain_id,
            target_domain_id,
            additional_metadata,
        ).await?;
        
        // Create the register transfer result
        match transfer_result {
            CrossDomainResourceResult::Transferred {
                transaction_id,
                block_height,
                timestamp,
                ..
            } => {
                Ok(RegisterTransferResult {
                    register_id: register.id.clone(),
                    source_domain_id: source_domain_id.to_string(),
                    target_domain_id: target_domain_id.to_string(),
                    transaction_id,
                    block_height,
                    timestamp,
                })
            },
            _ => Err(Box::new(Error::DomainDataError(format!(
                "Unexpected result type from transfer operation for register {}", register.id
            ))))
        }
    }
    
    /// Store a register in a domain
    pub async fn store_register(
        &self,
        register: &ResourceRegister,
        target_domain_id: &DomainId,
    ) -> DomainResult<RegisterStoreResult> {
        // Create the resource adapter
        let adapter = self.adapter_factory.create_adapter(target_domain_id).await?;
        
        // Store the register data
        let result = adapter.store_resource(&register.id, &register.data, &register.metadata).await?;
        
        // Create the register store result
        match result {
            CrossDomainResourceResult::Stored {
                transaction_id,
                block_height,
                timestamp,
                ..
            } => {
                Ok(RegisterStoreResult {
                    register_id: register.id.clone(),
                    domain_id: target_domain_id.to_string(),
                    transaction_id,
                    block_height,
                    timestamp,
                })
            },
            _ => Err(Box::new(Error::DomainDataError(format!(
                "Unexpected result type from store operation for register {}", register.id
            ))))
        }
    }
    
    /// Retrieve a register from a domain
    pub async fn retrieve_register(
        &self,
        register_id: &ContentId,
        domain_id: &DomainId,
    ) -> DomainResult<ResourceRegister> {
        // Create the resource adapter
        let adapter = self.adapter_factory.create_adapter(domain_id).await?;
        
        // Retrieve the register data
        let result = adapter.retrieve_resource(register_id).await?;
        
        // Create the register
        match result {
            CrossDomainResourceResult::Retrieved {
                contents,
                metadata,
                ..
            } => {
                Ok(ResourceRegister {
                    id: register_id.clone(),
                    data: contents,
                    metadata,
                })
            },
            _ => Err(Box::new(Error::DomainDataError(format!(
                "Unexpected result type from retrieve operation for resource {}", register_id
            ))))
        }
    }
    
    /// Verify a register exists in a domain
    pub async fn verify_register(
        &self,
        register_id: &ContentId,
        domain_id: &DomainId,
    ) -> DomainResult<bool> {
        // Create the resource adapter
        let adapter = self.adapter_factory.create_adapter(domain_id).await?;
        
        // Verify the register
        let result = adapter.verify_resource(register_id).await?;
        
        // Return whether the register exists
        match result {
            CrossDomainResourceResult::Verified { exists, .. } => {
                Ok(exists)
            },
            _ => Err(Box::new(Error::DomainDataError(format!(
                "Unexpected result type from verify operation for resource {}", register_id
            ))))
        }
    }
}

/// Resource register for cross-domain resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRegister {
    /// Register ID
    pub id: ContentId,
    /// Register data
    pub data: Vec<u8>,
    /// Register metadata
    pub metadata: HashMap<String, String>,
}

/// Result of a register store operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterStoreResult {
    /// Register ID
    pub register_id: ContentId,
    /// Domain ID
    pub domain_id: String,
    /// Transaction ID
    pub transaction_id: String,
    /// Block height where stored
    pub block_height: Option<BlockHeight>,
    /// Timestamp when stored
    pub timestamp: Option<Timestamp>,
}

/// Result of a register transfer operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterTransferResult {
    /// Register ID
    pub register_id: ContentId,
    /// Source domain ID
    pub source_domain_id: String,
    /// Target domain ID
    pub target_domain_id: String,
    /// Transaction ID
    pub transaction_id: String,
    /// Block height where stored
    pub block_height: Option<BlockHeight>,
    /// Timestamp when stored
    pub timestamp: Option<Timestamp>,
}

#[cfg(test)]
mod tests {
    
    
    
    // Test implementation would go here
} 
