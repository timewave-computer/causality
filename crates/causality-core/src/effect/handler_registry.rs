//! Effect handler registry for managing and executing effects
//!
//! This module provides a registry system for effect handlers that can be
//! dynamically registered and executed based on effect tags.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::lambda::{base::Value};
use crate::system::error::{Error, Result};

/// Result type for effect execution
pub type EffectResult = Result<Value>;

/// Error type for effect execution failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectExecutionError {
    /// Handler not found for the given effect tag
    HandlerNotFound(String),
    /// Handler execution failed
    ExecutionFailed(String),
    /// Invalid effect parameters
    InvalidParameters(String),
    /// Resource conflict
    ResourceConflict(String),
}

impl std::fmt::Display for EffectExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectExecutionError::HandlerNotFound(tag) => write!(f, "Handler not found for effect: {}", tag),
            EffectExecutionError::ExecutionFailed(msg) => write!(f, "Effect execution failed: {}", msg),
            EffectExecutionError::InvalidParameters(msg) => write!(f, "Invalid effect parameters: {}", msg),
            EffectExecutionError::ResourceConflict(msg) => write!(f, "Resource conflict: {}", msg),
        }
    }
}

impl std::error::Error for EffectExecutionError {}

/// Trait for effect handlers that can execute specific effects
pub trait EffectHandler: Send + Sync {
    /// Execute an effect with the given parameters
    fn execute(&self, params: Vec<Value>) -> EffectResult;
    
    /// Check if this handler can execute with the given capabilities
    fn can_execute_with_capabilities(&self, _capabilities: &[String]) -> bool {
        true // Default implementation allows all capabilities
    }
    
    /// Get the effect tag this handler supports
    fn effect_tag(&self) -> &str;
    
    /// Validate effect parameters before execution
    fn validate_params(&self, params: &[Value]) -> Result<()> {
        let _ = params; // Suppress unused parameter warning
        Ok(()) // Default implementation accepts all parameters
    }
}

/// Registry for managing effect handlers
pub struct EffectHandlerRegistry {
    handlers: RwLock<HashMap<String, Arc<dyn EffectHandler>>>,
    default_handler: Option<Arc<dyn EffectHandler>>,
}

impl std::fmt::Debug for EffectHandlerRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EffectHandlerRegistry")
            .field("handlers", &"<handlers map>")
            .field("default_handler", &self.default_handler.is_some())
            .finish()
    }
}

impl EffectHandlerRegistry {
    /// Create a new effect handler registry
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
            default_handler: None,
        }
    }
    
    /// Register an effect handler
    pub fn register_handler(&self, handler: Arc<dyn EffectHandler>) -> Result<()> {
        let tag = handler.effect_tag().to_string();
        let mut handlers = self.handlers.write()
            .map_err(|_| Error::serialization("Failed to acquire write lock"))?;
        handlers.insert(tag, handler);
        Ok(())
    }
    
    /// Get a handler for the given effect tag
    pub fn get_handler(&self, effect_tag: &str) -> Option<Arc<dyn EffectHandler>> {
        let handlers = self.handlers.read().ok()?;
        handlers.get(effect_tag).cloned()
    }
    
    /// Execute an effect by tag with parameters
    pub fn execute_effect(&self, effect_tag: &str, params: Vec<Value>) -> EffectResult {
        let handler = self.get_handler(effect_tag)
            .ok_or_else(|| Error::serialization(
                format!("No handler found for effect: {}", effect_tag)))?;
        
        handler.validate_params(&params)?;
        handler.execute(params)
    }
    
    /// List all registered effect tags
    pub fn list_effects(&self) -> Vec<String> {
        if let Ok(handlers) = self.handlers.read() {
            handlers.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
    
    /// Check if an effect is registered
    pub fn has_effect(&self, effect_tag: &str) -> bool {
        if let Ok(handlers) = self.handlers.read() {
            handlers.contains_key(effect_tag)
        } else {
            false
        }
    }
    
    /// Clear all handlers
    pub fn clear(&self) -> Result<()> {
        let mut handlers = self.handlers.write()
            .map_err(|_| Error::serialization("Failed to acquire write lock"))?;
        handlers.clear();
        Ok(())
    }
    
    /// Clone the registry (creates a new registry with the same handlers)
    pub fn clone_registry(&self) -> Result<Self> {
        let new_registry = Self::new();
        let handlers = self.handlers.read()
            .map_err(|_| Error::serialization("Failed to acquire read lock"))?;
        
        for (tag, handler) in handlers.iter() {
            let mut new_handlers = new_registry.handlers.write()
                .map_err(|_| Error::serialization("Failed to acquire write lock"))?;
            new_handlers.insert(tag.clone(), handler.clone());
        }
        
        Ok(new_registry)
    }
}

impl Default for EffectHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple effect handler for basic operations
pub struct SimpleEffectHandler {
    tag: String,
    handler_fn: Box<dyn Fn(Vec<Value>) -> EffectResult + Send + Sync>,
}

impl SimpleEffectHandler {
    /// Create a new simple effect handler
    pub fn new<F>(tag: String, handler_fn: F) -> Self 
    where 
        F: Fn(Vec<Value>) -> EffectResult + Send + Sync + 'static,
    {
        Self {
            tag,
            handler_fn: Box::new(handler_fn),
        }
    }
}

impl EffectHandler for SimpleEffectHandler {
    fn execute(&self, params: Vec<Value>) -> EffectResult {
        (self.handler_fn)(params)
    }
    
    fn effect_tag(&self) -> &str {
        &self.tag
    }
}

/// Utility function to handle string operations
fn _handle_string_operation(operation: &str, args: Vec<Value>) -> EffectResult {
    match operation {
        "concat" => {
            if args.len() != 2 {
                return Err(Error::serialization("concat requires exactly 2 arguments"));
            }
            // For now, work with Symbol values since Str is not available
            match (&args[0], &args[1]) {
                (Value::Symbol(a), Value::Symbol(b)) => {
                    let result = format!("{}{}", a.as_str(), b.as_str());
                    Ok(Value::Symbol(result.into()))
                },
                _ => Err(Error::serialization("concat requires symbol arguments")),
            }
        },
        _ => Err(Error::serialization(format!("String operation '{}' not implemented", operation))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    #[test]
    fn test_registry_creation() {
        let registry = EffectHandlerRegistry::new();
        assert!(registry.list_effects().is_empty());
    }
    
    #[test]
    fn test_handler_registration() {
        let registry = EffectHandlerRegistry::new();
        
        // Create a simple log handler
        let log_handler = Arc::new(SimpleEffectHandler::new(
            "log".to_string(),
            |_params| Ok(Value::Unit),
        ));
        
        registry.register_handler(log_handler).unwrap();
        
        assert!(registry.has_effect("log"));
        assert_eq!(registry.list_effects(), vec!["log"]);
    }
    
    #[test]
    fn test_effect_execution() {
        let registry = EffectHandlerRegistry::new();
        
        // Create a string concatenation handler
        let concat_handler = Arc::new(SimpleEffectHandler::new(
            "concat".to_string(),
            |params| _handle_string_operation("concat", params),
        ));
        
        registry.register_handler(concat_handler).unwrap();
        
        let result = registry.execute_effect("concat", vec![
            Value::Symbol("hello".into()), 
            Value::Symbol(" world".into())
        ]);
        
        assert!(result.is_ok());
        if let Ok(Value::Symbol(s)) = result {
            assert_eq!(s.as_str(), "hello world");
        } else {
            panic!("Expected symbol result");
        }
    }
    
    #[test]
    fn test_missing_handler() {
        let registry = EffectHandlerRegistry::new();
        
        let result = registry.execute_effect("nonexistent", vec![
            Value::Symbol("alice".into()),
            Value::Symbol("bob".into()),
        ]);
        
        assert!(result.is_err());
    }
} 