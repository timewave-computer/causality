// Error handling traits
// Core traits for the error handling system

use std::error::Error;
use crate::{ErrorCode, ErrorDomain, ErrorMessage, BoxError};

/// Core trait that all Causality errors must implement
pub trait CausalityError: Error + Send + Sync + 'static {
    /// Get the unique error code
    fn code(&self) -> ErrorCode;
    
    /// Get the error domain
    fn domain(&self) -> ErrorDomain;
    
    /// Convert to a standard error message format
    fn to_error_message(&self) -> ErrorMessage {
        ErrorMessage {
            code: self.code(),
            domain: self.domain(),
            message: self.to_string(),
            details: None,
        }
    }
    
    /// Get a unique identifier for the error (domain:code)
    fn error_id(&self) -> String {
        format!("{}:{}", self.domain(), self.code())
    }
}

/// Trait for error types that can provide error details
pub trait WithDetails: CausalityError {
    /// Add additional details to the error
    fn with_details(self, details: serde_json::Value) -> Self;
    
    /// Get the error details
    fn details(&self) -> Option<&serde_json::Value>;
}

/// Trait for error types that support retries
pub trait Retryable: CausalityError {
    /// Check if the error is retryable
    fn is_retryable(&self) -> bool;
    
    /// Get the recommended retry delay (if any)
    fn retry_after(&self) -> Option<std::time::Duration>;
    
    /// Create a non-retryable version of this error
    fn non_retryable(self) -> Self;
}

/// Trait for error sources in the Causality system
pub trait ErrorSource {
    /// Create an error with a specific code
    fn error(&self, code: ErrorCode, message: impl Into<String>) -> BoxError;
    
    /// Create an error with a specific domain and code
    fn error_with_domain(&self, domain: ErrorDomain, code: ErrorCode, message: impl Into<String>) -> BoxError;
}

/// Default error source that can be used anywhere
pub struct DefaultErrorSource;

impl ErrorSource for DefaultErrorSource {
    fn error(&self, code: ErrorCode, message: impl Into<String>) -> BoxError {
        self.error_with_domain(ErrorDomain::Core, code, message)
    }
    
    fn error_with_domain(&self, domain: ErrorDomain, code: ErrorCode, message: impl Into<String>) -> BoxError {
        Box::new(crate::custom_error::CustomError::new(domain, code, message))
    }
}

/// Global error source that can be used in any context
pub fn global_error_source() -> DefaultErrorSource {
    DefaultErrorSource
} 