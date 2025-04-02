//! Effect handler module
//!
//! This module defines traits and types for handling effects in the system.
//! Effect handlers are responsible for executing effects and returning outcomes.

use std::fmt::Debug;
use std::sync::Arc;
use async_trait::async_trait;

use super::{Effect, EffectContext, EffectOutcome};
use super::types::EffectTypeId;

/// Result type for handler operations
pub type HandlerResult<T> = Result<T, super::EffectError>;

/// Trait for handling effects
#[async_trait]
pub trait EffectHandler: Send + Sync + Debug {
    /// Get the effect types this handler can handle
    fn supported_effect_types(&self) -> Vec<EffectTypeId>;
    
    /// Handle an effect and return an outcome
    async fn handle(&self, effect: &dyn Effect, context: &dyn EffectContext) -> HandlerResult<EffectOutcome>;
}

/// A registry of effect handlers
#[derive(Debug, Default)]
pub struct EffectHandlerRegistry {
    /// Handlers by effect type
    handlers: Vec<Arc<dyn EffectHandler>>,
}

impl EffectHandlerRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
    
    /// Register a handler
    pub fn register(&mut self, handler: Arc<dyn EffectHandler>) {
        self.handlers.push(handler);
    }
    
    /// Find a handler for an effect
    pub fn find_handler(&self, effect_type: &EffectTypeId) -> Option<Arc<dyn EffectHandler>> {
        self.handlers.iter()
            .find(|h| h.supported_effect_types().contains(effect_type))
            .cloned()
    }
    
    /// Get all handlers
    pub fn handlers(&self) -> &[Arc<dyn EffectHandler>] {
        &self.handlers
    }
} 