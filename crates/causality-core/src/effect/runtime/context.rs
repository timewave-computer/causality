//! Context for effect execution
//!
//! This module defines the execution context for effects.

use std::collections::HashMap;
use std::fmt::Debug;
use std::any::Any;

/// Execution context for effects
///
/// This provides access to resources, capabilities, and metadata
/// during effect execution.
pub trait Context: ContextValue + Debug + Send + Sync {
    /// Get all metadata associated with this context
    fn metadata(&self) -> &HashMap<String, String>;
    
    /// Check if a capability is available
    fn has_capability(&self, capability: &str) -> bool;
    
    /// Check if a resource is available
    fn has_resource(&self, resource_id: &str) -> bool;
    
    /// Create a derived context with additional metadata
    fn with_metadata(&self, metadata: HashMap<String, String>) -> Box<dyn Context>;
}

/// Extension trait for getting typed values from a context
pub trait ContextValue {
    /// Get a value from the context, if it exists
    fn get_value(&self, key: &str) -> Option<&dyn Any>;
}

/// Extension methods for Context
pub trait ContextExt: Context {
    /// Get a typed value from the context
    fn get<T: 'static>(&self, key: &str) -> Option<&T> {
        self.get_value(key).and_then(|v| v.downcast_ref::<T>())
    }
} 