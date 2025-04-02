//! Error types for the effect runtime system
//!
//! This module defines error types used in the effect runtime system.

use std::fmt::Debug;
use thiserror::Error;

/// Result type for effect operations
pub type EffectResult<T> = Result<T, EffectError>;

/// Error type for effect operations
#[derive(Debug, Error, Clone)]
pub enum EffectError {
    #[error("Missing required capability: {0}")]
    MissingCapability(String),

    #[error("Missing required resource: {0}")]
    MissingResource(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Handler not found for effect type: {0}")]
    HandlerNotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Resource access denied: {0}")]
    ResourceAccessDenied(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Other error: {0}")]
    Other(String),

    #[error("Resource or object not found: {0}")]
    NotFound(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Duplicate registration: {0}")]
    DuplicateRegistration(String),
    
    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
} 