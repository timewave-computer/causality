// Common error utilities and helper functions
// Provides common error handling patterns used across the codebase

use crate::BoxError;

/// Create a "not found" error with the given entity name
pub fn not_found_error(entity: impl Into<String>) -> BoxError {
    let msg = format!("Entity not found: {}", entity.into());
    Box::new(crate::CommonError::NotFound(msg))
}

/// Create a permission denied error with the given reason
pub fn permission_denied_error(reason: impl Into<String>) -> BoxError {
    let msg = reason.into();
    Box::new(crate::CommonError::PermissionDenied(msg))
}

/// Create a validation error with the given reason
pub fn validation_error(reason: impl Into<String>) -> BoxError {
    let msg = reason.into();
    Box::new(crate::CommonError::ValidationFailed(msg))
}

/// Create an IO error with the given reason
pub fn io_error(reason: impl Into<String>) -> BoxError {
    let msg = reason.into();
    Box::new(crate::CommonError::IoError(msg))
}

/// Create a timeout error with the given operation
pub fn timeout_error(operation: impl Into<String>) -> BoxError {
    let msg = format!("Operation timed out: {}", operation.into());
    Box::new(crate::CommonError::Timeout(msg))
}

/// Create an unsupported operation error
pub fn unsupported_error(operation: impl Into<String>) -> BoxError {
    let msg = format!("Operation not supported: {}", operation.into());
    Box::new(crate::CommonError::Unsupported(msg))
}

/// Create an internal error with the given reason
pub fn internal_error(reason: impl Into<String>) -> BoxError {
    let msg = reason.into();
    Box::new(crate::CommonError::Internal(msg))
}

/// Convert a Result with any error type to a Result with BoxError
pub fn to_box_error<T, E>(result: Result<T, E>) -> Result<T, BoxError>
where
    E: std::error::Error + Send + Sync + 'static,
{
    result.map_err(|e| internal_error(e.to_string()))
}

// // Helper function to check if an error is of a specific code
// pub fn has_error_code(error: &BoxError, code: ErrorCode) -> bool {
//     error.code() == code
// }

// // Helper function to check if an error is from a specific domain
// pub fn is_error_from_domain(error: &BoxError, domain: ErrorDomain) -> bool {
//     error.domain() == domain
// }

// Removed has_error_code and is_error_from_domain as they are incompatible
// with the current CausalityError trait (which uses error_code -> &'static str).