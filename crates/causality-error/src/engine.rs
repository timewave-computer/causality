// Engine-specific error types
// These errors are specifically for the causality-engine crate

use thiserror::Error;
use crate::CausalityError;
use std::any::Any;

/// Engine-specific error codes
pub mod codes {
    use crate::ErrorCode;
    
    // Engine error codes start with 5000
    pub const HANDLER_NOT_FOUND: ErrorCode = ErrorCode(5001);
    pub const INVALID_INVOCATION: ErrorCode = ErrorCode(5002);
    pub const EXECUTION_FAILED: ErrorCode = ErrorCode(5003);
    pub const CONTEXT_ERROR: ErrorCode = ErrorCode(5004);
    pub const REGISTRY_ERROR: ErrorCode = ErrorCode(5005);
    pub const LOG_ERROR: ErrorCode = ErrorCode(5006);
    pub const PATTERN_ERROR: ErrorCode = ErrorCode(5007);
    pub const STORAGE_ERROR: ErrorCode = ErrorCode(5008);
    pub const SEGMENT_ERROR: ErrorCode = ErrorCode(5009);
    pub const SYNC_ERROR: ErrorCode = ErrorCode(5010);
    pub const CAPABILITY_ERROR: ErrorCode = ErrorCode(5011);
    pub const CALLBACK_ERROR: ErrorCode = ErrorCode(5012);
}

/// Engine-specific error types
#[derive(Error, Debug, Clone)]
pub enum EngineError {
    /// Handler not found
    #[error("Handler not found: {0}")]
    HandlerNotFound(String),
    
    /// Invalid invocation parameters
    #[error("Invalid invocation: {0}")]
    InvalidInvocation(String),
    
    /// Execution failure
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    /// Execution timeout
    #[error("Execution timed out: {0}")]
    ExecutionTimeout(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
    
    /// Invocation context error
    #[error("Context error: {0}")]
    ContextError(String),
    
    /// Registry operation error
    #[error("Registry error: {0}")]
    RegistryError(String),
    
    /// Log operation error
    #[error("Log error: {0}")]
    LogError(String),
    
    /// Invocation pattern error
    #[error("Pattern error: {0}")]
    PatternError(String),
    
    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),
    
    /// Segment error
    #[error("Segment error: {0}")]
    SegmentError(String),
    
    /// Sync error
    #[error("Sync error: {0}")]
    SyncError(String),
    
    /// Capability error
    #[error("Capability error: {0}")]
    CapabilityError(String),

    /// Callback error
    #[error("Callback error: {0}")]
    CallbackError(String),

    /// Serialization failed
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    
    /// Deserialization failed
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
    
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),
    
    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    /// Not found
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Other errors (general purpose)
    #[error("Other error: {0}")]
    Other(String),
}

impl CausalityError for EngineError {
    fn error_code(&self) -> &'static str {
        
        match self {
            EngineError::HandlerNotFound(_) => "ENGINE_HANDLER_NOT_FOUND",
            EngineError::InvalidInvocation(_) => "ENGINE_INVALID_INVOCATION",
            EngineError::ExecutionFailed(_) => "ENGINE_EXECUTION_FAILED",
            EngineError::ExecutionTimeout(_) => "ENGINE_EXECUTION_TIMEOUT",
            EngineError::InternalError(_) => "ENGINE_INTERNAL_ERROR",
            EngineError::ContextError(_) => "ENGINE_CONTEXT_ERROR",
            EngineError::RegistryError(_) => "ENGINE_REGISTRY_ERROR",
            EngineError::LogError(_) => "ENGINE_LOG_ERROR",
            EngineError::PatternError(_) => "ENGINE_PATTERN_ERROR",
            EngineError::StorageError(_) => "ENGINE_STORAGE_ERROR",
            EngineError::SegmentError(_) => "ENGINE_SEGMENT_ERROR",
            EngineError::SyncError(_) => "ENGINE_SYNC_ERROR",
            EngineError::CapabilityError(_) => "ENGINE_CAPABILITY_ERROR",
            EngineError::CallbackError(_) => "ENGINE_CALLBACK_ERROR",
            EngineError::SerializationFailed(_) => "ENGINE_SERIALIZATION_FAILED",
            EngineError::DeserializationFailed(_) => "ENGINE_DESERIALIZATION_FAILED",
            EngineError::IoError(_) => "ENGINE_IO_ERROR",
            EngineError::InvalidArgument(_) => "ENGINE_INVALID_ARGUMENT",
            EngineError::NotFound(_) => "ENGINE_NOT_FOUND",
            EngineError::ValidationError(_) => "ENGINE_VALIDATION_ERROR",
            EngineError::Other(_) => "ENGINE_OTHER",
        }
    }

    fn as_any(&self) -> &dyn Any { self }
}

/// Convenient Result type for engine operations
pub type EngineResult<T> = Result<T, EngineError>;

/// Convert from engine error to boxed error
impl From<EngineError> for Box<dyn CausalityError> {
    fn from(err: EngineError) -> Self {
        Box::new(err)
    }
}

/// Example conversion from storage error to engine error
impl From<crate::StorageError> for EngineError {
    fn from(err: crate::StorageError) -> Self {
        EngineError::StorageError(err.to_string())
    }
}

/// Convert from io::Error to EngineError
impl From<std::io::Error> for EngineError {
    fn from(err: std::io::Error) -> Self {
        EngineError::IoError(err.to_string())
    }
}

// Example implementations of error factory functions
impl EngineError {
    /// Create a new handler not found error
    pub fn handler_not_found(handler_id: impl Into<String>) -> Self {
        EngineError::HandlerNotFound(handler_id.into())
    }
    
    /// Create a new invalid invocation error
    pub fn invalid_invocation(message: impl Into<String>) -> Self {
        EngineError::InvalidInvocation(message.into())
    }
    
    /// Create a new execution failed error
    pub fn execution_failed(message: impl Into<String>) -> Self {
        EngineError::ExecutionFailed(message.into())
    }
    
    /// Create a new callback error
    pub fn callback_error(message: impl Into<String>) -> Self {
        EngineError::CallbackError(message.into())
    }
} 