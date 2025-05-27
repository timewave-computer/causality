// Combinator Lisp Validator
//
// This module provides a validator for the combinator Lisp expressions.
// It focuses on validating basic constraints with minimal resource usage.

// Environment-specific imports
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec::Vec};
#[cfg(not(feature = "std"))]
use core::fmt;
#[cfg(feature = "std")]
use std::{fmt, vec::Vec};

use causality_types::expr::ast::Expr;

use crate::interpreter::Interpreter;
use crate::{core::Evaluator, core::ExprContextual};

//-----------------------------------------------------------------------------
// Validator Implementation
//-----------------------------------------------------------------------------

/// Trait for validating expressions
pub trait Validate<T> {
    /// Validate an expression against constraints
    fn validate(&self, expr: &T, ctx: &impl ExprContextual) -> bool;
}

/// Trait for types that can be validated
pub trait Validatable {
    /// Get validation expressions for this type
    fn get_validation_exprs(&self) -> Vec<&Expr>;
}

/// Validator for combinator Lisp expressions
pub struct Validator {
    interpreter: Interpreter,
}

impl Validator {
    /// Create a new validator
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    /// Async validate implementation that waits for futures to complete
    pub async fn validate_async(
        &self,
        expr: &Expr,
        ctx: &impl ExprContextual,
    ) -> bool {
        match self.interpreter.evaluate_expr(expr, ctx).await {
            Ok(result) => crate::core::is_truthy(&result),
            Err(_) => false,
        }
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Validator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validator")
    }
}

impl Validate<Expr> for Validator {
    /// Validate a single expression
    /// Note: This is a synchronous API that returns a default value
    /// for async operations. Use validate_async for proper async handling.
    fn validate(&self, _expr: &Expr, _ctx: &impl ExprContextual) -> bool {
        // We can't properly evaluate in a sync context because evaluate_expr is async
        // Return a default value or log a warning
        false
    }
}

/// Validate a list of constraints on a resource asynchronously
pub async fn validate_resource_async(
    exprs: Vec<&Expr>,
    ctx: &impl ExprContextual,
) -> bool {
    let validator = Validator::new();

    for expr in exprs {
        if !validator.validate_async(expr, ctx).await {
            return false;
        }
    }

    true
}

/// Validate multiple constraints asynchronously
pub async fn validate_constraints_async(
    exprs: Vec<&Expr>,
    ctx: &impl ExprContextual,
) -> Vec<bool> {
    let validator = Validator::new();
    let mut results = Vec::with_capacity(exprs.len());

    for expr in exprs {
        results.push(validator.validate_async(expr, ctx).await);
    }

    results
}

/// Synchronous version that always returns an empty result
/// This is a placeholder for API compatibility
pub fn validate_constraints(
    _exprs: Vec<&Expr>,
    _ctx: &impl ExprContextual,
) -> Vec<bool> {
    Vec::new()
}

/// Synchronous version that always returns false
/// This is a placeholder for API compatibility
pub fn validate_resource(_exprs: Vec<&Expr>, _ctx: &impl ExprContextual) -> bool {
    false
}
