//! Unified error handling for causality-core
//!
//! This module provides a shared error pipeline using `thiserror` for structured
//! error types and `anyhow` for error context and chaining.

#![allow(clippy::result_large_err)]

use crate::{
    lambda::TypeInner,
    system::content_addressing::ResourceId,
    machine::instruction::RegisterId,
};
use thiserror::Error;
use anyhow;

/// Type alias for Results using our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the causality-core crate
#[derive(Error, Debug)]
pub enum Error {
    /// Type system errors
    #[error("type error: {0}")]
    Type(#[from] TypeError),
    
    /// Register machine errors
    #[error("machine error: {0}")]
    Machine(#[from] MachineError),
    
    /// Reduction/execution errors
    #[error("reduction error: {0}")]
    Reduction(#[from] ReductionError),
    
    /// Linearity errors
    #[error("linearity error: {0}")]
    Linearity(#[from] LinearityError),
    
    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Serialization errors
    #[error("serialization error: {0}")]
    Serialization(String),
    
    /// Generic errors with context
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Error kinds for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    TypeError,
    MachineError,
    ReductionError,
    LinearityError,
    IoError,
    SerializationError,
    Other,
}

impl Error {
    /// Get the kind of this error
    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::Type(_) => ErrorKind::TypeError,
            Error::Machine(_) => ErrorKind::MachineError,
            Error::Reduction(_) => ErrorKind::ReductionError,
            Error::Linearity(_) => ErrorKind::LinearityError,
            Error::Io(_) => ErrorKind::IoError,
            Error::Serialization(_) => ErrorKind::SerializationError,
            Error::Other(_) => ErrorKind::Other,
        }
    }
    
    /// Create an error with additional context
    pub fn context<C>(self, context: C) -> Self
    where
        C: std::fmt::Display + Send + Sync + 'static,
    {
        Error::Other(anyhow::Error::new(self).context(context))
    }
    
    /// Helper for serialization errors
    pub fn serialization<S: Into<String>>(msg: S) -> Self {
        Error::Serialization(msg.into())
    }
}

//-----------------------------------------------------------------------------
// Type System Errors
//-----------------------------------------------------------------------------

/// Errors related to the type system
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
    #[error("type mismatch: expected {expected:?}, found {found:?}")]
    Mismatch {
        expected: Box<TypeInner>,
        found: Box<TypeInner>,
    },
    
    #[error("unknown type: {0:?}")]
    UnknownType(TypeInner),
    
    #[error("invalid type constructor")]
    InvalidConstructor,
    
    #[error("type inference failed: {0}")]
    InferenceFailed(String),
    
    /// Type mismatch error  
    #[error("type mismatch: expected {expected:?}, found {found:?}")]
    TypeMismatch {
        expected: Box<TypeInner>,
        found: Box<TypeInner>,
    },
}

//-----------------------------------------------------------------------------
// Machine Errors
//-----------------------------------------------------------------------------

/// Errors that can occur during machine execution
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum MachineError {
    #[error("invalid register: {0:?}")]
    InvalidRegister(RegisterId),
    
    #[error("register {0:?} already consumed")]
    AlreadyConsumed(RegisterId),
    
    #[error("invalid resource: {0:?}")]
    InvalidResource(ResourceId),
    
    #[error("resource already consumed: {0:?}")]
    ResourceAlreadyConsumed(ResourceId),
    
    #[error("type mismatch: expected {expected:?}, found {found:?}")]
    TypeMismatch {
        expected: Box<TypeInner>,
        found: Box<TypeInner>,
    },
    
    #[error("call stack overflow: maximum depth exceeded")]
    CallStackOverflow,
    
    #[error("call stack underflow: no return address available")]
    CallStackUnderflow,
    
    #[error("feature not implemented")]
    NotImplemented,
    
    #[error("{0}")]
    Generic(String),
}

//-----------------------------------------------------------------------------
// Reduction Errors
//-----------------------------------------------------------------------------

/// Errors that can occur during reduction
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ReductionError {
    #[error("program counter out of bounds")]
    ProgramCounterOutOfBounds,
    
    #[error("maximum reduction steps exceeded")]
    MaxStepsExceeded,
    
    #[error("arity mismatch: expected {expected}, found {found}")]
    ArityMismatch { expected: usize, found: usize },
    
    #[error("register {0:?} does not contain a function")]
    NotAFunction(RegisterId),
    
    #[error("no matching pattern found")]
    NoMatchingPattern,
    
    #[error("constraint violation")]
    ConstraintViolation,
    
    #[error("not implemented: {0}")]
    NotImplemented(String),
    
    #[error("register not found: {0:?}")]
    RegisterNotFound(RegisterId),
    
    #[error("register already consumed: {0:?}")]
    RegisterAlreadyConsumed(RegisterId),
    
    #[error("register consumption failed: {0:?}")]
    RegisterConsumptionFailed(RegisterId),
    
    #[error("resource already consumed: {0:?}")]
    ResourceAlreadyConsumed(ResourceId),
    
    #[error("resource not found: {0:?}")]
    ResourceNotFound(ResourceId),
    
    #[error("invalid sum tag: {0}")]
    InvalidSumTag(crate::lambda::Symbol),
    
    #[error("register {0:?} does not contain a sum value")]
    NotASum(RegisterId),
    
    #[error("register {0:?} does not contain a resource")]
    NotAResource(RegisterId),
    
    #[error("register {0:?} does not contain a boolean")]
    NotABoolean(RegisterId),
    
    #[error("type mismatch in comparison")]
    TypeMismatch,
    
    #[error("unknown builtin function: {0}")]
    UnknownBuiltin(crate::lambda::Symbol),
    
    #[error("label not found: {0:?}")]
    LabelNotFound(crate::machine::instruction::Label),
    
    #[error("effect precondition failed")]
    EffectPreconditionFailed,
    
    #[error("no witness provider available")]
    NoWitnessProvider,
}

//-----------------------------------------------------------------------------
// Linearity Errors
//-----------------------------------------------------------------------------

/// Errors related to linear type checking
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LinearityError {
    #[error("linear resource used multiple times")]
    MultipleUse,
    
    #[error("linear resource not consumed")]
    NotConsumed,
    
    #[error("affine resource used after drop")]
    UseAfterDrop,
    
    #[error("relevant resource not used")]
    NotUsed,
    
    #[error("cannot copy linear resource")]
    CannotCopy,
    
    #[error("linearity mismatch")]
    LinearityMismatch,
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
        self.map_err(|e| e.into().context(context))
    }
    
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| e.into().context(f()))
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