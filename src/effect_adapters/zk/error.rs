//! Error handling for ZK effect adapters
//!
//! This module provides error types and utilities for handling errors
//! in ZK effect adapters, including validation and detailed error reporting.

use std::fmt;
use thiserror::Error;
use crate::error::Error as CausalityError;

/// Error types for ZK effect adapters
#[derive(Debug, Error)]
pub enum ZkError {
    /// Error validating input data
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// Error during proof generation
    #[error("Proof generation failed: {0}")]
    ProofGenerationFailed(String),
    
    /// Error during proof verification
    #[error("Proof verification failed: {0}")]
    ProofVerificationFailed(String),
    
    /// Error during witness generation
    #[error("Witness generation failed: {0}")]
    WitnessGenerationFailed(String),
    
    /// Error with program compilation
    #[error("Program compilation failed: {0}")]
    CompilationFailed(String),
    
    /// Error during code generation
    #[error("Code generation failed: {0}")]
    CodeGenerationFailed(String),
    
    /// Error during circuit execution
    #[error("Circuit execution failed: {0}")]
    CircuitExecutionFailed(String),
    
    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    /// Underlying causality error
    #[error("Causality error: {0}")]
    CausalityError(#[from] CausalityError),
    
    /// Unexpected error
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

/// Result type for ZK operations
pub type Result<T> = std::result::Result<T, ZkError>;

/// Validation error details
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Field that failed validation
    pub field: String,
    /// Validation message
    pub message: String,
    /// Validation code
    pub code: ValidationErrorCode,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.code, self.field, self.message)
    }
}

/// Validation error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationErrorCode {
    /// Missing required field
    MissingField,
    /// Invalid data format
    InvalidFormat,
    /// Value out of range
    OutOfRange,
    /// Unsupported operation
    UnsupportedOperation,
    /// Invalid data type
    InvalidType,
    /// Invalid state
    InvalidState,
    /// Generic validation error
    ValidationError,
}

impl fmt::Display for ValidationErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            ValidationErrorCode::MissingField => "E001",
            ValidationErrorCode::InvalidFormat => "E002",
            ValidationErrorCode::OutOfRange => "E003",
            ValidationErrorCode::UnsupportedOperation => "E004",
            ValidationErrorCode::InvalidType => "E005",
            ValidationErrorCode::InvalidState => "E006",
            ValidationErrorCode::ValidationError => "E999",
        };
        write!(f, "{}", code)
    }
}

/// Validator for ZK operations
pub trait ZkValidator {
    /// Validate an input
    fn validate(&self) -> Result<()>;
    
    /// Add validation error
    fn add_error(&mut self, field: &str, message: &str, code: ValidationErrorCode);
    
    /// Get validation errors
    fn errors(&self) -> &[ValidationError];
    
    /// Check if validation passed
    fn is_valid(&self) -> bool {
        self.errors().is_empty()
    }
}

/// Collection of validation errors
#[derive(Debug, Default)]
pub struct ValidationErrors {
    /// List of validation errors
    pub errors: Vec<ValidationError>,
}

impl ValidationErrors {
    /// Create a new empty validation errors collection
    pub fn new() -> Self {
        ValidationErrors {
            errors: Vec::new(),
        }
    }
    
    /// Add a validation error
    pub fn add(&mut self, field: impl Into<String>, message: impl Into<String>, code: ValidationErrorCode) {
        self.errors.push(ValidationError {
            field: field.into(),
            message: message.into(),
            code,
        });
    }
    
    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    /// Get the first error as a ZkError
    pub fn to_error(&self) -> Option<ZkError> {
        if self.errors.is_empty() {
            None
        } else {
            let first = &self.errors[0];
            Some(ZkError::InvalidInput(format!("{}", first)))
        }
    }
    
    /// Convert all errors to a single ZkError
    pub fn to_errors_string(&self) -> String {
        if self.errors.is_empty() {
            "No validation errors".to_string()
        } else {
            let mut result = String::new();
            for (i, error) in self.errors.iter().enumerate() {
                if i > 0 {
                    result.push_str("; ");
                }
                result.push_str(&format!("{}", error));
            }
            result
        }
    }
}

/// Helper function to validate a required field
pub fn validate_required<T: Default + PartialEq>(
    field_name: &str,
    value: &T,
    errors: &mut ValidationErrors,
) {
    if value == &T::default() {
        errors.add(
            field_name,
            "Field is required",
            ValidationErrorCode::MissingField,
        );
    }
}

/// Helper function to validate a value is within range
pub fn validate_range<T: PartialOrd>(
    field_name: &str,
    value: &T,
    min: &T,
    max: &T,
    errors: &mut ValidationErrors,
) {
    if value < min || value > max {
        errors.add(
            field_name,
            &format!("Value must be between {:?} and {:?}", min, max),
            ValidationErrorCode::OutOfRange,
        );
    }
}

/// Helper function to validate a string format using a regular expression
pub fn validate_format(
    field_name: &str,
    value: &str,
    pattern: &str,
    errors: &mut ValidationErrors,
) {
    // Avoid panic if regex is invalid, just report the error
    match regex::Regex::new(pattern) {
        Ok(re) => {
            if !re.is_match(value) {
                errors.add(
                    field_name,
                    &format!("Invalid format, must match pattern: {}", pattern),
                    ValidationErrorCode::InvalidFormat,
                );
            }
        }
        Err(_) => {
            errors.add(
                "validator",
                &format!("Invalid regex pattern: {}", pattern),
                ValidationErrorCode::ValidationError,
            );
        }
    }
} 