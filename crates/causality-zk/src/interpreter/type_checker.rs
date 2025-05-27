//! Runtime Type Checking for ZK Guest Environment
//!
//! This module provides runtime type checking for dynamic expressions in the ZK
//! guest environment. It ensures that values conform to their expected types
//! during execution. Type checking is essential for ensuring that ZK circuits
//! operate as expected with properly typed inputs and outputs.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

extern crate alloc;

use causality_types::expr::expr_type::TypeExpr;
use causality_types::expr::result::ExprResult;
use causality_types::expr::value::ValueExpr;

use super::core::InterpreterError;

//-----------------------------------------------------------------------------
// Runtime Type Checker
//-----------------------------------------------------------------------------

/// Runtime type checker for dynamic expressions

pub struct RuntimeTypeChecker;

impl Default for RuntimeTypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeTypeChecker {
    /// Create a new runtime type checker
    pub fn new() -> Self {
        Self
    }

    /// Check that a value has the expected type
    pub fn check_value_type(
        &self,
        value: &ExprResult,
        expected_type: TypeExpr,
    ) -> Result<(), InterpreterError> {
        match value {
            ExprResult::Value(v) => self.check_value_expr_type(v, expected_type),
            _ => Err(InterpreterError::type_mismatch(
                &format!("{:?}", expected_type),
                &format!("Non-value: {:?}", value),
            )),
        }
    }

    /// Check that a ValueExpr has the expected type
    fn check_value_expr_type(
        &self,
        value: &ValueExpr,
        expected_type: TypeExpr,
    ) -> Result<(), InterpreterError> {
        let matches = match (&expected_type, value) {
            (TypeExpr::Unit, ValueExpr::Unit) => true,
            (TypeExpr::Bool, ValueExpr::Bool(_)) => true,
            (TypeExpr::String, ValueExpr::String(_)) => true,
            (TypeExpr::Number, ValueExpr::Number(_)) => true,
            (TypeExpr::List(element_type), ValueExpr::List(elements)) => {
                // Check that all elements match the expected type
                elements.iter().all(|element| check_type_compatibility(element_type, element))
            }
            (TypeExpr::Map(_, _), ValueExpr::Map(_)) => true, // Matching Map with key and value type parameters
            (TypeExpr::Record(type_fields), ValueExpr::Record(value_fields)) => {
                // Check that all required fields are present and have correct types
                type_fields.iter().all(|(field_name, field_type)| {
                    value_fields.get(field_name)
                        .map(|field_value| check_type_compatibility(field_type, field_value))
                        .unwrap_or(false)
                })
            }
            _ => false,
        };

        if matches {
            Ok(())
        } else {
            Err(InterpreterError::type_mismatch(
                &format!("{:?}", expected_type),
                &format!("{:?}", value),
            ))
        }
    }

    /// Check arguments against expected types
    pub fn check_args(
        &self,
        args: &[ExprResult],
        expected_types: &[TypeExpr],
    ) -> Result<(), InterpreterError> {
        // Check arity first
        if args.len() != expected_types.len() {
            return Err(InterpreterError::invalid_arity(
                expected_types.len(),
                args.len(),
            ));
        }

        // Check each argument type
        for (i, (arg, expected_type)) in
            args.iter().zip(expected_types.iter()).enumerate()
        {
            self.check_value_type(arg, expected_type.clone()).map_err(
                |e| match e {
                    InterpreterError::TypeMismatch { expected, actual } => {
                        InterpreterError::TypeMismatch {
                            expected,
                            actual: format!("Argument {}: {}", i, actual),
                        }
                    }
                    other => other,
                },
            )?;
        }

        Ok(())
    }
}

/// Helper function to check type compatibility between a type and a value
fn check_type_compatibility(expected_type: &TypeExpr, value: &ValueExpr) -> bool {
    match (expected_type, value) {
        (TypeExpr::Unit, ValueExpr::Unit) => true,
        (TypeExpr::Bool, ValueExpr::Bool(_)) => true,
        (TypeExpr::String, ValueExpr::String(_)) => true,
        (TypeExpr::Number, ValueExpr::Number(_)) => true,
        (TypeExpr::List(element_type), ValueExpr::List(elements)) => {
            // Check that all elements match the expected type
            elements.iter().all(|element| check_type_compatibility(element_type, element))
        }
        (TypeExpr::Map(_, _), ValueExpr::Map(_)) => true, // Simplified map type checking
        (TypeExpr::Record(type_fields), ValueExpr::Record(value_fields)) => {
            // Check that all required fields are present and have correct types
            type_fields.iter().all(|(field_name, field_type)| {
                value_fields.get(field_name)
                    .map(|field_value| check_type_compatibility(field_type, field_value))
                    .unwrap_or(false)
            })
        }
        _ => false,
    }
}
