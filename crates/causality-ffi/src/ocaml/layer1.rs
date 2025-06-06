//! Layer 1 (Causality Lisp) bindings for OCaml FFI
//!
//! This module provides OCaml access to Causality Lisp functionality,
//! including expression construction, compilation, and evaluation.

#[cfg(feature = "ocaml-ffi")]
use crate::ocaml::{core_types::*, runtime::*};

#[cfg(feature = "ocaml-ffi")]
use ocaml::Value;
#[cfg(feature = "ocaml-ffi")]
use causality_lisp::ast::{LispValue as AstLispValue, Expr};

/// Create a constant expression
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn expr_const(value: LispValue) -> Value {
    let result = || -> Result<ExprId, String> {
        let core_value = value.to_core()
            .map_err(|e| format!("Failed to convert LispValue: {}", e))?;
        
        // Convert to AST LispValue
        let ast_value = match core_value {
            causality_core::lambda::base::Value::Unit => AstLispValue::Unit,
            causality_core::lambda::base::Value::Bool(b) => AstLispValue::Bool(b),
            causality_core::lambda::base::Value::Int(i) => AstLispValue::Int(i as i64),
            causality_core::lambda::base::Value::String(s) => AstLispValue::String(s.as_str().to_string()),
            causality_core::lambda::base::Value::Symbol(sym) => AstLispValue::Symbol(sym.as_str().to_string()),
            _ => return Err("Unsupported value type for constant".to_string()),
        };
        
        let expr = Expr::constant(ast_value);
        
        with_runtime_state(|state| {
            let expr_id = state.register_expression(expr);
            ExprId::new(expr_id)
        })
    };
    
    result_to_ocaml(result())
}

/// Get string representation of an expression for debugging
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn expr_to_string_debug(expr_id: ExprId) -> Value {
    let result = || -> Result<String, String> {
        with_runtime_state(|state| {
            state.get_expression(expr_id.id)
                .map(|expr| format!("{:?}", expr))
                .ok_or("Expression not found".to_string())
        })?
    };
    
    result_to_ocaml(result())
}

/// Check if an expression exists
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn expr_exists(expr_id: ExprId) -> bool {
    with_runtime_state(|state| {
        state.get_expression(expr_id.id).is_some()
    }).unwrap_or(false)
}

/// List all registered expressions
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn expr_list_all() -> Vec<ExprId> {
    with_runtime_state(|state| {
        state.expressions.keys()
            .map(|&id| ExprId::new(id))
            .collect()
    }).unwrap_or_default()
} 