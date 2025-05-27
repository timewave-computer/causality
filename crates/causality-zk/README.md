# Causality ZK

Zero-knowledge proof generation and verification for the Causality Resource Model framework. This crate provides ZK circuit implementations, proof generation utilities, and integration with ZK coprocessors for verifiable Resource operations.

## Overview

The `causality-zk` crate enables zero-knowledge proof capabilities for the Causality system, providing:

- **Resource Validation Circuits**: ZK circuits for verifying Resource state transitions
- **ProcessDataflowBlock Verification**: Proof generation for complex dataflow operations
- **Capability System Proofs**: ZK proofs for authorization and permission verification
- **ZK Coprocessor Integration**: Communication with external ZK proof services
- **Circuit Management**: Tools for managing and deploying ZK circuits

All ZK implementations maintain compatibility with the Resource Model's content-addressed, SSZ-serialized architecture.

## Core Components

### ZK Circuit System

Core traits and structures for ZK circuit management:

```rust
use causality_zk::circuit::{ZkCircuit, CircuitInput, CircuitOutput};

pub trait ZkCircuit {
    type Input: CircuitInput;
    type Output: CircuitOutput;
    
    fn circuit_id(&self) -> CircuitId;
    fn generate_proof(&self, input: Self::Input) -> Result<ZkProof, ZkError>;
    fn verify_proof(&self, proof: &ZkProof, public_inputs: &[u8]) -> Result<bool, ZkError>;
}
```

### Resource Validation Circuits

ZK circuits for Resource state validation:

```rust
use causality_zk::circuits::ResourceValidationCircuit;

let circuit = ResourceValidationCircuit::new(resource_type);
let input = ResourceValidationInput {
    resource_state: resource.value.clone(),
    validation_expr: resource.static_expr.clone(),
    domain_constraints: domain.constraints.clone(),
};

let proof = circuit.generate_proof(input)?;
let verified = circuit.verify_proof(&proof, &public_inputs)?;
```

### ProcessDataflowBlock Verification

ZK proofs for dataflow execution verification:

```rust
use causality_zk::circuits::DataflowVerificationCircuit;

let circuit = DataflowVerificationCircuit::new();
let input = DataflowVerificationInput {
    block_definition: dataflow_block,
    execution_trace: trace,
    input_resources: inputs,
    output_resources: outputs,
};

let proof = circuit.generate_proof(input)?;
```

### ZK Coprocessor Integration

Integration with external ZK proof services:

```rust
use causality_zk::coprocessor::{ZkCoprocessorClient, ProofRequest};

let client = ZkCoprocessorClient::new("https://coprocessor.valence.xyz").await?;
let request = ProofRequest {
    circuit_id: "resource-validation".to_string(),
    witness_data: witness_bytes,
    public_inputs: public_inputs_bytes,
};

let proof_id = client.submit_proof_request(request).await?;
let proof = client.get_proof(&proof_id).await?;
```

### Capability System Proofs

ZK proofs for capability-based authorization:

```rust
use causality_zk::circuits::CapabilityProofCircuit;

let circuit = CapabilityProofCircuit::new();
let input = CapabilityProofInput {
    user_capabilities: user_caps,
    required_capabilities: required_caps,
    resource_constraints: constraints,
    operation_context: context,
};

let proof = circuit.generate_proof(input)?;
```

## Circuit Implementations

### Resource State Transition Circuit

```rust
use causality_zk::circuits::StateTransitionCircuit;

let circuit = StateTransitionCircuit::new();
let input = StateTransitionInput {
    old_state: previous_resource_state,
    new_state: updated_resource_state,
    transition_logic: effect_expr,
    nullifier: resource_nullifier,
};

let proof = circuit.generate_proof(input)?;
```

### Merkle Tree Inclusion Circuit

```rust
use causality_zk::circuits::MerkleInclusionCircuit;

let circuit = MerkleInclusionCircuit::new(tree_depth);
let input = MerkleInclusionInput {
    leaf: resource_commitment,
    path: merkle_path,
    root: tree_root,
};

let proof = circuit.generate_proof(input)?;
```

### Cross-Domain Operation Circuit

```rust
use causality_zk::circuits::CrossDomainCircuit;

let circuit = CrossDomainCircuit::new();
let input = CrossDomainInput {
    source_domain: source_domain_id,
    target_domain: target_domain_id,
    operation: cross_domain_op,
    authorization_proof: auth_proof,
};

let proof = circuit.generate_proof(input)?;
```

## Proof Management

### Proof Storage and Retrieval

```rust
use causality_zk::storage::{ProofStorage, ProofMetadata};

let storage = ProofStorage::new(storage_config);

// Store proof with metadata
let metadata = ProofMetadata {
    circuit_id: "resource-validation".to_string(),
    proof_type: ProofType::ResourceValidation,
    created_at: SystemTime::now(),
    public_inputs_hash: hash_public_inputs(&public_inputs),
};

storage.store_proof(&proof_id, &proof, metadata).await?;

// Retrieve proof
let (proof, metadata) = storage.get_proof(&proof_id).await?;
```

### Batch Proof Generation

```rust
use causality_zk::batch::{BatchProofGenerator, BatchRequest};

let batch_generator = BatchProofGenerator::new(coprocessor_client);
let requests = vec![
    BatchRequest::new("circuit-1", input1),
    BatchRequest::new("circuit-2", input2),
    BatchRequest::new("circuit-3", input3),
];

let batch_id = batch_generator.submit_batch(requests).await?;
let proofs = batch_generator.get_batch_results(&batch_id).await?;
```

## Configuration

ZK system configuration:

```toml
[zk]
coprocessor_endpoint = "https://coprocessor.valence.xyz"
circuit_cache_dir = ".causality/circuits"
proof_storage_dir = ".causality/proofs"
batch_size = 10
timeout_seconds = 300

[zk.circuits]
resource_validation = "circuits/resource_validation.r1cs"
dataflow_verification = "circuits/dataflow_verification.r1cs"
capability_proof = "circuits/capability_proof.r1cs"

[zk.proving_keys]
resource_validation = "keys/resource_validation.pk"
dataflow_verification = "keys/dataflow_verification.pk"

[zk.verification_keys]
resource_validation = "keys/resource_validation.vk"
dataflow_verification = "keys/dataflow_verification.vk"
```

## Feature Flags

- **default**: Standard ZK features
- **coprocessor**: ZK coprocessor integration
- **batch-proving**: Batch proof generation
- **circuit-cache**: Circuit caching and optimization
- **async**: Asynchronous proof generation

## Module Structure

```
src/
├── lib.rs                    # Main library interface
├── circuit.rs                # Core circuit traits and types
├── circuits/                 # Circuit implementations
│   ├── resource_validation.rs
│   ├── dataflow_verification.rs
│   ├── capability_proof.rs
│   ├── state_transition.rs
│   └── merkle_inclusion.rs
├── coprocessor.rs            # ZK coprocessor integration
├── storage.rs                # Proof storage and retrieval
├── batch.rs                  # Batch proof generation
└── utils.rs                  # ZK utilities and helpers
```

This crate enables the Causality system to generate and verify zero-knowledge proofs for Resource operations, ensuring privacy and verifiability while maintaining the deterministic properties of the Resource Model.
