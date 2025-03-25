<!-- Operation model for registers -->
<!-- Original file: docs/src/register_operation_model.md -->

# Register Operation Model

This document outlines the register operation model within the unified resource architecture, detailing how concrete resource register operations integrate with the unified operation model.

## Core Concepts

### Register Operations

Register operations are concrete, low-level operations that directly manipulate the state of resources in the ResourceRegister. They represent the transition from abstract operations to physical state changes, serving as the concrete implementation for higher-level abstract operations.

Key characteristics of register operations:

1. **Concrete Implementation**: Direct representation of state changes in the resource register
2. **Atomic Operations**: Each operation represents an atomic state transition
3. **Deterministic Execution**: Operations produce consistent results when executed
4. **Verifiable State Changes**: All state changes can be verified against system rules
5. **Storage Integration**: Operations directly interact with the underlying storage layer

### Register Operation Types

```rust
/// Types of operations on the resource register
pub enum RegisterOperationType {
    /// Create a new register entry
    CreateRegister,
    
    /// Update an existing register entry
    UpdateRegister,
    
    /// Freeze a register entry
    FreezeRegister,
    
    /// Unfreeze a register entry
    UnfreezeRegister,
    
    /// Lock a register entry
    LockRegister,
    
    /// Unlock a register entry
    UnlockRegister,
    
    /// Consume a register entry
    ConsumeRegister,
    
    /// Transfer ownership of a register entry
    TransferRegister,
    
    /// Archive a register entry
    ArchiveRegister,
    
    /// Create a relationship between registers
    CreateRelationship,
    
    /// Update a relationship between registers
    UpdateRelationship,
    
    /// Remove a relationship between registers
    RemoveRelationship,
    
    /// Execute custom register logic
    ExecuteLogic,
    
    /// Custom register operation
    Custom(String),
}
```

### Operation Structure

```rust
/// A concrete operation on the resource register
pub struct ResourceRegisterOperation {
    /// Unique identifier for the operation
    id: OperationId,
    
    /// The type of register operation
    operation_type: RegisterOperationType,
    
    /// The primary resource register affected
    target_register: ResourceId,
    
    /// Additional resource registers involved
    related_registers: Vec<ResourceId>,
    
    /// The state before the operation
    pre_state: Option<RegisterState>,
    
    /// The state after the operation
    post_state: Option<RegisterState>,
    
    /// The specific changes to apply
    changes: RegisterChanges,
    
    /// The authorization for this operation
    authorization: Authorization,
    
    /// Parameters for the operation
    parameters: ParameterMap,
    
    /// Metadata for the operation
    metadata: MetadataMap,
}

/// Changes to apply to a register
pub struct RegisterChanges {
    /// Changes to resource properties
    property_changes: HashMap<String, Value>,
    
    /// Changes to resource logic
    logic_changes: Option<Box<dyn ResourceLogic>>,
    
    /// Changes to resource storage
    storage_changes: Option<StorageChange>,
    
    /// Changes to resource relationships
    relationship_changes: Vec<RelationshipChange>,
    
    /// Changes to resource capabilities
    capability_changes: Vec<CapabilityChange>,
}
```

## Integration with Operation Model

Register operations integrate with the unified operation model through:

1. **Transformation Layer**: Abstract operations are transformed into register operations
2. **Execution Context**: Operations are executed within their appropriate context
3. **Validation Pipeline**: Register operations undergo validation before execution
4. **Transaction System**: Register operations can be grouped into atomic transactions
5. **Effect System**: Register operations implement the effect interface

## Usage Examples

### Basic Register Operation

```rust
// Create a register operation
let register_operation = ResourceRegisterOperation::new(
    RegisterOperationType::CreateRegister,
    resource_id.clone()
)
.with_post_state(RegisterState::Initial)
.with_changes(RegisterChanges::new()
    .with_property("type", "token.fungible")
    .with_property("quantity", 100u64)
    .with_logic(Box::new(FungibleTokenLogic::new()))
    .with_storage(StorageStrategy::FullyOnChain { 
        visibility: StateVisibility::Public 
    })
)
.with_authorization(Authorization::from(creator.clone()))
.with_parameter("name", "My Token");

// Execute the register operation
let result = resource_register.execute_operation(register_operation)?;
```

### Resource Update Operation

```rust
// Create an update register operation
let update_operation = ResourceRegisterOperation::new(
    RegisterOperationType::UpdateRegister,
    resource_id.clone()
)
.with_pre_state(RegisterState::Active)
.with_post_state(RegisterState::Active)
.with_changes(RegisterChanges::new()
    .with_property("quantity", 150u64)
)
.with_authorization(Authorization::with_capabilities(
    invoker.clone(),
    vec![update_capability]
));

// Execute the update operation
let result = resource_register.execute_operation(update_operation)?;
```

### Resource Transfer Operation

```rust
// Create a transfer register operation
let transfer_operation = ResourceRegisterOperation::new(
    RegisterOperationType::TransferRegister,
    source_id.clone()
)
.with_related_registers(vec![destination_id.clone()])
.with_pre_state(RegisterState::Active)
.with_post_state(RegisterState::Active)
.with_changes(RegisterChanges::new()
    .with_property("owner", new_owner_id.to_string())
    .with_relationship_change(RelationshipChange::Remove {
        source: source_id.clone(),
        target: old_owner_id.clone(),
        relationship_type: RelationshipType::Ownership,
    })
    .with_relationship_change(RelationshipChange::Add {
        source: source_id.clone(),
        target: new_owner_id.clone(),
        relationship_type: RelationshipType::Ownership,
        metadata: None,
    })
)
.with_authorization(Authorization::with_capabilities(
    invoker.clone(),
    vec![transfer_capability]
))
.with_parameter("amount", 100u64);

// Execute the transfer operation
let result = resource_register.execute_operation(transfer_operation)?;
```

### Resource Lifecycle Operations

```rust
// Create a freeze register operation
let freeze_operation = ResourceRegisterOperation::new(
    RegisterOperationType::FreezeRegister,
    resource_id.clone()
)
.with_pre_state(RegisterState::Active)
.with_post_state(RegisterState::Frozen { 
    freezer: Some(admin_id.clone()),
    reason: Some("Security review".to_string()),
})
.with_authorization(Authorization::with_capabilities(
    admin.clone(),
    vec![freeze_capability]
));

// Execute the freeze operation
let result = resource_register.execute_operation(freeze_operation)?;

// Later, unfreeze the resource
let unfreeze_operation = ResourceRegisterOperation::new(
    RegisterOperationType::UnfreezeRegister,
    resource_id.clone()
)
.with_pre_state(RegisterState::Frozen { 
    freezer: Some(admin_id.clone()),
    reason: Some("Security review".to_string()),
})
.with_post_state(RegisterState::Active)
.with_authorization(Authorization::with_capabilities(
    admin.clone(),
    vec![unfreeze_capability]
));

// Execute the unfreeze operation
let result = resource_register.execute_operation(unfreeze_operation)?;
```

### Relationship Operations

```rust
// Create a relationship between resources
let relationship_operation = ResourceRegisterOperation::new(
    RegisterOperationType::CreateRelationship,
    resource1_id.clone()
)
.with_related_registers(vec![resource2_id.clone()])
.with_changes(RegisterChanges::new()
    .with_relationship_change(RelationshipChange::Add {
        source: resource1_id.clone(),
        target: resource2_id.clone(),
        relationship_type: RelationshipType::Dependency,
        metadata: Some(metadata_map! {
            "priority" => 1u32,
            "created_at" => time::now(),
        }),
    })
)
.with_authorization(Authorization::with_capabilities(
    invoker.clone(),
    vec![relationship_capability]
));

// Execute the relationship operation
let result = resource_register.execute_operation(relationship_operation)?;
```

### Executing Custom Logic

```rust
// Create an operation to execute custom resource logic
let execute_logic_operation = ResourceRegisterOperation::new(
    RegisterOperationType::ExecuteLogic,
    resource_id.clone()
)
.with_pre_state(RegisterState::Active)
.with_post_state(RegisterState::Active)
.with_changes(RegisterChanges::new())
.with_authorization(Authorization::with_capabilities(
    invoker.clone(),
    vec![execute_capability]
))
.with_parameter("function", "check_balance")
.with_parameter("args", json!({
    "account": account_id.to_string()
}));

// Execute the custom logic operation
let result = resource_register.execute_operation(execute_logic_operation)?;
let balance = result.get_value::<u64>("balance")?;
```

## Operation Transformation

The system transforms abstract operations into concrete register operations:

```rust
// Create an abstract operation
let abstract_operation = Operation::new(OperationType::TransferResource)
    .with_input(source_resource.clone())
    .with_output(destination_resource.clone())
    .with_parameter("amount", 100u64)
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::with_capabilities(
        invoker.clone(),
        vec![transfer_capability]
    ));

// Create a transformation service
let transformation_service = OperationTransformationService::new();

// Transform the abstract operation into concrete register operations
let register_operations = transformation_service.transform(&abstract_operation)?;

// Execute the register operations
for op in register_operations {
    let result = resource_register.execute_operation(op)?;
    // Process results
}
```

## Batch Operations

Register operations can be executed in batch for atomicity:

```rust
// Create a batch of register operations
let operations = vec![
    ResourceRegisterOperation::new(
        RegisterOperationType::UpdateRegister,
        resource1_id.clone()
    )
    .with_pre_state(RegisterState::Active)
    .with_post_state(RegisterState::Active)
    .with_changes(RegisterChanges::new()
        .with_property("quantity", 50u64)
    ),
    
    ResourceRegisterOperation::new(
        RegisterOperationType::UpdateRegister,
        resource2_id.clone()
    )
    .with_pre_state(RegisterState::Active)
    .with_post_state(RegisterState::Active)
    .with_changes(RegisterChanges::new()
        .with_property("quantity", 150u64)
    ),
    
    ResourceRegisterOperation::new(
        RegisterOperationType::CreateRelationship,
        resource1_id.clone()
    )
    .with_related_registers(vec![resource2_id.clone()])
    .with_changes(RegisterChanges::new()
        .with_relationship_change(RelationshipChange::Add {
            source: resource1_id.clone(),
            target: resource2_id.clone(),
            relationship_type: RelationshipType::Reference,
            metadata: None,
        })
    ),
];

// Execute the batch atomically
let batch_result = resource_register.execute_batch(operations)?;
```

## Transaction Support

Register operations can be grouped into transactions:

```rust
// Start a transaction
let transaction = resource_register.begin_transaction()?;

// Add operations to the transaction
transaction.add_operation(ResourceRegisterOperation::new(
    RegisterOperationType::UpdateRegister,
    resource1_id.clone()
)
.with_changes(RegisterChanges::new()
    .with_property("quantity", 50u64)
));

transaction.add_operation(ResourceRegisterOperation::new(
    RegisterOperationType::UpdateRegister,
    resource2_id.clone()
)
.with_changes(RegisterChanges::new()
    .with_property("quantity", 150u64)
));

// Commit the transaction
let results = transaction.commit()?;

// Or roll back if needed
// transaction.rollback()?;
```

## Storage Integration

Register operations integrate directly with the storage layer:

```rust
// Create a register operation with storage changes
let storage_operation = ResourceRegisterOperation::new(
    RegisterOperationType::UpdateRegister,
    resource_id.clone()
)
.with_pre_state(RegisterState::Active)
.with_post_state(RegisterState::Active)
.with_changes(RegisterChanges::new()
    .with_storage_change(StorageChange::ChangeStrategy(
        StorageStrategy::PartiallyOnChain {
            on_chain_properties: vec!["id", "type", "owner"],
            off_chain_storage: OffChainStorage::Ipfs,
            visibility: StateVisibility::Private,
        }
    ))
)
.with_authorization(Authorization::with_capabilities(
    admin.clone(),
    vec![storage_capability]
));

// Execute the storage operation
let result = resource_register.execute_operation(storage_operation)?;
```

## Effect Templates Integration

Register operations can be generated from effect templates:

```rust
// Create a transfer effect template
let transfer_template = TransferResourceTemplate::new();

// Create parameters for the template
let template_params = TemplateParams::new()
    .with_value("source_resource", source_resource.clone())
    .with_value("destination_resource", destination_resource.clone())
    .with_value("amount", 100u64)
    .with_value("invoker", invoker.clone())
    .with_value("capabilities", vec![transfer_capability.clone()]);

// Generate an abstract operation from the template
let abstract_operation = transfer_template.create_operation(template_params)?;

// Transform to register operations
let register_operations = transformation_service.transform(&abstract_operation)?;

// Execute the register operations
for op in register_operations {
    let result = resource_register.execute_operation(op)?;
    // Process results
}
```

## Validation Integration

Register operations undergo validation before execution:

```rust
// Create a register operation validator
let validator = RegisterOperationValidator::new(
    resource_register.clone(),
    lifecycle_manager.clone(),
    relationship_tracker.clone(),
    capability_registry.clone()
);

// Validate a register operation
let validation_result = validator.validate(&register_operation)?;

if validation_result.is_valid() {
    // Execute the operation
    let result = resource_register.execute_operation(register_operation)?;
    // Process the result
} else {
    // Handle validation failure
    println!("Validation failed: {}", validation_result.error_message().unwrap());
}
```

## Event Generation

Register operations generate events about state changes:

```rust
// Subscribe to register operation events
let subscription = event_system.subscribe(
    EventFilter::new()
        .with_event_type(EventType::RegisterOperation)
        .with_resource_id(resource_id.clone())
);

// Execute a register operation
let result = resource_register.execute_operation(register_operation)?;

// Process events
let events = subscription.collect_events()?;
for event in events {
    println!("Event: {} on resource {}", 
             event.event_type(), event.resource_id());
    
    // Access event details
    let operation_type = event.get_value::<RegisterOperationType>("operation_type")?;
    let pre_state = event.get_value::<Option<RegisterState>>("pre_state")?;
    let post_state = event.get_value::<Option<RegisterState>>("post_state")?;
    
    println!("Operation: {:?}, State Change: {:?} -> {:?}", 
             operation_type, pre_state, post_state);
}
```

## Best Practices

1. **Validate Before Execution**: Always validate register operations before execution.

2. **Use Transactions**: Group related register operations into transactions.

3. **Check State Transitions**: Ensure register operations include valid state transitions.

4. **Proper Authorization**: Always include proper authorization in register operations.

5. **Event Handling**: Subscribe to events to track register operation execution.

6. **Operation Idempotency**: Design operations to be idempotent where possible.

7. **Error Handling**: Implement robust error handling for operation failures.

8. **Transformation Validation**: Validate transformations from abstract to register operations.

9. **Batch Performance**: Batch similar operations for better performance.

10. **Audit Trail**: Maintain an audit trail of all register operations.

## Implementation Status

The register operation model is fully implemented in the Causality system:

- ✅ Core `ResourceRegisterOperation` structure
- ✅ All operation types
- ✅ Integration with the unified operation model
- ✅ Transformation services
- ✅ Batch operation support
- ✅ Transaction support
- ✅ Validation integration
- ✅ Storage integration
- ✅ Event generation

## Future Enhancements

1. **Distributed Register Operations**: Support for distributed register operations
2. **Operation Scheduling**: Scheduled execution of register operations
3. **Conditional Operations**: Register operations with conditional execution
4. **Operation Templates**: More sophisticated templates for common register operations
5. **Performance Optimizations**: Further optimizations for high-throughput scenarios 