//! Layer 1 (Linear Lambda Calculus) FFI interface for OCaml
//!
//! This module provides FFI bindings for the unified type system that seamlessly 
//! integrates structured types, session types, and location awareness.

use crate::ocaml::core_types::*;
use causality_core::{
    system::content_addressing::EntityId,
    Value as CoreValue, 
    TypeInner, BaseType, Location, SessionType,
    Term, TermKind,
};
use causality_lisp::ast::{LispValue as AstLispValue, Expr, ExprKind};

/// OCaml-compatible location type
#[cfg(feature = "ocaml-ffi")]
#[derive(Debug, Clone, PartialEq, Eq, ocaml::FromValue, ocaml::ToValue)]
pub enum OcamlLocation {
    Local,
    Remote(String),
    Domain(String),
    Any,
}

#[cfg(feature = "ocaml-ffi")]
impl From<Location> for OcamlLocation {
    fn from(loc: Location) -> Self {
        match loc {
            Location::Local => OcamlLocation::Local,
            Location::Remote(s) => OcamlLocation::Remote(s.to_hex()),
            Location::Domain(s) => OcamlLocation::Domain(s),
            Location::Any => OcamlLocation::Any,
            // Map other variants to reasonable defaults
            Location::Distributed(_) => OcamlLocation::Any,
            Location::Edge(s) => OcamlLocation::Domain(s),
            Location::Cloud(s) => OcamlLocation::Domain(s),
            Location::Composite(_) => OcamlLocation::Any,
            Location::Variable(_) => OcamlLocation::Any,
            Location::None => OcamlLocation::Local,
        }
    }
}

#[cfg(feature = "ocaml-ffi")]
impl From<OcamlLocation> for Location {
    fn from(loc: OcamlLocation) -> Self {
        match loc {
            OcamlLocation::Local => Location::Local,
            OcamlLocation::Remote(s) => Location::Remote(EntityId::from_hex(&s).unwrap_or(EntityId::ZERO)),
            OcamlLocation::Domain(s) => Location::Domain(s),
            OcamlLocation::Any => Location::Any,
        }
    }
}

/// Convert Core Value to AST LispValue for Layer 1 compatibility
pub fn convert_core_value_to_ast(value: &CoreValue) -> Result<AstLispValue, String> {
    match value {
        CoreValue::Unit => Ok(AstLispValue::Unit),
        CoreValue::Bool(b) => Ok(AstLispValue::Bool(*b)),
        CoreValue::Int(i) => Ok(AstLispValue::Int(*i as i64)),
        CoreValue::String(s) => Ok(AstLispValue::String(s.clone())),
        // Convert Str to Symbol by creating a new Symbol from the string
        CoreValue::Symbol(s) => {
            use causality_core::lambda::Symbol;
            Ok(AstLispValue::Symbol(Symbol::new(s.as_str())))
        },
        _ => Err(format!("Unsupported Core Value variant: {:?}", value)),
    }
}

/// Convert AST LispValue to Core Value
pub fn convert_ast_to_core_value(ast_value: &AstLispValue) -> Result<CoreValue, String> {
    match ast_value {
        AstLispValue::Unit => Ok(CoreValue::Unit),
        AstLispValue::Bool(b) => Ok(CoreValue::Bool(*b)),
        AstLispValue::Int(i) => Ok(CoreValue::Int(*i as u32)),
        AstLispValue::String(s) => Ok(CoreValue::String(s.clone())),
        // Convert Symbol to Str by getting the string representation
        AstLispValue::Symbol(sym) => {
            use causality_core::system::content_addressing::Str;
            Ok(CoreValue::Symbol(Str::new(sym.as_str())))
        },
        _ => Err(format!("Unsupported AST LispValue variant: {:?}", ast_value)),
    }
}

/// Create a constant expression
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn expr_const(value: LispValue) -> ocaml::Value {
    use crate::ocaml::runtime::with_runtime_state;
    use crate::ocaml::error_handling::result_to_ocaml;
    
    let result = || -> Result<ExprId, String> {
        let core_value = value.to_core()
            .map_err(|e| format!("Failed to convert LispValue: {}", e))?;
        
        // Convert to AST LispValue
        let ast_value = convert_core_value_to_ast(&core_value)?;
        let expr = Expr::new(ExprKind::Const(ast_value));
        
        with_runtime_state(|state| {
            let expr_id = state.register_expression(expr);
            Ok(ExprId::new(expr_id))
        })?
    };
    
    result_to_ocaml(result())
}

/// Create a record field access with location awareness
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn expr_record_access(expr_id: ExprId, field: String) -> ocaml::Value {
    use crate::ocaml::runtime::with_runtime_state;
    use crate::ocaml::error_handling::result_to_ocaml;
    
    let result = || -> Result<ExprId, String> {
        with_runtime_state(|state| {
            let base_expr = state.get_expression(expr_id.id)
                .ok_or("Base expression not found")?
                .clone();
            
            // Create record access expression
            let access_expr = Expr::new(ExprKind::RecordAccess {
                record: Box::new(base_expr),
                field,
            });
            
            let new_id = state.register_expression(access_expr);
            Ok(ExprId::new(new_id))
        })?
    };
    
    result_to_ocaml(result())
}

/// Create a session with expression
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn expr_with_session(session: String, role: String, body_id: ExprId) -> ocaml::Value {
    use crate::ocaml::runtime::with_runtime_state;
    use crate::ocaml::error_handling::result_to_ocaml;
    
    let result = || -> Result<ExprId, String> {
        with_runtime_state(|state| {
            let body_expr = state.get_expression(body_id.id)
                .ok_or("Body expression not found")?
                .clone();
                
            let session_expr = Expr::new(ExprKind::WithSession {
                session,
                role,
                body: Box::new(body_expr),
            });
            
            let expr_id = state.register_expression(session_expr);
            Ok(ExprId::new(expr_id))
        })?
    };
    
    result_to_ocaml(result())
}

/// Create a lambda expression
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn expr_lambda(params: Vec<String>, body_id: ExprId) -> ocaml::Value {
    use crate::ocaml::runtime::with_runtime_state;
    use crate::ocaml::error_handling::result_to_ocaml;
    use causality_lisp::ast::Param;
    
    let result = || -> Result<ExprId, String> {
        with_runtime_state(|state| {
            let body_expr = state.get_expression(body_id.id)
                .ok_or("Body expression not found")?
                .clone();
            
            let params = params.into_iter()
                .map(|name| Param::new(name))
                .collect();
                
            let lambda_expr = Expr::new(ExprKind::Lambda(params, Box::new(body_expr)));
            
            let expr_id = state.register_expression(lambda_expr);
            Ok(ExprId::new(expr_id))
        })?
    };
    
    result_to_ocaml(result())
}

/// Get string representation of an expression for debugging
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn expr_to_string_debug(expr_id: ExprId) -> ocaml::Value {
    use crate::ocaml::runtime::with_runtime_state;
    use crate::ocaml::error_handling::result_to_ocaml;
    
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
    use crate::ocaml::runtime::with_runtime_state;
    
    with_runtime_state(|state| {
        state.get_expression(expr_id.id).is_some()
    }).unwrap_or(false)
}

/// List all registered expressions
#[cfg(feature = "ocaml-ffi")]
#[ocaml::func]
pub fn expr_list_all() -> Vec<ExprId> {
    use crate::ocaml::runtime::with_runtime_state;
    
    with_runtime_state(|state| {
        state.expressions.keys()
            .map(|&id| ExprId::new(id))
            .collect()
    }).unwrap_or_default()
}
