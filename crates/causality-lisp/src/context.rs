//! Evaluation Context for Causality Combinator Lisp
//!
//! This module implements a minimal evaluation context for the combinator Lisp.

use std::collections::BTreeMap;

use anyhow::Result;
use causality_types::AsExprContext;
use causality_types::{
    core::id::{ExprId, ResourceId},
    core::str::Str,
    expr::{
        ast::Expr,
        result::{ExprError, ExprResult as TypesExprResult}, // Renamed to avoid conflict
        value::ValueExpr,
    },
};

use causality_core::id_to_hex;

use crate::core::ExprContextual;

//-----------------------------------------------------------------------------
// Default Expr Context
//-----------------------------------------------------------------------------

/// Basic context for expression evaluation
pub struct DefaultExprContext {
    /// The ID of this context
    pub id: String,
    /// The current expression being evaluated
    pub current_expr: Option<ExprId>,
    /// Symbol table for variable bindings
    pub symbols: BTreeMap<Str, TypesExprResult>,
}

impl DefaultExprContext {
    /// Create a new expression context
    pub fn new(id: impl Into<String>) -> Self {
        DefaultExprContext {
            id: id.into(),
            current_expr: None,
            symbols: BTreeMap::new(),
        }
    }

    /// Add a symbol to the context
    pub fn add_symbol(&mut self, name: impl Into<Str>, value: TypesExprResult) {
        self.symbols.insert(name.into(), value);
    }
}

impl AsExprContext for DefaultExprContext {
    fn get_resource_field(
        &self,
        _id: &ResourceId,
        _field: &str,
    ) -> Result<Option<ValueExpr>> {
        // For now, DefaultExprContext does not hold resources directly.
        // This would typically delegate to a broader context or state manager.
        Ok(None)
    }
    
    fn evaluate_expr(&self, _expr: &Expr) -> Result<ValueExpr> {
        // DefaultExprContext itself doesn't evaluate; it provides bindings.
        // Evaluation is done by the Interpreter using a context.
        // This might error or return a placeholder if called directly this way.
        Err(anyhow::anyhow!("DefaultExprContext cannot directly evaluate expressions. Use Interpreter."))
    }
    
    fn is_resource_available(&self, _id: &ResourceId) -> Result<bool> {
        // Similar to get_resource_field, assumes resources are managed elsewhere.
        Ok(false) // Default to not available unless a real state manager says otherwise.
    }
}

#[cfg(feature = "async")]
#[cfg_attr(not(feature = "std"), async_trait::async_trait(?Send))]
#[cfg_attr(feature = "std", async_trait::async_trait)]
impl ExprContextual for DefaultExprContext {
    async fn get_symbol(&self, _name: &causality_types::primitive::string::Str) -> Option<TypesExprResult> {
        // Default context does not have any symbols.
        None
    }

    async fn try_call_host_function(
        &self,
        _fn_name: &Str,
        _args: Vec<ValueExpr>,
    ) -> Option<Result<ValueExpr, ExprError>> {
        // Default context does not have any host functions.
        None
    }

    async fn is_effect_completed(
        &self,
        effect_id: &ExprId,
    ) -> Result<bool, ExprError> {
        log::warn!(
            "DefaultExprContext::is_effect_completed called for ID: {}. Defaulting to false.",
            id_to_hex(effect_id)
        );
        Ok(false) // Or an error if this should never be called on DefaultExprContext
    }

    async fn get_expr_by_id(&self, id: &ExprId) -> Result<&Expr, ExprError> {
        Err(ExprError::ExecutionError {
            message: format!(
                "DefaultExprContext::get_expr_by_id for id {} - Not Implemented",
                id_to_hex(id)
            )
            .into(),
        })
    }

    async fn define_symbol(
        &self,
        name: Str,
        _value: TypesExprResult,
    ) -> Result<(), ExprError> {
        Err(ExprError::ExecutionError {
            message: Str::from_string(format!("DefaultExprContext::define_symbol for name '{}' - Not Implemented", name)),
        })
    }
}
