// State transition validation module
// This file contains components for validating resource state transitions.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use thiserror::Error;
use serde::{Serialize, Deserialize};

use crate::resource::{ResourceId, ResourceTypeId};
use crate::resource::interface::ResourceState;
use super::context::ValidationContext;
use super::result::{ValidationResult, ValidationIssue, ValidationError, ValidationSeverity};

/// Error types specific to state transition validation
#[derive(Error, Debug, Clone)]
pub enum StateTransitionError {
    /// Invalid state transition
    #[error("Invalid state transition from {from} to {to}: {reason}")]
    InvalidTransition {
        from: String,
        to: String,
        reason: String,
    },
    
    /// State not found
    #[error("State not found: {0}")]
    StateNotFound(String),
    
    /// Disallowed transition
    #[error("Transition from {from} to {to} is not allowed")]
    DisallowedTransition {
        from: String,
        to: String,
    },
    
    /// Missing required condition
    #[error("Missing required condition for transition from {from} to {to}: {condition}")]
    MissingCondition {
        from: String,
        to: String,
        condition: String,
    },
    
    /// Internal error
    #[error("Internal state transition error: {0}")]
    InternalError(String),
}

/// A rule governing state transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransitionRule {
    /// Source state
    pub from_state: String,
    
    /// Target state
    pub to_state: String,
    
    /// Whether the transition is allowed
    pub allowed: bool,
    
    /// Additional conditions for the transition
    pub conditions: HashMap<String, String>,
    
    /// Required capabilities for the transition
    pub required_capabilities: Vec<String>,
    
    /// Description of the transition
    pub description: Option<String>,
}

impl StateTransitionRule {
    /// Create a new state transition rule
    pub fn new(from_state: impl Into<String>, to_state: impl Into<String>, allowed: bool) -> Self {
        Self {
            from_state: from_state.into(),
            to_state: to_state.into(),
            allowed,
            conditions: HashMap::new(),
            required_capabilities: Vec::new(),
            description: None,
        }
    }
    
    /// Add a condition to the rule
    pub fn with_condition(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.conditions.insert(key.into(), value.into());
        self
    }
    
    /// Add a required capability
    pub fn with_required_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capabilities.push(capability.into());
        self
    }
    
    /// Add a description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Check if the rule matches the given states
    pub fn matches(&self, from_state: &str, to_state: &str) -> bool {
        self.from_state == from_state && self.to_state == to_state
    }
    
    /// Check if the transition is allowed
    pub fn is_allowed(&self) -> bool {
        self.allowed
    }
    
    /// Check if all conditions are met
    pub fn conditions_met(&self, context: &HashMap<String, String>) -> bool {
        for (key, expected_value) in &self.conditions {
            if let Some(actual_value) = context.get(key) {
                if actual_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        true
    }
    
    /// Get missing conditions
    pub fn missing_conditions(&self, context: &HashMap<String, String>) -> Vec<String> {
        let mut missing = Vec::new();
        
        for (key, expected_value) in &self.conditions {
            if let Some(actual_value) = context.get(key) {
                if actual_value != expected_value {
                    missing.push(format!("{} (expected: {}, actual: {})", key, expected_value, actual_value));
                }
            } else {
                missing.push(format!("{} (missing)", key));
            }
        }
        
        missing
    }
}

/// Validator for resource state transitions
#[derive(Debug)]
pub struct StateTransitionValidator {
    /// Rules by resource type
    rules_by_type: RwLock<HashMap<ResourceTypeId, Vec<StateTransitionRule>>>,
    
    /// Default rules for all resource types
    default_rules: RwLock<Vec<StateTransitionRule>>,
}

impl StateTransitionValidator {
    /// Create a new state transition validator
    pub fn new() -> Self {
        let mut validator = Self {
            rules_by_type: RwLock::new(HashMap::new()),
            default_rules: RwLock::new(Vec::new()),
        };
        
        // Initialize default rules
        let default_rules = vec![
            // Created -> Active
            StateTransitionRule::new("Created", "Active", true)
                .with_description("Activate a newly created resource"),
                
            // Active -> Locked
            StateTransitionRule::new("Active", "Locked", true)
                .with_description("Lock an active resource"),
                
            // Locked -> Active
            StateTransitionRule::new("Locked", "Active", true)
                .with_description("Unlock a locked resource"),
                
            // Active -> Frozen
            StateTransitionRule::new("Active", "Frozen", true)
                .with_description("Freeze an active resource"),
                
            // Frozen -> Active
            StateTransitionRule::new("Frozen", "Active", true)
                .with_description("Unfreeze a frozen resource"),
                
            // Any -> Archived
            StateTransitionRule::new("*", "Archived", true)
                .with_description("Archive a resource"),
                
            // Any -> Consumed (except Archived)
            StateTransitionRule::new("*", "Consumed", true)
                .with_condition("current_state", "!Archived")
                .with_description("Consume a resource"),
                
            // Archived -> Consumed
            StateTransitionRule::new("Archived", "Consumed", false)
                .with_description("Cannot consume an archived resource"),
                
            // Consumed -> Any
            StateTransitionRule::new("Consumed", "*", false)
                .with_description("Cannot transition from a consumed state"),
        ];
        
        let mut rules_lock = validator.default_rules.write().unwrap();
        *rules_lock = default_rules;
        
        validator
    }
    
    /// Add a rule for a specific resource type
    pub fn add_rule_for_type(
        &self,
        resource_type: ResourceTypeId,
        rule: StateTransitionRule,
    ) -> Result<(), StateTransitionError> {
        let mut rules = self.rules_by_type.write().map_err(|e|
            StateTransitionError::InternalError(format!("Failed to acquire rules lock: {}", e))
        )?;
        
        let type_rules = rules.entry(resource_type).or_insert_with(Vec::new);
        type_rules.push(rule);
        
        Ok(())
    }
    
    /// Add a default rule for all resource types
    pub fn add_default_rule(
        &self,
        rule: StateTransitionRule,
    ) -> Result<(), StateTransitionError> {
        let mut rules = self.default_rules.write().map_err(|e|
            StateTransitionError::InternalError(format!("Failed to acquire default rules lock: {}", e))
        )?;
        
        rules.push(rule);
        
        Ok(())
    }
    
    /// Validate a state transition
    pub fn validate_transition(
        &self,
        resource_type: Option<&ResourceTypeId>,
        from_state: &str,
        to_state: &str,
        context: &HashMap<String, String>,
    ) -> Result<ValidationResult, ValidationError> {
        let mut result = ValidationResult::success();
        
        // Check type-specific rules first
        if let Some(resource_type) = resource_type {
            let rules = self.rules_by_type.read().map_err(|e|
                ValidationError::InternalError(format!("Failed to acquire rules lock: {}", e))
            )?;
            
            if let Some(type_rules) = rules.get(resource_type) {
                // Try to find a matching rule
                for rule in type_rules {
                    if rule.matches(from_state, to_state) || 
                       (rule.from_state == "*" && rule.to_state == to_state) ||
                       (rule.from_state == from_state && rule.to_state == "*") {
                        
                        if !rule.is_allowed() {
                            result.add_error(
                                format!("Transition from {} to {} is not allowed", from_state, to_state),
                                "DISALLOWED_TRANSITION",
                                "state_validator",
                            );
                            return Ok(result);
                        }
                        
                        if !rule.conditions_met(context) {
                            let missing = rule.missing_conditions(context);
                            for condition in missing {
                                result.add_error(
                                    format!("Missing condition for transition: {}", condition),
                                    "MISSING_CONDITION",
                                    "state_validator",
                                );
                            }
                            return Ok(result);
                        }
                        
                        // Found a matching rule that allows the transition
                        return Ok(result);
                    }
                }
            }
        }
        
        // Fall back to default rules
        let default_rules = self.default_rules.read().map_err(|e|
            ValidationError::InternalError(format!("Failed to acquire default rules lock: {}", e))
        )?;
        
        // Try to find a matching rule
        for rule in default_rules.iter() {
            if rule.matches(from_state, to_state) || 
               (rule.from_state == "*" && rule.to_state == to_state) ||
               (rule.from_state == from_state && rule.to_state == "*") {
                
                if !rule.is_allowed() {
                    result.add_error(
                        format!("Transition from {} to {} is not allowed", from_state, to_state),
                        "DISALLOWED_TRANSITION",
                        "state_validator",
                    );
                    return Ok(result);
                }
                
                if !rule.conditions_met(context) {
                    let missing = rule.missing_conditions(context);
                    for condition in missing {
                        result.add_error(
                            format!("Missing condition for transition: {}", condition),
                            "MISSING_CONDITION",
                            "state_validator",
                        );
                    }
                    return Ok(result);
                }
                
                // Found a matching rule that allows the transition
                return Ok(result);
            }
        }
        
        // No matching rule found, disallow by default
        result.add_error(
            format!("No rule found for transition from {} to {}", from_state, to_state),
            "NO_TRANSITION_RULE",
            "state_validator",
        );
        
        Ok(result)
    }
    
    /// Get all possible transitions from a state
    pub fn possible_transitions(
        &self,
        resource_type: Option<&ResourceTypeId>,
        from_state: &str,
    ) -> Result<HashSet<String>, StateTransitionError> {
        let mut transitions = HashSet::new();
        
        // Check type-specific rules first
        if let Some(resource_type) = resource_type {
            let rules = self.rules_by_type.read().map_err(|e|
                StateTransitionError::InternalError(format!("Failed to acquire rules lock: {}", e))
            )?;
            
            if let Some(type_rules) = rules.get(resource_type) {
                for rule in type_rules {
                    if (rule.from_state == from_state || rule.from_state == "*") && rule.is_allowed() {
                        if rule.to_state != "*" {
                            transitions.insert(rule.to_state.clone());
                        }
                    }
                }
            }
        }
        
        // Add default rules
        let default_rules = self.default_rules.read().map_err(|e|
            StateTransitionError::InternalError(format!("Failed to acquire default rules lock: {}", e))
        )?;
        
        for rule in default_rules.iter() {
            if (rule.from_state == from_state || rule.from_state == "*") && rule.is_allowed() {
                if rule.to_state != "*" {
                    transitions.insert(rule.to_state.clone());
                }
            }
        }
        
        Ok(transitions)
    }
}

#[async_trait]
impl super::validation::Validator for StateTransitionValidator {
    async fn validate(&self, context: &ValidationContext) -> Result<ValidationResult, ValidationError> {
        // Ensure we have both current and target states
        if context.current_state.is_none() || context.target_state.is_none() {
            return Ok(ValidationResult::incomplete("Missing current or target state"));
        }
        
        let current_state = context.current_state.as_ref().unwrap();
        let target_state = context.target_state.as_ref().unwrap();
        
        // Prepare context for condition checking
        let mut condition_context = HashMap::new();
        
        // Add basic state info
        condition_context.insert("current_state".to_string(), current_state.to_string());
        condition_context.insert("target_state".to_string(), target_state.to_string());
        
        // Add context data as conditions
        for (key, value) in &context.context_data {
            if let Ok(value_str) = String::from_utf8(value.clone()) {
                condition_context.insert(key.clone(), value_str);
            }
        }
        
        // Validate the transition
        self.validate_transition(
            context.resource_type.as_ref(),
            &current_state.to_string(),
            &target_state.to_string(),
            &condition_context,
        )
    }
    
    async fn validate_with_options(
        &self, 
        context: &ValidationContext,
        _options: super::context::ValidationOptions,
    ) -> Result<ValidationResult, ValidationError> {
        // Options don't affect state validation for now
        self.validate(context).await
    }
    
    fn name(&self) -> &str {
        "StateTransitionValidator"
    }
}

/// Helper function to validate a state transition
pub fn validate_state_transition(
    from_state: &ResourceState,
    to_state: &ResourceState,
    resource_type: Option<&ResourceTypeId>,
) -> Result<(), StateTransitionError> {
    let validator = StateTransitionValidator::new();
    let context = HashMap::new();
    
    let result = validator.validate_transition(
        resource_type,
        &from_state.to_string(),
        &to_state.to_string(),
        &context,
    ).map_err(|e| 
        StateTransitionError::InternalError(format!("Validation error: {}", e))
    )?;
    
    if !result.is_valid() {
        if let Some(first_error) = result.errors().first() {
            return Err(StateTransitionError::InvalidTransition {
                from: from_state.to_string(),
                to: to_state.to_string(),
                reason: first_error.message.clone(),
            });
        } else {
            return Err(StateTransitionError::InvalidTransition {
                from: from_state.to_string(),
                to: to_state.to_string(),
                reason: "Unknown validation error".to_string(),
            });
        }
    }
    
    Ok(())
} 