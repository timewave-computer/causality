//! Effect registry for managing effect handlers
//!
//! This module provides a registry for storing and retrieving effect handlers
//! based on effect type IDs.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

use causality_core::effect::runtime::error::{EffectError, EffectResult};
use causality_core::effect::runtime::core::handler::EffectHandler;
use causality_core::effect::runtime::types::id::EffectTypeId;

/// Registry for effect handlers
///
/// This registry maintains a mapping of effect type IDs to their handlers.
/// It allows for thread-safe registration and retrieval of handlers.
#[derive(Default)]
pub struct EffectRegistry {
    /// The map of effect type IDs to their handlers
    handlers: RwLock<HashMap<EffectTypeId, Arc<dyn EffectHandler>>>,
}

impl EffectRegistry {
    /// Create a new empty effect registry
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a handler for a specific effect type
    ///
    /// If a handler is already registered for this effect type, it will be replaced.
    pub fn register(
        &self,
        effect_type: EffectTypeId,
        handler: Arc<dyn EffectHandler>,
    ) {
        let mut handlers = self.handlers.write().unwrap();
        handlers.insert(effect_type, handler);
    }
    
    /// Get a handler for a specific effect type
    ///
    /// Returns an error if no handler is registered for the given effect type.
    pub fn get_handler(
        &self,
        effect_type: &EffectTypeId,
    ) -> EffectResult<Arc<dyn EffectHandler>> {
        let handlers = self.handlers.read().unwrap();
        handlers.get(effect_type).cloned().ok_or_else(|| EffectError::HandlerNotFound(effect_type.to_string()))
    }
    
    /// Check if a handler is registered for a specific effect type
    pub fn has_handler(&self, effect_type: &EffectTypeId) -> bool {
        let handlers = self.handlers.read().unwrap();
        handlers.contains_key(effect_type)
    }
    
    /// Get all registered effect types
    pub fn registered_effect_types(&self) -> Vec<EffectTypeId> {
        let handlers = self.handlers.read().unwrap();
        handlers.keys().cloned().collect()
    }
    
    /// Remove a handler for a specific effect type
    ///
    /// Returns the removed handler, or None if no handler was registered.
    pub fn unregister(
        &self,
        effect_type: &EffectTypeId,
    ) -> Option<Arc<dyn EffectHandler>> {
        let mut handlers = self.handlers.write().unwrap();
        handlers.remove(effect_type)
    }
}

impl fmt::Debug for EffectRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let handlers = match self.handlers.read() {
            Ok(handlers) => handlers,
            Err(_) => return write!(f, "EffectRegistry {{ <locked> }}"),
        };
        
        f.debug_struct("EffectRegistry")
            .field("handler_count", &handlers.len())
            .field("effect_types", &handlers.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use causality_core::effect::runtime::context::Context;
    use causality_core::effect::runtime::core::handler::EffectHandler;
    use causality_core::effect::runtime::types::id::EffectTypeId;
    use async_trait::async_trait;
    #[derive(Debug)]
    struct TestHandler;
    
    #[async_trait]
    impl EffectHandler for TestHandler {
        async fn can_handle(&self, effect_type: &EffectTypeId) -> bool {
            effect_type.to_string() == "test.effect"
        }
        
        async fn handle(
            &self,
            _effect_type: &EffectTypeId,
            _param: Box<dyn std::any::Any + Send>,
            _context: &dyn causality_core::effect::runtime::context::Context,
        ) -> Result<Box<dyn std::any::Any + Send>, EffectError> {
            Ok(Box::new("test_result".to_string()))
        }
    }
    
    #[tokio::test]
    async fn test_registry_operations() {
        let registry = EffectRegistry::new();
        let effect_type = EffectTypeId::new("test.effect");
        let handler = Arc::new(TestHandler);
        
        // Initially, no handler is registered
        assert!(!registry.has_handler(&effect_type));
        assert_eq!(registry.registered_effect_types().len(), 0);
        
        // Register a handler
        registry.register(effect_type.clone(), handler.clone());
        
        // Now the handler should be registered
        assert!(registry.has_handler(&effect_type));
        assert_eq!(registry.registered_effect_types().len(), 1);
        assert_eq!(registry.registered_effect_types()[0], effect_type);
        
        // Get the handler
        let retrieved_handler = registry.get_handler(&effect_type).unwrap();
        assert!(retrieved_handler.can_handle(&effect_type).await);
        
        // Unregister the handler
        let removed_handler = registry.unregister(&effect_type).unwrap();
        assert!(removed_handler.can_handle(&effect_type).await);
        
        // Now no handler should be registered
        assert!(!registry.has_handler(&effect_type));
        assert_eq!(registry.registered_effect_types().len(), 0);
    }
} 