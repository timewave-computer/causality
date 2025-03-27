// Succinct Domain Effects
//
// This module provides effect implementations specific to the Succinct domain,
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

pub mod prove;
pub mod verify;

pub use prove::SuccinctProveEffect;
pub use verify::SuccinctVerifyEffect;

/// Succinct domain ID
pub const SUCCINCT_DOMAIN_ID: &str = "succinct";

/// Succinct effect type identifiers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuccinctEffectType {
    /// Generate a proof
    Prove,
    /// Verify a proof
    Verify,
    /// Submit a proof on-chain
    Submit,
    /// Query a proof
    Query,
}

impl SuccinctEffectType {
    /// Convert to string representation
    pub fn as_str(&self) -> &str {
        match self {
            SuccinctEffectType::Prove => "succinct.prove",
            SuccinctEffectType::Verify => "succinct.verify",
            SuccinctEffectType::Submit => "succinct.submit",
            SuccinctEffectType::Query => "succinct.query",
        }
    }
}

/// Base trait for all Succinct effects
#[async_trait]
pub trait SuccinctEffect: DomainEffect {
    /// Get the Succinct effect type
    fn succinct_effect_type(&self) -> SuccinctEffectType;
    
    /// Get the circuit ID this effect operates on
    fn circuit_id(&self) -> &str;
    
    /// Check if this effect is read-only
    fn is_read_only(&self) -> bool {
        matches!(self.succinct_effect_type(), SuccinctEffectType::Verify | SuccinctEffectType::Query)
    }
}

/// Registry for Succinct effect handlers
#[derive(Debug)]
pub struct SuccinctEffectRegistry {
    /// Handlers by effect type
    handlers: HashMap<SuccinctEffectType, Arc<dyn SuccinctEffectHandler>>,
}

impl SuccinctEffectRegistry {
    /// Create a new Succinct effect registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
    
    /// Register a Succinct effect handler
    pub fn register_handler(&mut self, effect_type: SuccinctEffectType, handler: Arc<dyn SuccinctEffectHandler>) {
        self.handlers.insert(effect_type, handler);
    }
    
    /// Get a handler for the given effect type
    pub fn get_handler(&self, effect_type: &SuccinctEffectType) -> Option<Arc<dyn SuccinctEffectHandler>> {
        self.handlers.get(effect_type).cloned()
    }
    
    /// Execute a Succinct effect
    pub async fn execute(&self, effect: &dyn SuccinctEffect, context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        let effect_type = effect.succinct_effect_type();
        let handler = self.get_handler(&effect_type)
            .ok_or_else(|| EffectError::ExecutionError(
                format!("No handler found for Succinct effect type: {:?}", effect_type)
            ))?;
        
        handler.handle_succinct_effect(effect, context).await
    }
}

/// Trait for Succinct effect handlers
#[async_trait]
pub trait SuccinctEffectHandler: Send + Sync + Debug {
    /// Get the Succinct effect type this handler supports
    fn supported_effect_type(&self) -> SuccinctEffectType;
    
    /// Check if this handler can handle the given effect
    fn can_handle(&self, effect: &dyn SuccinctEffect) -> bool {
        effect.succinct_effect_type() == self.supported_effect_type()
    }
    
    /// Handle the Succinct effect
    async fn handle_succinct_effect(&self, effect: &dyn SuccinctEffect, context: &dyn EffectContext) -> EffectResult<EffectOutcome>;
} 