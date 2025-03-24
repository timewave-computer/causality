# ZK Circuit Specifications for Register Operations

## Overview

This document specifies the Zero-Knowledge circuits used to verify register operations in the Causality system. Each circuit is designed to validate specific aspects of register operations while maintaining privacy and security guarantees.

## Common Components

### Public Inputs

All register operation circuits expose the following public inputs:

```rust
struct RegisterPublicInputs {
    // The register ID (for existing registers) or commitment (for new registers)
    register_id_or_commitment: [u8; 32],
    
    // The nullifier if the register is being consumed
    nullifier: Option<[u8; 32]>,
    
    // Operation type identifier
    operation_type: u8,
    
    // Domain identifier (where applicable)
    domain_id: Option<[u8; 32]>,
    
    // Timestamp of the operation
    timestamp: u64,
    
    // Hash of additional public parameters specific to each operation
    parameters_hash: [u8; 32],
}
```

### Private Inputs

The common private inputs for register circuits include:

```rust
struct RegisterPrivateInputs {
    // The register data (serialized)
    register_data: Vec<u8>,
    
    // Authorization proof (can be a signature, ZK proof, or other)
    authorization: AuthorizationProof,
    
    // Register specific private data (serialized)
    private_data: Vec<u8>,
    
    // Register state before the operation (for existing registers)
    previous_state: Option<Vec<u8>>,
    
    // Randomness used for commitments
    randomness: [u8; 32],
}
```

## Circuit Specifications

### 1. RegisterCreateCircuit

Validates the creation of a new register.

#### Public Inputs
- `register_id_or_commitment`: Commitment to the new register's initial state
- `operation_type`: Set to `1` (CREATE)
- `domain_id`: The domain where the register is being created
- `parameters_hash`: Hash of serialized creation parameters

#### Private Inputs
- `register_data`: Initial register data
- `authorization`: Proof of authority to create the register
- `randomness`: Randomness used to generate the commitment

#### Constraints
1. Verify that the commitment correctly binds to the initial register data and randomness
2. Verify that the authorization is valid for register creation
3. Verify that the register data satisfies any type-specific constraints
4. Verify that the parameters hash correctly reflects the creation parameters

### 2. RegisterUpdateCircuit

Validates the update of an existing register.

#### Public Inputs
- `register_id_or_commitment`: ID of the register being updated
- `nullifier`: Nullifier for the consumed register
- `operation_type`: Set to `2` (UPDATE)
- `parameters_hash`: Hash of serialized update parameters

#### Private Inputs
- `register_data`: New register data after update
- `previous_state`: Current register state before update
- `authorization`: Proof of authority to update the register
- `randomness`: Randomness used for nullifier generation

#### Constraints
1. Verify that the register ID matches the hash of the previous state
2. Verify that the nullifier is correctly derived from the register ID and randomness
3. Verify that the authorization is valid for this specific update
4. Verify that the update operation preserves any invariants (e.g., conservation of value)
5. Verify that the parameters hash correctly reflects the update parameters

### 3. RegisterTransferCircuit

Validates the transfer of a register between domains.

#### Public Inputs
- `register_id_or_commitment`: ID of the register being transferred
- `nullifier`: Nullifier for the consumed register
- `operation_type`: Set to `3` (TRANSFER)
- `domain_id`: Destination domain ID
- `parameters_hash`: Hash of serialized transfer parameters

#### Private Inputs
- `register_data`: Register data being transferred
- `previous_state`: Current register state before transfer
- `authorization`: Proof of authority to transfer the register
- `randomness`: Randomness used for nullifier generation

#### Constraints
1. Verify that the register ID matches the hash of the previous state
2. Verify that the nullifier is correctly derived from the register ID and randomness
3. Verify that the authorization is valid for this transfer operation
4. Verify that the transfer preserves the register's essential properties
5. Verify that the parameters hash correctly reflects the transfer parameters

### 4. RegisterDeleteCircuit

Validates the deletion of a register.

#### Public Inputs
- `register_id_or_commitment`: ID of the register being deleted
- `nullifier`: Nullifier for the consumed register
- `operation_type`: Set to `4` (DELETE)
- `parameters_hash`: Hash of serialized deletion parameters

#### Private Inputs
- `previous_state`: Current register state before deletion
- `authorization`: Proof of authority to delete the register
- `randomness`: Randomness used for nullifier generation

#### Constraints
1. Verify that the register ID matches the hash of the previous state
2. Verify that the nullifier is correctly derived from the register ID and randomness
3. Verify that the authorization is valid for register deletion
4. Verify that any dependent resources are properly accounted for
5. Verify that the parameters hash correctly reflects the deletion parameters

### 5. RegisterMergeCircuit

Validates the merging of multiple registers into one.

#### Public Inputs
- Multiple `register_id_or_commitment` values (for source registers)
- Multiple `nullifier` values (one for each source register)
- New register commitment (for the resulting merged register)
- `operation_type`: Set to `5` (MERGE)
- `parameters_hash`: Hash of serialized merge parameters

#### Private Inputs
- Multiple `previous_state` values (one for each source register)
- `register_data`: New register data after merging
- `authorization`: Proof of authority to merge the registers
- `randomness`: Randomness used for nullifiers and commitment

#### Constraints
1. Verify that each register ID matches the hash of its previous state
2. Verify that each nullifier is correctly derived from its register ID and randomness
3. Verify that the authorization is valid for all registers being merged
4. Verify that the merge operation preserves resource conservation laws
5. Verify that the resulting register commitment correctly binds to the merged data
6. Verify that the parameters hash correctly reflects the merge parameters

### 6. RegisterSplitCircuit

Validates the splitting of a register into multiple new registers.

#### Public Inputs
- `register_id_or_commitment`: ID of the register being split
- `nullifier`: Nullifier for the consumed register
- Multiple new register commitments (one for each resulting register)
- `operation_type`: Set to `6` (SPLIT)
- `parameters_hash`: Hash of serialized split parameters

#### Private Inputs
- `previous_state`: Current register state before splitting
- Multiple new register data values (one for each resulting register)
- `authorization`: Proof of authority to split the register
- `randomness`: Randomness used for nullifier and commitments

#### Constraints
1. Verify that the register ID matches the hash of the previous state
2. Verify that the nullifier is correctly derived from the register ID and randomness
3. Verify that the authorization is valid for register splitting
4. Verify that the split operation preserves resource conservation laws
5. Verify that each new register commitment correctly binds to its register data
6. Verify that the parameters hash correctly reflects the split parameters

## Resource Conservation Circuits

### ResourceConservationCircuit

A meta-circuit that verifies conservation principles across register operations.

#### Public Inputs
- Multiple operation public inputs
- Total input value commitment
- Total output value commitment

#### Private Inputs
- Value amounts for each input register
- Value amounts for each output register
- Register data for all registers involved

#### Constraints
1. Verify that the sum of input values equals the sum of output values
2. Verify that the input and output value commitments are correct
3. Verify that special rules (e.g., minting, burning) are properly authorized

## Circuit Verification Workflow

The verification workflow for register operations follows these steps:

1. **Gather Inputs**: Collect all necessary public and private inputs for the operation
2. **Generate Proof**: Use the appropriate circuit to generate a Zero-Knowledge proof
3. **Publish Public Inputs**: Make the public inputs available for verification
4. **Verify Proof**: Verify the proof against the public inputs
5. **Record Operation**: Upon successful verification, record the operation in the fact system

## Circuit Implementation Guidelines

When implementing these circuits:

1. **Use Standard Primitives**: Leverage well-studied cryptographic primitives (SHA-256, Pedersen commitments)
2. **Minimize Constraints**: Optimize circuits to reduce proving time and verification cost
3. **Batch Operations**: Where possible, batch multiple operations into a single proof
4. **Version Compatibility**: Include circuit version information in public inputs
5. **Error Reporting**: Design circuits to provide meaningful verification failure information

## Security Considerations

The ZK circuits must adhere to these security principles:

1. **No Information Leakage**: Private inputs should not be deducible from public inputs
2. **Replay Protection**: Nullifiers must prevent double-spending
3. **Authorization Binding**: Proofs must be tightly bound to the specific operation
4. **Deterministic Results**: Circuit execution must be deterministic for the same inputs
5. **Composability**: Circuits should be composable for complex operations 