//! Error Handling System
//!
//! This module provides a unified error handling system for the Causality framework.
//! It combines structured errors (using thiserror) with flexible error propagation
//! (using anyhow) to create a consistent error handling approach across the codebase.
//!
//! The system uses categorized errors to help classify issues by their source or nature,
//! which aids in routing, logging, and handling errors appropriately throughout the
//! execution flow.

//-----------------------------------------------------------------------------
// Import
//-----------------------------------------------------------------------------

use anyhow::anyhow;
use anyhow::Error as AnyhowError;
use std::fmt;
use std::sync::Arc;
use thiserror::Error;

use crate::primitive::string::Str;
use crate::serialization::{Decode, DecodeError, Encode, SimpleSerialize};

//-----------------------------------------------------------------------------
// Error Categorie
//-----------------------------------------------------------------------------

/// Categories of errors in the Causality system.
///
/// These categories help classify errors by their source or nature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// General error category for common errors
    General,

    /// Errors related to serialization/deserialization processes
    Serialization,

    /// Errors related to expression evaluation or interpretation
    Expression,

    /// Errors related to type checking or type conversion
    Type,

    /// Errors related to resource handling or validation
    Resource,

    /// Errors related to storage operations (read/write/query)
    Storage,

    /// Errors related to registry operations (registration/lookup)
    Registry,

    /// Errors related to state transitions and validation
    StateTransition,

    /// Errors related to resource not found
    ResourceNotFound,

    /// Errors related to network operations or connectivity
    Network,

    /// Errors related to authentication processes
    Authentication,

    /// Errors related to authorization and permissions
    Authorization,

    /// Errors related to validation processes
    Validation,

    /// Errors related to system-level operations
    System,

    /// Errors related to time handling or temporal operations
    Time,

    /// Errors related to cross-domain messaging.
    Messaging,

    /// Errors during effect handling or execution.
    EffectHandling,

    /// Errors related to coordination protocols.
    Coordination,

    /// Errors during runtime operation processing.
    Runtime,

    /// Errors originating from external systems or boundaries.
    Boundary,

    /// Unknown or unclassified errors
    Unknown,
}

impl Encode for ErrorCategory {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        (*self as u8).as_ssz_bytes()
    }
}

impl Decode for ErrorCategory {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let value = u8::from_ssz_bytes(bytes)?;
        match value {
            0 => Ok(ErrorCategory::General),
            1 => Ok(ErrorCategory::Serialization),
            2 => Ok(ErrorCategory::Expression),
            3 => Ok(ErrorCategory::Type),
            4 => Ok(ErrorCategory::Resource),
            5 => Ok(ErrorCategory::Storage),
            6 => Ok(ErrorCategory::Registry),
            7 => Ok(ErrorCategory::StateTransition),
            8 => Ok(ErrorCategory::ResourceNotFound),
            9 => Ok(ErrorCategory::Network),
            10 => Ok(ErrorCategory::Authentication),
            11 => Ok(ErrorCategory::Authorization),
            12 => Ok(ErrorCategory::Validation),
            13 => Ok(ErrorCategory::System),
            14 => Ok(ErrorCategory::Time),
            15 => Ok(ErrorCategory::Messaging),
            16 => Ok(ErrorCategory::EffectHandling),
            17 => Ok(ErrorCategory::Coordination),
            18 => Ok(ErrorCategory::Runtime),
            19 => Ok(ErrorCategory::Boundary),
            20 => Ok(ErrorCategory::Unknown),
            _ => Err(DecodeError { message: "Invalid value for ErrorCategory".to_string() }),
        }
    }
}

impl SimpleSerialize for ErrorCategory {}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::General => write!(f, "General"),
            ErrorCategory::Serialization => write!(f, "Serialization"),
            ErrorCategory::Expression => write!(f, "Expression"),
            ErrorCategory::Type => write!(f, "Type"),
            ErrorCategory::Resource => write!(f, "Resource"),
            ErrorCategory::Storage => write!(f, "Storage"),
            ErrorCategory::Registry => write!(f, "Registry"),
            ErrorCategory::StateTransition => write!(f, "StateTransition"),
            ErrorCategory::Network => write!(f, "Network"),
            ErrorCategory::Authentication => write!(f, "Authentication"),
            ErrorCategory::Authorization => write!(f, "Authorization"),
            ErrorCategory::Validation => write!(f, "Validation"),
            ErrorCategory::System => write!(f, "System"),
            ErrorCategory::Time => write!(f, "Time"),
            ErrorCategory::Messaging => write!(f, "Messaging"),
            ErrorCategory::EffectHandling => write!(f, "EffectHandling"),
            ErrorCategory::Coordination => write!(f, "Coordination"),
            ErrorCategory::Runtime => write!(f, "Runtime"),
            ErrorCategory::Boundary => write!(f, "Boundary"),
            ErrorCategory::ResourceNotFound => write!(f, "Resource Not Found"),
            ErrorCategory::Unknown => write!(f, "Unknown"),
        }
    }
}

//-----------------------------------------------------------------------------
// Specific Error Type
//-----------------------------------------------------------------------------

/// Represents errors related to resource operations.
#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("Resource not found: {0}")]
    /// Error returned when a requested resource cannot be found
    NotFound(String),
    #[error("Invalid resource state: {0}")]
    /// Error returned when a resource is in an invalid state for the requested operation
    InvalidState(String),
    #[error("Permission denied for resource operation: {0}")]
    /// Error returned when the requester lacks permissions to perform the operation
    PermissionDenied(String),
    #[error("Resource validation failed: {0}")]
    /// Error returned when a resource fails validation checks
    ValidationFailed(String),
    #[error("Resource lock conflict: {0}")]
    /// Error returned when there's a locking conflict preventing the operation
    LockConflict(String),
    #[error("Unknown resource error: {0}")]
    /// Error returned when an unspecified or unexpected error occurs
    Unknown(String),
}

/// Errors specifically related to effect handling and execution, using anyhow.
#[derive(Debug, Error)]
#[error("Effect handling failed: {0}")]
pub struct EffectHandlingError(#[from] AnyhowError);

impl EffectHandlingError {
    /// Create a new EffectHandlingError from a message
    pub fn new<M: Into<String>>(message: M) -> Self {
        EffectHandlingError(anyhow::anyhow!(message.into()))
    }

    /// Create a new EffectHandlingError from an error that implements Error
    pub fn from_error<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
        EffectHandlingError(anyhow::Error::new(err))
    }
}

// Allow conversion from ResourceError into EffectHandlingError

impl From<ResourceError> for EffectHandlingError {
    fn from(err: ResourceError) -> Self {
        EffectHandlingError(AnyhowError::new(err))
    }
}

//-----------------------------------------------------------------------------
// Error Creation Helper
//-----------------------------------------------------------------------------

/// Create an error for a specific error category
pub fn categorized_error(
    category: ErrorCategory,
    message: impl Into<String>,
) -> anyhow::Error {
    anyhow!(format!("[{}] {}", category, message.into()))
}

/// Create a serialization error
pub fn serialization_error(message: impl Into<String>) -> anyhow::Error {
    categorized_error(ErrorCategory::Serialization, message)
}

/// Create an expression error
pub fn expr_error(message: impl Into<String>) -> anyhow::Error {
    categorized_error(ErrorCategory::Expression, message)
}

/// Create a type error
pub fn type_error(message: impl Into<String>) -> anyhow::Error {
    categorized_error(ErrorCategory::Type, message)
}

/// Create a resource error
pub fn resource_error(message: impl Into<String>) -> anyhow::Error {
    categorized_error(ErrorCategory::Resource, message)
}

/// Create a storage error
pub fn storage_error(message: impl Into<String>) -> anyhow::Error {
    categorized_error(ErrorCategory::Storage, message)
}

/// Create a registry error
pub fn registry_error(message: impl Into<String>) -> anyhow::Error {
    categorized_error(ErrorCategory::Registry, message)
}

/// Create a validation error
pub fn validation_error(message: impl Into<String>) -> anyhow::Error {
    categorized_error(ErrorCategory::Validation, message)
}

/// Create a time error
pub fn time_error(message: impl Into<String>) -> anyhow::Error {
    categorized_error(ErrorCategory::Time, message)
}

/// Create a messaging error
pub fn messaging_error(message: impl Into<String>) -> anyhow::Error {
    categorized_error(ErrorCategory::Messaging, message)
}

/// Create an effect handling error (generic anyhow version)
pub fn effect_handling_anyhow_error(message: impl Into<String>) -> anyhow::Error {
    categorized_error(ErrorCategory::EffectHandling, message)
}

/// Create a coordination error
pub fn coordination_error(message: impl Into<String>) -> anyhow::Error {
    categorized_error(ErrorCategory::Coordination, message)
}

/// Create a runtime error
pub fn runtime_error(message: impl Into<String>) -> anyhow::Error {
    categorized_error(ErrorCategory::Runtime, message)
}

/// Create a boundary error
pub fn boundary_error(message: impl Into<String>) -> anyhow::Error {
    categorized_error(ErrorCategory::Boundary, message)
}

/// Causal error type
///
/// A strongly-typed error format that includes:
/// - Error code (enum variant)
/// - Message
/// - Context
/// - Stack trace information
///
/// All error types in the Causality framework should derive
/// from this type for consistent error handling.
#[derive(Debug, Clone, PartialEq)]
pub struct CausalError {
    /// Error code (enum variant as a string)
    pub code: Str,
    /// Error message
    pub message: Str,
    /// Optional context
    pub context: Option<ErrorContext>,
}

impl Encode for CausalError {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.code.as_ssz_bytes());
        bytes.extend(self.message.as_ssz_bytes());
        bytes.extend(self.context.as_ssz_bytes());
        bytes
    }
}

impl Decode for CausalError {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode code (variable-length Str)
        let code = Str::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode code: {}", e) })?;
        
        // Calculate bytes consumed by code
        let code_bytes = code.as_ssz_bytes();
        offset += code_bytes.len();
        
        // Decode message (variable-length Str)
        let message = Str::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode message: {}", e) })?;
        
        // Calculate bytes consumed by message
        let message_bytes = message.as_ssz_bytes();
        offset += message_bytes.len();
        
        // Decode context (Option<ErrorContext>)
        let context = Option::<ErrorContext>::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode context: {}", e) })?;
        
        Ok(CausalError {
            code,
            message,
            context,
        })
    }
}

impl SimpleSerialize for CausalError {}

/// Error context for additional metadata
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorContext {
    /// Key for the context
    pub key: Str,
    /// Value for the context
    pub value: Str,
}

impl Encode for ErrorContext {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.key.as_ssz_bytes());
        bytes.extend(self.value.as_ssz_bytes());
        bytes
    }
}

impl Decode for ErrorContext {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        
        // Decode key (variable-length Str)
        let key = Str::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode key: {}", e) })?;
        
        // Calculate bytes consumed by key
        let key_bytes = key.as_ssz_bytes();
        offset += key_bytes.len();
        
        // Decode value (variable-length Str)
        let value = Str::from_ssz_bytes(&bytes[offset..])
            .map_err(|e| DecodeError { message: format!("Failed to decode value: {}", e) })?;
        
        Ok(ErrorContext { key, value })
    }
}

impl SimpleSerialize for ErrorContext {}

impl ErrorContext {
    /// Create a new error context
    pub fn new(key: impl Into<Str>, value: impl Into<Str>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl CausalError {
    /// Create a new CausalError
    pub fn new(code: impl Into<Str>, message: impl Into<Str>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            context: None,
        }
    }

    /// Set the context for the error
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Create a context with key and value directly
    pub fn with_key_value(
        mut self,
        key: impl Into<Str>,
        value: impl Into<Str>,
    ) -> Self {
        self.context = Some(ErrorContext::new(key, value));
        self
    }
}

impl fmt::Display for CausalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error {}: {}", self.code, self.message)?;
        if let Some(context) = &self.context {
            write!(f, " [{}: {}]", context.key, context.value)?;
        }
        Ok(())
    }
}

impl std::error::Error for CausalError {}

/// Generic error enum with context
///
/// This type provides a way to wrap existing errors with a context.
#[derive(Debug, Clone, PartialEq)]
pub enum GenericError {
    /// Wrapped error with a context string
    WithContext(Str, Box<GenericError>),
    /// Basic error with a message
    Message(Str),
}

impl Encode for GenericError {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            GenericError::WithContext(ctx, err) => {
                bytes.push(0u8); // Variant tag
                bytes.extend(ctx.as_ssz_bytes());
                bytes.extend(err.as_ssz_bytes());
            }
            GenericError::Message(msg) => {
                bytes.push(1u8); // Variant tag
                bytes.extend(msg.as_ssz_bytes());
            }
        }
        bytes
    }
}

impl Decode for GenericError {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError {
                message: "GenericError requires at least 1 byte for variant tag".to_string(),
            });
        }
        
        let variant = bytes[0];
        let data = &bytes[1..];
        
        match variant {
            0 => {
                // WithContext variant
                let mut offset = 0;
                
                // Decode context Str (variable-length)
                let ctx = Str::from_ssz_bytes(&data[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode context: {}", e) })?;
                
                // Calculate bytes consumed by context
                let ctx_bytes = ctx.as_ssz_bytes();
                offset += ctx_bytes.len();
                
                // Decode nested error
                let err = GenericError::from_ssz_bytes(&data[offset..])
                    .map_err(|e| DecodeError { message: format!("Failed to decode nested error: {}", e) })?;
                
                Ok(GenericError::WithContext(ctx, Box::new(err)))
            }
            1 => {
                // Message variant
                let msg = Str::from_ssz_bytes(data)
                    .map_err(|e| DecodeError { message: format!("Failed to decode message: {}", e) })?;
                Ok(GenericError::Message(msg))
            }
            _ => Err(DecodeError {
                message: format!("Invalid GenericError variant: {}", variant),
            }),
        }
    }
}

impl SimpleSerialize for GenericError {}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenericError::WithContext(ctx, err) => {
                write!(f, "{}: {}", ctx, err)
            }
            GenericError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for GenericError {}

impl From<String> for GenericError {
    fn from(s: String) -> Self {
        GenericError::Message(Str::from(s))
    }
}

impl From<&str> for GenericError {
    fn from(s: &str) -> Self {
        GenericError::Message(Str::from(s))
    }
}

impl GenericError {
    /// Create a new error with a message
    pub fn new(message: impl Into<Str>) -> Self {
        GenericError::Message(message.into())
    }

    /// Add context to an existing error
    pub fn context(self, context: impl Into<Str>) -> Self {
        GenericError::WithContext(context.into(), Box::new(self))
    }
}

/// Thread-safe shared error type
///
/// This type provides a way to share errors across threads.
/// It wraps a CausalError in an Arc for thread-safe sharing.
#[derive(Debug, Clone)]
pub struct SharedError(Arc<CausalError>);

impl SharedError {
    /// Create a new SharedError
    pub fn new(error: CausalError) -> Self {
        Self(Arc::new(error))
    }

    /// Get a reference to the inner CausalError
    pub fn inner(&self) -> &CausalError {
        &self.0
    }

    /// Convert to a new CausalError instance (cloning the data)
    pub fn to_error(&self) -> CausalError {
        self.0.as_ref().clone()
    }
}

impl fmt::Display for SharedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for SharedError {}
