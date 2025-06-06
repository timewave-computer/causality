# Causality

A programming environment for verifiable distributed programs using a linear resource model.

![](./causality.png)

## Architecture

Causality is built on linear resource programming: every resource is consumed exactly once, creating explicit causal ordering and eliminating entire classes of errors (double-spends, use-after-free, race conditions) by design. Resources are content-addressed through cryptographic hashing, enabling global deduplication, verifiable references, and natural distribution.

The system employs a mathematically grounded three-layer architecture where each layer has precise categorical foundations:

**Layer 0: Register Machine** - Minimal execution substrate with 11 instructions (`move`, `apply`, `alloc`, `consume`, `match`, `select`, `witness`, `check`, `perform`, `labelmarker`, `return`) operating on a linear resource heap. Designed for deterministic execution and efficient zero-knowledge circuit generation.

**Layer 1: Linear Lambda Calculus** - Pure functional programming with 11 primitives implementing Symmetric Monoidal Closed Category semantics. Provides unit types, tensor products, sum types, linear functions, and resource management. All operations compile to fixed-size ZK circuits.

**Layer 2: Effect Algebra** - Declarative programming through algebraic effects with capability-based access control. Effects separate interface from implementation, enabling cross-domain interoperability. Includes comprehensive record operations, object linearity, and intent-based orchestration.

## Core Principles

**Linear & Immutable**: Resources consumed exactly once, transformations create new instances, ensuring predictable state updates and resource safety.

**Self-describing**: Data, code, and effects treated uniformly as content-addressed resources, enabling consistent composition through algebraic effects and verifiable global state.

**Verifiable**: Static analysis ensures type safety while runtime privacy and integrity maintained through efficient zero-knowledge verification.

**Declarative & Composable**: Algebraic effects decouple interface from implementation, enabling cross-domain interoperability through direct-style effect composition.

## Resource Model

Resources are content-addressed entities identified by the SSZ hash of their canonical representation. A Resource binds:
- **Identity**: Content hash serving as global identifier
- **Value**: SSZ-serialized data with deterministic encoding
- **Logic**: Optional validation expressions for verification
- **Capabilities**: Access control tokens for field-level permissions
- **Domains**: Execution contexts enabling cross-domain operations

This unified model creates a recursive "code-as-data" architecture where resources can represent data, effects, handlers, capabilities, and system operations themselves.

## Algebraic Effects

Effects are pure data structures describing operations to be performed, separate from their implementation. This separation enables:

- **Composability**: Effects form a monad with well-defined composition laws
- **Polymorphism**: Same effect interface handled differently across domains  
- **Testability**: Effects can be mocked or simulated for testing
- **Verifiability**: Effect execution produces verifiable traces

Capability-based access control ensures fine-grained, unforgeable permissions over resources and their fields, with capabilities forming an algebraic structure supporting intersection, union, and implication operations.

## Zero-Knowledge Integration

The entire architecture is designed ZK-first:
- **Static Structure**: All data layouts determined at compile time
- **Fixed Circuits**: Compilation produces bounded, deterministic circuits
- **Content Addressing**: Enables efficient proof verification and composition
- **SSZ Merkleization**: Natural tree structure for selective disclosure

## Language Support

**Causality Lisp**: Functional language with 11 core primitives mapping directly to Layer 1 operations. Supports linear types, row polymorphism, and capability annotations.

**Rust DSL**: Native Rust integration through traits and macros for effect definition and handler implementation.

**OCaml DSL**: Functional DSL leveraging OCaml's type system for constructing well-formed expressions with S-expression interop.

## Crates

- `causality-core`: Layer 0 register machine, Layer 1 linear type system, content addressing
- `causality-compiler`: Three-layer compilation pipeline from Lisp to register instructions  
- `causality-lisp`: Linear lambda calculus interpreter with capability tracking
- `causality-runtime`: Resource lifecycle management and effect execution
- `causality-simulation`: Branching simulation engine with time-travel and optimization
- `causality-zk`: Zero-knowledge proof generation from execution traces
- `causality-api`: Integration traits for external systems and domains
- `causality-toolkit`: Standard library of effects, resources, and utilities

## Environment & Build

Uses Nix with Flakes for reproducible development. Enter with `nix develop`, build with `cargo build --all`, test with `cargo test --all`.
