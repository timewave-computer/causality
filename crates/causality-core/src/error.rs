// Core Error Types
//
// This module provides a unified error handling approach for the core crate.
// It defines common error types and result aliases to ensure consistency
// across the codebase.

use std::fmt;
use std::error::Error as StdError;
use thiserror::Error;

/// Resource error type
#[derive(Error, Debug)]
pub enum ResourceError {
    /// Invalid state transition
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),
    
    /// Invalid resource
    #[error("Invalid resource: {0}")]
    InvalidResource(String),
    
    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    /// Resource already exists
    #[error("Resource already exists: {0}")]
    AlreadyExists(String),
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Resource access denied
    #[error("Resource access denied: {0}")]
    AccessDenied(String),
    
    /// Invalid resource state
    #[error("Invalid resource state: {0}")]
    InvalidState(String),
    
    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// Other error
    #[error("{0}")]
    Other(String),
}

/// Core error type incorporating all possible error categories
#[derive(Error, Debug)]
pub enum Error {
    /// Resource-related errors
    #[error("Resource error: {0}")]
    ResourceError(#[from] ResourceError),
    
    /// Verification-related errors
    #[error("Verification error: {0}")]
    VerificationError(String),
    
    /// Actor-related errors
    #[error("Actor error: {0}")]
    ActorError(String),
    
    /// Concurrency-related errors
    #[error("Concurrency error: {0}")]
    ConcurrencyError(String),
    
    /// Time-related errors (using CoreTimeError to avoid confusion with time/error.rs TimeError)
    #[error("Time error: {0}")]
    CoreTimeError(#[from] CoreTimeError),
    
    /// Serialization-related errors
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Unknown errors
    #[error("Unknown error: {0}")]
    Unknown(String),

    /// Crypto error
    #[error("Crypto error: {0}")]
    CryptoError(String),

    /// SMT error
    #[error("SMT error: {0}")]
    SmtError(String),

    /// Committee error
    #[error("Committee error: {0}")]
    CommitteeError(String),

    /// Observation error
    #[error("Observation error: {0}")]
    ObservationError(String),

    /// Capability error
    #[error("Capability error: {0}")]
    CapabilityError(String),

    /// ZK error
    #[error("ZK error: {0}")]
    ZkError(String),
    
    /// Effect error
    #[error("Effect error: {0}")]
    EffectError(String),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Generic error
    #[error("{0}")]
    Other(String),
}

/// Convenient type alias for results that may error with our CoreError
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a new time error
    pub fn time(msg: impl Into<String>) -> Self {
        Self::CoreTimeError(CoreTimeError::Other(msg.into()))
    }

    /// Create a new concurrency error
    pub fn concurrency(msg: impl Into<String>) -> Self {
        Self::ConcurrencyError(msg.into())
    }

    /// Create a new verification error
    pub fn verification(msg: impl Into<String>) -> Self {
        Self::VerificationError(msg.into())
    }

    /// Create a new actor error
    pub fn actor(msg: impl Into<String>) -> Self {
        Self::ActorError(msg.into())
    }

    /// Create a new serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::SerializationError(msg.into())
    }
    
    /// Create a new crypto error
    pub fn crypto(msg: impl Into<String>) -> Self {
        Self::CryptoError(msg.into())
    }
    
    /// Create a new smt error
    pub fn smt(msg: impl Into<String>) -> Self {
        Self::SmtError(msg.into())
    }
    
    /// Create a new generic error
    pub fn new(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

/// Context extension trait for Result
pub trait ResultExt<T, E> {
    /// Add context to an error result
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static;
    
    /// Add context to an error result with a closure
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> ResultExt<T, E> for std::result::Result<T, E>
where
    E: StdError + Send + Sync + 'static
{
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static
    {
        self.map_err(|e| {
            let msg = format!("{}: {}", context, e);
            Error::Unknown(msg)
        })
    }
    
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C
    {
        self.map_err(|e| {
            let context = f();
            let msg = format!("{}: {}", context, e);
            Error::Unknown(msg)
        })
    }
}

/// Result type for resource operations
pub type ResourceResult<T> = std::result::Result<T, ResourceError>;

impl ResourceError {
    /// Create a new resource error from a string
    pub fn new(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

/// Time-related errors (basic version for core module)
#[derive(Debug, Error)]
pub enum CoreTimeError {
    /// Invalid timestamp
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
    
    /// Time source error
    #[error("Time source error: {0}")]
    TimeSourceError(String),
    
    /// Interval error
    #[error("Interval error: {0}")]
    IntervalError(String),
    
    /// Other error
    #[error("{0}")]
    Other(String),
}

impl CoreTimeError {
    /// Create a new time error from a string
    pub fn new(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
} 