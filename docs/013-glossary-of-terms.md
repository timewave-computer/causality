# 013: Glossary of Terms

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

**ProofId**
:   A unique identifier for a specific instance of a Zero-Knowledge Proof, typically derived from its SSZ content hash. Links a proof to the statement it verifies.

**Relevant Type**
:   A type qualifier indicating that a resource must be used at least once. It can be used multiple times but cannot be dropped unused.

**Resource (`ResourceId`)**
:   A fundamental concept in Causality representing a piece of data that is subject to linearity rules. Each resource is uniquely identified by a `ResourceId`.

**Row Types**
:   A type system feature used at Layer 1 to represent records or objects with extensible sets of fields (rows). Used for managing capabilities and performing compile-time checks on resource structures.

**SSZ (SimpleSerialize)**
:   A deterministic serialization standard used throughout Causality for hashing, content addressing, and ensuring consistent data representation across different parts of the system and network.

**Simulation Engine**
:   A tool for testing and debugging Causality applications by simulating the execution of Layers 0, 1, and 2 (especially TEGs) in a controlled, often in-memory, environment.

**State**
:   The collective information held by the Causality system at any point, including the status of all resources, active TEGs, registered Lisp expressions, and effect definitions.

**Temporal Effect Graph (TEG)**
:   A directed acyclic graph (DAG) used at Layer 2 to represent the causal relationships and dependencies between `Effect`s. TEGs orchestrate the execution of effects to fulfill `Intent`s.

**Typed Register Machine (TRM)**
:   The formal name for the Layer 0 execution environment, emphasizing its 11 core instructions and its role in managing typed values and resources.

**Unrestricted Type**
:   A type qualifier indicating that a resource can be used freely: zero or many times (i.e., it can be copied, dropped, or used multiple times).

**Witness (`WitnessId`)**
:   The private input data required to generate a Zero-Knowledge Proof. Identified by a `WitnessId`.

**Zero-Knowledge Proof (ZKP)**
:   A cryptographic technique allowing one party (the prover) to prove to another party (the verifier) that a statement is true, without revealing any information beyond the validity of the statement itself. Used in Causality for privacy-preserving verification of operations or properties.
