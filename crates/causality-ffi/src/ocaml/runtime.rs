//! OCaml runtime and global state management

#[cfg(feature = "ocaml-ffi")]
use std::collections::HashMap;
#[cfg(feature = "ocaml-ffi")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "ocaml-ffi")]
use once_cell::sync::Lazy;

#[cfg(feature = "ocaml-ffi")]
use ocaml::{FromValue, ToValue, Value, Runtime};

#[cfg(feature = "ocaml-ffi")]
use causality_core::lambda::base::Value as CoreLispValue;
#[cfg(feature = "ocaml-ffi")]
use causality_lisp::ast::{LispValue as AstLispValue, Expr, ExprKind};

#[cfg(feature = "ocaml-ffi")]
use crate::ocaml::core_types::ResourceId;

/// Global state for managing Rust objects from OCaml
#[cfg(feature = "ocaml-ffi")]
static RUNTIME_STATE: Lazy<Arc<Mutex<RuntimeState>>> = Lazy::new(|| {
    Arc::new(Mutex::new(RuntimeState::new()))
});

/// Runtime state for managing objects across FFI boundary
#[cfg(feature = "ocaml-ffi")]
#[derive(Debug)]
pub struct RuntimeState {
    /// Registered expressions by ID
    pub expressions: HashMap<u64, Expr>,
    
    /// Resource registry
    pub resources: HashMap<ResourceId, CoreLispValue>,
    
    /// Next expression ID
    pub next_expr_id: u64,
}

#[cfg(feature = "ocaml-ffi")]
impl RuntimeState {
    pub fn new() -> Self {
        Self {
            expressions: HashMap::new(),
            resources: HashMap::new(),
            next_expr_id: 1,
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

/// Initialize the Causality runtime for OCaml
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn causality_init() -> bool {
    // Initialize logging if needed
    env_logger::try_init().unwrap_or_default();
    
    // Initialize runtime state
    let _state = RUNTIME_STATE.lock().unwrap();
    
    log::info!("Causality OCaml FFI initialized");
    true
}

/// Get version information
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn causality_version() -> String {
    format!("Causality OCaml FFI v{}", env!("CARGO_PKG_VERSION"))
}

/// Cleanup resources (should be called before program exit)
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn causality_cleanup() -> bool {
    log::info!("Causality OCaml FFI cleanup");
    true
}

/// Convert Rust Result to OCaml result type
#[cfg(feature = "ocaml-ffi")]
pub fn result_to_ocaml<T, E>(result: Result<T, E>) -> Value
where
    T: ToValue,
    E: std::fmt::Display,
{
    match result {
        Ok(val) => {
            // OCaml: Ok val  
            unsafe {
                let ok_tag = Value::int(0);
                let val_ocaml = val.to_value();
                let tuple = Value::alloc_tuple(2);
                tuple.set_field(0, ok_tag);
                tuple.set_field(1, val_ocaml);
                tuple
            }
        }
        Err(err) => {
            // OCaml: Error msg
            unsafe {
                let error_tag = Value::int(1);
                let err_msg = Value::string(&err.to_string());
                let tuple = Value::alloc_tuple(2);
                tuple.set_field(0, error_tag);
                tuple.set_field(1, err_msg);
                tuple
            }
        }
    }
}

/// Extract Result from OCaml result type  
#[cfg(feature = "ocaml-ffi")]
pub fn result_from_ocaml<T>(value: Value) -> Result<T, String>
where
    T: FromValue,
{
    let tag = unsafe { value.field(0).int_val() };
    match tag {
        0 => {
            // Ok case
            let val = unsafe { value.field(1) };
            T::from_value(val).map_err(|_| "Conversion error".to_string())
        }
        1 => {
            // Error case
            let err_msg = unsafe { value.field(1).string_val() };
            Err(err_msg.to_string())
        }
        _ => Err("Invalid result tag".to_string()),
    }
}

/// Get access to the global runtime state
#[cfg(feature = "ocaml-ffi")]
pub fn with_runtime_state<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&mut RuntimeState) -> R,
{
    RUNTIME_STATE
        .lock()
        .map_err(|_| "Failed to acquire runtime state lock".to_string())
        .map(|mut state| f(&mut state))
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "ocaml-ffi")]
    use super::*;
    #[cfg(feature = "ocaml-ffi")]
    use ocaml::Runtime;
    
    #[cfg(feature = "ocaml-ffi")]
    #[test]
    fn test_runtime_initialization() {
        let rt = Runtime::init();
        rt.enter(|_rt| {
            assert!(causality_init());
            assert!(!causality_version().is_empty());
            assert!(causality_cleanup());
        });
    }
    
    #[cfg(feature = "ocaml-ffi")]
    #[test]
    fn test_runtime_state() {
        let mut state = RuntimeState::new();
        
        // Test expression registration  
        let expr = Expr::constant(AstLispValue::Int(42));
        let id = state.register_expression(expr.clone());
        
        assert_eq!(id, 1);
        assert!(state.get_expression(id).is_some());
        
        // Test that we get the same expression back
        if let Some(retrieved_expr) = state.get_expression(id) {
            // Just verify we can access it - detailed comparison would require more complex checking
            assert_eq!(retrieved_expr.kind(), expr.kind());
        }
    }
} 