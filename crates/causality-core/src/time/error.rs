// Error types for the time module

use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Represents an error within the time module
#[derive(Debug, Error, Clone, Serialize, Deserialize)] // Added Clone, Serialize, Deserialize if needed elsewhere
pub enum TimeError {
    #[error("Domain not found: {0}")]
    DomainNotFound(String),
    
    #[error("Invalid time value: {0}")]
    InvalidTimeValue(String),
    
    #[error("Synchronization error: {0}")]
    SynchronizationError(String),
    
    #[error("Effect execution error: {0}")]
    EffectError(String), // Changed from #[from] to String to avoid dependency cycles if EffectError uses TimeError
    
    #[error("Internal time error: {0}")]
    InternalError(String),
    
    #[error("Time Map error: {0}")]
    TimeMapError(String),

    #[error("Attestation error: {0}")]
    AttestationError(String),

    #[error("Operation error: {0}")]
    OperationError(String),

    // Add other specific errors as needed
} 