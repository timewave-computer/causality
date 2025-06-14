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

/// Token types for the lexer with position information
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

/// Token with position information for better error reporting
#[derive(Debug, Clone)]
pub struct PositionedToken {
    pub token: Token,
    pub line: usize,
    pub column: usize,
    pub start_pos: usize,
    pub end_pos: usize,
}

impl PositionedToken {
    pub fn new(token: Token, line: usize, column: usize, start_pos: usize, end_pos: usize) -> Self {
        Self { token, line, column, start_pos, end_pos }
    }
    
    /// Format token for error messages
    pub fn format_for_error(&self) -> String {
        match &self.token {
            Token::LeftParen => "'('".to_string(),
            Token::RightParen => "')'".to_string(),
            Token::Symbol(s) => format!("symbol '{}'", s),
            Token::Number(n) => format!("number {}", n),
            Token::Float(f) => format!("float {}", f),
            Token::String(s) => format!("string \"{}\"", s),
            Token::Bool(b) => format!("boolean {}", b),
            Token::EOF => "end of input".to_string(),
        }
    }
}

/// Lexer for tokenizing Lisp input with enhanced position tracking
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
    
    pub fn tokenize(&mut self) -> ParseResult<Vec<PositionedToken>> {
        let mut tokens = Vec::new();
        
        while self.position < self.input.len() {
            self.skip_whitespace();
            
            if self.position >= self.input.len() {
                break;
            }
            
            let start_line = self.line;
            let start_column = self.column;
            let start_pos = self.position;
            
            match self.current_char()? {
                '(' => {
                    self.advance();
                    tokens.push(PositionedToken::new(
                        Token::LeftParen, start_line, start_column, start_pos, self.position
                    ));
                }
                ')' => {
                    self.advance();
                    tokens.push(PositionedToken::new(
                        Token::RightParen, start_line, start_column, start_pos, self.position
                    ));
                }
                '"' => {
                    let token = self.read_string()?;
                    tokens.push(PositionedToken::new(
                        token, start_line, start_column, start_pos, self.position
                    ));
                }
                ch if ch.is_ascii_digit() || ch == '-' => {
                    let token = self.read_number()?;
                    tokens.push(PositionedToken::new(
                        token, start_line, start_column, start_pos, self.position
                    ));
                }
                '#' => {
                    let token = self.read_boolean()?;
                    tokens.push(PositionedToken::new(
                        token, start_line, start_column, start_pos, self.position
                    ));
                }
                ch if ch.is_alphabetic() || ch == '+' || ch == '*' || ch == '/' || ch == '=' || ch == '<' || ch == '>' => {
                    let token = self.read_symbol()?;
                    tokens.push(PositionedToken::new(
                        token, start_line, start_column, start_pos, self.position
                    ));
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
        
        tokens.push(PositionedToken::new(
            Token::EOF, self.line, self.column, self.position, self.position
        ));
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
        let start_line = self.line;
        let start_column = self.column;
        
        // Handle negative numbers
        if self.current_char()? == '-' {
            value.push('-');
            self.advance();
            
            // Check if there's a digit after the minus sign
            if self.position >= self.input.len() || !self.current_char()?.is_ascii_digit() {
                // This is not a number, it's likely a symbol starting with -
                // Backtrack and let it be handled as a symbol
                self.position -= 1;
                self.column -= 1;
                return self.read_symbol();
            }
        }
        
        // Must have at least one digit
        let mut has_digits = false;
        
        while self.position < self.input.len() {
            match self.current_char() {
                Ok(ch) if ch.is_ascii_digit() => {
                    has_digits = true;
                    value.push(ch);
                    self.advance();
                }
                Ok('.') if !is_float && has_digits => {
                    is_float = true;
                    value.push('.');
                    self.advance();
                }
                _ => break,
            }
        }
        
        if !has_digits {
            return Err(ParseError::InvalidNumber(value.clone(), start_line, start_column));
        }
        
        if is_float {
            let float_val = value.parse::<f64>().map_err(|_| {
                ParseError::InvalidNumber(value.clone(), start_line, start_column)
            })?;
            Ok(Token::Float(float_val))
        } else {
            let int_val = value.parse::<i64>().map_err(|_| {
                ParseError::InvalidNumber(value.clone(), start_line, start_column)
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

/// Parser for Causality Lisp expressions with enhanced error reporting
pub struct LispParser {
    tokens: Vec<PositionedToken>,
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
    
    fn current_token(&self) -> &PositionedToken {
        self.tokens.get(self.position).unwrap_or_else(|| {
            // Return a dummy EOF token if we're past the end
            static EOF_TOKEN: PositionedToken = PositionedToken {
                token: Token::EOF,
                line: 0,
                column: 0,
                start_pos: 0,
                end_pos: 0,
            };
            &EOF_TOKEN
        })
    }
    
    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }
    
    fn parse_expression(&mut self) -> ParseResult<Expr> {
        let current = self.current_token();
        match &current.token {
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
            Token::RightParen => {
                Err(ParseError::InvalidTokenSequence {
                    context: "unexpected closing parenthesis".to_string(),
                    suggestion: "remove the extra ')' or add an opening '(' before it".to_string(),
                    line: current.line,
                    column: current.column,
                })
            }
            Token::EOF => {
                Err(ParseError::UnexpectedEofInConstruct {
                    construct: "expression".to_string(),
                    hint: "add a complete expression before the end of input".to_string(),
                })
            }
        }
    }
    
    fn parse_list_or_special_form(&mut self) -> ParseResult<Expr> {
        let opening_paren = self.current_token().clone();
        self.advance(); // Skip '('
        
        // Check for empty list
        if matches!(self.current_token().token, Token::RightParen) {
            self.advance(); // Skip ')'
            return Ok(Expr::list(Vec::new()));
        }
        
        // Check if first token is a symbol and get its name
        let symbol_name = if let Token::Symbol(name) = &self.current_token().token {
            Some(name.to_string())
        } else {
            None
        };
        
        if let Some(name) = symbol_name {
            // Check for reserved special forms
            match name.as_str() {
                "lambda" | "let-tensor" | "case" | "tensor" | "inl" | "inr" | "alloc" | "consume" | "unit" | "let-unit" => {
                    self.parse_special_form(&name)
                }
                _ => {
                    // Parse as function call
                    let first = self.parse_expression()?;
                    self.parse_function_call(first)
                }
            }
        } else {
            // Parse as regular list
            let mut elements = Vec::new();
            while !matches!(self.current_token().token, Token::RightParen | Token::EOF) {
                elements.push(self.parse_expression()?);
            }
            
            if matches!(self.current_token().token, Token::RightParen) {
                self.advance(); // Skip ')'
            } else {
                return Err(ParseError::IncompleteConstruct {
                    construct: "list".to_string(),
                    expected: "closing parenthesis ')'".to_string(),
                    hint: format!("add ')' to close the list opened at line {}, column {}", opening_paren.line, opening_paren.column),
                    line: opening_paren.line,
                    column: opening_paren.column,
                });
            }
            
            Ok(Expr::list(elements))
        }
    }
    
    fn parse_special_form(&mut self, form_name: &str) -> ParseResult<Expr> {
        let form_token = self.current_token().clone();
        self.advance(); // Skip the form name
        
        match form_name {
            "lambda" => self.parse_lambda(&form_token),
            "let-tensor" => self.parse_let_tensor(&form_token),
            "case" => self.parse_case(&form_token),
            "tensor" => self.parse_tensor(&form_token),
            "inl" => self.parse_inl(&form_token),
            "inr" => self.parse_inr(&form_token),
            "alloc" => self.parse_alloc(&form_token),
            "consume" => self.parse_consume(&form_token),
            "unit" => self.parse_unit(&form_token),
            "let-unit" => self.parse_let_unit(&form_token),
            _ => {
                Err(ParseError::InvalidSpecialForm {
                    form: form_name.to_string(),
                    hint: "check the Causality Lisp documentation for valid special forms".to_string(),
                    line: form_token.line,
                    column: form_token.column,
                })
            }
        }
    }
    
    fn parse_lambda(&mut self, form_token: &PositionedToken) -> ParseResult<Expr> {
        // Parse parameter list
        self.expect_left_paren("lambda parameter list")?;
        let mut params = Vec::new();
        
        while !matches!(self.current_token().token, Token::RightParen) {
            if matches!(self.current_token().token, Token::EOF) {
                return Err(ParseError::IncompleteConstruct {
                    construct: "lambda parameter list".to_string(),
                    expected: "parameters followed by ')'".to_string(),
                    hint: "parameter names should be symbols like 'x' or 'value'".to_string(),
                    line: form_token.line,
                    column: form_token.column,
                });
            }
            
            let param_name = self.expect_symbol("lambda parameter")?;
            params.push(Param::new(Symbol::new(&param_name)));
        }
        self.expect_right_paren("lambda parameter list")?;
        
        // Parse body
        if matches!(self.current_token().token, Token::RightParen | Token::EOF) {
            return Err(ParseError::IncompleteConstruct {
                construct: "lambda expression".to_string(),
                expected: "body expression".to_string(),
                hint: "add an expression after the parameter list".to_string(),
                line: form_token.line,
                column: form_token.column,
            });
        }
        
        let body = self.parse_expression()?;
        self.expect_right_paren("lambda expression")?;
        
        Ok(Expr::lambda(params, body))
    }
    
    fn parse_let_unit(&mut self, form_token: &PositionedToken) -> ParseResult<Expr> {
        if matches!(self.current_token().token, Token::RightParen | Token::EOF) {
            return Err(ParseError::IncompleteConstruct {
                construct: "let-unit expression".to_string(),
                expected: "unit expression and body".to_string(),
                hint: "let-unit requires two expressions: (let-unit unit-expr body-expr)".to_string(),
                line: form_token.line,
                column: form_token.column,
            });
        }
        
        let unit_expr = self.parse_expression()?;
        
        if matches!(self.current_token().token, Token::RightParen | Token::EOF) {
            return Err(ParseError::IncompleteConstruct {
                construct: "let-unit expression".to_string(),
                expected: "body expression".to_string(),
                hint: "let-unit requires a body expression after the unit expression".to_string(),
                line: form_token.line,
                column: form_token.column,
            });
        }
        
        let body = self.parse_expression()?;
        self.expect_right_paren("let-unit expression")?;
        
        Ok(Expr::let_unit(unit_expr, body))
    }
    
    fn parse_let_tensor(&mut self, form_token: &PositionedToken) -> ParseResult<Expr> {
        if matches!(self.current_token().token, Token::RightParen | Token::EOF) {
            return Err(ParseError::IncompleteConstruct {
                construct: "let-tensor expression".to_string(),
                expected: "tensor expression, variable names, and body".to_string(),
                hint: "let-tensor requires: (let-tensor tensor-expr left-var right-var body-expr)".to_string(),
                line: form_token.line,
                column: form_token.column,
            });
        }
        
        let tensor_expr = self.parse_expression()?;
        let left_var = self.expect_symbol("left variable in let-tensor")?;
        let right_var = self.expect_symbol("right variable in let-tensor")?;
        
        if matches!(self.current_token().token, Token::RightParen | Token::EOF) {
            return Err(ParseError::IncompleteConstruct {
                construct: "let-tensor expression".to_string(),
                expected: "body expression".to_string(),
                hint: "let-tensor requires a body expression after the variable bindings".to_string(),
                line: form_token.line,
                column: form_token.column,
            });
        }
        
        let body = self.parse_expression()?;
        self.expect_right_paren("let-tensor expression")?;
        
        Ok(Expr::new(ExprKind::LetTensor(
            Box::new(tensor_expr),
            Symbol::new(&left_var),
            Symbol::new(&right_var),
            Box::new(body),
        )))
    }
    
    fn parse_case(&mut self, form_token: &PositionedToken) -> ParseResult<Expr> {
        if matches!(self.current_token().token, Token::RightParen | Token::EOF) {
            return Err(ParseError::IncompleteConstruct {
                construct: "case expression".to_string(),
                expected: "sum expression and branch handlers".to_string(),
                hint: "case requires: (case sum-expr left-var left-branch right-var right-branch)".to_string(),
                line: form_token.line,
                column: form_token.column,
            });
        }
        
        let sum_expr = self.parse_expression()?;
        let left_var = self.expect_symbol("left variable in case expression")?;
        let left_branch = self.parse_expression()?;
        let right_var = self.expect_symbol("right variable in case expression")?;
        let right_branch = self.parse_expression()?;
        self.expect_right_paren("case expression")?;
        
        Ok(Expr::case(
            sum_expr,
            Symbol::new(&left_var),
            left_branch,
            Symbol::new(&right_var),
            right_branch,
        ))
    }
    
    fn parse_tensor(&mut self, form_token: &PositionedToken) -> ParseResult<Expr> {
        if matches!(self.current_token().token, Token::RightParen | Token::EOF) {
            return Err(ParseError::IncompleteConstruct {
                construct: "tensor expression".to_string(),
                expected: "two expressions to combine".to_string(),
                hint: "tensor requires exactly two expressions: (tensor left-expr right-expr)".to_string(),
                line: form_token.line,
                column: form_token.column,
            });
        }
        
        let left = self.parse_expression()?;
        
        if matches!(self.current_token().token, Token::RightParen | Token::EOF) {
            return Err(ParseError::IncompleteConstruct {
                construct: "tensor expression".to_string(),
                expected: "second expression".to_string(),
                hint: "tensor requires exactly two expressions".to_string(),
                line: form_token.line,
                column: form_token.column,
            });
        }
        
        let right = self.parse_expression()?;
        self.expect_right_paren("tensor expression")?;
        
        Ok(Expr::tensor(left, right))
    }
    
    fn parse_inl(&mut self, form_token: &PositionedToken) -> ParseResult<Expr> {
        if matches!(self.current_token().token, Token::RightParen | Token::EOF) {
            return Err(ParseError::IncompleteConstruct {
                construct: "inl expression".to_string(),
                expected: "value expression".to_string(),
                hint: "inl requires one expression: (inl value-expr)".to_string(),
                line: form_token.line,
                column: form_token.column,
            });
        }
        
        let value = self.parse_expression()?;
        self.expect_right_paren("inl expression")?;
        
        Ok(Expr::new(ExprKind::Inl(Box::new(value))))
    }
    
    fn parse_inr(&mut self, form_token: &PositionedToken) -> ParseResult<Expr> {
        if matches!(self.current_token().token, Token::RightParen | Token::EOF) {
            return Err(ParseError::IncompleteConstruct {
                construct: "inr expression".to_string(),
                expected: "value expression".to_string(),
                hint: "inr requires one expression: (inr value-expr)".to_string(),
                line: form_token.line,
                column: form_token.column,
            });
        }
        
        let value = self.parse_expression()?;
        self.expect_right_paren("inr expression")?;
        
        Ok(Expr::new(ExprKind::Inr(Box::new(value))))
    }
    
    fn parse_alloc(&mut self, form_token: &PositionedToken) -> ParseResult<Expr> {
        if matches!(self.current_token().token, Token::RightParen | Token::EOF) {
            return Err(ParseError::IncompleteConstruct {
                construct: "alloc expression".to_string(),
                expected: "value expression to allocate".to_string(),
                hint: "alloc requires one expression: (alloc value-expr)".to_string(),
                line: form_token.line,
                column: form_token.column,
            });
        }
        
        let value = self.parse_expression()?;
        self.expect_right_paren("alloc expression")?;
        
        Ok(Expr::new(ExprKind::Alloc(Box::new(value))))
    }
    
    fn parse_consume(&mut self, form_token: &PositionedToken) -> ParseResult<Expr> {
        if matches!(self.current_token().token, Token::RightParen | Token::EOF) {
            return Err(ParseError::IncompleteConstruct {
                construct: "consume expression".to_string(),
                expected: "resource expression to consume".to_string(),
                hint: "consume requires one expression: (consume resource-expr)".to_string(),
                line: form_token.line,
                column: form_token.column,
            });
        }
        
        let resource = self.parse_expression()?;
        self.expect_right_paren("consume expression")?;
        
        Ok(Expr::new(ExprKind::Consume(Box::new(resource))))
    }
    
    fn parse_unit(&mut self, _form_token: &PositionedToken) -> ParseResult<Expr> {
        self.expect_right_paren("unit expression")?;
        Ok(Expr::new(ExprKind::UnitVal))
    }
    
    fn parse_function_call(&mut self, func: Expr) -> ParseResult<Expr> {
        let mut args = Vec::new();
        
        while !matches!(self.current_token().token, Token::RightParen | Token::EOF) {
            args.push(self.parse_expression()?);
        }
        
        if matches!(self.current_token().token, Token::EOF) {
            return Err(ParseError::UnexpectedEofInConstruct {
                construct: "function call".to_string(),
                hint: "add ')' to close the function call".to_string(),
            });
        }
        
        self.expect_right_paren("function call")?;
        Ok(Expr::apply(func, args))
    }
    
    fn expect_symbol(&mut self, context: &str) -> ParseResult<String> {
        let current = self.current_token();
        match &current.token {
            Token::Symbol(sym) => {
                let name = sym.to_string();
                self.advance();
                Ok(name)
            }
            _ => {
                Err(ParseError::expected_symbol_for(
                    context,
                    &current.format_for_error(),
                    current.line,
                    current.column,
                ))
            }
        }
    }
    
    fn expect_left_paren(&mut self, _context: &str) -> ParseResult<()> {
        let current = self.current_token();
        match &current.token {
            Token::LeftParen => {
                self.advance();
                Ok(())
            }
            _ => {
                Err(ParseError::expected_token(
                    "'('",
                    &current.format_for_error(),
                    current.line,
                    current.column,
                ))
            }
        }
    }
    
    fn expect_right_paren(&mut self, context: &str) -> ParseResult<()> {
        let current = self.current_token();
        match &current.token {
            Token::RightParen => {
                self.advance();
                Ok(())
            }
            Token::EOF => {
                Err(ParseError::UnexpectedEofInConstruct {
                    construct: context.to_string(),
                    hint: "add ')' to close the expression".to_string(),
                })
            }
            _ => {
                Err(ParseError::expected_token(
                    "')'",
                    &current.format_for_error(),
                    current.line,
                    current.column,
                ))
            }
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
    
    #[test]
    fn test_helpful_error_messages() {
        let mut parser = LispParser::new();
        
        // Test unclosed parenthesis
        let result = parser.parse("(+ 1 2");
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("function call"));
        assert!(error_msg.contains("add ')'"));
        
        // Test incomplete lambda
        parser = LispParser::new();
        let result = parser.parse("(lambda ())");
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("lambda expression"));
        assert!(error_msg.contains("body expression"));
        
        // Test unexpected closing paren
        parser = LispParser::new();
        let result = parser.parse(")");
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("unexpected closing parenthesis"));
    }
} 