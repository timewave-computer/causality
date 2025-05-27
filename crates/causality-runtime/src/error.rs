//! Runtime Error Types
//!
//! Error types and result definitions for the causality runtime system.

//-----------------------------------------------------------------------------
// Imports
//-----------------------------------------------------------------------------

use causality_types::core::error::ErrorCategory;
use thiserror::Error;

//-----------------------------------------------------------------------------
// Error Types
//-----------------------------------------------------------------------------

/// Runtime-specific error types
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Error returned when a resource is not found
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Error returned from contexts
    #[error("Context error: {0}")]
    ContextError(String),

    /// Error categorized with additional context
    #[error("Categorized error: {0} - {1}")]
    Categorized(ErrorCategory, String),

    /// Error returned from underlying systems
    #[error("System error: {0}")]
    System(String),

    /// Unknown or uncategorized error
    #[error("Unknown runtime error: {0}")]
    Unknown(String),
}

/// Shorthand for Result with RuntimeError
pub type Result<T> = std::result::Result<T, RuntimeError>;
