# Causality Core

The `causality-core` crate provides foundational building blocks for the Causality system. It implements cross-cutting primitives and utilities that provide essential functionality without carrying domain-specific assumptions.

## Architectural Role

As the name suggests, Core sits at the center of the dependency graph, with a minimal set of dependencies itself (only `causality-types` and `causality-crypto`). It provides infrastructure that other crates can build upon without creating circular dependencies.

```
     ┌─────────────┐
     │ causality-  │
     │    types    │
     └─┬─────────┬─┘
       │         │
       │         ▼
       │  ┌──────────┐
       │  │causality-│
       │  │  crypto  │
       │  └────┬─────┘
       │       │
       ▼       ▼
    ┌──────────────┐
    │  causality-  │
    │     core     │
    └──────┬───────┘
           │
         ┌─┴────────────┐
         │              │
┌────────┴─────┐ ┌──────┴──────┐ 
│  causality-  │ │ causality-  │ 
│   storage    │ │     db      │ 
└──────────────┘ └─────────────┘ 
```

## Core Components

The core crate includes:

1. **Time Management**
   - Time abstractions and utilities that standardize temporal operations
   - Logical time tracking for distributed systems
   - Clock synchronization primitives
   - Timestamp verification and validation

2. **Concurrency Primitives**
   - Thread-safe data structures 
   - Task scheduling abstractions
   - Actor model foundations
   - Event notification systems

3. **Verification Framework**
   - Common verification interfaces
   - Proof validation utilities
   - Verification context management
   - Pluggable verification strategies

4. **Actor System**
   - Lightweight actor abstraction
   - Message passing infrastructure
   - Actor supervision strategies
   - Distributed actor coordination

5. **Observation System** (formerly Committee)
   - External chain indexing
   - Fact extraction basics
   - Data provider abstractions
   - Log reconstruction primitives

6. **Error Handling**
   - Unified error types
   - Error context tracking
   - Result extension utilities

7. **Serialization Helpers**
   - Common serialization utilities
   - Schema management
   - Content-addressing support

## Separation of Concerns

The core crate maintains clear boundaries with other crates:

| Crate | Responsibility | Interaction with Core |
|-------|----------------|----------------------|
| `causality-types` | Fundamental type definitions | Core depends on Types |
| `causality-crypto` | Cryptographic operations | Core depends on Crypto |
| `causality-storage` | Storage abstractions and implementations | Uses Core's serialization and time utilities |
| `causality-db` | Database implementations | Uses Core's concurrency primitives |
| `causality-effects` | Effect system | Uses Core's actor system for effect execution |
| `causality-engine` | Execution engine | Leverages Core's verification framework |
| `causality-domain` | Domain-specific logic and adapters | Uses Core's observation primitives but adds domain-specific implementations |
| `causality-resource` | Resource management | Builds on Core's concurrency and verification |

### Observation System vs. Domain-Specific Observation

The Core crate provides the fundamental abstractions for the observation system, but with a clear separation from domain-specific implementations:

- **Core Responsibility**: Provides base abstractions for external system observation, data indexing, and fact extraction
- **Domain Crate Responsibility**: Implements domain-specific observers, adapters, and fact processing logic
- **Resource Crate Responsibility**: Implements resource-specific observation for tracking resource lifecycle events

This separation ensures that:

1. **Core remains domain-agnostic**: The base observation system doesn't depend on specific blockchains or protocols
2. **Domain implementations are pluggable**: Different domain implementations can be added without modifying core
3. **Resource-specific observation is isolated**: Resource lifecycle observation remains in the resource crate

## Design Principles

1. **Minimal Dependencies**: The core crate should depend only on `causality-types` and `causality-crypto`.

2. **Generality**: Core components should be domain-agnostic and widely applicable.

3. **Composability**: Primitives should be designed for composition to enable building complex behaviors.

4. **Performance**: As foundational building blocks, core components must be optimized for performance.

5. **Testability**: All core functionality should be easily testable in isolation.

6. **Clear Boundaries**: Maintain clear separation between core abstractions and domain-specific implementations.

## Future Directions

As the system evolves, we aim to:

1. Further refine the actor model for distributed operation
2. Enhance the verification system with more sophisticated proof types
3. Improve time synchronization capabilities for distributed deployments
4. Complete the transition from committee to observation terminology
5. Implement more specialized concurrency primitives for specific use cases
6. Strengthen the boundaries between core, domain-specific, and resource-specific functionality
