//! Layer 1 (Core Causality) FFI interface for OCaml

use crate::ocaml::core_types::*;
use causality_core::lambda::base::Value as CoreValue;
use causality_lisp::ast::{LispValue as AstLispValue, Expr};

/// Convert Core Value to AST LispValue for Layer 1 compatibility
pub fn convert_core_value_to_ast(value: &CoreValue) -> Result<AstLispValue, String> {
    match value {
        CoreValue::Unit => Ok(AstLispValue::Unit),
        CoreValue::Bool(b) => Ok(AstLispValue::Bool(*b)),
        CoreValue::Int(i) => Ok(AstLispValue::Int(*i as i64)),
        CoreValue::String(s) => Ok(AstLispValue::String(s.value.clone().into())),
        CoreValue::Symbol(sym) => Ok(AstLispValue::Symbol(sym.clone().into())),
        _ => Err(format!("Unsupported Core Value variant: {:?}", value)),
    }
}

/// Convert AST LispValue to Core Value
pub fn convert_ast_to_core_value(ast_value: &AstLispValue) -> Result<CoreValue, String> {
    match ast_value {
        AstLispValue::Unit => Ok(CoreValue::Unit),
        AstLispValue::Bool(b) => Ok(CoreValue::Bool(*b)),
        AstLispValue::Int(i) => Ok(CoreValue::Int(*i as u32)),
        AstLispValue::String(s) => Ok(CoreValue::String(s.value.clone().into())),
        AstLispValue::Symbol(sym) => Ok(CoreValue::Symbol(sym.value.clone().into())),
        _ => Err(format!("Unsupported AST LispValue variant: {:?}", ast_value)),
    }
}

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