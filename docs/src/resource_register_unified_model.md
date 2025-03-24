# Unified ResourceRegister Model

## Overview

The Unified ResourceRegister model provides a standardized approach to resource management in Causality. This document outlines the core concepts, components, and interactions in the unified model.

## Core Concepts

### ResourceRegister

The `ResourceRegister` is the foundational abstraction for representing resources in Causality. It provides:

- Uniform identity through register IDs
- Consistent state management through a standardized lifecycle
- Flexible storage strategies that can be tailored to different blockchain environments
- Relationship tracking between resources
- Integration with resource logic for validation and transformation rules
- Support for capability-based access control

### Register Lifecycle

Resources move through a well-defined lifecycle, managed by the `ResourceRegisterLifecycleManager`:

1. **Initial**: The resource has been created but is not yet active
2. **Active**: The resource is active and available for operations
3. **Locked**: The resource is temporarily unavailable, locked by another resource
4. **Frozen**: The resource is temporarily immutable but can be thawed
5. **Consumed**: The resource has been permanently consumed (terminal state)
6. **Pending**: The resource is awaiting confirmation/validation
7. **Archived**: The resource has been archived for historical reference

### Resource Relationships

Resources can have relationships with other resources, managed by the `RelationshipTracker`:

- **Parent-Child**: Hierarchical relationships between resources
- **Dependency**: One resource depends on another
- **Lock**: One resource has locked another
- **Consumption**: One resource has consumed another
- **Custom**: Domain-specific relationships defined by applications

## Architecture Components

### ResourceRegisterLifecycleManager

The lifecycle manager handles state transitions and ensures they follow valid paths. Key features:

- State validation to prevent invalid transitions
- Tracking of resource locking relationships
- Historical record of state transitions
- Validation of operations against current states

```rust
// Example of using the lifecycle manager
let mut lifecycle_manager = ResourceRegisterLifecycleManager::new();

// Register a new resource
lifecycle_manager.register_resource("resource1".to_string())?;

// Activate it
lifecycle_manager.activate(&"resource1".to_string())?;

// Lock it
lifecycle_manager.lock(
    &"resource1".to_string(),
    Some(&"locker1".to_string())
)?;
```

### RelationshipTracker

The relationship tracker maintains and manages relationships between resources. Key features:

- Adding/removing relationships of different types
- Querying resources by relationship type
- Finding all related resources
- Traversing relationship graphs (finding children, dependencies, etc.)
- Metadata associated with relationships

```rust
// Example of using the relationship tracker
let mut relationship_tracker = RelationshipTracker::new();

// Create a parent-child relationship
relationship_tracker.add_parent_child_relationship(
    "parent1".to_string(), 
    "child1".to_string(),
    None
)?;

// Query for child resources
let children = relationship_tracker.get_child_resources(&"parent1".to_string())?;
```

### Storage Strategies

Resources can be stored using different strategies:

- **FullyOnChain**: Fully on-chain storage with configurable visibility (Public, Private, Authorized)
- **CommitmentBased**: Store only a commitment on-chain for privacy and reduced footprint
- **Hybrid**: Combination of strategies for balancing privacy, efficiency, and transparency

## Effect Templates and Operations

The unified model provides templates for common operations, which can be used to create standardized operations:

- Creating resources
- Updating resources
- Locking/unlocking resources
- Freezing/unfreezing resources
- Consuming resources
- Transferring resource ownership
- Managing resource relationships

Each template is now integrated with the unified Operation model, which:

- Expresses operations at multiple abstraction levels
- Incorporates authorization through capabilities
- Manages temporal context for time-sensitive operations
- Supports validation at various execution phases
- Integrates with the ZK proof system where necessary

## Integration Points

### Domain-Specific Adaptations

The unified model can be adapted to specific domains through:

- Custom storage strategies
- Domain-specific relationship types
- Specialized lifecycle states through extensions
- Custom effect templates

### TEL Integration

The Temporal Effect Language (TEL) integrates with the unified model through:

- Operations that can be referenced in scripts
- Lifecycle state checks in script conditions
- Relationship constraints for authorization
- Integrated temporal facts for temporal validation
- Capability-based authorization checks

## Usage Examples

### Basic Resource Management

```rust
// Create a resource with properties and logic
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
    .with_authorization(Authorization::from(invoker.clone()));

// Execute the operation
let result = execute_operation(operation, &context).await?;

// Later, consume the resource
let consume_operation = Operation::new(OperationType::Consume)
    .with_input(resource.clone())
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(invoker.clone()));

let consume_result = execute_operation(consume_operation, &context).await?;
```

### Capability-Based Authorization

```rust
// Create a capability for the resource
let transfer_capability = Capability::new(
    Rights::from([Right::Transfer]),
    Targets::Resource(resource.id.clone()),
    CapabilityConstraints::new()
        .with_max_quantity(50)
        .with_expiration(time::now() + Duration::hours(24))
);

// Create an operation with capability-based authorization
let operation = Operation::new(OperationType::Transfer)
    .with_input(resource.clone())
    .with_output(resource.with_owner(recipient.clone()))
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::with_capabilities(
        invoker.clone(), 
        vec![transfer_capability]
    ));

// Validate and execute the operation
let validation_result = validate_operation(&operation, &validator)?;
if validation_result.is_valid {
    let execution_result = execute_operation(operation, &context).await?;
    // Process the result
}
```

### Resource Relationships with Operations

```rust
// Create resources
let parent_resource = ResourceRegister::new(
    "parent".to_string(),
    ResourceProperties::new(),
    CompositeResourceLogic::new(),
    StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
);

let child_resource = ResourceRegister::new(
    "child".to_string(),
    ResourceProperties::new(),
    AtomicResourceLogic::new(),
    StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
);

// Create an operation for establishing relationship
let relationship_operation = Operation::new(OperationType::CreateRelationship)
    .with_input(parent_resource.clone())
    .with_input(child_resource.clone())
    .with_parameter("relationship_type", RelationshipType::ParentChild)
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(invoker.clone()));

// Execute the operation
let result = execute_operation(relationship_operation, &context).await?;

// Later, query relationships through the relationship tracker
let relationship_tracker = context.get_relationship_tracker()?;
let children = relationship_tracker.get_relationships(
    &parent_resource.id,
    Some(RelationshipType::ParentChild),
    RelationshipDirection::Outgoing
)?;

// Process children resources
for child in children {
    // Access the target resource ID
    let child_id = &child.target_id;
    // ...
}
```

## Best Practices

1. **Use the Operation Model**: Create and execute operations using the unified Operation model rather than directly manipulating resources.

2. **Use Resource Logic**: Implement the ResourceLogic trait for your resources to enforce validation and transformation rules.

3. **Apply Capability-Based Access Control**: Use capabilities to clearly express and enforce access rights to resources.

4. **Respect Lifecycle States**: Always validate that a resource is in an appropriate state before performing operations.

5. **Use Temporal Context**: Include temporal context in operations for proper temporal validation.

6. **Explicitly Track Relationships**: Model dependencies between resources as explicit relationships rather than embedding references.

7. **Choose Appropriate Storage Strategy**: Select the storage strategy based on privacy, performance, and transparency needs.

8. **Validate Before Execution**: Validate operations before executing them, especially for costly or irreversible operations.

9. **Use Transactions**: Wrap multiple operations in transactions to maintain consistency.

10. **Document Relationships and Capabilities**: Include clear documentation about the relationships and required capabilities in your domain model. 