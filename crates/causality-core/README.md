# Causality Core

The foundational implementation of the Causality framework's three-layer architecture, providing register machine execution, linear lambda calculus, and effect algebra for distributed, zero-knowledge verifiable computation.

## Purpose

The `causality-core` crate serves as the architectural foundation of the Causality system, implementing the mathematical abstractions and execution models that enable verifiable distributed computation. It provides three distinct but integrated computational layers that work together to support everything from low-level register operations to high-level declarative programming.

### Key Responsibilities

- **Register Machine Implementation**: Provide a minimal, verifiable instruction set for deterministic computation
- **Linear Type System**: Enforce resource linearity and affine constraints at the type level
- **Effect Algebra**: Enable declarative programming through intent specification
- **Resource Management**: Implement content-addressed, nullifier-based resource lifecycle management
- **Domain Organization**: Provide capability-based access control and resource scoping

## Architecture Overview

The three-layer architecture represents different levels of computational abstraction:

### Layer 0: Register Machine Foundation
A minimal 11-instruction set virtual machine designed for verifiability and deterministic execution. This layer provides the computational foundation that can be easily proven in zero-knowledge systems.

### Layer 1: Linear Lambda Calculus  
A structured type system with configurable linearity constraints, enabling safe resource management and functional programming patterns while maintaining mathematical rigor.

### Layer 2: Effect Algebra
A declarative programming model where computations are expressed as effects that can be analyzed and optimized.

## Core Components

### Register Machine (`machine/`)

The register machine implements a minimal instruction set designed for verifiable computation:

```rust
use causality_core::machine::{MachineState, Instruction, RegisterId};

let mut machine = MachineState::new();

// Basic register operations
machine.execute_instruction(Instruction::Move { 
    src: RegisterId(0), 
    dst: RegisterId(1) 
})?;

// Resource lifecycle operations
machine.execute_instruction(Instruction::Alloc { 
    type_reg: RegisterId(2), 
    val_reg: RegisterId(3), 
    out_reg: RegisterId(4) 
})?;
```

**Core Instruction Set (11 Instructions):**
- **Movement**: `Move` for data movement between registers
- **Resource Operations**: `Alloc`, `Consume` for resource lifecycle
- **Control Flow**: `Apply`, `Match`, `Select`, `Return` for program control
- **Constraints**: `Check` for runtime constraint verification
- **Effects**: `Perform` for effect execution
- **External Interface**: `Witness` for external data input

### Linear Lambda Calculus (`lambda/`)

A type system that enforces linearity constraints to ensure safe resource usage:

```rust
use causality_core::lambda::{Value, Type, Linearity};

// Create values with linearity constraints
let linear_value = Value::with_linearity(42, Linearity::Linear);
```

**Linearity Levels:**
- **Linear**: Must be used exactly once
- **Affine**: Can be used at most once  
- **Relevant**: Must be used at least once
- **Unrestricted**: Can be used any number of times

### Effect Algebra (`effect/`)

A declarative programming model based on effect expressions:

```rust
use causality_core::effect::{EffectExpr, EffectExprKind};

// Define effect expressions
let effect = EffectExpr::new(EffectExprKind::Effect {
    name: "transfer".to_string(),
    args: vec![],
});
```

### Resource Model (`machine/resource.rs`)

The resource system provides content-addressed, immutable entities with privacy-preserving consumption:

```rust
use causality_core::machine::{Resource, ResourceHeap, Nullifier};

let mut heap = ResourceHeap::new();

// Resources are content-addressed and immutable
let resource = Resource::simple("test", vec![42]);
let resource_id = heap.store_resource(resource)?;

// Consumption uses nullifiers for privacy
let nullifier = heap.consume_resource(&resource_id)?;
```

**Key Properties:**
- **Content Addressing**: IDs determined by cryptographic hash of content
- **Immutability**: Resources never change after creation
- **Nullifier System**: Privacy-preserving consumption tracking
- **Zero-Knowledge Compatibility**: Designed for efficient proof generation

### Content Addressing System

All entities use deterministic, content-based identifiers:

```rust
use causality_core::{EntityId, ContentAddressable};

// IDs are deterministic based on content
let resource = Resource::simple("test", vec![42]);
let entity_id = resource.entity_id(); // SHA256 of content
```

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
