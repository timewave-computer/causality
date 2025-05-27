// EVM Transfer Effect
//
// This module provides the implementation of the EVM transfer effect,
// which allows transferring Ether and ERC20 tokens on Ethereum-based chains.

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

/// Type of token to transfer
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    /// Native Ether
    Native,
    /// ERC20 token
    ERC20(Address),
    /// ERC721 NFT
    ERC721(Address, u64),
}

/// Parameters for EVM transfer effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmTransferParams {
    /// Chain ID to operate on
    pub chain_id: u64,
    
    /// From address
    pub from: Address,
    
    /// To address
    pub to: Address,
    
    /// Token type
    pub token_type: TokenType,
    
    /// Amount to transfer (for fungible tokens)
    pub amount: Option<u128>,
    
    /// Gas parameters
    pub gas_params: Option<EvmGasParams>,
    
    /// Additional data
    pub data: Option<Vec<u8>>,
}

/// EVM Transfer Effect implementation
pub struct EvmTransferEffect {
    /// Unique identifier
    id: EffectId,
    
    /// Transfer parameters
    params: EvmTransferParams,
    
    /// Resource ID representing the source token
    source_resource_id: ContentId,
    
    /// Resource ID representing the destination token
    destination_resource_id: ContentId,
}

impl EvmTransferEffect {
    /// Create a new EVM transfer effect
    pub fn new(
        params: EvmTransferParams,
        source_resource_id: ContentId,
        destination_resource_id: ContentId,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            params,
            source_resource_id,
            destination_resource_id,
        }
    }
    
    /// Create a new EVM transfer effect with a specific ID
    pub fn with_id(
        id: EffectId,
        params: EvmTransferParams,
        source_resource_id: ContentId,
        destination_resource_id: ContentId,
    ) -> Self {
        Self {
            id,
            params,
            source_resource_id,
            destination_resource_id,
        }
    }
    
    /// Get the parameters for this transfer
    pub fn params(&self) -> &EvmTransferParams {
        &self.params
    }
    
    /// Get the source address
    pub fn from(&self) -> &Address {
        &self.params.from
    }
    
    /// Get the destination address
    pub fn to(&self) -> &Address {
        &self.params.to
    }
    
    /// Get the token type
    pub fn token_type(&self) -> &TokenType {
        &self.params.token_type
    }
    
    /// Get the transfer amount (for fungible tokens)
    pub fn amount(&self) -> Option<u128> {
        self.params.amount
    }
}

impl fmt::Debug for EvmTransferEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EvmTransferEffect")
            .field("id", &self.id)
            .field("chain_id", &self.params.chain_id)
            .field("from", &self.params.from)
            .field("to", &self.params.to)
            .field("token_type", &self.params.token_type)
            .field("amount", &self.params.amount)
            .finish()
    }
}

#[async_trait]
impl Effect for EvmTransferEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn type_id(&self) -> EffectTypeId {
        EffectTypeId::new("evm.transfer")
    }
    
    fn display_name(&self) -> String {
        "EVM Token Transfer".to_string()
    }
    
    fn description(&self) -> String {
        match &self.params.token_type {
            TokenType::Native => {
                format!(
                    "Transfer {} Ether from {} to {} on chain {}",
                    self.params.amount.unwrap_or(0),
                    self.params.from,
                    self.params.to,
                    self.params.chain_id
                )
            },
            TokenType::ERC20(contract) => {
                format!(
                    "Transfer {} ERC20 tokens ({}) from {} to {} on chain {}",
                    self.params.amount.unwrap_or(0),
                    contract,
                    self.params.from,
                    self.params.to,
                    self.params.chain_id
                )
            },
            TokenType::ERC721(contract, token_id) => {
                format!(
                    "Transfer ERC721 token {} ({}) from {} to {} on chain {}",
                    token_id,
                    contract,
                    self.params.from,
                    self.params.to,
                    self.params.chain_id
                )
            }
        }
    }
    
    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("chain_id".to_string(), self.params.chain_id.to_string());
        params.insert("from".to_string(), self.params.from.to_string());
        params.insert("to".to_string(), self.params.to.to_string());
        
        match &self.params.token_type {
            TokenType::Native => {
                params.insert("token_type".to_string(), "native".to_string());
                if let Some(amount) = self.params.amount {
                    params.insert("amount".to_string(), amount.to_string());
                }
            },
            TokenType::ERC20(contract) => {
                params.insert("token_type".to_string(), "erc20".to_string());
                params.insert("contract".to_string(), contract.to_string());
                if let Some(amount) = self.params.amount {
                    params.insert("amount".to_string(), amount.to_string());
                }
            },
            TokenType::ERC721(contract, token_id) => {
                params.insert("token_type".to_string(), "erc721".to_string());
                params.insert("contract".to_string(), contract.to_string());
                params.insert("token_id".to_string(), token_id.to_string());
            }
        }
        
        params
    }
    
    async fn execute(&self, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        // In a real implementation, this would call out to the EVM chain
        // For now, we'll just return a success outcome
        
        // Check capabilities
        if !context.has_capability(&self.source_resource_id, &causality_core::capability::Right::Transfer) {
            return Err(EffectError::CapabilityError(
                format!("Missing transfer capability for source resource: {}", self.source_resource_id)
            ));
        }
        
        if !context.has_capability(&self.destination_resource_id, &causality_core::capability::Right::Write) {
            return Err(EffectError::CapabilityError(
                format!("Missing write capability for destination resource: {}", self.destination_resource_id)
            ));
        }
        
        // Create outcome data
        let mut outcome_data = HashMap::new();
        outcome_data.insert("chain_id".to_string(), self.params.chain_id.to_string());
        outcome_data.insert("from".to_string(), self.params.from.to_string());
        outcome_data.insert("to".to_string(), self.params.to.to_string());
        
        match &self.params.token_type {
            TokenType::Native => {
                outcome_data.insert("token_type".to_string(), "native".to_string());
                if let Some(amount) = self.params.amount {
                    outcome_data.insert("amount".to_string(), amount.to_string());
                }
            },
            TokenType::ERC20(contract) => {
                outcome_data.insert("token_type".to_string(), "erc20".to_string());
                outcome_data.insert("contract".to_string(), contract.to_string());
                if let Some(amount) = self.params.amount {
                    outcome_data.insert("amount".to_string(), amount.to_string());
                }
            },
            TokenType::ERC721(contract, token_id) => {
                outcome_data.insert("token_type".to_string(), "erc721".to_string());
                outcome_data.insert("contract".to_string(), contract.to_string());
                outcome_data.insert("token_id".to_string(), token_id.to_string());
            }
        }
        
        // Return success outcome
        Ok(EffectOutcome::success(outcome_data)
            .with_affected_resource(self.source_resource_id.clone())
            .with_affected_resource(self.destination_resource_id.clone()))
    }
}

#[async_trait]
impl DomainEffect for EvmTransferEffect {
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
impl ResourceEffect for EvmTransferEffect {
    fn resource_id(&self) -> &ContentId {
        &self.source_resource_id
    }
    
    fn operation(&self) -> ResourceOperation {
        ResourceOperation::Transfer
    }
}

#[async_trait]
impl EvmEffect for EvmTransferEffect {
    fn evm_effect_type(&self) -> EvmEffectType {
        EvmEffectType::Transfer
    }
    
    fn chain_id(&self) -> u64 {
        self.params.chain_id
    }
    
    fn gas_params(&self) -> Option<EvmGasParams> {
        self.params.gas_params.clone()
    }
} 