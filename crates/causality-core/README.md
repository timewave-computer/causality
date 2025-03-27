# Causality Core

Core functionality for the Causality platform. This crate provides fundamental abstractions and implementations for resources, effects, capabilities, and agents.

## Overview

The `causality-core` crate contains the foundational components used across the Causality platform:

- Content addressing for all state objects
- Resource system for managing typed, content-addressed entities
- Effect system for composable operations
- Capability system for access control
- Agent resource system for modeling users, committees, and operators

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

4. **Resource System**
   - Content-addressed resources
   - Resource type registry
   - Resource storage and querying
   - Resource validation

5. **Agent Resource System**
   - User, Committee, and Operator agents as specialized resources
   - Capability-based authorization
   - Agent operations and relationships
   - Service status and messaging
   - Agent resource registry

6. **Observation System** (formerly Committee)
   - External chain indexing
   - Fact extraction basics
   - Data provider abstractions
   - Log reconstruction primitives

7. **Effect System**
   - Core effect definitions
   - Effect registry and handlers
   - Effect context management
   - Effect validation and execution

8. **Error Handling**
   - Unified error types
   - Error context tracking
   - Result extension utilities

9. **Serialization Helpers**
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
| `causality-engine` | Execution engine | Leverages Core's verification framework |
| `causality-domain` | Domain-specific logic and adapters | Uses Core's observation primitives but adds domain-specific implementations |

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

1. Further refine the agent resource system for a complete replacement of the actor model
2. Complete the specialized agent implementations (User, Committee, Operator)
3. Enhance the verification system with more sophisticated proof types
4. Improve time synchronization capabilities for distributed deployments
5. Complete the transition from committee to observation terminology
6. Implement more specialized concurrency primitives for specific use cases
7. Strengthen the boundaries between core, domain-specific, and agent-specific functionality

## Agent Resource System

The Agent Resource System is a unified architecture for modeling agents (users, committees, and operators) as specialized resources. It provides a consistent approach to identity, capabilities, and state management.

### Core Components

- **Agent Resources**: Specialized resources with agent-specific functionality
- **Agent Types**: User, Committee, and Operator agent implementations
- **Operation System**: Capability-checked operations for resource manipulation
- **Service Status**: Service advertisement and discovery
- **Obligation Manager**: Capability obligation tracking
- **Messaging System**: Agent-to-agent communication
- **Capability Bundles**: Predefined capability sets with delegation rules

### Using the Agent System

```rust
// Create a new user agent
let agent = AgentBuilder::new()
    .agent_type(AgentType::User)
    .state(AgentState::Active)
    .with_capability(Capability::new("resource123", "read", None))
    .with_metadata("display_name", "Alice")
    .build()
    .unwrap();

// Register with the agent registry
registry.register_agent(Box::new(agent.clone())).await?;

// Create and execute an operation
let operation = agent.create_operation()
    .target_resource(resource_id)
    .operation_type(OperationType::Read)
    .add_effect(read_effect)
    .build()?;

let result = agent.execute_operation(operation, context).await?;

// Send a message to another agent
let message = message_factory.create_message(
    agent.agent_id().clone(),
    recipient_id,
    MessageType::Request,
    content,
    metadata,
)?;

messaging_system.send_message(message).await?;
```

See the [Agent Resource System Architecture](../../docs/architecture/agent-resource-system.md) document for detailed information.

## Resource System

The Resource System provides a unified approach to content-addressed entities with consistent identity, storage, and access control.

### Core Components

- **Resource Trait**: Common interface for all resources
- **ResourceId**: Content-addressed resource identity
- **ResourceType**: Typed resource categories
- **ResourceStorage**: Content-addressed storage for resources
- **ResourceQuery**: Query capabilities for resource discovery

## Effect System

The Effect System enables composable, type-safe operations with capability verification.

### Core Components

- **Effect Trait**: Common interface for all effects
- **EffectContext**: Context for effect execution with capability checks
- **EffectOutcome**: Structured result of effect execution
- **EffectHandler**: Handler implementations for executing effects
- **EffectOrchestrator**: Coordination of composite effect execution

## Capability System

The Capability System provides fine-grained access control for resources and operations.

### Core Components

- **Capability**: Permission to perform operations on resources
- **CapabilityVerifier**: Validation of capability requirements
- **CapabilityRegistry**: Management of capabilities and constraints
- **CapabilityBundle**: Predefined capability sets with delegation rules
