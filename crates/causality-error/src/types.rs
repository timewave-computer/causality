// Types-specific error types
// These errors are specifically for the causality-types crate

use thiserror::Error;
use crate::CausalityError;
use std::any::Any;

/// Types-specific error codes
pub mod codes {
    use crate::ErrorCode;
    
    // Types error codes start with 2000
    pub const PARSE_ERROR: ErrorCode = ErrorCode(2001);
    pub const CONVERSION_ERROR: ErrorCode = ErrorCode(2002);
    pub const VALIDATION_ERROR: ErrorCode = ErrorCode(2003);
    pub const INCOMPATIBLE_TYPE: ErrorCode = ErrorCode(2004);
    pub const SERIALIZATION_ERROR: ErrorCode = ErrorCode(2005);
    pub const RESOURCE_ERROR: ErrorCode = ErrorCode(2006);
    pub const REGISTER_ERROR: ErrorCode = ErrorCode(2007);
}

/// Types-specific error types
#[derive(Error, Debug, Clone)]
pub enum TypesError {
    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Type conversion error
    #[error("Conversion error: {0}")]
    ConversionError(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Incompatible type error
    #[error("Incompatible type: {0}")]
    IncompatibleType(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Resource error
    #[error("Resource error: {0}")]
    ResourceError(String),
    
    /// Register error
    #[error("Register error: {0}")]
    RegisterError(String),
}

impl CausalityError for TypesError {
    fn error_code(&self) -> &'static str {
        
        match self {
            TypesError::ParseError(_) => "TYPES_PARSE_ERROR",
            TypesError::ConversionError(_) => "TYPES_CONVERSION_ERROR",
            TypesError::ValidationError(_) => "TYPES_VALIDATION_ERROR",
            TypesError::IncompatibleType(_) => "TYPES_INCOMPATIBLE_TYPE",
            TypesError::SerializationError(_) => "TYPES_SERIALIZATION_ERROR",
            TypesError::ResourceError(_) => "TYPES_RESOURCE_ERROR",
            TypesError::RegisterError(_) => "TYPES_REGISTER_ERROR",
        }
    }

    fn as_any(&self) -> &dyn Any { self }
}

/// Convenient Result type for types operations
pub type TypesResult<T> = Result<T, TypesError>;

/// Convert from types error to boxed error
impl From<TypesError> for Box<dyn CausalityError> {
    fn from(err: TypesError) -> Self {
        Box::new(err)
    }
}

// Helper methods for creating types errors
impl TypesError {
    /// Create a new parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        TypesError::ParseError(message.into())
    }
    
    /// Create a new conversion error
    pub fn conversion_error(message: impl Into<String>) -> Self {
        TypesError::ConversionError(message.into())
    }
    
    /// Create a new validation error
    pub fn validation_error(message: impl Into<String>) -> Self {
        TypesError::ValidationError(message.into())
    }
    
    /// Create a new resource error
    pub fn resource_error(message: impl Into<String>) -> Self {
        TypesError::ResourceError(message.into())
    }
} 