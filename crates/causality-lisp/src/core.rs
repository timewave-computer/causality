//! Core Definitions for the Causality Combinator Lisp
//!
//! This module provides the essential traits, interfaces, and utilities
//! for the Causality Combinator Lisp system.

use causality_types::{
    anyhow::Result,
    core::id::{ExprId, ResourceId},
    // ValueExprId was here, removed as unused
    core::str::Str,
    core::numeric::Number, // Added for Number::Integer, Number::BigInt
    expr::result::ExprError,
    expr::{ast::Expr, result::ExprResult, value::ValueExpr},
    provider::context::AsExprContext,
    // resource::Resource, // For AsExprContext methods
};

#[cfg(feature = "async")]
#[cfg_attr(not(feature = "std"), async_trait::async_trait(?Send))]
#[cfg_attr(feature = "std", async_trait::async_trait)]
pub trait ExprContextual: AsExprContext + Send + Sync {
    /// Get a symbol's value from the context
    async fn get_symbol(&self, name: &Str) -> Option<ExprResult>;

    /// Attempt to call a host function.
    /// Returns `Ok(Some(result))` if the host function was found and executed successfully.
    /// Returns `Ok(None)` if no host function with that name is registered in this context.
    /// Returns `Err(ExprError)` if the host function was found but its execution failed.
    async fn try_call_host_function(
        &self,
        fn_name: &Str,
        args: Vec<ValueExpr>,
    ) -> Option<Result<ValueExpr, ExprError>>;

    /// Check if a given effect, identified by its ID, has completed.
    /// Returns `Ok(true)` if the effect has completed, `Ok(false)` if it has not,
    /// or `Err(ExprError)` if the effect ID is unknown or an error occurs.
    async fn is_effect_completed(
        &self,
        effect_id: &ExprId,
    ) -> Result<bool, ExprError>;

    /// Retrieve an expression by its ID.
    /// This is crucial for evaluating lambda bodies or other expressions referenced by ID.
    async fn get_expr_by_id(&self, id: &ExprId) -> Result<&Expr, ExprError>;

    /// Defines a symbol in the current context.
    /// Implementers are expected to use interior mutability if necessary.
    async fn define_symbol(
        &self,
        name: Str,
        value: ExprResult,
    ) -> Result<(), ExprError>;

    async fn store_expr_for_lambda_body(&self, _expr: Box<Expr>) -> Result<ExprId, ExprError> {
        // Default implementation returns an error, as concrete types need to provide storage.
        Err(ExprError::ExecutionError { 
            message: Str::from("store_expr_for_lambda_body not implemented for this context"),
        })
    }
}

/// Expression evaluator trait
#[cfg(feature = "async")]
#[cfg_attr(not(feature = "std"), async_trait::async_trait(?Send))]
#[cfg_attr(feature = "std", async_trait::async_trait)]
pub trait Evaluator: Send + Sync {
    /// Evaluate an expression in the given context, producing a typed result
    async fn evaluate_expr(
        &self,
        expr: &Expr,
        ctx: &dyn ExprContextual,
    ) -> Result<ExprResult, ExprError>;
}

/// Evaluates an expression in a given context using a default interpreter.
pub async fn evaluate(expr: &Expr, ctx: &dyn ExprContextual) -> Result<ExprResult, ExprError> {
    // Initialize a default interpreter and delegate to it
    let interpreter = Interpreter::new();
    interpreter.evaluate_expr(expr, ctx).await
}

/// Trait for evaluable expressions
pub trait Evaluable {
    /// Convenience method to evaluate the expression
    fn evaluate(&self, _ctx: &impl ExprContextual) -> Result<ExprResult, ExprError>;
}

// Implementation of Evaluable for Expr
impl Evaluable for Expr {
    fn evaluate(&self, _ctx: &impl ExprContextual) -> Result<ExprResult, ExprError> {
        // This is a placeholder implementation
        // The actual evaluation is done by the interpreter
        Err(ExprError::ExecutionError {
            message: Str::from("Direct evaluation not implemented - use Evaluator"),
        })
    }
}

//-----------------------------------------------------------------------------
// BindingExprContext: A context wrapper for temporary symbol binding
//-----------------------------------------------------------------------------

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, vec::Vec};
#[cfg(feature = "std")]
use std::{collections::BTreeMap, vec::Vec};

/// A context that can hold bindings and optionally chain to a parent context.
// #[derive(Clone)] // Clone might be complex if we have non-Clone ExprContextual parent
#[derive(Debug)]
pub struct BindingExprContext<'a, C: ExprContextual + Send + Sync + ?Sized> {
    original_ctx: &'a C,
    binding_name: Str,
    binding_value: ExprResult,
}

impl<'a, C: ExprContextual + Send + Sync + ?Sized> BindingExprContext<'a, C> {
    /// Create a new binding context that wraps an existing context
    pub fn new(original_ctx: &'a C, name: Str, value: ValueExpr) -> Self {
        BindingExprContext {
            original_ctx,
            binding_name: name,
            binding_value: ExprResult::Value(value), // Ensure value is wrapped in ExprResult::Value
        }
    }

    /// Create a new binding context with multiple bindings.
    /// The `bindings` map should contain `Str` keys and `ExprResult` values.
    pub fn new_with_multiple<M>(
        original_ctx: &'a C,
        bindings: M,
    ) -> Self
    where
        M: Into<BTreeMap<Str, ExprResult>>
    {
        // This constructor needs a way to store multiple bindings.
        // For now, it will pick the first binding to satisfy the struct fields
        // and rely on the `ExprContextual` trait impl to handle multiple bindings logic.
        // This is a simplification and might need a struct redesign for true multiple initial bindings.
        let btree_bindings = bindings.into();
        let (name, value) = btree_bindings.into_iter().next().unwrap_or_else(|| 
            // Provide default dummy binding if map is empty, to satisfy struct fields.
            // This choice might affect behavior if not handled carefully in get_symbol.
            (Str::from("__dummy_binding_if_empty__"), ExprResult::Value(ValueExpr::Unit)) // Changed from Null to Unit
        );

        BindingExprContext {
            original_ctx,
            binding_name: name, 
            binding_value: value, 
        }
    }

    /// Set a symbol's value in this context.
    pub fn set_symbol(&mut self, _name: Str, value: ExprResult) {
        self.binding_value = value;
    }

    /// Get a symbol's value from this context or its parent(s).
    pub fn get_symbol(&self, name: &Str) -> Option<ExprResult> {
        if self.binding_name == *name {
            Some(self.binding_value.clone())
        } else {
            // This needs to be async if original_ctx.get_symbol is async
            // self.original_ctx.get_symbol(name) // This line will be handled by the ExprContextual impl
            panic!("BindingExprContext::get_symbol (non-async) should not be directly called if original_ctx.get_symbol is async. Use ExprContextual version.");
        }
    }
}

// Implementation of AsExprContext for BindingExprContext
impl<'a, C: ExprContextual + Send + Sync + ?Sized> AsExprContext
    for BindingExprContext<'a, C>
{
    fn get_resource_field(
        &self,
        id: &ResourceId,
        field: &str,
    ) -> anyhow::Result<Option<ValueExpr>> {
        self.original_ctx.get_resource_field(id, field)
    }

    fn evaluate_expr(&self, expr: &Expr) -> anyhow::Result<ValueExpr> {
        self.original_ctx.evaluate_expr(expr)
    }

    fn is_resource_available(&self, id: &ResourceId) -> anyhow::Result<bool> {
        self.original_ctx.is_resource_available(id)
    }
}

// Ensure ExprContextual uses the new BindingExprContext for its default `get_symbol` if applicable
// or that BindingExprContext correctly implements ExprContextual itself.

#[cfg_attr(feature = "std", async_trait::async_trait)]
impl<'a, C: ExprContextual + Send + Sync + ?Sized> ExprContextual
    for BindingExprContext<'a, C>
{
    async fn get_symbol(&self, name: &Str) -> Option<ExprResult> {
        if self.binding_name == *name {
            Some(self.binding_value.clone())
        } else {
            self.original_ctx.get_symbol(name).await
        }
    }

    async fn try_call_host_function(
        &self,
        fn_name: &Str,
        args: Vec<ValueExpr>,
    ) -> Option<Result<ValueExpr, ExprError>> {
        self.original_ctx
            .try_call_host_function(fn_name, args)
            .await
    }

    async fn is_effect_completed(
        &self,
        effect_id: &ExprId,
    ) -> Result<bool, ExprError> {
        self.original_ctx.is_effect_completed(effect_id).await
    }

    async fn get_expr_by_id(&self, id: &ExprId) -> Result<&Expr, ExprError> {
        self.original_ctx.get_expr_by_id(id).await
    }

    async fn define_symbol(
        &self,
        name: Str,
        _value: ExprResult,
    ) -> Result<(), ExprError> {
        Err(ExprError::ExecutionError { 
            message: Str::from(format!("Cannot define symbol '{}' in a let-binding context (BindingExprContext). Let bindings are immutable.", name))
        })
    }
}

//-----------------------------------------------------------------------------
// LambdaBindingContext: Context for lambda execution
//-----------------------------------------------------------------------------

/// Context for lambda execution, layering local bindings (captured + params)
/// over an outer context.
#[derive(Debug, Clone)]
pub struct LambdaBindingContext<'a, C: ExprContextual + Send + Sync + ?Sized> {
    pub outer_context: &'a C,
    pub bindings: BTreeMap<Str, ValueExpr>, // Stores ValueExpr directly for let/lambda params
}

impl<'a, C: ExprContextual + Send + Sync + ?Sized> LambdaBindingContext<'a, C> {
    pub fn new(outer_ctx: &'a C, local_bindings: BTreeMap<Str, ExprResult>) -> Self {
        let converted_bindings = local_bindings
            .into_iter()
            .filter_map(|(k, v)| {
                match v {
                    ExprResult::Value(val) => Some((k, val)),
                    ExprResult::Atom(atom) => match atom {
                        causality_types::expr::ast::Atom::Nil => Some((k, ValueExpr::Unit)),
                        causality_types::expr::ast::Atom::Boolean(b) => Some((k, ValueExpr::Bool(b))),
                        causality_types::expr::ast::Atom::String(s) => Some((k, ValueExpr::String(s))),
                        causality_types::expr::ast::Atom::Integer(i) => Some((k, ValueExpr::Number(Number::Integer(i)))),
                        // Atom does not have BigInt or Float variants
                    },
                    ExprResult::Bool(b) => Some((k, ValueExpr::Bool(b))),
                    ExprResult::Unit => Some((k, ValueExpr::Unit)),
                    _ => None, // Skip other ExprResult variants
                }
            })
            .collect();

        LambdaBindingContext {
            outer_context: outer_ctx,
            bindings: converted_bindings,
        }
    }
}

// Implement AsExprContext by delegating to outer_ctx for non-symbol operations
impl<'a, C: ExprContextual + Send + Sync + ?Sized> AsExprContext
    for LambdaBindingContext<'a, C>
{
    fn get_resource_field(
        &self,
        id: &ResourceId,
        field: &str,
    ) -> Result<Option<ValueExpr>> {
        self.outer_context.get_resource_field(id, field)
    }

    fn evaluate_expr(&self, expr: &Expr) -> Result<ValueExpr> {
        self.outer_context.evaluate_expr(expr)
    }

    fn is_resource_available(&self, id: &ResourceId) -> Result<bool> {
        self.outer_context.is_resource_available(id)
    }
}

#[cfg(feature = "async")]
#[cfg_attr(not(feature = "std"), async_trait::async_trait(?Send))]
#[cfg_attr(feature = "std", async_trait::async_trait)]
impl<'a, C: ExprContextual + Send + Sync + ?Sized> ExprContextual
    for LambdaBindingContext<'a, C>
{
    async fn get_symbol(&self, name: &Str) -> Option<ExprResult> {
        // Check local bindings first
        if let Some(local_val) = self.bindings.get(name) {
            return Some(ExprResult::Value(local_val.clone()));
        }
        // If not found locally, delegate to outer context
        self.outer_context.get_symbol(name).await
    }

    async fn try_call_host_function(
        &self,
        fn_name: &Str,
        args: Vec<ValueExpr>,
    ) -> Option<Result<ValueExpr, ExprError>> {
        self.outer_context.try_call_host_function(fn_name, args).await
    }

    async fn is_effect_completed(
        &self,
        effect_id: &ExprId,
    ) -> Result<bool, ExprError> {
        self.outer_context.is_effect_completed(effect_id).await
    }

    async fn get_expr_by_id(&self, id: &ExprId) -> Result<&Expr, ExprError> {
        self.outer_context.get_expr_by_id(id).await
    }

    async fn define_symbol(
        &self,
        name: Str,
        _value: ExprResult,
    ) -> Result<(), ExprError> {
        Err(ExprError::ExecutionError { 
            message: Str::from(format!("Cannot define symbol '{}' in a lambda execution context (LambdaBindingContext). Scope is fixed by parameters and captures.", name))
        })
    }
}

//-----------------------------------------------------------------------------
// Type Utilities
//-----------------------------------------------------------------------------

/// Get the type name of an expression result
pub fn type_name(result: &ExprResult) -> &'static str {
    match result {
        ExprResult::Value(value) => match value {
            ValueExpr::Number(_) => "number",
            ValueExpr::String(_) => "string",
            ValueExpr::Bool(_) => "bool",
            ValueExpr::Unit => "nil",
            ValueExpr::Nil => "nil",
            ValueExpr::List(_) => "list",
            ValueExpr::Map(_) => "map",
            ValueExpr::Record(_) => "record",
            ValueExpr::Ref(_) => "reference",
            ValueExpr::Lambda { .. } => "lambda",
        },
        ExprResult::Bool(_) => "bool",
        ExprResult::Atom(_) => "atom",
        ExprResult::Resource(_) => "resource",
        ExprResult::Combinator(_) => "combinator",
        ExprResult::Function(_) => "function",
        ExprResult::Unit => "nil",
        ExprResult::ExternalHostFnRef(_) => "host function",
    }
}

/// Check if a value is numeric
pub fn is_numeric(result: &ExprResult) -> bool {
    match result {
        ExprResult::Value(value) => matches!(value, ValueExpr::Number(_)),
        ExprResult::Atom(atom) => {
            matches!(atom, causality_types::expr::ast::Atom::Integer(_))
        }
        _ => false,
    }
}

/// Check if a value is truthy
// All values are truthy except for explicit false, nil, and zero
pub fn is_truthy(result: &ExprResult) -> bool {
    match result {
        ExprResult::Value(value) => match value {
            ValueExpr::Bool(b) => *b,
            ValueExpr::Number(n) => !matches!(n, causality_types::primitive::number::Number::Integer(0)),
            ValueExpr::String(s) => !s.is_empty(), // Empty string is falsy
            ValueExpr::Unit => false,              // nil is falsy
            ValueExpr::Nil => false,               // nil is falsy
            ValueExpr::List(list) => !list.0.is_empty(), // Empty list is falsy
            ValueExpr::Map(map) => !map.0.is_empty(), // Empty map is falsy
            ValueExpr::Record(record) => !record.0.is_empty(), // Empty record is falsy
            ValueExpr::Ref(_) => true, // References are always truthy
            ValueExpr::Lambda { .. } => true, // Lambdas (closures) are truthy
        },
        ExprResult::Bool(b) => *b,   // Direct booleans
        ExprResult::Atom(_) => true, // Atoms are truthy
        ExprResult::Resource(_) => true,
        ExprResult::Combinator(_) => true,
        ExprResult::Function(_) => true,
        ExprResult::Unit => false,
        ExprResult::ExternalHostFnRef(_) => true,
    }
}

//-----------------------------------------------------------------------------
// Concrete Interpreter Implementation
//-----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self
    }
}

#[cfg_attr(not(feature = "std"), async_trait::async_trait(?Send))]
#[cfg_attr(feature = "std", async_trait::async_trait)]
impl Evaluator for Interpreter {
    async fn evaluate_expr(
        &self,
        expr: &Expr,
        ctx: &dyn ExprContextual,
    ) -> Result<ExprResult, ExprError> {
        match expr {
            Expr::Atom(atom) => {
                // Convert atoms to ExprResult::Atom to match test expectations
                Ok(ExprResult::Atom(atom.clone()))
            }
            Expr::Const(value_expr) => {
                // Return the constant value
                Ok(ExprResult::Value(value_expr.clone()))
            }
            Expr::Var(name) => {
                // Look up variable in context
                ctx.get_symbol(name).await.ok_or_else(|| ExprError::ExecutionError {
                    message: Str::from(format!("Undefined variable: {}", name)),
                })
            }
            Expr::Lambda(_params, _body) => {
                // TODO: Implement lambda evaluation
                // For now, return a placeholder value
                Ok(ExprResult::Value(ValueExpr::String(Str::from("lambda"))))
            }
            Expr::Apply(_func_expr, _args) => {
                // TODO: Implement function application
                // For now, return a placeholder
                Ok(ExprResult::Value(ValueExpr::String(Str::from("apply_result"))))
            }
            Expr::Combinator(combinator) => {
                // Return the combinator as a result
                Ok(ExprResult::Combinator(combinator.clone()))
            }
            Expr::Dynamic(_step_limit, _inner_expr) => {
                // Dynamic evaluation is not yet implemented
                Err(ExprError::ExecutionError {
                    message: Str::from("Dynamic evaluation not yet implemented"),
                })
            }
        }
    }
}
