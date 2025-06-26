# 900: Glossary of Terms

This glossary provides definitions for key terms used throughout the Causality framework documentation.

---

**Affine Type**
:   A type qualifier indicating that a resource can be used at most once. It can be dropped (not used) without error, but not duplicated or used multiple times.

**Alloc**
:   A fundamental Layer 0 VM instruction (and corresponding Layer 1 Lisp primitive) used to create a new linear resource on the heap, returning its `ResourceId`.

**Capability**
:   A fine-grained permission or property associated with a resource, often managed using row types. Extracting a capability is typically a linear operation.

**Causality**
:   The overarching framework designed for building verifiable, resource-aware distributed applications. It emphasizes linearity, deterministic execution, and a layered architecture.

**Causality Lisp**
:   The functional programming language used at Layer 1 of Causality. It features 11 core primitives and is designed for defining resource transformations and application logic. Programs are represented as `Expr` ASTs.

**Choreography**
:   A global specification of a multi-party communication protocol that describes the interactions between multiple roles. Choreographies compile to individual session types for each participant through endpoint projection.

**Causality Toolkit**
:   The primary Rust crate (`causality-toolkit`) providing APIs, DSLs, and implementations for building applications with Causality.

**CircuitId**
:   A unique identifier for a Zero-Knowledge Proof circuit, typically derived from its SSZ content hash. Used to specify which circuit should be used for proof generation or verification.

**Consume**
:   A fundamental Layer 0 VM instruction (and corresponding Layer 1 Lisp primitive) used to destroy a linear resource, making its underlying value available and invalidating its `ResourceId` for future use.

**Content Addressing**
:   A method of uniquely identifying data (like `Expr`essions, `Effect`s, or ZKP circuits) by the hash of its content, typically using SSZ serialization. This ensures immutability and verifiability.

**Declarative Programming**
:   The paradigm used at Layer 2 of Causality, where developers specify *what* they want to achieve (via `Intent`s and `Effect`s) rather than *how* to achieve it.

**Determinism**
:   A core principle in Causality ensuring that given the same initial state and inputs, operations (especially `Handler`s and VM execution) will always produce the same outputs and state transitions.

**Duality**
:   A fundamental property of session types where two session types are complementary and can safely communicate. For example, a send operation (!T) has a dual receive operation (?T). Duality is automatically computed and verified to ensure deadlock-free communication.

**Domain (`DomainId`)**
:   A logical namespace or context within Layer 2 that groups related `Effect` types, `Handler`s, and governance rules. Identified by a unique `DomainId`.

**Effect (`EffectId`)**
:   A declarative representation of a state change or interaction within a `Domain` at Layer 2. Effects are data structures that describe an operation to be performed. Identified by an `EffectId` (SSZ content hash of the effect definition).

**Expr (`ExprId`)**
:   The Abstract Syntax Tree (AST) representation of a Causality Lisp program at Layer 1. `Expr`s are built from the 11 core Lisp primitives. Identified by an `ExprId` (SSZ content hash).

**Handler**
:   A pure function at Layer 2 responsible for processing an `Effect` (or a set of effects) and transforming it into one or more new `Effect`s, or indicating completion. Handlers embody the specific logic for how effects are actualized.

**Hint**
:   Optional information provided with an `Intent` or `Effect` to guide the Layer 2 orchestration system, potentially influencing `Handler` selection or TEG optimization.

**Intent**
:   A high-level, declarative request submitted by a user or system to Layer 2, expressing a desired outcome or state change. Intents are translated into `Effect`s and orchestrated via a Temporal Effect Graph (TEG).

**Layer 0 (Typed Register Machine)**
:   The foundational execution layer of Causality. It consists of a small, 9-instruction Typed Register Machine that operates directly on resources and values, enforcing linearity at the lowest level.

**Layer 1 (Causality Lisp & Structured Types)**
:   The language and type system layer. It features Causality Lisp for defining application logic and a system of row types and capabilities for fine-grained, compile-time resource management.

**Layer 2 (Declarative Programming & Orchestration)**
:   The highest architectural layer, focusing on declarative programming with `Intent`s, `Effect`s, `Handler`s, and their orchestration via Temporal Effect Graphs (TEGs).

**Linear Type**
:   A type qualifier indicating that a resource must be used exactly once. It cannot be dropped unused, duplicated, or used multiple times.

**Lisp Primitives**
:   The 11 fundamental building blocks of Causality Lisp programs (e.g., `lambda`, `app`, `let`, `alloc`, `consume`).

**Object**
:   A generalization of a `Resource` that can have configurable linearity qualifiers (linear, affine, relevant, unrestricted).

**Protocol**
:   In the context of session types, a protocol defines the sequence of communication operations between parties. Protocols are specified using session type syntax (!T.S for send, ?T.S for receive) and ensure type-safe communication.

**ProofId**
:   A unique identifier for a specific instance of a Zero-Knowledge Proof, typically derived from its SSZ content hash. Links a proof to the statement it verifies.

**Relevant Type**
:   A type qualifier indicating that a resource must be used at least once. It can be used multiple times but cannot be dropped unused.

**Resource (`ResourceId`)**
:   A fundamental concept in Causality representing a piece of data that is subject to linearity rules. Each resource is uniquely identified by a `ResourceId`.

**Row Types**
:   A type system feature used at Layer 1 to represent records or objects with extensible sets of fields (rows). Used for managing capabilities and performing compile-time checks on resource structures.

**Session Channel**
:   A linear resource that represents one endpoint of a session type communication. Session channels are typed with their current protocol state and must be used exactly once according to session type rules.

**Session Type**
:   A type system for describing communication protocols between distributed parties. Session types ensure type safety, deadlock freedom, and protocol compliance through static analysis and automatic duality checking.

**SessionId**
:   A unique identifier for a session type declaration or active session instance. Used to reference and manage session protocols within the system.

**SSZ (SimpleSerialize)**
:   A deterministic serialization standard used throughout Causality for hashing, content addressing, and ensuring consistent data representation across different parts of the system and network.

**Simulation Engine**
:   A tool for testing and debugging Causality applications by simulating the execution of Layers 0, 1, and 2 (especially TEGs) in a controlled, often in-memory, environment.

**State**
:   The collective information held by the Causality system at any point, including the status of all resources, active TEGs, registered Lisp expressions, and effect definitions.

**Temporal Effect Graph (TEG)**
:   A directed acyclic graph (DAG) used at Layer 2 to represent the causal relationships and dependencies between `Effect`s. TEGs orchestrate the execution of effects to fulfill `Intent`s.

**Typed Register Machine (TRM)**
:   The formal name for the Layer 0 execution environment, emphasizing its 5 fundamental instructions and its role in managing typed values and resources.

**Unrestricted Type**
:   A type qualifier indicating that a resource can be used freely: zero or many times (i.e., it can be copied, dropped, or used multiple times).

**Witness (`WitnessId`)**
:   The private input data required to generate a Zero-Knowledge Proof. Identified by a `WitnessId`.

**Zero-Knowledge Proof (ZKP)**
:   A cryptographic technique allowing one party (the prover) to prove to another party (the verifier) that a statement is true, without revealing any information beyond the validity of the statement itself. Used in Causality for privacy-preserving verification of operations or properties.

## Transform-Based Unification

**Transform**
:   The fundamental operation in Causality that unifies computation and communication. All operations are transformations `T: A → B` where location determines whether it's local computation or distributed communication.

**Computation-Communication Symmetry**
:   The fundamental recognition that computation and communication are the same mathematical operation, differing only by their source and target locations. This symmetry eliminates artificial distinctions and enables location transparency.

**Location Transparency**
:   The property that operations work the same whether data is local or remote, with location awareness provided through the type system rather than separate APIs.

**Automatic Protocol Derivation**
:   The process by which communication protocols are automatically generated from data access patterns, eliminating the need for manual protocol specification.

**Unified Constraints**
:   A single constraint language that works for both local field access and distributed communication, enabling seamless composition of local and remote operations.

**Transform Constraint**
:   A constraint in the unified system that can represent local computation, remote communication, data migration, or distributed synchronization using the same mathematical framework.

**Location-Aware Row Types**
:   Row types extended with location information that enable the same field operations to work on both local and remote data.

**Effect<From, To>**
:   The generic effect type where the source and target locations determine the operation type (local computation, remote communication, or data migration).

**Transform Definition**
:   A specification of how a transform operates, unified across function application, communication operations, and resource management.

**Location**
:   A specification of where data or computation resides, including Local, Remote(name), Domain(name), and Distributed variants.

**Data Migration**
:   A transform operation that moves data between locations using automatically derived migration protocols.

**Distributed Synchronization**
:   A transform operation that coordinates state across multiple locations with configurable consistency models.

**Capability Access**
:   A transform operation that includes capability verification as part of the unified constraint system.

**Transform Constraint System**
:   The unified system that resolves all types of constraints (local, remote, migration, synchronization, capability) through a single mathematical framework.

**Location Requirements**
:   Specifications in intents that describe preferred locations, allowed locations, migration strategies, and performance constraints.

**Migration Strategy**
:   A specification of how data should move between locations (copy, move, replicate, partition) with automatic protocol generation.

**Consistency Model**
:   A specification of how distributed operations should be coordinated (strong, eventual, causal, session consistency).

**Protocol Optimization**
:   The automatic batching and optimization of multiple operations into efficient communication protocols.

**Cross-Location Verification**
:   The capability system's ability to verify access permissions across different locations and security domains.

**Session Delegation**
:   The mechanism for delegating capabilities across locations using time-limited session-based protocols.

## Updated Core Terms

**Effect**
:   Now specifically refers to a transform operation with source and target locations. Effects unify computation (Effect<Local, Local>) and communication (Effect<Local, Remote>) under a single mathematical framework.

**Session Type**
:   Communication protocols that are automatically derived from row type operations rather than manually specified. Session types integrate seamlessly with the unified transform system.

**Intent**
:   A declarative specification using unified transform constraints that can describe both local computation and distributed operations through the same constraint language.

**Capability**
:   Access control mechanism extended with location awareness, supporting distributed capabilities and session-based delegation across locations.

**Domain**
:   Now refers to security and capability domains rather than separate computational contexts. Domains can span multiple locations and support cross-location capability verification.
