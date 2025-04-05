// Causality Error Handling Framework
// Central location for error types, traits, and handling utilities

use std::fmt;
use thiserror::Error;
use std::any::Any;
use std::error::Error as StdError;

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

// Public exports (Consolidated)
pub use common::*; // Includes helper functions like not_found_error etc.
pub use conversion::{IntoBoxError, ExternalError, to_box_error, map_error}; // Use * or be specific as needed
pub use crypto::{CryptoError, CryptoResult};
pub use domain::{DomainError, DomainResult};
pub use engine::{EngineError, EngineResult};
pub use storage::{StorageError, StorageResult};
pub use traits::{DefaultErrorSource, ErrorSource, Retryable, WithDetails, global_error_source};
pub use types::{TypesError, TypesResult};

/// Error domains representing different components of the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ErrorDomain {
    Core, Types, Crypto, Storage, Engine, Network, Domain, External,
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
    pub code: ErrorCode,
    pub domain: ErrorDomain,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Common error type (part of Core domain)
#[derive(Error, Debug)]
pub enum CommonError {
    #[error("Resource not found: {0}")] NotFound(String),
    #[error("Permission denied: {0}")] PermissionDenied(String),
    #[error("Validation failed: {0}")] ValidationFailed(String),
    #[error("IO error: {0}")] IoError(String),
    #[error("Serialization error: {0}")] SerializationError(String),
    #[error("Operation timed out: {0}")] Timeout(String),
    #[error("Operation not supported: {0}")] Unsupported(String),
    #[error("Internal error: {0}")] Internal(String),
}

// Implement CausalityError for CommonError
impl CausalityError for CommonError {
    fn error_code(&self) -> &'static str {
        match self {
            CommonError::NotFound(_) => "CORE_NOT_FOUND",
            CommonError::PermissionDenied(_) => "CORE_PERMISSION_DENIED",
            CommonError::ValidationFailed(_) => "CORE_VALIDATION_FAILED",
            CommonError::IoError(_) => "CORE_IO_ERROR",
            CommonError::SerializationError(_) => "CORE_SERIALIZATION_ERROR",
            CommonError::Timeout(_) => "CORE_TIMEOUT",
            CommonError::Unsupported(_) => "CORE_UNSUPPORTED",
            CommonError::Internal(_) => "CORE_INTERNAL",
        }
    }
    fn as_any(&self) -> &dyn Any { self }
}

/// Standard Result type using BoxError
pub type Result<T> = std::result::Result<T, BoxError>;
/// Shorthand for a boxed CausalityError
pub type BoxError = Box<dyn CausalityError>;

/// Custom error type definition and impls
pub mod custom_error {
    use super::*; // Use super to get CausalityError, ErrorDomain, etc.

    /// A custom error type that can be created dynamically
    #[derive(Debug, Clone)]
    pub struct CustomError {
        domain: ErrorDomain,
        code: ErrorCode, // Keep internal code/domain for construction
        message: String,
        details: Option<serde_json::Value>,
    }

    impl CustomError {
        pub fn new(domain: ErrorDomain, code: ErrorCode, message: impl Into<String>) -> Self {
            Self { domain, code, message: message.into(), details: None }
        }
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
    impl StdError for CustomError {}

    // Implement CausalityError for CustomError
    impl CausalityError for CustomError {
        fn error_code(&self) -> &'static str {
             match self.domain {
                ErrorDomain::Core => "CUSTOM_CORE_ERROR",
                ErrorDomain::Types => "CUSTOM_TYPES_ERROR",
                ErrorDomain::Crypto => "CUSTOM_CRYPTO_ERROR",
                ErrorDomain::Storage => "CUSTOM_STORAGE_ERROR",
                ErrorDomain::Engine => "CUSTOM_ENGINE_ERROR",
                ErrorDomain::Network => "CUSTOM_NETWORK_ERROR",
                ErrorDomain::Domain => "CUSTOM_DOMAIN_ERROR",
                ErrorDomain::External => "CUSTOM_EXTERNAL_ERROR",
                // Add other domains if necessary or handle generically
            }
        }
        fn as_any(&self) -> &dyn Any { self }
    }

    // Implement WithDetails for CustomError
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

// General Error enum (distinct from CommonError)
#[derive(Debug, Error)]
pub enum Error {
    #[error("Test error: {0}")] TestError(String),
    #[error("Invalid argument: {0}")] InvalidArgument(String),
    #[error("Not found: {0}")] NotFound(String),
    #[error("Already exists: {0}")] AlreadyExists(String),
    #[error("Permission denied: {0}")] PermissionDenied(String),
    #[error("Unauthorized: {0}")] Unauthorized(String),
    #[error("Invalid state: {0}")] InvalidState(String),
    #[error("Timeout: {0}")] Timeout(String),
    #[error("Busy: {0}")] Busy(String),
    #[error("Unavailable: {0}")] Unavailable(String),
    #[error("Data corruption: {0}")] DataCorruption(String),
    #[error("Network error: {0}")] NetworkError(String),
    #[error("Serialization error: {0}")] SerializationError(String),
    #[error("Deserialization error: {0}")] DeserializationError(String),
    #[error("Unknown error: {0}")] Unknown(String),
}

// The one true CausalityError trait definition
/// Base trait for all errors in the Causality system.
pub trait CausalityError: StdError + fmt::Debug + fmt::Display + Send + Sync + Any + 'static {
    /// Returns a unique static string code for this error type.
    fn error_code(&self) -> &'static str;

    /// Provides a brief description of the error (defaults to Display impl).
    fn description(&self) -> String { format!("{}", self) }

    /// Converts the error into a boxed trait object.
    fn into_boxed(self) -> Box<dyn CausalityError> where Self: Sized + CausalityError { Box::new(self) }

    /// Provides context specific to the error (optional).
    fn context(&self) -> Option<String> { None }

    /// Indicates if the error is temporary and retrying might succeed (optional).
    fn is_transient(&self) -> bool { false }

    /// Returns this error as a `&dyn Any` to allow downcasting.
    fn as_any(&self) -> &dyn Any;
} 