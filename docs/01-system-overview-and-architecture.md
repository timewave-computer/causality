# System Overview and Architecture

The Causality framework represents a resource-based computational model designed around content-addressed identifiers and functional programming principles. At its core, the system treats all computational entities as Resources that can be transformed through Intents and Effects, providing a unified approach to state management and computation.

## Foundational Concepts

The framework operates on several key abstractions that work together to create a coherent computational model. Resources serve as the fundamental unit of data and computation. Each Resource uniquely binds its state (data, represented by a `ValueExpr` and identified by a `ValueExprId`) to the specific logic (`Expr`, identified by an `ExprId`) that governs its behavior and transformations, all under a single, content-addressed `ResourceId`.

Intents express desired state changes within the system. Rather than imperative commands, Intents describe what transformation should occur, allowing the system to determine the optimal execution strategy. This declarative approach enables sophisticated optimization and reasoning about computational workflows.

Effects represent the actual state changes that occur when Intents are processed. They capture both the transformation logic and the resulting state changes, providing a complete audit trail of system evolution. Effects are scoped by Handlers, which define the execution context and capabilities available during processing. Importantly, core system operations and entities like Effects and Handlers are often themselves modeled as Resources, adhering to the same principles of content-addressing and verifiable logic. For complex, multi-step processes, the system can utilize `ProcessDataflowBlock`s, which are declarative Lisp S-expressions defining sequences of operations, orchestrated by Handlers.

## Core Type System

The type system centers around the Resource as its core primitive. A Resource is a content-addressed entity that encapsulates:
-   Its unique `id` (`EntityId`), which serves as a content-addressed identifier ensuring that Resources with identical content receive identical identifiers.
-   A human-readable `name` (`Str`) for semantic meaning, which does not affect the content-addressed identifier.
-   A `domain_id` (`DomainId`) that identifies the `TypedDomain` to which the Resource belongs, dictating its execution context.
-   A `resource_type` (`Str`) that categorizes the Resource (e.g., "token", "data_object").
-   A `quantity` (`u64`) for quantifiable resources.
-   A `timestamp` (`Timestamp`) marking its creation or last modification, primarily for temporal ordering.

All concrete data and state held within Resources are represented by `ValueExpr` instances. These provide a consistent and canonical way to structure information. `ValueExpr`s are serialized using SSZ (Simple Serialize), and the cryptographic hash of this serialized form (its Merkle root) yields a `ValueExprId`, ensuring data is uniquely referenced and verifiable by its content. This system is designed to be ZK-friendly, for example, by using specific integer types and avoiding non-deterministic types like floating-point numbers.

Behavior, validation rules, and transformation logic are defined by `Expr` instances (Expressions). `Expr`s are Lisp Abstract Syntax Trees (ASTs) representing executable logic. This "code-as-data" approach means logic itself can be treated as data. Like `ValueExpr`s, `Expr` ASTs are SSZ-serialized, and their Merkle root produces an `ExprId`.

Intent structures describe desired transformations through input and output ResourceFlow specifications. They include priority levels for execution ordering, optional expressions for complex logic, and optimization hints to guide the execution strategy. The framework supports various typed domains, allowing Intents to specify their preferred execution environment.

Effects capture the complete context of a transformation, including the source and target domains, the Handler responsible for execution, and detailed resource flows. They maintain references to the originating Intent and can include cost models and resource usage estimates for optimization purposes.

## Content Addressing System

Content addressing forms the backbone of the framework's identity and verifiability. Every critical entity—`Resource`, `ValueExpr`, and `Expr`—receives an identifier (`ResourceId`, `ValueExprId`, and `ExprId` respectively) that is the Merkle root of its canonical SSZ (Simple Serialize) representation. This ensures that identical content produces identical identifiers and allows for robust deduplication, caching, and verification across the system.

To provide authenticated and verifiable storage for these SSZ-identified entities, the system employs Sparse Merkle Trees (SMTs). Each entity's SSZ Merkle root (its ID) serves as its unique key within a global SMT, which maps this key to the entity's serialized data or relevant metadata. SMTs are authenticated data structures, meaning any data stored within them comes with a cryptographic proof (a Merkle proof) of its inclusion (or non-inclusion) relative to the SMT's root hash. This is crucial for data integrity, partial state disclosure, and ZK verification, as ZK circuits can operate on compact SMT proofs and SSZ roots instead of entire data structures.

Nullifiers extend the content addressing system to handle resource consumption. When a Resource is consumed, a Nullifier proves that the consumption occurred without revealing the original Resource content. This mechanism maintains system integrity, while allowing for privacy preserving operations in the future.

## Expression and Computation Model

The framework includes a Lisp-based expression system for defining computational logic. `Expr` instances (Lisp ASTs, identified by their `ExprId`) define the behavior, validation rules, and transformation logic associated with Resources or system operations.

A Lisp-flavored interpreter is responsible for evaluating these `Expr`s. To ensure deterministic and consistent evaluation across the system and over time, the system can use an `InterpreterId`, representing a commitment to a specific version or configuration of the interpreter (including its set of atomic operations/combinators and their semantics). The interpreter takes an `Expr` and an evaluation context (which might include a Resource's `ValueExpr` state) and produces a result, typically another `ValueExpr`.

The system favors a combinator-based approach within `Expr`s. Atomic combinators (pre-defined host functions with fixed semantics) provide primitive operations for arithmetic, logic, data structure manipulation, and system interaction. These are composed to build complex logic, promoting determinism and verifiability.

The expression system integrates with the broader framework through `ExprId` references, allowing complex computational logic to be stored as Resources (or associated with them) and referenced from Intents and Effects. This approach enables code reuse, versioning, and sophisticated dependency management.

## Domain and Execution Model

Typed Domains provide execution contexts with specific capabilities, constraints, and trust assumptions, identified by `DomainId`s. A Resource's `primary_domain_id` indicates its native environment and dictates its fundamental execution characteristics (e.g., whether its state transitions are ZK-provable). Its `contextual_scope_domain_ids` can grant its logic visibility or interaction capabilities with other specified domains.

Two primary types of domains are:
-   `VerifiableDomain`: Represents an environment where state transitions are expected to be ZK-provable. `Expr` logic for Resources primarily homed on a `VerifiableDomain` is designed for ZK circuits (deterministic, bounded, etc.).
-   `ServiceDomain`: Represents an interface to an external service or a set of off-chain operations that are not directly ZK-proven (e.g., RPC calls to chain nodes, interactions with third-party APIs). A Resource homed on a `ServiceDomain` has `Expr` logic defining how to interact with that service.

This distinction allows the system to clearly delineate between operations that are part of the core ZK-verified state machine and those that are interfaces to the outside world, while still modeling all interactions within the unified Resource framework.

The framework routes Intents to appropriate domains based on their requirements and optimization hints. This routing enables automatic optimization while maintaining the declarative nature of Intent specification.

## Serialization and Interoperability

All framework types implement Simple Serialize (SSZ) encoding, providing efficient and deterministic serialization. SSZ ensures that identical data structures produce identical serialized representations, supporting the content addressing system (via Merkle roots of SSZ data) and enabling cross-language interoperability.
