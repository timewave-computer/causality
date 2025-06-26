# 401: Layer 1/Layer 2 Architectural Separation

## Overview

This document describes the architectural separation between Layer 1 (Linear Lambda Calculus) and Layer 2 (Effect System), which maintains clean abstraction boundaries while ensuring mathematical purity and zero-knowledge circuit compatibility.

## Design Principles

The Causality architecture implements a clean separation between foundational mathematics (Layer 1) and application programming features (Layer 2) to achieve several critical goals:

1. **ZK Compatibility**: Static structure and monomorphic operations enable efficient zero-knowledge circuit generation
2. **Mathematical Purity**: Layer 1 maintains clean categorical semantics with exactly 5 fundamental instructions (Layer 0) and 11 primitives (Layer 1)
3. **Formal Verification**: Simple mathematical foundations enable complete formal analysis
4. **Rich Programming Model**: Layer 2 provides sophisticated features while compiling to verified Layer 1 code

## Current Architecture

### Layer 1: Pure Linear Lambda Calculus

Layer 1 is implemented in `causality-core/src/lambda/` and focuses exclusively on mathematical foundations:

```
lambda/
├── base.rs         # Core types (BaseType, Value, TypeInner)
├── linear.rs       # Linearity system and tracking
├── tensor.rs       # Tensor product implementation  
├── sum.rs          # Sum type implementation
├── function.rs     # Linear function implementation
├── symbol.rs       # Symbol type
├── term.rs         # AST and term representation
└── interface.rs    # Layer 0 compilation interface
```

**Key Characteristics**:
- **Layer 0: 5 fundamental instructions, Layer 1: 11 primitives** that compile to fixed-size ZK circuits
- **No built-in record types** - complex structures built with tensor products
- **No polymorphism** - all types monomorphic for ZK compatibility
- **Static structure** - all type layouts determined at compile time
- **Mathematical purity** - clean categorical semantics throughout

#### Layer 1 Core Features

##### 1. Mathematical Foundation
Layer 1 implements pure linear lambda calculus with categorical semantics, providing:
- **Base types**: Unit, Bool, Int, Symbol
- **Type constructors**: Tensor product (⊗), Sum types (⊕), Linear functions (⊸)
- **Resource management**: Linear allocation and consumption primitives
- **Clean compilation**: Direct mapping to Layer 0 register machine instructions

##### 2. The 11 Core Primitives
All Layer 1 computation is expressed through exactly 5 fundamental instructions (Layer 0) and 11 primitives (Layer 1):
- **Unit operations**: `unit`, `letunit`
- **Tensor operations**: `tensor`, `lettensor`
- **Sum operations**: `inl`, `inr`, `case`
- **Function operations**: `lambda`, `apply`
- **Resource operations**: `alloc`, `consume`

These primitives form a complete computational basis while maintaining mathematical rigor and ZK compatibility.

### Layer 2: Rich Programming Model

Layer 2 is implemented in `causality-core/src/effect/` and provides sophisticated programming abstractions:

```
effect/
├── core.rs         # Core effect types and operations
├── operations.rs   # Effect algebra operations  
├── capability.rs   # Capability-based access control system
├── object.rs       # Object model with linearity enforcement  
├── row.rs          # Record operations and row polymorphism
├── record.rs       # Record capability effects
├── intent.rs       # Intent-based programming
├── synthesis.rs    # Effect synthesis and compilation
├── teg.rs          # Temporal Effect Graph
├── resource.rs     # Resource algebra
├── causality.rs    # Causality tracking
└── pattern.rs      # Pattern matching
```

**Key Characteristics**:
- **Capability-based access control** for all resource operations
- **Object model** with linearity enforcement and capability checking
- **Record operations** with row polymorphism and schema management
- **Effect compilation** that resolves dynamic operations to static Layer 1 code
- **ZK compatibility** through static analysis and monomorphization

#### Layer 2 Core Features

##### 1. Capability System
The capability system provides fine-grained, unforgeable access control:
- **Capability levels**: Read, Write, Execute, Admin with implication relationships
- **Record-specific capabilities**: Field-level permissions and schema management
- **Capability algebra**: Compositional capability checking and derivation
- **Static resolution**: Capability requirements resolved at compile time

##### 2. Object Model
Linear objects with sophisticated linearity enforcement:
- **Linear objects**: Must be consumed exactly once
- **Affine objects**: May be consumed at most once (can be dropped)
- **Relevant objects**: Must be consumed at least once (can be copied)
- **Unrestricted objects**: No linearity constraints (standard reference semantics)

##### 3. Record Operations
Row polymorphism and structured data management:
- **Record schemas**: Type-safe field definitions with capability requirements
- **Row operations**: extend, restrict, project with compile-time verification
- **Schema resolution**: Dynamic record access compiled to static Layer 1 structures
- **Capability integration**: All field access mediated by capability system

## Compilation Architecture

The two-layer architecture enables a clean compilation pipeline that preserves both mathematical rigor and practical expressiveness:

### 1. Layer 2 → Layer 1 Compilation

```
Layer 2 Effects (capabilities, objects, records)
           ↓
    Capability Analysis & Schema Resolution
           ↓
    Monomorphization (remove polymorphism)
           ↓
    Effect-to-Lambda Compilation
           ↓
Layer 1 Terms (pure linear lambda calculus)
           ↓
Layer 0 Instructions (register machine)
           ↓
ZK Circuits (fixed-size, static structure)
```

### 2. Capability-Based Compilation Examples

#### Example 1: Dynamic Field Access
```rust
// Layer 2: Capability-based field access
access_field(account_resource, "balance", read_capability)

// After capability resolution and schema analysis:
// Layer 1: Static tensor operations
lettensor (account_data, metadata) = consume(account_resource) in
lettensor (balance, other_fields) = account_data in
alloc(balance)
```

#### Example 2: Record Update with Capabilities
```rust
// Layer 2: Capability-checked record update
update_field(record_resource, "amount", new_value, write_capability)

// After monomorphization:
// Layer 1: Pure structural operations
lettensor (old_amount, other_data) = consume(record_resource) in
alloc(tensor(new_value, other_data))
```

#### Example 3: Object Linearity Enforcement
```rust
// Layer 2: Linear object with capabilities
let linear_account = LinearObject::new(account_data, read_write_caps);
let balance = linear_account.access_field("balance")?;
let updated_account = linear_account.update_field("balance", new_balance)?;

// Layer 1: Pure tensor operations with linearity tracking
lettensor (account_fields, capabilities) = consume(account_object) in
lettensor (balance, other_data) = account_fields in
let new_fields = tensor(new_balance, other_data) in
alloc(tensor(new_fields, capabilities))
```

## Architectural Benefits

### 1. Zero-Knowledge Compatibility
- **Fixed structure**: All record layouts determined at compile time
- **Static control flow**: No dynamic field access in Layer 1
- **Bounded computation**: All operations compile to fixed-size circuits
- **Deterministic execution**: Same inputs produce identical circuits

### 2. Mathematical Purity
- **Clean semantics**: Layer 1 maintains categorical foundations
- **Formal verification**: 5 fundamental instructions (Layer 0) and 11 primitives (Layer 1) with precise mathematical meaning
- **Compositional reasoning**: Clear laws for tensor products, sums, and functions
- **Type safety**: Linear resource usage enforced by construction

### 3. Developer Experience
- **Rich programming model**: Capabilities, objects, and records available at Layer 2
- **Safety guarantees**: Capability system prevents unauthorized access
- **Linearity enforcement**: Object model provides flexible linearity patterns
- **Schema validation**: Record operations ensure type safety

### 4. Performance Optimization
- **Static compilation**: Dynamic operations resolved at compile time
- **Circuit optimization**: Fixed structure enables ZK circuit optimizations
- **Capability caching**: Static analysis enables capability resolution caching
- **Incremental compilation**: Changes to Layer 2 don't require Layer 1 recompilation

## Implementation Details

### Type System Integration

The separation enables sophisticated type checking at each layer:

#### Layer 1 Type System
```
τ ::= 1                     -- Unit type
    | Bool | Int | Symbol   -- Base types  
    | τ₁ ⊗ τ₂              -- Tensor product
    | τ₁ ⊕ τ₂              -- Sum types
    | τ₁ ⊸ τ₂              -- Linear functions
    | Resource τ            -- Linear resource handles
```

#### Layer 2 Type Extensions
```
σ ::= τ                     -- Layer 1 types
    | Effect τ              -- Effectful computations
    | Object L τ            -- Linear objects (L ∈ {Linear, Affine, Relevant, Unrestricted})
    | Record ρ              -- Record types with row polymorphism
    | Capability α          -- Access capabilities
    | Intent                -- Declarative specifications
```

### Compilation Guarantees

The architecture provides several key guarantees:

1. **Capability Soundness**: All Layer 2 capability operations compile to Layer 1 code that respects access restrictions
2. **Linearity Preservation**: Object linearity constraints are enforced throughout compilation
3. **Schema Consistency**: Record operations maintain type safety across compilation stages
4. **ZK Compatibility**: All Layer 2 constructs compile to ZK-compatible Layer 1 primitives

### Module Dependencies

The clean separation is enforced through module dependencies:

```
Layer 2 (effect/) → Layer 1 (lambda/) → Layer 0 (register machine)

// Layer 2 can import Layer 1 types and operations
use causality_core::lambda::{Term, Value, BaseType};

// Layer 1 cannot import Layer 2 (enforced by module structure)
// This ensures mathematical purity is maintained
```

## Future Extensibility

This architectural separation enables several future directions:

### 1. Independent Layer Evolution
- **Layer 1 stability**: Mathematical foundations can remain stable while Layer 2 evolves
- **Layer 2 extensions**: New programming features can be added without affecting Layer 1
- **Pluggable backends**: Layer 1 could target different ZK proof systems or execution models

### 2. Domain-Specific Extensions
- **Specialized capability systems**: Domain-specific access control patterns
- **Custom object models**: Application-specific linearity patterns
- **Record extensions**: Domain-specific field types and validation rules

### 3. Optimization Opportunities
- **Layer-specific optimization**: Each layer can be optimized for its specific role
- **Cross-layer analysis**: Whole-program optimization across layer boundaries
- **Circuit specialization**: Domain-specific circuit generation strategies

## Conclusion

The Layer 1/Layer 2 architectural separation successfully achieves the goal of providing both mathematical rigor and practical expressiveness. Layer 1 serves as a pure mathematical foundation optimized for zero-knowledge circuits and formal verification, while Layer 2 provides the rich programming model needed for real-world applications.

This separation ensures that Causality can provide strong mathematical guarantees while maintaining excellent developer experience, positioning it as a robust foundation for verifiable distributed systems. The capability system elegantly bridges the gap between mathematical purity and practical access control, enabling sophisticated applications while preserving the formal properties that make verification possible. 