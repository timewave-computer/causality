// Core Error Types
//
// This module provides a unified error handling approach for the core crate.
// It defines common error types and result aliases to ensure consistency
// across the codebase.

use std::fmt;
use std::error::Error as StdError;
use thiserror::Error;
use serde::{Serialize, Deserialize};

use crate::resource::types::{ResourceId, ResourceType};
use crate::resource::interface::StateTransitionError;
use crate::crypto::CryptoError;
use crate::smt::SmtError;
use crate::resource::ResourceError;

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
    
    /// Time-related errors
    #[error("Time error: {0}")]
    TimeError(String),
    
    /// Serialization-related errors
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Unknown errors
    #[error("Unknown error: {0}")]
    Unknown(String),

    /// Crypto error
    #[error("Crypto error: {0}")]
    CryptoError(#[from] CryptoError),

    /// SMT error
    #[error("SMT error: {0}")]
    SmtError(#[from] SmtError),

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
}

/// Convenient type alias for results that may error with our CoreError
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a new time error
    pub fn time(msg: impl Into<String>) -> Self {
        Self::TimeError(msg.into())
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
}

/// Convert from any error that implements std::error::Error
impl<E> From<E> for Error 
where
    E: StdError + Send + Sync + 'static
{
    fn from(err: E) -> Self {
        Self::Unknown(err.to_string())
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
            Self::Unknown(msg)
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
            Self::Unknown(msg)
        })
    }
}

/// Error types for resource operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ResourceError {
    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(ResourceId),
    
    /// Resource already exists
    #[error("Resource already exists: {0}")]
    AlreadyExists(ResourceId),
    
    /// Resource type mismatch
    #[error("Resource type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: ResourceType,
        actual: ResourceType,
    },
    
    /// Invalid state transition
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Content addressing error
    #[error("Content addressing error: {0}")]
    ContentAddressingError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Resource Registry error
    #[error("Resource registry error: {0}")]
    RegistryError(String),
    
    /// Reference error
    #[error("Reference error: {0}")]
    ReferenceError(String),
    
    /// Store error
    #[error("Store error: {0}")]
    StoreError(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<StateTransitionError> for ResourceError {
    fn from(err: StateTransitionError) -> Self {
        ResourceError::InvalidStateTransition(err.to_string())
    }
}

/// Result type for resource operations
pub type ResourceResult<T> = std::result::Result<T, ResourceError>; 