//! Lexer Implementation for Causality Lisp
//!
//! Tokenization logic for the Causality Lisp parser.
//! Converts source code strings into tokens while preserving source location.

//-----------------------------------------------------------------------------
// Lexer Implementation
//-----------------------------------------------------------------------------

#[cfg(not(feature = "std"))]
use crate::compatibility::{Chars, Peekable};
use crate::parser::error::{ParseError, ParseResult, SourceLocation};

//-----------------------------------------------------------------------------
// Token Definition
//-----------------------------------------------------------------------------

/// Token types for the Lisp parser
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Left parenthesis '('
    LParen,

    /// Right parenthesis ')'
    RParen,

    /// Left bracket '['
    LBracket,

    /// Right bracket ']'
    RBracket,

    /// Single quote '\''
    Quote,

    /// Integer literal
    Integer(i64),

    /// String literal
    String(String),

    /// Boolean literal
    Boolean(bool),

    /// Nil/null literal
    Nil,

    /// Symbol (identifier)
    Symbol(String),
}

/// A token with its source location
#[derive(Debug, Clone)]
pub struct TokenWithLocation {
    /// The token
    pub token: Token,

    /// Source location
    pub location: SourceLocation,
}

//-----------------------------------------------------------------------------
// Lexer Implementation
//-----------------------------------------------------------------------------

/// Lexer for tokenizing Lisp code
pub struct Lexer<'a> {
    /// The input string being processed
    input: &'a str,

    /// Current position in the input string
    pos: usize,

    /// Current line number (1-based)
    line: usize,

    /// Current column number (1-based)
    column: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Get the current source location
    fn current_location(&self) -> SourceLocation {
        SourceLocation::new(self.line, self.column)
    }

    /// Advance the input by one character
    fn advance(&mut self) -> Option<char> {
        let c = self.input.chars().nth(self.pos);

        if let Some(ch) = c {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        c
    }

    /// Peek the next character without advancing
    fn peek(&mut self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip comments (starting with semicolon to end of line)
    fn skip_comment(&mut self) {
        if let Some(';') = self.peek() {
            self.advance(); // consume the semicolon

            while let Some(c) = self.advance() {
                if c == '\n' {
                    break;
                }
            }
        }
    }

    /// Skip whitespace and comments
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.skip_whitespace();

            if let Some(';') = self.peek() {
                self.skip_comment();
            } else {
                break;
            }
        }
    }

    /// Parse a string literal
    fn parse_string(&mut self) -> ParseResult<Token> {
        let start_loc = self.current_location();

        // Consume the opening quote
        self.advance();

        let mut value = String::new();
        let mut escaped = false;

        loop {
            match self.peek() {
                None => {
                    return Err(ParseError::unexpected_eof(
                        "closing quote",
                        start_loc.line,
                        start_loc.column,
                    ));
                }
                Some('"') if !escaped => {
                    self.advance(); // consume closing quote
                    break;
                }
                Some('\\') if !escaped => {
                    self.advance(); // consume backslash
                    escaped = true;
                }
                Some(c) => {
                    if escaped {
                        match c {
                            'n' => value.push('\n'),
                            'r' => value.push('\r'),
                            't' => value.push('\t'),
                            '\\' => value.push('\\'),
                            '"' => value.push('"'),
                            _ => value.push(c),
                        }
                        escaped = false;
                    } else {
                        value.push(c);
                    }
                    self.advance();
                }
            }
        }

        Ok(Token::String(value))
    }

    /// Parse a numeric literal
    fn parse_number(&mut self) -> ParseResult<Token> {
        let start_loc = self.current_location();
        let mut value = String::new();

        // Check for negative sign
        if let Some('-') = self.peek() {
            value.push('-');
            self.advance();
        }

        // Parse digits
        let mut has_digits = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                value.push(c);
                self.advance();
                has_digits = true;
            } else {
                break;
            }
        }

        if !has_digits {
            return Err(ParseError::lexical_error(
                "Invalid number format",
                start_loc.line,
                start_loc.column,
            ));
        }

        // Parse as integer
        match value.parse::<i64>() {
            Ok(n) => Ok(Token::Integer(n)),
            Err(_) => Err(ParseError::lexical_error(
                format!("Integer value out of range: {}", value),
                start_loc.line,
                start_loc.column,
            )),
        }
    }

    /// Parse a symbol or keyword
    fn parse_symbol(&mut self) -> ParseResult<Token> {
        let mut value = String::new();

        while let Some(c) = self.peek() {
            if c.is_whitespace() || "()[]'\"`;".contains(c) {
                break;
            }

            value.push(c);
            self.advance();
        }

        // Check for special symbols
        Ok(match value.as_str() {
            "nil" | "null" => Token::Nil,
            "true" | "#t" => Token::Boolean(true),
            "false" | "#f" => Token::Boolean(false),
            _ => Token::Symbol(value),
        })
    }

    /// Get the next token
    pub fn next_token(&mut self) -> ParseResult<Option<TokenWithLocation>> {
        self.skip_whitespace_and_comments();

        if self.peek().is_none() {
            return Ok(None);
        }

        let location = self.current_location();
        let token = match self.peek().unwrap() {
            '(' => {
                self.advance();
                Ok(Token::LParen)
            }
            ')' => {
                self.advance();
                Ok(Token::RParen)
            }
            '[' => {
                self.advance();
                Ok(Token::LBracket)
            }
            ']' => {
                self.advance();
                Ok(Token::RBracket)
            }
            '\'' => {
                self.advance();
                Ok(Token::Quote)
            }
            '"' => self.parse_string(),
            '-' => {
                if let Some(next) = self.input.chars().nth(self.pos + 1) {
                    if next.is_ascii_digit() {
                        self.parse_number()
                    } else {
                        self.parse_symbol()
                    }
                } else {
                    self.parse_symbol()
                }
            }
            c if c.is_ascii_digit() => self.parse_number(),
            _ => self.parse_symbol(),
        }?;

        Ok(Some(TokenWithLocation { token, location }))
    }

    /// Tokenize the entire input
    pub fn tokenize(&mut self) -> ParseResult<Vec<TokenWithLocation>> {
        let mut tokens = Vec::new();

        while let Some(token) = self.next_token()? {
            tokens.push(token);
        }

        Ok(tokens)
    }
}

//-----------------------------------------------------------------------------
// Helper Function
//-----------------------------------------------------------------------------

/// Tokenize a string
pub fn tokenize(input: &str) -> ParseResult<Vec<TokenWithLocation>> {
    Lexer::new(input).tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_expressions() {
        let input = "(+ 1 2)";
        let tokens = tokenize(input).unwrap();

        assert_eq!(tokens.len(), 5); // LParen, +, 1, 2, RParen
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::Symbol("+".to_string()));
        assert_eq!(tokens[2].token, Token::Integer(1));
        assert_eq!(tokens[3].token, Token::Integer(2));
        assert_eq!(tokens[4].token, Token::RParen);
    }

    #[test]
    fn test_tokenize_with_comments_and_whitespace() {
        let input = "
        ; This is a comment
        (define x 10) ; Another comment
        ";
        let tokens = tokenize(input).unwrap();

        assert_eq!(tokens.len(), 5); // LParen, define, x, 10, RParen
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::Symbol("define".to_string()));
        assert_eq!(tokens[2].token, Token::Symbol("x".to_string()));
        assert_eq!(tokens[3].token, Token::Integer(10));
        assert_eq!(tokens[4].token, Token::RParen);
    }

    #[test]
    fn test_tokenize_strings() {
        let input = r#"(display "Hello, world!")"#;
        let tokens = tokenize(input).unwrap();

        assert_eq!(tokens.len(), 4); // LParen, display, string, RParen
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::Symbol("display".to_string()));
        assert_eq!(tokens[2].token, Token::String("Hello, world!".to_string()));
        assert_eq!(tokens[3].token, Token::RParen);
    }

    #[test]
    fn test_tokenize_special_forms() {
        let input = "(fn (x y) (+ x y))";

        let tokens = tokenize(input).unwrap();

        assert_eq!(tokens.len(), 12);
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::Symbol("fn".to_string()));
        assert_eq!(tokens[2].token, Token::LParen);
        assert_eq!(tokens[3].token, Token::Symbol("x".to_string()));
        assert_eq!(tokens[4].token, Token::Symbol("y".to_string()));
        assert_eq!(tokens[5].token, Token::RParen);
        assert_eq!(tokens[6].token, Token::LParen);
        assert_eq!(tokens[7].token, Token::Symbol("+".to_string()));
        assert_eq!(tokens[8].token, Token::Symbol("x".to_string()));
        assert_eq!(tokens[9].token, Token::Symbol("y".to_string()));
        assert_eq!(tokens[10].token, Token::RParen);
        assert_eq!(tokens[11].token, Token::RParen);
    }
}
