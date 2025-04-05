// Error conversion utilities
//
// This module contains utilities for handling error conversions between 
// different error types in the system.

use std::fmt::{Debug, Display};
use causality_error::{EngineError, CausalityError};
use std::ops::{Deref, DerefMut};
 // Import Any for downcasting

/// Extension trait for converting from Box<dyn CausalityError> to EngineError
pub trait BoxErrorConversions {
    /// Convert a boxed error to an EngineError
    fn to_engine_error(self) -> EngineError;
}

impl BoxErrorConversions for Box<dyn CausalityError> {
    fn to_engine_error(self) -> EngineError {
        // Try to display the error as a string
        EngineError::execution_failed(format!("Error: {}", self))
    }
}

/// A wrapper for Box<dyn CausalityError> to avoid orphan rule violations
pub struct BoxedCausalityError(pub Box<dyn CausalityError>);

impl From<BoxedCausalityError> for EngineError {
    fn from(error: BoxedCausalityError) -> Self {
        EngineError::execution_failed(format!("Causality error: {}", error.0))
    }
}

impl From<Box<dyn CausalityError>> for BoxedCausalityError {
    fn from(error: Box<dyn CausalityError>) -> Self {
        Self(error)
    }
}

impl Deref for BoxedCausalityError {
    type Target = Box<dyn CausalityError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BoxedCausalityError {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A wrapper for double boxed errors to avoid orphan rule violations
pub struct DoubleBoxedCausalityError(pub Box<Box<dyn CausalityError>>);

impl From<DoubleBoxedCausalityError> for EngineError {
    fn from(error: DoubleBoxedCausalityError) -> Self {
        // Safe to use string representation for double boxed errors
        EngineError::execution_failed(format!("Double-boxed causality error: {}", error.0))
    }
}

impl From<Box<Box<dyn CausalityError>>> for DoubleBoxedCausalityError {
    fn from(error: Box<Box<dyn CausalityError>>) -> Self {
        Self(error)
    }
}

/// A wrapper for serde_json::Error to avoid orphan rule violations
pub struct SerdeJsonError(pub serde_json::Error);

impl From<SerdeJsonError> for EngineError {
    fn from(error: SerdeJsonError) -> Self {
        EngineError::execution_failed(format!("JSON serialization error: {}", error.0))
    }
}

impl From<serde_json::Error> for SerdeJsonError {
    fn from(error: serde_json::Error) -> Self {
        Self(error)
    }
}

/// Wrapper for IO errors
#[derive(Debug)]
pub struct IoError(pub std::io::Error);

/// Helper function to convert IO errors to EngineError
pub fn io_error_to_engine_error(err: std::io::Error) -> EngineError {
    EngineError::execution_failed(format!("IO error: {}", err))
}

/// Helper function to convert any error with Display to EngineError
pub fn display_error_to_engine_error<E: Display>(err: E) -> EngineError {
    EngineError::execution_failed(format!("Error: {}", err))
}

/// Convert a CausalityError to an EngineError
pub fn causality_error_to_engine_error(err: Box<dyn CausalityError>) -> EngineError {
    // For boxed causality errors, use the string representation
    EngineError::execution_failed(format!("Causality error: {}", err))
}

/// Helper function to convert Box<dyn CausalityError> to EngineError if possible
/// Uses Any for downcasting
pub fn convert_boxed_error(err: Box<dyn CausalityError>) -> EngineError {
    // Use CausalityError::as_any() to get &dyn Any, then call downcast_ref
    if let Some(engine_err) = err.as_any().downcast_ref::<EngineError>() {
        engine_err.clone() // Clone the error if downcast succeeds
    } else {
        // Fallback or wrap if it's not an EngineError
        EngineError::InternalError(format!("Failed to downcast CausalityError to EngineError: {}", err))
    }
}

/// Helper function to convert Box<Box<dyn CausalityError>> to EngineError
pub fn convert_double_boxed_error(err: Box<Box<dyn CausalityError>>) -> EngineError {
    // For double boxed causality errors, use the string representation 
    EngineError::execution_failed(format!("Double-boxed causality error: {}", err))
}

/// Helper function to convert any error to Box<dyn CausalityError>
pub fn to_boxed_causality_error<E: Display + Debug + Send + Sync + 'static>(err: E) -> Box<dyn CausalityError> {
    // Convert to EngineError first, which implements CausalityError
    let engine_err = EngineError::execution_failed(format!("Error: {}", err));
    Box::new(engine_err)
}

// Implement From for IoError wrapper to avoid orphan rule violations
impl From<IoError> for EngineError {
    fn from(err: IoError) -> Self {
        io_error_to_engine_error(err.0)
    }
}

/// Convert an error to a causality error string safely
pub fn error_to_string<E: Display>(err: E) -> String {
    err.to_string()
}

/// Shorthand to convert any error to an EngineError
pub fn to_engine_error<E: Display + Debug + Send + Sync + 'static>(err: E) -> EngineError {
    display_error_to_engine_error(err)
}

/// Convert a serde_json error to an EngineError
pub fn serde_error_to_engine_error(err: serde_json::Error) -> EngineError {
    SerdeJsonError(err).into()
}

/// Convert SerdeJsonError to EngineError
pub fn convert_serde_error(err: serde_json::Error) -> EngineError {
    SerdeJsonError(err).into()
}

// You might need a similar function to convert EngineError back to Box<dyn CausalityError>
// for functions that specifically require the boxed trait object.
pub fn box_engine_error(err: EngineError) -> Box<dyn CausalityError> {
    Box::new(err)
} 