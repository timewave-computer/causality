// EVM Storage Effect
//
// This module provides the implementation of the EVM storage effect,
// which allows storing data on Ethereum-based chains.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use causality_core::effect::{
    Effect, EffectContext, EffectId, EffectOutcome, EffectResult, EffectError,
    DomainEffect, ResourceEffect, ResourceOperation, EffectTypeId
};
use causality_core::resource::ContentId;
use causality_crypto::address::Address;

use super::{EvmEffect, EvmEffectType, EvmGasParams, EVM_DOMAIN_ID};

/// Storage operation type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageOperation {
    /// Write to contract storage
    Write,
    /// Read from contract storage
    Read,
    /// Store data on-chain 
    Store,
    /// Emit an event
    EmitEvent,
}

/// Parameters for EVM storage effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmStorageParams {
    /// Chain ID to operate on
    pub chain_id: u64,
    
    /// Contract address
    pub contract_address: Address,
    
    /// Storage operation
    pub operation: StorageOperation,
    
    /// Storage slot (for read/write)
    pub slot: Option<[u8; 32]>,
    
    /// Data to store
    pub data: Option<Vec<u8>>,
    
    /// Event name (for emitting events)
    pub event_name: Option<String>,
    
    /// Gas parameters
    pub gas_params: Option<EvmGasParams>,
}

/// EVM Storage Effect implementation
pub struct EvmStorageEffect {
    /// Unique identifier
    id: EffectId,
    
    /// Storage parameters
    params: EvmStorageParams,
    
    /// Resource ID representing the contract
    resource_id: ContentId,
}

impl EvmStorageEffect {
    /// Create a new EVM storage effect
    pub fn new(
        params: EvmStorageParams,
        resource_id: ContentId,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            params,
            resource_id,
        }
    }
    
    /// Create a new EVM storage effect with a specific ID
    pub fn with_id(
        id: EffectId,
        params: EvmStorageParams,
        resource_id: ContentId,
    ) -> Self {
        Self {
            id,
            params,
            resource_id,
        }
    }
    
    /// Get the parameters for this storage operation
    pub fn params(&self) -> &EvmStorageParams {
        &self.params
    }
    
    /// Get the contract address
    pub fn contract_address(&self) -> &Address {
        &self.params.contract_address
    }
    
    /// Get the storage operation
    pub fn storage_operation(&self) -> &StorageOperation {
        &self.params.operation
    }
    
    /// Check if this is a read-only operation
    pub fn is_read_only(&self) -> bool {
        matches!(self.params.operation, StorageOperation::Read)
    }
}

impl fmt::Debug for EvmStorageEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EvmStorageEffect")
            .field("id", &self.id)
            .field("chain_id", &self.params.chain_id)
            .field("contract_address", &self.params.contract_address)
            .field("operation", &self.params.operation)
            .finish()
    }
}

#[async_trait]
impl Effect for EvmStorageEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn type_id(&self) -> EffectTypeId {
        EffectTypeId::new("evm.storage")
    }
    
    fn display_name(&self) -> String {
        match self.params.operation {
            StorageOperation::Write => "EVM Storage Write".to_string(),
            StorageOperation::Read => "EVM Storage Read".to_string(),
            StorageOperation::Store => "EVM Data Storage".to_string(),
            StorageOperation::EmitEvent => "EVM Event Emission".to_string(),
        }
    }
    
    fn description(&self) -> String {
        match self.params.operation {
            StorageOperation::Write => {
                format!(
                    "Write to storage at contract {} on chain {}",
                    self.params.contract_address,
                    self.params.chain_id
                )
            },
            StorageOperation::Read => {
                format!(
                    "Read from storage at contract {} on chain {}",
                    self.params.contract_address,
                    self.params.chain_id
                )
            },
            StorageOperation::Store => {
                format!(
                    "Store data at contract {} on chain {}",
                    self.params.contract_address,
                    self.params.chain_id
                )
            },
            StorageOperation::EmitEvent => {
                format!(
                    "Emit event {} from contract {} on chain {}",
                    self.params.event_name.as_deref().unwrap_or("unknown"),
                    self.params.contract_address,
                    self.params.chain_id
                )
            },
        }
    }
    
    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("chain_id".to_string(), self.params.chain_id.to_string());
        params.insert("contract_address".to_string(), self.params.contract_address.to_string());
        params.insert("operation".to_string(), format!("{:?}", self.params.operation));
        
        if let Some(slot) = &self.params.slot {
            params.insert("slot".to_string(), hex::encode(slot));
        }
        
        if let Some(event_name) = &self.params.event_name {
            params.insert("event_name".to_string(), event_name.clone());
        }
        
        if let Some(data) = &self.params.data {
            if data.len() <= 64 {
                params.insert("data".to_string(), hex::encode(data));
            } else {
                params.insert("data_size".to_string(), data.len().to_string());
            }
        }
        
        params
    }
    
    async fn execute(&self, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        // In a real implementation, this would call out to the EVM chain
        // For now, we'll just return a success outcome
        
        // Check capabilities
        let required_right = match self.params.operation {
            StorageOperation::Write | StorageOperation::Store | StorageOperation::EmitEvent => {
                causality_core::capability::Right::Write
            },
            StorageOperation::Read => {
                causality_core::capability::Right::Read
            },
        };
        
        if !context.has_capability(&self.resource_id, &required_right) {
            return Err(EffectError::CapabilityError(
                format!("Missing {:?} capability for resource: {}", required_right, self.resource_id)
            ));
        }
        
        // Create outcome data
        let mut outcome_data = HashMap::new();
        outcome_data.insert("chain_id".to_string(), self.params.chain_id.to_string());
        outcome_data.insert("contract_address".to_string(), self.params.contract_address.to_string());
        outcome_data.insert("operation".to_string(), format!("{:?}", self.params.operation));
        
        match self.params.operation {
            StorageOperation::Read => {
                if let Some(slot) = &self.params.slot {
                    outcome_data.insert("slot".to_string(), hex::encode(slot));
                    // In a real implementation, we would read the storage value
                    // For now, just provide a dummy value
                    outcome_data.insert("value".to_string(), "0x0000000000000000000000000000000000000000000000000000000000000000".to_string());
                }
            },
            StorageOperation::Write => {
                if let Some(slot) = &self.params.slot {
                    outcome_data.insert("slot".to_string(), hex::encode(slot));
                }
                if let Some(data) = &self.params.data {
                    outcome_data.insert("data_hash".to_string(), hex::encode(causality_crypto::Hash::digest(data).as_bytes()));
                }
            },
            StorageOperation::Store => {
                if let Some(data) = &self.params.data {
                    outcome_data.insert("data_size".to_string(), data.len().to_string());
                    outcome_data.insert("data_hash".to_string(), hex::encode(causality_crypto::Hash::digest(data).as_bytes()));
                }
            },
            StorageOperation::EmitEvent => {
                if let Some(event_name) = &self.params.event_name {
                    outcome_data.insert("event_name".to_string(), event_name.clone());
                }
                if let Some(data) = &self.params.data {
                    outcome_data.insert("data_size".to_string(), data.len().to_string());
                    outcome_data.insert("data_hash".to_string(), hex::encode(causality_crypto::Hash::digest(data).as_bytes()));
                }
            },
        }
        
        // Return success outcome
        Ok(EffectOutcome::success(outcome_data)
            .with_affected_resource(self.resource_id.clone()))
    }
}

#[async_trait]
impl DomainEffect for EvmStorageEffect {
    fn domain_id(&self) -> &str {
        EVM_DOMAIN_ID
    }
    
    fn domain_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("chain_id".to_string(), self.params.chain_id.to_string());
        params
    }
}

#[async_trait]
impl ResourceEffect for EvmStorageEffect {
    fn resource_id(&self) -> &ContentId {
        &self.resource_id
    }
    
    fn operation(&self) -> ResourceOperation {
        match self.params.operation {
            StorageOperation::Read => ResourceOperation::Read,
            StorageOperation::Write => ResourceOperation::Update,
            StorageOperation::Store => ResourceOperation::Update,
            StorageOperation::EmitEvent => ResourceOperation::Update,
        }
    }
}

#[async_trait]
impl EvmEffect for EvmStorageEffect {
    fn evm_effect_type(&self) -> EvmEffectType {
        EvmEffectType::Storage
    }
    
    fn chain_id(&self) -> u64 {
        self.params.chain_id
    }
    
    fn is_read_only(&self) -> bool {
        matches!(self.params.operation, StorageOperation::Read)
    }
    
    fn gas_params(&self) -> Option<EvmGasParams> {
        self.params.gas_params.clone()
    }
} 