//! TEL Lisp Bridge
//!
//! Bridges between TEL/Causality types and Lisp interpreter types.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

use anyhow::Result;
use causality_types::expr::ExprError as LispError;
use causality_types::core::number::Number;
use causality_types::expr::ast::Atom as LispAtom;
use causality_types::expr::result::ExprResult as LispValue;
use causality_types::expr::value::ValueExpr;

//-----------------------------------------------------------------------------
// Error Types
//-----------------------------------------------------------------------------

/// Error types for bridging TEL and Lisp values.
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Type mismatch during Lisp to ValueExpr conversion: {0}")]
    LispToValueMismatch(String),
    #[error("Type mismatch during ValueExpr to Lisp conversion: {0}")]
    ValueToLispMismatch(String),
    #[error("Unsupported Lisp type for conversion: {0}")]
    UnsupportedLispType(String),
    #[error("Unsupported ValueExpr type for conversion: {0}")]
    UnsupportedValueType(String),
    #[error("Lisp execution error during conversion: {0}")]
    LispExecutionError(#[from] LispError),
    #[error("Conversion failed: {0}")]
    ConversionFailed(String),
}

//-----------------------------------------------------------------------------
// Conversion Functions
//-----------------------------------------------------------------------------

/// Converts a `causality_types::expr::value::ValueExpr` to a `causality_types::expr::result::ExprResult`.
pub fn value_expr_to_lisp_value(
    value_expr: ValueExpr,
) -> Result<LispValue, BridgeError> {
    Ok(LispValue::Value(value_expr)) 
}

/// Converts a `causality_types::expr::result::ExprResult` to a `causality_types::expr::value::ValueExpr`.
pub fn lisp_value_to_value_expr(
    lisp_value: LispValue,
) -> Result<ValueExpr, LispError> {
    match lisp_value {
        LispValue::Value(v) => Ok(v),
        LispValue::Atom(atom) => {
            // LispAtom is causality_types::expr::ast::Atom
            match atom {
                LispAtom::Integer(i) => Ok(ValueExpr::Number(Number::Integer(i))),
                LispAtom::String(s) => Ok(ValueExpr::String(s)), 
                LispAtom::Boolean(b) => Ok(ValueExpr::Bool(b)),
                LispAtom::Nil => Ok(ValueExpr::Nil),
            }
        }
        LispValue::Bool(b) => Ok(ValueExpr::Bool(b)),
        LispValue::Unit => Ok(ValueExpr::Nil),
        // Other ExprResult variants are not directly convertible to a single ValueExpr
        _ => Err(BridgeError::UnsupportedLispType(format!(
            "LispValue variant {:?} not supported for direct conversion to ValueExpr.",
            lisp_value
        ))),
    }
}

// Helper to attempt to get i64 from LispValue if it's a number
// This depends on the actual structure of LispValue::Number
// For example, if LispValue has a method like `try_as_i64()` or similar.
// This is illustrative and needs to be adapted to `causality_lisp::ValueExpr`.
/*
fn lisp_value_try_into_i64(lisp_value: &LispValue) -> Result<i64, BridgeError> {
    match lisp_value {
        LispValue::Atom(causality_lisp::Atom::Integer(i)) => Ok(*i),
        // Add other numeric LispValue variants if they exist and can be converted to i64
        _ => Err(BridgeError::LispToValueMismatch(format!("Expected Lisp Integer atom, found {:?}", lisp_value))),
    }
}
*/ 
