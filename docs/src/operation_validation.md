# Operation Validation

This document outlines the operation validation system within the unified resource architecture, describing how operations are validated based on the unified operation model.

## Core Concepts

### Operation Validation

Operation validation ensures that operations conform to:

1. **Structural Validation**: Ensuring the operation structure is valid
2. **Semantic Validation**: Verifying the operation makes sense in context
3. **Authorization Validation**: Checking that the operation is properly authorized
4. **Resource Validation**: Ensuring the operation is valid for the involved resources
5. **Temporal Validation**: Verifying temporal consistency of the operation
6. **Transaction Validation**: Validating the operation within its transaction context

This multi-layered validation approach ensures that operations are valid before they are executed, maintaining system integrity.

### Operation Model

The validation system works with the unified operation model:

```rust
/// A unified operation in the system
pub struct Operation {
    /// Unique identifier for the operation
    id: OperationId,
    
    /// The type of operation
    operation_type: OperationType,
    
    /// Abstract representation of the operation
    abstract_representation: Box<dyn Effect>,
    
    /// Concrete implementation of the operation
    concrete_implementation: Option<ResourceRegisterOperation>,
    
    /// Execution context for the operation
    context: ExecutionContext,
    
    /// Input resources for the operation
    inputs: Vec<ResourceHandle>,
    
    /// Output resources for the operation
    outputs: Vec<ResourceHandle>,
    
    /// Authorization for the operation
    authorization: Authorization,
    
    /// Parameters for the operation
    parameters: ParameterMap,
    
    /// Temporal dependencies for the operation
    temporal_dependencies: Vec<FactId>,
    
    /// Metadata for the operation
    metadata: MetadataMap,
}

/// The execution context for an operation
pub struct ExecutionContext {
    /// The phase of execution
    phase: ExecutionPhase,
    
    /// The environment for execution
    environment: ExecutionEnvironment,
    
    /// The domain where execution occurs
    domain: Option<DomainId>,
    
    /// Transaction context if part of a transaction
    transaction_context: Option<TransactionContext>,
    
    /// Verification context for the operation
    verification_context: Option<VerificationContext>,
}

/// The phase of operation execution
pub enum ExecutionPhase {
    /// Planning phase - operation is being prepared
    Planning,
    
    /// Validation phase - operation is being validated
    Validation,
    
    /// Execution phase - operation is being executed
    Execution,
    
    /// Commitment phase - operation results are being committed
    Commitment,
    
    /// Verification phase - operation is being verified
    Verification,
}
```

## Validation Components

The operation validation system consists of several specialized components:

```rust
/// The main operation validator
pub struct OperationValidator {
    /// Validates operation structure
    structural_validator: StructuralValidator,
    
    /// Validates operation semantics
    semantic_validator: SemanticValidator,
    
    /// Validates operation authorization
    authorization_validator: AuthorizationValidator,
    
    /// Validates resources in operations
    resource_validator: ResourceValidator,
    
    /// Validates temporal aspects of operations
    temporal_validator: TemporalValidator,
    
    /// Validates transaction context
    transaction_validator: TransactionValidator,
    
    /// Registry of operation-specific validators
    operation_validator_registry: OperationValidatorRegistry,
    
    /// Configuration for validation
    config: OperationValidationConfig,
}

/// Configuration for the operation validator
pub struct OperationValidationConfig {
    /// Whether to validate structure
    validate_structure: bool,
    
    /// Whether to validate semantics
    validate_semantics: bool,
    
    /// Whether to validate authorization
    validate_authorization: bool,
    
    /// Whether to validate resources
    validate_resources: bool,
    
    /// Whether to validate temporal aspects
    validate_temporal: bool,
    
    /// Whether to validate transaction context
    validate_transaction: bool,
    
    /// Whether to use operation-specific validators
    use_operation_specific_validators: bool,
}
```

## Integration with Resource System

The operation validator integrates with the resource system:

1. **ResourceRegister**: Validates operations against the resource register
2. **Lifecycle Management**: Ensures operations respect lifecycle states
3. **Relationship Tracking**: Validates relationships remain valid after operations
4. **Capability System**: Verifies required capabilities for operations
5. **Effect Templates**: Validates effect template instantiations

## Usage Examples

### Basic Operation Validation

```rust
// Create an operation validator
let validator = OperationValidator::new(
    OperationValidationConfig::default()
        .with_validate_structure(true)
        .with_validate_semantics(true)
        .with_validate_authorization(true)
        .with_validate_resources(true)
        .with_validate_temporal(true)
        .with_validate_transaction(true)
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

// Validate the operation
let validation_result = validator.validate(&operation)?;

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

### Structural Validation

```rust
// Create a structural validator
let structural_validator = StructuralValidator::new();

// Validate operation structure
let validation_result = structural_validator.validate(&operation)?;

if !validation_result.is_valid() {
    // Handle structural validation issues
    println!("Structural validation failed: {}", validation_result.error_message().unwrap());
    
    // Get structural validation details
    let missing_fields = validation_result
        .get_value::<Vec<String>>("missing_fields")
        .unwrap_or_default();
    
    println!("Missing fields: {:?}", missing_fields);
}
```

### Semantic Validation

```rust
// Create a semantic validator
let semantic_validator = SemanticValidator::new();

// Validate operation semantics
let validation_result = semantic_validator.validate(&operation)?;

if !validation_result.is_valid() {
    // Handle semantic validation issues
    println!("Semantic validation failed: {}", validation_result.error_message().unwrap());
    
    // Get semantic validation details
    let semantic_issues = validation_result
        .get_value::<Vec<String>>("semantic_issues")
        .unwrap_or_default();
    
    println!("Semantic issues: {:?}", semantic_issues);
}
```

### Authorization Validation

```rust
// Create an authorization validator
let authorization_validator = AuthorizationValidator::new(
    capability_registry.clone()
);

// Validate operation authorization
let validation_result = authorization_validator.validate(&operation)?;

if !validation_result.is_valid() {
    // Handle authorization validation issues
    println!("Authorization validation failed: {}", validation_result.error_message().unwrap());
    
    // Get authorization validation details
    let missing_capabilities = validation_result
        .get_value::<Vec<Right>>("missing_capabilities")
        .unwrap_or_default();
    
    println!("Missing capabilities: {:?}", missing_capabilities);
}
```

### Resource Validation

```rust
// Create a resource validator
let resource_validator = ResourceValidator::new(
    resource_register.clone(),
    relationship_tracker.clone(),
    lifecycle_manager.clone()
);

// Validate operation resources
let validation_result = resource_validator.validate(&operation)?;

if !validation_result.is_valid() {
    // Handle resource validation issues
    println!("Resource validation failed: {}", validation_result.error_message().unwrap());
    
    // Get resource validation details
    let resource_issues = validation_result
        .get_value::<Vec<(ResourceId, String)>>("resource_issues")
        .unwrap_or_default();
    
    for (resource_id, issue) in resource_issues {
        println!("Issue with resource {:?}: {}", resource_id, issue);
    }
}
```

### Temporal Validation

```rust
// Create a temporal validator
let temporal_validator = TemporalValidator::new(
    fact_store.clone(),
    time_map.clone()
);

// Validate temporal aspects of the operation
let validation_result = temporal_validator.validate(&operation)?;

if !validation_result.is_valid() {
    // Handle temporal validation issues
    println!("Temporal validation failed: {}", validation_result.error_message().unwrap());
    
    // Get temporal validation details
    let temporal_issues = validation_result
        .get_value::<Vec<String>>("temporal_issues")
        .unwrap_or_default();
    
    println!("Temporal issues: {:?}", temporal_issues);
}
```

### Transaction Validation

```rust
// Create a transaction validator
let transaction_validator = TransactionValidator::new();

// Create a transaction context
let transaction_context = TransactionContext::new()
    .with_transaction_id(transaction_id.clone())
    .with_operations(vec![
        previous_operation1.clone(),
        previous_operation2.clone(),
        operation.clone(),
    ]);

// Validate operation in transaction context
let validation_result = transaction_validator.validate(&operation, &transaction_context)?;

if !validation_result.is_valid() {
    // Handle transaction validation issues
    println!("Transaction validation failed: {}", validation_result.error_message().unwrap());
    
    // Get transaction validation details
    let transaction_issues = validation_result
        .get_value::<Vec<String>>("transaction_issues")
        .unwrap_or_default();
    
    println!("Transaction issues: {:?}", transaction_issues);
}
```

### Phase-Specific Validation

```rust
// Different validation rules can apply based on execution phase
match operation.context().phase() {
    ExecutionPhase::Planning => {
        // Validate planning-phase constraints
        let planning_result = validator.validate_planning_phase(&operation)?;
        if !planning_result.is_valid() {
            // Handle planning phase validation issues
            return Err(ValidationError::new("Planning phase validation failed"));
        }
    },
    ExecutionPhase::Validation => {
        // Validate validation-phase constraints
        let validation_result = validator.validate_validation_phase(&operation)?;
        if !validation_result.is_valid() {
            // Handle validation phase issues
            return Err(ValidationError::new("Validation phase validation failed"));
        }
    },
    ExecutionPhase::Execution => {
        // Validate execution-phase constraints
        let execution_result = validator.validate_execution_phase(&operation)?;
        if !execution_result.is_valid() {
            // Handle execution phase issues
            return Err(ValidationError::new("Execution phase validation failed"));
        }
    },
    ExecutionPhase::Commitment => {
        // Validate commitment-phase constraints
        let commitment_result = validator.validate_commitment_phase(&operation)?;
        if !commitment_result.is_valid() {
            // Handle commitment phase issues
            return Err(ValidationError::new("Commitment phase validation failed"));
        }
    },
    ExecutionPhase::Verification => {
        // Validate verification-phase constraints
        let verification_result = validator.validate_verification_phase(&operation)?;
        if !verification_result.is_valid() {
            // Handle verification phase issues
            return Err(ValidationError::new("Verification phase validation failed"));
        }
    },
}
```

### Operation-Type Specific Validation

```rust
// Register operation-specific validators
let transfer_validator = TransferOperationValidator::new();
let create_validator = CreateOperationValidator::new();
let consume_validator = ConsumeOperationValidator::new();

validator.register_operation_validator(
    OperationType::TransferResource,
    Box::new(transfer_validator)
);
validator.register_operation_validator(
    OperationType::CreateResource,
    Box::new(create_validator)
);
validator.register_operation_validator(
    OperationType::ConsumeResource,
    Box::new(consume_validator)
);

// Validation will now automatically use the appropriate validator for each operation type
let transfer_result = validator.validate(&transfer_operation)?;
let create_result = validator.validate(&create_operation)?;
let consume_result = validator.validate(&consume_operation)?;
```

### Validation Pipelines

```rust
// Create a validation pipeline
let pipeline = ValidationPipeline::new()
    .add_step("structure", Box::new(structural_validator.clone()))
    .add_step("semantics", Box::new(semantic_validator.clone()))
    .add_step("authorization", Box::new(authorization_validator.clone()))
    .add_step("resources", Box::new(resource_validator.clone()))
    .add_step("temporal", Box::new(temporal_validator.clone()))
    .add_step("transaction", Box::new(transaction_validator.clone()));

// Run the validation pipeline
let pipeline_result = pipeline.execute(&operation)?;

// Check overall result
if pipeline_result.is_valid() {
    // All validations passed
    println!("Operation passed all validation checks");
} else {
    // Find which step failed
    let failed_step = pipeline_result.failed_step().unwrap();
    let step_result = pipeline_result.step_result(failed_step).unwrap();
    
    println!("Validation failed at step {}: {}", 
             failed_step, step_result.error_message().unwrap());
}
```

### Batch Operation Validation

```rust
// Validate a batch of operations
let operations = vec![
    operation1.clone(),
    operation2.clone(),
    operation3.clone(),
];

// Validate the entire batch
let batch_result = validator.validate_batch(&operations)?;

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

## Specialized Operation Types

Different operation types have specialized validation requirements:

### Resource Creation Operations

```rust
// Validate a resource creation operation
let create_operation = Operation::new(OperationType::CreateResource)
    .with_output(ResourceRegister::new(
        "resource1",
        ResourceProperties::new()
            .with_fungibility_domain("token")
            .with_quantity(100),
        FungibleTokenLogic::new(),
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    ))
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(creator.clone()));

// Specific validation for creation operations
let validation_result = create_validator.validate(&create_operation)?;

if !validation_result.is_valid() {
    // Handle creation validation issues
    println!("Creation validation failed: {}", validation_result.error_message().unwrap());
}
```

### Resource Transfer Operations

```rust
// Validate a resource transfer operation
let transfer_operation = Operation::new(OperationType::TransferResource)
    .with_input(source_resource.clone())
    .with_output(destination_resource.clone())
    .with_parameter("amount", 100u64)
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::with_capabilities(
        invoker.clone(),
        vec![transfer_capability]
    ));

// Specific validation for transfer operations
let validation_result = transfer_validator.validate(&transfer_operation)?;

if !validation_result.is_valid() {
    // Handle transfer validation issues
    println!("Transfer validation failed: {}", validation_result.error_message().unwrap());
}
```

### Resource Lifecycle Operations

```rust
// Validate a resource freezing operation
let freeze_operation = Operation::new(OperationType::FreezeResource)
    .with_input(resource.clone())
    .with_output(resource.with_state(RegisterState::Frozen {
        freezer: Some(admin_id.clone()),
        reason: Some("Security review".to_string()),
    }))
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::with_capabilities(
        admin.clone(),
        vec![freeze_capability]
    ));

// Create a lifecycle operation validator
let lifecycle_validator = LifecycleOperationValidator::new(
    lifecycle_manager.clone()
);

// Validate the lifecycle operation
let validation_result = lifecycle_validator.validate(&freeze_operation)?;

if !validation_result.is_valid() {
    // Handle lifecycle validation issues
    println!("Lifecycle validation failed: {}", validation_result.error_message().unwrap());
}
```

### Cross-Domain Operations

```rust
// Validate a cross-domain operation
let cross_domain_operation = Operation::new(OperationType::CrossDomainTransfer)
    .with_input(source_resource.clone())
    .with_parameter("target_domain", target_domain_id.to_string())
    .with_parameter("recipient", recipient_address.to_string())
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::with_capabilities(
        invoker.clone(),
        vec![cross_domain_capability]
    ));

// Create a cross-domain operation validator
let cross_domain_validator = CrossDomainOperationValidator::new(
    domain_registry.clone()
);

// Validate the cross-domain operation
let validation_result = cross_domain_validator.validate(&cross_domain_operation)?;

if !validation_result.is_valid() {
    // Handle cross-domain validation issues
    println!("Cross-domain validation failed: {}", validation_result.error_message().unwrap());
}
```

### Operation Transformation Validation

```rust
// Validate the transformation of an abstract operation to concrete implementation
let operation = Operation::new(OperationType::TransferResource)
    .with_input(source_resource.clone())
    .with_output(destination_resource.clone())
    .with_parameter("amount", 100u64)
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::with_capabilities(
        invoker.clone(),
        vec![transfer_capability]
    ));

// Create a transformation validator
let transformation_validator = TransformationValidator::new();

// Generate a concrete implementation
let concrete_implementation = ResourceRegisterOperation::new(
    RegisterOperationType::UpdateRegister,
    // Implementation details
);

// Validate the transformation
let validation_result = transformation_validator.validate_transformation(
    &operation,
    &concrete_implementation
)?;

if !validation_result.is_valid() {
    // Handle transformation validation issues
    println!("Transformation validation failed: {}", validation_result.error_message().unwrap());
}
```

## Effect Integration Validation

```rust
// Validate integration with the effect system
let effect = TransferEffect::new(
    source_resource.id(),
    destination_resource.id(),
    100u64
);

// Create an effect integration validator
let effect_validator = EffectIntegrationValidator::new();

// Validate effect integration
let validation_result = effect_validator.validate_effect_integration(
    &operation,
    &effect
)?;

if !validation_result.is_valid() {
    // Handle effect integration validation issues
    println!("Effect integration validation failed: {}", validation_result.error_message().unwrap());
}
```

## Best Practices

1. **Early Validation**: Validate operations as early as possible in their lifecycle.

2. **Complete Checks**: Run all validation checks appropriate for the operation type.

3. **Phase-Specific Validation**: Apply different validation rules based on execution phase.

4. **Transaction Context**: Consider the entire transaction when validating operations.

5. **Custom Validators**: Create specialized validators for domain-specific operations.

6. **Thorough Error Reporting**: Provide detailed error messages for validation failures.

7. **Validation Order**: Follow a consistent order of validation steps.

8. **Validation Caching**: Cache validation results for performance in long transactions.

9. **Pre-flight Validation**: Use a pre-flight validation phase for external operations.

10. **Validation Metrics**: Monitor validation performance and failure patterns.

## Implementation Status

The operation validation system is fully implemented in the Causality system:

- ✅ Core `OperationValidator` structure
- ✅ Structural validation
- ✅ Semantic validation
- ✅ Authorization validation
- ✅ Resource validation
- ✅ Temporal validation
- ✅ Transaction validation
- ✅ Phase-specific validation
- ✅ Operation-type-specific validation
- ✅ Validation pipelines

## Future Enhancements

1. **Declarative Validation Rules**: Support for declarative validation rules
2. **Validation DSL**: Domain-specific language for defining validation rules
3. **ML-Based Anomaly Detection**: Machine learning for detecting anomalous operations
4. **Optimization**: Performance optimizations for high-throughput validation
5. **Interactive Validation**: Interactive validation for complex operations 