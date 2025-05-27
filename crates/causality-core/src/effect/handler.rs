//! Effect handler module
//!
//! This module defines traits and types for handling effects in the system.
//! Effect handlers are responsible for executing effects and returning outcomes.

use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use std::collections::HashMap;

use super::{Effect, EffectContext, EffectOutcome, EffectError};
use super::types::EffectTypeId;

/// Result type for effect handlers
pub type HandlerResult<T> = std::result::Result<T, EffectError>;

/// The EffectHandler trait that effect handlers must implement
#[async_trait]
pub trait EffectHandler: Debug + Send + Sync {
    /// Returns the type of effects this handler can process
    fn supported_effect_types(&self) -> Vec<EffectTypeId>;
    
    /// Handles an effect
    async fn handle(&self, effect: &dyn Effect, context: &dyn EffectContext) -> HandlerResult<EffectOutcome>;
}

/// Registry for effect handlers
#[derive(Debug, Default)]
pub struct EffectHandlerRegistry {
    /// The registered handlers by effect type ID
    handlers: HashMap<EffectTypeId, Vec<Arc<dyn EffectHandler>>>,
}

impl EffectHandlerRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
    
    /// Register a handler for specific effect types
    pub fn register(&mut self, handler: Arc<dyn EffectHandler>) -> &mut Self {
        let effect_types = handler.supported_effect_types();
        
        for effect_type in effect_types {
            self.handlers
                .entry(effect_type)
                .or_insert_with(Vec::new)
                .push(Arc::clone(&handler));
        }
        
        self
    }
    
    /// Find handlers for a specific effect type
    pub fn find_handlers(&self, effect_type_id: &EffectTypeId) -> Vec<Arc<dyn EffectHandler>> {
        self.handlers
            .get(effect_type_id)
            .map(|handlers| handlers.clone())
            .unwrap_or_default()
    }
    
    /// Check if there are handlers for a specific effect type
    pub fn has_handlers_for(&self, effect_type_id: &EffectTypeId) -> bool {
        self.handlers
            .get(effect_type_id)
            .map(|handlers| !handlers.is_empty())
            .unwrap_or(false)
    }
    
    /// Get all registered effect type IDs
    pub fn registered_effect_types(&self) -> Vec<EffectTypeId> {
        self.handlers.keys().cloned().collect()
    }
} 