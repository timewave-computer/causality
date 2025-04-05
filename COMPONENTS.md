# Core Components

This document provides an overview of the key components we've implemented or fixed in the Causality project.

## Core Components

### Content Addressing

We've standardized the content addressing system across all modules:

- `ContentId` from `causality_types` is used universally for all content addresses
- All references to `ContentHash` have been standardized to work with the crypto module
- Content-addressable structs use a consistent serialization pattern for generating digests

### Time System

The time system provides:

- `TimeMap` for tracking timing-related information with a simple interface
- Timestamps are tracked in a standardized format
- Provider-based implementation allowing for simulations and real-time tracking
- Domain-specific time tracking for causal consistency

### Effect System

The effect system allows for algebraic effects:

- `Effect` trait defines the basic effect interface
- `EffectHandler` trait for implementing effect handlers
- `EffectRegistry` for registering and looking up handlers
- Content-addressable effects for deterministic execution

### Fact System

The fact system provides causal tracking:

- `FactId` for uniquely identifying facts
- `FactSnapshot` for capturing the state at specific points
- `FactDependency` for tracking causal relationships
- `FactEffectTracker` for monitoring causal relationships between facts and effects

### Invocation System

The invocation system provides:

- `InvocationPattern` for defining different ways to invoke handlers
- `InvocationContext` for tracking execution context
- `ContextPropagator` for passing context between invocations
- Multiple invocation patterns (Direct, Callback, Continuation, etc.)

## Engine Components

### Operation Management

The operation management system:

- `Operation` type representing an atomic action
- `AbstractExecutor` for executing operations in different contexts
- `RegisterExecutor` for register-based operations
- `ZkExecutor` for zero-knowledge operations

### Resource Management

The resource management system:

- `ResourceRegisterTrait` for managing register-based resources
- Factory functions for creating standard effects (transfer, deposit, withdrawal)
- Content-addressable resource references

### Execution System

The execution system provides:

- `ContentAddressableExecutor` trait for code execution by content hash
- `ExecutionContext` for tracking execution state
- `ExecutionValue` for typed values during execution
- `ExecutionEvent` for tracking events during execution

## Core Interfaces

We've standardized several critical interfaces:

1. **LogStorage**: Interface for storing execution logs
2. **FactSnapshot**: Interface for capturing causal snapshots
3. **TimeProvider**: Interface for time-related services
4. **Effect**: Core interface for all effects
5. **ContextStorage**: Interface for storing and retrieving execution contexts

## Next Steps

The following components still need work:

1. TimeMap implementation in core package
2. Full implementation of missing modules in engine package
3. Invocation system integration with effect system
4. Testing framework updates to use new interfaces 