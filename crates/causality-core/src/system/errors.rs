//! Common error types for the Causality framework

use std::fmt;

//-----------------------------------------------------------------------------
// Error Types
//-----------------------------------------------------------------------------

/// Core error type for the Causality system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CausalityError {
    /// Serialization/deserialization error
    SerializationError(String),
    
    /// Type system error
    TypeError(String),
    
    /// Linear resource error
    LinearityError(String),
    
    /// Machine execution error
    MachineError(String),
    
    /// Content addressing error
    ContentAddressingError(String),
    
    /// SMT operation error
    SmtError(String),
    
    /// Invalid state error
    InvalidState(String),
    
    /// Resource not found
    ResourceNotFound(String),
    
    /// Generic error with custom message
    Custom(String),
}

//-----------------------------------------------------------------------------
// Trait Implementations
//-----------------------------------------------------------------------------

impl fmt::Display for CausalityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CausalityError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            CausalityError::TypeError(msg) => write!(f, "Type error: {}", msg),
            CausalityError::LinearityError(msg) => write!(f, "Linearity error: {}", msg),
            CausalityError::MachineError(msg) => write!(f, "Machine error: {}", msg),
            CausalityError::ContentAddressingError(msg) => write!(f, "Content addressing error: {}", msg),
            CausalityError::SmtError(msg) => write!(f, "SMT error: {}", msg),
            CausalityError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            CausalityError::ResourceNotFound(id) => write!(f, "Resource not found: {}", id),
            CausalityError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CausalityError {}

/// Result type using CausalityError
pub type Result<T> = std::result::Result<T, CausalityError>;

//-----------------------------------------------------------------------------
// From Implementations
//-----------------------------------------------------------------------------

/// Convert from SSZ decode errors
impl From<ssz::DecodeError> for CausalityError {
    fn from(err: ssz::DecodeError) -> Self {
        CausalityError::SerializationError(format!("SSZ decode error: {:?}", err))
    }
}

/// Convert from hex decode errors  
impl From<hex::FromHexError> for CausalityError {
    fn from(err: hex::FromHexError) -> Self {
        CausalityError::SerializationError(format!("Hex decode error: {:?}", err))
    }
}

/// Convert from UTF-8 errors
impl From<std::string::FromUtf8Error> for CausalityError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        CausalityError::SerializationError(format!("UTF-8 decode error: {:?}", err))
    }
} 