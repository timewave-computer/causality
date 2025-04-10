//! Mock implementations for testing
//!
//! This module provides mock implementations of core types for testing
//! purposes, allowing for easy creation of unit tests without complex
//! dependencies.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use causality_error::{EngineResult, EngineError};
use crate::execution::{ExecutionContext, Value};

/// Mock implementation of Engine for testing
#[derive(Default)]
pub struct MockEngine {
    /// Internal variables for state tracking
    variables: RwLock<HashMap<String, Value>>,
}

impl MockEngine {
    /// Create a new mock engine
    pub fn new() -> Self {
        Self {
            variables: RwLock::new(HashMap::new()),
        }
    }
    
    /// Execute a value synchronously in the given context
    pub fn execute_sync(&self, value: &Value, _context: &ExecutionContext) -> EngineResult<Value> {
        // For testing purposes, just return the value as is
        Ok(value.clone())
    }
    
    /// Set a variable in the engine
    pub fn set_variable(&self, name: String, value: Value) -> EngineResult<()> {
        let mut variables = self.variables.write().map_err(|_| {
            EngineError::execution_failed("Failed to acquire write lock".to_string())
        })?;
        
        variables.insert(name, value);
        Ok(())
    }
    
    /// Get a variable from the engine
    pub fn get_variable(&self, name: &str) -> EngineResult<Option<Value>> {
        let variables = self.variables.read().map_err(|_| {
            EngineError::execution_failed("Failed to acquire read lock".to_string())
        })?;
        
        Ok(variables.get(name).cloned())
    }
}
