//! TEL Lisp Adapter
//!
//! Adapts the TEL execution context to be usable by the Lisp interpreter.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

use anyhow::Result; // This Result is from anyhow, used in AsExprContext impl
use async_trait::async_trait;
use std::collections::BTreeMap; // For Map conversion

use causality_lisp::core::ExprContextual as LispCoreExprContextual; // Alias to avoid clash
// Aliases for lisp interpreter's value and error types
use causality_types::expr::ValueExpr as LispValue;

use causality_types::{
    core::{
        id::{ResourceId},
        str::Str,
    },
    provider::context::{AsExprContext as TypesAsExprContext, TelContextInterface as TypesTelContextInterface}, // Used TypesAsExprContext
    expr::{
        ast::Expr as TypesExpr, 
        value::{ValueExpr, ValueExprMap, ValueExprVec, Number}, // Added ValueExpr for proper import
        result::{ExprResult as TypesExprResult, ExprError as TypesExprError, TypeErrorData},
    },
};
use std::sync::Arc;
use tokio::sync::{Mutex as TokioMutex, MutexGuard as TokioMutexGuard}; 
use crate::tel::traits::HostCallError; // Already imported earlier by `context.rs` changes - check if needed here

use causality_types::expr::ExprError as LispExprError;

use causality_core::id_to_hex; // Import function directly

//-----------------------------------------------------------------------------
// Bridge Error Type
//-----------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum LispBridgeError {
    #[error("Unsupported Lisp value for conversion: {0:?}")]
    UnsupportedLispValue(LispValue), 
    #[error("Unsupported Types value for conversion: {0:?}")]
    UnsupportedTypesValue(ValueExpr), 
    #[error("Recursive conversion error: {0}")]
    RecursiveConversion(String),
}

//-----------------------------------------------------------------------------
// Public Conversion Functions
//-----------------------------------------------------------------------------

pub fn from_lisp_value(lisp_val: LispValue) -> Result<ValueExpr, LispBridgeError> {
    match lisp_val {
        LispValue::Nil => Ok(ValueExpr::Nil),
        LispValue::Unit => Ok(ValueExpr::Unit),
        LispValue::Bool(b) => Ok(ValueExpr::Bool(b)),
        LispValue::String(s) => Ok(ValueExpr::String(s)), // Str is Copy
        LispValue::Number(n) => match n {
            Number::Integer(i) => Ok(ValueExpr::Number(Number::Integer(i))),
            _ => Err(LispBridgeError::UnsupportedLispValue(LispValue::Number(n))), // Only integers supported from Lisp
        },
        LispValue::List(l_list) => {
            let mut t_vec = Vec::new();
            for item in l_list.0.iter() { // Iterate over Vec inside ValueExprVec
                t_vec.push(from_lisp_value(item.clone())?);
            }
            Ok(ValueExpr::List(ValueExprVec(t_vec)))
        }
        LispValue::Map(l_map) => {
            let mut t_map_data = BTreeMap::new();
            for (k, v) in l_map.0.iter() { // Iterate over BTreeMap inside ValueExprMap
                t_map_data.insert(*k, from_lisp_value(v.clone())?);
            }
            Ok(ValueExpr::Map(ValueExprMap(t_map_data)))
        }
        LispValue::Record(l_rec) => {
            let mut t_map_data = BTreeMap::new();
            for (k, v) in l_rec.0.iter() { // Iterate over BTreeMap inside ValueExprMap
                t_map_data.insert(*k, from_lisp_value(v.clone())?);
            }
            Ok(ValueExpr::Record(ValueExprMap(t_map_data)))
        }
        LispValue::Ref(id) => Ok(ValueExpr::Ref(id)),
        LispValue::Lambda { params, body_expr_id, captured_env } => Ok(ValueExpr::Lambda {
            params,
            body_expr_id,
            captured_env,
        }),
        // Explicitly catch other variants if LispValue could diverge, though it's ValueExpr now
        // _ => Err(LispBridgeError::UnsupportedLispValue(lisp_val)), 
    }
}

pub fn to_lisp_value(types_val: ValueExpr) -> Result<LispValue, LispBridgeError> {
    match types_val {
        ValueExpr::Nil => Ok(LispValue::Nil),
        ValueExpr::Unit => Ok(LispValue::Unit),
        ValueExpr::Bool(b) => Ok(LispValue::Bool(b)),
        ValueExpr::String(s) => Ok(LispValue::String(s)), // Str is Copy
        ValueExpr::Number(n) => match n {
            Number::Integer(i) => Ok(LispValue::Number(Number::Integer(i))),
            _ => Err(LispBridgeError::UnsupportedTypesValue(ValueExpr::Number(n))), // Only integers supported to Lisp
        },
        ValueExpr::List(t_list) => {
            let mut l_list_vec = Vec::new();
            for item in t_list.0.iter() { // Iterate over Vec inside ValueExprVec
                l_list_vec.push(to_lisp_value(item.clone()).map_err(|e| LispBridgeError::RecursiveConversion(format!("{:?}", e)))?);
            }
            Ok(LispValue::List(ValueExprVec(l_list_vec)))
        }
        ValueExpr::Map(t_map) => {
            let mut l_map_data = BTreeMap::new();
            for (k, v) in t_map.0.iter() { // Iterate over BTreeMap inside ValueExprMap
                l_map_data.insert(*k, to_lisp_value(v.clone()).map_err(|e| LispBridgeError::RecursiveConversion(format!("{:?}", e)))?);
            }
            Ok(LispValue::Map(ValueExprMap(l_map_data)))
        }
        ValueExpr::Record(rec_val) => {
            let mut l_map_data = BTreeMap::new();
            for (k, v) in rec_val.0.iter() { // Iterate over BTreeMap inside ValueExprMap
                l_map_data.insert(*k, to_lisp_value(v.clone()).map_err(|e| LispBridgeError::RecursiveConversion(format!("{:?}", e)))?);
            }
            Ok(LispValue::Record(ValueExprMap(l_map_data)))
        }
        ValueExpr::Ref(id) => Ok(LispValue::Ref(id)),
        ValueExpr::Lambda { params, body_expr_id, captured_env } => Ok(LispValue::Lambda {
            params,
            body_expr_id,
            captured_env,
        }),
        // All ValueExpr variants are covered.
        // The previous catch-all with ResourceId, ExprId etc. was based on an outdated ValueExpr.
    }
}

//-----------------------------------------------------------------------------
// Adapter Implementation
//-----------------------------------------------------------------------------

/// Wraps a `TelContextInterface` to make it suitable for the `causality-lisp` interpreter.
#[derive(Clone)]
pub struct TelLispAdapter {
    context: Arc<TokioMutex<dyn TypesTelContextInterface + Send + Sync>>, // Use TypesTelContextInterface
}

//-----------------------------------------------------------------------------
// Core Methods
//-----------------------------------------------------------------------------

impl TelLispAdapter {
    pub fn new(
        ctx: Arc<TokioMutex<dyn TypesTelContextInterface + Send + Sync>>, // Use TypesTelContextInterface
    ) -> Self {
        Self { context: ctx }
    }

    async fn get_context_lock(
        &self,
    ) -> TokioMutexGuard<'_, dyn TypesTelContextInterface + Send + Sync> { // Use TypesTelContextInterface
        self.context.lock().await
    }

    fn get_context_blocking_lock(
        &self,
    ) -> TokioMutexGuard<'_, dyn TypesTelContextInterface + Send + Sync> { // Use TypesTelContextInterface
        self.context.blocking_lock()
    }
}

//-----------------------------------------------------------------------------
// AsExprContext Implementation (from causality_types::provider::context)
//-----------------------------------------------------------------------------

// Use the alias TypesAsExprContext which points to causality_types::provider::context::AsExprContext
impl TypesAsExprContext for TelLispAdapter {
    fn get_resource_field(
        &self,
        id: &ResourceId,
        field: &str,
    ) -> Result<Option<ValueExpr>> { // Result from anyhow, ValueExpr is types
        let ec_guard = self.get_context_blocking_lock();
        ec_guard.get_resource_field(id, field)
    }

    fn evaluate_expr(&self, expr: &TypesExpr) -> Result<ValueExpr> { // Expr is TypesExpr, ValueExpr is types
        let ec_guard = self.get_context_blocking_lock();
        ec_guard.evaluate_expr(expr)
    }

    fn is_resource_available(&self, id: &ResourceId) -> Result<bool> {
        let ec_guard = self.get_context_blocking_lock();
        ec_guard.is_resource_available(id)
    }
}

//-----------------------------------------------------------------------------
// ExprContextual Implementation (from causality_lisp::core)
//-----------------------------------------------------------------------------
#[async_trait]
impl LispCoreExprContextual for TelLispAdapter { // Use aliased LispCoreExprContextual
    async fn get_symbol(&self, name: &Str) -> Option<TypesExprResult> { // Changed to async
        let ctx_guard = self.get_context_lock().await; // Use async lock
        // Delegate to the wrapped TypesTelContextInterface's resolve_lisp_symbol method
        // This is the most complete way to get a symbol, including functions/lambdas.
        ctx_guard.resolve_lisp_symbol(name) // Remove .await as the method returns Option<ExprResult> directly
    }

    async fn try_call_host_function(
        &self,
        fn_name: &Str,
        args: Vec<ValueExpr>,
    ) -> Option<Result<ValueExpr, TypesExprError>> { // Returns TypesExprError (aliased LispError)
        let mut ctx_guard = self.get_context_lock().await;
        match ctx_guard.call_host_function(fn_name, args) { // Pass fn_name directly
            Ok(value_expr) => Some(Ok(value_expr)),
            Err(lisp_error) => {
                // Check if the error is a ReferenceError for the specific function name
                if let TypesExprError::ReferenceError { name: ref_name } = &lisp_error {
                    if ref_name == fn_name {
                        log::debug!("Host function '{}' not found by TelContextInterface (via ReferenceError).
", fn_name);
                        return None; // As per ExprContextual trait, return None if function not found
                    }
                }
                // For other errors, propagate them
                log::warn!("Host function '{}' call resulted in LispError: {:?}. Propagating.", fn_name, lisp_error);
                Some(Err(lisp_error)) 
            }
        }
    }

    async fn is_effect_completed(&self, _effect_id: &causality_types::primitive::ids::ExprId) -> Result<bool, TypesExprError> {
        // Since effect completion tracking is not implemented in this adapter,
        // return false to indicate effects are not considered completed
        Ok(false)
    }

    async fn get_expr_by_id(&self, _id: &causality_types::primitive::ids::ExprId) -> Result<&TypesExpr, TypesExprError> {
        // Expression storage by ID is not implemented in this adapter
        Err(TypesExprError::ExecutionError { 
            message: "Expression storage by ID not implemented in TelLispAdapter".into() 
        })
    }

    async fn define_symbol(&self, _name: Str, _value: TypesExprResult) -> Result<(), TypesExprError> {
        // Symbol definition is not supported in this adapter since TelContextInterface
        // doesn't provide symbol storage capabilities
        Err(TypesExprError::ExecutionError { 
            message: "Symbol definition not supported in TelLispAdapter".into() 
        })
    }
}

//-----------------------------------------------------------------------------
// Error Conversions
//-----------------------------------------------------------------------------

impl From<LispExprError> for HostCallError {
    fn from(lisp_err: LispExprError) -> Self {
        match lisp_err {
            LispExprError::TypeError(type_error_data) => {
                HostCallError::ArgumentError(format!(
                    "Lisp type error: {}{}",
                    type_error_data.message,
                    type_error_data.expr
                        .as_ref()
                        .map(|e| format!(" in expression {}", e))
                        .unwrap_or_default()
                ))
            }
            LispExprError::ReferenceError { name } => {
                // If a Lisp ReferenceError occurs, it often means a host function wasn't found by that name.
                HostCallError::FunctionNotFound(name.to_string()) 
            }
            LispExprError::ExecutionError { message } => {
                HostCallError::ExecutionError(format!(
                    "Lisp execution error: {}",
                    message
                ))
            }
            LispExprError::PermissionError { message, resource } => {
                HostCallError::PermissionDenied(format!(
                    "Lisp permission error: {}{}",
                    message,
                    resource
                        .map(|r| format!(" for resource {}", id_to_hex(&r))) // Updated call
                        .unwrap_or_else(|| " for unspecified resource".to_string())
                ))
            }
        }
    }
}

impl From<LispBridgeError> for HostCallError {
    fn from(bridge_err: LispBridgeError) -> Self {
        HostCallError::LispBridgeError(format!("{}", bridge_err)) 
    }
}

impl HostCallError {
    pub fn into_types_expr_error(self) -> TypesExprError { // TypesExprError is causality_types::expr::result::ExprError
        match self {
            HostCallError::ResourceNotFound(s) => TypesExprError::ReferenceError { name: format!("Resource not found: {}", s).into() },
            HostCallError::NullifierNotFound(s) => TypesExprError::ReferenceError { name: format!("Nullifier not found: {}", s).into() },
            HostCallError::NullifierAlreadySpent(s) => TypesExprError::PermissionError {
                message: format!("Nullifier already spent: {}", s).into(),
                resource: None, // HostCallError doesn't retain ResourceId directly here
            },
            HostCallError::HandlerNotFound(s) => TypesExprError::ExecutionError { message: format!("Handler not found for effect type: {}", s).into() },
            HostCallError::InvalidInput(s) => TypesExprError::TypeError(Box::new(TypeErrorData {
                message: s.into(),
                expr: None,
            })),
            HostCallError::EffectFailed(s) => TypesExprError::ExecutionError { message: format!("Effect failed: {}", s).into() },
            HostCallError::BehaviorFailed(s) => TypesExprError::ExecutionError { message: format!("Behavior failed: {}", s).into() },
            HostCallError::InternalError(s) => TypesExprError::ExecutionError { message: format!("Internal error: {}", s).into() },
            HostCallError::TypeCheckingError(s) => TypesExprError::TypeError(Box::new(TypeErrorData {
                message: s.into(),
                expr: None,
            })),
            HostCallError::ContextError(s) => TypesExprError::ExecutionError { message: format!("Context error: {}", s).into() },
            HostCallError::LispBridgeError(s) => TypesExprError::ExecutionError { message: format!("Lisp bridge error: {}", s).into() },
            HostCallError::GraphError(s) => TypesExprError::ExecutionError { message: format!("Graph error: {}", s).into() },
            HostCallError::ArgumentError(s) => TypesExprError::TypeError(Box::new(TypeErrorData {
                message: s.into(),
                expr: None,
            })),
            HostCallError::ExecutionError(s) => TypesExprError::ExecutionError { message: s.into() },
            HostCallError::FunctionNotFound(s) => TypesExprError::ReferenceError { name: s.into() },
            HostCallError::PermissionDenied(s) => TypesExprError::PermissionError {
                message: s.into(),
                resource: None, // HostCallError stores a combined message
            },
            HostCallError::Custom(s) => TypesExprError::ExecutionError { message: s.into() },
        }
    }

    pub fn into_lisp_runtime_error(self) -> LispExprError { // LispExprError is causality_types::expr::result::ExprError
        self.into_types_expr_error()
    }
}
