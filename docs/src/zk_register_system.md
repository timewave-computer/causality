# ZK-Based Register System

This document outlines the implementation of the Zero-Knowledge (ZK) Register System as described in [ADR-006: ZK-Based Register System for Domain Adapters](./adr_006_zk_registers.md).

## Overview

The ZK-Based Register System provides a secure and private method for managing state and resource transitions across multiple domains. Each register operation (create, update, transfer, delete) is paired with a zero-knowledge proof, allowing one domain to validate operations without revealing sensitive information about the register contents.

## System Components

The register system consists of the following components:

### 1. Core Data Structures

- **Register**: A data structure containing ID, owner, domain, contents, and timestamps
- **RegisterId**: A unique identifier for registers
- **RegisterContents**: The contents of a register (supports multiple types)
- **RegisterOperation**: An enum representing operations on registers

### 2. Zero-Knowledge Infrastructure

- **Circuit**: A trait representing a ZK circuit with methods for generating and verifying proofs
- **Proof**: A structure containing proof data and the proof system used
- **PublicInputs/WitnessInputs**: HashMaps containing the public and private inputs to circuits
- **Prover**: A service for generating ZK proofs for register operations
- **Verifier**: A service for verifying ZK proofs for register operations
- **ZkpService**: A high-level service for managing ZK operations with caching

### 3. Register Circuits

Specialized circuits for different register operations:

- **RegisterCreateCircuit**: Proves the correctness of register creation
- **RegisterUpdateCircuit**: Validates updates to register contents
- **RegisterTransferCircuit**: Proves ownership transfer while preserving contents
- **RegisterDeleteCircuit**: Validates register deletion

### 4. Utility Functions

- **Hash functions**: For creating commitments to register contents
- **Nullifier generation**: Prevents double-spending of registers
- **Resource delta calculation**: Ensures resource conservation laws
- **Public/witness input generation**: Creates inputs for circuits

## Implementation Details

### Proof Generation Process

1. A register operation is initiated (create, update, transfer, delete)
2. The `ZkpService` generates appropriate public and witness inputs
3. The relevant circuit is selected based on the operation
4. The Prover generates a proof using the inputs
5. The proof is cached for potential reuse
6. The proof is returned for verification by other domains

### Proof Verification Process

1. A domain receives a proof and public inputs for a register operation
2. The `ZkpService` selects the appropriate circuit for verification
3. The Verifier checks the proof against the public inputs
4. If valid, the operation is accepted; if invalid, it is rejected

### One-Time Use Register Model

Registers follow a one-time use model:
- Each register has a state (Active, Consumed, etc.)
- Operations on registers consume the current register and create new ones
- Nullifiers prevent reuse of consumed registers
- This creates a clear state transition history

## Proof Systems

The implementation supports multiple proof systems:

- **Groth16**: Default system for most operations
- **Plonk**: Planned implementation for more complex operations
- **Stark**: Planned implementation for batch operations

## Performance Considerations

- **Proof caching**: The `ZkpService` caches proofs to avoid regeneration
- **Key management**: Proving and verification keys are cached and managed
- **Batched verification**: Multiple proofs can be verified in a batch

## Security Aspects

- **Nullification**: Prevents register double-spending
- **Authorization**: Multiple methods (signature, delegation, timelock, etc.)
- **Resource conservation**: Validates that resources are conserved in operations

## Integration Points

The register system integrates with:

- **Resource system**: Manages resources through registers
- **Domain adapters**: Utilize ZK proofs for cross-domain operations
- **TEL language**: Exposes register operations at the language level
- **Fact system**: Records register operations as facts

## Future Work

- **Circuit optimization**: Reduce proving time and constraint count
- **Advanced operations**: Implement merge/split operations for registers
- **Recursive proofs**: Enable verification of proof chains
- **Garbage collection**: Implement register lifecycling and archival

## Example Workflow

```rust
// Create a register
let register = Register {
    id: "reg-123".to_string(),
    owner: Address::from_string("addr-abc").unwrap(),
    domain: "test-domain".to_string(),
    contents: RegisterContents::String("test content".to_string()),
    state: 1, // Active
    timestamps: RegisterTimestamps {
        created: 1000,
        updated: 1000,
    },
};

// Get the ZKP service
let zkp_service = ZkpServiceImpl::default();

// Generate a proof for register creation
let sender = Address::from_string("sender-123").unwrap();
let proof = zkp_service.prove_register_creation(&register, &sender).await?;

// Verify the proof
let public_inputs = create_public_inputs_for_register_creation(&register, &sender)?;
let is_valid = zkp_service.verify_register_proof(
    &RegisterOperation::Create,
    &proof,
    &public_inputs
).await?;

assert!(is_valid);
```

## References

1. [ADR-006: ZK-Based Register System for Domain Adapters](./adr_006_zk_registers.md)
2. [Zero Knowledge Proofs: An illustrated primer](https://blog.cryptographyengineering.com/2014/11/27/zero-knowledge-proofs-illustrated-primer/)
3. [Groth16 Protocol](https://eprint.iacr.org/2016/260.pdf) 