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

/// Parse-time errors with enhanced context and helpful suggestions
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
    
    // Enhanced error types with better context
    #[error("Expected {expected} but found {found} at line {line}, column {column}")]
    ExpectedToken {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },
    
    #[error("Expected symbol for {context} but found {found} at line {line}, column {column}")]
    ExpectedSymbol {
        context: String,
        found: String,
        line: usize,
        column: usize,
    },
    
    #[error("Invalid special form '{form}' at line {line}, column {column}\n  Hint: {hint}")]
    InvalidSpecialForm {
        form: String,
        hint: String,
        line: usize,
        column: usize,
    },
    
    #[error("Incomplete {construct} at line {line}, column {column}\n  Expected: {expected}\n  Hint: {hint}")]
    IncompleteConstruct {
        construct: String,
        expected: String,
        hint: String,
        line: usize,
        column: usize,
    },
    
    #[error("Malformed {construct} at line {line}, column {column}\n  Error: {description}\n  Hint: {hint}")]
    MalformedConstruct {
        construct: String,
        description: String,
        hint: String,
        line: usize,
        column: usize,
    },
    
    #[error("Too {issue} arguments for '{form}' at line {line}, column {column}\n  Expected: {expected}\n  Found: {found}")]
    ArgumentCount {
        form: String,
        issue: String, // "many" or "few"
        expected: String,
        found: usize,
        line: usize,
        column: usize,
    },
    
    #[error("Invalid token sequence at line {line}, column {column}\n  Context: {context}\n  Suggestion: {suggestion}")]
    InvalidTokenSequence {
        context: String,
        suggestion: String,
        line: usize,
        column: usize,
    },
    
    #[error("Unexpected end of input while parsing {construct}\n  Hint: {hint}")]
    UnexpectedEofInConstruct {
        construct: String,
        hint: String,
    },
    
    #[error("Reserved keyword '{keyword}' used as {usage} at line {line}, column {column}\n  Hint: {hint}")]
    ReservedKeyword {
        keyword: String,
        usage: String,
        hint: String,
        line: usize,
        column: usize,
    },
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
    
    /// Arithmetic overflow
    #[error("Arithmetic overflow: {0}")]
    ArithmeticOverflow(String),
    
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

/// Helper functions for creating common error patterns
impl ParseError {
    /// Create an error for when a symbol is expected in a specific context
    pub fn expected_symbol_for(context: &str, found: &str, line: usize, column: usize) -> Self {
        Self::ExpectedSymbol {
            context: context.to_string(),
            found: found.to_string(),
            line,
            column,
        }
    }
    
    /// Create an error for when a specific token is expected
    pub fn expected_token(expected: &str, found: &str, line: usize, column: usize) -> Self {
        Self::ExpectedToken {
            expected: expected.to_string(),
            found: found.to_string(),
            line,
            column,
        }
    }
    
    /// Create an error for malformed constructs with helpful hints
    pub fn malformed_construct(construct: &str, description: &str, hint: &str, line: usize, column: usize) -> Self {
        Self::MalformedConstruct {
            construct: construct.to_string(),
            description: description.to_string(),
            hint: hint.to_string(),
            line,
            column,
        }
    }
    
    /// Create an error for incomplete constructs
    pub fn incomplete_construct(construct: &str, expected: &str, hint: &str, line: usize, column: usize) -> Self {
        Self::IncompleteConstruct {
            construct: construct.to_string(),
            expected: expected.to_string(),
            hint: hint.to_string(),
            line,
            column,
        }
    }
    
    /// Create an error for wrong argument counts
    pub fn wrong_argument_count(form: &str, expected: &str, found: usize, line: usize, column: usize) -> Self {
        let issue = if found > expected.parse::<usize>().unwrap_or(0) { "many" } else { "few" };
        Self::ArgumentCount {
            form: form.to_string(),
            issue: issue.to_string(),
            expected: expected.to_string(),
            found,
            line,
            column,
        }
    }
}

/// Result types for convenience
pub type ParseResult<T> = Result<T, ParseError>;
pub type EvalResult<T> = Result<T, EvalError>;
pub type TypeResult<T> = Result<T, TypeError>;
pub type LispResult<T> = Result<T, LispError>; 