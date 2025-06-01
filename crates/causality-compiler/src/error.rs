//! Compilation errors for the Causality compiler

use std::fmt;

/// Result type for compilation operations
pub type CompileResult<T> = Result<T, CompileError>;

/// Errors that can occur during compilation
#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {
    /// Parse error in the source code
    ParseError {
        message: String,
        location: Option<Location>,
    },
    
    /// Type checking error
    TypeError {
        message: String,
        expected: Option<String>,
        found: Option<String>,
        location: Option<Location>,
    },
    
    /// Compilation error from Layer 2 to Layer 1
    Layer2Error {
        message: String,
        location: Option<Location>,
    },
    
    /// Compilation error from Layer 1 to Layer 0  
    Layer1Error {
        message: String,
        location: Option<Location>,
    },
    
    /// Unknown symbol or identifier
    UnknownSymbol {
        symbol: String,
        location: Option<Location>,
    },
    
    /// Invalid arity (wrong number of arguments)
    InvalidArity {
        expected: usize,
        found: usize,
        location: Option<Location>,
    },
    
    /// Generic compilation error
    CompilationError {
        message: String,
        location: Option<Location>,
    },
}

/// Location in source code
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::ParseError { message, location } => {
                if let Some(loc) = location {
                    write!(f, "Parse error at {}:{}: {}", loc.line, loc.column, message)
                } else {
                    write!(f, "Parse error: {}", message)
                }
            }
            CompileError::TypeError { message, expected, found, location } => {
                let type_info = match (expected, found) {
                    (Some(exp), Some(fnd)) => format!(" (expected {}, found {})", exp, fnd),
                    (Some(exp), None) => format!(" (expected {})", exp),
                    (None, Some(fnd)) => format!(" (found {})", fnd),
                    (None, None) => String::new(),
                };
                
                if let Some(loc) = location {
                    write!(f, "Type error at {}:{}: {}{}", loc.line, loc.column, message, type_info)
                } else {
                    write!(f, "Type error: {}{}", message, type_info)
                }
            }
            CompileError::Layer2Error { message, location } => {
                if let Some(loc) = location {
                    write!(f, "Layer 2 compilation error at {}:{}: {}", loc.line, loc.column, message)
                } else {
                    write!(f, "Layer 2 compilation error: {}", message)
                }
            }
            CompileError::Layer1Error { message, location } => {
                if let Some(loc) = location {
                    write!(f, "Layer 1 compilation error at {}:{}: {}", loc.line, loc.column, message)
                } else {
                    write!(f, "Layer 1 compilation error: {}", message)
                }
            }
            CompileError::UnknownSymbol { symbol, location } => {
                if let Some(loc) = location {
                    write!(f, "Unknown symbol '{}' at {}:{}", symbol, loc.line, loc.column)
                } else {
                    write!(f, "Unknown symbol '{}'", symbol)
                }
            }
            CompileError::InvalidArity { expected, found, location } => {
                if let Some(loc) = location {
                    write!(f, "Invalid arity at {}:{}: expected {} arguments, found {}", 
                           loc.line, loc.column, expected, found)
                } else {
                    write!(f, "Invalid arity: expected {} arguments, found {}", expected, found)
                }
            }
            CompileError::CompilationError { message, location } => {
                if let Some(loc) = location {
                    write!(f, "Compilation error at {}:{}: {}", loc.line, loc.column, message)
                } else {
                    write!(f, "Compilation error: {}", message)
                }
            }
        }
    }
}

impl std::error::Error for CompileError {}

impl From<&str> for CompileError {
    fn from(message: &str) -> Self {
        CompileError::CompilationError {
            message: message.to_string(),
            location: None,
        }
    }
}

impl From<String> for CompileError {
    fn from(message: String) -> Self {
        CompileError::CompilationError {
            message,
            location: None,
        }
    }
} 