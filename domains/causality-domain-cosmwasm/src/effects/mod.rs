// CosmWasm Domain Effects
//
// This module provides effect implementations specific to the CosmWasm domain,
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

pub mod execute;
pub mod query;
pub mod instantiate;

pub use execute::CosmWasmExecuteEffect;
pub use query::CosmWasmQueryEffect;
pub use instantiate::CosmWasmInstantiateEffect;

/// CosmWasm domain ID
pub const COSMWASM_DOMAIN_ID: &str = "cosmwasm";

/// CosmWasm effect type identifiers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CosmWasmEffectType {
    /// Execute a contract call
    Execute,
    /// Query a contract
    Query,
    /// Instantiate a contract
    Instantiate,
    /// Migrate a contract
    Migrate,
    /// Upload a contract code
    Upload,
}

impl CosmWasmEffectType {
    /// Convert to string representation
    pub fn as_str(&self) -> &str {
        match self {
            CosmWasmEffectType::Execute => "cosmwasm.execute",
            CosmWasmEffectType::Query => "cosmwasm.query",
            CosmWasmEffectType::Instantiate => "cosmwasm.instantiate",
            CosmWasmEffectType::Migrate => "cosmwasm.migrate",
            CosmWasmEffectType::Upload => "cosmwasm.upload",
        }
    }
}

/// Gas parameters for CosmWasm transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmWasmGasParams {
    /// Gas limit
    pub gas_limit: u64,
    
    /// Gas price (in native token)
    pub gas_price: Option<String>,
    
    /// Fee amount (in native token)
    pub fee_amount: Option<String>,
    
    /// Fee denom (token symbol)
    pub fee_denom: Option<String>,
}

/// Base trait for all CosmWasm effects
#[async_trait]
pub trait CosmWasmEffect: DomainEffect {
    /// Get the CosmWasm effect type
    fn cosmwasm_effect_type(&self) -> CosmWasmEffectType;
    
    /// Get the chain ID this effect operates on
    fn chain_id(&self) -> &str;
    
    /// Check if this effect is read-only
    fn is_read_only(&self) -> bool {
        matches!(self.cosmwasm_effect_type(), CosmWasmEffectType::Query)
    }
    
    /// Get gas parameters for this effect
    fn gas_params(&self) -> Option<CosmWasmGasParams> {
        None
    }
}

/// Registry for CosmWasm effect handlers
#[derive(Debug)]
pub struct CosmWasmEffectRegistry {
    /// Handlers by effect type
    handlers: HashMap<CosmWasmEffectType, Arc<dyn CosmWasmEffectHandler>>,
}

impl CosmWasmEffectRegistry {
    /// Create a new CosmWasm effect registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
    
    /// Register a CosmWasm effect handler
    pub fn register_handler(&mut self, effect_type: CosmWasmEffectType, handler: Arc<dyn CosmWasmEffectHandler>) {
        self.handlers.insert(effect_type, handler);
    }
    
    /// Get a handler for the given effect type
    pub fn get_handler(&self, effect_type: &CosmWasmEffectType) -> Option<Arc<dyn CosmWasmEffectHandler>> {
        self.handlers.get(effect_type).cloned()
    }
    
    /// Execute a CosmWasm effect
    pub async fn execute(&self, effect: &dyn CosmWasmEffect, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        let effect_type = effect.cosmwasm_effect_type();
        let handler = self.get_handler(&effect_type)
            .ok_or_else(|| EffectError::ExecutionError(
                format!("No handler found for CosmWasm effect type: {:?}", effect_type)
            ))?;
        
        handler.handle_cosmwasm_effect(effect, context).await
    }
}

/// Trait for CosmWasm effect handlers
#[async_trait]
pub trait CosmWasmEffectHandler: Send + Sync + Debug {
    /// Get the CosmWasm effect type this handler supports
    fn supported_effect_type(&self) -> CosmWasmEffectType;
    
    /// Check if this handler can handle the given effect
    fn can_handle(&self, effect: &dyn CosmWasmEffect) -> bool {
        effect.cosmwasm_effect_type() == self.supported_effect_type()
    }
    
    /// Handle the CosmWasm effect
    async fn handle_cosmwasm_effect(&self, effect: &dyn CosmWasmEffect, context: &dyn EffectContext) -> EffectResult<EffectOutcome>;
} 