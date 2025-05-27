//! ZK Guest Interpreter Module
//!
//! This module provides expression interpretation capabilities for the ZK guest
//! environment. It includes a lightweight combinator Lisp interpreter designed
//! for no_std environments with runtime type checking and step counting.
//!
//! The interpreter supports:
//! - Dynamic expression evaluation with step counting
//! - Runtime type checking for ZK circuit compatibility
//! - Combinator-based Lisp evaluation
//! - Constraint validation

//-----------------------------------------------------------------------------
// Interpreter Components
//-----------------------------------------------------------------------------

pub mod combinators;
pub mod core;
pub mod type_checker;

pub use core::{
    interpret_dynamic_expr, interpret_expr, interpret_expr_with_step_limit,
    validate_constraints, InterpreterError, StepCounter, ZkCombinatorInterpreter,
};

pub use type_checker::RuntimeTypeChecker;
