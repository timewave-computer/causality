// Validation rule module
// This file contains components for defining and managing validation rules.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use serde::{Serialize, Deserialize};

use crate::resource::{ResourceId, ResourceTypeId};

use super::result::{ValidationSeverity, ValidationResult, ValidationIssue};
use super::context::ValidationContext;

/// Error types specific to validation rules
#[derive(Error, Debug, Clone)]
pub enum ValidationRuleError {
    /// Rule not found
    #[error("Validation rule not found: {0}")]
    RuleNotFound(String),
    
    /// Rule execution error
    #[error("Rule execution error: {0}")]
    ExecutionError(String),
    
    /// Invalid rule definition
    #[error("Invalid rule definition: {0}")]
    InvalidDefinition(String),
    
    /// Internal error
    #[error("Internal validation rule error: {0}")]
    InternalError(String),
}

/// Validation rule condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Equal to
    Equals(String, String),
    
    /// Not equal to
    NotEquals(String, String),
    
    /// Greater than
    GreaterThan(String, String),
    
    /// Less than
    LessThan(String, String),
    
    /// Contains
    Contains(String, String),
    
    /// Starts with
    StartsWith(String, String),
    
    /// Ends with
    EndsWith(String, String),
    
    /// Matches regex
    Regex(String, String),
    
    /// Custom expression
    Expression(String),
    
    /// And
    And(Vec<RuleCondition>),
    
    /// Or
    Or(Vec<RuleCondition>),
    
    /// Not
    Not(Box<RuleCondition>),
    
    /// Always true
    AlwaysTrue,
    
    /// Always false
    AlwaysFalse,
}

/// Action to take when a rule is violated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    /// Allow the operation
    Allow,
    
    /// Deny the operation
    Deny,
    
    /// Warn but allow
    Warn(String),
    
    /// Log
    Log(String),
    
    /// Custom action
    Custom(String),
}

/// Validation rule for resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule ID
    pub id: String,
    
    /// Rule name
    pub name: String,
    
    /// Rule description
    pub description: Option<String>,
    
    /// Resource types this rule applies to
    pub resource_types: Vec<ResourceTypeId>,
    
    /// Rule condition
    pub condition: RuleCondition,
    
    /// Action to take if condition is met
    pub action: RuleAction,
    
    /// Rule severity
    pub severity: ValidationSeverity,
    
    /// Additional rule metadata
    pub metadata: HashMap<String, String>,
}

impl ValidationRule {
    /// Create a new validation rule
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        condition: RuleCondition,
        action: RuleAction,
        severity: ValidationSeverity,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            resource_types: Vec::new(),
            condition,
            action,
            severity,
            metadata: HashMap::new(),
        }
    }
    
    /// Add a description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Add a resource type
    pub fn with_resource_type(mut self, resource_type: ResourceTypeId) -> Self {
        self.resource_types.push(resource_type);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Rule engine for evaluating validation rules
#[derive(Debug)]
pub struct ValidationRuleEngine {
    /// Rules by ID
    rules: RwLock<HashMap<String, ValidationRule>>,
    
    /// Rules by resource type
    rules_by_type: RwLock<HashMap<ResourceTypeId, Vec<String>>>,
}

impl ValidationRuleEngine {
    /// Create a new validation rule engine
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(HashMap::new()),
            rules_by_type: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a validation rule
    pub fn register_rule(&self, rule: ValidationRule) -> Result<(), ValidationRuleError> {
        let rule_id = rule.id.clone();
        
        // Add to rules registry
        {
            let mut rules = self.rules.write().map_err(|e|
                ValidationRuleError::InternalError(format!("Failed to acquire rules lock: {}", e))
            )?;
            
            rules.insert(rule_id.clone(), rule.clone());
        }
        
        // Add to type index
        {
            let mut rules_by_type = self.rules_by_type.write().map_err(|e|
                ValidationRuleError::InternalError(format!("Failed to acquire rules by type lock: {}", e))
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
    
    /// Get a rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Result<Option<ValidationRule>, ValidationRuleError> {
        let rules = self.rules.read().map_err(|e|
            ValidationRuleError::InternalError(format!("Failed to acquire rules lock: {}", e))
        )?;
        
        Ok(rules.get(rule_id).cloned())
    }
    
    /// Get rules for a resource type
    pub fn get_rules_for_type(&self, resource_type: &ResourceTypeId) -> Result<Vec<ValidationRule>, ValidationRuleError> {
        let rules_by_type = self.rules_by_type.read().map_err(|e|
            ValidationRuleError::InternalError(format!("Failed to acquire rules by type lock: {}", e))
        )?;
        
        let rules = self.rules.read().map_err(|e|
            ValidationRuleError::InternalError(format!("Failed to acquire rules lock: {}", e))
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
    
    /// Evaluate a condition
    fn evaluate_condition(
        &self,
        condition: &RuleCondition,
        context: &ValidationContext,
    ) -> Result<bool, ValidationRuleError> {
        match condition {
            RuleCondition::Equals(key, value) => {
                if let Some(ctx_value) = context.get_string_context(key) {
                    Ok(ctx_value == *value)
                } else {
                    Ok(false)
                }
            },
            RuleCondition::NotEquals(key, value) => {
                if let Some(ctx_value) = context.get_string_context(key) {
                    Ok(ctx_value != *value)
                } else {
                    Ok(true) // If key doesn't exist, it's not equal
                }
            },
            RuleCondition::GreaterThan(key, value) => {
                if let Some(ctx_value) = context.get_string_context(key) {
                    // Try parsing as numbers
                    if let (Ok(ctx_num), Ok(val_num)) = (ctx_value.parse::<f64>(), value.parse::<f64>()) {
                        return Ok(ctx_num > val_num);
                    }
                    // Otherwise compare lexicographically
                    Ok(ctx_value > *value)
                } else {
                    Ok(false)
                }
            },
            RuleCondition::LessThan(key, value) => {
                if let Some(ctx_value) = context.get_string_context(key) {
                    // Try parsing as numbers
                    if let (Ok(ctx_num), Ok(val_num)) = (ctx_value.parse::<f64>(), value.parse::<f64>()) {
                        return Ok(ctx_num < val_num);
                    }
                    // Otherwise compare lexicographically
                    Ok(ctx_value < *value)
                } else {
                    Ok(false)
                }
            },
            RuleCondition::Contains(key, value) => {
                if let Some(ctx_value) = context.get_string_context(key) {
                    Ok(ctx_value.contains(value))
                } else {
                    Ok(false)
                }
            },
            RuleCondition::StartsWith(key, value) => {
                if let Some(ctx_value) = context.get_string_context(key) {
                    Ok(ctx_value.starts_with(value))
                } else {
                    Ok(false)
                }
            },
            RuleCondition::EndsWith(key, value) => {
                if let Some(ctx_value) = context.get_string_context(key) {
                    Ok(ctx_value.ends_with(value))
                } else {
                    Ok(false)
                }
            },
            RuleCondition::Regex(key, pattern) => {
                if let Some(ctx_value) = context.get_string_context(key) {
                    // In a real implementation, use a regex library
                    // For now, just simulate it
                    Ok(ctx_value.contains(pattern))
                } else {
                    Ok(false)
                }
            },
            RuleCondition::Expression(expr) => {
                // In a real implementation, evaluate an expression
                // For now, just add a simple check
                Ok(expr == "true")
            },
            RuleCondition::And(conditions) => {
                for cond in conditions {
                    if !self.evaluate_condition(cond, context)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            },
            RuleCondition::Or(conditions) => {
                for cond in conditions {
                    if self.evaluate_condition(cond, context)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            },
            RuleCondition::Not(cond) => {
                let result = self.evaluate_condition(cond, context)?;
                Ok(!result)
            },
            RuleCondition::AlwaysTrue => Ok(true),
            RuleCondition::AlwaysFalse => Ok(false),
        }
    }
    
    /// Evaluate a rule
    pub fn evaluate_rule(
        &self,
        rule: &ValidationRule,
        context: &ValidationContext,
    ) -> Result<ValidationResult, ValidationRuleError> {
        let mut result = ValidationResult::success();
        
        // Evaluate the condition
        let condition_met = self.evaluate_condition(&rule.condition, context)?;
        
        // If condition is met, apply the action
        if condition_met {
            match &rule.action {
                RuleAction::Allow => {
                    // No action needed, already success
                },
                RuleAction::Deny => {
                    let message = rule.description.as_ref()
                        .unwrap_or(&format!("Rule '{}' violated", rule.name))
                        .clone();
                        
                    let issue = ValidationIssue::new(
                        rule.severity,
                        message,
                        format!("RULE_{}", rule.id),
                        "rule_engine",
                    );
                    
                    result.add_issue(issue);
                },
                RuleAction::Warn(message) => {
                    let issue = ValidationIssue::new(
                        ValidationSeverity::Warning,
                        message.clone(),
                        format!("RULE_{}", rule.id),
                        "rule_engine",
                    );
                    
                    result.add_issue(issue);
                },
                RuleAction::Log(message) => {
                    let issue = ValidationIssue::new(
                        ValidationSeverity::Info,
                        message.clone(),
                        format!("RULE_{}", rule.id),
                        "rule_engine",
                    );
                    
                    result.add_issue(issue);
                },
                RuleAction::Custom(action) => {
                    // In a real implementation, execute custom action
                    // For now, just add an info message
                    let issue = ValidationIssue::new(
                        ValidationSeverity::Info,
                        format!("Custom action '{}' would execute", action),
                        format!("RULE_{}", rule.id),
                        "rule_engine",
                    );
                    
                    result.add_issue(issue);
                },
            }
        }
        
        Ok(result)
    }
    
    /// Evaluate rules for a resource type
    pub fn evaluate_rules_for_type(
        &self,
        resource_type: &ResourceTypeId,
        context: &ValidationContext,
    ) -> Result<ValidationResult, ValidationRuleError> {
        let mut result = ValidationResult::success();
        
        // Get rules for this resource type
        let rules = self.get_rules_for_type(resource_type)?;
        
        // Evaluate each rule
        for rule in rules {
            let rule_result = self.evaluate_rule(&rule, context)?;
            
            // Merge results
            result.merge(rule_result);
            
            // If this is a critical error and validation failed, stop processing
            if !result.is_valid() && rule.severity == ValidationSeverity::Critical {
                break;
            }
        }
        
        Ok(result)
    }
} 