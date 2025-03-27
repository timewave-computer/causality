# Causality Effect System

The Effect System is a core component of the Causality platform, providing a unified approach to modeling and executing side effects in a controlled, content-addressed manner. This document outlines the key components and recent enhancements to the Effect System.

## Core Components

### Effect Interface

The foundation of the system is the `Effect` trait, which defines a unit of work that can be executed to produce a side effect:

```rust
pub trait Effect: Send + Sync {
    fn id(&self) -> &EffectId;
    fn type_id(&self) -> EffectTypeId;
    fn boundary(&self) -> ExecutionBoundary;
    fn name(&self) -> String;
    fn is_valid(&self) -> bool;
    fn dependencies(&self) -> Vec<EffectId>;
    fn modifications(&self) -> Vec<String>;
    fn clone_effect(&self) -> Box<dyn Effect>;
    fn as_any(&self) -> &dyn std::any::Any;
}
```

All effects in the system implement this trait, providing a consistent interface for handling side effects regardless of their specific implementation details.

### Effect Context

The `EffectContext` manages capabilities, resources, and metadata for effect execution:

```rust
pub trait EffectContext: Send + Sync + Clone {
    fn capabilities(&self) -> &[Capability];
    fn has_capability(&self, capability: &Capability) -> bool;
    fn resources(&self) -> &HashSet<ResourceId>;
    fn has_resource(&self, resource: &ResourceId) -> bool;
    fn metadata(&self) -> &HashMap<String, String>;
    fn with_additional_capabilities(&self, capabilities: Vec<Capability>) -> Box<dyn EffectContext>;
    fn with_additional_resources(&self, resources: HashSet<ResourceId>) -> Box<dyn EffectContext>;
    fn with_additional_metadata(&self, metadata: HashMap<String, String>) -> Box<dyn EffectContext>;
    fn clone_context(&self) -> Box<dyn EffectContext>;
}
```

The context ensures proper capability checking and resource management during effect execution.

### Effect Registry

The `EffectRegistry` manages effect handlers and orchestrates effect execution:

```rust
pub trait EffectRegistry: Send + Sync + Debug {
    fn register_handler(&mut self, handler: Arc<dyn EffectHandler>) -> EffectRegistryResult<()>;
    fn get_handler(&self, effect_type_id: &EffectTypeId) -> EffectRegistryResult<Arc<dyn EffectHandler>>;
    fn execute_effect(&self, effect: &dyn Effect, context: &dyn EffectContext) -> EffectResult<EffectOutcome>;
    fn register_domain_handler(&mut self, handler: Arc<dyn DomainEffectHandler>) -> EffectRegistryResult<()>;
    fn execute_domain_effect(&self, effect: &dyn DomainEffect, context: &dyn EffectContext) -> EffectResult<EffectOutcome>;
    fn clone_registry(&self) -> Arc<dyn EffectRegistry>;
}
```

The registry routes effects to the appropriate handlers and executes them with the provided context.

## Recent Enhancements

### ADR12.1: Core Effect Registry Implementation
- Created `EffectTypeId` with content addressing in `types.rs`
- Implemented `EffectRegistry` trait with handler registration in `registry.rs`
- Added selector pattern for effect type resolution

### ADR12.2: Effect Context Enhancement
- Added resource capability checking to context
- Implemented context propagation mechanisms
- Added context metadata tracking for execution boundaries

### ADR12.3: Resource Effect Interface Extension
- Created comprehensive `ResourceEffect` trait in `resource.rs`
- Implemented standard resource operations (Create, Read, Update, Delete, Transfer)
- Added resource capability verification
- Implemented basic resource effect handler

### ADR12.4: Domain Effect Framework
- Enhanced `DomainEffect` trait with domain identification
- Created domain-specific context adaptations
- Implemented domain parameter validation
- Created domain capability mapping

### ADR12.5: Effect Orchestration System
- Implemented orchestration status tracking
- Created orchestration plan with steps and dependencies
- Built basic orchestrator for executing effect sequences
- Added cross-domain orchestration support

### ADR12.6: Effect Storage System
- Created effect storage interface for persistence
- Implemented content-addressed effect repository
- Added serialization/deserialization support for effects
- Implemented history tracking for effect execution

## Content Addressing

All components in the Effect System fully leverage content addressing principles:

1. Every effect has a unique content-addressed `EffectId`
2. Effect types are identified by content-addressed `EffectTypeId`
3. Effect outcomes and execution records are content-addressed
4. Effect storage is content-addressed for immutability and verification

## Integration With Resources

Effects operate on resources through capability-checked operations:

1. Effects declare the resources they require access to
2. The context enforces capability checks for resource access
3. Resource effects provide a standard interface for resource operations
4. Cross-domain operations use domain-specific adapters

## Execution Model

The Effect System uses a three-layer architecture for execution:

1. **Effect Definition Layer**: Effects declare what they do
2. **Constraint Layer**: Capabilities and resources control what can be done
3. **Execution Layer**: Handlers implement how effects are executed

## Current Status

The Effect System now provides:

- ✅ Comprehensive content addressing support
- ✅ Domain-specific effect handling
- ✅ Resource integration
- ✅ Cross-domain orchestration
- ✅ Persistence and history tracking
- ✅ Capability validation
- ✅ Metadata propagation

## Future Directions

Planned enhancements include:

- Resource Type Registry
- Cross-Domain Resource Protocol
- Domain Integration Layer
- Resource Storage Implementation
- Consensus Integration
- Cross-Domain Resource Reference Protocol 