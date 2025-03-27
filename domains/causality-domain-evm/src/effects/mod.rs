// EVM Domain Effects
//
// This module provides effect implementations specific to the Ethereum domain,
// using the core effect interfaces from causality-core.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use causality_core::effect::{
    Effect, EffectContext, EffectId, EffectOutcome, EffectResult, EffectError,
    DomainEffect, DomainEffectHandler, ResourceEffect, ResourceOperation
};
use causality_core::resource::ContentId;

pub mod transfer;
pub mod storage;

pub use transfer::EvmTransferEffect;
pub use storage::EvmStorageEffect;

/// Ethereum domain ID
pub const EVM_DOMAIN_ID: &str = "evm";

/// EVM effect type identifiers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvmEffectType {
    /// Transfer tokens or assets
    Transfer,
    /// Store data on-chain
    Storage,
    /// Execute a contract call
    ContractCall,
    /// Deploy a contract
    ContractDeploy,
    /// Query blockchain state
    Query,
}

impl EvmEffectType {
    /// Convert to string representation
    pub fn as_str(&self) -> &str {
        match self {
            EvmEffectType::Transfer => "evm.transfer",
            EvmEffectType::Storage => "evm.storage",
            EvmEffectType::ContractCall => "evm.contract_call",
            EvmEffectType::ContractDeploy => "evm.contract_deploy",
            EvmEffectType::Query => "evm.query",
        }
    }
}

/// Base trait for all EVM effects
#[async_trait]
pub trait EvmEffect: DomainEffect {
    /// Get the EVM effect type
    fn evm_effect_type(&self) -> EvmEffectType;
    
    /// Get the chain ID this effect operates on
    fn chain_id(&self) -> u64;
    
    /// Check if this effect is read-only
    fn is_read_only(&self) -> bool {
        matches!(self.evm_effect_type(), EvmEffectType::Query)
    }
    
    /// Get gas parameters for this effect
    fn gas_params(&self) -> Option<EvmGasParams> {
        None
    }
}

/// Gas parameters for Ethereum transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmGasParams {
    /// Gas limit
    pub gas_limit: u64,
    
    /// Gas price (for legacy transactions)
    pub gas_price: Option<u64>,
    
    /// Max fee per gas (for EIP-1559 transactions)
    pub max_fee_per_gas: Option<u64>,
    
    /// Max priority fee per gas (for EIP-1559 transactions)
    pub max_priority_fee_per_gas: Option<u64>,
}

/// Registry for EVM effect handlers
#[derive(Debug)]
pub struct EvmEffectRegistry {
    /// Handlers by effect type
    handlers: HashMap<EvmEffectType, Arc<dyn EvmEffectHandler>>,
}

impl EvmEffectRegistry {
    /// Create a new EVM effect registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
    
    /// Register an EVM effect handler
    pub fn register_handler(&mut self, effect_type: EvmEffectType, handler: Arc<dyn EvmEffectHandler>) {
        self.handlers.insert(effect_type, handler);
    }
    
    /// Get a handler for the given effect type
    pub fn get_handler(&self, effect_type: &EvmEffectType) -> Option<Arc<dyn EvmEffectHandler>> {
        self.handlers.get(effect_type).cloned()
    }
    
    /// Execute an EVM effect
    pub async fn execute(&self, effect: &dyn EvmEffect, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        let effect_type = effect.evm_effect_type();
        let handler = self.get_handler(&effect_type)
            .ok_or_else(|| EffectError::ExecutionError(
                format!("No handler found for EVM effect type: {:?}", effect_type)
            ))?;
        
        handler.handle_evm_effect(effect, context).await
    }
}

/// Trait for EVM effect handlers
#[async_trait]
pub trait EvmEffectHandler: Send + Sync + Debug {
    /// Get the EVM effect type this handler supports
    fn supported_effect_type(&self) -> EvmEffectType;
    
    /// Check if this handler can handle the given effect
    fn can_handle(&self, effect: &dyn EvmEffect) -> bool {
        effect.evm_effect_type() == self.supported_effect_type()
    }
    
    /// Handle the EVM effect
    async fn handle_evm_effect(&self, effect: &dyn EvmEffect, context: &dyn EffectContext) -> EffectResult<EffectOutcome>;
} 