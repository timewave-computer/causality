//! Parser for Causality Lisp
//!
//! This module provides parsing functionality for Causality Lisp expressions,
//! handling all 11 Layer 1 primitives and integration with the AST.

use crate::{
    ast::{Expr, ExprKind, LispValue, Param},
    error::{ParseError},
};
use causality_core::{
    lambda::Symbol,
    system::content_addressing::Str,
};

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Token types for the lexer
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftParen,
    RightParen,
    Symbol(Symbol),
    Number(i64),
    Float(f64),
    String(Str),
    Bool(bool),
    EOF,
}

/// Lexer for tokenizing Lisp input
pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> ParseResult<Vec<Token>> {
        let mut tokens = Vec::new();
        
        while self.position < self.input.len() {
            self.skip_whitespace();
            
            if self.position >= self.input.len() {
                break;
            }
            
            match self.current_char()? {
                '(' => {
                    tokens.push(Token::LeftParen);
                    self.advance();
                }
                ')' => {
                    tokens.push(Token::RightParen);
                    self.advance();
                }
                '"' => {
                    tokens.push(self.read_string()?);
                }
                ch if ch.is_ascii_digit() || ch == '-' => {
                    tokens.push(self.read_number()?);
                }
                '#' => {
                    tokens.push(self.read_boolean()?);
                }
                ch if ch.is_alphabetic() || ch == '+' || ch == '*' || ch == '/' || ch == '=' || ch == '<' || ch == '>' => {
                    tokens.push(self.read_symbol()?);
                }
                _ => {
                    return Err(ParseError::UnexpectedChar(
                        self.current_char()?,
                        self.line,
                        self.column,
                    ));
                }
            }
        }
        
        tokens.push(Token::EOF);
        Ok(tokens)
    }
    
    fn current_char(&self) -> ParseResult<char> {
        self.input
            .chars()
            .nth(self.position)
            .ok_or(ParseError::UnexpectedEof)
    }
    
    fn advance(&mut self) {
        if self.position < self.input.len() {
            if self.current_char().unwrap_or('\0') == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }
    
    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() {
            match self.current_char() {
                Ok(' ') | Ok('\t') | Ok('\n') | Ok('\r') => {
                    self.advance();
                }
                _ => break,
            }
        }
    }
    
    fn read_string(&mut self) -> ParseResult<Token> {
        self.advance(); // Skip opening quote
        let mut value = String::new();
        
        while self.position < self.input.len() {
            match self.current_char()? {
                '"' => {
                    self.advance(); // Skip closing quote
                    return Ok(Token::String(Str::new(&value)));
                }
                '\\' => {
                    self.advance();
                    match self.current_char()? {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '"' => value.push('"'),
                        ch => {
                            return Err(ParseError::InvalidEscape(ch, self.line, self.column));
                        }
                    }
                    self.advance();
                }
                ch => {
                    value.push(ch);
                    self.advance();
                }
            }
        }
        
        Err(ParseError::UnclosedString(self.line, self.column))
    }
    
    fn read_number(&mut self) -> ParseResult<Token> {
        let mut value = String::new();
        let mut is_float = false;
        
        // Handle negative numbers
        if self.current_char()? == '-' {
            value.push('-');
            self.advance();
        }
        
        while self.position < self.input.len() {
            match self.current_char() {
                Ok(ch) if ch.is_ascii_digit() => {
                    value.push(ch);
                    self.advance();
                }
                Ok('.') if !is_float => {
                    is_float = true;
                    value.push('.');
                    self.advance();
                }
                _ => break,
            }
        }
        
        if is_float {
            let float_val = value.parse::<f64>().map_err(|_| {
                ParseError::InvalidNumber(value.clone(), self.line, self.column)
            })?;
            Ok(Token::Float(float_val))
        } else {
            let int_val = value.parse::<i64>().map_err(|_| {
                ParseError::InvalidNumber(value.clone(), self.line, self.column)
            })?;
            Ok(Token::Number(int_val))
        }
    }
    
    fn read_boolean(&mut self) -> ParseResult<Token> {
        self.advance(); // Skip '#'
        
        match self.current_char()? {
            't' => {
                self.advance();
                Ok(Token::Bool(true))
            }
            'f' => {
                self.advance();
                Ok(Token::Bool(false))
            }
            ch => Err(ParseError::UnexpectedChar(ch, self.line, self.column)),
        }
    }
    
    fn read_symbol(&mut self) -> ParseResult<Token> {
        let mut value = String::new();
        
        while self.position < self.input.len() {
            match self.current_char() {
                Ok(ch) if ch.is_alphanumeric() || ch == '-' || ch == '_' || ch == '+' || ch == '*' || ch == '/' || ch == '=' || ch == '<' || ch == '>' => {
                    value.push(ch);
                    self.advance();
                }
                _ => break,
            }
        }
        
        // Check for boolean literals
        match value.as_str() {
            "true" => Ok(Token::Bool(true)),
            "false" => Ok(Token::Bool(false)),
            _ => Ok(Token::Symbol(Symbol::new(&value))),
        }
    }
}

/// Parser for Causality Lisp expressions
pub struct LispParser {
    tokens: Vec<Token>,
    position: usize,
}

impl LispParser {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            position: 0,
        }
    }
    
    /// Parse a Lisp expression from text
    pub fn parse(&mut self, input: &str) -> ParseResult<Expr> {
        let mut lexer = Lexer::new(input.to_string());
        self.tokens = lexer.tokenize()?;
        self.position = 0;
        self.parse_expression()
    }
    
    fn current_token(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::EOF)
    }
    
    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }
    
    fn parse_expression(&mut self) -> ParseResult<Expr> {
        match self.current_token() {
            Token::LeftParen => self.parse_list_or_special_form(),
            Token::Number(n) => {
                let value = *n;
                self.advance();
                Ok(Expr::constant(LispValue::Int(value)))
            }
            Token::Float(f) => {
                let value = *f;
                self.advance();
                Ok(Expr::constant(LispValue::Float(value)))
            }
            Token::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(Expr::constant(LispValue::String(value)))
            }
            Token::Bool(b) => {
                let value = *b;
                self.advance();
                Ok(Expr::constant(LispValue::Bool(value)))
            }
            Token::Symbol(sym) => {
                let symbol = sym.clone();
                self.advance();
                Ok(Expr::variable(symbol))
            }
            _ => Err(ParseError::UnexpectedEof),
        }
    }
    
    fn parse_list_or_special_form(&mut self) -> ParseResult<Expr> {
        self.advance(); // Skip '('
        
        // Check if first token is a symbol and get its name
        let symbol_name = if let Token::Symbol(name) = self.current_token() {
            Some(name.to_string())
        } else {
            None
        };
        
        if let Some(name) = symbol_name {
            let first = self.parse_expression()?;
            self.parse_special_form_or_call(name, first)
        } else {
            // Parse as regular list
            let mut elements = Vec::new();
            while !matches!(self.current_token(), Token::RightParen | Token::EOF) {
                elements.push(self.parse_expression()?);
            }
            
            if matches!(self.current_token(), Token::RightParen) {
                self.advance(); // Skip ')'
            } else {
                return Err(ParseError::UnclosedParen(0, 0)); // TODO: proper line/column
            }
            
            Ok(Expr::list(elements))
        }
    }
    
    fn parse_special_form_or_call(&mut self, name: String, first: Expr) -> ParseResult<Expr> {
        match name.as_str() {
            "lambda" => self.parse_lambda(),
            "let-tensor" => self.parse_let_tensor(),
            "case" => self.parse_case(),
            "tensor" => self.parse_tensor(),
            "inl" => self.parse_inl(),
            "inr" => self.parse_inr(),
            "alloc" => self.parse_alloc(),
            "consume" => self.parse_consume(),
            "unit" => self.parse_unit(),
            "let-unit" => self.parse_let_unit(),
            _ => self.parse_function_call(first),
        }
    }
    
    fn parse_lambda(&mut self) -> ParseResult<Expr> {
        // Parse parameter list
        self.expect_left_paren()?;
        let mut params = Vec::new();
        while !matches!(self.current_token(), Token::RightParen) {
            let param_name = self.expect_symbol()?;
            params.push(Param::new(Symbol::new(&param_name)));
        }
        self.expect_right_paren()?;
        
        // Parse body
        let body = self.parse_expression()?;
        self.expect_right_paren()?;
        
        Ok(Expr::lambda(params, body))
    }
    
    fn parse_let_unit(&mut self) -> ParseResult<Expr> {
        let unit_expr = self.parse_expression()?;
        let body = self.parse_expression()?;
        self.expect_right_paren()?;
        
        Ok(Expr::let_unit(unit_expr, body))
    }
    
    fn parse_let_tensor(&mut self) -> ParseResult<Expr> {
        let tensor_expr = self.parse_expression()?;
        let left_var = self.expect_symbol()?;
        let right_var = self.expect_symbol()?;
        let body = self.parse_expression()?;
        self.expect_right_paren()?;
        
        Ok(Expr::new(ExprKind::LetTensor(
            Box::new(tensor_expr),
            Symbol::new(&left_var),
            Symbol::new(&right_var),
            Box::new(body),
        )))
    }
    
    fn parse_case(&mut self) -> ParseResult<Expr> {
        let sum_expr = self.parse_expression()?;
        let left_var = self.expect_symbol()?;
        let left_branch = self.parse_expression()?;
        let right_var = self.expect_symbol()?;
        let right_branch = self.parse_expression()?;
        self.expect_right_paren()?;
        
        Ok(Expr::case(
            sum_expr,
            Symbol::new(&left_var),
            left_branch,
            Symbol::new(&right_var),
            right_branch,
        ))
    }
    
    fn parse_tensor(&mut self) -> ParseResult<Expr> {
        let left = self.parse_expression()?;
        let right = self.parse_expression()?;
        self.expect_right_paren()?;
        
        Ok(Expr::tensor(left, right))
    }
    
    fn parse_inl(&mut self) -> ParseResult<Expr> {
        let value = self.parse_expression()?;
        self.expect_right_paren()?;
        
        Ok(Expr::new(ExprKind::Inl(Box::new(value))))
    }
    
    fn parse_inr(&mut self) -> ParseResult<Expr> {
        let value = self.parse_expression()?;
        self.expect_right_paren()?;
        
        Ok(Expr::new(ExprKind::Inr(Box::new(value))))
    }
    
    fn parse_alloc(&mut self) -> ParseResult<Expr> {
        let value = self.parse_expression()?;
        self.expect_right_paren()?;
        
        Ok(Expr::new(ExprKind::Alloc(Box::new(value))))
    }
    
    fn parse_consume(&mut self) -> ParseResult<Expr> {
        let resource = self.parse_expression()?;
        self.expect_right_paren()?;
        
        Ok(Expr::new(ExprKind::Consume(Box::new(resource))))
    }
    
    fn parse_unit(&mut self) -> ParseResult<Expr> {
        self.expect_right_paren()?;
        
        Ok(Expr::new(ExprKind::UnitVal))
    }
    
    fn parse_function_call(&mut self, func: Expr) -> ParseResult<Expr> {
        let mut args = Vec::new();
        
        while !matches!(self.current_token(), Token::RightParen) {
            args.push(self.parse_expression()?);
        }
        self.expect_right_paren()?;
        
        Ok(Expr::apply(func, args))
    }
    
    fn expect_symbol(&mut self) -> ParseResult<String> {
        match self.current_token() {
            Token::Symbol(sym) => {
                let name = sym.to_string();
                self.advance();
                Ok(name)
            }
            _ => Err(ParseError::InvalidSyntax("Expected symbol".to_string())),
        }
    }
    
    fn expect_left_paren(&mut self) -> ParseResult<()> {
        match self.current_token() {
            Token::LeftParen => {
                self.advance();
                Ok(())
            }
            _ => Err(ParseError::InvalidSyntax("Expected '('".to_string())),
        }
    }
    
    fn expect_right_paren(&mut self) -> ParseResult<()> {
        match self.current_token() {
            Token::RightParen => {
                self.advance();
                Ok(())
            }
            _ => Err(ParseError::InvalidSyntax("Expected ')'".to_string())),
        }
    }
}

impl Default for LispParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_number() {
        let mut parser = LispParser::new();
        let expr = parser.parse("42").unwrap();
        
        match expr.kind {
            ExprKind::Const(LispValue::Int(42)) => {}
            _ => panic!("Expected integer constant"),
        }
    }
    
    #[test]
    fn test_parse_symbol() {
        let mut parser = LispParser::new();
        let expr = parser.parse("foo").unwrap();
        
        match expr.kind {
            ExprKind::Var(_) => {}
            _ => panic!("Expected variable"),
        }
    }
    
    #[test]
    fn test_parse_list() {
        let mut parser = LispParser::new();
        let expr = parser.parse("(+ 1 2)").unwrap();
        
        match expr.kind {
            ExprKind::Apply(_, args) => {
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected function application"),
        }
    }
} 