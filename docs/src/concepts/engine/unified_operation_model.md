<!-- Unified model for operations -->
<!-- Original file: docs/src/unified_operation_model.md -->

# Unified Operation Model

This document outlines the unified operation model within the Causality system, which consolidates various operational concepts (Effects, Operations, and ResourceRegister Operations) into a single comprehensive model.

## Core Concepts

### The Operation Model

The unified `Operation` model serves as the central abstraction for all state-changing actions within the Causality system. It encapsulates:

1. **What is being done**: The type and purpose of the operation
2. **What resources are involved**: Input and output resources
3. **Who is performing it**: Authorization information
4. **When it occurs**: Temporal context
5. **How it's executed**: Execution context and phases
6. **Why it's happening**: Purpose and metadata

This unification eliminates redundant transformation logic, ensures consistent validation, simplifies proof generation, and improves debugging by providing a single source of truth for all operations.

### Operation Context and Phases

Operations flow through several execution phases:

1. **Planning**: Initial specification of the operation
2. **Validation**: Verification of operation validity
3. **Preparation**: Setup for execution
4. **Execution**: Performing the actual state changes
5. **Verification**: Post-execution validation
6. **Observation**: Recording operation outcomes

Each phase has specific requirements and constraints, allowing for precise control over operation lifecycle.

## Structure

```rust
/// Unified operation model for all state-changing actions
pub struct Operation {
    /// Unique identifier for the operation
    id: OperationId,
    
    /// Type of operation being performed
    operation_type: OperationType,
    
    /// Abstract representation of the operation
    abstract_repr: AbstractOperation,
    
    /// Concrete implementation details
    concrete_impl: Option<ConcreteOperation>,
    
    /// Context for operation execution
    execution_context: ExecutionContext,
    
    /// Input resources and states
    inputs: Vec<ResourceState>,
    
    /// Output resources and expected states
    outputs: Vec<ResourceState>,
    
    /// Authorization information
    authorization: Authorization,
    
    /// Dependencies on other operations or facts
    dependencies: Vec<Dependency>,
    
    /// Temporal constraints and context
    temporal_context: TemporalContext,
    
    /// Custom validations for this operation
    validations: Vec<Box<dyn OperationValidator>>,
    
    /// Metadata associated with this operation
    metadata: Option<MetadataMap>,
}

/// Type of operation
pub enum OperationType {
    /// Resource creation operation
    CreateResource,
    
    /// Resource update operation
    UpdateResource,
    
    /// Resource deletion operation
    DeleteResource,
    
    /// Resource transfer operation
    TransferResource,
    
    /// Resource freeze operation
    FreezeResource,
    
    /// Resource unfreeze operation
    UnfreezeResource,
    
    /// Resource consume operation
    ConsumeResource,
    
    /// Resource type registration
    RegisterResourceType,
    
    /// Program execution
    ExecuteProgram,
    
    /// Custom operation type
    Custom(String),
}

/// Execution context for an operation
pub struct ExecutionContext {
    /// Current execution phase
    phase: ExecutionPhase,
    
    /// Domain context for this operation
    domain: Option<DomainId>,
    
    /// Trace for debugging and auditing
    trace: Option<TraceId>,
    
    /// Operation-specific settings
    settings: HashMap<String, Value>,
}

/// Phases of operation execution
pub enum ExecutionPhase {
    /// Planning phase - initial specification
    Planning,
    
    /// Validation phase - checking validity
    Validation,
    
    /// Preparation phase - setup for execution
    Preparation,
    
    /// Execution phase - performing the operation
    Execution,
    
    /// Verification phase - post-execution validation
    Verification,
    
    /// Observation phase - recording outcomes
    Observation,
}

/// Authorization information for an operation
pub struct Authorization {
    /// Entity performing the operation
    entity: ResourceId,
    
    /// Capabilities being exercised
    capabilities: Vec<Capability>,
    
    /// Delegation information if acting on behalf of another
    delegated_by: Option<ResourceId>,
    
    /// Authorization proof
    proof: Option<AuthorizationProof>,
}

/// Dependencies for an operation
pub enum Dependency {
    /// Dependency on another operation
    Operation(OperationId),
    
    /// Dependency on a temporal fact
    TemporalFact(TemporalFactId),
    
    /// Dependency on a resource state
    ResourceState {
        /// Resource identifier
        resource_id: ResourceId,
        
        /// Required state
        state: RegisterState,
    },
    
    /// Custom dependency
    Custom {
        /// Type of dependency
        dependency_type: String,
        
        /// Identifier for the dependency
        id: String,
    },
}
```

## Integration with Resource System

The unified operation model integrates with the resource system:

1. **ResourceRegister**: Provides resources for operations
2. **LifecycleManager**: Validates state transitions in operations
3. **CapabilityRegistry**: Verifies authorization for operations
4. **RelationshipTracker**: Enforces relationship constraints
5. **Effect Templates**: Standardizes common operations
6. **TemporalFacts**: Provides temporal context for operations

## Usage Examples

### Basic Resource Operations

```rust
// Create a resource creation operation
let create_operation = Operation::new(OperationType::CreateResource)
    .with_inputs([])
    .with_outputs([
        ResourceState::new(
            ResourceId::new(),
            ResourceType::FungibleToken,
            RegisterState::Initial
        )
    ])
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(creator_id.clone()));

// Execute the operation
let result = operation_executor.execute(create_operation).await?;
let resource_id = result.outputs()[0].resource_id().clone();

// Update the resource
let update_operation = Operation::new(OperationType::UpdateResource)
    .with_inputs([
        ResourceState::from_id(
            resource_id.clone(),
            RegisterState::Active
        )
    ])
    .with_outputs([
        ResourceState::from_id_with_properties(
            resource_id.clone(),
            RegisterState::Active,
            metadata_map! {
                "name" => "Updated Token",
                "decimals" => 8u8,
            }
        )
    ])
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(updater_id.clone()));

// Execute the update
operation_executor.execute(update_operation).await?;
```

### Resource Transfer with Capabilities

```rust
// Create a transfer operation with capability authorization
let transfer_operation = Operation::new(OperationType::TransferResource)
    .with_inputs([
        ResourceState::from_id_with_owner(
            resource_id.clone(),
            RegisterState::Active,
            current_owner_id.clone()
        )
    ])
    .with_outputs([
        ResourceState::from_id_with_owner(
            resource_id.clone(),
            RegisterState::Active,
            new_owner_id.clone()
        )
    ])
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(
        Authorization::with_capabilities(
            current_owner_id.clone(),
            vec![
                transfer_capability.clone()
            ]
        )
    );

// Validate the operation
let validation_result = operation_validator.validate(&transfer_operation)?;
if validation_result.is_valid {
    // Execute the transfer
    operation_executor.execute(transfer_operation).await?;
}
```

### Operations with Dependencies

```rust
// Create an operation with dependencies
let consume_operation = Operation::new(OperationType::ConsumeResource)
    .with_inputs([
        ResourceState::from_id(
            resource_id.clone(),
            RegisterState::Active
        )
    ])
    .with_outputs([
        ResourceState::from_id(
            resource_id.clone(),
            RegisterState::Consumed {
                consumed_at: time::now(),
                consumer: Some(consumer_id.clone()),
            }
        )
    ])
    .with_dependencies([
        // Depend on a temporal fact
        Dependency::TemporalFact(temporal_fact_id.clone()),
        
        // Depend on another operation
        Dependency::Operation(prerequisite_operation_id.clone()),
    ])
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(consumer_id.clone()));

// Execute with dependency validation
let result = operation_executor
    .execute_with_dependency_validation(consume_operation).await;

// Inspect result
match result {
    Ok(_) => println!("Resource consumed successfully"),
    Err(e) => match e {
        OperationError::DependencyNotSatisfied { dependency } => {
            println!("Dependency not satisfied: {:?}", dependency);
        },
        _ => println!("Operation failed: {:?}", e),
    },
}
```

### Multi-Phase Operation Execution

```rust
// Create an operation
let operation = Operation::new(OperationType::UpdateResource)
    .with_inputs([input_resource.clone()])
    .with_outputs([output_resource.clone()])
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(updater_id.clone()));

// Execute through all phases
let result = operation_executor.execute_multi_phase(
    operation,
    vec![
        ExecutionPhase::Validation,
        ExecutionPhase::Preparation,
        ExecutionPhase::Execution,
        ExecutionPhase::Verification,
    ]
).await?;

// Inspect phase results
for (phase, phase_result) in result.phase_results().iter() {
    println!("Phase {:?}: {:?}", phase, phase_result.status());
    
    if let Some(events) = phase_result.events() {
        for event in events {
            println!("  Event: {:?}", event);
        }
    }
}
```

### Abstract and Concrete Operations

```rust
// Define an abstract operation
let abstract_operation = AbstractOperation::new(
    OperationType::TransferResource,
    vec![
        OperationParameter::new("amount", ParameterType::U64),
        OperationParameter::new("token_id", ParameterType::ResourceId),
        OperationParameter::new("recipient", ParameterType::ResourceId),
    ]
);

// Create a concrete implementation
let concrete_operation = ConcreteOperation::from_parameters(
    hashmap! {
        "amount".to_string() => 100u64.into(),
        "token_id".to_string() => token_id.clone().into(),
        "recipient".to_string() => recipient_id.clone().into(),
    }
);

// Create the full operation
let operation = Operation::new(OperationType::TransferResource)
    .with_abstract_repr(abstract_operation)
    .with_concrete_impl(concrete_operation)
    .with_inputs([
        ResourceState::from_id_with_owner(
            token_id.clone(),
            RegisterState::Active,
            sender_id.clone()
        )
    ])
    .with_outputs([
        ResourceState::from_id_with_owner(
            token_id.clone(),
            RegisterState::Active,
            recipient_id.clone()
        )
    ])
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(sender_id.clone()));

// Execute the operation
operation_executor.execute(operation).await?;
```

### Custom Operation Validation

```rust
// Define a custom validator
struct TokenTransferValidator;

impl OperationValidator for TokenTransferValidator {
    fn validate(&self, operation: &Operation) -> Result<ValidationResult> {
        if operation.operation_type() != OperationType::TransferResource {
            return Ok(ValidationResult::valid()); // Not applicable
        }
        
        // Example: Check if transfer amount is within limits
        if let Some(concrete) = operation.concrete_impl() {
            if let Some(amount) = concrete.get_parameter::<u64>("amount") {
                if *amount > 1_000_000 {
                    return Ok(ValidationResult::invalid(
                        "Transfer amount exceeds maximum limit of 1,000,000"
                    ));
                }
            }
        }
        
        Ok(ValidationResult::valid())
    }
}

// Create an operation with custom validation
let transfer_operation = Operation::new(OperationType::TransferResource)
    // Set up operation parameters
    .with_inputs([input_resource.clone()])
    .with_outputs([output_resource.clone()])
    .with_concrete_impl(
        ConcreteOperation::from_parameters(
            hashmap! {
                "amount".to_string() => 2_000_000u64.into(), // Exceeds limit
            }
        )
    )
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(sender_id.clone()))
    .with_validation(Box::new(TokenTransferValidator));

// Validate the operation
let validation_result = operation_validator.validate(&transfer_operation)?;
assert!(!validation_result.is_valid);
assert_eq!(
    validation_result.reason(),
    Some("Transfer amount exceeds maximum limit of 1,000,000")
);
```

### Temporal Context for Operations

```rust
// Create a temporal context
let temporal_context = TemporalContext::new()
    .with_execution_time(time::now())
    .with_valid_from(time::now() - Duration::hours(1))
    .with_valid_until(time::now() + Duration::hours(1))
    .with_fact_dependencies(vec![
        temporal_fact_id1.clone(),
        temporal_fact_id2.clone(),
    ]);

// Create an operation with temporal context
let operation = Operation::new(OperationType::UpdateResource)
    .with_inputs([input_resource.clone()])
    .with_outputs([output_resource.clone()])
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(updater_id.clone()))
    .with_temporal_context(temporal_context);

// Execute with temporal validation
let result = temporal_validator.validate(&operation)?;
if result.is_valid {
    // All temporal constraints are satisfied
    operation_executor.execute(operation).await?;
}
```

### Batch Operations

```rust
// Create multiple operations
let operations = vec![
    Operation::new(OperationType::UpdateResource)
        .with_inputs([resource1_input.clone()])
        .with_outputs([resource1_output.clone()])
        .with_context(ExecutionContext::new(ExecutionPhase::Planning))
        .with_authorization(Authorization::from(updater_id.clone())),
    
    Operation::new(OperationType::UpdateResource)
        .with_inputs([resource2_input.clone()])
        .with_outputs([resource2_output.clone()])
        .with_context(ExecutionContext::new(ExecutionPhase::Planning))
        .with_authorization(Authorization::from(updater_id.clone())),
    
    Operation::new(OperationType::UpdateResource)
        .with_inputs([resource3_input.clone()])
        .with_outputs([resource3_output.clone()])
        .with_context(ExecutionContext::new(ExecutionPhase::Planning))
        .with_authorization(Authorization::from(updater_id.clone())),
];

// Execute operations as a batch (atomically)
let results = operation_executor.execute_batch(operations).await?;

// Inspect results
for (i, result) in results.iter().enumerate() {
    println!("Operation {}: {:?}", i, result.status());
}
```

## Operation Composition

Operations can be composed to create more complex behaviors:

```rust
// Create a composite operation from individual operations
let composite = CompositeOperation::new()
    .add_operation(create_operation.clone())
    .add_operation(update_operation.clone())
    .add_operation(transfer_operation.clone())
    .with_dependencies([
        // Create must happen before update
        (0, 1),
        // Update must happen before transfer
        (1, 2),
    ]);

// Execute the composite operation
let results = operation_executor.execute_composite(composite).await?;

// Check if all operations succeeded
let all_successful = results.operations().iter()
    .all(|result| result.status() == OperationStatus::Success);

assert!(all_successful);
```

## Integration with Effect Templates

```rust
// Create an operation from an effect template
let template = ResourceTransferTemplate::new();

// Configure the template
let configured_template = template.configure(
    token_id.clone(),
    amount,
    sender_id.clone(),
    recipient_id.clone()
);

// Generate the operation
let operation = configured_template.generate_operation(
    ExecutionContext::new(ExecutionPhase::Planning)
)?;

// Execute the template-generated operation
operation_executor.execute(operation).await?;
```

## Best Practices

1. **Use Typed Operations**: Use type-safe operations when possible to catch errors at compile time.

2. **Validate Before Execution**: Always validate operations before executing them.

3. **Check Capabilities**: Always include appropriate capabilities in operation authorization.

4. **Include Temporal Context**: Always specify temporal context for time-sensitive operations.

5. **Use Concrete Implementations**: Include concrete implementations for better debugging.

6. **Leverage Dependencies**: Use explicit dependencies to ensure correct operation sequencing.

7. **Custom Validation**: Add custom validation for domain-specific constraints.

8. **Audit Trail**: Include trace information for auditing.

9. **Error Handling**: Handle operation errors gracefully with appropriate recovery strategies.

10. **Use Templates**: Standardize common operations using effect templates.

## Security Considerations

1. **Capability Verification**: Always verify capabilities before operation execution.

2. **Input/Output Validation**: Validate all inputs and expected outputs.

3. **Authorization Proofs**: Include authorization proofs for sensitive operations.

4. **Temporal Bounds**: Use temporal constraints to limit operation validity periods.

5. **Idempotency**: Design operations to be idempotent when possible.

## Implementation Status

The unified operation model is fully implemented in the Causality system:

- ✅ Core `Operation` structure
- ✅ Operation types and execution phases
- ✅ Integration with the resource lifecycle
- ✅ Capability-based authorization
- ✅ Temporal validation
- ✅ Operation composition
- ✅ Effect template integration
- ✅ Batch operations
- ✅ Comprehensive validation framework

## Future Enhancements

1. **Operation Workflows**: Support for complex operation workflows with conditional branching
2. **Cross-Domain Operations**: Improved support for operations spanning multiple domains
3. **Operation Optimization**: Automatic optimization of operation sequences
4. **Zero-Knowledge Proofs**: Enhanced support for privacy-preserving operations
5. **Operation Analytics**: Tools for analyzing operation patterns and performance
6. **Distributed Operations**: Support for distributed operation execution
7. **Operation Versioning**: Explicit versioning for operation compatibility
8. **Operation Templates**: More sophisticated templates for common operation patterns
