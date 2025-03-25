<!-- Documentation about ZK VM integration -->
<!-- Original file: docs/src/zk_vm_integration.md -->

# ZK-VM Integration

This document describes the integration of external Zero-Knowledge Virtual Machines (ZK-VMs) into the system, specifically focusing on the Succinct ZK-VM backend.

## Overview

The ZK-VM integration provides a unified framework for generating and verifying zero-knowledge proofs using established ZK-VM technologies. This implementation supports:

- Compiling and running guest programs in ZK environments
- Generating and verifying cryptographic proofs
- Cross-chain verification through on-chain contracts
- Resource estimation for ZK operations

## Architecture

The ZK-VM integration follows a layered architecture:

1. **Core Types Layer**: Common data structures and interfaces (`types.rs`)
2. **VM Layer**: Implementation of specific ZK-VM backends (`vm.rs`)
3. **Adapter Layer**: Unified API for interacting with ZK-VMs (`adapter.rs`)
4. **Domain Adapter Integration**: Connecting domain-specific operations to ZK-VMs

```
┌───────────────────────────────────────────┐
│               Domain Adapters             │
└───────────────┬───────────────────────────┘
                │
┌───────────────▼───────────────────────────┐
│               ZkAdapter                   │
└───────────────┬───────────────────────────┘
                │
┌───────────────▼───────────────────────────┐
│        Specific ZK-VM Implementation      │
│        (SuccinctVm)                       │
└───────────────┬───────────────────────────┘
                │
┌───────────────▼───────────────────────────┐
│        ZK-VM Common Types                 │
└───────────────────────────────────────────┘
```

## Key Components

### Common Types

The `types.rs` module defines the core data structures used across the ZK-VM integration:

- `ProgramId`: Identifier for ZK programs
- `PublicInputs`: Collection of public inputs for ZK programs
- `ProofData`: Proof data generated during ZK program execution
- `VerificationKey`: Key used to verify ZK proofs
- `ProofOptions`: Configuration options for proof generation
- `ExecutionStats`: Performance metrics for ZK operations

### Succinct VM Implementation

The `vm.rs` module implements the Succinct ZK-VM backend:

- `SuccinctVm`: Core implementation of the Succinct ZK-VM
  - Compiles guest code to Succinct programs
  - Generates ZK proofs through the Succinct service
  - Verifies proofs using Succinct's verification protocol
  - Generates verification contracts for on-chain verification
  - Estimates resource requirements for proving operations

### ZK Adapter

The `adapter.rs` module provides a unified interface for ZK-VM operations:

- `ZkAdapter`: Trait defining the common interface for all ZK-VM backends
- `SuccinctAdapter`: Implementation of the `ZkAdapter` trait for Succinct
- `ZkAdapterFactory`: Factory for creating ZK adapters based on specified backends

## Usage Examples

### Basic Proof Generation and Verification

```rust
use crate::domain_adapters::succinct::{default_adapter, PublicInputs};
use std::collections::HashMap;

// Create a Succinct adapter
let adapter = default_adapter()?.with_api_key("your-api-key");

// Example Rust program for ZK execution
let source_code = r#"
fn main() {
    let input = env::get_input("number").unwrap();
    let number: u32 = serde_json::from_str(&input).unwrap();
    
    // Verify that number is a perfect square
    let root = (number as f64).sqrt() as u32;
    assert_eq!(root * root, number, "Input is not a perfect square");
    
    env::set_output("root", &root);
}
"#;

// Compile the program
let program_id = adapter.compile_program(source_code, Some("perfect-square"))?;

// Prepare inputs
let mut public_inputs = PublicInputs::new();
public_inputs.add("number", &16u32)?;

let private_inputs = HashMap::new();

// Generate a proof
let proof = adapter.prove(
    &program_id,
    &public_inputs,
    &private_inputs,
    None,  // Use default options
)?;

// Verify the proof
let is_valid = adapter.verify(&program_id, &proof, &public_inputs)?;
assert!(is_valid);

// Extract the output from the journal
let journal = proof.journal.unwrap();
// In a real implementation, you would deserialize the journal to get outputs
```

### Generate Verification Contract

```rust
use crate::domain_adapters::succinct::{default_adapter, PublicInputs};

// Create a Succinct adapter
let adapter = default_adapter()?.with_api_key("your-api-key");

// Compile a program (or use an existing program ID)
let program_id = /* ... */;

// Generate a verification contract for Ethereum
let contract = adapter.generate_verification_contract(
    &program_id,
    "ethereum",
)?;

// The contract can now be deployed to Ethereum to enable on-chain verification
```

## On-Chain Verification

The ZK-VM integration supports generating verification contracts for multiple blockchains:

### Ethereum

For Ethereum and EVM-compatible chains, the system generates a Solidity contract that integrates with Succinct's verification infrastructure:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.16;

import "@succinctlabs/succinctx/ISuccinctGateway.sol";

contract SuccinctVerifier_example {
    // The Succinct gateway contract
    ISuccinctGateway public immutable succinctGateway;
    
    // The function ID of the Succinct function
    bytes32 public immutable functionId;
    
    constructor(address _succinctGatewayAddr, bytes32 _functionId) {
        succinctGateway = ISuccinctGateway(_succinctGatewayAddr);
        functionId = _functionId;
    }
    
    function verify(
        bytes calldata input,
        bytes calldata proof
    ) public view returns (bool) {
        return succinctGateway.verifyProof(functionId, input, proof);
    }
}
```

### Solana

For Solana, the system generates a Rust program that integrates with Succinct's Solana verification infrastructure:

```rust
// Solana program for verifying Succinct proofs
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Verify the Succinct proof
    // In a real implementation, this would use a Solana program
    // that implements the Succinct verification algorithm
    
    Ok(())
}
```

## Performance Considerations

When working with ZK-VMs, consider the following performance aspects:

1. **Proving Time**: Generating ZK proofs is computationally intensive and can take several seconds to minutes depending on program complexity.

2. **Memory Usage**: ZK proving requires significant memory, typically 512MB to several GB depending on program complexity.

3. **Verification Time**: Proof verification is much faster than generation, typically taking milliseconds.

4. **Proof Size**: Proofs range from 10KB to 100KB depending on the program and compression settings.

5. **On-Chain Gas Costs**: On-chain verification has gas costs proportional to the complexity of the verification logic and input data size.

## Future Work

The ZK-VM integration roadmap includes:

1. **Additional ZK-VM Backends**: Support for other ZK-VM technologies beyond Succinct.

2. **Proof Aggregation**: Combining multiple proofs into a single proof for more efficient verification.

3. **Automated Code Generation**: Streamlining the process of creating ZK-provable programs from high-level specifications.

4. **Performance Optimizations**: Reducing proving time and memory requirements through algorithmic improvements.

5. **Cross-Chain Proof Propagation**: Building infrastructure for securely sharing proofs across different blockchain networks.

## References

- [Succinct ZK Documentation](https://docs.succinct.xyz/)
- [ADR-006: ZK-Based Register System](./adr_006_zk_registers.md)
- [Work Plan 006: ZK-Based Register System Implementation](../work/006.md) 