// Domain-Specific Effect Validation Protocols
//
// This module provides protocols for validating domain-specific effects
// before they are submitted to blockchain networks.

use std::collections::HashMap;
use thiserror::Error;

use crate::error::Result;
use crate::types::DomainId;
use crate::domain_adapters::interfaces::{VmType, MultiVmAdapterConfig};

/// Validation context containing information about the effect being validated
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Domain ID
    pub domain_id: DomainId,
    /// VM type
    pub vm_type: VmType,
    /// Effect type
    pub effect_type: String,
    /// Effect parameters
    pub params: HashMap<String, Vec<u8>>,
    /// Validation rules
    pub rules: Vec<ValidationRule>,
    /// Extra context data
    pub extra: HashMap<String, String>,
}

impl ValidationContext {
    /// Create a new validation context
    pub fn new(domain_id: DomainId, vm_type: VmType, effect_type: impl Into<String>) -> Self {
        Self {
            domain_id,
            vm_type,
            effect_type: effect_type.into(),
            params: HashMap::new(),
            rules: Vec::new(),
            extra: HashMap::new(),
        }
    }
    
    /// From adapter config
    pub fn from_config(config: &MultiVmAdapterConfig, effect_type: impl Into<String>) -> Self {
        Self::new(config.domain_id.clone(), config.vm_type.clone(), effect_type)
    }
    
    /// Add a parameter
    pub fn add_param(&mut self, key: impl Into<String>, value: impl Into<Vec<u8>>) -> &mut Self {
        self.params.insert(key.into(), value.into());
        self
    }
    
    /// Add a validation rule
    pub fn add_rule(&mut self, rule: ValidationRule) -> &mut Self {
        self.rules.push(rule);
        self
    }
    
    /// Add extra context data
    pub fn add_extra(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// Validation rule type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationRuleType {
    /// Required field
    RequiredField,
    /// Field format
    Format,
    /// Field length
    Length,
    /// Value range
    Range,
    /// Field relationship
    Relationship,
    /// Custom validation
    Custom,
}

/// Validation rule
#[derive(Debug, Clone)]
pub struct ValidationRule {
    /// Rule type
    pub rule_type: ValidationRuleType,
    /// Field name
    pub field: String,
    /// Rule parameters
    pub params: HashMap<String, String>,
    /// Error message
    pub error_message: String,
    /// Severity level
    pub severity: ValidationSeverity,
}

/// Validation rule builder
pub struct ValidationRuleBuilder {
    rule: ValidationRule,
}

impl ValidationRuleBuilder {
    /// Create a new rule builder
    pub fn new(rule_type: ValidationRuleType, field: impl Into<String>) -> Self {
        Self {
            rule: ValidationRule {
                rule_type,
                field: field.into(),
                params: HashMap::new(),
                error_message: String::new(),
                severity: ValidationSeverity::Error,
            },
        }
    }
    
    /// Add a parameter
    pub fn param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.rule.params.insert(key.into(), value.into());
        self
    }
    
    /// Set error message
    pub fn error_message(mut self, message: impl Into<String>) -> Self {
        self.rule.error_message = message.into();
        self
    }
    
    /// Set severity
    pub fn severity(mut self, severity: ValidationSeverity) -> Self {
        self.rule.severity = severity;
        self
    }
    
    /// Build the rule
    pub fn build(self) -> ValidationRule {
        self.rule
    }
}

/// Validation severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    /// Information only
    Info,
    /// Warning (validation will pass with warnings)
    Warning,
    /// Error (validation will fail)
    Error,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Validation passed
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<ValidationError>,
    /// Validation warnings
    pub warnings: Vec<ValidationError>,
    /// Extra result data
    pub data: HashMap<String, String>,
}

impl ValidationResult {
    /// Create a new valid result
    pub fn valid() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            data: HashMap::new(),
        }
    }
    
    /// Create a new invalid result
    pub fn invalid(errors: Vec<ValidationError>) -> Self {
        Self {
            valid: false,
            errors,
            warnings: Vec::new(),
            data: HashMap::new(),
        }
    }
    
    /// Add an error
    pub fn add_error(&mut self, error: ValidationError) -> &mut Self {
        self.errors.push(error);
        self.valid = false;
        self
    }
    
    /// Add a warning
    pub fn add_warning(&mut self, warning: ValidationError) -> &mut Self {
        self.warnings.push(warning);
        self
    }
    
    /// Add extra data
    pub fn add_data(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.data.insert(key.into(), value.into());
        self
    }
    
    /// Merge with another result
    pub fn merge(&mut self, other: ValidationResult) -> &mut Self {
        self.valid = self.valid && other.valid;
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.data.extend(other.data);
        self
    }
}

/// Validation error
#[derive(Debug, Clone, Error)]
#[error("{}: {}", field, message)]
pub struct ValidationError {
    /// Field name
    pub field: String,
    /// Error message
    pub message: String,
    /// Error code
    pub code: String,
    /// Severity level
    pub severity: ValidationSeverity,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(
        field: impl Into<String>,
        message: impl Into<String>,
        code: impl Into<String>,
        severity: ValidationSeverity,
    ) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: code.into(),
            severity,
        }
    }
    
    /// Create a new error level validation error
    pub fn error(field: impl Into<String>, message: impl Into<String>, code: impl Into<String>) -> Self {
        Self::new(field, message, code, ValidationSeverity::Error)
    }
    
    /// Create a new warning level validation error
    pub fn warning(field: impl Into<String>, message: impl Into<String>, code: impl Into<String>) -> Self {
        Self::new(field, message, code, ValidationSeverity::Warning)
    }
    
    /// Create a new info level validation error
    pub fn info(field: impl Into<String>, message: impl Into<String>, code: impl Into<String>) -> Self {
        Self::new(field, message, code, ValidationSeverity::Info)
    }
}

/// Trait for effect validation
pub trait EffectValidator {
    /// Validate an effect
    fn validate(&self, context: &ValidationContext) -> Result<ValidationResult>;
    
    /// Get the VM type this validator supports
    fn vm_type(&self) -> VmType;
    
    /// Get the effect types this validator supports
    fn supported_effect_types(&self) -> Vec<String>;
    
    /// Check if this validator supports a specific effect type
    fn supports_effect_type(&self, effect_type: &str) -> bool {
        self.supported_effect_types().contains(&effect_type.to_string())
    }
}

/// Factory for creating effect validators
pub trait EffectValidatorFactory {
    /// Create a validator for a specific effect type
    fn create_validator(&self, effect_type: &str, vm_type: &VmType) -> Option<Box<dyn EffectValidator>>;
    
    /// Get all supported effect types
    fn supported_effect_types(&self) -> Vec<String>;
    
    /// Get all supported VM types
    fn supported_vm_types(&self) -> Vec<VmType>;
}

/// Registry for effect validators
#[derive(Debug, Default)]
pub struct EffectValidatorRegistry {
    /// Factories by VM type
    factories: HashMap<VmType, Vec<Box<dyn EffectValidatorFactory>>>,
    /// Validators by (VM type, effect type)
    validators: HashMap<(VmType, String), Box<dyn EffectValidator>>,
}

impl EffectValidatorRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            validators: HashMap::new(),
        }
    }
    
    /// Register a factory
    pub fn register_factory(&mut self, vm_type: VmType, factory: Box<dyn EffectValidatorFactory>) {
        let factories = self.factories.entry(vm_type).or_insert_with(Vec::new);
        factories.push(factory);
    }
    
    /// Register a validator
    pub fn register_validator(&mut self, validator: Box<dyn EffectValidator>) {
        let vm_type = validator.vm_type();
        for effect_type in validator.supported_effect_types() {
            self.validators.insert((vm_type.clone(), effect_type), validator.clone());
        }
    }
    
    /// Get a validator for a specific effect type and VM type
    pub fn get_validator(&self, effect_type: &str, vm_type: &VmType) -> Option<&Box<dyn EffectValidator>> {
        // Try to get a direct validator
        if let Some(validator) = self.validators.get(&(vm_type.clone(), effect_type.to_string())) {
            return Some(validator);
        }
        
        // Try to create a validator from factories
        if let Some(factories) = self.factories.get(vm_type) {
            for factory in factories {
                if let Some(validator) = factory.create_validator(effect_type, vm_type) {
                    // In a real implementation, we would add the validator to the cache
                    // For now, we can't do that due to the borrow checker
                    return Some(&validator);
                }
            }
        }
        
        None
    }
    
    /// Validate an effect
    pub fn validate(&self, context: &ValidationContext) -> Result<ValidationResult> {
        let validator = self.get_validator(&context.effect_type, &context.vm_type)
            .ok_or_else(|| crate::error::Error::NotFoundError(format!(
                "No validator found for effect type '{}' and VM type '{:?}'",
                context.effect_type, context.vm_type
            )))?;
        
        validator.validate(context)
    }
}

/// Required field validator
pub struct RequiredFieldValidator;

impl RequiredFieldValidator {
    /// Validate required fields
    pub fn validate_required_fields(
        context: &ValidationContext,
        field_names: &[&str],
    ) -> ValidationResult {
        let mut result = ValidationResult::valid();
        
        for &field in field_names {
            if !context.params.contains_key(field) {
                result.add_error(ValidationError::error(
                    field,
                    format!("Field '{}' is required", field),
                    "E001",
                ));
            }
        }
        
        result
    }
}

/// Base implementation of common effect validation rules
pub fn validate_common_rules(context: &ValidationContext) -> Result<ValidationResult> {
    let mut result = ValidationResult::valid();
    
    // Apply each rule
    for rule in &context.rules {
        match rule.rule_type {
            ValidationRuleType::RequiredField => {
                if !context.params.contains_key(&rule.field) {
                    let error = ValidationError::new(
                        rule.field.clone(),
                        rule.error_message.clone(),
                        "E001",
                        rule.severity,
                    );
                    
                    match rule.severity {
                        ValidationSeverity::Error => {
                            result.add_error(error);
                        }
                        ValidationSeverity::Warning => {
                            result.add_warning(error);
                        }
                        _ => {}
                    }
                }
            }
            ValidationRuleType::Length => {
                if let Some(value) = context.params.get(&rule.field) {
                    if let (Some(min), Some(max)) = (
                        rule.params.get("min").and_then(|s| s.parse::<usize>().ok()),
                        rule.params.get("max").and_then(|s| s.parse::<usize>().ok()),
                    ) {
                        if value.len() < min || value.len() > max {
                            let error = ValidationError::new(
                                rule.field.clone(),
                                rule.error_message.clone(),
                                "E002",
                                rule.severity,
                            );
                            
                            match rule.severity {
                                ValidationSeverity::Error => {
                                    result.add_error(error);
                                }
                                ValidationSeverity::Warning => {
                                    result.add_warning(error);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            // Additional rule types would be implemented here
            _ => {}
        }
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation_context() {
        let mut context = ValidationContext::new(
            DomainId::new("test"),
            VmType::ZkVm,
            "test_effect",
        );
        
        context.add_param("param1", vec![1, 2, 3]);
        context.add_rule(
            ValidationRuleBuilder::new(ValidationRuleType::RequiredField, "param2")
                .error_message("param2 is required")
                .build()
        );
        
        assert_eq!(context.domain_id.as_ref(), "test");
        assert_eq!(context.params.get("param1"), Some(&vec![1, 2, 3]));
        assert_eq!(context.rules.len(), 1);
    }
    
    #[test]
    fn test_required_field_validation() {
        let mut context = ValidationContext::new(
            DomainId::new("test"),
            VmType::ZkVm,
            "test_effect",
        );
        
        context.add_param("param1", vec![1, 2, 3]);
        
        let result = RequiredFieldValidator::validate_required_fields(
            &context,
            &["param1", "param2"],
        );
        
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].field, "param2");
    }
} 