# System Overview and Architecture

The Causality framework represents a resource-based computational model designed around content-addressed identifiers and functional programming principles. At its core, the system treats all computational entities as Resources that can be transformed through Intents and Effects, providing a unified approach to state management and computation.

## Foundational Concepts

The framework operates on several key abstractions that work together to create a coherent computational model. Resources serve as the fundamental unit of data and computation, representing anything from simple data structures to complex computational processes. Each Resource carries a unique content-addressed identifier, ensuring that identical content produces identical identifiers across the system.

Intents express desired state changes within the system. Rather than imperative commands, Intents describe what transformation should occur, allowing the system to determine the optimal execution strategy. This declarative approach enables sophisticated optimization and reasoning about computational workflows.

Effects represent the actual state changes that occur when Intents are processed. They capture both the transformation logic and the resulting state changes, providing a complete audit trail of system evolution. Effects are scoped by Handlers, which define the execution context and capabilities available during processing.

## Core Type System

The type system centers around several primary structures that encode the framework's computational model. The Resource type contains an EntityId for content addressing, a human-readable name, a DomainId indicating its execution context, a resource type string for categorization, a quantity field for quantifiable resources, and a timestamp marking its creation or last modification.

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

Intent structures describe desired transformations through input and output ResourceFlow specifications. They include priority levels for execution ordering, optional expressions for complex logic, and optimization hints to guide the execution strategy. The framework supports various typed domains, allowing Intents to specify their preferred execution environment.

Effects capture the complete context of a transformation, including the source and target domains, the Handler responsible for execution, and detailed resource flows. They maintain references to the originating Intent and can include cost models and resource usage estimates for optimization purposes.

## Content Addressing System

Content addressing forms the backbone of the framework's identity system. Every entity receives an identifier derived from its content, ensuring that identical data structures produce identical identifiers regardless of when or where they are created. This property enables powerful deduplication, caching, and verification capabilities.

The system uses 32-byte identifiers generated through cryptographic hashing of the entity's serialized content. EntityId, DomainId, NodeId, and other identifier types all follow this pattern, providing type safety while maintaining the underlying content-addressed property.

Nullifiers extend the content addressing system to handle resource consumption. When a Resource is consumed, a Nullifier proves that the consumption occurred without revealing the original Resource content. This mechanism maintains system integrity, while allowing for privacy preserving operations in the future.

## Expression and Computation Model

The framework includes a Lisp-based expression system for defining computational logic. Expressions can represent simple values, complex data structures, function applications, and combinator-based computations. The system supports both traditional Lisp constructs and specialized combinators optimized for resource manipulation.

Atomic combinators provide primitive operations for arithmetic, logic, data structure manipulation, and system interaction. These combinators can be composed into complex expressions that define transformation logic for Intents and Effects. The combinator approach enables both functional programming patterns and efficient execution optimization.

The expression system integrates with the broader framework through ExprId references, allowing complex computational logic to be stored as Resources and referenced from Intents and Effects. This approach enables code reuse, versioning, and sophisticated dependency management.

## Domain and Execution Model

Typed domains provide execution contexts with specific capabilities and constraints. Verifiable domains support zero-knowledge proof generation and deterministic execution, making them suitable for privacy-preserving computations. Service domains allow external API access and non-deterministic operations, enabling integration with existing systems.

Compute domains optimize for computational intensity and parallel execution, providing high-performance environments for data processing workloads. Each domain type offers different trade-offs between performance, verifiability, and integration capabilities.

The framework routes Intents to appropriate domains based on their requirements and optimization hints. This routing enables automatic optimization while maintaining the declarative nature of Intent specification.

## Serialization and Interoperability

All framework types implement Simple Serialize (SSZ) encoding, providing efficient and deterministic serialization. SSZ ensures that identical data structures produce identical serialized representations, supporting the content addressing system and enabling cross-language interoperability.

The serialization system handles complex nested structures, optional fields, and variable-length data while maintaining deterministic output. This property is crucial for content addressing and enables verification of data integrity across system boundaries.

## Current Implementation Status

The framework currently provides complete type definitions with SSZ serialization support, a functional Lisp interpreter with combinator support, comprehensive testing utilities through the causality-toolkit crate, and content-addressed identifier generation and verification.

The OCaml integration through ml_causality offers equivalent type definitions and expression handling, enabling functional programming approaches to framework usage. The Nix-based development environment ensures reproducible builds and consistent tooling across development machines.

Runtime execution capabilities remain under development, with the current focus on establishing solid foundations for type safety, serialization, and expression evaluation. The architecture supports future expansion into distributed execution, advanced optimization strategies, and sophisticated domain-specific capabilities. 