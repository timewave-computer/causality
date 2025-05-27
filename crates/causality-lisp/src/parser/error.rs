//! Parser Error Types
//!
//! Error types and utilities for the Causality Lisp parser, including
//! source location tracking and appropriate error categories.

//-----------------------------------------------------------------------------
// Error Types
//-----------------------------------------------------------------------------

#[cfg(feature = "std")]
use std::error::Error;
#[cfg(feature = "std")]
use std::fmt;

#[cfg(not(feature = "std"))]
use crate::compatibility::fmt;

/// Source location in the input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// Comprehensive error type for parsing Lisp expressions
#[derive(Debug)]
pub enum ParseError {
    /// Error during lexical analysis (tokenization)
    LexicalError {
        /// Error message
        message: String,
        /// Source location
        location: SourceLocation,
    },

    /// Error during syntactic analysis (parsing)
    SyntaxError {
        /// Error message
        message: String,
        /// Source location
        location: SourceLocation,
    },

    /// Error during semantic analysis (type checking, etc.)
    SemanticError {
        /// Error message
        message: String,
        /// Source location
        location: SourceLocation,
    },

    /// Unexpected end of input
    UnexpectedEOF {
        /// Expected token or syntax
        expected: String,
        /// Last known location
        location: SourceLocation,
    },
}

impl ParseError {
    /// Create a new lexical error
    pub fn lexical_error(
        message: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Self::LexicalError {
            message: message.into(),
            location: SourceLocation::new(line, column),
        }
    }

    /// Create a new syntax error
    pub fn syntax_error(
        message: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Self::SyntaxError {
            message: message.into(),
            location: SourceLocation::new(line, column),
        }
    }

    /// Create a new semantic error
    pub fn semantic_error(
        message: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Self::SemanticError {
            message: message.into(),
            location: SourceLocation::new(line, column),
        }
    }

    /// Create a new unexpected EOF error
    pub fn unexpected_eof(
        expected: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        Self::UnexpectedEOF {
            expected: expected.into(),
            location: SourceLocation::new(line, column),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LexicalError { message, location } => {
                write!(f, "Lexical error at {}: {}", location, message)
            }
            Self::SyntaxError { message, location } => {
                write!(f, "Syntax error at {}: {}", location, message)
            }
            Self::SemanticError { message, location } => {
                write!(f, "Semantic error at {}: {}", location, message)
            }
            Self::UnexpectedEOF { expected, location } => {
                write!(
                    f,
                    "Unexpected end of input at {}: expected {}",
                    location, expected
                )
            }
        }
    }
}

#[cfg(feature = "std")]
impl Error for ParseError {}

/// Result type for parser operations
pub type ParseResult<T> = Result<T, ParseError>;
