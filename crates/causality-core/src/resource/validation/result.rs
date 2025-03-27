// Validation result types
// This file contains types for representing the results of resource validation operations.

use std::fmt;
use std::collections::HashMap;
use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Severity level of validation issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Informational messages that don't affect validation success
    Info,
    
    /// Warning messages that don't prevent operation but should be noted
    Warning,
    
    /// Error messages that prevent the operation
    Error,
    
    /// Critical errors that indicate serious system issues
    Critical,
}

impl fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationSeverity::Info => write!(f, "INFO"),
            ValidationSeverity::Warning => write!(f, "WARNING"),
            ValidationSeverity::Error => write!(f, "ERROR"),
            ValidationSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Status of a validation operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationStatus {
    /// Validation succeeded with no issues
    Success,
    
    /// Validation succeeded with warnings
    SuccessWithWarnings,
    
    /// Validation failed due to errors
    Failed,
    
    /// Validation could not be completed
    Incomplete,
}

impl fmt::Display for ValidationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationStatus::Success => write!(f, "SUCCESS"),
            ValidationStatus::SuccessWithWarnings => write!(f, "SUCCESS_WITH_WARNINGS"),
            ValidationStatus::Failed => write!(f, "FAILED"),
            ValidationStatus::Incomplete => write!(f, "INCOMPLETE"),
        }
    }
}

/// A validation issue (error, warning, or info)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Severity of the issue
    pub severity: ValidationSeverity,
    
    /// Message describing the issue
    pub message: String,
    
    /// Code representing the issue type
    pub code: String,
    
    /// Source of the validation issue
    pub source: String,
    
    /// Additional context for the issue
    pub context: HashMap<String, String>,
}

impl ValidationIssue {
    /// Create a new validation issue
    pub fn new(
        severity: ValidationSeverity, 
        message: impl Into<String>,
        code: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            message: message.into(),
            code: code.into(),
            source: source.into(),
            context: HashMap::new(),
        }
    }
    
    /// Create a new error validation issue
    pub fn error(message: impl Into<String>, code: impl Into<String>, source: impl Into<String>) -> Self {
        Self::new(ValidationSeverity::Error, message, code, source)
    }
    
    /// Create a new warning validation issue
    pub fn warning(message: impl Into<String>, code: impl Into<String>, source: impl Into<String>) -> Self {
        Self::new(ValidationSeverity::Warning, message, code, source)
    }
    
    /// Create a new info validation issue
    pub fn info(message: impl Into<String>, code: impl Into<String>, source: impl Into<String>) -> Self {
        Self::new(ValidationSeverity::Info, message, code, source)
    }
    
    /// Create a new critical validation issue
    pub fn critical(message: impl Into<String>, code: impl Into<String>, source: impl Into<String>) -> Self {
        Self::new(ValidationSeverity::Critical, message, code, source)
    }
    
    /// Add context to the issue
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// Error representing a validation failure
#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    /// Structure validation error
    #[error("Structure validation error: {0}")]
    StructureError(String),
    
    /// State validation error
    #[error("State validation error: {0}")]
    StateError(String),
    
    /// Schema validation error
    #[error("Schema validation error: {0}")]
    SchemaError(String),
    
    /// Permission validation error
    #[error("Permission validation error: {0}")]
    PermissionError(String),
    
    /// Relationship validation error
    #[error("Relationship validation error: {0}")]
    RelationshipError(String),
    
    /// Custom validation error
    #[error("Custom validation error: {0}")]
    CustomError(String),
    
    /// Internal validation error
    #[error("Internal validation error: {0}")]
    InternalError(String),
}

/// Result of a validation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Status of the validation
    pub status: ValidationStatus,
    
    /// Issues found during validation
    pub issues: Vec<ValidationIssue>,
    
    /// Metadata about the validation
    pub metadata: HashMap<String, String>,
}

impl ValidationResult {
    /// Create a new successful validation result
    pub fn success() -> Self {
        Self {
            status: ValidationStatus::Success,
            issues: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Create a new failed validation result
    pub fn failed(issues: Vec<ValidationIssue>) -> Self {
        Self {
            status: ValidationStatus::Failed,
            issues,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a new incomplete validation result
    pub fn incomplete(message: impl Into<String>) -> Self {
        let issue = ValidationIssue::error(
            message.into(),
            "VALIDATION_INCOMPLETE",
            "validation_system",
        );
        
        Self {
            status: ValidationStatus::Incomplete,
            issues: vec![issue],
            metadata: HashMap::new(),
        }
    }
    
    /// Add an issue to the validation result
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
        self.update_status();
    }
    
    /// Add an error to the validation result
    pub fn add_error(&mut self, message: impl Into<String>, code: impl Into<String>, source: impl Into<String>) {
        self.add_issue(ValidationIssue::error(message, code, source));
    }
    
    /// Add a warning to the validation result
    pub fn add_warning(&mut self, message: impl Into<String>, code: impl Into<String>, source: impl Into<String>) {
        self.add_issue(ValidationIssue::warning(message, code, source));
    }
    
    /// Add an info message to the validation result
    pub fn add_info(&mut self, message: impl Into<String>, code: impl Into<String>, source: impl Into<String>) {
        self.add_issue(ValidationIssue::info(message, code, source));
    }
    
    /// Add metadata to the validation result
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
    
    /// Add multiple issues to the validation result
    pub fn add_issues(&mut self, issues: Vec<ValidationIssue>) {
        self.issues.extend(issues);
        self.update_status();
    }
    
    /// Merge another validation result into this one
    pub fn merge(&mut self, other: ValidationResult) {
        self.issues.extend(other.issues);
        self.metadata.extend(other.metadata);
        self.update_status();
    }
    
    /// Check if the validation passed
    pub fn is_valid(&self) -> bool {
        matches!(self.status, ValidationStatus::Success | ValidationStatus::SuccessWithWarnings)
    }
    
    /// Get all errors in the validation result
    pub fn errors(&self) -> Vec<&ValidationIssue> {
        self.issues.iter()
            .filter(|i| matches!(i.severity, ValidationSeverity::Error | ValidationSeverity::Critical))
            .collect()
    }
    
    /// Get all warnings in the validation result
    pub fn warnings(&self) -> Vec<&ValidationIssue> {
        self.issues.iter()
            .filter(|i| matches!(i.severity, ValidationSeverity::Warning))
            .collect()
    }
    
    /// Get all info messages in the validation result
    pub fn info(&self) -> Vec<&ValidationIssue> {
        self.issues.iter()
            .filter(|i| matches!(i.severity, ValidationSeverity::Info))
            .collect()
    }
    
    /// Update the status based on the current issues
    fn update_status(&mut self) {
        if self.issues.is_empty() {
            self.status = ValidationStatus::Success;
            return;
        }
        
        let has_errors = self.issues.iter()
            .any(|i| matches!(i.severity, ValidationSeverity::Error | ValidationSeverity::Critical));
            
        if has_errors {
            self.status = ValidationStatus::Failed;
        } else {
            self.status = ValidationStatus::SuccessWithWarnings;
        }
    }
} 