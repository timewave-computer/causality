# Resource Model and Core Types

The Resource model forms the conceptual foundation of the Causality framework, providing a unified abstraction for representing data, computational processes, and system state. This model treats all entities within the system as Resources that can be created, transformed, and consumed through well-defined operations.

## Resource Structure and Properties

Resources in the Causality framework encapsulate both data and metadata necessary for content addressing and system operation. Each Resource contains an EntityId that serves as its content-addressed identifier, ensuring that Resources with identical content receive identical identifiers regardless of their creation context.

The Resource structure includes a human-readable name field that provides semantic meaning without affecting the content-addressed identifier. This separation allows for meaningful labeling while preserving the mathematical properties of content addressing.

```rust
pub struct Resource {
    pub id: EntityId,
    pub name: Str,
    pub domain_id: DomainId,
    pub resource_type: Str,
    pub quantity: u64,
    pub timestamp: Timestamp,
}
```

Domain identification through the domain_id field establishes the execution context for Resource operations. Different domains provide varying capabilities, from verifiable computation environments to service integration contexts. The resource_type field enables categorization and type-based operations, while the quantity field supports quantifiable resources like tokens or computational credits.

Timestamps provide temporal ordering and versioning capabilities, though they do not affect the content-addressed identifier. This design allows for temporal reasoning while maintaining the deterministic properties essential for content addressing.

## Intent Model and Transformation Logic

Intents represent desired state transformations within the system, expressing what should happen rather than how it should be accomplished. This declarative approach enables the framework to optimize execution strategies while maintaining clear separation between specification and implementation.

The Intent structure captures transformation requirements through input and output ResourceFlow specifications. These flows describe the types and quantities of Resources required for the transformation and the expected outputs. The framework uses this information for validation, optimization, and resource allocation.

```rust
pub struct Intent {
    pub id: EntityId,
    pub name: Str,
    pub domain_id: DomainId,
    pub priority: u32,
    pub inputs: Vec<ResourceFlow>,
    pub outputs: Vec<ResourceFlow>,
    pub expression: Option<ExprId>,
    pub timestamp: Timestamp,
    pub optimization_hint: Option<ExprId>,
    pub target_typed_domain: Option<TypedDomain>,
    pub process_dataflow_hint: Option<ProcessDataflowInitiationHint>,
}
```

Priority levels enable ordering of Intent execution when multiple Intents compete for resources or execution capacity. The optional expression field allows complex transformation logic to be specified through the framework's Lisp-based expression system.

Optimization hints provide guidance to the execution engine about preferred strategies or constraints. These hints can influence domain selection, resource allocation, and execution ordering without mandating specific implementation approaches.

## Effect Model and State Changes

Effects represent the actual state changes that occur when Intents are processed. They capture both the transformation logic and the resulting system state modifications, providing a complete record of system evolution.

The Effect structure includes comprehensive information about the transformation context, including source and target domains, the Handler responsible for execution, and detailed resource flows. This information enables auditing, debugging, and system analysis.

```rust
pub struct Effect {
    pub id: EntityId,
    pub name: Str,
    pub domain_id: DomainId,
    pub effect_type: Str,
    pub inputs: Vec<ResourceFlow>,
    pub outputs: Vec<ResourceFlow>,
    pub expression: Option<ExprId>,
    pub timestamp: Timestamp,
    pub resources: Vec<ResourceFlow>,
    pub nullifiers: Vec<ResourceFlow>,
    pub scoped_by: HandlerId,
    pub intent_id: Option<ExprId>,
    pub source_typed_domain: TypedDomain,
    pub target_typed_domain: TypedDomain,
    pub cost_model: Option<EffectCostModel>,
    pub resource_usage_estimate: Option<ResourceUsageEstimate>,
    pub originating_dataflow_instance: Option<ProcessDataflowInstanceId>,
}
```

Nullifiers within Effects prove that specific Resources were consumed during the transformation. This mechanism enables privacy-preserving resource management by proving consumption without revealing the consumed Resource content.

Cost models and resource usage estimates support optimization and resource planning. These fields enable the framework to make informed decisions about execution strategies and resource allocation.

## Resource Flow and Transformation Patterns

ResourceFlow structures describe the movement of Resources through the system during transformations. They specify resource types, quantities, and domain contexts, enabling precise specification of transformation requirements and outputs.

```rust
pub struct ResourceFlow {
    pub resource_type: Str,
    pub quantity: u64,
    pub domain_id: DomainId,
}
```

Resource flows enable the framework to validate that Intents have sufficient inputs and that Effects produce the expected outputs. This validation occurs at both the type level and the quantity level, ensuring system consistency.

The domain context within resource flows enables cross-domain transformations, where Resources move between different execution environments. This capability supports complex workflows that span multiple computational contexts.

## Handler Model and Execution Context

Handlers define the execution context and capabilities available during Effect processing. They encapsulate the logic necessary to transform Intents into Effects, providing the bridge between declarative specifications and actual computation.

```rust
pub struct Handler {
    pub id: HandlerId,
    pub name: Str,
    pub domain_id: DomainId,
    pub handler_type: Str,
    pub priority: u32,
    pub expression: Option<ExprId>,
    pub timestamp: Timestamp,
}
```

Handler types enable specialization for different kinds of transformations. Some handlers might focus on data transformation, others on external service integration, and still others on complex computational workflows.

Priority levels among handlers enable conflict resolution when multiple handlers could process the same Intent. The framework can select the most appropriate handler based on priority, capabilities, and current system state.

## Transaction Model and Atomicity

Transactions group multiple Effects into atomic units, ensuring that either all Effects in a transaction succeed or none do. This mechanism provides consistency guarantees for complex multi-step operations.

```rust
pub struct Transaction {
    pub id: EntityId,
    pub name: Str,
    pub domain_id: DomainId,
    pub effects: Vec<EntityId>,
    pub timestamp: Timestamp,
}
```

Transaction boundaries enable rollback and recovery mechanisms, ensuring system consistency even in the presence of failures. The framework can use transaction information to implement sophisticated error handling and recovery strategies.

## Type Safety and Validation

The framework's type system provides compile-time and runtime validation of Resource operations. Type constraints ensure that only valid transformations can be specified and executed, reducing the likelihood of runtime errors.

Trait implementations provide common interfaces for Resource manipulation, enabling generic algorithms while maintaining type safety. The AsResource trait, for example, enables uniform handling of Resource-like entities regardless of their specific implementation.

```rust
pub trait AsResource {
    fn resource_type(&self) -> &Str;
    fn quantity(&self) -> u64;
    fn matches_pattern(&self, pattern: &ResourcePattern) -> bool;
}
```

Pattern matching capabilities enable sophisticated Resource selection and filtering operations. ResourcePattern structures can specify constraints on resource types, domains, quantities, and other properties.

## Serialization and Content Addressing

All core types implement Simple Serialize (SSZ) encoding, ensuring deterministic serialization that supports content addressing. The serialization format handles complex nested structures while maintaining the property that identical data produces identical serialized output.

Content addressing relies on this deterministic serialization to generate consistent identifiers. The framework computes identifiers by hashing the SSZ-encoded representation of the entity, ensuring that content changes result in identifier changes.

This approach enables powerful deduplication, caching, and verification capabilities. Identical Resources can be detected and shared across the system, reducing storage requirements and enabling efficient data management.

## Integration with Expression System

The core types integrate seamlessly with the framework's expression system through ExprId references. Complex transformation logic can be stored as expressions and referenced from Intents, Effects, and Handlers.

This integration enables code reuse, versioning, and sophisticated dependency management. Expressions can be treated as first-class Resources, enabling meta-programming and dynamic system behavior.

The expression system supports both functional programming patterns and imperative-style operations, providing flexibility in how transformations are specified and implemented. 