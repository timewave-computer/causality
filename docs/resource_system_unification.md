# Resource System Unification

This document outlines the unified resource system architecture as implemented in the Causality project. The unification simplifies and consolidates multiple resource-related components into a coherent, consistent system.

## 1. Core Components

### 1.1 ResourceRegister

The `ResourceRegister` is the unified abstraction that combines logical resource properties with physical register characteristics. This simplifies the mental model and implementation by reducing the number of concepts developers need to manage.

Key aspects:
- **Identity**: Unique identifier for the resource (`ResourceId`)
- **Logical Properties**: Resource type, fungibility domain, quantity, and metadata
- **Physical Properties**: State, nullifier key, and storage strategy
- **Provenance**: Controller label for tracking provenance across domains
- **Temporal Context**: Time map snapshot for temporal validation

### 1.2 Lifecycle Management

The `ResourceRegisterLifecycleManager` manages the lifecycle states of resources and enforces valid state transitions:

Resource States:
- **Initial**: The resource has been registered but not yet activated
- **Active**: The resource is active and available for operations
- **Locked**: The resource is temporarily locked by another resource/actor
- **Frozen**: The resource is frozen and not available for most operations
- **Consumed**: The resource has been consumed (terminal state)
- **Archived**: The resource has been archived for historical purposes
- **Pending**: The resource is in a pending state waiting for some condition

### 1.3 Storage Strategies

Resources can use different storage strategies:
- **FullyOnChain**: All fields are stored on-chain with configurable visibility
- **CommitmentBased**: Minimal on-chain footprint with ZK proofs for operations
- **Hybrid**: Critical fields on-chain, others as commitments

### 1.4 Effect Templates

The effect template system provides standardized patterns for resource operations:
- **Basic Operations**: Create, update, lock, unlock, freeze, unfreeze, consume, transfer
- **Boundary Awareness**: Resource operations with boundary management integration
- **Cross-Domain Operations**: Operations that span multiple domains
- **Capability Validation**: Operations validated against capability requirements
- **Time Map Integration**: Operations validated for temporal consistency
- **Commitment-Based Operations**: Operations with on-chain commitment for verification

## 2. Integration Architecture

### 2.1 Effect-Lifecycle Integration

The effect template system integrates with the lifecycle manager through:
1. **State Validation**: Effects validate current state before operations
2. **Transition Enforcement**: Effects enforce valid state transitions
3. **Relationship Management**: Effect templates manage relationships between resources
4. **Capability Checking**: Effects validate capabilities before operations

### 2.2 Time Map Integration

The integration with the Time Map ensures temporal consistency:
1. **Resource State Snapshot**: Records resource state at specific time points
2. **Temporal Validation**: Validates that operations respect temporal ordering
3. **Cross-Domain Synchronization**: Synchronizes resource states across domains
4. **Conflict Resolution**: Provides mechanisms to detect and handle temporal conflicts

### 2.3 Boundary Management

Resources can cross boundaries with proper validation:
1. **Boundary Aware Effects**: Effects check boundary crossing permissions
2. **Domain Transitions**: Effects manage cross-domain resource operations
3. **Metadata Tracking**: Operations track boundary crossing metadata

## 3. Usage Examples

### 3.1 Basic Resource Lifecycle

```rust
// Create a resource
let resource = ResourceRegister::new(
    ResourceId::new(),
    ResourceProperties::new()
        .with_fungibility_domain("token")
        .with_quantity(100),
    FungibleTokenLogic::new(),  // Implements ResourceLogic trait
    StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
);

// Create an operation for resource creation
let operation = Operation::new(OperationType::Create)
    .with_output(resource.clone())
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(invoker.clone()));

// Execute the operation
let result = execute_operation(operation, &effect_context).await?;
```

### 3.2 Resource with Capability Validation

```rust
// Create a capability for resource freezing
let freeze_capability = Capability::new(
    Rights::from([Right::Freeze]),
    Targets::Resource(resource.id.clone()),
    CapabilityConstraints::new()
        .with_expiration(time::now() + Duration::hours(24))
);

// Create an operation for freezing with capability validation
let operation = Operation::new(OperationType::Freeze)
    .with_input(resource.clone())
    .with_output(resource.with_state(RegisterState::Frozen))
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::with_capabilities(
        invoker.clone(), 
        vec![freeze_capability]
    ));

// Execute the operation
let result = execute_operation(operation, &effect_context).await?;
```

### 3.3 Time Map Synchronized Resources

```rust
// Get current temporal context
let temporal_context = time_service.get_current_temporal_context()?;

// Create an operation with temporal context validation
let operation = Operation::new(OperationType::Transfer)
    .with_input(resource.clone())
    .with_output(resource.with_owner(recipient.clone()))
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(invoker.clone()))
    .with_temporal_context(temporal_context);

// Add temporal facts this operation depends on
let balance_fact = time_service.observe_fact(
    FactQuery::new(resource.id.clone(), FactType::Balance),
    resource.domain_id.clone()
).await?;

operation.add_fact_dependency(balance_fact.fact_id);

// Execute the operation
let result = execute_operation(operation, &effect_context).await?;
```

## 4. Effect Template Constraints

Effect templates enforce the following constraints:

1. **Lifecycle State Constraints**: Effect templates check if the operation is valid for the current resource state.
2. **Capability Constraints**: Effects validate that the invoker has the required capabilities.
3. **Relationship Constraints**: Effects validate relationship constraints for operations.
4. **Temporal Constraints**: Effects validate temporal consistency with time map snapshots.
5. **Boundary Constraints**: Effects validate boundary crossing permissions.

These constraints ensure that resources maintain integrity and consistency throughout their lifecycle, even in complex multi-domain environments.

## 5. Usage in the Codebase

The unified resource system can now be used throughout the codebase with a simple and consistent API:

```rust
use causality::{
    ResourceRegister, 
    RegisterState, 
    StorageStrategy,
    StateVisibility,
    Operation,
    OperationType,
    ExecutionContext,
    ExecutionPhase,
    Authorization,
};

// Create a resource register
let resource = ResourceRegister::new(
    "resource1".to_string(),
    ResourceProperties::new()
        .with_fungibility_domain("token")
        .with_quantity(100),
    FungibleTokenLogic::new(),
    StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
);

// Create an operation for the resource
let operation = Operation::new(OperationType::Create)
    .with_output(resource.clone())
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from("user1".into()));

// Execute the operation
let result = execute_operation(operation, &context).await?;

// Work with the resource
if result.success {
    println!("Resource created: {}", resource.id);
}
```
