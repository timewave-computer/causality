// TEL error handling
// Original file: src/tel/error.rs

// Error definitions for the Temporal Effect Language
use std::fmt;
use thiserror::Error;

/// Error types for TEL operations
#[derive(Debug, Error)]
pub enum TelError {
    #[error("Invalid effect: {0}")]
    InvalidEffect(String),
    
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    
    #[error("Domain error: {0}")]
    DomainError(String),
    
    #[error("Resource error: {0}")]
    ResourceError(String),
    
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    
    #[error("Resource access denied: {0}")]
    ResourceAccessDenied(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Compilation error: {0}")]
    CompilationError(String),
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("Zero-knowledge error: {0}")]
    ZkError(String),
    
    #[error("Time error: {0}")]
    TimeError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Adapter not found: {0}")]
    AdapterNotFound(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Resource snapshot not found
    #[error("Resource snapshot not found: {0}")]
    ResourceSnapshotNotFound(String),
    
    /// Resource snapshot error
    #[error("Resource snapshot error: {0}")]
    ResourceSnapshotError(String),
}

/// Result type for TEL operations
pub type TelResult<T> = Result<T, TelError>;

/// Type for identifiers of effects
pub type EffectId = [u8; 32];

/// Status of effect execution
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    /// Effect executed successfully
    Success,
    /// Effect execution failed
    Failed(String),
    /// Effect execution pending
    Pending,
    /// Effect execution partially successful
    PartialSuccess {
        successful_domains: Vec<String>,
        failed_domains: std::collections::HashMap<String, String>,
    },
} 