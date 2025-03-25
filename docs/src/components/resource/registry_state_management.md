<!-- State management for the registry -->
<!-- Original file: docs/src/registry_state_management.md -->

# Registry State Management in Causality

## Overview

This document describes the state management mechanisms for registry entities within the Causality architecture. Registry state management encompasses the processes and systems responsible for maintaining, tracking, and validating the states of registered entities across their lifecycles. Effective state management ensures consistency, integrity, and accurate representation of system resources.

## Core Concepts

### Entity State Model

Each registry entity maintains a state that evolves throughout its lifecycle:

```rust
pub struct EntityState<T> {
    /// Entity identifier
    entity_id: EntityId,
    
    /// Current state data
    state_data: T,
    
    /// State revision number
    revision: u64,
    
    /// Timestamp of last update
    last_updated: Timestamp,
    
    /// Who or what last updated this state
    updated_by: Identity,
    
    /// Validation status
    validation_status: ValidationStatus,
    
    /// Metadata associated with this state
    metadata: HashMap<String, Value>,
}
```

### State Transitions

State transitions define how entities move between states:

```rust
pub struct StateTransition<T> {
    /// Entity identifier
    entity_id: EntityId,
    
    /// Previous state
    previous_state: Option<EntityState<T>>,
    
    /// New state
    new_state: EntityState<T>,
    
    /// Reason for the transition
    transition_reason: TransitionReason,
    
    /// Timestamp of the transition
    timestamp: Timestamp,
    
    /// Who or what initiated this transition
    initiated_by: Identity,
    
    /// Associated operation that caused this transition
    operation: Option<OperationId>,
}
```

## State Management System

### State Manager

The State Manager coordinates entity state lifecycle:

```rust
pub struct StateManager<T> {
    /// State storage
    state_storage: Arc<dyn StateStorage<T>>,
    
    /// State validators
    validators: Vec<Box<dyn StateValidator<T>>>,
    
    /// State transition handlers
    transition_handlers: Vec<Box<dyn StateTransitionHandler<T>>>,
    
    /// State change observers
    observers: Vec<Box<dyn StateChangeObserver<T>>>,
    
    /// Configuration
    config: StateManagerConfig,
}

impl<T: Send + Sync + Clone + 'static> StateManager<T> {
    /// Initialize state for a new entity
    pub fn initialize_state(
        &self,
        entity_id: EntityId,
        initial_state: T,
        context: &StateContext,
    ) -> Result<EntityState<T>, StateError> {
        // Create initial state
        let state = EntityState {
            entity_id,
            state_data: initial_state,
            revision: 0,
            last_updated: system.current_time(),
            updated_by: context.identity().clone(),
            validation_status: ValidationStatus::Pending,
            metadata: HashMap::new(),
        };
        
        // Validate initial state
        let validated_state = self.validate_state(state, context)?;
        
        // Store the state
        self.state_storage.store_state(&validated_state)?;
        
        // Notify observers
        for observer in &self.observers {
            observer.on_state_initialized(&validated_state, context)?;
        }
        
        Ok(validated_state)
    }
    
    /// Update entity state
    pub fn update_state(
        &self,
        entity_id: EntityId,
        state_update: StateUpdate<T>,
        context: &StateContext,
    ) -> Result<EntityState<T>, StateError> {
        // Get current state
        let current_state = self.state_storage.get_state(&entity_id)?;
        
        // Apply the update
        let (new_state, diff) = self.apply_update(&current_state, state_update)?;
        
        // Create state transition
        let transition = StateTransition {
            entity_id,
            previous_state: Some(current_state.clone()),
            new_state: new_state.clone(),
            transition_reason: state_update.reason().clone(),
            timestamp: system.current_time(),
            initiated_by: context.identity().clone(),
            operation: context.operation_id(),
        };
        
        // Process the transition
        let processed_transition = self.process_transition(transition, context)?;
        
        // Store the updated state
        self.state_storage.store_state(&processed_transition.new_state)?;
        
        // Record the transition
        self.state_storage.record_transition(&processed_transition)?;
        
        // Notify observers
        for observer in &self.observers {
            observer.on_state_updated(&current_state, &processed_transition.new_state, &diff, context)?;
        }
        
        Ok(processed_transition.new_state)
    }
    
    /// Validate entity state
    fn validate_state(
        &self,
        state: EntityState<T>,
        context: &StateContext,
    ) -> Result<EntityState<T>, StateError> {
        let mut validation_results = Vec::new();
        
        // Run all validators
        for validator in &self.validators {
            let result = validator.validate_state(&state, context)?;
            validation_results.push(result);
        }
        
        // Determine overall validation status
        let validation_status = if validation_results.iter().all(|r| r.is_valid()) {
            ValidationStatus::Valid
        } else {
            let errors: Vec<_> = validation_results.iter()
                .filter(|r| !r.is_valid())
                .map(|r| r.error_message().unwrap_or_default())
                .collect();
            
            ValidationStatus::Invalid(errors.join(", "))
        };
        
        // Update validation status
        let mut validated_state = state;
        validated_state.validation_status = validation_status;
        
        Ok(validated_state)
    }
    
    /// Process a state transition
    fn process_transition(
        &self,
        transition: StateTransition<T>,
        context: &StateContext,
    ) -> Result<StateTransition<T>, StateError> {
        let mut processed_transition = transition;
        
        // Apply transition handlers
        for handler in &self.transition_handlers {
            if handler.handles_transition(&processed_transition) {
                processed_transition = handler.handle_transition(processed_transition, context)?;
            }
        }
        
        Ok(processed_transition)
    }
}
```

### State Storage

Manages persistence of entity states:

```rust
pub trait StateStorage<T>: Send + Sync {
    /// Store entity state
    fn store_state(&self, state: &EntityState<T>) -> Result<(), StateError>;
    
    /// Get current state for an entity
    fn get_state(&self, entity_id: &EntityId) -> Result<EntityState<T>, StateError>;
    
    /// Get state history for an entity
    fn get_state_history(
        &self,
        entity_id: &EntityId,
        range: Option<TimeRange>,
    ) -> Result<Vec<EntityState<T>>, StateError>;
    
    /// Record a state transition
    fn record_transition(&self, transition: &StateTransition<T>) -> Result<(), StateError>;
    
    /// Get transitions for an entity
    fn get_transitions(
        &self,
        entity_id: &EntityId,
        range: Option<TimeRange>,
    ) -> Result<Vec<StateTransition<T>>, StateError>;
}
```

### State Validation

Validates entity states:

```rust
pub trait StateValidator<T>: Send + Sync {
    /// Validate an entity state
    fn validate_state(
        &self,
        state: &EntityState<T>,
        context: &StateContext,
    ) -> Result<ValidationResult, ValidationError>;
    
    /// Get validator name
    fn name(&self) -> &str;
}
```

### State Observer

Observes state changes:

```rust
pub trait StateChangeObserver<T>: Send + Sync {
    /// Called when state is initialized
    fn on_state_initialized(
        &self,
        state: &EntityState<T>,
        context: &StateContext,
    ) -> Result<(), ObserverError>;
    
    /// Called when state is updated
    fn on_state_updated(
        &self,
        old_state: &EntityState<T>,
        new_state: &EntityState<T>,
        diff: &StateDiff<T>,
        context: &StateContext,
    ) -> Result<(), ObserverError>;
    
    /// Called when state is deleted
    fn on_state_deleted(
        &self,
        state: &EntityState<T>,
        context: &StateContext,
    ) -> Result<(), ObserverError>;
}
```

## Specialized Registry State Managers

### Resource State Manager

Manages resource states:

```rust
pub struct ResourceStateManager {
    /// Core state manager
    state_manager: StateManager<ResourceState>,
    
    /// Resource-specific validators
    resource_validators: Vec<Box<dyn ResourceStateValidator>>,
    
    /// Resource-specific observers
    resource_observers: Vec<Box<dyn ResourceStateObserver>>,
}

impl ResourceStateManager {
    /// Initialize state for a new resource
    pub fn initialize_resource_state(
        &self,
        resource: &Resource,
        context: &StateContext,
    ) -> Result<ResourceState, StateError> {
        // Create initial state
        let initial_state = ResourceState {
            resource_id: resource.id(),
            attributes: resource.attributes().clone(),
            status: ResourceStatus::Active,
            version: 1,
        };
        
        // Initialize state
        let entity_state = self.state_manager.initialize_state(
            resource.id().into(),
            initial_state.clone(),
            context,
        )?;
        
        Ok(entity_state.state_data)
    }
    
    /// Update resource attributes
    pub fn update_resource_attributes(
        &self,
        resource_id: ResourceId,
        attribute_updates: HashMap<String, Value>,
        reason: &str,
        context: &StateContext,
    ) -> Result<ResourceState, StateError> {
        // Create state update
        let update = StateUpdate::new()
            .with_updater(|state: &mut ResourceState| {
                for (key, value) in &attribute_updates {
                    state.attributes.insert(key.clone(), value.clone());
                }
                state.version += 1;
                Ok(())
            })
            .with_reason(TransitionReason::AttributeUpdate(reason.to_string()));
        
        // Apply update
        let entity_state = self.state_manager.update_state(
            resource_id.into(),
            update,
            context,
        )?;
        
        Ok(entity_state.state_data)
    }
    
    /// Change resource status
    pub fn change_resource_status(
        &self,
        resource_id: ResourceId,
        new_status: ResourceStatus,
        reason: &str,
        context: &StateContext,
    ) -> Result<ResourceState, StateError> {
        // Create state update
        let update = StateUpdate::new()
            .with_updater(|state: &mut ResourceState| {
                state.status = new_status;
                state.version += 1;
                Ok(())
            })
            .with_reason(TransitionReason::StatusChange(reason.to_string()));
        
        // Apply update
        let entity_state = self.state_manager.update_state(
            resource_id.into(),
            update,
            context,
        )?;
        
        Ok(entity_state.state_data)
    }
}
```

### Capability State Manager

Manages capability states:

```rust
pub struct CapabilityStateManager {
    /// Core state manager
    state_manager: StateManager<CapabilityState>,
}

impl CapabilityStateManager {
    /// Initialize state for a new capability
    pub fn initialize_capability_state(
        &self,
        capability: &Capability,
        context: &StateContext,
    ) -> Result<CapabilityState, StateError> {
        // Create initial state
        let initial_state = CapabilityState {
            capability_id: capability.id(),
            status: CapabilityStatus::Active,
            permissions: capability.permissions().clone(),
            constraints: capability.constraints().clone(),
            version: 1,
        };
        
        // Initialize state
        let entity_state = self.state_manager.initialize_state(
            capability.id().into(),
            initial_state.clone(),
            context,
        )?;
        
        Ok(entity_state.state_data)
    }
    
    /// Revoke a capability
    pub fn revoke_capability(
        &self,
        capability_id: CapabilityId,
        reason: &str,
        context: &StateContext,
    ) -> Result<CapabilityState, StateError> {
        // Create state update
        let update = StateUpdate::new()
            .with_updater(|state: &mut CapabilityState| {
                state.status = CapabilityStatus::Revoked;
                state.version += 1;
                Ok(())
            })
            .with_reason(TransitionReason::Revocation(reason.to_string()));
        
        // Apply update
        let entity_state = self.state_manager.update_state(
            capability_id.into(),
            update,
            context,
        )?;
        
        Ok(entity_state.state_data)
    }
}
```

## State Change Tracking

### State Diffing

Tracks differences between states:

```rust
pub struct StateDiff<T> {
    /// Entity ID
    entity_id: EntityId,
    
    /// Changed fields
    changed_fields: Vec<FieldChange<T>>,
    
    /// Revision change
    revision_change: (u64, u64),
    
    /// Timestamp
    timestamp: Timestamp,
}

pub enum FieldChange<T> {
    /// Simple field change
    Simple {
        /// Field name
        field: String,
        /// Old value
        old_value: Value,
        /// New value
        new_value: Value,
    },
    
    /// Complex state-specific change
    Complex {
        /// Change type
        change_type: String,
        /// Change representation
        representation: String,
        /// Custom diff data
        diff_data: T,
    },
}
```

### State Snapshots

Tracks point-in-time entity states:

```rust
pub struct StateSnapshot<T> {
    /// Entity ID
    entity_id: EntityId,
    
    /// Snapshot timestamp
    timestamp: Timestamp,
    
    /// State at this point in time
    state: EntityState<T>,
    
    /// Snapshot reason
    reason: SnapshotReason,
    
    /// Snapshot metadata
    metadata: HashMap<String, Value>,
}

impl<T: Clone> StateManager<T> {
    /// Create a snapshot of current entity state
    pub fn create_snapshot(
        &self,
        entity_id: EntityId,
        reason: SnapshotReason,
        context: &StateContext,
    ) -> Result<StateSnapshot<T>, StateError> {
        // Get current state
        let current_state = self.state_storage.get_state(&entity_id)?;
        
        // Create snapshot
        let snapshot = StateSnapshot {
            entity_id,
            timestamp: system.current_time(),
            state: current_state,
            reason,
            metadata: HashMap::new(),
        };
        
        // Store snapshot
        self.state_storage.store_snapshot(&snapshot)?;
        
        Ok(snapshot)
    }
}
```

## State Consistency

### Transactional State Updates

Ensures atomic state changes across multiple entities:

```rust
pub struct StateTransaction {
    /// Transaction ID
    id: TransactionId,
    
    /// State updates to apply
    updates: Vec<(EntityId, Box<dyn AnyStateUpdate>)>,
    
    /// State context
    context: StateContext,
}

impl StateManager<T> {
    /// Execute a state transaction
    pub fn execute_transaction(
        &self,
        transaction: StateTransaction,
    ) -> Result<TransactionResult, TransactionError> {
        // Begin storage transaction
        let storage_tx = self.state_storage.begin_transaction()?;
        
        let mut results = Vec::new();
        let mut error = None;
        
        // Apply each update within the transaction
        for (entity_id, update) in transaction.updates {
            // Get current state
            let current_state = match self.state_storage.get_state_in_transaction(&entity_id, &storage_tx) {
                Ok(state) => state,
                Err(e) => {
                    error = Some(TransactionError::StateFetchFailed(entity_id, e.to_string()));
                    break;
                }
            };
            
            // Apply the update
            let update_result = match self.apply_update_in_transaction(
                &current_state,
                update,
                &transaction.context,
                &storage_tx,
            ) {
                Ok(result) => result,
                Err(e) => {
                    error = Some(TransactionError::UpdateFailed(entity_id, e.to_string()));
                    break;
                }
            };
            
            results.push((entity_id, update_result));
        }
        
        // If any error occurred, rollback
        if error.is_some() {
            self.state_storage.rollback_transaction(storage_tx)?;
            return Err(error.unwrap());
        }
        
        // Otherwise, commit
        self.state_storage.commit_transaction(storage_tx)?;
        
        Ok(TransactionResult {
            transaction_id: transaction.id,
            results,
            timestamp: system.current_time(),
        })
    }
}
```

### Cross-Domain State Consistency

Maintains consistency across domains:

```rust
pub struct CrossDomainStateCoordinator {
    /// Local state manager
    local_state_manager: Arc<dyn AnyStateManager>,
    
    /// Cross-domain messenger
    messenger: Arc<CrossDomainMessenger>,
    
    /// Conflict resolution policies
    conflict_policies: ConflictResolutionPolicies,
}

impl CrossDomainStateCoordinator {
    /// Synchronize entity state with another domain
    pub fn synchronize_state(
        &self,
        entity_id: EntityId,
        target_domain: DomainId,
    ) -> Result<SyncResult, SyncError> {
        // Get local state
        let local_state = self.local_state_manager.get_any_state(&entity_id)?;
        
        // Create synchronization message
        let sync_message = CrossDomainMessage::StateSynchronization {
            entity_id,
            state: local_state.clone(),
            source_domain: system.domain_id(),
            timestamp: system.current_time(),
        };
        
        // Send message
        let response = self.messenger.send_and_wait_response(
            target_domain,
            sync_message,
            Duration::from_secs(10),
        )?;
        
        // Process response
        match response {
            CrossDomainMessage::StateSyncResult { result, .. } => {
                // Handle result
                match result {
                    SyncResultType::Success => {
                        Ok(SyncResult {
                            entity_id,
                            target_domain,
                            status: SyncStatus::Success,
                            timestamp: system.current_time(),
                        })
                    }
                    SyncResultType::Conflict { remote_state } => {
                        // Resolve conflict
                        self.resolve_state_conflict(entity_id, local_state, remote_state, target_domain)
                    }
                    SyncResultType::Error { message } => {
                        Err(SyncError::RemoteError(message))
                    }
                }
            }
            _ => Err(SyncError::UnexpectedResponse),
        }
    }
    
    /// Resolve a state conflict between domains
    fn resolve_state_conflict(
        &self,
        entity_id: EntityId,
        local_state: Box<dyn AnyEntityState>,
        remote_state: Box<dyn AnyEntityState>,
        remote_domain: DomainId,
    ) -> Result<SyncResult, SyncError> {
        // Get conflict resolution policy
        let policy = self.conflict_policies.get_policy_for_entity_type(local_state.entity_type())?;
        
        // Apply policy to resolve conflict
        let resolution = policy.resolve_conflict(&local_state, &remote_state)?;
        
        match resolution {
            ConflictResolution::UseLocal => {
                // Push local state again, forcing update
                self.force_sync_state(entity_id, remote_domain)
            }
            ConflictResolution::UseRemote => {
                // Update local state with remote state
                self.update_local_with_remote(entity_id, remote_state)
            }
            ConflictResolution::Merge(merged_state) => {
                // Update both local and remote with merged state
                self.sync_merged_state(entity_id, merged_state, remote_domain)
            }
        }
    }
}
```

## Usage Examples

### Managing Resource State

```rust
// Create the state context
let context = StateContext::new()
    .with_identity(caller_identity)
    .with_operation_id(operation_id)
    .with_metadata("source", "api_request");

// Initialize a resource's state
let resource_state = resource_state_manager.initialize_resource_state(
    &resource,
    &context,
)?;

println!("Initialized resource state: {:?}", resource_state);

// Update resource attributes
let updated_state = resource_state_manager.update_resource_attributes(
    resource.id(),
    HashMap::from([
        ("status".to_string(), Value::String("published".to_string())),
        ("last_modified".to_string(), Value::String(system.current_time().to_string())),
    ]),
    "Publishing document",
    &context,
)?;

println!("Updated resource attributes: {:?}", updated_state);

// Archive a resource
let archived_state = resource_state_manager.change_resource_status(
    resource.id(),
    ResourceStatus::Archived,
    "Document is no longer needed",
    &context,
)?;

println!("Archived resource: {:?}", archived_state);
```

### Transactional State Updates

```rust
// Create a transaction for updating multiple resources
let mut transaction = StateTransaction::new(TransactionId::generate(), context.clone());

// Add updates to transaction
transaction.add_update(
    document.id().into(),
    Box::new(ResourceStateUpdate::new()
        .with_status_change(ResourceStatus::Published)
        .with_reason("Publishing document")),
);

transaction.add_update(
    metadata.id().into(),
    Box::new(ResourceStateUpdate::new()
        .with_attribute_update("status", Value::String("published".to_string()))
        .with_reason("Updating metadata to match document")),
);

// Execute the transaction
let result = state_manager.execute_transaction(transaction)?;

println!("Transaction completed with {} updates", result.results.len());
```

### State History and Snapshots

```rust
// Get state history for a resource
let state_history = resource_state_manager.get_resource_state_history(
    resource_id,
    TimeRange::new(
        Timestamp::days_ago(30),
        system.current_time(),
    ),
)?;

println!("Resource has undergone {} state changes in the last 30 days", state_history.len());

// Create a named snapshot
let snapshot = resource_state_manager.create_snapshot(
    resource_id.into(),
    SnapshotReason::UserInitiated("Pre-migration backup".to_string()),
    &context,
)?;

println!("Created state snapshot at {}", snapshot.timestamp);

// Restore from a snapshot
let restored_state = resource_state_manager.restore_from_snapshot(
    resource_id.into(),
    snapshot.timestamp,
    "Reverting failed migration",
    &context,
)?;

println!("Restored resource state: {:?}", restored_state);
```

### Cross-Domain State Synchronization

```rust
// Synchronize resource state with another domain
let sync_result = cross_domain_state_coordinator.synchronize_state(
    resource_id.into(),
    remote_domain_id,
)?;

println!("Synchronization result: {:?}", sync_result.status);

// Handle potential conflicts
if let SyncStatus::Conflict(details) = sync_result.status {
    println!("State conflict detected: {}", details);
    
    // Resolve manually or via policy
    let resolution = conflict_resolver.resolve_conflict(
        resource_id.into(),
        remote_domain_id,
        ConflictResolutionPolicy::PreferLocal,
    )?;
    
    println!("Conflict resolved with strategy: {:?}", resolution.strategy);
}
```

## Implementation Status

The current implementation status of Registry State Management:

- ✅ Core state management abstractions
- ✅ Basic state storage and retrieval
- ✅ Resource state management
- ⚠️ Capability state management (partially implemented)
- ⚠️ Relationship state management (partially implemented)
- ⚠️ State transaction support (partially implemented)
- ❌ State snapshots (not yet implemented)
- ❌ Cross-domain state synchronization (not yet implemented)

## Future Enhancements

Planned future enhancements for Registry State Management:

1. **Time-Travel State Queries**: Enhanced support for historical state queries
2. **State Compression**: Efficient storage for historical states
3. **State Versioning**: First-class versioning of entity schemas and states
4. **State Subscriptions**: Real-time notifications for state changes
5. **Predictive State Management**: Predicting future state changes based on historical patterns
6. **State Dependencies**: Tracking and enforcing dependencies between entity states
7. **Advanced Conflict Resolution**: More sophisticated conflict resolution strategies 