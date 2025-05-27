// Content Addressing Storage Errors

use thiserror::Error;
use crate::crypto_primitives::HashError;
use crate::content_addressing::canonical::CanonicalSerializationError;

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

    /// IO error, often from the underlying storage backend
    #[error("IO error: {0}")]
    IoError(String),

    /// Serialization or deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Error during content hashing
    #[error("Hash error: {0}")]
    HashError(#[from] HashError),

    /// Error during canonical serialization needed for hashing
    #[error("Canonical serialization error: {0}")]
    CanonicalError(#[from] CanonicalSerializationError),

    /// Configuration error in the storage backend
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// An unspecified internal error occurred
    #[error("Internal storage error: {0}")]
    InternalError(String),
} 