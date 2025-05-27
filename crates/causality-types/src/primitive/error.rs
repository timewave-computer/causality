//! Unified Error Handling System
//!
//! This module provides a comprehensive error handling system for the Causality framework,
//! combining categorized errors, rich context information, and ZK-compatible bounded types.

use anyhow::anyhow;
use anyhow::Error as AnyhowError;
use std::fmt;
use std::sync::Arc;
use std::collections::HashMap;
use thiserror::Error;

use crate::primitive::string::Str;
use crate::system::serialization::{Decode, DecodeError, Encode, SimpleSerialize};

//-----------------------------------------------------------------------------
// Error Categories
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
// Rich Error Context System
//-----------------------------------------------------------------------------

/// Maximum length for error context values to ensure bounded size for ZK compatibility
pub const MAX_CONTEXT_VALUE_LENGTH: usize = 256;

/// Maximum number of context entries to ensure bounded size for ZK compatibility
pub const MAX_CONTEXT_ENTRIES: usize = 8;

/// Maximum string length for error contexts
pub const MAX_CONTEXT_STRING_LENGTH: usize = 64;

/// A bounded string type for error context values with guaranteed maximum length
/// for ZK compatibility.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundedString(pub String);

impl Encode for BoundedString {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.0.as_ssz_bytes()
    }
}

impl Decode for BoundedString {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let s = String::from_ssz_bytes(bytes)?;
        if s.len() > MAX_CONTEXT_STRING_LENGTH {
            Err(DecodeError { message: "BoundedString length exceeds MAX_CONTEXT_STRING_LENGTH".to_string() })
        } else {
            Ok(BoundedString(s))
        }
    }
}

impl SimpleSerialize for BoundedString {}

impl BoundedString {
    /// Create a new bounded string from any string-like type
    pub fn new(value: impl Into<String>) -> Self {
        let mut s = value.into();
        if s.len() > MAX_CONTEXT_STRING_LENGTH {
            s.truncate(MAX_CONTEXT_STRING_LENGTH);
        }
        Self(s)
    }

    /// Get the string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BoundedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Source code location where an error occurred
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    /// Source file path
    pub file: BoundedString,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
}

impl Encode for SourceLocation {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.file.as_ssz_bytes());
        bytes.extend(self.line.as_ssz_bytes());
        bytes.extend(self.column.as_ssz_bytes());
        bytes
    }
}

impl Decode for SourceLocation {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        let file = BoundedString::from_ssz_bytes(&bytes[offset..])?;
        offset += file.as_ssz_bytes().len();
        let line = u32::from_ssz_bytes(&bytes[offset..])?;
        offset += std::mem::size_of::<u32>();
        let column = u32::from_ssz_bytes(&bytes[offset..])?;
        Ok(SourceLocation { file, line, column })
    }
}

impl SimpleSerialize for SourceLocation {}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// Metadata that can be attached to an error for additional context
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorMetadata {
    /// Key-value pairs providing context for the error
    pub context: HashMap<BoundedString, BoundedString>,
    /// The source location where the error occurred
    pub location: Option<SourceLocation>,
    /// Error category for classification
    pub category: ErrorCategory,
    /// Error code for reference
    pub code: Option<BoundedString>,
}

impl Encode for ErrorMetadata {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.context.as_ssz_bytes());
        bytes.extend(self.location.as_ssz_bytes());
        bytes.extend(self.category.as_ssz_bytes());
        bytes.extend(self.code.as_ssz_bytes());
        bytes
    }
}

impl Decode for ErrorMetadata {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;

        let context_bytes_len = u64::from_ssz_bytes(&bytes[offset..offset + std::mem::size_of::<u64>()])? as usize;
        offset += std::mem::size_of::<u64>();
        let context = HashMap::<BoundedString, BoundedString>::from_ssz_bytes(&bytes[offset..offset+context_bytes_len])?;
        offset += context_bytes_len;

        let location_bytes_len = u64::from_ssz_bytes(&bytes[offset..offset + std::mem::size_of::<u64>()])? as usize;
        offset += std::mem::size_of::<u64>();
        let location = Option::<SourceLocation>::from_ssz_bytes(&bytes[offset..offset+location_bytes_len])?;
        offset += location_bytes_len;
        
        let category = ErrorCategory::from_ssz_bytes(&bytes[offset..])?;
        offset += 1; // ErrorCategory is 1 byte

        let code_bytes_len = u64::from_ssz_bytes(&bytes[offset..offset + std::mem::size_of::<u64>()])? as usize;
        offset += std::mem::size_of::<u64>();
        let code = Option::<BoundedString>::from_ssz_bytes(&bytes[offset..offset+code_bytes_len])?;

        Ok(ErrorMetadata {
            context,
            location,
            category,
            code,
        })
    }
}

impl SimpleSerialize for ErrorMetadata {}

impl ErrorMetadata {
    /// Create new error metadata with a given category
    pub fn new(category: ErrorCategory) -> Self {
        Self {
            context: HashMap::new(),
            location: None,
            category,
            code: None,
        }
    }

    /// Add a context entry
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if self.context.len() < MAX_CONTEXT_ENTRIES {
            self.context.insert(BoundedString::new(key), BoundedString::new(value));
        }
        self
    }

    /// Set the source location
    pub fn with_location(mut self, file: &str, line: u32, column: u32) -> Self {
        self.location = Some(SourceLocation {
            file: BoundedString::new(file),
            line,
            column,
        });
        self
    }

    /// Set the error code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(BoundedString::new(code));
        self
    }
}

//-----------------------------------------------------------------------------
// Contextual Error Implementation
//-----------------------------------------------------------------------------

/// Error type with detailed context information
#[derive(Debug, Clone)]
pub struct ContextualError {
    /// Primary error message
    pub message: String,
    /// Error metadata including context, location, and category
    pub metadata: ErrorMetadata,
    /// Optional cause of this error
    pub cause: Option<Arc<ContextualError>>,
}

impl ContextualError {
    /// Create a new contextual error with given message and metadata
    pub fn new(message: impl Into<String>, metadata: ErrorMetadata) -> Self {
        Self {
            message: message.into(),
            metadata,
            cause: None,
        }
    }

    /// Add a cause to this error
    pub fn with_cause(mut self, cause: ContextualError) -> Self {
        self.cause = Some(Arc::new(cause));
        self
    }

    /// Get the error category
    pub fn category(&self) -> ErrorCategory {
        self.metadata.category
    }

    /// Get the source location if available
    pub fn location(&self) -> Option<&SourceLocation> {
        self.metadata.location.as_ref()
    }

    /// Check if this error has a specific context key
    pub fn has_context(&self, key: &str) -> bool {
        for k in self.metadata.context.keys() {
            if k.0 == key {
                return true;
            }
        }
        false
    }

    /// Get a context value for a key if it exists
    pub fn context(&self, key: &str) -> Option<&str> {
        for (k, v) in &self.metadata.context {
            if k.0 == key {
                return Some(&v.0);
            }
        }
        None
    }
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;

        if let Some(location) = &self.metadata.location {
            write!(f, " [at {}]", location)?;
        }

        if let Some(code) = &self.metadata.code {
            write!(f, " [code: {}]", code.0)?;
        }

        if !self.metadata.context.is_empty() {
            write!(f, "\nContext:")?;
            for (key, value) in &self.metadata.context {
                write!(f, "\n  {}: {}", key.0, value.0)?;
            }
        }

        if let Some(cause) = &self.cause {
            write!(f, "\nCaused by: {}", cause)?;
        }

        Ok(())
    }
}

impl std::error::Error for ContextualError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

/// Trait for objects that provide context information for errors.
pub trait AsErrorContext: Send + Sync {
    /// Get the error code (unique identifier)
    fn error_code(&self) -> u32;

    /// Get the error category
    fn error_category(&self) -> ErrorCategory;

    /// Get a human-readable error message
    fn error_message(&self) -> String;

    /// Get the source location if available
    fn source_location(&self) -> Option<SourceLocation>;

    /// Get additional context key-value pairs
    fn context_entries(&self) -> HashMap<BoundedString, BoundedString>;

    /// Check if this is a final error that cannot be retried
    fn is_final(&self) -> bool {
        true // Default is that errors are final (cannot be retried)
    }

    /// Check if this error should be reported to the user
    fn should_report(&self) -> bool {
        true // Default is that all errors should be reported
    }

    /// Create a contextualized error with metadata
    fn create_error(&self, message: String, metadata: ErrorMetadata) -> ContextualError {
        ContextualError::new(message, metadata)
    }
}

/// Default implementation of ErrorContext
pub struct DefaultErrorContext;

impl AsErrorContext for DefaultErrorContext {
    fn error_code(&self) -> u32 {
        0
    }

    fn error_category(&self) -> ErrorCategory {
        ErrorCategory::General
    }

    fn error_message(&self) -> String {
        "Default error context".to_string()
    }

    fn source_location(&self) -> Option<SourceLocation> {
        None
    }

    fn context_entries(&self) -> HashMap<BoundedString, BoundedString> {
        HashMap::new()
    }
}

//-----------------------------------------------------------------------------
// Specific Error Types
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

    /// Create a new EffectHandlingError from another error
    pub fn from_error<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
        EffectHandlingError(anyhow::anyhow!(err))
    }
}

impl From<ResourceError> for EffectHandlingError {
    fn from(err: ResourceError) -> Self {
        EffectHandlingError::from_error(err)
    }
}

//-----------------------------------------------------------------------------
// Simple Error Types for Compatibility
//-----------------------------------------------------------------------------

/// Basic error type for simple error handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalError {
    /// Error code (enum variant as a string)
    pub code: Str,
    /// Error message
    pub message: Str,
    /// Optional context
    pub context: Option<ErrorContext>,
}

/// Simple error context for backward compatibility
#[derive(Debug, Clone, PartialEq, Eq)]
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
        let key = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += key.as_ssz_bytes().len();
        let value = Str::from_ssz_bytes(&bytes[offset..])?;
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
        let code = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += code.as_ssz_bytes().len();
        let message = Str::from_ssz_bytes(&bytes[offset..])?;
        offset += message.as_ssz_bytes().len();
        let context = Option::<ErrorContext>::from_ssz_bytes(&bytes[offset..])?;
        Ok(CausalError {
            code,
            message,
            context,
        })
    }
}

impl SimpleSerialize for CausalError {}

impl CausalError {
    /// Create a new CausalError
    pub fn new(code: impl Into<Str>, message: impl Into<Str>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            context: None,
        }
    }

    /// Add context to the error
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Add context using key-value pair
    pub fn with_key_value(mut self, key: impl Into<Str>, value: impl Into<Str>) -> Self {
        self.context = Some(ErrorContext::new(key, value));
        self
    }
}

impl fmt::Display for CausalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(context) = &self.context {
            write!(f, " ({}={})", context.key, context.value)?;
        }
        Ok(())
    }
}

impl std::error::Error for CausalError {}

//-----------------------------------------------------------------------------
// Helper Functions
//-----------------------------------------------------------------------------

/// Create a categorized error with anyhow
pub fn categorized_error(category: ErrorCategory, message: impl Into<String>) -> anyhow::Error {
    anyhow!("[{}] {}", category, message.into())
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

/// Create an effect handling error
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

//-----------------------------------------------------------------------------
// Helper Macro
//-----------------------------------------------------------------------------

/// Helper macro to create an error with source location
#[macro_export]
macro_rules! contextualized_error {
    ($ctx:expr, $category:expr, $message:expr) => {
        $ctx.create_error(
            $message,
            $crate::primitive::error::ErrorMetadata::new($category)
                .with_location(file!(), line!(), 0)
        )
    };
    ($ctx:expr, $category:expr, $message:expr, $($key:expr => $value:expr),*) => {{
        let mut metadata = $crate::primitive::error::ErrorMetadata::new($category)
            .with_location(file!(), line!(), 0);
        $(
            metadata = metadata.with_context($key, $value);
        )*
        $ctx.create_error($message, metadata)
    }};
} 