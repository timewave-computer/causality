// Storage Effects for ResourceRegister
//
// This module implements effects for storage operations used with
// the unified ResourceRegister model as defined in ADR-021.
// It provides effects for storing, reading, and managing ResourceRegisters
// with various storage strategies.

use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use ethers::types::Address as EthAddress;

use crate::effect::{
    Effect, EffectContext, EffectOutcome, EffectResult, EffectError,
    ExecutionBoundary
};
use crate::resource::api::Right;
use crate::resource::resource_register::{
    ResourceRegister,
    NullifierId, StorageStrategy, StateVisibility
};
use crate::crypto::hash::{ContentId, ContentAddressed};
use crate::crypto::merkle::Commitment;
use crate::types::{*};
use crate::crypto::hash::ContentId;;
use crate::domain::{DomainInfo, DomainType, DomainStatus};
use crate::domain_adapters::evm::storage_strategy::EthereumStorageEffectFactory;
use crate::domain_adapters::cosmwasm::storage_strategy::CosmWasmStorageEffectFactory;
use crate::domain_adapters::cosmwasm::CosmWasmAdapterConfig;
use std::str::FromStr;

use super::types::{ResourceChangeType, ResourceChange};

/// Result of a storage operation
#[derive(Debug, Clone)]
pub enum StoreResult {
    /// Storage succeeded with a transaction ID
    Success { transaction_id: String },
    /// Storage was deferred (will be completed later)
    Deferred { operation_id: String },
    /// Storage failed
    Failure { reason: String },
}

/// Result of a read operation
#[derive(Debug, Clone)]
pub enum ReadResult {
    /// Full register data was read
    FullRegister(ResourceRegister),
    /// Partial data was read
    PartialData { 
        fields: HashSet<String>,
        values: serde_json::Value,
    },
    /// Only commitment was read
    CommitmentOnly(Commitment),
    /// Read failed
    Failure { reason: String },
}

/// Effect for storing a register on-chain
pub struct StoreOnChainEffect {
    content_id: ContentId,
    fields: HashSet<String>,
    domain_id: DomainId,
    invoker: Address,
    display_name: String,
}

impl StoreOnChainEffect {
    /// Create a new effect to store a register on-chain
    pub fn new(
        content_id: ContentId,
        fields: HashSet<String>,
        domain_id: DomainId,
        invoker: Address,
    ) -> Self {
        Self {
            content_id,
            fields,
            domain_id,
            invoker,
            display_name: "Store Register On-Chain".to_string(),
        }
    }
    
    /// Create from a legacy ContentId (for backward compatibility)
    pub fn from_resource_id(
        resource_id: ContentId,
        fields: HashSet<String>,
        domain_id: DomainId,
        invoker: Address,
    ) -> Self {
        // Convert ContentId to ContentId
        let content_id = ContentId::from(resource_id);
        Self::new(content_id, fields, domain_id, invoker)
    }
}

#[async_trait]
impl Effect for StoreOnChainEffect {
    fn name(&self) -> &str {
        "store_on_chain"
    }
    
    fn description(&self) -> &str {
        "Stores register data on-chain"
    }
    
    fn required_capabilities(&self) -> Vec<(ContentId, Right)> {
        // Convert ContentId back to ContentId for compatibility with capability system
        let resource_id = ContentId::from(self.content_id.clone());
        vec![(resource_id, Right::Write)]
    }
    
    async fn execute(&self, context: EffectContext) -> EffectResult<EffectOutcome> {
        // In a real implementation, this would interact with the domain adapter
        // For now, we'll implement a simple success result
        
        let result = StoreResult::Success {
            transaction_id: format!("tx-{}-{}", self.domain_id, self.content_id),
        };
        
        // For resource changes, convert ContentId back to ContentId for compatibility
        let resource_id = ContentId::from(self.content_id.clone());
        
        let resource_change = ResourceChange {
            resource_id,
            change_type: ResourceChangeType::Updated,
            previous_state_hash: Some("previous-hash".to_string()),
            new_state_hash: "new-hash".to_string(),
        };
        
        Ok(EffectOutcome {
            execution_id: context.execution_id,
            success: true,
            result: Some(serde_json::to_value(result).map_err(|e| 
                EffectError::ExecutionError(format!("Failed to serialize result: {}", e)))?),
            error: None,
            resource_changes: vec![resource_change],
            metadata: Default::default(),
        })
    }
    
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        // Storage effects execute outside the system (on-chain)
        boundary == ExecutionBoundary::OutsideSystem
    }
    
    fn preferred_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::OutsideSystem
    }
}

/// Effect for reading a register from the chain
pub struct ReadFromChainEffect {
    content_id: ContentId,
    fields: HashSet<String>,
    domain_id: DomainId,
    invoker: Address,
    display_name: String,
}

impl ReadFromChainEffect {
    /// Create a new effect to read a register from the chain
    pub fn new(
        content_id: ContentId,
        fields: HashSet<String>,
        domain_id: DomainId,
        invoker: Address,
    ) -> Self {
        Self {
            content_id,
            fields,
            domain_id,
            invoker,
            display_name: "Read Register From Chain".to_string(),
        }
    }
    
    /// Create from a legacy ContentId (for backward compatibility)
    pub fn from_resource_id(
        resource_id: ContentId,
        fields: HashSet<String>,
        domain_id: DomainId,
        invoker: Address,
    ) -> Self {
        // Convert ContentId to ContentId
        let content_id = ContentId::from(resource_id);
        Self::new(content_id, fields, domain_id, invoker)
    }
}

#[async_trait]
impl Effect for ReadFromChainEffect {
    fn name(&self) -> &str {
        "read_from_chain"
    }
    
    fn description(&self) -> &str {
        "Reads register data from on-chain storage"
    }
    
    fn required_capabilities(&self) -> Vec<(ContentId, Right)> {
        // Convert ContentId back to ContentId for compatibility with capability system
        let resource_id = ContentId::from(self.content_id.clone());
        vec![(resource_id, Right::Read)]
    }
    
    async fn execute(&self, context: EffectContext) -> EffectResult<EffectOutcome> {
        // In a real implementation, this would read from the on-chain storage
        // For now, we'll return mock data
        
        // Create a mock ResourceRegister with the unified model's fields
        let mock_register = ResourceRegister::new(
            self.content_id.clone(),
            crate::resource::resource_register::ResourceLogic::Fungible,
            crate::resource::resource_register::FungibilityDomain("ETH".to_string()),
            crate::resource::resource_register::Quantity(100),
            Default::default(),
            StorageStrategy::FullyOnChain { 
                visibility: StateVisibility::Public 
            },
        );
        
        let result = ReadResult::FullRegister(mock_register);
        
        Ok(EffectOutcome {
            execution_id: context.execution_id,
            success: true,
            result: Some(serde_json::to_value(result).map_err(|e| 
                EffectError::ExecutionError(format!("Failed to serialize result: {}", e)))?),
            error: None,
            resource_changes: vec![],
            metadata: Default::default(),
        })
    }
    
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        // Read effects can execute in both boundaries
        true
    }
    
    fn preferred_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::OutsideSystem
    }
}

/// Effect for storing a commitment on-chain
pub struct StoreCommitmentEffect {
    content_id: ContentId,
    commitment: Commitment,
    domain_id: DomainId,
    invoker: Address,
    display_name: String,
}

impl StoreCommitmentEffect {
    /// Create a new effect to store a commitment on-chain
    pub fn new(
        content_id: ContentId,
        commitment: Commitment,
        domain_id: DomainId,
        invoker: Address,
    ) -> Self {
        Self {
            content_id,
            commitment,
            domain_id,
            invoker,
            display_name: "Store Commitment On-Chain".to_string(),
        }
    }
    
    /// Create from a legacy ContentId (for backward compatibility)
    pub fn from_resource_id(
        resource_id: ContentId,
        commitment: Commitment,
        domain_id: DomainId,
        invoker: Address,
    ) -> Self {
        // Convert ContentId to ContentId
        let content_id = ContentId::from(resource_id);
        Self::new(content_id, commitment, domain_id, invoker)
    }
}

#[async_trait]
impl Effect for StoreCommitmentEffect {
    fn name(&self) -> &str {
        "store_commitment"
    }
    
    fn description(&self) -> &str {
        "Stores a commitment to register data on-chain"
    }
    
    fn required_capabilities(&self) -> Vec<(ContentId, Right)> {
        // Convert ContentId back to ContentId for compatibility with capability system
        let resource_id = ContentId::from(self.content_id.clone());
        vec![(resource_id, Right::Write)]
    }
    
    async fn execute(&self, context: EffectContext) -> EffectResult<EffectOutcome> {
        // In a real implementation, this would store the commitment on-chain
        // For now, we'll implement a simple success result
        
        let result = StoreResult::Success {
            transaction_id: format!("tx-commitment-{}-{}", self.domain_id, self.content_id),
        };
        
        // For resource changes, convert ContentId back to ContentId for compatibility
        let resource_id = ContentId::from(self.content_id.clone());
        
        let resource_change = ResourceChange {
            resource_id,
            change_type: ResourceChangeType::Updated,
            previous_state_hash: Some("previous-hash".to_string()),
            new_state_hash: "new-commitment-hash".to_string(),
        };
        
        Ok(EffectOutcome {
            execution_id: context.execution_id,
            success: true,
            result: Some(serde_json::to_value(result).map_err(|e| 
                EffectError::ExecutionError(format!("Failed to serialize result: {}", e)))?),
            error: None,
            resource_changes: vec![resource_change],
            metadata: Default::default(),
        })
    }
    
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        // Storage effects execute outside the system (on-chain)
        boundary == ExecutionBoundary::OutsideSystem
    }
    
    fn preferred_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::OutsideSystem
    }
}

/// Effect for storing a nullifier on-chain
pub struct StoreNullifierEffect {
    content_id: ContentId,
    nullifier: NullifierId,
    domain_id: DomainId,
    invoker: Address,
    display_name: String,
}

impl StoreNullifierEffect {
    /// Create a new effect to store a nullifier on-chain
    pub fn new(
        content_id: ContentId,
        nullifier: NullifierId,
        domain_id: DomainId,
        invoker: Address,
    ) -> Self {
        Self {
            content_id,
            nullifier,
            domain_id,
            invoker,
            display_name: "Store Nullifier On-Chain".to_string(),
        }
    }
    
    /// Create from a legacy ContentId (for backward compatibility)
    pub fn from_resource_id(
        resource_id: ContentId,
        nullifier: NullifierId,
        domain_id: DomainId,
        invoker: Address,
    ) -> Self {
        // Convert ContentId to ContentId
        let content_id = ContentId::from(resource_id);
        Self::new(content_id, nullifier, domain_id, invoker)
    }
}

#[async_trait]
impl Effect for StoreNullifierEffect {
    fn name(&self) -> &str {
        "store_nullifier"
    }
    
    fn description(&self) -> &str {
        "Stores a nullifier on-chain to prevent double-spending"
    }
    
    fn required_capabilities(&self) -> Vec<(ContentId, Right)> {
        // Convert ContentId back to ContentId for compatibility with capability system
        let resource_id = ContentId::from(self.content_id.clone());
        vec![(resource_id, Right::Write)]
    }
    
    async fn execute(&self, context: EffectContext) -> EffectResult<EffectOutcome> {
        // In a real implementation, this would store the nullifier on-chain
        // For now, we'll implement a simple success result
        
        let result = StoreResult::Success {
            transaction_id: format!("tx-nullifier-{}-{}", self.domain_id, self.content_id),
        };
        
        // For resource changes, convert ContentId back to ContentId for compatibility
        let resource_id = ContentId::from(self.content_id.clone());
        
        let resource_change = ResourceChange {
            resource_id,
            change_type: ResourceChangeType::Updated,
            previous_state_hash: Some("previous-hash".to_string()),
            new_state_hash: "new-nullifier-hash".to_string(),
        };
        
        Ok(EffectOutcome {
            execution_id: context.execution_id,
            success: true,
            result: Some(serde_json::to_value(result).map_err(|e| 
                EffectError::ExecutionError(format!("Failed to serialize result: {}", e)))?),
            error: None,
            resource_changes: vec![resource_change],
            metadata: Default::default(),
        })
    }
    
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        // Storage effects execute outside the system (on-chain)
        boundary == ExecutionBoundary::OutsideSystem
    }
    
    fn preferred_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::OutsideSystem
    }
}

/// Create factory functions for constructing storage effects

/// Create a store on-chain effect for a ResourceRegister using ContentId
pub fn create_store_on_chain_effect(
    content_id: ContentId,
    fields: HashSet<String>,
    domain_id: DomainId,
    invoker: Address,
) -> Arc<dyn Effect> {
    Arc::new(StoreOnChainEffect::new(
        content_id,
        fields,
        domain_id,
        invoker,
    ))
}

/// Create a read from chain effect for a ResourceRegister using ContentId
pub fn create_read_from_chain_effect(
    content_id: ContentId,
    fields: HashSet<String>,
    domain_id: DomainId,
    invoker: Address,
) -> Arc<dyn Effect> {
    Arc::new(ReadFromChainEffect::new(
        content_id,
        fields,
        domain_id,
        invoker,
    ))
}

/// Create a store commitment effect for a ResourceRegister using ContentId
pub fn create_store_commitment_effect(
    content_id: ContentId,
    commitment: Commitment,
    domain_id: DomainId,
    invoker: Address,
) -> Arc<dyn Effect> {
    Arc::new(StoreCommitmentEffect::new(
        content_id,
        commitment,
        domain_id,
        invoker,
    ))
}

/// Create a store nullifier effect for a ResourceRegister using ContentId
pub fn create_store_nullifier_effect(
    content_id: ContentId,
    nullifier: NullifierId,
    domain_id: DomainId,
    invoker: Address,
) -> Arc<dyn Effect> {
    Arc::new(StoreNullifierEffect::new(
        content_id,
        nullifier,
        domain_id,
        invoker,
    ))
}

/// Backward compatibility: Create a store on-chain effect for a ContentId
pub fn create_store_on_chain_effect_from_resource_id(
    resource_id: ContentId,
    fields: HashSet<String>,
    domain_id: DomainId,
    invoker: Address,
) -> Arc<dyn Effect> {
    let content_id = ContentId::from(resource_id);
    create_store_on_chain_effect(content_id, fields, domain_id, invoker)
}

/// Backward compatibility: Create a read from chain effect for a ContentId
pub fn create_read_from_chain_effect_from_resource_id(
    resource_id: ContentId,
    fields: HashSet<String>,
    domain_id: DomainId,
    invoker: Address,
) -> Arc<dyn Effect> {
    let content_id = ContentId::from(resource_id);
    create_read_from_chain_effect(content_id, fields, domain_id, invoker)
}

/// Create a domain-specific storage effect for storing a register on-chain
pub fn create_domain_specific_store_effect(
    content_id: ContentId,
    fields: HashSet<String>,
    domain_id: DomainId,
    invoker: Address,
    domain_info: &DomainInfo,
) -> Result<Arc<dyn Effect>, EffectError> {
    // For backward compatibility
    let resource_id = ContentId::from(content_id.clone());
    
    match domain_info.domain_type {
        DomainType::EVM => {
            // Create an Ethereum-specific storage effect
            // We need to extract the contract address from domain info
            let contract_address_str = domain_info.metadata.get("register_contract")
                .and_then(|v| v.as_str())
                .ok_or_else(|| EffectError::ConfigurationError(
                    "Missing register_contract address in domain metadata".to_string()
                ))?;
                
            let contract_address = ethers::types::Address::from_str(contract_address_str)
                .map_err(|e| EffectError::ConfigurationError(
                    format!("Invalid contract address: {}", e)
                ))?;
                
            let rpc_url = domain_info.endpoints.get(0)
                .ok_or_else(|| EffectError::ConfigurationError(
                    "Missing RPC URL in domain endpoints".to_string()
                ))?;
                
            // Create the factory
            let factory = EthereumStorageEffectFactory::new(
                contract_address,
                rpc_url.clone(),
                domain_id.clone(),
            );
            
            // Create the effect (using ContentId for backward compatibility)
            factory.create_store_effect(resource_id, fields, invoker)
                .map_err(|e| EffectError::ExecutionError(e.to_string()))
        },
        DomainType::CosmWasm => {
            // Create a CosmWasm-specific storage effect
            // We need to extract contract address and chain info from domain info
            let contract_address = domain_info.metadata.get("register_contract")
                .and_then(|v| v.as_str())
                .ok_or_else(|| EffectError::ConfigurationError(
                    "Missing register_contract address in domain metadata".to_string()
                ))?
                .to_string();
                
            let grpc_url = domain_info.endpoints.get(0)
                .ok_or_else(|| EffectError::ConfigurationError(
                    "Missing gRPC URL in domain endpoints".to_string()
                ))?
                .to_string();
                
            let lcd_url = domain_info.endpoints.get(1)
                .map(|s| s.to_string());
                
            // Extract network info from metadata
            let chain_id = domain_info.metadata.get("chain_id")
                .and_then(|v| v.as_str())
                .unwrap_or(&domain_id.0)
                .to_string();
                
            let network_type = domain_info.metadata.get("network_type")
                .and_then(|v| v.as_str())
                .unwrap_or("mainnet")
                .to_string();
                
            let prefix = domain_info.metadata.get("prefix")
                .and_then(|v| v.as_str())
                .unwrap_or("cosmos")
                .to_string();
                
            let gas_price = domain_info.metadata.get("gas_price")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.025);
                
            let gas_adjustment = domain_info.metadata.get("gas_adjustment")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.3);
                
            // Create CosmWasm adapter config
            let config = CosmWasmAdapterConfig {
                chain_id,
                grpc_url,
                lcd_url,
                network_type,
                gas_price,
                gas_adjustment,
                prefix,
                extra: Default::default(),
            };
            
            // Create the factory
            let factory = CosmWasmStorageEffectFactory::new(
                contract_address,
                config,
                domain_id.clone(),
            );
            
            // Create the effect (using ContentId for backward compatibility)
            factory.create_store_effect(resource_id, fields, invoker)
                .map_err(|e| EffectError::ExecutionError(e.to_string()))
        },
        _ => {
            // For other domain types, use the generic effect with ContentId
            let effect = StoreOnChainEffect::new(content_id, fields, domain_id, invoker);
            Ok(Arc::new(effect))
        }
    }
}

/// Create a domain-specific storage effect for storing a commitment on-chain
pub fn create_domain_specific_commitment_effect(
    content_id: ContentId,
    commitment: Commitment,
    domain_id: DomainId,
    invoker: Address,
    domain_info: &DomainInfo,
) -> Result<Arc<dyn Effect>, EffectError> {
    // For backward compatibility
    let resource_id = ContentId::from(content_id.clone());
    
    match domain_info.domain_type {
        DomainType::EVM => {
            // Create an Ethereum-specific commitment effect
            // We need to extract the contract address from domain info
            let contract_address_str = domain_info.metadata.get("register_contract")
                .and_then(|v| v.as_str())
                .ok_or_else(|| EffectError::ConfigurationError(
                    "Missing register_contract address in domain metadata".to_string()
                ))?;
                
            let contract_address = ethers::types::Address::from_str(contract_address_str)
                .map_err(|e| EffectError::ConfigurationError(
                    format!("Invalid contract address: {}", e)
                ))?;
                
            let rpc_url = domain_info.endpoints.get(0)
                .ok_or_else(|| EffectError::ConfigurationError(
                    "Missing RPC URL in domain endpoints".to_string()
                ))?;
                
            // Create the factory
            let factory = EthereumStorageEffectFactory::new(
                contract_address,
                rpc_url.clone(),
                domain_id.clone(),
            );
            
            // Create the effect (using ContentId for backward compatibility)
            factory.create_commitment_effect(resource_id, commitment, invoker)
                .map_err(|e| EffectError::ExecutionError(e.to_string()))
        },
        DomainType::CosmWasm => {
            // Create a CosmWasm-specific commitment effect
            // We need to extract contract address and chain info from domain info
            let contract_address = domain_info.metadata.get("register_contract")
                .and_then(|v| v.as_str())
                .ok_or_else(|| EffectError::ConfigurationError(
                    "Missing register_contract address in domain metadata".to_string()
                ))?
                .to_string();
                
            let grpc_url = domain_info.endpoints.get(0)
                .ok_or_else(|| EffectError::ConfigurationError(
                    "Missing gRPC URL in domain endpoints".to_string()
                ))?
                .to_string();
                
            let lcd_url = domain_info.endpoints.get(1)
                .map(|s| s.to_string());
                
            // Extract network info from metadata
            let chain_id = domain_info.metadata.get("chain_id")
                .and_then(|v| v.as_str())
                .unwrap_or(&domain_id.0)
                .to_string();
                
            let network_type = domain_info.metadata.get("network_type")
                .and_then(|v| v.as_str())
                .unwrap_or("mainnet")
                .to_string();
                
            let prefix = domain_info.metadata.get("prefix")
                .and_then(|v| v.as_str())
                .unwrap_or("cosmos")
                .to_string();
                
            let gas_price = domain_info.metadata.get("gas_price")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.025);
                
            let gas_adjustment = domain_info.metadata.get("gas_adjustment")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.3);
                
            // Create CosmWasm adapter config
            let config = CosmWasmAdapterConfig {
                chain_id,
                grpc_url,
                lcd_url,
                network_type,
                gas_price,
                gas_adjustment,
                prefix,
                extra: Default::default(),
            };
            
            // Create the factory
            let factory = CosmWasmStorageEffectFactory::new(
                contract_address,
                config,
                domain_id.clone(),
            );
            
            // Create the effect (using ContentId for backward compatibility)
            factory.create_commitment_effect(resource_id, commitment, invoker)
                .map_err(|e| EffectError::ExecutionError(e.to_string()))
        },
        _ => {
            // For other domain types, use the generic effect with ContentId
            let effect = StoreCommitmentEffect::new(content_id, commitment, domain_id, invoker);
            Ok(Arc::new(effect))
        }
    }
} 
