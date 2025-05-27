//! Parser Utilities
//!
//! Helper functions and utilities for the Causality Lisp parser,
//! providing AST construction and parsing convenience functions.

//-----------------------------------------------------------------------------
// Parser Utilities
//-----------------------------------------------------------------------------

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use causality_types::expr::ast::{Atom, AtomicCombinator, Expr, ExprBox, ExprVec};

use crate::parser::ast_builder::AstBuilder;
use crate::parser::error::ParseResult;
use crate::parser::lexer::TokenWithLocation;

//-----------------------------------------------------------------------------
// Symbol Utilities
//-----------------------------------------------------------------------------

/// Extract the symbol name from an expression, if it's a variable
pub fn extract_symbol(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Var(s) => Some(s.to_string()),
        _ => None,
    }
}

// Removed unused is_symbol function

//-----------------------------------------------------------------------------
// Parser Utilitie
//-----------------------------------------------------------------------------

/// Parse a single expression from tokens
pub fn parse_expr(tokens: &[TokenWithLocation]) -> ParseResult<Expr> {
    let mut parser = AstBuilder::new(tokens);
    parser.parse_expr()
}

/// Parse a program (multiple expressions) from tokens
pub fn parse_program(tokens: &[TokenWithLocation]) -> ParseResult<Vec<Expr>> {
    let mut parser = AstBuilder::new(tokens);
    parser.parse_program()
}

//-----------------------------------------------------------------------------
// AST Construction Utilities
//-----------------------------------------------------------------------------

/// Create a function application expression
pub fn make_application(func: Expr, args: Vec<Expr>) -> Expr {
    Expr::Apply(ExprBox(Box::new(func)), ExprVec(args))
}

/// Create a nil value
pub fn make_nil() -> Expr {
    Expr::Atom(Atom::Nil)
}

/// Create an if expression
pub fn make_if(condition: Expr, then_expr: Expr, else_expr: Expr) -> Expr {
    // If special form: use the If combinator with the condition, then and else expressions
    let args = vec![condition, then_expr, else_expr];
    make_application(Expr::Combinator(AtomicCombinator::If), args)
}

/// Create a list expression from a vector of expressions
pub fn make_list(items: Vec<Expr>) -> Expr {
    // If empty list, return nil
    if items.is_empty() {
        return make_nil();
    }

    // Otherwise create a list application
    make_application(Expr::Combinator(AtomicCombinator::List), items)
}

/// Create an addition expression
pub fn make_add(left: Expr, right: Expr) -> Expr {
    let args = vec![left, right];
    make_application(Expr::Combinator(AtomicCombinator::Add), args)
}

/// Create a subtraction expression
pub fn make_subtract(left: Expr, right: Expr) -> Expr {
    let args = vec![left, right];
    make_application(Expr::Combinator(AtomicCombinator::Sub), args)
}

/// Create a multiplication expression
pub fn make_multiply(left: Expr, right: Expr) -> Expr {
    let args = vec![left, right];
    make_application(Expr::Combinator(AtomicCombinator::Mul), args)
}

/// Create a division expression
pub fn make_divide(left: Expr, right: Expr) -> Expr {
    let args = vec![left, right];
    make_application(Expr::Combinator(AtomicCombinator::Div), args)
}

/// Create an equality comparison expression
pub fn make_eq(left: Expr, right: Expr) -> Expr {
    let args = vec![left, right];
    make_application(Expr::Combinator(AtomicCombinator::Eq), args)
}

/// Create a greater than comparison expression
pub fn make_gt(left: Expr, right: Expr) -> Expr {
    let args = vec![left, right];
    make_application(Expr::Combinator(AtomicCombinator::Gt), args)
}

/// Create a greater than or equal comparison expression
pub fn make_gte(left: Expr, right: Expr) -> Expr {
    let args = vec![left, right];
    make_application(Expr::Combinator(AtomicCombinator::Gte), args)
}

/// Create a less than comparison expression
pub fn make_lt(left: Expr, right: Expr) -> Expr {
    let args = vec![left, right];
    make_application(Expr::Combinator(AtomicCombinator::Lt), args)
}

/// Create a less than or equal comparison expression
pub fn make_lte(left: Expr, right: Expr) -> Expr {
    let args = vec![left, right];
    make_application(Expr::Combinator(AtomicCombinator::Lte), args)
}

/// Create an and expression
pub fn make_and(exprs: Vec<Expr>) -> Expr {
    make_application(Expr::Combinator(AtomicCombinator::And), exprs)
}

/// Create an or expression
pub fn make_or(exprs: Vec<Expr>) -> Expr {
    make_application(Expr::Combinator(AtomicCombinator::Or), exprs)
}

/// Create a not expression
pub fn make_not(expr: Expr) -> Expr {
    make_application(Expr::Combinator(AtomicCombinator::Not), vec![expr])
}
