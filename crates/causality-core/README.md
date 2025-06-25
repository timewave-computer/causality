# Causality Core

Foundational implementation of Causality's three-layer architecture providing unified transform-based computation and communication with content-addressed linear resources and zero-knowledge verifiability.

## Architecture

Causality implements three integrated computational layers:

### Layer 0: Register Machine (5 Fundamental Instructions)
Minimal register machine based on symmetric monoidal closed category theory:
- `transform` - Apply morphisms (unifies function calls, effects, protocols)
- `alloc` - Allocate linear resources (data, channels, functions)  
- `consume` - Consume linear resources (deallocation, cleanup)
- `compose` - Sequential composition (control flow, chaining)
- `tensor` - Parallel composition (parallel data, concurrency)

### Layer 1: Linear Lambda Calculus
Content-addressed expressions with unified type system:
- Linear type system with resource tracking
- Content-addressed AST nodes via `ExprId(EntityId)`
- Row types for records with location awareness
- Session types for communication protocols
- Compilation to Layer 0 instructions

### Layer 2: Transform-Based Effects
Unified computation and communication through transforms:
- Location-transparent operations (`Effect<From, To>`)
- Automatic protocol derivation from access patterns
- Intent-based declarative programming
- Capability-based access control
- Cross-chain coordination

## Core Components

### Machine Layer (`machine/`)
- **RegisterFile**: Fixed 32-register execution environment
- **ResourceHeap**: Content-addressed linear resource storage
- **Instruction**: 5 fundamental instruction types
- **Reduction**: Deterministic execution engine

### Lambda Layer (`lambda/`)  
- **Term**: Content-addressed expression system
- **TypeChecker**: Linear type inference and checking
- **Location**: Location-aware type system
- **SessionLinear**: Session type integration

### Effect Layer (`effect/`)
- **Transform**: Unified computation/communication operations
- **Intent**: Declarative programming interface  
- **Capability**: Fine-grained access control
- **Record/Row**: Structured data with location awareness
- **CrossChain**: Multi-chain coordination

### System Layer (`system/`)
- **ContentAddressing**: Deterministic entity identification
- **Domain**: Capability domains and scoping
- **Serialization**: SSZ-based deterministic encoding

## Key Features

- **Transform Unification**: Computation and communication as unified transformations
- **Content Addressing**: All entities identified by cryptographic hash
- **Linear Resources**: Use-once resource semantics with nullifier tracking
- **Location Transparency**: Same API for local and distributed operations
- **Automatic Protocols**: Communication protocols derived from transform patterns
- **ZK Compatibility**: Circuit-friendly design throughout

## Mathematical Foundation

Built on symmetric monoidal closed category theory:
- **Objects**: Linear resources (data, channels, functions, protocols)  
- **Morphisms**: Transformations between resources
- **Monoidal Structure**: Parallel composition (⊗)
- **Closure**: Internal hom (→) for functions and protocols
- **Symmetry**: Resource braiding and swapping

## Purpose

The `causality-core` crate serves as the architectural foundation of the Causality system, implementing the mathematical abstractions and execution models that enable verifiable distributed computation. It provides three distinct but integrated computational layers that work together to support everything from low-level register operations to high-level declarative programming.

### Key Responsibilities

- **Register Machine Implementation**: Provide a minimal, verifiable instruction set for deterministic computation
- **Linear Type System**: Enforce resource linearity and affine constraints at the type level
- **Effect Algebra**: Enable declarative programming through intent specification
- **Resource Management**: Implement content-addressed, nullifier-based resource lifecycle management
- **Domain Organization**: Provide capability-based access control and resource scoping

## Layer Integration

The three layers work together seamlessly:

### Layer 0 → Layer 1 Integration
The register machine provides the execution foundation for lambda calculus operations. Lambda expressions compile down to sequences of register machine instructions.

### Layer 1 → Layer 2 Integration  
The type system enables structured parameters for effects. Effects can manipulate typed values while preserving linearity constraints.

### Layer 2 → Layer 0 Integration
Effects ultimately compile down to register machine instruction sequences, providing high-level abstractions that generate low-level execution plans.

## Design Philosophy

### Mathematical Foundation
The system is built on solid mathematical principles:
- **Linear Logic**: Resource usage follows linear logic principles
- **Type Theory**: Strong static type system with linearity constraints
- **Content Addressing**: Cryptographic integrity built into addressing

### Verifiability by Design
Every component is designed for zero-knowledge proof generation:
- **Deterministic Operations**: All computations are reproducible
- **Minimal Instruction Set**: Register machine designed for circuit compilation
- **Content Addressing**: Cryptographic integrity throughout

### Compositional Architecture
The system emphasizes composition at every level:
- **Instruction Composition**: Complex programs built from simple instructions
- **Effect Composition**: Complex behaviors built from simple effects
- **Type Composition**: Complex types built from simple primitives

## Performance Characteristics

### Register Machine Performance
- **Instruction Overhead**: Minimal per-instruction cost
- **Memory Management**: Efficient register allocation and heap management
- **Nullifier Operations**: Constant-time nullifier verification

### Resource System Performance
- **Content Addressing**: O(1) lookup for existing resources
- **Nullifier Verification**: Constant-time verification operations
- **Serialization**: Efficient encoding/decoding

## Testing Framework

The crate includes comprehensive testing infrastructure:

```rust
#[test]
fn test_linearity_constraints() {
    let mut state = MachineState::new();
    
    // Store value in register
    state.store_register(RegisterId::new(1), MachineValue::Int(42), None);
    
    // Consume register
    assert!(state.consume_register(RegisterId::new(1)).is_ok());
    
    // Try to consume again - should fail
    assert!(state.consume_register(RegisterId::new(1)).is_err());
}
```

This comprehensive foundation enables the construction of complex distributed systems while maintaining mathematical rigor and verifiability throughout.
