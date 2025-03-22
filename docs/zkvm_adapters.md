# ZK-VM Powered Domain Adapters

This document provides an overview of the ZK-VM powered domain adapters, their architecture, and integration with the Valence protocol.

## Overview

ZK-VM powered domain adapters are specialized implementations that leverage zero-knowledge virtual machines to enable verifiable computation across different blockchain environments. These adapters extend the functionality of standard domain adapters by adding the ability to generate cryptographic proofs that can be verified by other chains without revealing sensitive information.

## Architecture

The ZK-VM adapters follow a layered architecture:

```
┌───────────────────────────────────────────┐
│             Domain Adapter API            │
└───────────────┬───────────────────────────┘
                │
┌───────────────▼───────────────────────────┐
│             ZK-VM Adapter Base            │
└───────────────┬───────────────────────────┘
                │
        ┌───────┴───────┐
        │               │
┌───────▼─────┐ ┌───────▼─────┐
│ RISC Zero   │ │  Succinct   │     ZK-VM Backends
│   Backend   │ │   Backend   │
└───────┬─────┘ └───────┬─────┘
        │               │
        └───────┬───────┘
                │
┌───────────────▼───────────────────────────┐
│           Target VM Adapters              │
│  (EVM, CosmWasm, Solana, etc.)            │
└───────────────────────────────────────────┘
```

### Components

1. **ZK-VM Base Adapter**: Provides common functionality for all ZK-VM powered adapters.
2. **ZK-VM Backends**: Specific implementations for different ZK-VM technologies like RISC Zero and Succinct.
3. **Target VM Adapters**: Implementations for specific target VMs like EVM, CosmWasm, etc.

## Key Features

### Verifiable Computation

ZK-VM adapters enable verifiable computation by executing operations in a ZK-VM environment and generating proofs that can be verified by any party, including other blockchain VMs.

```rust
// Generate a proof for a contract deployment
let proof = adapter.generate_proof(
    "deploy_contract",
    &contract_bytecode,
    &private_inputs
)?;

// Verify the proof on another chain
let is_valid = adapter.verify_proof(&proof)?;
```

### Cross-Chain Interoperability

ZK-VM adapters facilitate secure cross-chain operations by providing verifiable proofs that can be checked by target chains.

```rust
// Generate a proof on Ethereum
let proof = ethereum_adapter.generate_proof(...)?;

// Verify the proof on Cosmos
let is_valid = cosmos_adapter.verify_external_proof(&proof)?;
```

### Privacy-Preserving Operations

ZK proofs enable privacy-preserving operations by allowing verification without revealing sensitive information.

## Implemented Adapters

### ZK-VM Powered EVM Adapter

The ZK-VM powered EVM adapter (`ZkEvmAdapter`) enables verifiable Ethereum operations using ZK proofs. It supports:

- Contract deployment with verification
- Function execution with proofs
- State queries with ZK guarantees
- Cross-chain verification

### Guest Programs

ZK-VM adapters use guest programs that run inside the ZK-VM environment. These programs:

1. Read public and private inputs
2. Execute the requested operation
3. Generate output values and commitments
4. Produce assertions that can be verified

Example guest program:

```rust
// Example ZK-VM guest program for EVM operations
fn guest_entrypoint() {
    // Read inputs
    let public_inputs = env::read_public_inputs::<PublicInputs>();
    let private_inputs = env::read_private_inputs::<PrivateInputs>();
    
    // Log operation for debugging
    env::log(&format!("Executing operation: {}", public_inputs.operation_type));
    
    // Execute operation
    let result = execute_operation(&public_inputs, &private_inputs);
    
    // Commit result
    env::commit_result(&result);
}
```

## Setup and Configuration

To use a ZK-VM adapter, first configure it with the appropriate settings:

```rust
let zkvm_config = ZkVmAdapterConfig {
    domain_id: DomainId::new("ethereum-test"),
    target_vm_type: VmType::Evm,
    zkvm_backend: ZkVmBackend::RiscZero,
    guest_program_path: Some("examples/zkvm_guest_program.rs".to_string()),
    guest_program_id: None,
    proving_api_endpoint: None,
    auth_token: None,
    debug_mode: true,
    extra_config: HashMap::new(),
};

let evm_config = ZkEvmAdapterConfig {
    base_config: zkvm_config,
    chain_id: 1337, // Local test chain
    rpc_endpoints: vec!["http://localhost:8545".to_string()],
    gas_price: Some("1".to_string()),
    verifier_contract: None,
    private_key: None,
};

// Create adapter
let adapter = ZkEvmAdapter::new(evm_config);
```

## Testing

ZK-VM adapters include comprehensive test suites that verify:

1. Correct adapter creation
2. Proof generation and verification
3. Cross-chain operations
4. Effect validation

Testing can be performed using the provided test script:

```bash
./scripts/test_zkvm_evm.sh
```

## Integration with Domain Adapter Registry

ZK-VM adapters can be registered with the domain adapter registry:

```rust
// Register ZK-EVM adapter factory
let zkvm_evm_factory = Box::new(zkvm_evm::ZkEvmAdapterFactory::new());
registry.zkvm_registry_mut().register_factory(zkvm_evm_factory)?;
```

## Future Development

Future development of ZK-VM adapters will focus on:

1. Supporting additional target VMs (Solana, Move, etc.)
2. Improving proof generation performance
3. Adding more complex cross-chain operations
4. Enhancing privacy-preserving features
5. Supporting recursive proofs for complex workflows

## Conclusion

ZK-VM powered domain adapters represent a significant advancement in the Valence protocol, enabling secure, verifiable, and privacy-preserving cross-chain operations. By leveraging zero-knowledge technology, these adapters provide a foundation for complex multi-VM workflows with strong security guarantees. 