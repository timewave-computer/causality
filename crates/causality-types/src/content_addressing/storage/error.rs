// Error types for content-addressed storage operations

use thiserror::Error;
use crate::crypto_primitives::HashError;

/// Error type for storage operations
#[derive(Debug, Error)]
pub enum StorageError {
    /// Object not found
    #[error("Object not found: {0}")]
    NotFound(String),
    
    /// Duplicate object in storage
    #[error("Duplicate object: {0}")]
    Duplicate(String),
    
    /// Hash mismatch during verification
    #[error("Hash mismatch: {0}")]
    HashMismatch(String),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    /// Hash error
    #[error("Hash error: {0}")]
    HashError(#[from] HashError),
}

/// Convenience type alias for Results with StorageError
pub type StorageResult<T> = Result<T, StorageError>; 