<!-- Zero-knowledge integration for CosmWasm -->
<!-- Original file: docs/src/cosmwasm_zk.md -->

# CosmWasm ZK-VM Integration

## Overview

The CosmWasm ZK integration enables zero-knowledge proof generation and verification for CosmWasm smart contracts. This allows for private and verifiable execution of CosmWasm contracts on Cosmos-based blockchains, with proofs that can be verified by the Causality Register System.

This adapter implements the `DomainAdapter` trait and provides capabilities for:

- Compiling Rust code to CosmWasm WASM with ZK circuit integration
- Deploying contracts to CosmWasm-compatible blockchains
- Executing contract calls with optional ZK proof generation
- Verifying the correctness of contract execution using ZK proofs
- Bridge functionality for interacting with CosmWasm chains

## Architecture

The CosmWasm ZK adapter is structured as follows:

```
causality/src/domain_adapters/cosmwasm_zk/
├── adapter.rs     # Main DomainAdapter implementation
├── bridge.rs      # Connection to CosmWasm blockchains
├── effects.rs     # Effect definitions for ZK operations
├── mod.rs         # Module exports
├── tests.rs       # Unit tests for the adapter
├── types.rs       # Data structures and type definitions
└── vm.rs          # ZK virtual machine implementation
```

### Components

#### CosmWasmZkAdapter

The main adapter that implements the `DomainAdapter` trait, providing the interface for the Causality system to interact with CosmWasm contracts and ZK proofs.

#### CosmWasmZkVm

The ZK-VM implementation that handles compilation, execution, and proof generation for CosmWasm contracts.

#### CosmWasmZkBridge

Bridge functionality for connecting to CosmWasm blockchains, deploying contracts, executing calls, and verifying proofs on-chain.

#### Effects

- `CompileEffect`: Compiles Rust code to CosmWasm WASM with ZK circuit integration
- `ExecuteContractEffect`: Executes a CosmWasm contract call
- `ProveEffect`: Generates a ZK proof of correct contract execution
- `VerifyEffect`: Verifies a ZK proof against expected inputs and outputs

## Usage

### Registering the Adapter

To use the CosmWasm ZK adapter in your application, register it with the Domain Registry:

```rust
use causality::domain_adapters::cosmwasm_zk::CosmWasmZkAdapter;
use causality::domain::domain_registry::DomainRegistry;

let mut registry = DomainRegistry::new();
let cosmwasm_adapter = CosmWasmZkAdapter::new();
registry.register_adapter(Box::new(cosmwasm_adapter));
```

### Compiling and Deploying Contracts

```rust
use causality::effect::Effect;

// Compile a CosmWasm contract with ZK circuit
let mut compile_effect = Effect::new("compile");
compile_effect.add_param("source", source_code);
compile_effect.add_param("program_id", "my_program");

let compile_result = domain.execute_effect(&compile_effect)?;
let program_id = compile_result.get_as_string("program_id")?;

// Deploy the compiled contract
let mut deploy_effect = Effect::new("deploy_contract");
deploy_effect.add_param("program_id", program_id);
deploy_effect.add_param("init_msg", r#"{"count": 0}"#);
deploy_effect.add_param("label", "my-counter-contract");

let deploy_result = domain.execute_effect(&deploy_effect)?;
let contract_address = deploy_result.get_as_string("contract_address")?;
```

### Executing Contracts with Proof Generation

```rust
// Execute a contract call with proof generation
let mut prove_effect = Effect::new("prove");
prove_effect.add_param("contract_address", contract_address);
prove_effect.add_param("method", "increment");
prove_effect.add_param("inputs", r#"{"value": 5}"#);
prove_effect.add_param("expected_output", r#"{"new_count": 5}"#);

let prove_result = domain.execute_effect(&prove_effect)?;
let proof_id = prove_result.get_as_string("proof_id")?;
```

### Verifying Proofs

```rust
// Verify a proof of contract execution
let mut verify_effect = Effect::new("verify");
verify_effect.add_param("proof_id", proof_id);
verify_effect.add_param("contract_address", contract_address);
verify_effect.add_param("method", "increment");
verify_effect.add_param("inputs", r#"{"value": 5}"#);
verify_effect.add_param("expected_output", r#"{"new_count": 5}"#);

let verify_result = domain.execute_effect(&verify_effect)?;
let is_valid = verify_result.get_as_bool("is_valid")?;
```

## ZK Proof System

The CosmWasm ZK adapter uses a custom zero-knowledge proving system that supports general-purpose computation. Key aspects of the system include:

1. **Circuit Generation**: Automatically generates a ZK circuit from the CosmWasm contract bytecode
2. **Prover**: Creates proofs of correct contract execution without revealing the contract state
3. **Verifier**: Efficiently verifies proofs against public inputs and claimed outputs
4. **Integration**: Seamlessly integrates with the Causality Register System for proof storage and verification

## Limitations

- The ZK proving system currently has performance limitations for complex contracts
- Proof generation can be resource-intensive for large state transitions
- Not all CosmWasm contract features are supported in ZK mode
- The maximum proof size is limited to 1MB

## Future Work

- Optimize circuit generation for common CosmWasm patterns
- Reduce proving time for complex contracts
- Support for recursive proofs to verify contract interactions
- Direct integration with popular Cosmos blockchains' verification systems
- Batched proof verification for improved efficiency 