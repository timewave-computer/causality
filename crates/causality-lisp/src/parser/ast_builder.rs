//! AST Builder for Causality Lisp Parser
//!
//! Converts token streams into Expr AST nodes, implementing the transformation
//! from lexical tokens to a structured Abstract Syntax Tree.

//-----------------------------------------------------------------------------
// AST Builder
//-----------------------------------------------------------------------------

#[cfg(not(feature = "std"))]
use crate::compatibility::{Iter, Peekable};
#[cfg(feature = "std")]
use std::iter::Peekable;
#[cfg(feature = "std")]
use std::slice::Iter;

use causality_types::primitive::string::Str;
use causality_types::expr::ast::{Atom, Expr};
use causality_core::lisp_adapter::ExprExt;
use causality_types::expr::ast::{AtomicCombinator, ExprBox};

use crate::parser::error::{ParseError, ParseResult};
use crate::parser::lexer::{Token, TokenWithLocation};
use crate::parser::utils;

//-----------------------------------------------------------------------------
// Parser Implementation
//-----------------------------------------------------------------------------

/// AST Builder that converts tokens into Expr AST nodes
pub struct AstBuilder<'a> {
    /// Token stream
    tokens: Peekable<Iter<'a, TokenWithLocation>>,
}

impl<'a> AstBuilder<'a> {
    /// Create a new AST builder
    pub fn new(tokens: &'a [TokenWithLocation]) -> Self {
        Self {
            tokens: tokens.iter().peekable(),
        }
    }

    /// Peek at the next token without consuming it
    fn peek(&mut self) -> Option<&'a TokenWithLocation> {
        self.tokens.peek().copied()
    }

    /// Get the next token
    fn next(&mut self) -> Option<&'a TokenWithLocation> {
        self.tokens.next()
    }

    /// Expect a specific token type
    fn expect(&mut self, expected: Token) -> ParseResult<&'a TokenWithLocation> {
        if let Some(token_loc) = self.next() {
            if token_loc.token == expected {
                Ok(token_loc)
            } else {
                Err(ParseError::syntax_error(
                    format!("Expected {:?}, got {:?}", expected, token_loc.token),
                    token_loc.location.line,
                    token_loc.location.column,
                ))
            }
        } else {
            let last = self.tokens.clone().last();
            let (line, column) = last
                .map(|t| (t.location.line, t.location.column))
                .unwrap_or((1, 1));

            Err(ParseError::unexpected_eof(
                format!("{:?}", expected),
                line,
                column,
            ))
        }
    }

    /// Parse an atom (literal or symbol)
    fn parse_atom(&mut self, token_loc: &'a TokenWithLocation) -> ParseResult<Expr> {
        // Create an expression based on the token type
        match &token_loc.token {
            Token::Symbol(s) => Ok(Expr::Var(Str::from(s.as_str()))),
            Token::String(s) => Ok(Expr::string(s.clone())),
            Token::Integer(n) => Ok(Expr::integer(*n)),
            Token::Boolean(b) => Ok(Expr::boolean(*b)),
            Token::Nil => Ok(Expr::Atom(Atom::Nil)),
            _ => Err(ParseError::syntax_error(
                format!("Expected atom, got {:?}", token_loc.token),
                token_loc.location.line,
                token_loc.location.column,
            )),
        }
    }

    /// Parse a quoted expression
    fn parse_quoted(&mut self) -> ParseResult<Expr> {
        // Skip the quote token
        let quote_loc = self.next().unwrap();

        if self.peek().is_some() {
            let expr = self.parse_expr()?;

            // Create a quoted expression (represented as (quote expr))
            let quote_sym = Expr::Var(Str::from("quote"));
            let args = vec![expr];

            Ok(utils::make_application(quote_sym, args))
        } else {
            Err(ParseError::unexpected_eof(
                "expression after quote",
                quote_loc.location.line,
                quote_loc.location.column,
            ))
        }
    }

    /// Parse a list expression
    fn parse_list(&mut self) -> ParseResult<Expr> {
        // Skip the opening paren
        let open_loc = self.next().unwrap();

        // Check for empty list
        if let Some(next) = self.peek() {
            if next.token == Token::RParen {
                self.next(); // consume closing paren
                return Ok(utils::make_nil()); // Empty list is represented as nil
            }
        } else {
            return Err(ParseError::unexpected_eof(
                "closing parenthesis",
                open_loc.location.line,
                open_loc.location.column,
            ));
        }

        // Parse the first element to detect special forms
        let first = self.parse_expr()?;

        // Check for special forms
        if let Some(sym) = utils::extract_symbol(&first) {
            match sym.as_str() {
                "if" => return self.parse_if(open_loc),
                "let" => return self.parse_let(open_loc),
                "and" => return self.parse_and(open_loc),
                "or" => return self.parse_or(open_loc),
                "not" => return self.parse_not(open_loc),
                "fn" => return self.parse_lambda(open_loc),
                // Binary operations using combinators
                "+" => return self.parse_binary_op(utils::make_add, open_loc),
                "-" => {
                    return self.parse_binary_op(utils::make_subtract, open_loc);
                }
                "*" => {
                    return self.parse_binary_op(utils::make_multiply, open_loc);
                }
                "/" => {
                    return self.parse_binary_op(utils::make_divide, open_loc);
                }
                "=" | "eq" => {
                    return self.parse_binary_op(utils::make_eq, open_loc);
                }
                ">" | "gt" => {
                    return self.parse_binary_op(utils::make_gt, open_loc);
                }
                ">=" | "gte" => {
                    return self.parse_binary_op(utils::make_gte, open_loc);
                }
                "<" | "lt" => {
                    return self.parse_binary_op(utils::make_lt, open_loc);
                }
                "<=" | "lte" => {
                    return self.parse_binary_op(utils::make_lte, open_loc);
                }
                _ => {}
            }
        }

        // This is a regular function application
        let mut args = vec![first];

        // Parse remaining expressions
        while let Some(next) = self.peek() {
            if next.token == Token::RParen {
                self.next(); // Consume closing paren

                // Check if it's a function call
                if args.is_empty() {
                    return Ok(utils::make_nil()); // Empty list is nil
                }

                let func = args.remove(0);
                return Ok(utils::make_application(func, args));
            } else {
                args.push(self.parse_expr()?);
            }
        }

        // If we get here, we're missing a closing paren
        Err(ParseError::unexpected_eof(
            "closing parenthesis",
            open_loc.location.line,
            open_loc.location.column,
        ))
    }

    /// Helper for parsing if expressions
    fn parse_if(&mut self, _open_loc: &TokenWithLocation) -> ParseResult<Expr> {
        // We've already consumed (if
        let condition = self.parse_expr()?;
        let then_expr = self.parse_expr()?;
        let else_expr = self.parse_expr()?;

        // Expect closing paren
        self.expect(Token::RParen)?;

        Ok(utils::make_if(condition, then_expr, else_expr))
    }

    /// Helper for parsing let expressions
    fn parse_let(&mut self, open_loc: &TokenWithLocation) -> ParseResult<Expr> {
        // We've already consumed (let
        // Using the Let combinator for our implementation

        // Expect opening paren for bindings
        self.expect(Token::LParen)?;

        // Parse bindings
        let mut binding_exprs = Vec::new();

        // Keep parsing pairs until we hit the closing paren
        while let Some(token) = self.peek() {
            if token.token == Token::RParen {
                self.next(); // Consume closing paren
                break;
            }

            // Each binding is a pair: (name value)
            self.expect(Token::LParen)?;

            // Parse the name
            if let Some(token_loc) = self.next() {
                if let Token::Symbol(name) = &token_loc.token {
                    let name_expr = Expr::Var(Str::from(name.as_str()));
                    binding_exprs.push(name_expr);

                    // Parse the value
                    let value = self.parse_expr()?;
                    binding_exprs.push(value);

                    // Expect closing paren for this binding
                    self.expect(Token::RParen)?;
                } else {
                    return Err(ParseError::syntax_error(
                        format!(
                            "Expected variable name in let binding, got {:?}",
                            token_loc.token
                        ),
                        token_loc.location.line,
                        token_loc.location.column,
                    ));
                }
            } else {
                return Err(ParseError::unexpected_eof(
                    "variable name in let binding",
                    open_loc.location.line,
                    open_loc.location.column,
                ));
            }
        }

        // Parse body expression
        let body = self.parse_expr()?;

        // Expect closing paren for whole let expression
        self.expect(Token::RParen)?;

        // Create a let expression using the Let combinator
        // Format: (Let [bindings...] body)
        let binding_list = utils::make_list(binding_exprs);
        let let_args = vec![binding_list, body];
        Ok(utils::make_application(
            Expr::Combinator(AtomicCombinator::Let),
            let_args,
        ))
    }

    /// Helper for parsing and expressions
    fn parse_and(&mut self, open_loc: &TokenWithLocation) -> ParseResult<Expr> {
        // We've already consumed (and
        let mut args = Vec::new();

        // Parse arguments
        while let Some(next) = self.peek() {
            if next.token == Token::RParen {
                self.next(); // Consume closing paren
                return Ok(utils::make_and(args));
            } else {
                args.push(self.parse_expr()?);
            }
        }

        // If we get here, we ran out of tokens
        Err(ParseError::unexpected_eof(
            "closing parenthesis",
            open_loc.location.line,
            open_loc.location.column,
        ))
    }

    /// Helper for parsing or expressions
    fn parse_or(&mut self, open_loc: &TokenWithLocation) -> ParseResult<Expr> {
        // We've already consumed (or
        let mut args = Vec::new();

        // Parse arguments
        while let Some(next) = self.peek() {
            if next.token == Token::RParen {
                self.next(); // Consume closing paren
                return Ok(utils::make_or(args));
            } else {
                args.push(self.parse_expr()?);
            }
        }

        // If we get here, we ran out of tokens
        Err(ParseError::unexpected_eof(
            "closing parenthesis",
            open_loc.location.line,
            open_loc.location.column,
        ))
    }

    /// Helper for parsing not expressions
    fn parse_not(&mut self, _open_loc: &TokenWithLocation) -> ParseResult<Expr> {
        // We've already consumed (not
        let arg = self.parse_expr()?;

        // Expect closing paren
        self.expect(Token::RParen)?;

        Ok(utils::make_not(arg))
    }

    /// Helper for parsing binary operations
    fn parse_binary_op(
        &mut self,
        constructor: fn(Expr, Expr) -> Expr,
        _open_loc: &TokenWithLocation,
    ) -> ParseResult<Expr> {
        // Parse the left and right operands
        let left = self.parse_expr()?;
        let right = self.parse_expr()?;

        // Consume the closing paren
        self.expect(Token::RParen)?;

        // Create the binary operation expression using the constructor function
        Ok(constructor(left, right))
    }

    /// Helper for parsing lambda expressions (fn)
    fn parse_lambda(&mut self, _open_loc: &TokenWithLocation) -> ParseResult<Expr> {
        // Parse parameter list
        self.expect(Token::LParen)?;

        let mut params = Vec::new();

        // Parse all parameters until closing paren
        while let Some(token) = self.peek() {
            if token.token == Token::RParen {
                self.next(); // consume closing paren
                break;
            }

            // Get parameter name
            if let Some(token_loc) = self.next() {
                match &token_loc.token {
                    Token::Symbol(name) => {
                        params.push(name.clone());
                    }
                    _ => {
                        return Err(ParseError::syntax_error(
                            format!(
                                "Expected parameter name, got {:?}",
                                token_loc.token
                            ),
                            token_loc.location.line,
                            token_loc.location.column,
                        ));
                    }
                }
            } else {
                return Err(ParseError::unexpected_eof(
                    "parameter list",
                    _open_loc.location.line,
                    _open_loc.location.column,
                ));
            }
        }

        // Convert param names to Str
        let param_strs = params.into_iter().map(Str::from).collect::<Vec<_>>();

        // Parse body
        let body = self.parse_expr()?;

        // Expect closing paren for lambda
        self.expect(Token::RParen)?;

        // In our combinator-based approach, we use the Lambda constructor directly
        Ok(Expr::Lambda(param_strs, ExprBox(Box::new(body))))
    }

    /// Parse an expression
    pub fn parse_expr(&mut self) -> ParseResult<Expr> {
        if let Some(token_loc) = self.peek() {
            match token_loc.token {
                Token::LParen => self.parse_list(),
                Token::Quote => self.parse_quoted(),
                _ => {
                    let token = self.next().unwrap();
                    self.parse_atom(token)
                }
            }
        } else {
            let last = self.tokens.clone().last();
            let (line, column) = last
                .map(|t| (t.location.line, t.location.column))
                .unwrap_or((1, 1));

            Err(ParseError::unexpected_eof("expression", line, column))
        }
    }

    /// Parse the entire program
    pub fn parse_program(&mut self) -> ParseResult<Vec<Expr>> {
        let mut exprs = Vec::new();

        while self.peek().is_some() {
            exprs.push(self.parse_expr()?);
        }

        Ok(exprs)
    }
}

//-----------------------------------------------------------------------------
// Test
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::{
        expr::value::{ValueExpr, Number},
        expr::ast::{Expr, Atom, AtomicCombinator},
        core::str::Str,
    };
    use crate::parser::lexer::tokenize;
    use crate::parser::utils::parse_expr;

    #[test]
    fn test_parse_atom() {
        let tokens = tokenize("42").unwrap();
        let expr = parse_expr(&tokens).unwrap();
        assert!(matches!(expr, Expr::Const(ValueExpr::Number(Number::Integer(_)))));

        let tokens = tokenize("\"hello\"").unwrap();
        let expr = parse_expr(&tokens).unwrap();
        assert!(matches!(expr, Expr::Const(ValueExpr::String(_))));

        let tokens = tokenize("true").unwrap();
        let expr = parse_expr(&tokens).unwrap();
        assert!(matches!(expr, Expr::Const(ValueExpr::Bool(true))));

        let tokens = tokenize("nil").unwrap();
        let expr = parse_expr(&tokens).unwrap();
        assert!(matches!(expr, Expr::Atom(Atom::Nil)));

        let tokens = tokenize("x").unwrap();
        let expr = parse_expr(&tokens).unwrap();
        assert!(matches!(expr, Expr::Var(_)));
    }

    #[test]
    fn test_parse_list() {
        let tokens = tokenize("(+ 1 2)").unwrap();
        let expr = parse_expr(&tokens).unwrap();

        match &expr {
            Expr::Apply(func, args) => {
                // Check if function is the Add combinator
                if let Expr::Combinator(AtomicCombinator::Add) = *func.0 {
                    // Good
                } else {
                    panic!("Expected Add combinator, got {:?}", func.0);
                }

                // Check arguments
                assert_eq!(args.len(), 2);

                // Check first arg is 1
                if let Expr::Const(ValueExpr::Number(Number::Integer(n))) = args[0] {
                    assert_eq!(n, 1);
                } else {
                    panic!("Expected Integer(1), got {:?}", args[0]);
                }

                // Check second arg is 2
                if let Expr::Const(ValueExpr::Number(Number::Integer(n))) = args[1] {
                    assert_eq!(n, 2);
                } else {
                    panic!("Expected Integer(2), got {:?}", args[1]);
                }
            }
            _ => panic!("Expected Apply, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_if() {
        let tokens = tokenize("(if true 1 2)").unwrap();
        let expr = parse_expr(&tokens).unwrap();

        match &expr {
            Expr::Apply(func, args) => {
                // Check if function is the If combinator
                if let Expr::Combinator(AtomicCombinator::If) = *func.0 {
                    // Good
                } else {
                    panic!("Expected If combinator, got {:?}", func.0);
                }

                // Check arguments
                assert_eq!(args.len(), 3);

                // Check condition is true
                if let Expr::Const(ValueExpr::Bool(b)) = args[0] {
                    assert!(b);
                } else {
                    panic!("Expected Boolean(true), got {:?}", args[0]);
                }

                // Check then_expr is 1
                if let Expr::Const(ValueExpr::Number(Number::Integer(n))) = args[1] {
                    assert_eq!(n, 1);
                } else {
                    panic!("Expected Integer(1), got {:?}", args[1]);
                }

                // Check else_expr is 2
                if let Expr::Const(ValueExpr::Number(Number::Integer(n))) = args[2] {
                    assert_eq!(n, 2);
                } else {
                    panic!("Expected Integer(2), got {:?}", args[2]);
                }
            }
            _ => panic!("Expected Apply, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_and_or() {
        let tokens = tokenize("(and true false)").unwrap();
        let expr = parse_expr(&tokens).unwrap();

        match &expr {
            Expr::Apply(func, args) => {
                // Check if function is the And combinator
                if let Expr::Combinator(AtomicCombinator::And) = *func.0 {
                    // Good
                } else {
                    panic!("Expected And combinator, got {:?}", func.0);
                }

                // Check arguments
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Apply, got {:?}", expr),
        }

        let tokens = tokenize("(or true false)").unwrap();
        let expr = parse_expr(&tokens).unwrap();

        match &expr {
            Expr::Apply(func, args) => {
                // Check if function is the Or combinator
                if let Expr::Combinator(AtomicCombinator::Or) = *func.0 {
                    // Good
                } else {
                    panic!("Expected Or combinator, got {:?}", func.0);
                }

                // Check arguments
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Apply, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_not() {
        let tokens = tokenize("(not true)").unwrap();
        let expr = parse_expr(&tokens).unwrap();

        match &expr {
            Expr::Apply(func, args) => {
                // Check if function is the Not combinator
                if let Expr::Combinator(AtomicCombinator::Not) = *func.0 {
                    // Good
                } else {
                    panic!("Expected Not combinator, got {:?}", func.0);
                }

                // Check arguments
                assert_eq!(args.len(), 1);

                // Check arg is true
                if let Expr::Const(ValueExpr::Bool(b)) = args[0] {
                    assert!(b);
                } else {
                    panic!("Expected Boolean(true), got {:?}", args[0]);
                }
            }
            _ => panic!("Expected Apply, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_lambda() {
        let input = "(fn (x y) (+ x y))";
        let tokens = tokenize(input).unwrap();
        let expr = parse_expr(&tokens).unwrap();

        // Expect the result to be a Lambda expression
        if let Expr::Lambda(params, body) = expr {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].to_string(), "x");
            assert_eq!(params[1].to_string(), "y");

            // Check body is an application of + to x and y
            if let Expr::Apply(func, args) = &*body.0 {
                if let Expr::Combinator(AtomicCombinator::Add) = &*func.0 {
                    assert_eq!(args.len(), 2);

                    // Check first arg is x
                    if let Expr::Var(name) = &args[0] {
                        assert_eq!(name.to_string(), "x");
                    } else {
                        panic!("Expected Var(x), got {:?}", args[0]);
                    }

                    // Check second arg is y
                    if let Expr::Var(name) = &args[1] {
                        assert_eq!(name.to_string(), "y");
                    } else {
                        panic!("Expected Var(y), got {:?}", args[1]);
                    }
                } else {
                    panic!("Expected Add combinator, got {:?}", func.0);
                }
            } else {
                panic!("Expected Apply, got {:?}", body.0);
            }
        } else {
            panic!("Expected Lambda, got {:?}", expr);
        }
    }
}
