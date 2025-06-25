//! Error handling for the Causality system
//!
//! This module provides comprehensive error types and handling for all Causality operations,
//! including type errors, resource management errors, and system-level failures.

#![allow(clippy::result_large_err)]

use thiserror::Error;

/// Core system error type
///
/// This error type encompasses all possible failures in the Causality system,
/// from type checking errors to resource management failures.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum Error {
    /// Type system errors
    #[error("Type error: {0}")]
    Type(#[from] Box<TypeError>),

    /// Resource management errors
    #[error("Resource error: {message}")]
    Resource { message: String },

    /// Linear resource violations
    #[error("Linear resource violation: {0}")]
    Linearity(#[from] LinearityError),

    /// Content addressing errors
    #[error("Content addressing error: {message}")]
    ContentAddressing { message: String },

    /// Serialization errors
    #[error("Serialization error: {message}")]
    Serialization { message: String },

    /// Storage errors
    #[error("Storage error: {message}")]
    Storage { message: String },

    /// Network errors
    #[error("Network error: {message}")]
    Network { message: String },

    /// Validation errors
    #[error("Validation error: {message}")]
    Validation { message: String },

    /// Generic system error
    #[error("System error: {message}")]
    System { message: String },
}

/// Type system error variants
///
/// Detailed error information for type checking and inference failures.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum TypeError {
    /// Type mismatch between expected and actual types
    #[error("Type mismatch: expected {expected}, found {actual}")]
    Mismatch { expected: String, actual: String },

    /// Unification failure during type inference
    #[error("Cannot unify types: {left} and {right}")]
    UnificationFailure { left: String, right: String },

    /// Unknown type variable
    #[error("Unknown type variable: {name}")]
    UnknownTypeVariable { name: String },

    /// Occurs check failure (infinite type)
    #[error("Occurs check failed: type variable {var} occurs in {type_expr}")]
    OccursCheck { var: String, type_expr: String },

    /// Kind mismatch (type vs kind level)
    #[error("Kind mismatch: expected {expected}, found {actual}")]
    KindMismatch { expected: String, actual: String },

    /// Arity mismatch for type constructors
    #[error("Arity mismatch for {constructor}: expected {expected} arguments, found {actual}")]
    ArityMismatch {
        constructor: String,
        expected: usize,
        actual: usize,
    },

    /// Linear resource type error
    #[error("Linear resource error: {message}")]
    LinearResource { message: String },

    /// Session type error
    #[error("Session type error: {message}")]
    SessionType { message: String },

    /// Effect type error
    #[error("Effect type error: {message}")]
    EffectType { message: String },

    /// Row type error
    #[error("Row type error: {message}")]
    RowType { message: String },

    /// Constraint solving error
    #[error("Constraint solving error: {message}")]
    ConstraintSolving { message: String },

    /// Generic type error
    #[error("Type error: {message}")]
    Generic { message: String },
}

/// Linear resource management errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum LinearityError {
    /// Resource used more than once
    #[error("Resource used multiple times: {resource}")]
    MultipleUse { resource: String },

    /// Resource not used
    #[error("Resource not used: {resource}")]
    NotUsed { resource: String },

    /// Resource used after being consumed
    #[error("Resource used after consumption: {resource}")]
    UseAfterConsumption { resource: String },

    /// Generic linearity error
    #[error("Linearity error: {message}")]
    Generic { message: String },
}

/// Result type for Causality operations
pub type Result<T> = std::result::Result<T, Error>;

/// Result type for type checking operations
pub type TypeResult<T> = std::result::Result<T, TypeError>;

/// Error classification for handling and recovery
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// Fatal errors that should stop execution
    Fatal,
    /// Recoverable errors that can be retried
    Recoverable,
    /// Validation errors from user input
    Validation,
    /// Type checking errors
    Type,
    /// Resource management errors
    Resource,
    /// Serialization/deserialization errors
    SerializationError,
}

impl Error {
    /// Get the error classification
    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::Type(_) => ErrorKind::Type,
            Error::Resource { .. } => ErrorKind::Resource,
            Error::Linearity(_) => ErrorKind::Resource,
            Error::ContentAddressing { .. } => ErrorKind::Resource,
            Error::Serialization { .. } => ErrorKind::SerializationError,
            Error::Storage { .. } => ErrorKind::Recoverable,
            Error::Network { .. } => ErrorKind::Recoverable,
            Error::Validation { .. } => ErrorKind::Validation,
            Error::System { .. } => ErrorKind::Fatal,
        }
    }

    /// Helper for serialization errors
    pub fn serialization<S: Into<String>>(msg: S) -> Self {
        Error::Serialization { message: msg.into() }
    }

    /// Create a resource error
    pub fn resource(message: impl Into<String>) -> Self {
        Self::Resource {
            message: message.into(),
        }
    }

    /// Create a content addressing error
    pub fn content_addressing(message: impl Into<String>) -> Self {
        Self::ContentAddressing {
            message: message.into(),
        }
    }

    /// Create a storage error
    pub fn storage(message: impl Into<String>) -> Self {
        Self::Storage {
            message: message.into(),
        }
    }

    /// Create a network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create a system error
    pub fn system(message: impl Into<String>) -> Self {
        Self::System {
            message: message.into(),
        }
    }
}

impl TypeError {
    /// Create a type mismatch error
    pub fn mismatch(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::Mismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create a unification failure error
    pub fn unification_failure(left: impl Into<String>, right: impl Into<String>) -> Self {
        Self::UnificationFailure {
            left: left.into(),
            right: right.into(),
        }
    }

    /// Create an unknown type variable error
    pub fn unknown_type_variable(name: impl Into<String>) -> Self {
        Self::UnknownTypeVariable { name: name.into() }
    }

    /// Create an occurs check error
    pub fn occurs_check(var: impl Into<String>, type_expr: impl Into<String>) -> Self {
        Self::OccursCheck {
            var: var.into(),
            type_expr: type_expr.into(),
        }
    }

    /// Create a kind mismatch error
    pub fn kind_mismatch(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::KindMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create an arity mismatch error
    pub fn arity_mismatch(
        constructor: impl Into<String>,
        expected: usize,
        actual: usize,
    ) -> Self {
        Self::ArityMismatch {
            constructor: constructor.into(),
            expected,
            actual,
        }
    }

    /// Create a linear resource error
    pub fn linear_resource(message: impl Into<String>) -> Self {
        Self::LinearResource {
            message: message.into(),
        }
    }

    /// Create a session type error
    pub fn session_type(message: impl Into<String>) -> Self {
        Self::SessionType {
            message: message.into(),
        }
    }

    /// Create an effect type error
    pub fn effect_type(message: impl Into<String>) -> Self {
        Self::EffectType {
            message: message.into(),
        }
    }

    /// Create a row type error
    pub fn row_type(message: impl Into<String>) -> Self {
        Self::RowType {
            message: message.into(),
        }
    }

    /// Create a constraint solving error
    pub fn constraint_solving(message: impl Into<String>) -> Self {
        Self::ConstraintSolving {
            message: message.into(),
        }
    }

    /// Create a generic type error
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }
}

impl LinearityError {
    /// Create a multiple use error
    pub fn multiple_use(resource: impl Into<String>) -> Self {
        Self::MultipleUse {
            resource: resource.into(),
        }
    }

    /// Create a not used error
    pub fn not_used(resource: impl Into<String>) -> Self {
        Self::NotUsed {
            resource: resource.into(),
        }
    }

    /// Create a use after consumption error
    pub fn use_after_consumption(resource: impl Into<String>) -> Self {
        Self::UseAfterConsumption {
            resource: resource.into(),
        }
    }

    /// Create a generic linearity error
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }
}

// Implement From<TypeError> for Error to handle the boxing
impl From<TypeError> for Error {
    fn from(err: TypeError) -> Self {
        Self::Type(Box::new(err))
    }
}

//-----------------------------------------------------------------------------
// Conversion Helpers
//-----------------------------------------------------------------------------

/// Extension trait for adding context to Results
pub trait ResultExt<T> {
    /// Add context to an error
    fn context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static;
    
    /// Add lazy context to an error
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: Into<Error>,
{
    fn context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let error = e.into();
            Error::system(format!("{}: {}", context, error))
        })
    }
    
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let error = e.into();
            Error::system(format!("{}: {}", f(), error))
        })
    }
}

//-----------------------------------------------------------------------------
// Common error patterns
//-----------------------------------------------------------------------------

/// Helper macro for creating context-aware errors
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err($crate::system::Error::Other(anyhow::anyhow!($msg)))
    };
    ($err:expr $(,)?) => {
        return Err($crate::system::Error::Other(anyhow::anyhow!($err)))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::system::Error::Other(anyhow::anyhow!($fmt, $($arg)*)))
    };
}

/// Helper macro for ensuring conditions
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            return Err($crate::system::Error::Other(anyhow::anyhow!($msg)))
        }
    };
    ($cond:expr, $err:expr $(,)?) => {
        if !$cond {
            return Err($crate::system::Error::Other(anyhow::anyhow!($err)))
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            return Err($crate::system::Error::Other(anyhow::anyhow!($fmt, $($arg)*)))
        }
    };
} 