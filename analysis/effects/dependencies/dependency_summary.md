# Effect Module Dependency Analysis

This document summarizes the dependencies between the causality-effects module and other crates in the system.

## Dependencies OF causality-effects

The effects module imports from the following crates:

- **causality-domain**: Most significant dependency with many imports for domain-specific functionality
- **causality-resource**: Significant dependency for resource management functionality
- **causality-common**: Minor dependency for common utilities
- **causality-tel**: Limited or no direct dependencies

Key imports from other crates:
```
From causality-domain:
- DomainAdapter
- DomainRegistry
- DomainId
- Various domain-specific types (EVM, CosmWasm, etc.)

From causality-resource:
- ResourceAccess
- ResourceLifecycle
- ResourceLocking
- ResourceDependency
```

## Dependencies ON causality-effects

The following crates likely import from the effects module:

- **causality-domain**: For effect handler implementations
- **causality-tel**: For TEL-based effect execution
- Various domain-specific crates: For implementing domain-specific effects

## Effect Implementations Outside the Effects Crate

There appear to be Effect trait implementations outside the effects crate, particularly in domain-specific crates.

## Effect Handler Implementations Outside the Effects Crate

Domain adapters likely implement the EffectHandler trait to enable domain-specific effect handling.

## Resource Trait Implementations Outside the Effects Crate

The Resource traits (ResourceAccess, ResourceLifecycle, etc.) are likely implemented by:
- Domain adapters
- Resource managers in the causality-resource crate

## Key Integration Points

Based on the analysis, the following represent key integration points between the effects module and the rest of the system:

1. **Effect Definition and Execution**
   - The `Effect` trait interface must be preserved for external implementations
   - The `EffectOutcome` type must maintain compatibility with consumers

2. **Effect Handler System** 
   - The `EffectHandler` trait must maintain compatibility with domain adapters
   - The handler registration mechanism must support external handlers

3. **Resource System Integration**
   - Resource trait interfaces must be compatible with external implementations
   - Resource managers must work with both effect and non-effect consumers

4. **Domain-Specific Adapters**
   - Domain-specific effect definitions must preserve interfaces
   - Domain adapter integration must be maintained

## Dependency Reduction Opportunities

1. **Resource Management**
   - Reduce duplication between effect-specific and general resource management
   - Create cleaner abstractions for resource capabilities

2. **Domain Integration**
   - Simplify the domain adapter to effect handler bridge
   - Reduce boilerplate in domain-specific effect implementations

3. **Utility Functions**
   - Move generic utilities to common module
   - Remove redundant helper functions

These integration points should be carefully preserved or provided with clear migration paths during the refactoring process. 