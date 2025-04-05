// Error handling traits
// Core traits for the error handling system

use crate::{BoxError, ErrorCode, ErrorDomain};
use crate::CausalityError;

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