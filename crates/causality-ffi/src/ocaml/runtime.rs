//! OCaml runtime integration for Causality FFI
//!
//! This module provides the OCaml runtime interface for the Causality system,
//! including initialization, expression management, and cleanup.

use std::collections::HashMap;
use std::sync::{Mutex, Arc};

use ocaml::{FromValue, ToValue, Value};
use crate::ocaml::{core_types::*, error_handling::*};

use causality_lisp::ast::{Expr};

/// Global runtime state for FFI operations
static RUNTIME_STATE: Mutex<Option<Arc<RuntimeState>>> = Mutex::new(None);

/// Runtime state container
pub struct RuntimeState {
    /// Storage for expressions with unique IDs
    pub expressions: HashMap<u64, Expr>,
    /// Next available expression ID
    next_expr_id: u64,
    /// Runtime configuration
    config: RuntimeConfig,
}

/// Configuration for the FFI runtime
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Enable debug logging
    pub debug: bool,
    /// Maximum number of stored expressions
    pub max_expressions: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            debug: false,
            max_expressions: 10000,
        }
    }
}

impl RuntimeState {
    pub fn new() -> Self {
        Self {
            expressions: HashMap::new(),
            next_expr_id: 1,
            config: RuntimeConfig::default(),
        }
    }
    
    pub fn register_expression(&mut self, expr: Expr) -> u64 {
        let id = self.next_expr_id;
        self.next_expr_id += 1;
        self.expressions.insert(id, expr);
        id
    }
    
    pub fn get_expression(&self, id: u64) -> Option<&Expr> {
        self.expressions.get(&id)
    }
}

/// Initialize the Causality runtime
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn causality_init() -> bool {
    match RUNTIME_STATE.lock() {
        Ok(mut state) => {
            if state.is_none() {
                *state = Some(Arc::new(RuntimeState::new()));
                true
            } else {
                false // Already initialized
            }
        }
        Err(_) => false,
    }
}

/// Get runtime version information
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn causality_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Cleanup runtime resources
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn causality_cleanup() -> bool {
    match RUNTIME_STATE.lock() {
        Ok(mut state) => {
            *state = None;
            true
        }
        Err(_) => false,
    }
}

/// Execute operation with runtime state
pub fn with_runtime_state<T, F>(f: F) -> Result<T, String>
where
    F: FnOnce(&mut RuntimeState) -> T,
{
    let state_guard = RUNTIME_STATE.lock()
        .map_err(|_| "Failed to acquire runtime lock".to_string())?;
    
    match state_guard.as_ref() {
        Some(state_arc) => {
            // For this simplified version, we'll return an error since we can't easily get mutable access
            Err("Runtime state access not implemented in simplified version".to_string())
        }
        None => Err("Runtime not initialized".to_string()),
    }
}

#[cfg(feature = "ocaml-ffi")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_initialization() {
        // Test basic runtime operations (without OCaml runtime dependency)
        let mut state = RuntimeState::new();
        
        // Test that we can create the runtime state
        assert_eq!(state.next_expr_id, 1);
        assert!(state.expressions.is_empty());
    }
} 