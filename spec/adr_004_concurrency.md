# ADR-004: Concurrency Model

**Note: This ADR is superseded by [ADR-032: Agent-Based Resource System](./adr_032_consolidated_agent_resource_system.md), which implements a comprehensive resource-based concurrency model with RAII guards and deterministic scheduling.**

## Status

Accepted

## Implementation Status

This ADR has been fully implemented with a comprehensive resource-scoped concurrency model. The key components implemented include:

1. **ResourceGuard and RAII Pattern**:
   - Implemented a robust `ResourceGuard<T>` in `src/concurrency/primitives/resource_guard.rs`
   - Uses RAII pattern for automatic resource release when guards go out of scope
   - Provides `Deref`/`DerefMut` traits for convenient access to resources
   - Includes additional features like `map()` for resource transformation

2. **ResourceManager**:
   - Implemented `ResourceManager` for handling registration, acquisition, and release of resources
   - Uses a wait queue mechanism to manage resource contention
   - Supports deadlock detection and prevention
   - Provides thread-safe access with `SharedResourceManager`

3. **Concurrency Primitives**:
   - Implements all proposed primitives: `barrier`, `race`, `fork`, and `timeout`
   - `Barrier` pattern with resource availability and condition support
   - `Race` pattern with variants like `race_ok`, `race_result`, and `race_until`
   - `Fork` pattern with `fork_join`, `fork_try_join`, and `fork_each` for parallel execution

4. **Wait Queue and Scheduler**:
   - Implements a deterministic `WaitQueue` for resource requests
   - `TaskScheduler` manages concurrent tasks with resource requirements
   - Prioritizes tasks based on priority and age
   - Provides a mechanism for deadlock detection

5. **Task Management**:
   - `TaskId` system for uniquely identifying concurrent operations
   - `ResourceContext` for tracking resource allocations
   - Support for acquiring multiple resources atomically
   - Proper cleanup on task completion

6. **Integration with Resource System**:
   - Resource-based concurrency integrates with the ResourceRegister model
   - Fine-grained locking at the resource level for maximum parallelism
   - Support for read vs. write locks for concurrent operations
   - Safe concurrent access to content-addressed resources

7. **VM Integration**:
   - `ResourceVmIntegration` for VM memory management
   - Resource allocation within the VM environment
   - Safe loading/storing of resources between VM memory and the resource system
   - VM memory segments with controlled access

The implementation provides a solid foundation for safe, deterministic concurrent execution in a resource-scoped model. It includes several enhancements beyond the original proposal, particularly in the areas of type-safe resource access, sophisticated concurrency patterns, and deadlock prevention.

For more details, see [docs/src/concurrency.md](/docs/src/concurrency.md).


## Context

Causality programs are **distributed, cross-domain programs** that operate across multiple chains and distributed ledgers. These programs must handle **asynchronous events** originating from different domains, while maintaining **strong causal consistency** across all system components.

Concurrency in this system is uniquely challenging because:

- Programs operate across **multiple independent domains**, each with its own clock, ordering rules, and finality guarantees
- Each program manages **multiple resources**, each with its own causal log and domain dependencies
- **Cross-program and cross-domain effects** need safe concurrency and atomicity across network and ledger boundaries
- Causality must preserve both **internal causal consistency** (program effects) and **external consistency** (domain observations)


## Decision

### Core Principle: Resource-Scoped Concurrency

The unit of concurrency in Causality is **the resource**, not the program, actor, or domain.

- Each resource maintains its own **per-resource effect log**
- Programs apply effects to resources, and these effects are **causally ordered per-resource**
- Disjoint resource sets can be acted upon concurrently
- All external resource interactions (deposits, withdrawals, transfers) flow through **account programs** that manage resource operations

This model **closely resembles**:
- **Software Transactional Memory (STM)** (resources act like transactional cells with versioned histories)
- **Optimistic Concurrency Control** (conflicts are resolved only when effects actually apply)
- **CRDT-inspired causal graphs**, where effects may fork and merge in a provable and replayable manner


### High-Level Principles

1. **Resource-Scoped Execution**: Programs apply effects to resources, and each resource processes its effects sequentially. Different resources can advance independently.

2. **Account Programs as Resource Managers**: External resource ownership resides in **account programs**, which expose:
    - Deposit/withdrawal interfaces for external resources
    - Fact observation links (external deposits become observed facts)
    - Internal transfer interfaces (cross-program resource transfers)

3. **Effects as Atomic Execution Units**: Each effect is an atomic state transition for a specific resource. Effects are the **fundamental units of execution and causality**.

4. **Per-Resource Effect Logs**: Each resource maintains a **content-addressed, append-only log** of all effects applied to it.

5. **Per-Resource Locks**: Effects for a given resource apply sequentially under lock. Disjoint resources do not block each other.

6. **External Consistency via Facts**: All external state (domain balances, prices, etc.) is observed and proven by users. Effects depending on external facts carry **fact snapshots**, linking internal causal domains to external domain states.


## System-Level Concurrency

At the **system level**, concurrency is managed through:

```rust
/// Manager for concurrent resource operations
pub struct ResourceConcurrencyManager {
    /// Per-resource locks
    resource_locks: HashMap<ResourceID, RwLock<()>>,
    /// Effect pipeline
    effect_pipeline: mpsc::Receiver<EffectOperation>,
    /// Resource scheduler
    scheduler: Arc<ResourceScheduler>,
    /// Time map for causal tracking
    time_map: Arc<TimeMap>,
}

impl ResourceConcurrencyManager {
    /// Submit a new effect for application
    pub fn submit_effect(&self, effect: Effect) -> Result<EffectID> {
        // Implementation
    }
    
    /// Process all pending effects
    pub fn process_effects(&mut self) -> Result<Vec<AppliedEffect>> {
        // Implementation
    }
    
    /// Acquire a lock on a resource
    pub fn acquire_resource_lock(&self, resource_id: &ResourceID) -> Result<ResourceGuard> {
        // Implementation
    }
}
```

The concurrency system provides:

1. **Global Effect Pipeline**: Proposed effects from all programs enter a global queue
2. **Resource Scheduler**: Effects are scheduled based on their resource access patterns
3. **Resource Locks**: When an effect applies to a resource, the system acquires an exclusive lock on that resource
4. **Per-Resource Logs**: Applied effects are appended to each resource's effect log
5. **Time Map Observations**: Effects carry **time map snapshots**, proving which external facts were observed at the time of application
6. **Parallel Account Programs**: Each account program processes requests for its user independently, allowing external actor requests to scale


## Program-Level Concurrency Primitives

Within a program, developers express concurrent workflows using **temporal combinators** in the Rust concurrency library:

```rust
/// Concurrency primitives for Causality programs
pub mod concurrency {
    /// Watch a resource until a condition is satisfied
    pub async fn watch<T: ResourceValue>(
        resource_id: ResourceID, 
        condition: impl Fn(&T) -> bool
    ) -> Result<T> {
        // Implementation
    }
    
    /// Create a barrier that resolves when all conditions are met
    pub async fn barrier(
        conditions: Vec<impl Future<Output = Result<()>>>
    ) -> Result<()> {
        // Implementation
    }
    
    /// Execute multiple futures concurrently, returning when any completes
    pub async fn race<T>(
        futures: Vec<impl Future<Output = Result<T>>>
    ) -> Result<T> {
        // Implementation
    }
    
    /// Spawn a concurrent task
    pub async fn fork<T>(
        task: impl Future<Output = Result<T>>
    ) -> JoinHandle<Result<T>> {
        // Implementation
    }
}
```

These primitives enable:

1. **watch** - Observe a resource or domain until a condition is satisfied
2. **barrier** - Wait until multiple conditions are satisfied
3. **race** - Execute multiple branches concurrently; return when any completes
4. **fork** - Spawn a concurrent child program or concurrent internal branch
5. **invoke** - Call another program asynchronously
6. **callback** - Register a response handler for an async invocation


## Resource Guards and Safe Access

To ensure safe concurrent access to resources, the system provides resource guards:

```rust
/// A guard providing safe access to a resource
pub struct ResourceGuard<'a> {
    /// Reference to the manager
    manager: &'a ResourceConcurrencyManager,
    /// ID of the resource
    resource_id: ResourceID,
    /// Lock type (read or write)
    lock_type: LockType,
}

impl<'a> ResourceGuard<'a> {
    /// Get the contents of the resource
    pub fn contents(&self) -> &ResourceContents {
        // Implementation
    }
    
    /// Update the resource contents (only available for write locks)
    pub fn update(&mut self, new_contents: ResourceContents) -> Result<()> {
        // Implementation
    }
    
    /// Release the guard (called on drop)
    fn release(self) {
        // Implementation
    }
}
```


## Logical Time and Causality

Causality does not rely on global clocks. Instead, it establishes ordering through:

```rust
/// Tracks causal relationships across domains
pub struct TimeMap {
    /// Lamport clock for internal events
    internal_clock: LamportClock,
    /// Domain timestamps for external events
    domain_timestamps: HashMap<DomainID, Vec<DomainTimestamp>>,
    /// Causal dependencies between events
    dependencies: HashMap<EventID, HashSet<EventID>>,
}

impl TimeMap {
    /// Record a new event in the time map
    pub fn record_event(&mut self, event: Event) -> Result<EventID> {
        // Implementation
    }
    
    /// Check if event A happened before event B
    pub fn happened_before(&self, a: &EventID, b: &EventID) -> Result<bool> {
        // Implementation
    }
    
    /// Get a snapshot of the time map
    pub fn snapshot(&self) -> TimeMapSnapshot {
        // Implementation
    }
}
```

The time system provides:

1. **Per-Resource Effect Logs** - Define causal order within each resource
2. **Fact Snapshots** - Link effects to the external facts they observed
3. **Program Vector Clocks** - Track internal causal order within each program


## Integration with RISC-V VM

The concurrency model integrates with the RISC-V VM through:

```rust
/// Manages resource concurrency within the VM
pub struct VmResourceManager {
    /// VM runtime
    vm: Arc<VirtualMachine>,
    /// Resource concurrency manager
    concurrency_manager: Arc<ResourceConcurrencyManager>,
    /// Resource access tracking
    access_tracker: ResourceAccessTracker,
}

impl VmResourceManager {
    /// Record resource access during VM execution
    pub fn track_resource_access(&mut self, resource_id: ResourceID, access_type: AccessType) {
        // Implementation
    }
    
    /// Validate resource access patterns
    pub fn validate_resource_access(&self) -> Result<()> {
        // Implementation
    }
    
    /// Apply effects to resources
    pub fn apply_effects(&mut self, effects: Vec<Effect>) -> Result<Vec<AppliedEffect>> {
        // Implementation
    }
}
```

VM resource operations follow these principles:

1. **Resource Access Tracking**: The VM tracks all resource reads and writes during execution
2. **Access Validation**: Before committing effects, the VM validates that all resource accesses were properly declared
3. **Effect Application**: Resource effects are applied atomically after successful VM execution
4. **Rollback on Failure**: If VM execution fails, all resource effects are rolled back


## Safe Concurrency Invariants

The concurrency model guarantees:

- **Per-resource sequential consistency:** Each resource sees a totally ordered sequence of effects
- **Cross-resource causal safety:** Effects across resources only depend on each other via explicit causal links
- **External consistency:** Each program's external dependencies are explicitly proven via observed facts in its effect log
- **Replayability:** Full replay of any program's execution requires only:
    - Its own effect log
    - The referenced fact logs
- **No direct resource ownership:** Programs do not own resources directly. All external resources are mediated via account programs with resource-based access


## Example: Cross-domain Swap

### Rust Implementation

```rust
/// A cross-domain token swap program
pub async fn cross_domain_swap(
    eth_amount: u64,
    sol_amount: u64,
    eth_user: Address,
    sol_user: Address,
    timeout: Duration,
) -> Result<()> {
    // Create resources for escrows
    let eth_escrow = create_escrow("ethereum", eth_amount)?;
    let sol_escrow = create_escrow("solana", sol_amount)?;
    
    // Fork deposit operations to run concurrently
    let eth_deposit = fork(async {
        deposit_to_escrow(eth_escrow, eth_amount, eth_user).await
    });
    
    let sol_deposit = fork(async {
        deposit_to_escrow(sol_escrow, sol_amount, sol_user).await
    });
    
    // Wait for both deposits to complete
    barrier(vec![eth_deposit, sol_deposit]).await?;
    
    // Race between successful swap and timeout
    race(vec![
        // Success path
        async {
            // Atomically claim both escrows
            claim_escrow(eth_escrow, sol_user).await?;
            claim_escrow(sol_escrow, eth_user).await?;
            Ok(())
        },
        // Timeout path
        async {
            time::sleep(timeout).await;
            // Refund both escrows
            refund_escrow(eth_escrow).await?;
            refund_escrow(sol_escrow).await?;
            Ok(())
        }
    ]).await?;
    
    Ok(())
}
```

### Resource Operations

```rust
/// Create an escrow resource
fn create_escrow(domain: &str, amount: u64) -> Result<ResourceID> {
    let resource_id = ResourceID::new();
    let contents = ResourceContents::TokenBalance {
        token_type: TokenType::from_domain(domain),
        address: Address::escrow(),
        amount,
    };
    
    RESOURCE_MANAGER.create_resource(contents, Address::system())?;
    Ok(resource_id)
}

/// Deposit funds to an escrow
async fn deposit_to_escrow(
    resource_id: ResourceID, 
    amount: u64, 
    from: Address
) -> Result<()> {
    // Create a fact for the external deposit
    let fact = Fact::new_deposit(from, amount);
    
    // Observe the fact in the domain
    FACT_OBSERVER.observe_fact(fact).await?;
    
    // Update the resource with a lock
    let guard = RESOURCE_MANAGER.acquire_resource_lock(resource_id)?;
    if let ResourceContents::TokenBalance { amount: ref mut balance, .. } = guard.contents() {
        *balance += amount;
    }
    
    Ok(())
}
```


## Architectural Integration

| Component | Role |
|---|---|
| ResourceManager | Manages resource lifecycle and operations |
| ResourceLedger | Tracks per-resource logs and locks |
| Account Programs | External resource gateway, isolates user from direct resource ownership |
| TimeMap | Links internal effects to external facts, preserving cross-domain consistency |
| VirtualMachine | Executes resource operations in a controlled environment |
| Concurrency primitives | Enables structured concurrent operations on resources |


## Simulation and Replay

- In **simulations**, in-memory locks simulate resource-level concurrency
- In **local/multi-process mode**, each actor maintains independent process-local logs
- In **geo-distributed mode**, locks translate into distributed lease/acquire operations
- Replay uses only the effect logs (per-resource) and fact logs (per-domain), meaning replay requires no live RPC queries


## Performance Considerations

The resource concurrency model includes optimizations for:

1. **Lock Granularity**: Fine-grained locks at the resource level maximize parallelism
2. **Read vs. Write Locks**: Shared read locks for concurrent non-mutating operations
3. **Batched Operations**: Multiple resource operations can be batched in a single transaction
4. **Predictive Scheduling**: The scheduler can predict resource access patterns based on program analysis
5. **Resource Locality**: Resources accessed together are stored together for improved cache efficiency


## Summary

- Causality programs describe **concurrent goals** declaratively
- The system schedules actual concurrency based on **resource safety**
- Programs remain replayable and auditable at all times
- No program can violate causal consistency or race against external facts
- Resource-scoped concurrency fits the **account program model** directly
- The integration with the RISC-V VM enables safe, verifiable concurrent execution
