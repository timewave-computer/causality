// Causality Error Handling Framework
// Central location for error types, traits, and handling utilities

use std::fmt;
use thiserror::Error;

// Re-export common error handling tools for convenience
pub use anyhow;
pub use thiserror;

// Module structure
mod traits;
mod conversion;
mod macros;
mod common;

// Include sub-modules
mod domain;
mod storage;
mod crypto;
mod types;
mod engine;

// Re-export core traits and types
pub use traits::CausalityError;
pub use traits::WithDetails;
pub use traits::Retryable;
pub use traits::ErrorSource;
pub use traits::DefaultErrorSource;
pub use traits::global_error_source;

// Re-export conversion utilities
pub use conversion::IntoBoxError;
pub use conversion::ExternalError;
pub use conversion::to_box_error;
pub use conversion::map_error;

// Re-export common error utilities
pub use common::not_found_error;
pub use common::permission_denied_error;
pub use common::validation_error;
pub use common::io_error;
pub use common::timeout_error;
pub use common::unsupported_error;
pub use common::internal_error;
pub use common::has_error_code;
pub use common::is_error_from_domain;

// Re-export error types from submodules
pub use domain::DomainError;
pub use domain::DomainResult;
pub use storage::StorageError;
pub use storage::StorageResult;
pub use crypto::CryptoError;
pub use crypto::CryptoResult;
pub use types::TypesError;
pub use types::TypesResult;
pub use engine::EngineError;
pub use engine::EngineResult;

/// Error domains representing different components of the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ErrorDomain {
    /// Core domain (system-wide)
    Core,
    /// Type system
    Types,
    /// Crypto operations
    Crypto,
    /// Storage operations
    Storage,
    /// Engine operations
    Engine,
    /// Network operations
    Network,
    /// Domain-specific (blockchain domains)
    Domain,
    /// External services
    External,
}

impl fmt::Display for ErrorDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorDomain::Core => write!(f, "core"),
            ErrorDomain::Types => write!(f, "types"),
            ErrorDomain::Crypto => write!(f, "crypto"),
            ErrorDomain::Storage => write!(f, "storage"),
            ErrorDomain::Engine => write!(f, "engine"),
            ErrorDomain::Network => write!(f, "network"),
            ErrorDomain::Domain => write!(f, "domain"),
            ErrorDomain::External => write!(f, "external"),
        }
    }
}

/// Error code structure for categorizing errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ErrorCode(pub u32);

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}", self.0)
    }
}

/// Standard error message format for serialization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorMessage {
    /// Error code
    pub code: ErrorCode,
    /// Error domain
    pub domain: ErrorDomain,
    /// Human-readable message
    pub message: String,
    /// Additional error details (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// The base error type for common system-wide errors
#[derive(Error, Debug, Clone)]
pub enum CommonError {
    /// Not found error
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    /// Permission error
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Validation error
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),
    
    /// Unsupported operation
    #[error("Operation not supported: {0}")]
    Unsupported(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl CausalityError for CommonError {
    fn code(&self) -> ErrorCode {
        match self {
            CommonError::NotFound(_) => ErrorCode(1001),
            CommonError::PermissionDenied(_) => ErrorCode(1002),
            CommonError::ValidationFailed(_) => ErrorCode(1003),
            CommonError::IoError(_) => ErrorCode(1004),
            CommonError::SerializationError(_) => ErrorCode(1005),
            CommonError::Timeout(_) => ErrorCode(1006),
            CommonError::Unsupported(_) => ErrorCode(1007),
            CommonError::Internal(_) => ErrorCode(1008),
        }
    }
    
    fn domain(&self) -> ErrorDomain {
        ErrorDomain::Core
    }
}

/// Standard Result type for Causality operations
pub type Result<T> = std::result::Result<T, BoxError>;

/// Shorthand for a boxed CausalityError
pub type BoxError = Box<dyn CausalityError>;

/// Custom error type for extension
pub mod custom_error {
    use super::*;
    
    /// A custom error type that can be created dynamically
    #[derive(Debug, Clone)]
    pub struct CustomError {
        domain: ErrorDomain,
        code: ErrorCode,
        message: String,
        details: Option<serde_json::Value>,
    }
    
    impl CustomError {
        /// Create a new custom error
        pub fn new(domain: ErrorDomain, code: ErrorCode, message: impl Into<String>) -> Self {
            Self {
                domain,
                code,
                message: message.into(),
                details: None,
            }
        }
        
        /// Add details to the error
        pub fn with_details(mut self, details: serde_json::Value) -> Self {
            self.details = Some(details);
            self
        }
    }
    
    impl fmt::Display for CustomError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    
    impl std::error::Error for CustomError {}
    
    impl CausalityError for CustomError {
        fn code(&self) -> ErrorCode {
            self.code
        }
        
        fn domain(&self) -> ErrorDomain {
            self.domain
        }
        
        fn to_error_message(&self) -> ErrorMessage {
            ErrorMessage {
                code: self.code,
                domain: self.domain,
                message: self.message.clone(),
                details: self.details.clone(),
            }
        }
    }
    
    impl WithDetails for CustomError {
        fn with_details(mut self, details: serde_json::Value) -> Self {
            self.details = Some(details);
            self
        }
        
        fn details(&self) -> Option<&serde_json::Value> {
            self.details.as_ref()
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    /// This is a test error used for demos
    #[error("Test error: {0}")]
    TestError(String),

    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Already exists error
    #[error("Already exists: {0}")]
    AlreadyExists(String),

    /// Permission denied error
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Unauthorized error
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Invalid state error
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Timeout error
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Busy error
    #[error("Busy: {0}")]
    Busy(String),

    /// Unavailable error
    #[error("Unavailable: {0}")]
    Unavailable(String),

    /// Data corruption error
    #[error("Data corruption: {0}")]
    DataCorruption(String),
    
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Unknown error
    #[error("Unknown error: {0}")]
    Unknown(String),
} 