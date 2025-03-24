# Resource Validation

This document outlines the resource validation system within the unified resource architecture, describing how operations on resources are validated for consistency, correctness, and compliance with system constraints.

## Core Concepts

### Resource Validation

Resource validation is the process of ensuring that operations on resources conform to:

1. **Structural Validation**: Ensuring the resource structure is valid
2. **State Validation**: Verifying the resource state allows the operation
3. **Capability Validation**: Checking that the requester has necessary capabilities
4. **Relationship Validation**: Ensuring relationships remain valid
5. **Temporal Validation**: Verifying temporal consistency
6. **Business Logic Validation**: Enforcing domain-specific rules

This multi-layered validation approach ensures system integrity and prevents invalid operations from corrupting resource states.

### Validation Context

```rust
/// Context for validation operations
pub struct ValidationContext {
    /// The operation being validated
    operation: Operation,
    
    /// The current resource states
    resource_states: HashMap<ResourceId, RegisterState>,
    
    /// The capability registry for authorization
    capability_registry: CapabilityRegistry,
    
    /// The relationship tracker for relationship validation
    relationship_tracker: RelationshipTracker,
    
    /// The temporal fact validator for temporal validation
    temporal_validator: TemporalValidator,
    
    /// The resource register for current resource data
    resource_register: ResourceRegister,
    
    /// Custom validators
    custom_validators: Vec<Box<dyn CustomValidator>>,
}
```

### Validation Pipeline

All operations go through a validation pipeline consisting of these stages:

1. **Pre-validation**: Initial checks for basic validity
2. **Structural validation**: Ensure resource structure is valid
3. **State validation**: Check resource states allow the operation
4. **Capability validation**: Verify required capabilities
5. **Relationship validation**: Ensure relationships remain valid
6. **Temporal validation**: Verify temporal consistency
7. **Business logic validation**: Apply domain-specific rules
8. **Custom validation**: Run custom validators
9. **Post-validation**: Final holistic checks

## Validator Components

The validation system consists of several specialized components:

```rust
/// The main resource validator
pub struct ResourceValidator {
    /// Validates resource structure
    structural_validator: StructuralValidator,
    
    /// Validates resource states
    state_validator: StateValidator,
    
    /// Validates capabilities
    capability_validator: CapabilityValidator,
    
    /// Validates relationships
    relationship_validator: RelationshipValidator,
    
    /// Validates temporal aspects
    temporal_validator: TemporalValidator,
    
    /// Validates business logic
    business_logic_validator: BusinessLogicValidator,
    
    /// Registry of custom validators
    custom_validator_registry: CustomValidatorRegistry,
    
    /// Configuration for validation
    config: ValidationConfig,
}

/// Configuration for the validator
pub struct ValidationConfig {
    /// Whether to validate structure
    validate_structure: bool,
    
    /// Whether to validate state
    validate_state: bool,
    
    /// Whether to validate capabilities
    validate_capabilities: bool,
    
    /// Whether to validate relationships
    validate_relationships: bool,
    
    /// Whether to validate temporal aspects
    validate_temporal: bool,
    
    /// Whether to validate business logic
    validate_business_logic: bool,
    
    /// Whether to apply custom validators
    apply_custom_validators: bool,
}
```

## Integration with Operation Model

The validator integrates with the unified operation model:

1. **Operation Validation**: Validates operations before execution
2. **Transaction Validation**: Validates entire transactions
3. **Effect Template Integration**: Provides validation for effect templates
4. **Custom Validation Rules**: Allows domain-specific validation

## Usage Examples

### Basic Operation Validation

```rust
// Create a resource validator
let validator = ResourceValidator::new(
    ValidationConfig::default()
        .with_validate_structure(true)
        .with_validate_state(true)
        .with_validate_capabilities(true)
        .with_validate_relationships(true)
        .with_validate_temporal(true)
        .with_validate_business_logic(true)
);

// Create an operation to validate
let operation = Operation::new(OperationType::TransferResource)
    .with_input(source_resource.clone())
    .with_output(destination_resource.clone())
    .with_parameter("amount", 100u64)
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::with_capabilities(
        invoker.clone(),
        vec![transfer_capability]
    ));

// Create a validation context
let context = ValidationContext::new()
    .with_operation(operation.clone())
    .with_resource_states(resource_states.clone())
    .with_capability_registry(capability_registry.clone())
    .with_relationship_tracker(relationship_tracker.clone())
    .with_temporal_validator(temporal_validator.clone())
    .with_resource_register(resource_register.clone());

// Validate the operation
let validation_result = validator.validate(&context)?;

if validation_result.is_valid() {
    // Execute the operation
    let result = execute_operation(operation, &execution_context).await?;
    // Process the result
} else {
    // Handle validation failure
    println!("Validation failed: {}", validation_result.error_message().unwrap());
    for issue in validation_result.issues() {
        println!("  - {}: {}", issue.severity, issue.message);
    }
}
```

### Validating Resource Structure

```rust
// Create a structural validator
let structural_validator = StructuralValidator::new();

// Validate a resource structure
let validation_result = structural_validator.validate_resource(&resource)?;

if !validation_result.is_valid() {
    // Handle structural validation issues
    println!("Structural validation failed: {}", validation_result.error_message().unwrap());
    for issue in validation_result.issues() {
        println!("  - {}: {}", issue.severity, issue.message);
    }
}
```

### State Validation

```rust
// Create a state validator
let state_validator = StateValidator::new();

// Define an operation and its required states
let operation_type = OperationType::TransferResource;
let required_states = vec![RegisterState::Active];

// Validate resource state for an operation
let resource_id = resource.id();
let current_state = resource_states.get(&resource_id).unwrap();

let validation_result = state_validator.validate_state(
    resource_id,
    current_state,
    &operation_type,
    &required_states
)?;

if !validation_result.is_valid() {
    // Handle state validation issues
    println!("State validation failed: Resource state {:?} does not allow operation {:?}",
             current_state, operation_type);
}
```

### Capability Validation

```rust
// Create a capability validator
let capability_validator = CapabilityValidator::new();

// Validate capabilities for an operation
let validation_result = capability_validator.validate(
    &operation.authorization().capabilities(),
    &operation
)?;

if !validation_result.is_valid() {
    // Handle capability validation issues
    println!("Capability validation failed: {}", validation_result.error_message().unwrap());
    
    // Get missing capabilities
    let missing_capabilities = validation_result
        .get_value::<Vec<Right>>("missing_rights")
        .unwrap_or_default();
    
    println!("Missing capabilities: {:?}", missing_capabilities);
}
```

### Relationship Validation

```rust
// Create a relationship validator
let relationship_validator = RelationshipValidator::new(relationship_tracker.clone());

// Validate relationships for a resource operation
let resource_id = resource.id();
let operation_type = OperationType::ConsumeResource;

let validation_result = relationship_validator.validate_for_operation(
    &resource_id,
    &operation_type
)?;

if !validation_result.is_valid() {
    // Handle relationship validation issues
    println!("Relationship validation failed: {}", validation_result.error_message().unwrap());
    
    // Get blocking relationships
    let blocking_relationships = validation_result
        .get_value::<Vec<(ResourceId, RelationshipType)>>("blocking_relationships")
        .unwrap_or_default();
    
    println!("Blocking relationships: {:?}", blocking_relationships);
}
```

### Temporal Validation

```rust
// Create a temporal validator
let temporal_validator = TemporalValidator::new();

// Validate temporal aspects of an operation
let validation_result = temporal_validator.validate(&operation)?;

if !validation_result.is_valid() {
    // Handle temporal validation issues
    println!("Temporal validation failed: {}", validation_result.error_message().unwrap());
    
    // Get invalid temporal dependencies
    let invalid_dependencies = validation_result
        .get_value::<Vec<FactId>>("invalid_dependencies")
        .unwrap_or_default();
    
    println!("Invalid temporal dependencies: {:?}", invalid_dependencies);
}
```

### Business Logic Validation

```rust
// Create a business logic validator with custom rules
let business_logic_validator = BusinessLogicValidator::new()
    .with_rule("transfer_limit", Box::new(|operation, context| {
        // Check if operation is a transfer
        if operation.operation_type() != OperationType::TransferResource {
            return Ok(ValidationResult::valid());
        }
        
        // Get transfer amount
        let amount = operation.get_parameter::<u64>("amount").unwrap_or(0);
        
        // Apply a business rule: transfers over 1000 are not allowed
        if amount > 1000 {
            return Ok(ValidationResult::invalid(
                "Transfer amount exceeds the maximum limit of 1000"
            ));
        }
        
        Ok(ValidationResult::valid())
    }));

// Validate against business logic
let validation_result = business_logic_validator.validate(&operation, &context)?;

if !validation_result.is_valid() {
    // Handle business logic validation issues
    println!("Business logic validation failed: {}", validation_result.error_message().unwrap());
}
```

### Custom Validators

```rust
// Create a custom validator
struct TransferLimitValidator;

impl CustomValidator for TransferLimitValidator {
    fn validate(&self, operation: &Operation, context: &ValidationContext) -> Result<ValidationResult> {
        // Only apply to transfer operations
        if operation.operation_type() != OperationType::TransferResource {
            return Ok(ValidationResult::valid());
        }
        
        // Get transfer amount
        let amount = operation
            .get_parameter::<u64>("amount")
            .unwrap_or(0);
        
        // Get the source resource
        let source_resource = match operation.inputs().first() {
            Some(input) => input.as_resource()?,
            None => return Ok(ValidationResult::invalid("Missing source resource")),
        };
        
        // Apply resource-specific transfer limits
        let resource_type = source_resource.resource_type();
        let max_transfer = match resource_type.as_str() {
            "token.fungible" => 10000,
            "token.non_fungible" => 5,
            "certificate" => 1,
            _ => 100, // Default limit
        };
        
        if amount > max_transfer {
            return Ok(ValidationResult::invalid(format!(
                "Transfer amount {} exceeds the maximum limit of {} for resource type {}",
                amount, max_transfer, resource_type
            )));
        }
        
        Ok(ValidationResult::valid())
    }
    
    fn id(&self) -> &str {
        "transfer_limit_validator"
    }
    
    fn description(&self) -> &str {
        "Validates that transfer amounts don't exceed type-specific limits"
    }
}

// Register the custom validator
validator.register_custom_validator(Box::new(TransferLimitValidator));
```

### Batch Validation

```rust
// Validate a batch of operations
let operations = vec![
    operation1.clone(),
    operation2.clone(),
    operation3.clone(),
];

// Validate the entire batch
let batch_result = validator.validate_batch(&operations, &context)?;

if batch_result.is_valid() {
    // All operations are valid
    println!("All operations are valid");
} else {
    // Handle batch validation issues
    println!("Batch validation failed");
    
    // Get results for individual operations
    let individual_results = batch_result.individual_results();
    for (i, result) in individual_results.iter().enumerate() {
        if !result.is_valid() {
            println!("Operation {} failed validation: {}", 
                     i, result.error_message().unwrap());
        }
    }
}
```

### Validation Results

```rust
/// Result of a validation operation
pub struct ValidationResult {
    /// Whether the validation passed
    valid: bool,
    
    /// The error message if validation failed
    error_message: Option<String>,
    
    /// Detailed validation issues
    issues: Vec<ValidationIssue>,
    
    /// Additional metadata about the validation result
    metadata: MetadataMap,
}

/// A validation issue
pub struct ValidationIssue {
    /// The severity of the issue
    severity: ValidationSeverity,
    
    /// The issue code
    code: String,
    
    /// The issue message
    message: String,
    
    /// The location of the issue (if applicable)
    location: Option<String>,
    
    /// Additional context about the issue
    context: MetadataMap,
}

/// Severity levels for validation issues
pub enum ValidationSeverity {
    /// Informational issue - doesn't affect validity
    Info,
    
    /// Warning issue - doesn't affect validity but may indicate a problem
    Warning,
    
    /// Error issue - affects validity
    Error,
    
    /// Critical issue - affects validity and requires immediate attention
    Critical,
}
```

## Validation Rules Registry

A central registry maintains validation rules:

```rust
// Get validation rules for a specific resource type
let token_rules = validation_rule_registry.get_rules_for_resource_type("token.fungible")?;

// Get validation rules for a specific operation type
let transfer_rules = validation_rule_registry.get_rules_for_operation(OperationType::TransferResource)?;

// Register a new validation rule
validation_rule_registry.register_rule(
    "token.fungible",
    OperationType::TransferResource,
    Box::new(MyValidationRule::new())
)?;
```

## Best Practices

1. **Validate Early**: Apply validation as early as possible in the operation lifecycle.

2. **Complete Validation**: Ensure all validation types are applied for critical operations.

3. **Fail Fast**: Use early-return patterns for efficiency when validation failures are detected.

4. **Comprehensive Results**: Return detailed validation results with clear error messages.

5. **Custom Validators**: Use custom validators for domain-specific validation needs.

6. **Transaction Boundaries**: Consider entire transactions when validating operations.

7. **Temporal Consistency**: Ensure operations maintain temporal consistency.

8. **Relationship Integrity**: Validate that relationships remain valid after operations.

9. **Capability Verification**: Always verify that requesters have necessary capabilities.

10. **Metadata Validation**: Validate metadata against schemas where appropriate.

## Implementation Status

The resource validation system is fully implemented in the Causality system:

- ✅ Core `ResourceValidator` structure
- ✅ Structural validation
- ✅ State validation
- ✅ Capability validation
- ✅ Relationship validation
- ✅ Temporal validation
- ✅ Business logic validation
- ✅ Custom validator system
- ✅ Batch validation
- ✅ Validation rule registry

## Future Enhancements

1. **Declarative Validation Rules**: Support for declarative validation rules
2. **Validation Pipelines**: More sophisticated validation pipelines with hooks
3. **Validation Caching**: Caching validation results for performance
4. **Predictive Validation**: Pre-validate likely operations for improved UX
5. **Machine Learning-Based Validation**: Using ML for anomaly detection in operations 