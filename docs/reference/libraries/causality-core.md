# causality-core Library Reference

*This document provides reference information for the `causality-core` crate.*

*Last updated: 2023-08-20*

## Overview

The `causality-core` crate implements the foundational systems of Causality, including the unified resource system, effect system, and capability system. It provides the core abstractions and implementations that other components build upon.

## Key Modules

### causality_core::resource

Resource system implementation, providing core resource management and access control.

```rust
use causality_core::resource::{
    ResourceManager,
    ResourceGuard,
    ResourceLock,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `ResourceManager` | Central manager for resource instances |
| `ResourceGuard<T>` | RAII guard for safe resource access |
| `ResourceLock` | Lock for resource concurrency control |
| `ResourceRegistry` | Registry of available resource types |

### causality_core::effect

Effect system implementation, providing type-safe effectful computation.

```rust
use causality_core::effect::{
    EffectHandler,
    EffectSystem,
    EffectResult,
    Effectful,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `EffectHandler<E>` | Handler for a specific effect type |
| `EffectSystem` | Registry and executor for effects |
| `EffectResult<T, E>` | Result of an effectful computation |
| `Effectful<T, E>` | Trait for effectful computations |

### causality_core::capability

Capability system implementation, providing capability-based security.

```rust
use causality_core::capability::{
    CapabilityVerifier,
    CapabilityChain,
    CapabilityGuard,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `CapabilityVerifier` | Verifies capability chains |
| `CapabilityChain` | Chain of capabilities with delegation |
| `CapabilityGuard` | RAII guard for capability usage |

### causality_core::content

Content addressing system implementation.

```rust
use causality_core::content::{
    ContentStore,
    ContentVerifier,
    ContentEntry,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `ContentStore` | Storage for content-addressed data |
| `ContentVerifier` | Verifies content hashes |
| `ContentEntry` | Entry in the content store |

### causality_core::time

Time system implementation.

```rust
use causality_core::time::{
    TimeSystem,
    TimelineBuilder,
    CausalClock,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `TimeSystem` | System for managing causal time |
| `TimelineBuilder` | Builder for causal timelines |
| `CausalClock` | Clock for tracking causal time |

### causality_core::typemap

Type-safe registry for heterogeneous types.

```rust
use causality_core::typemap::{
    TypeMap,
    TypeMapKey,
    TypeMapEntry,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `TypeMap` | Map of types to values |
| `TypeMapKey<T>` | Key for TypeMap entries |
| `TypeMapEntry<T>` | Entry in a TypeMap |

## Effect System Integration

As part of ADR-032, the effect system has been integrated directly into the `causality-core` crate. This integration provides a unified approach to handling side effects within the Causality system.

### Three-Layer Architecture

The effect system follows a three-layer architecture:

1. **Algebraic Effect Layer**
   - Defines the core algebraic effect abstractions
   - Implementation of effect handlers and continuations
   - Type-safe effect composition

2. **Effect Constraints Layer**
   - Defines constraints on effects based on capabilities
   - Links effect execution to the capability system
   - Provides effect authorization and audit

3. **Domain Implementation Layer**
   - Domain-specific effect implementations
   - Integration with external systems
   - Resource-specific effect handlers

### Effect Handler Registration

Effect handlers are registered with the `EffectSystem`:

```rust
// Create an effect system
let mut effect_system = EffectSystem::new();

// Register a handler for IO effects
effect_system.register_handler(
    IOEffectHandler::new()
)?;

// Register a handler for state effects
effect_system.register_handler(
    StateEffectHandler::new()
)?;
```

### Effectful Computations

The `Effectful` trait defines computations that can have effects:

```rust
// Define an effectful computation
fn read_and_process() -> impl Effectful<String, IOEffect> {
    Effectful::new(|ctx| {
        // Perform an IO effect
        let data = ctx.perform(ReadFile { path: "data.txt".into() })?;
        
        // Process the data
        let processed = data.to_uppercase();
        
        Ok(processed)
    })
}
```

### Effect Execution

Effects are executed by the `EffectSystem`:

```rust
// Execute an effectful computation
let result = effect_system.execute(
    read_and_process(), 
    EffectContext::new()
)?;
```

### Resource-Effect Integration

Resources can expose effectful operations:

```rust
impl Database {
    // Query the database with an effectful operation
    pub fn query(&self, query: String) -> impl Effectful<QueryResult, DatabaseEffect> {
        Effectful::new(move |ctx| {
            // Perform a database effect
            let result = ctx.perform(ExecuteQuery { query })?;
            Ok(result)
        })
    }
}
```

## Resource Locking and Concurrency

The resource system provides RAII-based locking for resource concurrency:

```rust
// Acquire a resource lock
let db_guard = resource_manager.get::<Database>("main_db", LockMode::Write)?;

// Use the resource (automatically unlocked when guard is dropped)
let result = db_guard.query("SELECT * FROM users")?;
```

## Usage Example

```rust
use causality_core::{
    resource::{ResourceManager, LockMode},
    effect::{EffectSystem, Effectful, EffectContext},
    capability::{CapabilityVerifier},
};

// Create a resource manager
let mut resource_manager = ResourceManager::new();

// Register resource types
resource_manager.register::<Database>("database")?;
resource_manager.register::<FileSystem>("filesystem")?;

// Create an effect system
let mut effect_system = EffectSystem::new();

// Register effect handlers
effect_system.register_handler(DatabaseEffectHandler::new())?;
effect_system.register_handler(FileSystemEffectHandler::new())?;

// Create a capability verifier
let capability_verifier = CapabilityVerifier::new();

// Create a resource instance
resource_manager.create("main_db", Database::new())?;

// Define an effectful computation
fn export_users() -> impl Effectful<(), DatabaseEffect + FileSystemEffect> {
    Effectful::new(|ctx| {
        // Get database resource
        let db = ctx.perform(GetResource { 
            id: "main_db".into(), 
            lock_mode: LockMode::Read 
        })?;
        
        // Query the database
        let users = db.query("SELECT * FROM users")?;
        
        // Write to filesystem
        ctx.perform(WriteFile { 
            path: "users.json".into(), 
            content: serde_json::to_string(&users)?.into_bytes()
        })?;
        
        Ok(())
    })
}

// Execute the computation
let result = effect_system.execute(
    export_users(),
    EffectContext::new()
)?;
```

## References

- [ADR-032: Role-Based Resource System](../../../spec/adr_032-role-based-resource-system.md)
- [System Contract](../../../spec/system_contract.md)
- [Effect System Architecture](../../architecture/core/effect-system.md) 