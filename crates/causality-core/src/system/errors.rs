//! Common error types for the Causality framework

use std::fmt;
use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

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

//-----------------------------------------------------------------------------
// Enhanced Error Handling
//-----------------------------------------------------------------------------

/// Logging levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Logger trait
pub trait AsLogger: Send + Sync {
    fn log(&self, level: LogLevel, message: &str);
}

/// Mock logger for testing
#[derive(Debug, Default)]
pub struct MockLogger {
    pub messages: std::sync::Mutex<Vec<(LogLevel, String)>>,
}

impl MockLogger {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn get_messages(&self) -> Vec<(LogLevel, String)> {
        self.messages.lock().unwrap().clone()
    }
}

impl AsLogger for MockLogger {
    fn log(&self, level: LogLevel, message: &str) {
        self.messages.lock().unwrap().push((level, message.to_string()));
    }
}

/// Error categories for classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Validation errors (bad inputs, constraints violated)
    Validation,
    
    /// Resource errors (not found, insufficient, etc.)
    Resource,
    
    /// Network/external boundary errors
    Boundary,
    
    /// Internal system errors
    Internal,
    
    /// Resource not found
    ResourceNotFound,
}

/// Enhanced error metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetadata {
    /// Error category
    pub category: ErrorCategory,
    
    /// Additional context fields
    pub context: BTreeMap<String, String>,
    
    /// Timestamp
    pub timestamp: u64,
}

impl ErrorMetadata {
    pub fn new(category: ErrorCategory) -> Self {
        Self {
            category,
            context: BTreeMap::new(),
            timestamp: crate::system::utils::get_current_time_ms(),
        }
    }
    
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }
}

/// Enhanced contextual error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualError {
    /// Error message
    pub message: String,
    
    /// Error metadata
    pub metadata: ErrorMetadata,
}

impl ContextualError {
    pub fn new(message: impl Into<String>, metadata: ErrorMetadata) -> Self {
        Self {
            message: message.into(),
            metadata,
        }
    }
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({:?})", self.message, self.metadata.category)
    }
}

impl std::error::Error for ContextualError {}

/// Trait for creating errors with context
pub trait AsErrorContext: Send + Sync {
    fn create_error(&self, message: String, metadata: ErrorMetadata) -> ContextualError;
}

/// Default error context implementation
#[derive(Debug, Default)]
pub struct DefaultErrorContext;

impl AsErrorContext for DefaultErrorContext {
    fn create_error(&self, message: String, metadata: ErrorMetadata) -> ContextualError {
        ContextualError::new(message, metadata)
    }
} 