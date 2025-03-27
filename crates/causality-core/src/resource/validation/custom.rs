// Custom validation module
// This file contains components for custom validation rules.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};

use crate::resource_types::{ResourceId, ResourceTypeId};
use crate::resource::ResourceState;

use super::context::ValidationContext;
use super::result::{ValidationResult, ValidationIssue, ValidationError, ValidationSeverity};

/// Error types specific to custom validation
#[derive(Error, Debug, Clone)]
pub enum CustomValidationError {
    /// Rule not found
    #[error("Custom validation rule not found: {0}")]
    RuleNotFound(String),
    
    /// Rule execution error
    #[error("Rule execution error: {0}")]
    RuleExecutionError(String),
    
    /// Invalid rule format
    #[error("Invalid rule format: {0}")]
    InvalidRuleFormat(String),
    
    /// Internal error
    #[error("Internal custom validation error: {0}")]
    InternalError(String),
}

/// Context for custom validation
#[derive(Debug, Clone)]
pub struct CustomValidationContext {
    /// Resource ID
    pub resource_id: Option<ResourceId>,
    
    /// Resource type
    pub resource_type: Option<ResourceTypeId>,
    
    /// Current resource state
    pub current_state: Option<ResourceState>,
    
    /// Target resource state
    pub target_state: Option<ResourceState>,
    
    /// Context data
    pub context_data: HashMap<String, Vec<u8>>,
}

impl From<&ValidationContext> for CustomValidationContext {
    fn from(context: &ValidationContext) -> Self {
        Self {
            resource_id: context.resource_id.clone(),
            resource_type: context.resource_type.clone(),
            current_state: context.current_state.clone(),
            target_state: context.target_state.clone(),
            context_data: context.context_data.clone(),
        }
    }
}

/// Custom validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomValidationRule {
    /// Rule ID
    pub id: String,
    
    /// Rule name
    pub name: String,
    
    /// Rule description
    pub description: Option<String>,
    
    /// Resource types this rule applies to
    pub resource_types: Vec<ResourceTypeId>,
    
    /// Rule script or definition
    pub rule_definition: String,
    
    /// Rule language or format
    pub rule_format: String,
    
    /// Rule severity level
    pub severity: ValidationSeverity,
    
    /// Additional rule metadata
    pub metadata: HashMap<String, String>,
}

/// Trait for custom validators
#[async_trait]
pub trait CustomValidator: Send + Sync + std::fmt::Debug {
    /// Validate using a custom rule
    async fn validate(
        &self,
        context: &ValidationContext,
    ) -> Result<ValidationResult, ValidationError>;
    
    /// Get validator name
    fn name(&self) -> &str;
    
    /// Get supported rule formats
    fn supported_formats(&self) -> Vec<String>;
    
    /// Register a custom rule
    fn register_rule(&self, rule: CustomValidationRule) -> Result<(), CustomValidationError>;
}

/// A simple custom validator implementation that uses embedded rules
#[derive(Debug)]
pub struct SimpleCustomValidator {
    /// Name of this validator
    name: String,
    
    /// Rules by ID
    rules: RwLock<HashMap<String, CustomValidationRule>>,
    
    /// Rules by resource type
    rules_by_type: RwLock<HashMap<ResourceTypeId, Vec<String>>>,
}

impl SimpleCustomValidator {
    /// Create a new simple custom validator
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rules: RwLock::new(HashMap::new()),
            rules_by_type: RwLock::new(HashMap::new()),
        }
    }
    
    /// Get a rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Result<Option<CustomValidationRule>, CustomValidationError> {
        let rules = self.rules.read().map_err(|e|
            CustomValidationError::InternalError(format!("Failed to acquire rules lock: {}", e))
        )?;
        
        Ok(rules.get(rule_id).cloned())
    }
    
    /// Get rules for a resource type
    pub fn get_rules_for_type(&self, resource_type: &ResourceTypeId) -> Result<Vec<CustomValidationRule>, CustomValidationError> {
        let rules_by_type = self.rules_by_type.read().map_err(|e|
            CustomValidationError::InternalError(format!("Failed to acquire rules by type lock: {}", e))
        )?;
        
        let rules = self.rules.read().map_err(|e|
            CustomValidationError::InternalError(format!("Failed to acquire rules lock: {}", e))
        )?;
        
        let mut result = Vec::new();
        
        if let Some(rule_ids) = rules_by_type.get(resource_type) {
            for id in rule_ids {
                if let Some(rule) = rules.get(id) {
                    result.push(rule.clone());
                }
            }
        }
        
        Ok(result)
    }
    
    /// Evaluate a rule
    fn evaluate_rule(
        &self,
        rule: &CustomValidationRule,
        context: &CustomValidationContext,
    ) -> Result<ValidationResult, CustomValidationError> {
        let mut result = ValidationResult::success();
        
        // In a real implementation, this would interpret or execute the rule
        // For this example, we'll just return some dummy results based on the rule format
        
        match rule.rule_format.as_str() {
            "script" => {
                // For script format, we'd execute some script engine here
                // Just simulate it for this example
                result.add_info(
                    format!("Script rule {} would execute here", rule.id),
                    "CUSTOM_RULE_SCRIPT",
                    "custom_validator",
                );
            },
            "json" => {
                // For JSON format, we'd interpret JSON rule definition
                // Just simulate it for this example
                result.add_info(
                    format!("JSON rule {} would be interpreted here", rule.id),
                    "CUSTOM_RULE_JSON",
                    "custom_validator",
                );
            },
            "wasm" => {
                // For WASM format, we'd execute a WebAssembly module
                // Just simulate it for this example
                result.add_info(
                    format!("WASM rule {} would execute here", rule.id),
                    "CUSTOM_RULE_WASM",
                    "custom_validator",
                );
            },
            _ => {
                return Err(CustomValidationError::InvalidRuleFormat(
                    format!("Unsupported rule format: {}", rule.rule_format)
                ));
            }
        }
        
        // Add a warning to simulate rule results
        result.add_warning(
            format!("Custom rule {} executed with simulated results", rule.id),
            "CUSTOM_RULE_RESULT",
            "custom_validator",
        );
        
        Ok(result)
    }
}

#[async_trait]
impl CustomValidator for SimpleCustomValidator {
    async fn validate(
        &self,
        context: &ValidationContext,
    ) -> Result<ValidationResult, ValidationError> {
        let mut result = ValidationResult::success();
        
        // Check if we have a resource type to validate against
        if let Some(resource_type) = &context.resource_type {
            // Get rules for this resource type
            let rules = self.get_rules_for_type(resource_type)
                .map_err(|e| ValidationError::CustomError(e.to_string()))?;
            
            if rules.is_empty() {
                // No rules for this type
                result.add_info(
                    format!("No custom validation rules for resource type: {}", resource_type),
                    "NO_CUSTOM_RULES",
                    "custom_validator",
                );
                return Ok(result);
            }
            
            // Create custom validation context
            let custom_context = CustomValidationContext::from(context);
            
            // Evaluate each rule
            for rule in rules {
                let rule_result = self.evaluate_rule(&rule, &custom_context)
                    .map_err(|e| ValidationError::CustomError(e.to_string()))?;
                
                // Merge results
                result.merge(rule_result);
                
                // If this is a critical error and we have issues, we can stop early
                if !result.is_valid() && rule.severity == ValidationSeverity::Critical {
                    break;
                }
            }
        } else {
            // No resource type specified
            result.add_warning(
                "Cannot run custom validation without resource type",
                "MISSING_RESOURCE_TYPE",
                "custom_validator",
            );
        }
        
        Ok(result)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn supported_formats(&self) -> Vec<String> {
        vec![
            "script".to_string(),
            "json".to_string(), 
            "wasm".to_string(),
        ]
    }
    
    fn register_rule(&self, rule: CustomValidationRule) -> Result<(), CustomValidationError> {
        let rule_id = rule.id.clone();
        
        // Check if rule format is supported
        if !self.supported_formats().contains(&rule.rule_format) {
            return Err(CustomValidationError::InvalidRuleFormat(
                format!("Unsupported rule format: {}", rule.rule_format)
            ));
        }
        
        // Add to rules registry
        {
            let mut rules = self.rules.write().map_err(|e|
                CustomValidationError::InternalError(format!("Failed to acquire rules lock: {}", e))
            )?;
            
            rules.insert(rule_id.clone(), rule.clone());
        }
        
        // Add to type index
        {
            let mut rules_by_type = self.rules_by_type.write().map_err(|e|
                CustomValidationError::InternalError(format!("Failed to acquire rules by type lock: {}", e))
            )?;
            
            for resource_type in &rule.resource_types {
                let type_rules = rules_by_type
                    .entry(resource_type.clone())
                    .or_insert_with(Vec::new);
                    
                if !type_rules.contains(&rule_id) {
                    type_rules.push(rule_id.clone());
                }
            }
        }
        
        Ok(())
    }
}

/// Register a custom validator with a registry
pub fn register_custom_validator<V: CustomValidator + 'static>(
    registry: &super::validation::ResourceValidator,
    validator: V,
) -> Result<(), ValidationError> {
    registry.register_custom_validator(validator)
} 