# Causality Types

Core type definitions and trait interfaces for the Causality Resource Model framework. This crate contains **only type definitions and trait interfaces** with zero implementation code, serving as the foundation for the entire Causality system.

## Overview

The Causality system is built around the **Resource** primitive - content-addressed entities that uniquely bind their state (data) to the specific logic that governs their behavior and transformations. This crate defines the fundamental types that enable deterministic and verifiable computation across the system.

## Design Principles

1. **Types Only**: Contains only type definitions and trait interfaces with zero implementation code
2. **Content-Addressed**: All critical entities are identified by the Merkle root of their SSZ-serialized form
3. **SSZ Serialization**: Exclusively uses SSZ (Simple Serialize) for deterministic, ZK-compatible serialization
4. **Resource-Centric**: Everything is modeled as or relates to Resources with verifiable state and logic
5. **Domain-Aware**: Supports typed domains (VerifiableDomain, ServiceDomain) for different execution contexts
6. **Immutable**: All state objects are immutable and content-addressed
7. **Single-Use**: Resources can only be consumed exactly once (enforced through nullifiers)

## Core Architecture

### Resources (`Resource`)

Resources are the fundamental building blocks of the system. Each Resource comprises:

- **`id` (`ResourceId`)**: Content-addressed identifier (Merkle root of SSZ-serialized Resource)
- **`value` (`ValueExprId`)**: Points to a `ValueExpr` representing the Resource's state/data
- **`static_expr` (`Option<ExprId>`)**: Optional logic for off-chain validation and static constraints
- **`primary_domain_id` (`DomainId`)**: Primary domain determining execution characteristics
- **`contextual_scope_domain_ids` (`Vec<DomainId>`)**: Additional domains the Resource can interact with
- **`ephemeral` (bool)**: Whether the Resource is temporary or persistent

For specialized Resources (Effects, Handlers), their behavioral logic is typically embedded within their `value` field as `ExprId` references.

### Value Expressions (`ValueExpr`)

All concrete data and state within Resources are represented by `ValueExpr` instances:

- **`Unit`**: Represents nil or empty value
- **`Bool(bool)`**: Boolean values
- **`String(Str)`**: Textual data using specialized `Str` type
- **`Number(Number)`**: Numeric data (primarily `Integer(i64)` for determinism)
- **`List(ValueExprVec)`**: Ordered sequences of `ValueExpr`s
- **`Map(ValueExprMap)`**: Key-value stores with `Str` keys
- **`Record(ValueExprMap)`**: Struct-like maps for structured data
- **`Ref(ValueExprRef)`**: References to other `ValueExprId`s or `ExprId`s
- **`Lambda`**: First-class function values with captured environments

All `ValueExpr` instances are SSZ-serialized and identified by their `ValueExprId` (Merkle root).

### Expressions (`Expr`)

Behavior, validation rules, and transformation logic are defined by `Expr` ASTs:

- **`Atom(atom::Atom)`**: Literal atomic values
- **`Const(ValueExpr)`**: Embedded constant values
- **`Var(Str)`**: Named variables resolved in evaluation context
- **`Lambda(Vec<Str>, Box<Expr>)`**: Anonymous functions
- **`Apply(Box<Expr>, Vec<Expr>)`**: Function applications
- **`Combinator(AtomicCombinator)`**: Predefined atomic operations
- **`Dynamic(u32, Box<Expr>)`**: Step-bounded evaluation for ZK circuits

`Expr` ASTs are SSZ-serialized and identified by their `ExprId` (Merkle root).

### Typed Domains

The system supports different execution environments through typed domains:

- **`VerifiableDomain`**: Environments where state transitions are ZK-provable
- **`ServiceDomain`**: Interfaces to external services (RPC, API integrations)
  - `RpcServiceDomain`: For blockchain RPC calls
  - `ApiIntegrationDomain`: For third-party API interactions

Resources' `primary_domain_id` determines their execution context and available operations.

### Effects and Handlers

System operations are modeled as Resources:

- **Effects**: Represent intents to perform operations, structured as Resources with operational logic in their `value` field
- **Handlers**: Implement behavior for specific Effect kinds, also structured as Resources with logic in their `value` field
- **Intents**: Commitments to transform resources with satisfaction constraints

### Process Dataflow Blocks

Complex multi-step, multi-domain processes can be defined declaratively:

- **`ProcessDataflowBlock`**: Lisp S-expression structures defining workflow sequences
- **`ProcessLayer`**: Stages of execution within dataflows
- **`EffectNode`**: Individual effects within process layers
- **`ProcessEdge`**: Connections between effects with gating conditions

## Expression System

### Atomic Combinators

The system provides predefined combinators for Resource logic:

**Control Flow**: `if`, `and`, `or`, `not`
**Arithmetic**: `add`, `sub`, `mul`, `div` (with symbolic aliases `+`, `-`, `*`, `/`)
**Comparison**: `eq`, `gt`, `lt`, `gte`, `lte`
**Data Access**: `get-field`, `get-context-value`, `nth`, `len`
**Construction**: `list`, `record`, `make-map`, `cons`
**Type Predicates**: `is-string?`, `is-integer?`, `is-list?`
**String Operations**: `string-concat`, `string-to-upper`

### Evaluation Strategies

- **Off-Chain First**: Static logic (`static_expr`) executed by off-chain runtime
- **ZK Coprocessor**: Dynamic evaluation and proof verification for ZK-critical operations
- **Ahead-of-Time Compilation**: Future optimization for performance-critical evaluations

## Serialization and Storage

### SSZ Integration

All types use SSZ (Simple Serialize) for:
- Deterministic serialization
- Content-addressed identification via Merkle roots
- ZK-circuit compatibility
- Verifiable storage in Sparse Merkle Trees (SMTs)

### Sparse Merkle Trees (SMT)

Content-addressed entities are stored in SMTs providing:
- Cryptographic proofs of existence/non-existence
- Data integrity guarantees
- Partial state disclosure for privacy
- Proof-carrying data capabilities

## Type System Features

### Content-Addressed IDs

Core identifier types all derive from SSZ Merkle roots:
- `ResourceId`, `ValueExprId`, `ExprId`, `TypeExprId`
- `EffectId`, `HandlerId`, `IntentId`, `TransactionId`
- `DomainId`, `CapabilityId`, `MessageId`

### Conversion Traits

Unified type system with conversion traits:
- `AsResource`, `AsEffect`, `AsHandler`, `AsIntent`
- `ToValueExpr`, `FromValueExpr`
- `AsIdentifiable`, `HasDomainId`

### Error Handling

Comprehensive error types:
- `ResourceError`, `EffectHandlingError`
- `ConversionError`, `HandlerError`
- `ErrorCategory` for classification

## Module Organization

### Core (`core/`)
- `id.rs`: Content-addressed identifier types
- `resource.rs`: Resource type definition
- `effect.rs`: Effect type definition
- `handler.rs`: Handler type definition
- `intent.rs`: Intent type definition
- `transaction.rs`: Transaction type definition
- `traits.rs`: Core trait definitions
- `error.rs`: Error type definitions

### Expression System (`expr/`)
- `ast.rs`: Expression AST definitions
- `value.rs`: Value expression types
- `expr_type.rs`: Type expression system
- `result.rs`: Evaluation result types

### Graph System (`graph/`)
- Temporal Effect Graph (TEG) types
- Subgraph definitions
- Edge and node abstractions

### Provider Interfaces (`provider/`)
- `context.rs`: Execution context traits
- `registry.rs`: Registry interface traits
- `messenger.rs`: Message passing interfaces

### Patterns (`pattern/`)
- `message.rs`: Message pattern types
- Common interaction patterns

## Integration with Other Crates

This crate serves as the foundation for:

- **`causality-lisp`**: Implements the Lisp interpreter for `Expr` evaluation
- **`causality-runtime`**: Provides execution environments and host functions
- **`causality-simulation`**: Simulates Resource interactions and TEG execution
- **`causality-compiler`**: Compiles high-level definitions to TEG structures

## ZK Compatibility

All types are designed for Zero-Knowledge proof systems:
- Deterministic SSZ serialization
- No floating-point numbers
- Bounded dynamic evaluation
- Content-addressed verification
- SMT-based authenticated storage

## Usage

This crate is imported by all other Causality crates to access core type definitions. It provides the minimal set of types needed for the Resource Model while maintaining compatibility with both `std` and `no_std` environments.

```rust
use causality_types::{
    Resource, ValueExpr, Expr, ResourceId, 
    AsResource, ToValueExpr, FromValueExpr
};
```

The types defined here enable the construction of verifiable, deterministic systems where all state and behavior is explicitly modeled through Resources and their associated logic.
