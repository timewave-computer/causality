<!-- API documentation for resource registers -->
<!-- Original file: docs/src/api/resource_register_api.md -->

# ResourceRegister API Documentation

This document provides detailed API documentation for the ResourceRegister module and its related components in the Causality framework.

## Core Types and Interfaces

### `ResourceId`

A unique identifier for resources within the system.

```rust
pub type ResourceId = String;
```

### `RegisterState`

An enumeration of possible states for a resource in the lifecycle management system.

```rust
pub enum RegisterState {
    /// Initial state after registration, not yet activated
    Pending,
    
    /// Resource is active and available for use
    Active,
    
    /// Resource is temporarily locked for operations
    Locked,
    
    /// Resource is frozen (cannot be modified but can be read)
    Frozen,
    
    /// Resource has been consumed and is no longer available
    Consumed,
    
    /// Resource has been archived for historical purposes
    Archived,
    
    /// Resource is not active but exists in the system
    Inactive,
}
```

### `Metadata`

A key-value store for attaching arbitrary metadata to resources and relationships.

```rust
pub type Metadata = HashMap<String, String>;
```

## ResourceRegisterLifecycleManager

Manages the lifecycle states of resources, including state transitions and history tracking.

### Creation

```rust
/// Create a new lifecycle manager
pub fn new() -> Self {
    ResourceRegisterLifecycleManager {
        current_states: HashMap::new(),
        locked_by: HashMap::new(),
        transition_history: HashMap::new(),
    }
}
```

### Resource Registration and State Management

```rust
/// Register a new resource (puts it in Pending state)
pub fn register_resource(&mut self, resource_id: ResourceId) -> Result<(), Error>;

/// Get the current state of a resource
pub fn get_state(&self, resource_id: &ResourceId) -> Result<RegisterState, Error>;

/// Activate a resource (transition from Pending to Active)
pub fn activate(&mut self, resource_id: &ResourceId) -> Result<(), Error>;

/// Lock a resource for processing (transition to Locked)
pub fn lock(
    &mut self, 
    resource_id: &ResourceId, 
    metadata: Option<Metadata>
) -> Result<(), Error>;

/// Unlock a resource (transition from Locked back to Active)
pub fn unlock(&mut self, resource_id: &ResourceId) -> Result<(), Error>;

/// Freeze a resource (transition to Frozen state)
pub fn freeze(
    &mut self, 
    resource_id: &ResourceId, 
    metadata: Option<Metadata>
) -> Result<(), Error>;

/// Unfreeze a resource (transition from Frozen back to Active)
pub fn unfreeze(&mut self, resource_id: &ResourceId) -> Result<(), Error>;

/// Consume a resource (transition to terminal Consumed state)
pub fn consume(&mut self, resource_id: &ResourceId) -> Result<(), Error>;

/// Archive a resource (transition to terminal Archived state)
pub fn archive(&mut self, resource_id: &ResourceId) -> Result<(), Error>;

/// Deactivate a resource (transition to Inactive state)
pub fn deactivate(&mut self, resource_id: &ResourceId) -> Result<(), Error>;
```

### State Transition History

```rust
/// Get the transition history for a resource
pub fn get_transition_history(
    &self, 
    resource_id: &ResourceId
) -> Result<Vec<StateTransitionRecord>, Error>;

/// Record a state transition with optional metadata
fn record_transition(
    &mut self,
    resource_id: &ResourceId,
    previous_state: RegisterState,
    new_state: RegisterState,
    metadata: Option<Metadata>,
) -> Result<(), Error>;
```

### Validation

```rust
/// Check if an operation is valid for the current state
pub fn is_operation_valid(
    &self, 
    resource_id: &ResourceId, 
    operation: RegisterOperationType
) -> Result<bool, Error>;

/// Validate a state transition
fn validate_transition(
    &self,
    resource_id: &ResourceId,
    current_state: RegisterState,
    target_state: RegisterState,
) -> Result<(), Error>;
```

## RelationshipTracker

Manages relationships between resources, including parent-child, dependency, and custom relationships.

### Creation

```rust
/// Create a new relationship tracker
pub fn new() -> Self {
    RelationshipTracker {
        relationships: Vec::new(),
        index: RelationshipIndex::new(),
    }
}
```

### Relationship Management

```rust
/// Add a parent-child relationship between resources
pub fn add_parent_child_relationship(
    &mut self,
    parent_id: ResourceId,
    child_id: ResourceId,
    metadata: Option<Metadata>,
) -> Result<(), Error>;

/// Add a dependency relationship between resources
pub fn add_dependency_relationship(
    &mut self,
    dependent_id: ResourceId,
    dependency_id: ResourceId,
    metadata: Option<Metadata>,
) -> Result<(), Error>;

/// Add a custom relationship between resources
pub fn add_custom_relationship(
    &mut self,
    from_id: ResourceId,
    to_id: ResourceId,
    relationship_type: String,
    direction: RelationshipDirection,
    metadata: Option<Metadata>,
) -> Result<(), Error>;

/// Remove a relationship by ID
pub fn remove_relationship(&mut self, relationship_id: usize) -> Result<(), Error>;

/// Remove all relationships between two resources
pub fn remove_relationships_between(
    &mut self,
    resource_a: &ResourceId,
    resource_b: &ResourceId,
) -> Result<(), Error>;
```

### Relationship Querying

```rust
/// Get all relationships for a resource
pub fn get_relationships_for_resource(
    &self, 
    resource_id: &ResourceId
) -> Result<Vec<&ResourceRelationship>, Error>;

/// Get relationships between two resources
pub fn get_relationships_between(
    &self,
    resource_a: &ResourceId,
    resource_b: &ResourceId,
) -> Result<Vec<&ResourceRelationship>, Error>;

/// Get relationships by type
pub fn get_relationships_by_type(
    &self,
    relationship_type: &str,
) -> Result<Vec<&ResourceRelationship>, Error>;

/// Get child resources for a parent
pub fn get_child_resources(
    &self, 
    parent_id: &ResourceId
) -> Result<Vec<ResourceId>, Error>;

/// Get parent resources for a child
pub fn get_parent_resources(
    &self, 
    child_id: &ResourceId
) -> Result<Vec<ResourceId>, Error>;

/// Get dependencies for a resource
pub fn get_dependencies(
    &self, 
    dependent_id: &ResourceId
) -> Result<Vec<ResourceId>, Error>;

/// Get dependents of a resource
pub fn get_dependents(
    &self, 
    dependency_id: &ResourceId
) -> Result<Vec<ResourceId>, Error>;

/// Check if a resource has a specific relationship type
pub fn has_relationship_of_type(
    &self,
    resource_id: &ResourceId,
    relationship_type: &str,
) -> Result<bool, Error>;

/// Update metadata for a relationship
pub fn update_relationship_metadata(
    &mut self,
    from_id: &ResourceId,
    to_id: &ResourceId,
    new_metadata: &Metadata,
) -> Result<(), Error>;
```

## Storage Strategies

Different strategies for storing resource data.

### `StorageStrategy`

An enumeration of available resource storage strategies.

```rust
pub enum StorageStrategy {
    /// Full on-chain storage of the resource
    OnChain,
    
    /// Store only a commitment to the resource on-chain
    Commitment,
    
    /// Store a nullifier to prevent double-usage
    Nullifier,
    
    /// Hybrid approach combining multiple strategies
    Hybrid(Vec<StorageStrategy>),
    
    /// Custom storage strategy with a name
    Custom(String),
}
```

### `StorageAdapter`

Abstract interface for storage operations.

```rust
pub trait StorageAdapter {
    /// Store a resource
    fn store(&self, resource_id: &ResourceId, data: &[u8]) -> Result<(), Error>;
    
    /// Retrieve a resource
    fn retrieve(&self, resource_id: &ResourceId) -> Result<Vec<u8>, Error>;
    
    /// Check if a resource exists
    fn exists(&self, resource_id: &ResourceId) -> Result<bool, Error>;
    
    /// Delete a resource
    fn delete(&self, resource_id: &ResourceId) -> Result<(), Error>;
}
```

## Error Handling

The module uses a common error type for consistent error handling.

```rust
/// Error types for resource operations
pub enum ResourceError {
    /// Resource not found
    NotFound(ResourceId),
    
    /// Invalid state transition
    InvalidStateTransition {
        resource_id: ResourceId,
        from: RegisterState,
        to: RegisterState,
    },
    
    /// Operation not allowed in current state
    OperationNotAllowed {
        resource_id: ResourceId,
        operation: RegisterOperationType,
        current_state: RegisterState,
    },
    
    /// Resource locked by another resource
    ResourceLocked {
        resource_id: ResourceId,
        locked_by: ResourceId,
    },
    
    /// Storage error
    StorageError(String),
    
    /// Relationship error
    RelationshipError(String),
    
    /// Other general errors
    Other(String),
}
```

## Integration with Effect System

The resource system integrates with the effect system through effect templates.

```rust
/// Create a resource effect template
pub fn create_resource_effect<T: ResourceData>(
    resource_id: ResourceId,
    data: T,
    storage_strategy: StorageStrategy,
) -> Effect;

/// Update a resource effect template
pub fn update_resource_effect<T: ResourceData>(
    resource_id: &ResourceId,
    data: T,
) -> Effect;

/// Lock a resource effect template
pub fn lock_resource_effect(
    resource_id: &ResourceId,
    locker_id: &ResourceId,
    metadata: Option<Metadata>,
) -> Effect;

/// Consume a resource effect template
pub fn consume_resource_effect(
    resource_id: &ResourceId,
) -> Effect;
```

## Usage Examples

For usage examples, please refer to the following documents:

- [Relationship Tracking Example](../examples/relationship_tracking_example.md)
- [Lifecycle Management Example](../examples/lifecycle_management_example.md)
- [Integrated Resource Management Example](../examples/integrated_resource_management_example.md)

## Best Practices

1. Always check resource state before operations
2. Use effect templates for consistent behavior
3. Model dependencies explicitly as relationships
4. Record metadata during state transitions for auditability
5. Choose appropriate storage strategies based on use case 