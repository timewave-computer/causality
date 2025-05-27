//!
//! # Lisp S-Expression Parser
//!
//! This module is responsible for parsing text-based Lisp S-expressions into the
//! `causality_types::expr::ast::Expr` Abstract Syntax Tree (AST) representation used by
//! the Causality Lisp interpreter and other tools.
//!
//! ## Process:
//! 1.  **Lexical Analysis (`lexer`)**: The input string is first tokenized into a sequence
//!     of fundamental Lisp elements like parentheses, symbols, literals (strings, numbers, booleans),
//!     and comments.
//! 2.  **Syntactic Analysis (`utils`, `ast_builder`)**: The token stream is then parsed to
//!     construct the hierarchical `Expr` AST. This involves recognizing list structures,
//!     atomic values, and ensuring balanced parentheses.
//!
//! ## Key Components:
//! - `lexer`: Contains the tokenizer logic.
//! - `error`: Defines parsing error types (`ParseError`, `ParseResult`).
//! - `ast_builder`: Helper functions to construct `Expr` variants from tokens (indirectly used via `utils`).
//! - `utils`: Core parsing functions like `parse_expr` and `parse_program` that consume tokens.
//!
//! ## Public API:
//! - `parse(input: &str) -> ParseResult<Expr>`: Parses a single S-expression.
//! - `parse_program_str(input: &str) -> ParseResult<Vec<Expr>>`: Parses a sequence of S-expressions.
//! - `parse_first(input: &str) -> ParseResult<Option<Expr>>`: Parses the first S-expression if present.
//!
//! The parser aims to be robust and provide informative error messages upon encountering
//! malformed input.

//-----------------------------------------------------------------------------
// Parser Module
//-----------------------------------------------------------------------------

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub mod ast_builder;
pub mod error;
pub mod lexer;
pub mod utils;

use causality_types::expr::ast::Expr;

use self::error::ParseResult;
use self::lexer::tokenize;
use self::utils::{parse_expr, parse_program};

//-----------------------------------------------------------------------------
// Public API
//-----------------------------------------------------------------------------

/// Parse a single Lisp expression from a string
pub fn parse(input: &str) -> ParseResult<Expr> {
    let tokens = tokenize(input)?;
    parse_expr(&tokens)
}

/// Parse a Lisp program (multiple expressions) from a string
pub fn parse_program_str(input: &str) -> ParseResult<Vec<Expr>> {
    let tokens = tokenize(input)?;
    parse_program(&tokens)
}

/// Parse the first expression from a string
pub fn parse_first(input: &str) -> ParseResult<Option<Expr>> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        Ok(None)
    } else {
        Ok(Some(parse_expr(&tokens)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::expr::ast::AtomicCombinator;

    #[test]
    fn test_parse_simple() {
        let input = "(+ 1 2)";
        let expr = parse(input).unwrap();

        // In our combinator architecture, this should be an Apply with Add combinator
        match expr {
            Expr::Apply(func, _) => match *func.0 {
                Expr::Combinator(AtomicCombinator::Add) => (),
                _ => panic!("Expected Add combinator, got {:?}", func.0),
            },
            _ => panic!("Expected Apply expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_nested() {
        let input = "(if (> x 0) (+ x 1) (- x 1))";
        let expr = parse(input).unwrap();

        // In our combinator architecture, this should be an Apply with If combinator
        match expr {
            Expr::Apply(func, _) => match *func.0 {
                Expr::Combinator(AtomicCombinator::If) => (),
                _ => panic!("Expected If combinator, got {:?}", func.0),
            },
            _ => panic!("Expected Apply expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_program() {
        let input = "
        (let ((x 10)) x)
        (if (> x 0)
            (+ x 1)
            (- x 1))
        ";

        let program = parse_program_str(input).unwrap();
        assert_eq!(program.len(), 2);

        // Verify the first expression is a Let application
        match &program[0] {
            Expr::Apply(func, _) => match *func.0 {
                Expr::Combinator(AtomicCombinator::Let) => (),
                _ => panic!("Expected Let combinator, got {:?}", func.0),
            },
            _ => panic!("Expected Apply expression for Let, got {:?}", program[0]),
        }

        // Verify the second expression is an If application
        match &program[1] {
            Expr::Apply(func, _) => match *func.0 {
                Expr::Combinator(AtomicCombinator::If) => (),
                _ => panic!("Expected If combinator, got {:?}", func.0),
            },
            _ => {
                panic!("Expected Apply expression for If, got {:?}", program[1])
            }
        }
    }

    #[test]
    fn test_parse_error() {
        let input = "(+ 1 2"; // Missing closing paren
        let result = parse(input);
        assert!(result.is_err());
    }
}
