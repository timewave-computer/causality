//! Adapter for causality-lisp to work with causality-types
//!
//! This module provides extension traits and utility functions to bridge
//! the gap between causality-types and causality-lisp during refactoring.

use causality_types::{
    core::{
        id::AsId,
        numeric::Number,
        str::Str,
    },
    expr::{
        ast::Expr,
        value::{ValueExpr, ValueExprRef},
    },
};
use crate::utils::core::id_to_hex;

//-----------------------------------------------------------------------------
// String Extension Trait
//-----------------------------------------------------------------------------

/// Extension trait for Str to provide string manipulation methods
pub trait StrExt {
    /// Get a character iterator from a Str
    fn chars(&self) -> std::str::Chars;
    
    /// Convert Str to standard string slice
    fn as_str(&self) -> &str;
}

impl StrExt for Str {
    fn chars(&self) -> std::str::Chars {
        // First convert the bytes to a string slice
        std::str::from_utf8(self.as_ref())
            .unwrap_or_default()
            .chars()
    }
    
    fn as_str(&self) -> &str {
        // Convert the bytes to a string slice
        std::str::from_utf8(self.as_ref())
            .unwrap_or_default()
    }
}

//-----------------------------------------------------------------------------
// ExprBox Utilities
//-----------------------------------------------------------------------------

// Re-export the ExprBox type from causality-types
pub use causality_types::expr::ast::ExprBox;

/// Adapter functions for ExprBox
pub mod expr_box {
    use super::*;
    
    /// Wrap an Expr in an ExprBox
    pub fn new(expr: Expr) -> ExprBox {
        ExprBox(Box::new(expr))
    }
}

//-----------------------------------------------------------------------------
// ID Utilities
//-----------------------------------------------------------------------------

/// Convert ID type to hex string without requiring a reference
pub fn id_to_hex_owned<T: AsId>(id: T) -> String {
    id_to_hex(&id)
}

//-----------------------------------------------------------------------------
// Expr Adapter Functions
//-----------------------------------------------------------------------------

/// Extension trait to add backwards compatibility methods to Expr
pub trait ExprExt {
    /// Creates a string literal expression
    fn string(s: String) -> Expr;
    
    /// Creates an integer literal expression
    fn integer(n: i64) -> Expr;
    
    /// Creates a boolean literal expression
    fn boolean(b: bool) -> Expr;
}

impl ExprExt for Expr {
    fn string(s: String) -> Expr {
        // Convert to our Str type using from_string
        let str_val = Str::from_string(s);
        Expr::Const(ValueExpr::String(str_val))
    }
    
    fn integer(n: i64) -> Expr {
        Expr::Const(ValueExpr::Number(Number::Integer(n)))
    }
    
    fn boolean(b: bool) -> Expr {
        Expr::Const(ValueExpr::Bool(b))
    }
}

//-----------------------------------------------------------------------------
// Value Expression Reference Adapter
//-----------------------------------------------------------------------------

/// Adapter functions for ValueExprRef to handle missing variants
pub mod value_expr_ref {
    use super::*;
    
    /// Emulate the Type variant from the old ValueExprRef
    pub fn is_type(_value: &ValueExprRef) -> bool {
        // Implement based on the new structure
        false
    }
    
    /// Emulate the Capability variant from the old ValueExprRef
    pub fn is_capability(_value: &ValueExprRef) -> bool {
        // Implement based on the new structure
        false
    }
}
