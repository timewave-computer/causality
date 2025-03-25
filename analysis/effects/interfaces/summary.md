# Effect Module Public Interface Catalog

This document catalogs the public interfaces exposed by the causality-effects module.

## Traits

Based on the analysis, there are approximately 24 public traits exposed by the causality-effects module. The key traits include:

```
Effect
EffectContext
EffectExecutionEngine
EffectHandler
EffectHandlerRegistry
EffectOutcome
EffectParameters
ResourceAccess
ResourceDependency
ResourceLifecycle
ResourceLocking
```

## Structs

There are approximately 105 public structs exposed by the causality-effects module. Key structs include:

```
EffectDispatcher
EffectHandlerAdapter
EffectRegistry
EffectResourceImplementation
ResourceAccessManager
ResourceDependencyManager
ResourceLockManager
ResourceLifecycleManager
```

## Enums

There are approximately 28 public enums exposed by the causality-effects module. Key enums include:

```
AccessError
DependencyError
EffectError
EffectExecutionError
EffectRegistrationError
LifecycleError
LockError
ResourceError
```

## Functions

There are approximately 370 public functions exposed by the causality-effects module. This high number indicates significant API surface area that could be reduced.

## Modules

There are approximately 25 public modules in the causality-effects module, including:

```
capability
constraints
domain_effect
resource
templates
```

## Cross-Crate Usage

The effects module is used by other crates in various ways, particularly for handling domain-specific operations and resource management.

## Core Interfaces

Based on analysis, the following interfaces appear to be core to the effects system:

1. **Effect Definition and Execution**
   - `Effect` trait: Core trait defining what an effect is
   - `EffectParameters` trait: Defines parameters for effects
   - `EffectOutcome` trait: Represents the result of an effect execution
   - `EffectContext`: Provides context for effect execution

2. **Effect Handling**
   - `EffectHandler` trait: Defines how to handle an effect
   - `EffectHandlerRegistry`: Registry for effect handlers
   - `EffectExecutionEngine`: Executes effects using appropriate handlers

3. **Resource Management**
   - `ResourceAccess`: Controls access to resources
   - `ResourceLifecycle`: Manages resource lifecycle (creation, destruction)
   - `ResourceLocking`: Handles locking of resources
   - `ResourceDependency`: Manages dependencies between resources

4. **Domain Integration**
   - `domain_effect` module: Connects domain-specific operations with the effect system

These core interfaces form the foundation of the effect system and should be preserved while simplifying the overall codebase. 