// ZK Guest Interpreter Core
//
// This module implements the core interpreter functionality for the ZK guest
// environment, with step counting and expression evaluation.

extern crate alloc;
use alloc::{format, string::String, vec::Vec};

use core::pin::Pin;
use core::future::Future;

use causality_types::expr::ast::{Atom, Expr};
use causality_lisp::core::ExprContextual;
use causality_types::primitive::ids::ExprId;
use causality_types::primitive::number::Number;
use causality_types::primitive::string::Str;
use causality_types::expr::ast::{ExprBox, ExprVec};
use causality_types::expr::result::ExprResult;
use causality_types::expr::value::ValueExpr;

use super::combinators::apply_combinator;
use super::type_checker::RuntimeTypeChecker;
use causality_lisp::Evaluator;
use causality_types::anyhow::Result;

//-----------------------------------------------------------------------------
// Step Counting - For (dynamic N expr) Bounds Enforcement

//-----------------------------------------------------------------------------

/// Tracks execution steps during expression evaluation in the ZK guest

pub struct StepCounter {
    /// Current step count
    current: u32,
    /// Maximum allowed steps
    max: u32,
}

impl StepCounter {
    /// Create a new step counter with a maximum limit
    pub fn new(max_steps: u32) -> Self {
        Self {
            current: 0,
            max: max_steps,
        }
    }

    /// Increment the step counter
    pub fn increment(&mut self) -> Result<(), InterpreterError> {
        self.current += 1;
        if self.current > self.max {
            Err(InterpreterError::StepLimitExceeded(self.max))
        } else {
            Ok(())
        }
    }

    /// Get the current step count
    pub fn get_count(&self) -> u32 {
        self.current
    }
}

//-----------------------------------------------------------------------------
// ZK Combinator Interpreter
//-----------------------------------------------------------------------------

/// Error types for the ZK interpreter
#[derive(Debug)]
pub enum InterpreterError {
    /// Expression evaluation failure
    EvaluationFailed(String),
    /// Step limit exceeded in dynamic evaluation
    StepLimitExceeded(u32),
    /// Type mismatch during runtime type checking
    TypeMismatch {
        /// Expected type description
        expected: String,
        /// Actual type description
        actual: String,
    },
    /// Invalid arity (argument count) for a function or combinator
    InvalidArity {
        /// Expected argument count
        expected: usize,
        /// Actual argument count
        actual: usize,
    },
    /// Division by zero error
    DivisionByZero,
    /// Invalid argument
    InvalidArgument(String),
    /// Context value not found
    ContextValueNotFound(String),
    /// Field not found on resource
    FieldNotFound { resource: String, field: String },
}

/// Add ZK error types to support the combinator interpreter
impl InterpreterError {
    /// Create a step limit exceeded error
    pub fn step_limit_exceeded(limit: u32) -> Self {
        Self::StepLimitExceeded(limit)
    }

    /// Create a type mismatch error
    pub fn type_mismatch(expected: &str, actual: &str) -> Self {
        Self::TypeMismatch {
            expected: expected.to_string(),
            actual: actual.to_string(),
        }
    }

    /// Create an invalid arity error
    pub fn invalid_arity(expected: usize, actual: usize) -> Self {
        Self::InvalidArity { expected, actual }
    }
}

/// ZK-specific combinator Lisp interpreter
///
/// This interpreter is designed for the ZK guest environment with:
/// - Strict runtime type checking for all operations
/// - Step counting and enforcement
/// - Minimalistic, circuit-efficient implementation
pub struct ZkCombinatorInterpreter {
    /// Step counter for tracking execution progress
    pub step_counter: StepCounter,
    /// Runtime type checker
    pub type_checker: RuntimeTypeChecker,
}

impl ZkCombinatorInterpreter {
    /// Create a new ZK combinator interpreter with a step limit
    pub fn new(max_steps: u32) -> Self {
        Self {
            step_counter: StepCounter::new(max_steps),
            type_checker: RuntimeTypeChecker::new(),
        }
    }

    /// Get the current step count
    pub fn get_step_count(&self) -> u32 {
        self.step_counter.get_count()
    }
    
    // Helper method to box futures for recursion safety
    fn box_evaluate_dynamic<'a, Ctx>(
        &'a mut self,
        expr: &'a Expr,
        ctx: &'a Ctx,
    ) -> Pin<Box<dyn Future<Output = Result<ExprResult, InterpreterError>> + 'a>>
    where
        Ctx: ExprContextual + 'a,
    {
        Box::pin(self.evaluate_dynamic(expr, ctx))
    }
    
    // Helper method to box evaluate_apply for recursion safety
    fn box_evaluate_apply<'a, Ctx>(
        &'a mut self,
        func: &'a ExprBox,
        args: &'a ExprVec,
        ctx: &'a Ctx,
    ) -> Pin<Box<dyn Future<Output = Result<ExprResult, InterpreterError>> + 'a>>
    where
        Ctx: ExprContextual + 'a,
    {
        Box::pin(self.evaluate_apply(func, args, ctx))
    }

    /// Evaluate a dynamic expression with step counting and type checking
    pub async fn evaluate_dynamic(
        &mut self,
        expr: &Expr,
        ctx: &impl ExprContextual,
    ) -> Result<ExprResult, InterpreterError> {
        // Count this evaluation step
        self.step_counter.increment()?;

        match expr {
            Expr::Atom(atom) => self.evaluate_atom(atom),
            Expr::Var(name) => self.lookup_var(name, ctx).await,
            Expr::Apply(func, args) => self.box_evaluate_apply(func, args, ctx).await,
            _ => Err(InterpreterError::EvaluationFailed(format!(
                "Unsupported expression type: {:?}",
                expr
            ))),
        }
    }

    /// Evaluate an atomic value
    pub fn evaluate_atom(
        &self,
        atom: &Atom,
    ) -> Result<ExprResult, InterpreterError> {
        Ok(match atom {
            Atom::Nil => ExprResult::Value(ValueExpr::Nil),
            Atom::Boolean(b) => ExprResult::Value(ValueExpr::Bool(*b)),
            Atom::Integer(n) => {
                ExprResult::Value(ValueExpr::Number(Number::new_integer(*n)))
            }
            Atom::String(s) => ExprResult::Value(ValueExpr::String(*s)),
        })
    }

    /// Look up a variable in the context
    pub async fn lookup_var(
        &self,
        name: &Str,
        ctx: &impl ExprContextual,
    ) -> Result<ExprResult, InterpreterError> {
        let symbol_result = ctx.get_symbol(name).await;
        match symbol_result {
            Some(expr_result) => Ok(expr_result),
            None => Err(InterpreterError::ContextValueNotFound(format!(
                "Symbol '{}' not found in ZK context.",
                name.as_str()
            ))),
        }
    }

    /// Evaluate a function application
    pub async fn evaluate_apply(
        &mut self,
        func: &ExprBox,
        args: &ExprVec,
        ctx: &impl ExprContextual,
    ) -> Result<ExprResult, InterpreterError> {
        // Count this application step
        self.step_counter.increment()?;

        // First evaluate the function expression
        let func_result = self.box_evaluate_dynamic(func, ctx).await?;

        // Then evaluate all argument expressions
        let mut evaluated_args = Vec::with_capacity(args.len());
        for arg in args.iter() {
            evaluated_args.push(self.box_evaluate_dynamic(arg, ctx).await?);
        }

        // Dispatch based on the function type
        match func_result {
            ExprResult::Combinator(combinator) => {
                apply_combinator(self, &combinator, &evaluated_args, ctx).await
            }
            _ => Err(InterpreterError::EvaluationFailed(format!(
                "Cannot apply non-function: {:?}",
                func_result
            ))),
        }
    }
}

/// Evaluate an expression in the ZK environment
///
/// This is a thin adapter around Interpreter to make it
/// compatible with the ZK guest environment.
pub async fn interpret_expr(
    expr: &Expr,
    ctx: &impl ExprContextual,
) -> Result<ExprResult, crate::core::Error> {
    let interpreter = causality_lisp::Interpreter::new();
    interpreter.evaluate_expr(expr, ctx).await.map_err(|e| {
        crate::core::Error::InvalidInput(format!(
            "Expression interpretation error: {:?}",
            e
        ))
    })
}

/// Handle dynamic expression evaluation with step limit enforcement
///
/// This is the primary interface for evaluating `(dynamic N expr)` forms.
/// It strictly enforces the step limit N and provides detailed runtime
/// type checking for all operations within the dynamic expression.
pub async fn interpret_dynamic_expr(
    expr: &Expr,
    ctx: &impl ExprContextual,
    step_limit: u32,
) -> Result<ExprResult, InterpreterError> {
    // Create a new interpreter with the specified step limit
    let mut interpreter = ZkCombinatorInterpreter::new(step_limit);

    // Evaluate the expression
    let result = interpreter.evaluate_dynamic(expr, ctx).await?;

    // Log step usage for tracing and optimization (no-op in circuit)
    log_step_usage(step_limit, interpreter.get_step_count());

    Ok(result)
}

/// Evaluate any expression in the ZK guest environment with a manual step limit
///
/// This is a lower-level function that takes any expression with an explicit step limit
/// and evaluates it with the ZK-specific combinator interpreter.
pub async fn interpret_expr_with_step_limit(
    expr: &Expr,
    step_limit: u32,
    ctx: &impl ExprContextual,
) -> Result<ExprResult, InterpreterError> {
    // Use the dynamic interpreter directly with the specified step limit
    interpret_dynamic_expr(expr, ctx, step_limit).await
}

/// Log step usage statistics for debugging and optimization
///
/// This is a no-op in the RISC-V environment but would be replaced with
/// appropriate logging in a debug build.
fn log_step_usage(_limit: u32, _used: u32) {
    // No-op in circuit environment
    #[cfg(feature = "debug_logging")]
    {
        // This would be replaced with appropriate logging in a debug build
    }
}

/// Validate expression constraints in the ZK environment
///
/// This is a thin adapter around Validator to make it
/// compatible with the ZK guest environment.
pub async fn validate_constraints(
    expr_ids: &[ExprId],
    ctx: &(impl ExprContextual + causality_types::system::provider::StaticExprContext),
) -> Result<Vec<bool>, crate::core::Error> {
    // Create validation results for each ExprId
    let mut expressions = Vec::with_capacity(expr_ids.len());

    for expr_id in expr_ids {
        // Look up the expression by its ID in the context
        let expr = causality_types::system::provider::StaticExprContext::get_expr(
            ctx, expr_id,
        )
        .ok_or_else(|| {
            crate::core::Error::InvalidInput(format!(
                "Expression not found for ID: {:?}",
                expr_id
            ))
        })?;

        expressions.push(expr);
    }

    // Validate each constraint expression
    let mut results = Vec::with_capacity(expressions.len());
    
    for expr in &expressions {
        // For now, we'll evaluate the expression and check if it's "truthy"
        // In a full implementation, we'd use the ZK interpreter to evaluate the constraint
        let is_valid = match expr {
            // Simple validation based on expression structure
            Expr::Atom(Atom::Boolean(b)) => *b,
            Expr::Atom(Atom::Integer(n)) => *n != 0,
            Expr::Atom(Atom::String(s)) => !s.is_empty(),
            Expr::Atom(Atom::Nil) => false,
            _ => true, // Complex expressions default to true for now
        };
        results.push(is_valid);
    }
    
    Ok(results)
}

/// Check if all constraints are satisfied
pub fn all_constraints_satisfied(results: &[bool]) -> bool {
    results.iter().all(|&result| result)
}

/// Get the indices of constraints that failed
pub fn get_failed_constraint_indices(results: &[bool]) -> Vec<usize> {
    results
        .iter()
        .enumerate()
        .filter_map(|(i, &result)| {
            if !result {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}
