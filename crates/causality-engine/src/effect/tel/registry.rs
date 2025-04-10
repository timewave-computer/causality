//! TEL Effect Registry
//!
//! This module provides a registry for TEL effects and handlers,
//! integrating with the causality-core effect system.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use causality_core::effect::{
    Effect as CoreEffect,
    EffectRegistry as CoreEffectRegistry,
    EffectHandler as CoreEffectHandler,
    EffectContext as CoreEffectContext,
    EffectOutcome as CoreEffectOutcome,
    EffectResult as CoreEffectResult,
    EffectError as CoreEffectError,
    BasicEffectRegistry,
    HandlerResult,
};

use async_trait::async_trait;
use serde_json::Value;

use causality_tel::types::effect::{TelEffect, EffectError};

/// Registry type to use with TelEffectRegistry
#[derive(Debug)]
pub enum RegistryType {
    /// Basic registry
    Basic(BasicEffectRegistry),
    /// Thread-safe registry
    ThreadSafe(causality_core::effect::ThreadSafeEffectRegistry),
}

impl RegistryType {
    /// Create a new basic registry
    pub fn new_basic() -> Self {
        RegistryType::Basic(BasicEffectRegistry::new())
    }
    
    /// Register a handler
    pub fn register_handler<H>(&mut self, handler: H) -> Result<(), causality_core::effect::EffectError> 
    where 
        H: CoreEffectHandler + 'static
    {
        match self {
            RegistryType::Basic(registry) => {
                registry.register_handler(Arc::new(handler))?;
                Ok(())
            },
            RegistryType::ThreadSafe(registry) => {
                registry.register_handler(Arc::new(handler))?;
                Ok(())
            },
        }
    }
}

/// Handler for TEL effects
#[derive(Clone)]
pub struct TelEffectHandler {
    /// Effect name this handler can process
    pub effect_name: String,
    
    /// Handler implementation function
    pub handler_fn: Arc<dyn Fn(Value, &dyn CoreEffectContext) -> CoreEffectResult<Value> + Send + Sync>,
}

// Implement Debug manually for TelEffectHandler
impl fmt::Debug for TelEffectHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TelEffectHandler")
            .field("effect_name", &self.effect_name)
            .field("handler_fn", &"<function>")
            .finish()
    }
}

impl TelEffectHandler {
    /// Create a new TEL effect handler
    pub fn new(
        effect_name: impl Into<String>,
        handler_fn: impl Fn(Value, &dyn CoreEffectContext) -> CoreEffectResult<Value> + Send + Sync + 'static
    ) -> Self {
        Self {
            effect_name: effect_name.into(),
            handler_fn: Arc::new(handler_fn),
        }
    }
}

#[async_trait]
impl CoreEffectHandler for TelEffectHandler {
    fn supported_effect_types(&self) -> Vec<String> {
        vec![self.effect_name.clone()]
    }
    
    async fn handle(&self, effect: &dyn CoreEffect, context: &dyn CoreEffectContext) -> HandlerResult<CoreEffectOutcome> {
        // Create a simple parameter map - we don't need to serialize the effect
        let mut params = HashMap::new();
        params.insert("effect_name".to_string(), effect.effect_type().to_string());
        
        // Call the handler function
        let result = (self.handler_fn)(serde_json::Value::Null, context)?;
        
        // Convert result to effect outcome
        let mut outcome_data = HashMap::new();
        if let serde_json::Value::Object(map) = result {
            for (key, value) in map {
                outcome_data.insert(key, value.to_string());
            }
        } else {
            outcome_data.insert("result".to_string(), result.to_string());
        }
        
        Ok(CoreEffectOutcome::success(outcome_data))
    }
}

/// Registry for TEL effects and handlers
#[derive(Debug)]
pub struct TelEffectRegistry {
    /// The core effect registry
    core_registry: RegistryType,
    
    /// Effect handlers by name
    handlers: HashMap<String, Arc<TelEffectHandler>>,
}

impl TelEffectRegistry {
    /// Create a new TEL effect registry
    pub fn new() -> Self {
        Self {
            core_registry: RegistryType::new_basic(),
            handlers: HashMap::new(),
        }
    }
    
    /// Create a new TEL effect registry with an existing registry
    pub fn with_registry(registry: RegistryType) -> Self {
        Self {
            core_registry: registry,
            handlers: HashMap::new(),
        }
    }
    
    /// Register a TEL effect handler
    pub fn register_handler(&mut self, handler: TelEffectHandler) -> Result<(), EffectError> {
        let effect_name = handler.effect_name.clone();
        
        // Register with the core registry
        self.core_registry.register_handler(handler.clone())
            .map_err(|e| EffectError::CoreError(format!("Failed to register handler: {}", e)))?;
            
        // Store in our local registry too
        self.handlers.insert(effect_name, Arc::new(handler));
        
        Ok(())
    }
    
    /// Get a handler for an effect
    pub fn get_handler(&self, effect_name: &str) -> Option<Arc<TelEffectHandler>> {
        self.handlers.get(effect_name).cloned()
    }
}

impl Default for TelEffectRegistry {
    fn default() -> Self {
        Self::new()
    }
} 