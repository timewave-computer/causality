# Unified System Components Architecture

*This document explains the unified system components architecture based on [ADR-032](../../spec/adr_032_consolidated_agent_resource_system.md).*

*Last updated: 2023-08-20*

## Overview

The Causality system is built on a foundation of unified system components that work together to provide a coherent, modular architecture. This document explains the key components, how they interact, and how they are implemented in the codebase.

## Component Architecture

The system is organized around the following primary components:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          Causality System Architecture                      │
│                                                                             │
│  ┌───────────────┐    ┌───────────────┐    ┌───────────────┐               │
│  │    Agents     │    │  Operations   │    │   Resources   │               │
│  │               │    │               │    │               │               │
│  │  ┌─────────┐  │    │  ┌─────────┐  │    │  ┌─────────┐  │               │
│  │  │ User    │  │    │  │ Request │  │    │  │ State   │  │               │
│  │  │ Agent   │──┼────┼─▶│ Effect  │──┼────┼─▶│ Change  │  │               │
│  │  └─────────┘  │    │  └─────────┘  │    │  └─────────┘  │               │
│  │  ┌─────────┐  │    │  ┌─────────┐  │    │  ┌─────────┐  │               │
│  │  │Committee│  │    │  │Present  │  │    │  │Resource │  │               │
│  │  │ Agent   │──┼────┼─▶│Capabil.│──┼────┼─▶│ Logic   │  │               │
│  │  └─────────┘  │    │  └─────────┘  │    │  └─────────┘  │               │
│  │  ┌─────────┐  │    │  ┌─────────┐  │    │  ┌─────────┐  │               │
│  │  │Operator │  │    │  │Validate │  │    │  │Lifecycle│  │               │
│  │  │ Agent   │──┼────┼─▶│ Auth.   │──┼────┼─▶│ Manage  │  │               │
│  │  └─────────┘  │    │  └─────────┘  │    │  └─────────┘  │               │
│  └───────────────┘    └───────────────┘    └───────────────┘               │
│                                                                             │
│  ┌───────────────┐    ┌───────────────┐    ┌───────────────┐               │
│  │  Capabilities │    │    Effects    │    │    Facts      │               │
│  │               │    │               │    │               │               │
│  │  ┌─────────┐  │    │  ┌─────────┐  │    │  ┌─────────┐  │               │
│  │  │ Issue   │  │    │  │ Effect  │  │    │  │Temporal │  │               │
│  │  │ Tokens  │◄─┼────┼──│ Trait   │◄─┼────┼──│ Facts   │  │               │
│  │  └─────────┘  │    │  └─────────┘  │    │  └─────────┘  │               │
│  │  ┌─────────┐  │    │  ┌─────────┐  │    │  ┌─────────┐  │               │
│  │  │Delegate │  │    │  │  Core   │  │    │  │Causal   │  │               │
│  │  │ Rights  │◄─┼────┼──│ Effects │◄─┼────┼──│Ordering │  │               │
│  │  └─────────┘  │    │  └─────────┘  │    │  └─────────┘  │               │
│  │  ┌─────────┐  │    │  ┌─────────┐  │    │  ┌─────────┐  │               │
│  │  │Constrain│  │    │  │ Effect  │  │    │  │External │  │               │
│  │  │ Usage   │◄─┼────┼──│Handlers │◄─┼────┼──│ Facts   │  │               │
│  │  └─────────┘  │    │  └─────────┘  │    │  └─────────┘  │               │
│  └───────────────┘    └───────────────┘    └───────────────┘               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Key Components

1. **Resources**: Stateful objects with lifecycles and metadata
2. **Capabilities**: Tokens of authority to perform operations on resources
3. **Effects**: Abstract, composable actions that can change system state
4. **Operations**: Requests to perform effects with authorization
5. **Agents**: Entities that hold capabilities and perform operations
6. **Facts**: Records of temporal and logical dependencies between actions

## Integration in Codebase

The unified system components architecture is implemented in the Causality codebase as follows:

### Crate Structure

```
┌─────────────────────────────────────────────────────────────────┐
│                   Causality Crate Structure                     │
│                                                                 │
│  ┌─────────────────┐                                            │
│  │ causality-types │                                            │
│  └────────┬────────┘                                            │
│           │                                                     │
│           ▼                                                     │
│  ┌─────────────────┐                                            │
│  │ causality-core  │                                            │
│  └────────┬────────┘                                            │
│           │                                                     │
│           ├─────────────────┬───────────────┐                   │
│           │                 │               │                   │
│           ▼                 ▼               ▼                   │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐   │
│  │ causality-agent │ │causality-domain │ │  causality-vm   │   │
│  └────────┬────────┘ └────────┬────────┘ └────────┬────────┘   │
│           │                   │                    │            │
│           │                   │                    │            │
│           └───────────────────┼────────────────────┘            │
│                               │                                 │
│                               ▼                                 │
│                     ┌─────────────────┐                         │
│                     │causality-engine │                         │
│                     └─────────────────┘                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Effect Integration in Core

The Effect System is integrated directly into the `causality-core` crate:

```
┌─────────────────────────────────────────────────────────────────┐
│                    causality-core Modules                       │
│                                                                 │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐           │
│  │  resource   │   │  capability │   │    agent    │           │
│  │             │   │             │   │             │           │
│  │ - Resource  │   │ - Capability│   │ - Agent     │           │
│  │ - Accessor  │◄──┤ - Registry  │◄──┤ - Profile   │           │
│  │ - Lock Mgr  │   │ - Rights    │   │ - Status    │           │
│  └──────┬──────┘   └──────┬──────┘   └──────┬──────┘           │
│         │                 │                  │                  │
│         │                 │                  │                  │
│         ▼                 ▼                  ▼                  │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐           │
│  │   effect    │   │    time     │   │   content   │           │
│  │             │   │             │   │             │           │
│  │ - Effect<R> │   │ - TimeMap   │   │ - Content   │           │
│  │ - Handlers  │◄──┤ - Clock     │◄──┤   Addressed │           │
│  │ - Interpreter│  │ - Temporal  │   │ - Hashing   │           │
│  └──────┬──────┘   └──────┬──────┘   └─────────────┘           │
│         │                 │                                     │
│         │                 │                                     │
│         ▼                 ▼                                     │
│  ┌─────────────┐   ┌─────────────┐                             │
│  │  operation  │   │   domain    │                             │
│  │             │   │             │                             │
│  │ - Operation │   │ - Domain    │                             │
│  │ - Auth      │◄──┤   Adapter   │                             │
│  │ - Execution │   │ - Crossing  │                             │
│  └─────────────┘   └─────────────┘                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

The Effect System provides interfaces for defining, executing, and composing effects:

```rust
/// Effect interface defining common operations
pub trait Effect<R>: ContentAddressed {
    /// Execute the effect with the given handler
    fn execute(self, handler: &dyn EffectHandler) -> EffectOutcome<R>;
    
    /// Get the effect's unique identifier
    fn effect_id(&self) -> EffectId;
    
    /// Get the resources this effect requires
    fn resources(&self) -> Vec<ResourceId>;
    
    /// Get the capabilities required for this effect
    fn required_capabilities(&self) -> Vec<Capability>;
    
    /// Compose with another effect
    fn and_then<U, F>(self, f: F) -> ComposedEffect<Self, F, R, U>
    where
        F: FnOnce(R) -> Box<dyn Effect<U>>,
        Self: Sized;
}
```

## Three-Layer Effect Architecture

The Effect System follows a three-layer architecture:

### 1. Algebraic Effect Layer

This layer defines the core interfaces and abstractions:

- `Effect<R>` trait: Base interface for all effects
- `EffectOutcome<T>`: Result type for effect execution
- `Continuation<I,O>`: Interface for effect composition
- Core effect types and implementations

### 2. Effect Constraints Layer

This layer handles validation and constraints:

- Resource requirements declaration
- Capability requirements validation
- Type constraints enforcement
- Cross-domain validation
- Concurrency control

### 3. Domain Implementation Layer

This layer provides concrete implementations:

- Domain-specific effect handlers
- Resource management in effects
- Time integration
- ZK-VM integration for proof generation
- Cross-domain operations

## Component Interaction Flow

```
┌───────────┐     ┌───────────┐     ┌───────────┐     ┌───────────┐
│   Agent   │     │ Operation │     │  Effect   │     │ Resource  │
│           │     │           │     │           │     │           │
└─────┬─────┘     └─────┬─────┘     └─────┬─────┘     └─────┬─────┘
      │                 │                 │                 │
      │ 1. Initiate     │                 │                 │
      │─────────────────>                 │                 │
      │                 │                 │                 │
      │ 2. Present      │                 │                 │
      │   Capabilities  │                 │                 │
      │─────────────────>                 │                 │
      │                 │                 │                 │
      │                 │ 3. Validate     │                 │
      │                 │    Auth         │                 │
      │                 │─────────────────>                 │
      │                 │                 │                 │
      │                 │                 │ 4. Execute      │
      │                 │                 │    Effect       │
      │                 │                 │─────────────────>
      │                 │                 │                 │
      │                 │                 │ 5. Apply        │
      │                 │                 │    Resource     │
      │                 │                 │    Logic        │
      │                 │                 │<─────────────────
      │                 │                 │                 │
      │                 │ 6. Return       │                 │
      │                 │    Result       │                 │
      │                 │<─────────────────                 │
      │                 │                 │                 │
      │ 7. Receive      │                 │                 │
      │    Response     │                 │                 │
      │<─────────────────                 │                 │
      │                 │                 │                 │
```

## Key Benefits of Unified Architecture

1. **Unified Type System**: Resources, effects, capabilities, and operations share a consistent type system.
2. **Reduced Complexity**: Integration of effects into the core crate simplifies dependencies.
3. **Consistent Resource Access**: The Resource Accessor pattern provides a uniform way to interact with resources.
4. **Capability-Based Security**: All operations require appropriate capabilities.
5. **Composable Effects**: Effects can be composed into complex pipelines while maintaining type safety.
6. **Agent-Based Interaction**: The agent model provides a clear entry point for system interactions.
7. **Resource Concurrency**: Explicit resource locks with deterministic wait queues enable safe concurrent access.

## Implementation Examples

### Agents Using Effects

```rust
// Agent performing an operation using effects
async fn transfer_between_resources(
    agent_id: AgentId,
    source_id: ResourceId,
    target_id: ResourceId,
    amount: u64,
    capability_registry: &dyn CapabilityRegistry,
    resource_manager: &ResourceManager,
) -> Result<(), ResourceError> {
    // Verify agent has appropriate capabilities
    let has_capability = capability_registry
        .verify_capability(
            &agent_id,
            &source_id,
            CapabilityType::Withdraw,
        ).await?;
        
    if !has_capability {
        return Err(ResourceError::InsufficientCapabilities);
    }
    
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

### RAII Resource Guards

```rust
// Resource guard implementation using RAII pattern
pub struct ResourceGuard {
    manager: Arc<ResourceLockManager>,
    resource_id: ResourceId,
    mode: AccessMode,
}

impl ResourceGuard {
    pub fn new(
        manager: Arc<ResourceLockManager>,
        resource_id: ResourceId,
        mode: AccessMode,
    ) -> Self {
        Self {
            manager,
            resource_id,
            mode,
        }
    }
    
    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
    
    pub fn mode(&self) -> AccessMode {
        self.mode
    }
}

// Automatically release lock when guard goes out of scope
impl Drop for ResourceGuard {
    fn drop(&mut self) {
        self.manager.release(&self.resource_id, self.mode);
    }
}
```

## References

- [ADR-032: Role-Based Resource System](../../spec/adr_032_consolidated_agent_resource_system.md)
- [System Contract](../../spec/system_contract.md) 