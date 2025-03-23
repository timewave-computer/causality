# Resource Lifecycle Management

This document outlines the resource lifecycle management system within the unified resource architecture, focusing on how the `ResourceRegisterLifecycleManager` handles resource state transitions and validation.

## Core Concepts

### Resource Lifecycle States

Resources in the Causality system flow through a well-defined lifecycle with distinct states:

1. **Initial**: The resource has been registered but not fully activated
2. **Active**: The resource is active and available for operations
3. **Locked**: The resource is temporarily unavailable, locked by another entity
4. **Frozen**: The resource is frozen and unavailable for most operations
5. **Consumed**: The resource has been permanently consumed (terminal state)
6. **Archived**: The resource has been archived for historical purposes
7. **Pending**: The resource is in an intermediate state awaiting some condition

These states provide a clear model for how resources can be used throughout their lifecycle and enforce appropriate constraints on operations.

### Lifecycle Manager

The `ResourceRegisterLifecycleManager` is responsible for:

1. **State Tracking**: Maintaining the current state of all resources
2. **Transition Validation**: Ensuring state transitions follow allowed paths
3. **Operation Validation**: Validating operations against current states
4. **Relationship Enforcement**: Maintaining consistency in resource relationships during state changes
5. **Event Generation**: Producing events for state transitions

## Structure

```rust
/// Manages the lifecycle of resource registers
pub struct ResourceRegisterLifecycleManager {
    /// Internal state storage for resources
    resource_states: HashMap<ResourceId, RegisterState>,
    
    /// Tracks relationships between resources
    relationship_tracker: RelationshipTracker,
    
    /// Capability registry for authorization
    capability_registry: CapabilityRegistry,
    
    /// Event emitter for lifecycle events
    event_emitter: EventEmitter,
    
    /// Configuration for the lifecycle manager
    config: LifecycleConfig,
}

/// Configuration for the lifecycle manager
pub struct LifecycleConfig {
    /// Whether to validate relationships during transitions
    validate_relationships: bool,
    
    /// Whether to validate capabilities during transitions
    validate_capabilities: bool,
    
    /// Whether to emit events for transitions
    emit_events: bool,
    
    /// Custom validation rules for transitions
    custom_validators: Vec<Box<dyn TransitionValidator>>,
}

/// The state of a resource register
pub enum RegisterState {
    /// Initial state after registration
    Initial,
    
    /// Resource is active and available
    Active,
    
    /// Resource is temporarily locked
    Locked {
        /// Who or what locked the resource
        locker: Option<ResourceId>,
        
        /// When the lock expires (if applicable)
        expires_at: Option<Timestamp>,
    },
    
    /// Resource is frozen (more restrictive than locked)
    Frozen {
        /// Who or what froze the resource
        freezer: Option<ResourceId>,
        
        /// Reason for freezing
        reason: Option<String>,
    },
    
    /// Resource has been consumed (terminal state)
    Consumed {
        /// When the resource was consumed
        consumed_at: Timestamp,
        
        /// Who or what consumed the resource
        consumer: Option<ResourceId>,
    },
    
    /// Resource has been archived
    Archived {
        /// When the resource was archived
        archived_at: Timestamp,
        
        /// Reason for archiving
        reason: Option<String>,
    },
    
    /// Resource is in a pending state
    Pending {
        /// Target state after pending condition is met
        target_state: Box<RegisterState>,
        
        /// Condition that must be met
        condition: PendingCondition,
    },
}
```

## State Transitions

The lifecycle manager enforces valid state transitions according to this state machine:

```
                       ┌─────────┐
                       │         │
                       ▼         │
  ┌────────┐        ┌───────┐    │
  │Initial │───────▶│Active │────┘
  └────────┘        └───┬───┘
                        │
                        ├────────────┬─────────────┐
                        │            │             │
                        ▼            ▼             ▼
                    ┌───────┐    ┌───────┐    ┌────────┐
                    │Locked │    │Frozen │    │Pending │
                    └───┬───┘    └───┬───┘    └────┬───┘
                        │            │             │
                        │            │        (condition met)
                        │            │             │
                        │            │             │
                        └────────────┴─────────┐   │
                                               │   │
                                               ▼   ▼
   ┌─────────┐                           ┌─────────────┐
   │Archived │◀──────────────────────────│   Active    │
   └─────────┘                           └──────┬──────┘
                                                │
                                                ▼
                                          ┌──────────┐
                                          │Consumed  │
                                          └──────────┘
```

The allowed transitions include:
- Initial → Active
- Active → Locked, Frozen, Pending, Consumed, or Archived
- Locked → Active
- Frozen → Active
- Pending → Active (when condition is met)
- Active → Archived

## Integration with Operation Model

The lifecycle manager integrates with the unified operation model via:

1. **Operation Validation**: Operations are validated against current resource states
2. **State Transitions**: Operations can trigger state transitions
3. **Transaction Contexts**: Lifecycle state changes can be part of larger transactions
4. **Capability Verification**: State transitions require appropriate capabilities

## Usage Examples

### Basic Lifecycle Management

```rust
// Create a lifecycle manager
let mut lifecycle_manager = ResourceRegisterLifecycleManager::new(
    LifecycleConfig::default()
        .with_validate_relationships(true)
        .with_validate_capabilities(true)
        .with_emit_events(true)
);

// Register a new resource
lifecycle_manager.register_resource(resource_id.clone())?;

// Check the current state
let state = lifecycle_manager.get_state(&resource_id)?;
assert_eq!(state, RegisterState::Initial);

// Activate the resource
lifecycle_manager.activate(&resource_id)?;
let state = lifecycle_manager.get_state(&resource_id)?;
assert_eq!(state, RegisterState::Active);

// Lock the resource
lifecycle_manager.lock(
    &resource_id,
    Some(&locker_id),
    Some(time::now() + Duration::minutes(30))
)?;
let state = lifecycle_manager.get_state(&resource_id)?;
assert!(matches!(state, RegisterState::Locked { .. }));

// Unlock the resource
lifecycle_manager.unlock(&resource_id)?;
let state = lifecycle_manager.get_state(&resource_id)?;
assert_eq!(state, RegisterState::Active);

// Consume the resource
lifecycle_manager.consume(
    &resource_id,
    Some(&consumer_id)
)?;
let state = lifecycle_manager.get_state(&resource_id)?;
assert!(matches!(state, RegisterState::Consumed { .. }));
```

### Integration with Operation Model

```rust
// Create an operation for resource activation
let operation = Operation::new(OperationType::ActivateResource)
    .with_input(resource.with_state(RegisterState::Initial))
    .with_output(resource.with_state(RegisterState::Active))
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(invoker.clone()));

// Validate using the lifecycle manager
let validation_result = lifecycle_manager.validate_operation(&operation)?;
if validation_result.is_valid {
    // Perform the state transition
    lifecycle_manager.apply_operation(&operation)?;
    
    // Execute the operation
    let result = execute_operation(operation, &context).await?;
    // Process result
}
```

### Relationship-Aware Transitions

```rust
// Example of attempting to consume a resource with dependents
let resource_id = resource.id();
let dependent_id = dependent_resource.id();

// Set up a dependency relationship
lifecycle_manager.relationship_tracker().add_relationship(
    &dependent_id,
    &resource_id,
    RelationshipType::Dependency,
    None
)?;

// Try to consume the resource (should fail due to dependency)
let result = lifecycle_manager.consume(&resource_id, None);
assert!(result.is_err());
assert_eq!(
    result.unwrap_err().to_string(),
    "Cannot consume resource with dependent resources"
);

// Remove the dependency and try again
lifecycle_manager.relationship_tracker().remove_relationship(
    &dependent_id,
    &resource_id,
    RelationshipType::Dependency
)?;

// Now consumption should succeed
lifecycle_manager.consume(&resource_id, None)?;
let state = lifecycle_manager.get_state(&resource_id)?;
assert!(matches!(state, RegisterState::Consumed { .. }));
```

### Capability-Based State Transitions

```rust
// Create capabilities for state transitions
let freeze_capability = Capability::new(
    Rights::from([Right::Freeze]),
    Targets::Resource(resource.id.clone()),
    None
);

// Register the capability
capability_registry.register_capability(freeze_capability.clone(), admin.id())?;

// Create an operation with the capability
let operation = Operation::new(OperationType::FreezeResource)
    .with_input(resource.clone())
    .with_output(resource.with_state(RegisterState::Frozen {
        freezer: Some(admin.id().clone()),
        reason: Some("Security review".to_string()),
    }))
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::with_capabilities(
        admin.clone(),
        vec![freeze_capability]
    ));

// Validate and apply the operation
let validation_result = lifecycle_manager.validate_operation(&operation)?;
if validation_result.is_valid {
    lifecycle_manager.apply_operation(&operation)?;
    
    // Verify state change
    let state = lifecycle_manager.get_state(&resource.id())?;
    assert!(matches!(state, RegisterState::Frozen { .. }));
}
```

### Conditional State Transitions

```rust
// Create a pending state with a condition
let condition = PendingCondition::TemporalFact(
    temporal_fact_id.clone()
);

// Set resource to pending state
lifecycle_manager.set_pending(
    &resource_id,
    RegisterState::Active,
    condition.clone()
)?;

let state = lifecycle_manager.get_state(&resource_id)?;
assert!(matches!(state, RegisterState::Pending { .. }));

// When the condition is met, check the state
lifecycle_manager.check_pending_conditions()?;

// If the temporal fact is now available, resource should be active
let state = lifecycle_manager.get_state(&resource_id)?;
assert_eq!(state, RegisterState::Active);
```

## Batch Operations

The lifecycle manager supports batch operations for atomicity:

```rust
// Create a batch of state transitions
let batch = vec![
    StateTransition::new(resource1_id.clone(), RegisterState::Active),
    StateTransition::new(resource2_id.clone(), RegisterState::Locked {
        locker: Some(locker_id.clone()),
        expires_at: Some(time::now() + Duration::hours(1)),
    }),
    StateTransition::new(resource3_id.clone(), RegisterState::Frozen {
        freezer: Some(admin_id.clone()),
        reason: Some("Maintenance".to_string()),
    }),
];

// Apply the batch atomically
lifecycle_manager.apply_batch(batch)?;

// Verify all states
assert_eq!(
    lifecycle_manager.get_state(&resource1_id)?,
    RegisterState::Active
);
assert!(matches!(
    lifecycle_manager.get_state(&resource2_id)?,
    RegisterState::Locked { .. }
));
assert!(matches!(
    lifecycle_manager.get_state(&resource3_id)?,
    RegisterState::Frozen { .. }
));
```

## Lifecycle Events

The lifecycle manager generates events for state transitions:

```rust
// Subscribe to lifecycle events
let subscription = lifecycle_manager.event_emitter().subscribe(
    EventFilter::new()
        .with_event_type(EventType::ResourceStateChange)
        .with_resource_id(resource_id.clone())
);

// Perform a state transition
lifecycle_manager.activate(&resource_id)?;

// Check for the event
let events = subscription.collect_events()?;
assert_eq!(events.len(), 1);
let event = &events[0];
assert_eq!(event.resource_id(), &resource_id);
assert_eq!(
    event.get_value::<RegisterState>("previous_state")?,
    RegisterState::Initial
);
assert_eq!(
    event.get_value::<RegisterState>("new_state")?,
    RegisterState::Active
);
```

## Custom Validators

Custom validators can be added to enforce domain-specific rules:

```rust
// Create a custom validator
struct MyCustomValidator;

impl TransitionValidator for MyCustomValidator {
    fn validate(
        &self,
        resource_id: &ResourceId,
        current_state: &RegisterState,
        new_state: &RegisterState,
        context: &ValidationContext,
    ) -> Result<ValidationResult> {
        // Custom validation logic
        if current_state == &RegisterState::Active &&
            matches!(new_state, RegisterState::Consumed { .. }) {
            // Example: Only allow consumption of resources older than 7 days
            let creation_time = context.get_resource_creation_time(resource_id)?;
            let min_age = Duration::days(7);
            
            if time::now() - creation_time < min_age {
                return Ok(ValidationResult::invalid(
                    "Resource must be at least 7 days old to be consumed"
                ));
            }
        }
        
        Ok(ValidationResult::valid())
    }
}

// Add the custom validator to the lifecycle manager
lifecycle_manager.config_mut().add_validator(Box::new(MyCustomValidator));
```

## Best Practices

1. **Use Operation Model**: Use the operation model for state transitions rather than direct API calls for better tracking and validation.

2. **Check Current State**: Always check the current state before attempting state transitions.

3. **Validate Relationships**: Ensure resources maintain valid relationships during lifecycle changes.

4. **Use Capability Validation**: Validate capabilities for sensitive state transitions like freezing or consuming.

5. **Handle Locked Resources**: Implement proper lock management including timeouts and deadlock prevention.

6. **Audit State Changes**: Log all state changes for audit purposes.

7. **Use Batch Operations**: Group related state changes in atomic batches.

8. **Consider Temporal Facts**: Use temporal facts for condition-based state transitions.

9. **Implement Proper Error Handling**: Provide clear error messages for invalid state transitions.

10. **Use Events**: Subscribe to lifecycle events to react to state changes.

## Implementation Status

The resource lifecycle management system is fully implemented in the Causality system:

- ✅ Core `ResourceRegisterLifecycleManager` structure
- ✅ Complete state machine with validation
- ✅ Integration with the operation model
- ✅ Relationship-aware transitions
- ✅ Capability-based authorization
- ✅ Conditional state transitions
- ✅ Batch operations
- ✅ Event system

## Future Enhancements

1. **Hierarchical State Model**: Support for more complex hierarchical states
2. **State Transition Hooks**: Customizable hooks for state transitions
3. **Archival Strategy**: More sophisticated archival strategy with data tiering
4. **State Visualization**: Tools for visualizing resource state transitions
5. **State Prediction**: Predictive analysis of future state transitions 