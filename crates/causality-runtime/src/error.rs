//! Runtime Error Types
//!
//! This module defines error types used throughout the causality-runtime crate.

use thiserror::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Runtime errors
#[derive(Error, Debug)]
pub enum RuntimeError {
    /// Execution error
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    /// Registry error
    #[error("Registry error: {0}")]
    RegistryError(String),
    
    /// Translator error
    #[error("Translator error: {0}")]
    TranslatorError(String),
    
    /// Resource error
    #[error("Resource error: {0}")]
    ResourceError(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Context error
    #[error("Context error: {0}")]
    ContextError(String),
    
    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
    
    /// External error
    #[error("External error: {0}")]
    ExternalError(String),
}

/// Result type for runtime operations
pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// Error conversion trait for easy conversion to RuntimeError
pub trait IntoRuntimeError {
    /// Convert to RuntimeError
    fn into_runtime_error(self) -> RuntimeError;
}

// Implement conversion from std::io::Error
impl From<std::io::Error> for RuntimeError {
    fn from(err: std::io::Error) -> Self {
        RuntimeError::ExternalError(format!("IO error: {}", err))
    }
}

// Implement conversion from serde_json::Error
impl From<serde_json::Error> for RuntimeError {
    fn from(err: serde_json::Error) -> Self {
        RuntimeError::ExternalError(format!("JSON error: {}", err))
    }
}

// Implement conversion from anyhow::Error
impl From<anyhow::Error> for RuntimeError {
    fn from(err: anyhow::Error) -> Self {
        RuntimeError::ExternalError(format!("Error: {}", err))
    }
}

// Implement conversion from string types
impl From<String> for RuntimeError {
    fn from(s: String) -> Self {
        RuntimeError::InternalError(s)
    }
}

impl From<&str> for RuntimeError {
    fn from(s: &str) -> Self {
        RuntimeError::InternalError(s.to_string())
    }
}

// Implement conversion from causality_core::effect::EffectError
impl From<causality_core::effect::EffectError> for RuntimeError {
    fn from(err: causality_core::effect::EffectError) -> Self {
        RuntimeError::ExecutionError(format!("Effect error: {}", err))
    }
} 