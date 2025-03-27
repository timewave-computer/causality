# Resource System

*This document is derived from [ADR-003](../../../spec/adr_003_resource.md), [ADR-021](../../../spec/adr_021_resource_register_unification.md), [ADR-030](../../../spec/adr_030_deffered_hashing_out_of_vm.md), and the [System Contract](../../../spec/system_contract.md).*

*Last updated: 2023-08-25*

## Overview

The Resource System is a core architectural component of Causality that provides a universal framework for representing, managing, and operating on digital assets across domains. It enables content-addressed identification, lifecycle management, cross-domain operations, and secure access control for all resources in the system.

The Resource System unifies logical resource abstractions with their physical storage representation, providing a consistent model for both in-memory operations and on-chain state management.

## Core Concepts

### Unified ResourceRegister Model

The Resource System is built around the unified ResourceRegister model, which combines logical resource properties with physical register characteristics:

```rust
struct ResourceRegister {
    // Identity
    id: RegisterId,
    
    // Logical properties
    resource_logic: ResourceLogic,
    fungibility_domain: FungibilityDomain,
    quantity: Quantity,
    metadata: Value,
    
    // Physical properties
    state: RegisterState,
    nullifier_key: NullifierKey,
    
    // Provenance tracking
    controller_label: ControllerLabel,
    
    // Temporal context
    observed_at: TimeMapSnapshot,
}
```

This unified approach:
- Provides a single coherent abstraction for developers
- Ensures logical and physical representations stay in sync
- Simplifies cross-domain operations
- Reduces duplication in the codebase

### Content-Addressed Identity

All resources are content-addressed, meaning their identity is derived from their content:

```rust
trait ContentAddressed {
    /// Calculate the content hash of this object
    fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError>;
    
    /// Get the content hash of this object
    fn content_hash(&self) -> &ContentHash;
    
    /// Set the content hash of this object
    fn with_content_hash(self, hash: ContentHash) -> Self;
}
```

Benefits of content addressing:
- **Immutability**: Resources can't be changed without changing their identity
- **Verifiability**: Content hashes provide cryptographic verification
- **Deduplication**: Identical resources have identical identifiers
- **Composition**: Resources can securely reference other resources

### Resource Lifecycle States

Resources in the system transition through a well-defined set of states:

```rust
enum RegisterState {
    /// Register is active and can be operated on
    Active,
    
    /// Register is locked for a specific operation
    Locked {
        operation_id: OperationId,
        expiry: Timestamp,
    },
    
    /// Register is frozen (operations suspended)
    Frozen {
        reason: String,
        authority: Address,
    },
    
    /// Register is marked for deletion
    PendingDeletion {
        scheduled_time: Timestamp,
    },
    
    /// Register contains a tombstone (was deleted)
    Tombstone {
        deletion_time: Timestamp,
        content_hash: ContentHash,
    },
}
```

The Resource System enforces valid state transitions and tracks the full history of state changes.

### Resource-Scoped Concurrency

Resources are protected by explicit locks with deterministic wait queues, using a RAII (Resource Acquisition Is Initialization) pattern for automatic resource release:

```rust
/// Resource lock manager
pub struct ResourceLockManager {
    locks: Mutex<HashMap<ResourceId, LockEntry>>,
}

struct LockEntry {
    holder: Option<TaskId>,
    wait_queue: VecDeque<WaitingTask>,
}

/// Resource guard that auto-releases on drop (RAII pattern)
pub struct ResourceGuard {
    manager: Arc<ResourceLockManager>,
    resource: ResourceId,
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        self.manager.release(self.resource.clone());
    }
}
```

The resource-scoped concurrency provides:
- Explicit lock acquisition with multiple access modes
- Deterministic wait queues for consistent scheduling
- RAII-style resource guards for automatic release
- Deadlock prevention through ordered lock acquisition
- Fine-grained control over concurrent resource access
- Integration with the effect system for resource operations

### Resource Storage Strategies

Resources can be stored using different strategies depending on domain requirements:

```rust
enum StorageStrategy {
    // Full on-chain storage - all fields available to EVM
    FullyOnChain {
        visibility: StateVisibility,
    },
    
    // Commitment-based with ZK proofs - minimal on-chain footprint
    CommitmentBased {
        commitment: Option<Commitment>,
        nullifier: Option<NullifierId>,
    },
    
    // Hybrid - critical fields on-chain, others as commitments
    Hybrid {
        on_chain_fields: HashSet<FieldName>,
        remaining_commitment: Option<Commitment>,
    },
}
```

This flexibility allows:
- Different domains to implement storage differently
- Support for diverse on-chain availability requirements
- Optimized storage for different use cases

### Resource Relationships

Resources can establish and track relationships with other resources:

```rust
struct ResourceRelationship {
    // Source and target resources
    source_id: ResourceId,
    target_id: ResourceId,
    
    // Relationship type
    relationship_type: RelationshipType,
    
    // Metadata about the relationship
    metadata: HashMap<String, Value>,
}

enum RelationshipType {
    // Hierarchical relationship
    ParentChild,
    
    // Dependency relationship
    Dependency,
    
    // Mirror of resource in another domain
    Mirror,
    
    // Reference to another resource
    Reference,
    
    // Ownership relationship
    Ownership,
    
    // Custom relationship type
    Custom(String),
}
```

## Resource System Architecture

### Component Architecture

The Resource System consists of several key components:

1. **ResourceManager**: Central component for managing resources
   - Creates, reads, updates, and deletes resources
   - Enforces access control and authorization
   - Coordinates cross-domain operations

2. **ResourceAccessor**: Pattern for accessing resources
   - Type-safe access to resources
   - Domain-specific resource operations
   - Integration with the capability system

3. **ResourceLifecycleManager**: Manages resource lifecycle
   - Enforces valid state transitions
   - Records state change history
   - Implements domain-specific lifecycle logic

4. **RelationshipTracker**: Tracks relationships between resources
   - Manages resource dependencies
   - Enforces relationship constraints
   - Provides relationship queries

5. **ResourceStorageManager**: Manages resource storage
   - Implements different storage strategies
   - Handles content addressing and verification
   - Integrates with domain-specific storage

6. **ResourceLockManager**: Manages concurrent resource access
   - Provides explicit resource locking with deterministic wait queues
   - Issues ResourceGuard objects for RAII-style release
   - Prevents deadlocks through ordered lock acquisition
   - Supports different access modes (read, write, exclusive)

### The Resource Accessor Pattern

The Resource Accessor Pattern provides improved performance and security when working with resources:

```rust
/// Generic resource accessor trait
trait ResourceAccessor<R: Resource>: Send + Sync + 'static {
    /// Get a resource by ID
    async fn get(&self, id: &ResourceId) -> Result<Option<R>, ResourceError>;
    
    /// Query resources by criteria
    async fn query(&self, query: &ResourceQuery) -> Result<Vec<R>, ResourceError>;
    
    /// Create a new resource
    async fn create(&self, resource: R) -> Result<ResourceId, ResourceError>;
    
    /// Update an existing resource
    async fn update(&self, id: &ResourceId, resource: R) -> Result<(), ResourceError>;
    
    /// Delete a resource
    async fn delete(&self, id: &ResourceId) -> Result<(), ResourceError>;
    
    /// Acquire a lock on a resource
    async fn acquire(&self, id: &ResourceId, mode: AccessMode) -> Result<ResourceGuard, ResourceError>;
}
```

This pattern:
- Enables deferred content hash computation
- Implements specialized resource access for each domain
- Provides type-safe resource operations
- Integrates security checks into the accessor
- Provides concurrency control through lock acquisition

## Resource Operations with Effect System

The Resource System integrates with the Effect System for managing operations on resources. This integration follows the three-layer architecture:

### Algebraic Effect Layer

Resource operations are defined as effects:

```rust
/// Resource acquisition effect
pub struct AcquireResourceEffect<R> {
    resource_id: ResourceId,
    mode: AccessMode,
    continuation: Box<dyn Continuation<ResourceGuard, R>>,
}

impl<R> Effect<R> for AcquireResourceEffect<R> {
    fn execute(self, handler: &dyn EffectHandler) -> EffectOutcome<R> {
        let guard = handler.handle_acquire_resource(self.resource_id, self.mode)?;
        EffectOutcome::Success(self.continuation.apply(guard))
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        vec![self.resource_id.clone()]
    }
}
```

### Effect Constraints Layer

Resource effects declare their requirements:

```rust
/// Effect with resource constraints
impl<R> AcquireResourceEffect<R> {
    /// Get the resources required by this effect
    fn resources(&self) -> Vec<ResourceId> {
        vec![self.resource_id.clone()]
    }
    
    /// Get the capabilities required for this effect
    fn required_capabilities(&self) -> Vec<Capability> {
        match self.mode {
            AccessMode::Read => vec![Capability::ResourceRead(self.resource_id.clone())],
            AccessMode::Write => vec![Capability::ResourceWrite(self.resource_id.clone())],
            AccessMode::Exclusive => vec![Capability::ResourceExclusive(self.resource_id.clone())],
        }
    }
}
```

### Domain Implementation Layer

Domain-specific implementation of resource operations:

```rust
/// Domain adapter for resource operations
impl DomainAdapter for EthereumAdapter {
    /// Handle a resource acquisition effect
    async fn handle_effect<R>(
        &self,
        effect: Box<dyn Effect<R>>,
        context: &EffectContext,
    ) -> Result<EffectOutcome<R>, EffectError> {
        match effect.effect_type() {
            EffectType::AcquireResource => {
                let acquire_effect = effect.downcast::<AcquireResourceEffect<R>>()
                    .ok_or(EffectError::InvalidEffectType)?;
                
                // Domain-specific resource acquisition logic
                let resource_id = acquire_effect.resource_id();
                let mode = acquire_effect.mode();
                
                // Acquire the resource lock
                let guard = self.resource_manager.acquire(resource_id, mode).await?;
                
                // Apply the continuation
                let result = acquire_effect.continuation().apply(guard);
                Ok(EffectOutcome::Success(result))
            },
            // Other effect types
            _ => Err(EffectError::UnsupportedEffectType),
        }
    }
}
```

## Resource Locking and Concurrency Control

### Lock Acquisition Process

```rust
/// Acquire a lock on a resource
pub async fn acquire_resource(
    resource_id: ResourceId,
    mode: AccessMode,
    manager: &ResourceLockManager,
) -> Result<ResourceGuard, ResourceError> {
    match mode {
        // Read locks can be shared
        AccessMode::Read => {
            manager.acquire_shared(resource_id).await
        },
        // Write locks are exclusive to writers but allow readers
        AccessMode::Write => {
            manager.acquire_write(resource_id).await
        },
        // Exclusive locks prevent all other access
        AccessMode::Exclusive => {
            manager.acquire_exclusive(resource_id).await
        },
    }
}
```

### Wait Queue Management

```rust
/// Add a task to the wait queue for a resource
async fn add_to_wait_queue(
    resource_id: ResourceId, 
    task_id: TaskId,
    mode: AccessMode,
    manager: &ResourceLockManager,
) -> Result<ResourceGuard, ResourceError> {
    // Create a future that will be completed when the lock is available
    let (sender, receiver) = oneshot::channel();
    
    // Add to the queue
    {
        let mut locks = manager.locks.lock().unwrap();
        let entry = locks.entry(resource_id.clone()).or_insert_with(|| LockEntry {
            holder: None,
            wait_queue: VecDeque::new(),
        });
        
        entry.wait_queue.push_back(WaitingTask {
            task_id,
            mode,
            waker: sender,
        });
    }
    
    // Wait for the lock
    receiver.await.map_err(|_| ResourceError::LockAcquisitionFailed)?
}
```

### RAII-style Resource Guards

```rust
/// Example of using a resource guard
pub async fn transfer_between_resources(
    source_id: ResourceId,
    target_id: ResourceId,
    amount: u64,
    manager: &ResourceLockManager,
) -> Result<(), ResourceError> {
    // Acquire locks on both resources in a consistent order to prevent deadlocks
    let (source_id, target_id) = if source_id < target_id {
        (source_id, target_id)
    } else {
        (target_id, source_id)
    };
    
    // Acquire locks - these will be automatically released when they go out of scope
    let source_guard = manager.acquire(source_id, AccessMode::Write).await?;
    let target_guard = manager.acquire(target_id, AccessMode::Write).await?;
    
    // Perform the transfer
    let source = read_resource(&source_guard)?;
    let target = read_resource(&target_guard)?;
    
    // Update the resources
    if source.balance < amount {
        return Err(ResourceError::InsufficientBalance);
    }
    
    update_resource(&source_guard, |r| r.balance -= amount)?;
    update_resource(&target_guard, |r| r.balance += amount)?;
    
    // Guards are automatically released when they go out of scope
    Ok(())
}
```

## Creating a Resource

```rust
// Create a new resource
async fn create_token(
    manager: &ResourceManager,
    owner: &Address,
    token_type: TokenType,
    amount: u64,
) -> Result<ResourceId, ResourceError> {
    // Create the resource
    let token = ResourceRegister::new()
        .with_resource_logic(ResourceLogic::Fungible)
        .with_fungibility_domain(FungibilityDomain::Local)
        .with_quantity(amount)
        .with_metadata(json!({
            "type": token_type,
            "owner": owner,
            "created_at": current_time(),
        }))
        .with_controller_label(owner.to_string());
    
    // Create the resource
    manager.create(token).await
}
```

## Updating a Resource

```rust
// Update a resource
async fn update_token_quantity(
    manager: &ResourceManager,
    token_id: &ResourceId,
    new_quantity: u64,
) -> Result<(), ResourceError> {
    // Acquire a lock on the resource
    let guard = manager.acquire(token_id.clone(), AccessMode::Write).await?;
    
    // Update the resource
    manager.update_with_guard(&guard, |token| {
        token.quantity = new_quantity;
    Ok(())
    }).await
}
```

## Deleting a Resource

```rust
// Delete a resource
async fn burn_token(
    manager: &ResourceManager,
    token_id: &ResourceId,
) -> Result<(), ResourceError> {
    // Acquire an exclusive lock on the resource
    let guard = manager.acquire(token_id.clone(), AccessMode::Exclusive).await?;
    
    // Delete the resource
    manager.delete_with_guard(&guard).await
}
```

## Cross-Domain Operations

```rust
// Transfer a resource across domains
async fn cross_domain_transfer(
    source_manager: &ResourceManager,
    target_domain: &DomainId,
    resource_id: &ResourceId,
) -> Result<ResourceId, ResourceError> {
    // Acquire the resource lock
    let guard = source_manager.acquire(resource_id.clone(), AccessMode::Exclusive).await?;
    
    // Get the resource
    let resource = source_manager.get_with_guard(&guard).await?;
    
    // Create a proof of ownership
    let proof = source_manager.create_ownership_proof(resource_id).await?;
    
    // Get the target domain adapter
    let target_adapter = get_domain_adapter(target_domain)?;
    
    // Transfer the resource
    let target_id = target_adapter.import_resource(resource, proof).await?;
    
    // Mark the source resource as transferred
    source_manager.update_with_guard(&guard, |r| {
        r.state = RegisterState::Tombstone {
            deletion_time: current_time(),
            content_hash: r.content_hash().clone(),
        };
        Ok(())
    }).await?;
    
    Ok(target_id)
}
```

## Content Addressing Performance

Deferred content hashing improves performance:

```rust
// Create a resource with deferred content hash calculation
async fn create_resource_with_deferred_hash(
    accessor: &impl ResourceAccessor<ResourceRegister>,
    resource: ResourceRegister,
) -> Result<ResourceId, ResourceError> {
    // Create the resource without calculating the hash
    let id = accessor.create_deferred(resource).await?;
    
    // The hash will be calculated later during background processing
    
    Ok(id)
}
```

## Where Implemented

The Resource System is implemented in the following crates and modules:

| Component | Crate | Module |
|-----------|-------|--------|
| Resource Manager | `causality-core` | `causality_core::resource` |
| Resource Register | `causality-core` | `causality_core::resource::register` |
| Resource Accessor | `causality-core` | `causality_core::resource::accessor` |
| Content Addressing | `causality-core` | `causality_core::content` |
| Lock Manager | `causality-core` | `causality_core::resource::concurrency` |
| Storage Strategies | `causality-core` | `causality_core::resource::storage` |
| Cross-Domain Transfer | `causality-domains` | `causality_domains::transfer` |

## References

- [ADR-003: Resource System](../../../spec/adr_003_resource.md)
- [ADR-021: Resource Register Unification](../../../spec/adr_021_resource_register_unification.md)
- [ADR-030: Deferred Hashing](../../../spec/adr_030_deffered_hashing_out_of_vm.md)
- [System Contract](../../../spec/system_contract.md)
- [Effect System](./effect-system.md)
- [Agent-Based Resources](./role-based-resources.md)
