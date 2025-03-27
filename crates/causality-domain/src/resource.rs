// Resource integration for domains
// Original file: src/domain/resource_integration.rs

// Domain Resource Integration
//
// This module provides the integration between domain adapters and the resource register system,
// enabling cross-domain resource operations and storage.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use causality_types::{Error, Result};
use crate::domain::{BlockHeight, Timestamp};
use causality_engine_types::FactType;
use causality_crypto::ContentId;
use crate::domain::{
    adapter::FactQuery,
    DomainAdapter, DomainRegistry, DomainId,
    selection::DomainSelectionStrategy, Transaction
};
use :ResourceRegister:causality_core::resource::Resource::{ResourceRegister, RegisterState};
#[cfg(feature = "cosmwasm_zk")]
use causality_domain_succinct::ZkResourceAdapter;

/// Represents a cross-domain resource operation
#[derive(Debug, Clone)]
pub enum CrossDomainResourceOperation {
    /// Store a resource in a domain
    Store {
        resource_id: ContentId,
        target_domain_id: DomainId,
        contents: Vec<u8>,
        metadata: HashMap<String, String>,
    },
    
    /// Retrieve a resource from a domain
    Retrieve {
        resource_id: ContentId,
        source_domain_id: DomainId,
    },
    
    /// Transfer a resource between domains
    Transfer {
        resource_id: ContentId,
        source_domain_id: DomainId,
        target_domain_id: DomainId,
        metadata: HashMap<String, String>,
    },
    
    /// Verify a resource exists in a domain
    Verify {
        resource_id: ContentId,
        domain_id: DomainId,
    },

    /// Store a ResourceRegister in a domain (for unified model)
    StoreRegister {
        register: :ResourceRegister:causality_core::resource::Resource::ResourceRegister,
        target_domain_id: DomainId,
    },
    
    /// Transfer a ResourceRegister between domains (for unified model)
    TransferRegister {
        register: :ResourceRegister:causality_core::resource::Resource::ResourceRegister,
        source_domain_id: DomainId,
        target_domain_id: DomainId,
        additional_metadata: HashMap<String, String>,
    }
}

/// Result of a cross-domain resource operation
#[derive(Debug, Clone)]
pub enum CrossDomainResourceResult {
    /// Resource has been stored successfully with a transaction receipt
    Stored {
        resource_id: ContentId,
        domain_id: DomainId,
        transaction_id: String,
        block_height: Option<BlockHeight>,
        timestamp: Option<Timestamp>,
    },
    
    /// Resource has been retrieved successfully
    Retrieved {
        resource_id: ContentId,
        domain_id: DomainId,
        contents: Vec<u8>,
        metadata: HashMap<String, String>,
    },
    
    /// Resource has been transferred successfully
    Transferred {
        resource_id: ContentId,
        source_domain_id: DomainId,
        target_domain_id: DomainId,
        transaction_id: String,
        block_height: Option<BlockHeight>,
        timestamp: Option<Timestamp>,
    },
    
    /// Resource verification result
    Verified {
        resource_id: ContentId,
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
        resource_id: &ContentId, 
        contents: &[u8], 
        metadata: &HashMap<String, String>
    ) -> Result<CrossDomainResourceResult>;
    
    /// Retrieve a resource from the domain
    async fn retrieve_resource(
        &self, 
        resource_id: &ContentId
    ) -> Result<CrossDomainResourceResult>;
    
    /// Verify a resource exists in the domain
    async fn verify_resource(
        &self, 
        resource_id: &ContentId
    ) -> Result<CrossDomainResourceResult>;
    
    /// Check if a resource operation is allowed in this domain
    async fn validate_operation(
        &self, 
        resource_id: &ContentId, 
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
        resource_id: &ContentId, 
        contents: &[u8], 
        metadata: &HashMap<String, String>
    ) -> Result<CrossDomainResourceResult> {
        // In a full implementation, this would create and submit a CosmWasm contract transaction
        // to store the resource data
        
        // Create resource storage transaction
        let tx = Transaction {
            data: contents.to_vec(),
            transaction_type: "store_resource".to_string(),
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
            block_height: Some(receipt.block_height),
            timestamp: None, // Would extract from receipt in a full implementation
        })
    }
    
    async fn retrieve_resource(
        &self, 
        resource_id: &ContentId
    ) -> Result<CrossDomainResourceResult> {
        // For now, we create a fact query to retrieve the resource data
        let fact_query = FactQuery {
            domain_id: self.domain_adapter.domain_id().clone(),
            fact_type: "resource_data".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("resource_id".to_string(), resource_id.to_string());
                params.insert("operation".to_string(), "store".to_string());
                params
            },
            block_height: None,
            block_hash: None,
            timestamp: None,
        };
        
        // Query the domain
        let (fact_type, fact_meta) = self.domain_adapter.observe_fact(&fact_query).await?;
        
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
            _ => Err(Error::DomainDataError(format!("Unexpected fact type returned for resource {}", resource_id)))
        }
    }
    
    async fn verify_resource(
        &self, 
        resource_id: &ContentId
    ) -> Result<CrossDomainResourceResult> {
        // Similar to retrieve, but just checks existence
        let fact_query = FactQuery {
            domain_id: self.domain_adapter.domain_id().clone(),
            fact_type: "resource_exists".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("resource_id".to_string(), resource_id.to_string());
                params.insert("query_type".to_string(), "contract".to_string());
                params
            },
            block_height: None,
            block_hash: None,
            timestamp: None,
        };
        
        // Query the domain
        let result = self.domain_adapter.observe_fact(&fact_query).await;
        
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
    
    async fn validate_operation(
        &self, 
        resource_id: &ContentId, 
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
        resource_id: &ContentId, 
        contents: &[u8], 
        metadata: &HashMap<String, String>
    ) -> Result<CrossDomainResourceResult> {
        // In a full implementation, this would create and submit an Ethereum transaction
        // to store the resource data in a smart contract
        
        // Create resource storage transaction
        let tx = Transaction {
            data: contents.to_vec(),
            transaction_type: "store_resource".to_string(),
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
            block_height: Some(receipt.block_height),
            timestamp: None, // Would extract from receipt in a full implementation
        })
    }
    
    async fn retrieve_resource(
        &self, 
        resource_id: &ContentId
    ) -> Result<CrossDomainResourceResult> {
        // For now, we create a fact query to retrieve the resource data
        let fact_query = FactQuery {
            domain_id: self.domain_adapter.domain_id().clone(),
            fact_type: "resource_data".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("resource_id".to_string(), resource_id.to_string());
                params.insert("operation".to_string(), "store".to_string());
                params
            },
            block_height: None,
            block_hash: None,
            timestamp: None,
        };
        
        // Query the domain
        let (fact_type, fact_meta) = self.domain_adapter.observe_fact(&fact_query).await?;
        
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
            _ => Err(Error::DomainDataError(format!("Unexpected fact type returned for resource {}", resource_id)))
        }
    }
    
    async fn verify_resource(
        &self, 
        resource_id: &ContentId
    ) -> Result<CrossDomainResourceResult> {
        // Similar to retrieve, but just checks existence
        let fact_query = FactQuery {
            domain_id: self.domain_adapter.domain_id().clone(),
            fact_type: "resource_exists".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("resource_id".to_string(), resource_id.to_string());
                params
            },
            block_height: None,
            block_hash: None,
            timestamp: None,
        };
        
        // Query the domain
        let result = self.domain_adapter.observe_fact(&fact_query).await;
        
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
    
    async fn validate_operation(
        &self, 
        resource_id: &ContentId, 
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
        // Get the domain adapter from the registry
        let domain_adapter = self.domain_registry.get_adapter(domain_id)
            .ok_or_else(|| Error::DomainAdapterNotFound(domain_id.clone()))?;
        
        // Get domain info
        let domain_info = self.domain_registry.get_domain_info(domain_id)
            .ok_or_else(|| Error::DomainNotFound(domain_id.clone()))?;
        
        // Create the appropriate resource adapter based on domain type
        match domain_info.domain_type.as_str() {
            "cosmos" | "cosmwasm" => {
                Ok(Box::new(CosmWasmResourceAdapter::new(domain_adapter)))
            },
            "ethereum" | "evm" => {
                Ok(Box::new(EvmResourceAdapter::new(domain_adapter)))
            },
            #[cfg(feature = "cosmwasm_zk")]
            "zk" | "zkvm" | "risc0" | "succinct" => {
                // Use the new ZK resource adapter for ZK domains
                Ok(Box::new(causality_domain_succinct::ZkResourceAdapter::new(domain_adapter)))
            },
            #[cfg(not(feature = "cosmwasm_zk"))]
            "zk" | "zkvm" | "risc0" | "succinct" => {
                // Without ZK feature, return an error
                Err(Error::UnsupportedDomainType(format!("{} (requires 'cosmwasm_zk' feature)", domain_info.domain_type)))
            },
            _ => {
                Err(Error::UnsupportedDomainType(domain_info.domain_type.clone()))
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

            CrossDomainResourceOperation::StoreRegister { register, target_domain_id } => {
                // Create adapter for target domain
                let adapter = self.adapter_factory.create_adapter(target_domain_id).await?;
                
                // Validate operation
                if !adapter.validate_operation(&register.id, &operation).await? {
                    return Err(Error::AccessDenied(format!(
                        "StoreRegister operation not allowed for resource {} in domain {}", 
                        register.id, target_domain_id
                    )));
                }
                
                // Store the resource
                adapter.store_resource(&register.id, &register.contents, &register.metadata).await
            },
            
            CrossDomainResourceOperation::TransferRegister { register, source_domain_id, target_domain_id, additional_metadata } => {
                // Validate the source domain
                let source_adapter = self.adapter_factory.create_adapter(&source_domain_id).await?;
                
                // Check if the register exists in the source domain
                let verify_result = source_adapter.verify_resource(&register.id).await?;
                
                match verify_result {
                    CrossDomainResourceResult::Verified { exists, .. } if exists => {
                        // Register exists in source domain, proceed with transfer
                    },
                    _ => {
                        return Err(Error::DomainDataError(format!(
                            "Register {} not found in source domain {}", register.id, source_domain_id
                        )));
                    }
                }
                
                // Validate the target domain
                let target_adapter = self.adapter_factory.create_adapter(&target_domain_id).await?;
                
                // Create transfer operation for ResourceRegister
                let operation = CrossDomainResourceOperation::TransferRegister {
                    register: register.clone(),
                    source_domain_id,
                    target_domain_id,
                    additional_metadata,
                };
                
                // Validate operations on both source and target domains
                if !source_adapter.validate_operation(&register.id, &operation).await? {
                    return Err(Error::AccessDenied(format!(
                        "Transfer operation not allowed for register {} in source domain {}", 
                        register.id, source_domain_id
                    )));
                }
                
                if !target_adapter.validate_operation(&register.id, &operation).await? {
                    return Err(Error::AccessDenied(format!(
                        "Transfer operation not allowed for register {} in target domain {}", 
                        register.id, target_domain_id
                    )));
                }
                
                // Execute the transfer operation
                self.execute_operation(operation).await
            },
        }
    }
    
    /// Store a resource in the most appropriate domain based on selection strategy
    pub async fn store_resource_by_strategy(
        &self,
        resource_id: ContentId,
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
    
    /// Store a ResourceRegister in the most appropriate domain based on selection strategy
    pub async fn store_resource_register_by_strategy(
        &self,
        register: :ResourceRegister:causality_core::resource::Resource::ResourceRegister,
        required_capabilities: std::collections::HashSet<String>,
        preferences: HashMap<String, String>,
    ) -> Result<CrossDomainResourceResult> {
        // Create the most appropriate domain resource adapter based on selection strategy
        let adapter = self.adapter_factory.create_adapter_by_strategy(
            &required_capabilities,
            &preferences,
            self.default_strategy.as_ref(),
        ).await?;
        
        // Create storage operation for ResourceRegister
        let operation = CrossDomainResourceOperation::StoreRegister {
            register,
            target_domain_id: adapter.domain_id().clone(),
        };
        
        // Execute the operation
        self.execute_operation(operation).await
    }
    
    /// Transfer a ResourceRegister between domains
    pub async fn transfer_resource_register(
        &self,
        register: ResourceRegister,
        source_domain_id: DomainId,
        target_domain_id: DomainId,
        additional_metadata: HashMap<String, String>,
    ) -> Result<CrossDomainResourceResult> {
        // Validate the source domain
        let source_adapter = self.adapter_factory.create_adapter(&source_domain_id).await?;
        
        // Check if the register exists in the source domain
        let verify_result = source_adapter.verify_resource(&register.id).await?;
        
        match verify_result {
            CrossDomainResourceResult::Verified { exists, .. } if exists => {
                // Register exists in source domain, proceed with transfer
            },
            _ => {
                return Err(Error::DomainDataError(format!(
                    "Register {} not found in source domain {}", register.id, source_domain_id
                )));
            }
        }
        
        // Validate the target domain
        let target_adapter = self.adapter_factory.create_adapter(&target_domain_id).await?;
        
        // Create transfer operation for ResourceRegister
        let operation = CrossDomainResourceOperation::TransferRegister {
            register: register.clone(),
            source_domain_id,
            target_domain_id,
            additional_metadata,
        };
        
        // Validate operations on both source and target domains
        if !source_adapter.validate_operation(&register.id, &operation).await? {
            return Err(Error::AccessDenied(format!(
                "Transfer operation not allowed for register {} in source domain {}", 
                register.id, source_domain_id
            )));
        }
        
        if !target_adapter.validate_operation(&register.id, &operation).await? {
            return Err(Error::AccessDenied(format!(
                "Transfer operation not allowed for register {} in target domain {}", 
                register.id, target_domain_id
            )));
        }
        
        // Execute the transfer operation
        self.execute_operation(operation).await
    }
    
    /// Batch transfer multiple ResourceRegisters between domains
    pub async fn batch_transfer_resource_registers(
        &self,
        registers: Vec<ResourceRegister>,
        source_domain_id: DomainId,
        target_domain_id: DomainId,
        additional_metadata: HashMap<String, String>,
    ) -> Result<Vec<Result<CrossDomainResourceResult>>> {
        let mut results = Vec::new();
        
        for register in registers {
            let result = self.transfer_resource_register(
                register,
                source_domain_id.clone(),
                target_domain_id.clone(),
                additional_metadata.clone()
            ).await;
            
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Retrieve a ResourceRegister from a domain
    pub async fn retrieve_resource_register(
        &self,
        resource_id: ContentId,
        source_domain_id: DomainId,
    ) -> Result<ResourceRegister> {
        // Create retrieve operation
        let operation = CrossDomainResourceOperation::Retrieve {
            resource_id: resource_id.clone(),
            source_domain_id,
        };
        
        // Execute the operation
        let result = self.execute_operation(operation).await?;
        
        // Convert the result to a ResourceRegister
        match result {
            CrossDomainResourceResult::Retrieved { resource_id, contents, metadata, .. } => {
                // Create a ResourceRegister from the retrieved data
                let register = ResourceRegister {
                    id: resource_id,
                    resource_logic: Default::default(),
                    fungibility_domain: Default::default(),
                    quantity: Default::default(),
                    metadata: metadata.clone(),
                    state: RegisterState::Active,
                    nullifier_key: None,
                    controller_label: None,
                    observed_at: Default::default(),
                    storage_strategy: Default::default(),
                    contents,
                    version: 1,
                    controller: None,
                    lifecycle_manager: None,
                };
                
                Ok(register)
            },
            _ => Err(Error::DomainDataError(format!(
                "Unexpected result type from retrieve operation for resource {}", resource_id
            ))),
        }
    }
    
    /// Verify a ResourceRegister exists in a domain
    pub async fn verify_resource_register(
        &self,
        resource_id: ContentId,
        domain_id: DomainId,
    ) -> Result<bool> {
        // Create verify operation
        let operation = CrossDomainResourceOperation::Verify {
            resource_id: resource_id.clone(),
            domain_id,
        };
        
        // Execute the operation
        let result = self.execute_operation(operation).await?;
        
        // Extract the verification result
        match result {
            CrossDomainResourceResult::Verified { exists, .. } => {
                Ok(exists)
            },
            _ => Err(Error::DomainDataError(format!(
                "Unexpected result type from verify operation for resource {}", resource_id
            ))),
        }
    }
    
    /// Get the adapter factory
    pub fn adapter_factory(&self) -> &Arc<DomainResourceAdapterFactory> {
        &self.adapter_factory
    }

    // Create a ResourceRegister using its base constructor for testing or utility purposes
    // This is a helper method to ensure we properly use the ResourceRegister type
    #[cfg(test)]
    pub fn create_test_register(&self, id: ContentId) -> ResourceRegister {
        ResourceRegister {
            id,
            resource_logic: Default::default(),
            fungibility_domain: Default::default(),
            quantity: Default::default(),
            metadata: Default::default(),
            state: Default::default(),
            nullifier_key: None,
            controller_label: None,
            observed_at: Default::default(),
            storage_strategy: Default::default(),
            contents: Vec::new(),
            version: 1,
            controller: None,
            lifecycle_manager: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_domain::PreferredDomainStrategy;
    
    // Test implementation would go here
} 
