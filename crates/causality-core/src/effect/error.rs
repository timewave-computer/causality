// Effect system error types
//
// This module defines error types specific to the effect system.

use thiserror::Error;

/// Effect system errors
#[derive(Debug, Error)]
pub enum EffectSystemError {
    /// Generic error
    #[error("Effect error: {0}")]
    Generic(String),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Runtime error
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

/// Result type for effect system operations
pub type EffectSystemResult<T> = Result<T, EffectSystemError>; 