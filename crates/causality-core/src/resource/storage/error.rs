use thiserror::Error;
use causality_types::crypto_primitives::HashError;
use causality_types::content_addressing::canonical::CanonicalSerializationError;

/// Error type for content-addressed storage operations
#[derive(Debug, Error)]
pub enum ContentAddressedStorageError {
    /// Object not found in storage
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
    
    /// Hash error
    #[error("Hash error: {0}")]
    HashError(#[from] HashError),
    
    /// Canonical serialization error
    #[error("Canonical serialization error: {0}")]
    CanonicalError(#[from] CanonicalSerializationError),
} 