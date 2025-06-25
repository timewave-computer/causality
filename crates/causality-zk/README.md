# Causality ZK

Zero-knowledge proof framework compiling register machine instructions to arithmetic circuits with multi-backend support for verifiable computation.

## Core Components

### Circuit Compiler
Transforms register machine instructions to arithmetic circuits:

```rust
use causality_zk::{ZkCircuit, CircuitCompiler};

let compiler = CircuitCompiler::new();
let circuit = compiler.compile_program(instructions)?;
```

**Instruction Mapping:**
- `transform` → morphism application constraints
- `alloc`/`consume` → resource lifecycle validation
- `compose`/`tensor` → sequential/parallel composition
- `witness` → public/private input separation

### Backend Abstraction
Unified interface for multiple proving systems:

```rust
use causality_zk::{ZkBackend, ProofGenerator, BackendType};

let backend = causality_zk::create_backend(BackendType::Valence);
let proof_generator = ProofGenerator::new(backend);
```

**Supported Backends:**
- **Mock**: Configurable testing with success rate control
- **SP1**: Local proving infrastructure
- **Valence**: Production coprocessor integration

### Witness Management
Schema-based witness validation:

```rust
use causality_zk::{WitnessSchema, WitnessRule, ZkWitness};

let schema = WitnessSchema::new()
    .with_rule("balance", WitnessRule::Range { min: 0, max: 1000000 });
let witness = ZkWitness::new("circuit_id", public_inputs, private_inputs);
```

### Proof Management
Content-addressed proof storage and verification:

```rust
use causality_zk::{ProofManager, ProofMetadata};

let proof_manager = ProofManager::new();
let proof_id = proof_manager.store_proof(proof, metadata)?;
```

## Key Features

- **Circuit Compilation**: Register machine to arithmetic circuit mapping
- **Multi-Backend Support**: Unified interface for different provers
- **Witness Validation**: Type-safe witness generation with schema
- **Content Addressing**: Deterministic circuit and proof identification
- **Cross-Domain Proofs**: Support for distributed proof generation

## Integration

- **Layer 0**: Direct compilation from 5 fundamental instructions
- **Storage Proofs**: Verifiable content-addressed storage operations
- **Verification**: Built-in proof verification with public input validation
- **Error Handling**: Comprehensive error types for different failure modes
