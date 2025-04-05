// Storage-specific error types
// These errors are specifically for the causality-storage crate

use thiserror::Error;
use crate::CausalityError;
use std::any::Any;

/// Storage-specific error codes
pub mod codes {
    use crate::ErrorCode;
    
    // Storage error codes start with 4000
    pub const DATABASE_ERROR: ErrorCode = ErrorCode(4001);
    pub const KEY_NOT_FOUND: ErrorCode = ErrorCode(4002);
    pub const SERIALIZATION_ERROR: ErrorCode = ErrorCode(4003);
    pub const TRANSACTION_ERROR: ErrorCode = ErrorCode(4004);
    pub const CONNECTION_ERROR: ErrorCode = ErrorCode(4005);
    pub const SCHEMA_ERROR: ErrorCode = ErrorCode(4006);
    pub const CONSTRAINT_ERROR: ErrorCode = ErrorCode(4007);
}

/// Storage-specific error types
#[derive(Error, Debug, Clone)]
pub enum StorageError {
    /// Database operation error
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    /// Key not found in storage
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),
    
    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    /// Schema error
    #[error("Schema error: {0}")]
    SchemaError(String),
    
    /// Constraint violation
    #[error("Constraint error: {0}")]
    ConstraintError(String),
}

impl CausalityError for StorageError {
    fn error_code(&self) -> &'static str {
        match self {
            StorageError::DatabaseError(_) => "STORAGE_DATABASE_ERROR",
            StorageError::KeyNotFound(_) => "STORAGE_KEY_NOT_FOUND",
            StorageError::SerializationError(_) => "STORAGE_SERIALIZATION_ERROR",
            StorageError::TransactionError(_) => "STORAGE_TRANSACTION_ERROR",
            StorageError::ConnectionError(_) => "STORAGE_CONNECTION_ERROR",
            StorageError::SchemaError(_) => "STORAGE_SCHEMA_ERROR",
            StorageError::ConstraintError(_) => "STORAGE_CONSTRAINT_ERROR",
        }
    }

    fn as_any(&self) -> &dyn Any { self }
}

/// Convenient Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Convert from storage error to boxed error
impl From<StorageError> for Box<dyn CausalityError> {
    fn from(err: StorageError) -> Self {
        Box::new(err)
    }
}

// Helper methods for creating storage errors
impl StorageError {
    /// Create a new key not found error
    pub fn key_not_found(key: impl Into<String>) -> Self {
        StorageError::KeyNotFound(key.into())
    }
    
    /// Create a new serialization error
    pub fn serialization_error(message: impl Into<String>) -> Self {
        StorageError::SerializationError(message.into())
    }
    
    /// Create a new database error
    pub fn database_error(message: impl Into<String>) -> Self {
        StorageError::DatabaseError(message.into())
    }
} 