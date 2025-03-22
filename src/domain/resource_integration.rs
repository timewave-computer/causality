// Domain Resource Integration
//
// This module provides the integration between domain adapters and the resource register system,
// enabling cross-domain resource operations and storage.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::error::{Error, Result};
use crate::types::{ResourceId, BlockHeight, BlockHash, Timestamp};
use crate::domain::{
    DomainAdapter, DomainRegistry, DomainId, DomainType, DomainStatus,
    selection::DomainSelectionStrategy
};
use crate::resource::{
    Register, RegisterId, RegisterContents, RegisterState, 
    ResourceRegister, StorageStrategy
};

/// Represents a cross-domain resource operation
#[derive(Debug, Clone)]
pub enum CrossDomainResourceOperation {
    /// Store a resource in a domain
    Store {
        resource_id: ResourceId,
        target_domain_id: DomainId,
        contents: Vec<u8>,
        metadata: HashMap<String, String>,
    },
    
    /// Retrieve a resource from a domain
    Retrieve {
        resource_id: ResourceId,
        source_domain_id: DomainId,
    },
    
    /// Transfer a resource between domains
    Transfer {
        resource_id: ResourceId,
        source_domain_id: DomainId,
        target_domain_id: DomainId,
        metadata: HashMap<String, String>,
    },
    
    /// Verify a resource exists in a domain
    Verify {
        resource_id: ResourceId,
        domain_id: DomainId,
    },
}

/// Result of a cross-domain resource operation
#[derive(Debug, Clone)]
pub enum CrossDomainResourceResult {
    /// Resource has been stored successfully with a transaction receipt
    Stored {
        resource_id: ResourceId,
        domain_id: DomainId,
        transaction_id: String,
        block_height: Option<BlockHeight>,
        timestamp: Option<Timestamp>,
    },
    
    /// Resource has been retrieved successfully
    Retrieved {
        resource_id: ResourceId,
        domain_id: DomainId,
        contents: Vec<u8>,
        metadata: HashMap<String, String>,
    },
    
    /// Resource has been transferred successfully
    Transferred {
        resource_id: ResourceId,
        source_domain_id: DomainId,
        target_domain_id: DomainId,
        transaction_id: String,
        block_height: Option<BlockHeight>,
        timestamp: Option<Timestamp>,
    },
    
    /// Resource verification result
    Verified {
        resource_id: ResourceId,
        domain_id: DomainId,
        exists: bool,
        metadata: HashMap<String, String>,
    },
}

/// Trait for domain-specific resource storage strategies
#[async_trait]
pub trait DomainResourceAdapter: Send + Sync {
    /// Get the domain ID
    fn domain_id(&self) -> &DomainId;
    
    /// Store a resource in the domain
    async fn store_resource(
        &self, 
        resource_id: &ResourceId, 
        contents: &[u8], 
        metadata: &HashMap<String, String>
    ) -> Result<CrossDomainResourceResult>;
    
    /// Retrieve a resource from the domain
    async fn retrieve_resource(
        &self, 
        resource_id: &ResourceId
    ) -> Result<CrossDomainResourceResult>;
    
    /// Verify a resource exists in the domain
    async fn verify_resource(
        &self, 
        resource_id: &ResourceId
    ) -> Result<CrossDomainResourceResult>;
    
    /// Check if a resource operation is allowed in this domain
    async fn validate_operation(
        &self, 
        resource_id: &ResourceId, 
        operation: &CrossDomainResourceOperation
    ) -> Result<bool>;
}

/// Adapter for CosmWasm resource operations
pub struct CosmWasmResourceAdapter {
    domain_adapter: Arc<dyn DomainAdapter>,
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
        resource_id: &ResourceId, 
        contents: &[u8], 
        metadata: &HashMap<String, String>
    ) -> Result<CrossDomainResourceResult> {
        // In a full implementation, this would create and submit a CosmWasm contract transaction
        // to store the resource data
        
        // Create resource storage transaction
        let tx = crate::domain::Transaction {
            tx_type: "store_resource".to_string(),
            from: metadata.get("from").cloned().unwrap_or_default(),
            to: metadata.get("contract").cloned().unwrap_or_default(),
            data: contents.to_vec(),
            value: "0".to_string(),
            gas_limit: metadata.get("gas_limit").cloned().unwrap_or_else(|| "200000".to_string()),
            gas_price: metadata.get("gas_price").cloned(),
            nonce: metadata.get("nonce").cloned(),
            signature: None,
            metadata: metadata.clone(),
        };
        
        // Submit transaction
        let tx_id = self.domain_adapter.submit_transaction(tx).await?;
        
        // Wait for transaction to be confirmed
        let receipt = self.domain_adapter.wait_for_confirmation(&tx_id, Some(30000)).await?;
        
        // Return result
        Ok(CrossDomainResourceResult::Stored {
            resource_id: resource_id.clone(),
            domain_id: self.domain_adapter.domain_id().clone(),
            transaction_id: tx_id.to_string(),
            block_height: receipt.block_height,
            timestamp: None, // Would extract from receipt in a full implementation
        })
    }
    
    async fn retrieve_resource(
        &self, 
        resource_id: &ResourceId
    ) -> Result<CrossDomainResourceResult> {
        // In a full implementation, this would query a CosmWasm contract for resource data
        
        // For now, we create a fact query to retrieve the resource data
        let fact_query = crate::domain::FactQuery {
            selectors: vec!["resource_data".to_string()],
            metadata: HashMap::from([
                ("resource_id".to_string(), resource_id.to_string()),
                ("query_type".to_string(), "contract".to_string()),
            ]),
        };
        
        // Query the domain
        let facts = self.domain_adapter.observe_fact(&fact_query).await?;
        
        // Extract resource data from facts
        if let Some(fact) = facts.first() {
            let contents = fact.get_data_as_bytes("data")
                .map_err(|e| Error::DomainDataError(format!("Failed to extract resource data: {}", e)))?;
                
            let mut metadata = HashMap::new();
            for (key, value) in &fact.data {
                if key != "data" {
                    metadata.insert(key.clone(), value.clone());
                }
            }
            
            Ok(CrossDomainResourceResult::Retrieved {
                resource_id: resource_id.clone(),
                domain_id: self.domain_adapter.domain_id().clone(),
                contents,
                metadata,
            })
        } else {
            Err(Error::NotFound(format!("Resource {} not found in domain {}", resource_id, self.domain_adapter.domain_id())))
        }
    }
    
    async fn verify_resource(
        &self, 
        resource_id: &ResourceId
    ) -> Result<CrossDomainResourceResult> {
        // Similar to retrieve, but just checks existence
        let fact_query = crate::domain::FactQuery {
            selectors: vec!["resource_exists".to_string()],
            metadata: HashMap::from([
                ("resource_id".to_string(), resource_id.to_string()),
                ("query_type".to_string(), "contract".to_string()),
            ]),
        };
        
        // Query the domain
        let facts = self.domain_adapter.observe_fact(&fact_query).await?;
        
        // Extract existence information
        let exists = !facts.is_empty();
        
        // Extract metadata
        let metadata = if let Some(fact) = facts.first() {
            fact.data.clone()
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
    
    async fn validate_operation(
        &self, 
        resource_id: &ResourceId, 
        operation: &CrossDomainResourceOperation
    ) -> Result<bool> {
        // In a full implementation, this would check if the operation is allowed
        // For now, we'll assume all operations are valid for CosmWasm
        Ok(true)
    }
}

/// Adapter for EVM resource operations
pub struct EvmResourceAdapter {
    domain_adapter: Arc<dyn DomainAdapter>,
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
        resource_id: &ResourceId, 
        contents: &[u8], 
        metadata: &HashMap<String, String>
    ) -> Result<CrossDomainResourceResult> {
        // In a full implementation, this would create and submit an Ethereum transaction
        // to store the resource data in a smart contract
        
        // Create resource storage transaction
        let tx = crate::domain::Transaction {
            tx_type: "store_resource".to_string(),
            from: metadata.get("from").cloned().unwrap_or_default(),
            to: metadata.get("contract").cloned().unwrap_or_default(),
            data: contents.to_vec(),
            value: "0".to_string(),
            gas_limit: metadata.get("gas_limit").cloned().unwrap_or_else(|| "200000".to_string()),
            gas_price: metadata.get("gas_price").cloned(),
            nonce: metadata.get("nonce").cloned(),
            signature: None,
            metadata: metadata.clone(),
        };
        
        // Submit transaction
        let tx_id = self.domain_adapter.submit_transaction(tx).await?;
        
        // Wait for transaction to be confirmed
        let receipt = self.domain_adapter.wait_for_confirmation(&tx_id, Some(30000)).await?;
        
        // Return result
        Ok(CrossDomainResourceResult::Stored {
            resource_id: resource_id.clone(),
            domain_id: self.domain_adapter.domain_id().clone(),
            transaction_id: tx_id.to_string(),
            block_height: receipt.block_height,
            timestamp: None, // Would extract from receipt in a full implementation
        })
    }
    
    async fn retrieve_resource(
        &self, 
        resource_id: &ResourceId
    ) -> Result<CrossDomainResourceResult> {
        // In a full implementation, this would call an Ethereum contract to retrieve resource data
        
        // For now, we create a fact query to retrieve the resource data
        let fact_query = crate::domain::FactQuery {
            selectors: vec!["resource_data".to_string()],
            metadata: HashMap::from([
                ("resource_id".to_string(), resource_id.to_string()),
                ("query_type".to_string(), "contract".to_string()),
            ]),
        };
        
        // Query the domain
        let facts = self.domain_adapter.observe_fact(&fact_query).await?;
        
        // Extract resource data from facts
        if let Some(fact) = facts.first() {
            let contents = fact.get_data_as_bytes("data")
                .map_err(|e| Error::DomainDataError(format!("Failed to extract resource data: {}", e)))?;
                
            let mut metadata = HashMap::new();
            for (key, value) in &fact.data {
                if key != "data" {
                    metadata.insert(key.clone(), value.clone());
                }
            }
            
            Ok(CrossDomainResourceResult::Retrieved {
                resource_id: resource_id.clone(),
                domain_id: self.domain_adapter.domain_id().clone(),
                contents,
                metadata,
            })
        } else {
            Err(Error::NotFound(format!("Resource {} not found in domain {}", resource_id, self.domain_adapter.domain_id())))
        }
    }
    
    async fn verify_resource(
        &self, 
        resource_id: &ResourceId
    ) -> Result<CrossDomainResourceResult> {
        // Similar to retrieve, but just checks existence
        let fact_query = crate::domain::FactQuery {
            selectors: vec!["resource_exists".to_string()],
            metadata: HashMap::from([
                ("resource_id".to_string(), resource_id.to_string()),
                ("query_type".to_string(), "contract".to_string()),
            ]),
        };
        
        // Query the domain
        let facts = self.domain_adapter.observe_fact(&fact_query).await?;
        
        // Extract existence information
        let exists = !facts.is_empty();
        
        // Extract metadata
        let metadata = if let Some(fact) = facts.first() {
            fact.data.clone()
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
    
    async fn validate_operation(
        &self, 
        resource_id: &ResourceId, 
        operation: &CrossDomainResourceOperation
    ) -> Result<bool> {
        // In a full implementation, this would check if the operation is allowed
        // For now, we'll assume all operations are valid for EVM
        Ok(true)
    }
}

/// Factory for creating domain resource adapters
pub struct DomainResourceAdapterFactory {
    domain_registry: Arc<DomainRegistry>,
}

impl DomainResourceAdapterFactory {
    /// Create a new domain resource adapter factory
    pub fn new(domain_registry: Arc<DomainRegistry>) -> Self {
        Self { domain_registry }
    }
    
    /// Create a domain resource adapter for a specific domain
    pub async fn create_adapter(&self, domain_id: &DomainId) -> Result<Box<dyn DomainResourceAdapter>> {
        // Get the domain adapter
        let domain_adapter = self.domain_registry.get_adapter(domain_id)?;
        
        // Get domain info to determine adapter type
        let domain_info = domain_adapter.domain_info().await?;
        
        // Create the appropriate resource adapter based on domain type
        match domain_info.domain_type {
            DomainType::CosmWasm => {
                Ok(Box::new(CosmWasmResourceAdapter::new(domain_adapter)))
            },
            DomainType::EVM => {
                Ok(Box::new(EvmResourceAdapter::new(domain_adapter)))
            },
            _ => {
                Err(Error::UnsupportedOperation(format!("Unsupported domain type: {:?}", domain_info.domain_type)))
            }
        }
    }
    
    /// Create the most appropriate domain resource adapter based on selection strategy
    pub async fn create_adapter_by_strategy(
        &self,
        required_capabilities: &std::collections::HashSet<String>,
        preferences: &HashMap<String, String>,
        strategy: &dyn DomainSelectionStrategy,
    ) -> Result<Box<dyn DomainResourceAdapter>> {
        // Get all adapters
        let adapters = self.domain_registry.get_all_adapters()?;
        
        // Use the strategy to select a domain
        let domain_id = strategy.select_domain(&adapters, required_capabilities, preferences).await?;
        
        // Create the adapter for the selected domain
        self.create_adapter(&domain_id).await
    }
}

/// Manager for cross-domain resource operations
pub struct CrossDomainResourceManager {
    adapter_factory: Arc<DomainResourceAdapterFactory>,
    default_strategy: Box<dyn DomainSelectionStrategy>,
}

impl CrossDomainResourceManager {
    /// Create a new cross-domain resource manager
    pub fn new(
        domain_registry: Arc<DomainRegistry>,
        default_strategy: Box<dyn DomainSelectionStrategy>,
    ) -> Self {
        let adapter_factory = Arc::new(DomainResourceAdapterFactory::new(domain_registry));
        Self {
            adapter_factory,
            default_strategy,
        }
    }
    
    /// Execute a cross-domain resource operation
    pub async fn execute_operation(
        &self,
        operation: CrossDomainResourceOperation,
    ) -> Result<CrossDomainResourceResult> {
        // Handle operation based on type
        match &operation {
            CrossDomainResourceOperation::Store { resource_id, target_domain_id, contents, metadata } => {
                // Create adapter for target domain
                let adapter = self.adapter_factory.create_adapter(target_domain_id).await?;
                
                // Validate operation
                if !adapter.validate_operation(resource_id, &operation).await? {
                    return Err(Error::AccessDenied(format!(
                        "Operation not allowed for resource {} in domain {}", 
                        resource_id, target_domain_id
                    )));
                }
                
                // Store the resource
                adapter.store_resource(resource_id, contents, metadata).await
            },
            
            CrossDomainResourceOperation::Retrieve { resource_id, source_domain_id } => {
                // Create adapter for source domain
                let adapter = self.adapter_factory.create_adapter(source_domain_id).await?;
                
                // Validate operation
                if !adapter.validate_operation(resource_id, &operation).await? {
                    return Err(Error::AccessDenied(format!(
                        "Operation not allowed for resource {} in domain {}", 
                        resource_id, source_domain_id
                    )));
                }
                
                // Retrieve the resource
                adapter.retrieve_resource(resource_id).await
            },
            
            CrossDomainResourceOperation::Transfer { 
                resource_id, 
                source_domain_id, 
                target_domain_id, 
                metadata 
            } => {
                // Create adapters for source and target domains
                let source_adapter = self.adapter_factory.create_adapter(source_domain_id).await?;
                let target_adapter = self.adapter_factory.create_adapter(target_domain_id).await?;
                
                // Validate operations
                if !source_adapter.validate_operation(resource_id, &operation).await? {
                    return Err(Error::AccessDenied(format!(
                        "Transfer operation not allowed for resource {} in source domain {}", 
                        resource_id, source_domain_id
                    )));
                }
                
                if !target_adapter.validate_operation(resource_id, &operation).await? {
                    return Err(Error::AccessDenied(format!(
                        "Transfer operation not allowed for resource {} in target domain {}", 
                        resource_id, target_domain_id
                    )));
                }
                
                // Retrieve the resource from the source domain
                let result = source_adapter.retrieve_resource(resource_id).await?;
                
                // Extract contents and metadata
                if let CrossDomainResourceResult::Retrieved { contents, metadata: source_metadata, .. } = result {
                    // Merge metadata
                    let mut combined_metadata = source_metadata;
                    for (key, value) in metadata {
                        combined_metadata.insert(key.clone(), value.clone());
                    }
                    
                    // Store the resource in the target domain
                    let store_result = target_adapter.store_resource(resource_id, &contents, &combined_metadata).await?;
                    
                    // Return transfer result
                    if let CrossDomainResourceResult::Stored { transaction_id, block_height, timestamp, .. } = store_result {
                        Ok(CrossDomainResourceResult::Transferred {
                            resource_id: resource_id.clone(),
                            source_domain_id: source_domain_id.clone(),
                            target_domain_id: target_domain_id.clone(),
                            transaction_id,
                            block_height,
                            timestamp,
                        })
                    } else {
                        Err(Error::DomainError("Unexpected result type from store operation".to_string()))
                    }
                } else {
                    Err(Error::DomainError("Unexpected result type from retrieve operation".to_string()))
                }
            },
            
            CrossDomainResourceOperation::Verify { resource_id, domain_id } => {
                // Create adapter for domain
                let adapter = self.adapter_factory.create_adapter(domain_id).await?;
                
                // Validate operation
                if !adapter.validate_operation(resource_id, &operation).await? {
                    return Err(Error::AccessDenied(format!(
                        "Verify operation not allowed for resource {} in domain {}", 
                        resource_id, domain_id
                    )));
                }
                
                // Verify the resource
                adapter.verify_resource(resource_id).await
            },
        }
    }
    
    /// Store a resource in the most appropriate domain based on selection strategy
    pub async fn store_resource_by_strategy(
        &self,
        resource_id: ResourceId,
        contents: Vec<u8>,
        metadata: HashMap<String, String>,
        required_capabilities: std::collections::HashSet<String>,
        preferences: HashMap<String, String>,
    ) -> Result<CrossDomainResourceResult> {
        // Create the most appropriate domain resource adapter based on selection strategy
        let adapter = self.adapter_factory.create_adapter_by_strategy(
            &required_capabilities,
            &preferences,
            self.default_strategy.as_ref(),
        ).await?;
        
        // Create storage operation
        let operation = CrossDomainResourceOperation::Store {
            resource_id: resource_id.clone(),
            target_domain_id: adapter.domain_id().clone(),
            contents,
            metadata,
        };
        
        // Execute the operation
        self.execute_operation(operation).await
    }
    
    /// Get the adapter factory
    pub fn adapter_factory(&self) -> &Arc<DomainResourceAdapterFactory> {
        &self.adapter_factory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::selection::PreferredDomainStrategy;
    
    // Test implementation would go here
} 