//! Runtime error types and handling

use thiserror::Error;

/// Runtime execution errors
#[derive(Error, Debug, Clone)]
pub enum RuntimeError {
    #[error("Effect execution failed: {message}")]
    ExecutionFailed { message: String },
    
    #[error("Handler error: {message}")]
    HandlerError { message: String },
    
    #[error("Resource error: {message}")]
    ResourceError { message: String },
    
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
    
    #[error("Linearity violation: {message}")]
    LinearityViolation { message: String },
    
    #[error("Effect not handled: {effect_type}")]
    UnhandledEffect { effect_type: String },
    
    #[error("Machine error: {0}")]
    MachineError(#[from] causality_core::machine::MachineError),
    
    #[error("Register error: {0}")]
    RegisterError(String),
    
    #[error("Memory error: {0}")]
    MemoryError(String),
    
    #[error("Internal error: {message}")]
    Internal { message: String },
}

/// Result type for runtime operations
pub type RuntimeResult<T> = Result<T, RuntimeError>;

impl RuntimeError {
    /// Create an execution failure error
    pub fn execution_failed(message: impl Into<String>) -> Self {
        Self::ExecutionFailed { message: message.into() }
    }
    
    /// Create a handler error
    pub fn handler_error(message: impl Into<String>) -> Self {
        Self::HandlerError { message: message.into() }
    }
    
    /// Create a resource error
    pub fn resource_error(message: impl Into<String>) -> Self {
        Self::ResourceError { message: message.into() }
    }
    
    /// Create a type mismatch error
    pub fn type_mismatch(message: impl Into<String>) -> Self {
        Self::TypeMismatch(message.into())
    }
    
    /// Create a linearity violation error
    pub fn linearity_violation(message: impl Into<String>) -> Self {
        Self::LinearityViolation { message: message.into() }
    }
    
    /// Create an unhandled effect error
    pub fn unhandled_effect(effect_type: impl Into<String>) -> Self {
        Self::UnhandledEffect { effect_type: effect_type.into() }
    }
    
    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal { message: message.into() }
    }
    
    /// Create a register not found error
    pub fn register_not_found(register_id: causality_core::machine::RegisterId) -> Self {
        Self::RegisterError(format!("Register {:?} not found", register_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let err = RuntimeError::execution_failed("test message");
        assert!(matches!(err, RuntimeError::ExecutionFailed { .. }));
    }
    
    #[test]
    fn test_error_conversion() {
        // Test that we can create machine errors
        let machine_err = causality_core::machine::MachineError::InvalidRegister(
            causality_core::machine::RegisterId::new(0)
        );
        let runtime_err = RuntimeError::from(machine_err);
        assert!(matches!(runtime_err, RuntimeError::MachineError(_)));
    }
    
    #[test]
    fn test_result_type() {
        let success: RuntimeResult<i32> = Ok(42);
        assert_eq!(success.unwrap(), 42);
        
        let failure: RuntimeResult<i32> = Err(RuntimeError::execution_failed("test"));
        assert!(failure.is_err());
    }
} 