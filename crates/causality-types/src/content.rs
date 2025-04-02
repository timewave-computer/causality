use thiserror::Error;

// Import canonical types from crypto_primitives
use crate::crypto_primitives::HashError;

/// Errors that can occur during content addressing
#[derive(Debug, Error)]
pub enum ContentAddressingError {
    #[error("Invalid content hash format: {0}")]
    InvalidFormat(String),
    
    #[error("Content not found: {0}")]
    NotFound(String),
    
    #[error("Content validation error: {0}")]
    ValidationError(String),
    
    #[error("Hashing error: {0}")]
    HashingError(String),
    
    // Add a variant to wrap HashError from crypto_primitives
    #[error("Underlying hash error: {0}")]
    CryptoHashError(#[from] HashError),
} 