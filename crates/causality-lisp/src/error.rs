//! Error types for the Causality Lisp language implementation

use thiserror::Error;

/// Main error type for Lisp operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum LispError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
    
    #[error("Evaluation error: {0}")]
    Eval(#[from] EvalError),
    
    #[error("Type error: {0}")]
    Type(#[from] TypeError),
}

/// Parse-time errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    #[error("Unexpected end of input")]
    UnexpectedEof,
    
    #[error("Unexpected character '{0}' at line {1}, column {2}")]
    UnexpectedChar(char, usize, usize),
    
    #[error("Unclosed string literal at line {0}, column {1}")]
    UnclosedString(usize, usize),
    
    #[error("Unclosed parentheses at line {0}, column {1}")]
    UnclosedParen(usize, usize),
    
    #[error("Unexpected closing parenthesis at line {0}, column {1}")]
    UnexpectedCloseParen(usize, usize),
    
    #[error("Invalid number format '{0}' at line {1}, column {2}")]
    InvalidNumber(String, usize, usize),
    
    #[error("Invalid escape sequence '\\{0}' at line {1}, column {2}")]
    InvalidEscape(char, usize, usize),
    
    #[error("Empty expression")]
    EmptyExpression,
    
    #[error("Invalid syntax: {0}")]
    InvalidSyntax(String),
}

/// Runtime evaluation errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum EvalError {
    /// Invalid function call
    #[error("Invalid function call: {0}")]
    InvalidCall(String),
    
    /// Type mismatch error
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        expected: String,
        found: String,
    },
    
    /// Arity mismatch error
    #[error("Arity mismatch: expected {expected} arguments, found {found}")]
    ArityMismatch {
        expected: usize,
        found: usize,
    },
    
    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,
    
    /// Stack overflow
    #[error("Stack overflow")]
    StackOverflow,
    
    /// Resource not available
    #[error("Resource error: {0}")]
    ResourceError(String),
    
    /// Effect handling error
    #[error("Effect error: {0}")]
    EffectError(String),
    
    /// Generic evaluation error
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    
    /// Unbound variable error
    #[error("Unbound variable: {0}")]
    UnboundVariable(String),
    
    /// Undefined variable error (alias for backward compatibility)
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    
    /// Unknown built-in function
    #[error("Unknown built-in function: {0}")]
    UnknownBuiltin(String),
    
    /// Not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    
    /// Not callable error
    #[error("Value is not callable")]
    NotCallable,
    
    /// Invalid sum tag error
    #[error("Invalid sum tag")]
    InvalidSumTag,
    
    /// Resource not found error
    #[error("Resource not found")]
    ResourceNotFound,
    
    /// Invalid resource reference error
    #[error("Invalid resource reference")]
    InvalidResourceRef,
    
    /// Linear type violation error
    #[error("Linear type violation: {0}")]
    LinearityViolation(String),
}

/// Type system errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected}, found {found}")]
    Mismatch { expected: String, found: String },
    
    #[error("Undefined type '{0}'")]
    UndefinedType(String),
    
    #[error("Cannot unify types {0} and {1}")]
    UnificationError(String, String),
    
    #[error("Occurs check failed: {0} occurs in {1}")]
    OccursCheck(String, String),
    
    #[error("Linear type error: {0}")]
    LinearityError(String),
    
    #[error("Resource type error: {0}")]
    ResourceError(String),
    
    #[error("Effect type error: {0}")]
    EffectTypeError(String),
}

/// Result types for convenience
pub type ParseResult<T> = Result<T, ParseError>;
pub type EvalResult<T> = Result<T, EvalError>;
pub type TypeResult<T> = Result<T, TypeError>;
pub type LispResult<T> = Result<T, LispError>; 