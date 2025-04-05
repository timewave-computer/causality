// Error conversion utilities
// Provides tools for converting between different error types

use thiserror::Error;
use crate::{CausalityError, BoxError};
use std::any::Any;

/// Trait for converting any error type to a BoxError
pub trait IntoBoxError {
    /// Convert the error into a BoxError
    fn into_box_error(self) -> BoxError;
}

// Implement for anything that already implements CausalityError
impl<E: CausalityError> IntoBoxError for E {
    fn into_box_error(self) -> BoxError {
        Box::new(self)
    }
}

// Implement for std::io::Error
impl IntoBoxError for std::io::Error {
    fn into_box_error(self) -> BoxError {
        Box::new(ExternalError::Io(self.to_string()))
    }
}

// Implement for string types
impl IntoBoxError for String {
    fn into_box_error(self) -> BoxError {
        Box::new(ExternalError::Other(self))
    }
}

impl IntoBoxError for &str {
    fn into_box_error(self) -> BoxError {
        Box::new(ExternalError::Other(self.to_string()))
    }
}

// Implement for anyhow::Error
impl IntoBoxError for anyhow::Error {
    fn into_box_error(self) -> BoxError {
        Box::new(ExternalError::Other(self.to_string()))
    }
}

/// External error types that come from outside the system
#[derive(Error, Debug)]
pub enum ExternalError {
    /// IO error
    #[error("IO error: {0}")]
    Io(String),
    
    /// Serde error
    #[error("Serialization error: {0}")]
    Serde(String),
    
    /// Other external error
    #[error("{0}")]
    Other(String),
}

impl CausalityError for ExternalError {
    fn error_code(&self) -> &'static str {
        match self {
            ExternalError::Io(_) => "EXTERNAL_IO",
            ExternalError::Serde(_) => "EXTERNAL_SERDE",
            ExternalError::Other(_) => "EXTERNAL_OTHER",
        }
    }

    fn as_any(&self) -> &dyn Any { self }
}

// Implement for serde_json::Error
impl IntoBoxError for serde_json::Error {
    fn into_box_error(self) -> BoxError {
        Box::new(ExternalError::Serde(self.to_string()))
    }
}

// Implement for bincode::Error
#[cfg(feature = "bincode")]
impl IntoBoxError for bincode::Error {
    fn into_box_error(self) -> BoxError {
        Box::new(ExternalError::Serde(self.to_string()))
    }
}

/// Helper function to convert any error to a BoxError
pub fn to_box_error<E: IntoBoxError>(err: E) -> BoxError {
    err.into_box_error()
}

/// Helper function to convert a Result with any error type to a Result with BoxError
pub fn map_error<T, E: IntoBoxError>(result: Result<T, E>) -> Result<T, BoxError> {
    result.map_err(|e| e.into_box_error())
} 