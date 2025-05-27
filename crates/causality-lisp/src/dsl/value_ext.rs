//-----------------------------------------------------------------------------
// ValueExpr Extension Methods for Testing and DSL Construction
//-----------------------------------------------------------------------------

use causality_types::{
    core::numeric::Number,
    expr::value::ValueExpr,
};

/// Extension trait for ValueExpr to provide convenient constructors
pub trait ValueExprExt {
    /// Creates a numeric (integer) ValueExpr from an i64 value
    fn integer(value: i64) -> ValueExpr;
    
    /// Creates a boolean ValueExpr
    fn bool(value: bool) -> ValueExpr;
}

impl ValueExprExt for ValueExpr {
    fn integer(value: i64) -> ValueExpr {
        ValueExpr::Number(Number::Integer(value))
    }
    
    fn bool(value: bool) -> ValueExpr {
        ValueExpr::Bool(value)
    }
}
