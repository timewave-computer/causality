# Causality ZK

Zero-knowledge proof framework for the Causality system that compiles register machine instructions into arithmetic circuits and provides comprehensive proving infrastructure with multiple backend support for distributed, verifiable computation.

## Purpose

The `causality-zk` crate serves as the **zero-knowledge bridge** for the Causality system, transforming Layer 0 register machine instructions into arithmetic circuits suitable for zero-knowledge proof generation. It provides a complete ZK development framework with backend abstraction, enabling verifiable computation across different proving systems while maintaining the deterministic properties of the three-layer architecture.

### Key Responsibilities

- **Circuit Compilation**: Transform register machine instructions into arithmetic circuit representations
- **Multi-Backend Support**: Provide unified interface for different proving systems (Mock, SP1, Valence)
- **Proof Generation**: Generate zero-knowledge proofs for program execution
- **Proof Verification**: Verify proofs with public input validation
- **Witness Management**: Handle private witness data with schema validation
- **Content Addressing**: Manage circuits and proofs with deterministic identifiers

## Architecture Overview

The ZK framework is designed around several key architectural principles:

### Backend Abstraction
A unified interface that supports multiple proving backends:
- **Mock Backend**: Development and testing with configurable behaviors
- **SP1 Backend**: Local proving with SP1 infrastructure
- **Valence Backend**: Production proving with Valence coprocessors

### Circuit Compilation
Systematic mapping from register machine instructions to arithmetic constraints:
- **Instruction Mapping**: Each register machine instruction maps to specific circuit constraints
- **Type Preservation**: Linear type constraints enforced at the circuit level
- **Optimization**: Circuit optimization for proof efficiency

### Witness Processing
Comprehensive handling of private and public data:
- **Schema Validation**: Type-safe witness generation and validation
- **Privacy Preservation**: Proper separation of public and private inputs
- **Deterministic Generation**: Content-addressed witness management

## Core Components

### ZK Circuit Compiler (`circuit.rs`)

Transforms register machine instructions into arithmetic circuits:

```rust
use causality_zk::{ZkCircuit, CircuitCompiler, Instruction};

// Compile register machine program to ZK circuit
let instructions = vec![
    Instruction::Witness { out_reg: RegisterId(0) },
    Instruction::Move { src: RegisterId(0), dst: RegisterId(1) },
    Instruction::Alloc { type_reg: RegisterId(2), val_reg: RegisterId(3), out_reg: RegisterId(4) },
];

let compiler = CircuitCompiler::new();
let circuit = compiler.compile_program(instructions)?;
```

**Instruction Set Mapping:**
- **Data Movement**: `Move` → equality constraints
- **Resource Operations**: `Alloc`, `Consume` → type and linearity validation circuits
- **Control Flow**: `Apply`, `Select`, `Match`, `Return` → conditional constraints
- **External Interface**: `Witness` for public/private input separation

### Backend Abstraction Layer (`backends/`)

Unified interface for multiple proving systems:

```rust
use causality_zk::{ZkBackend, ProofGenerator, BackendType};

// Create backend (Mock, SP1, or Valence)
let backend = causality_zk::create_backend(BackendType::Valence);

// Generate proof using unified interface
let proof_generator = ProofGenerator::new(backend);
let proof = proof_generator.generate_proof(&circuit, &witness)?;

// Verify proof
let verified = proof_generator.verify_proof(&proof, &public_inputs)?;
```

**Backend Implementations:**
- **Mock Backend**: Configurable testing backend with success rate control
- **SP1 Backend**: Integration with SP1 proving infrastructure
- **Valence Backend**: Production-ready Valence coprocessor integration

### Witness Management System (`witness.rs`)

Schema-based witness validation and generation:

```rust
use causality_zk::{WitnessSchema, WitnessRule, ZkWitness};

// Define witness schema with validation rules
let schema = WitnessSchema::new()
    .with_rule("balance", WitnessRule::Range { min: 0, max: 1000000 })
    .with_rule("owner", WitnessRule::Boolean)
    .with_rule("amount", WitnessRule::NonZero);

// Create and validate witness
let witness = ZkWitness::new("circuit_id".to_string(), vec![500], vec![1, 2, 3]);
schema.validate(&witness)?;
```

**Witness Features:**
- **Schema Definition**: Type-safe witness structure definition
- **Validation Rules**: Comprehensive validation rule system
- **Public/Private Separation**: Clear separation of public and private data
- **Content Addressing**: Deterministic witness identification

### Proof Management (`proof.rs`)

Content-addressed proof storage and verification:

```rust
use causality_zk::{ProofManager, ProofId, ProofMetadata};

let proof_manager = ProofManager::new();

// Store proof with metadata
let proof_id = proof_manager.store_proof(proof, metadata)?;

// Retrieve and verify stored proof
let stored_proof = proof_manager.get_proof(&proof_id)?;
let verification_result = proof_manager.verify_stored_proof(&proof_id)?;
```

## Circuit Instruction Mapping

Complete mapping from register machine instructions to arithmetic circuits:

### Data Movement Instructions
```rust
// Move instruction: dst = src
match instruction {
    Instruction::Move { src, dst } => {
        circuit.add_constraint(register[dst] - register[src]); // dst == src
    }
}
```

### Resource Operations
```rust
// Resource allocation with type validation
match instruction {
    Instruction::Alloc { type_reg, val_reg, out_reg } => {
        circuit.add_type_validation_constraint(type_reg, val_reg);
        circuit.add_resource_creation_constraint(out_reg);
    }
    Instruction::Consume { resource_reg } => {
        circuit.add_nullifier_generation_constraint(resource_reg);
        circuit.add_linearity_constraint(resource_reg);
    }
}
```

### Control Flow Instructions
```rust
// Conditional execution and function calls
match instruction {
    Instruction::Select { cond, a, b, out } => {
        circuit.add_constraint(out - (cond * a + (1 - cond) * b));
    }
    Instruction::Apply { fn_reg, arg_reg, out_reg } => {
        circuit.add_function_call_constraint(fn_reg, arg_reg, out_reg);
    }
}
```

## Backend Configuration

### Mock Backend Configuration
```rust
use causality_zk::backends::{MockBackend, MockConfig};

let config = MockConfig {
    success_rate: 1.0,  // Always succeed for testing
    proof_delay: Duration::from_millis(100),
    verification_delay: Duration::from_millis(50),
};
let backend = MockBackend::with_config(config);
```

### SP1 Backend Configuration
```rust
use causality_zk::backends::{Sp1Backend, Sp1Config};

let config = Sp1Config {
    use_remote_prover: false,
    timeout_secs: 300,
    recursion_enabled: true,
};
let backend = Sp1Backend::with_config(config);
```

### Valence Backend Configuration
```rust
use causality_zk::backends::{ValenceBackend, ValenceConfig};

let config = ValenceConfig {
    endpoint: "http://prover.timewave.computer:37281".to_string(),
    timeout: Duration::from_secs(600),
    auto_deploy: true,
};
let backend = ValenceBackend::with_config(config);
```

## Design Philosophy

### Verifiability by Design
Every component is designed to maintain verifiability:
- **Deterministic Compilation**: Same program always produces same circuit
- **Cryptographic Integrity**: All artifacts are cryptographically verified
- **Transparent Proof Generation**: Clear audit trail for all proof operations

### Backend Agnostic
The framework abstracts over different proving systems:
- **Unified Interface**: Consistent API across all backends
- **Backend Selection**: Runtime backend selection based on requirements
- **Migration Support**: Easy migration between different proving systems

### Performance Optimization
Optimized for both proving time and verification efficiency:
- **Circuit Optimization**: Multiple optimization passes for efficient circuits
- **Parallel Processing**: Leverage multiple cores for proof generation
- **Caching**: Intelligent caching of compilation and verification results

## Testing Framework

Comprehensive testing across all ZK components:

```rust
// Property-based testing for circuit correctness
#[test]
fn test_circuit_preserves_semantics() {
    proptest!(|(program in any_valid_program())| {
        let direct_result = execute_program_directly(&program);
        let circuit = compile_to_circuit(&program);
        let proof = generate_proof(&circuit, &witness);
        let verified_result = verify_and_extract_result(&proof);
        assert_eq!(direct_result, verified_result);
    });
}

// Backend compatibility testing
#[test]
fn test_backend_compatibility() {
    let circuit = create_test_circuit();
    let witness = create_test_witness();
    
    for backend_type in [BackendType::Mock, BackendType::SP1, BackendType::Valence] {
        let backend = create_backend(backend_type);
        let proof = backend.generate_proof(&circuit, &witness)?;
        assert!(backend.verify_proof(&proof, &public_inputs)?);
    }
}
```

This comprehensive ZK framework enables the Causality system to generate zero-knowledge proofs for all register machine programs while maintaining the mathematical properties and deterministic execution characteristics essential for distributed verifiable computation.
