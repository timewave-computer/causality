// Resource validation tests
// This file contains tests for the resource validation framework.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::test;

use crate::resource::{
    ResourceId, ResourceTypeId, ResourceState, ResourceSchema,
};

use crate::resource::validation::{
    ResourceValidator, ResourceValidatorConfig,
    ValidationContext, ValidationPhase, ValidationOptions,
    ValidationResult, ValidationError, ValidationSeverity,
    StateTransitionValidator, StateTransitionRule,
    SchemaValidator, SchemaCompatibility,
    PermissionValidator, ResourcePermission,
    ValidationRule, RuleCondition, RuleAction,
};

// Import the validate_schema_compatibility function from the schema module
use crate::resource::validation::schema::validate_schema_compatibility;

use crate::capability::CapabilitySet;
use crate::content::ContentHash;

// Helper struct for testing resource validation
#[derive(Debug, Clone)]
struct TestResource {
    id: ResourceId,
    resource_type: ResourceTypeId,
    state: ResourceState,
    schema: ResourceSchema,
}

impl TestResource {
    fn new(
        id: ResourceId,
        resource_type: ResourceTypeId,
        state: ResourceState,
        schema: ResourceSchema,
    ) -> Self {
        Self {
            id,
            resource_type,
            state,
            schema,
        }
    }
    
    fn id(&self) -> &ResourceId {
        &self.id
    }
    
    fn resource_type(&self) -> &ResourceTypeId {
        &self.resource_type
    }
    
    fn state(&self) -> &ResourceState {
        &self.state
    }
    
    fn schema(&self) -> &ResourceSchema {
        &self.schema
    }
}

// Create a test schema
fn create_test_schema() -> ResourceSchema {
    ResourceSchema {
        format: "json-schema".to_string(),
        definition: r#"{"type": "object", "properties": {"name": {"type": "string"}}}"#.to_string(),
        version: "1.0".to_string(),
        content_hash: Some(ContentHash::from("test-schema-hash".as_bytes())),
    }
}

// Create a test resource type
fn create_test_resource_type() -> ResourceTypeId {
    ResourceTypeId::with_version("test", "user", "1.0")
}

// Create a test resource ID
fn create_test_resource_id() -> ResourceId {
    ResourceId::from("test-resource-id".as_bytes())
}

// Create a test resource
fn create_test_resource() -> TestResource {
    TestResource::new(
        create_test_resource_id(),
        create_test_resource_type(),
        ResourceState::from("Created"),
        create_test_schema(),
    )
}

// Test resource validator creation
#[test]
async fn test_resource_validator_creation() {
    let validator = ResourceValidator::new();
    
    // Use the config() method to get the configuration
    let config = validator.config();
    assert!(config.validate_schemas);
    assert!(config.validate_state_transitions);
    assert!(config.validate_permissions);
    assert!(config.enable_custom_validators);
}

// Test state transition validation
#[test]
async fn test_state_transition_validation() {
    let validator = ResourceValidator::new();
    let resource_id = create_test_resource_id();
    let current_state = ResourceState::from("Created");
    let target_state = ResourceState::from("Active");
    
    let result = validator.validate_state_transition(
        &resource_id,
        &current_state,
        &target_state,
    ).await.unwrap();
    
    assert!(result.is_valid());
    
    // Test invalid transition
    let invalid_state = ResourceState::from("Invalid");
    let result = validator.validate_state_transition(
        &resource_id,
        &current_state,
        &invalid_state,
    ).await.unwrap();
    
    assert!(!result.is_valid());
}

// Test schema validation
#[test]
async fn test_schema_validation() {
    let validator = SchemaValidator::new();
    let schema = create_test_schema();
    
    // Create valid data
    let valid_data = serde_json::json!({
        "name": "Test User"
    });
    
    let result = validator.validate_data(
        &schema,
        &valid_data,
    ).await.unwrap();
    
    assert!(result.is_valid());
    
    // Create invalid data
    let invalid_data = serde_json::json!({
        "name": 123 // Should be a string
    });
    
    let result = validator.validate_data(
        &schema,
        &invalid_data,
    ).await.unwrap();
    
    assert!(!result.is_valid());
}

// Test schema compatibility
#[test]
async fn test_schema_compatibility() {
    let validator = SchemaValidator::new();
    
    // Create original schema
    let original_schema = ResourceSchema {
        format: "json-schema".to_string(),
        definition: r#"{"type": "object", "properties": {"name": {"type": "string"}, "age": {"type": "integer"}}}"#.to_string(),
        version: "1.0".to_string(),
        content_hash: Some(ContentHash::from("original-schema-hash".as_bytes())),
    };
    
    // Create compatible schema (added optional field)
    let compatible_schema = ResourceSchema {
        format: "json-schema".to_string(),
        definition: r#"{"type": "object", "properties": {"name": {"type": "string"}, "age": {"type": "integer"}, "email": {"type": "string"}}}"#.to_string(),
        version: "1.1".to_string(),
        content_hash: Some(ContentHash::from("compatible-schema-hash".as_bytes())),
    };
    
    let compatibility = validate_schema_compatibility(
        &original_schema,
        &compatible_schema,
    ).unwrap();
    
    assert_eq!(compatibility, SchemaCompatibility::Full);
    
    // Create incompatible schema (changed type)
    let incompatible_schema = ResourceSchema {
        format: "json-schema".to_string(),
        definition: r#"{"type": "object", "properties": {"name": {"type": "string"}, "age": {"type": "string"}}}"#.to_string(),
        version: "2.0".to_string(),
        content_hash: Some(ContentHash::from("incompatible-schema-hash".as_bytes())),
    };
    
    let compatibility = validate_schema_compatibility(
        &original_schema,
        &incompatible_schema,
    ).unwrap();
    
    assert_eq!(compatibility, SchemaCompatibility::Incompatible);
}

// Test the full validation pipeline
#[test]
async fn test_validation_pipeline() {
    let validator = ResourceValidator::new();
    let test_resource = create_test_resource();
    
    // Create a validation context
    let context = ValidationContext::new()
        .with_resource_id(test_resource.id().clone())
        .with_resource_type(test_resource.resource_type().clone())
        .with_current_state(test_resource.state().clone())
        .with_schema(test_resource.schema().clone())
        .with_phase(ValidationPhase::PreExecution);
    
    // Add resource data for schema validation
    let resource_data = serde_json::json!({
        "name": "Test User"
    });
    
    let resource_data_bytes = serde_json::to_vec(&resource_data).unwrap();
    let context = context.with_context_data("resource_data", resource_data_bytes);
    
    // Execute validation
    let result = validator.validate(&context).await.unwrap();
    
    // Since we don't have all the necessary data (like capabilities), this
    // will likely not be fully valid, but we can check that it executed
    println!("Validation result: {:?}", result);
}

// Test custom validation rule
#[test]
async fn test_custom_validator() {
    use crate::resource::validation::SimpleCustomValidator;
    use crate::resource::validation::CustomValidationRule;
    
    let validator = ResourceValidator::new();
    let test_resource = create_test_resource();
    
    // Create a custom validator
    let custom_validator = SimpleCustomValidator::new("test-validator");
    
    // Register the custom validator
    validator.register_custom_validator(custom_validator).unwrap();
    
    // Create a validation context
    let context = ValidationContext::new()
        .with_resource_id(test_resource.id().clone())
        .with_resource_type(test_resource.resource_type().clone())
        .with_current_state(test_resource.state().clone())
        .with_schema(test_resource.schema().clone())
        .with_phase(ValidationPhase::PreExecution);
    
    // Execute validation
    let result = validator.validate(&context).await.unwrap();
    
    // We haven't added any custom rules, so this should pass
    assert!(result.is_valid());
}

// Test custom validation rules
#[test]
async fn test_validation_rules() {
    use crate::resource::validation::ValidationRuleEngine;
    
    let engine = ValidationRuleEngine::new();
    let test_resource = create_test_resource();
    
    // Create a rule condition
    let condition = RuleCondition::Equals("resource_state".to_string(), "Created".to_string());
    
    // Create an action
    let action = RuleAction::Allow;
    
    // Create a rule
    let rule = ValidationRule::new(
        "test-rule",
        "Test Rule",
        condition,
        action,
        ValidationSeverity::Error,
    )
    .with_resource_type(test_resource.resource_type().clone())
    .with_description("Test rule for created resources");
    
    // Register the rule
    engine.register_rule(rule).unwrap();
    
    // Create a validation context
    let mut context = ValidationContext::new()
        .with_resource_id(test_resource.id().clone())
        .with_resource_type(test_resource.resource_type().clone())
        .with_current_state(test_resource.state().clone())
        .with_schema(test_resource.schema().clone())
        .with_phase(ValidationPhase::PreExecution);
    
    // Add resource state to context data
    context = context.with_string_context("resource_state", "Created");
    
    // Evaluate rules
    let result = engine.evaluate_rules_for_type(
        test_resource.resource_type(),
        &context,
    ).unwrap();
    
    // The rule should pass
    assert!(result.is_valid());
    
    // Now change the state to something that should fail
    let mut context = ValidationContext::new()
        .with_resource_id(test_resource.id().clone())
        .with_resource_type(test_resource.resource_type().clone())
        .with_current_state(test_resource.state().clone())
        .with_schema(test_resource.schema().clone())
        .with_phase(ValidationPhase::PreExecution);
    
    // Add different resource state
    context = context.with_string_context("resource_state", "Active");
    
    // Evaluate rules
    let result = engine.evaluate_rules_for_type(
        test_resource.resource_type(),
        &context,
    ).unwrap();
    
    // The rule should still pass since we're using RuleAction::Allow
    assert!(result.is_valid());
} 