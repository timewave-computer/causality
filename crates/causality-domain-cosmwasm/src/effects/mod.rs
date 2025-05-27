// CosmWasm Effect Types
//
// This module provides effect implementations for CosmWasm operations.

mod execute;
mod query;

// Re-export effect types
pub use execute::{CosmWasmExecuteEffect, CosmWasmExecuteParams, Coin};
pub use query::{CosmWasmQueryEffect, CosmWasmQueryParams};

use causality_core::effect::{EffectContext, EffectResult};

/// Domain ID for CosmWasm effects
pub const COSMWASM_DOMAIN_ID: &str = "cosmwasm";

/// CosmWasm effect types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CosmWasmEffectType {
    /// Execute a contract
    Execute,
    /// Query a contract
    Query,
}

/// Gas parameters for CosmWasm operations
#[derive(Debug, Clone)]
pub struct CosmWasmGasParams {
    /// Gas limit
    pub gas_limit: Option<u64>,
    /// Gas price (in smallest denomination)
    pub gas_price: Option<f64>,
    /// Fee amount (in smallest denomination)
    pub fee_amount: Option<u64>,
    /// Fee denomination
    pub fee_denom: Option<String>,
}

/// Trait for CosmWasm effects
#[async_trait::async_trait]
pub trait CosmWasmEffect: Send + Sync {
    /// Get the CosmWasm effect type
    fn cosmwasm_effect_type(&self) -> CosmWasmEffectType;
    
    /// Get the chain ID
    fn chain_id(&self) -> &str;
    
    /// Get gas parameters (if any)
    fn gas_params(&self) -> Option<CosmWasmGasParams>;
    
    /// Handle the effect with the given handler
    async fn handle_with_handler(&self, handler: &dyn CosmWasmEffectHandler, context: &dyn EffectContext) -> EffectResult;
}

/// Handler for CosmWasm effects
#[async_trait::async_trait]
pub trait CosmWasmEffectHandler: Send + Sync {
    /// Handle a CosmWasm effect
    async fn handle_cosmwasm_effect(&self, effect: &dyn CosmWasmEffect, context: &dyn EffectContext) -> EffectResult;
} 