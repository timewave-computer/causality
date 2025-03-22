# ZK-VM Powered EVM Adapter

This document describes the implementation of the ZK-VM powered Ethereum Virtual Machine (EVM) adapter, which uses zero-knowledge virtual machines to provide verifiable Ethereum operations.

## Overview

The ZK-VM powered EVM adapter allows verifiable execution of Ethereum operations using zero-knowledge proofs. This enables:

1. Provable execution of Ethereum smart contracts
2. Verified state transitions without revealing private data
3. Cross-chain verification of Ethereum operations
4. Scalable off-chain computation with on-chain verification

## Architecture

The ZK-VM EVM adapter follows a layered architecture:

```
┌───────────────────────────────────────────┐
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
│           ZK-EVM Adapter                  │
└───────────────────────────────────────────┘
```

## Components

### ZkEvmAdapter

The `ZkEvmAdapter` is the main implementation of the ZK-VM powered EVM adapter:

```rust
pub struct ZkEvmAdapter {
    base_config: ZkVmAdapterConfig,
    chain_id: u64,
    rpc_endpoints: Vec<String>,
    gas_price: Option<String>,
    verifier_contract: Option<String>,
    private_key: Option<String>,
}
```

### Configuration

The adapter is configured using `ZkEvmAdapterConfig`:

```rust
pub struct ZkEvmAdapterConfig {
    pub base_config: ZkVmAdapterConfig,
    pub chain_id: u64,
    pub rpc_endpoints: Vec<String>,
    pub gas_price: Option<String>,
    pub verifier_contract: Option<String>,
    pub private_key: Option<String>,
}
```

### Supported Effect Types

The adapter supports the following effect types:

```rust
pub enum ZkEvmEffectType {
    DeployContract,
    ExecuteFunction,
    TransferEth,
    UpdateState,
}
```

## Implementation Details

### Guest Programs

The adapter uses guest programs that run inside the ZK-VM environment to perform operations and generate proofs. The guest program for EVM operations follows this structure:

```rust
// Example ZK-VM guest program for EVM operations
fn guest_entrypoint() {
    // Read inputs
    let public_inputs = env::read_public_inputs::<PublicInputs>();
    let private_inputs = env::read_private_inputs::<PrivateInputs>();
    
    // Execute operation
    let result = execute_operation(&public_inputs, &private_inputs);
    
    // Commit result
    env::commit_result(&result);
}
```

### Proof Generation

The adapter generates ZK proofs for Ethereum operations:

```rust
fn generate_proof(
    &self,
    effect_type: &str,
    params: &serde_json::Value,
    private_inputs: &serde_json::Value
) -> Result<ZkProof>
```

Example for deploying a contract:

```rust
let deploy_params = serde_json::json!({
    "bytecode": "0x...",
    "constructor_args": "0x...",
});

let private_inputs = serde_json::json!({
    "wallet_address": "0x1234567890123456789012345678901234567890",
    "nonce": 42,
    "gas_price": "5000000000",
    "gas_limit": 3000000,
    "chain_id": 1,
});

let proof = adapter.generate_proof(
    ZkEvmEffectType::DeployContract.as_str(),
    &deploy_params,
    &private_inputs,
)?;
```

### Proof Verification

The adapter can verify proofs both locally and on-chain:

```rust
fn verify_proof(&self, proof: &ZkProof) -> Result<bool>
fn verify_proof_on_chain(&self, proof: &ZkProof, verifier_contract: Option<&str>) -> Result<String>
```

### Effect Validation

The `ZkEvmEffectValidator` validates effects before they are executed:

```rust
pub struct ZkEvmEffectValidator;

impl ZkEvmEffectValidator {
    pub fn new() -> Self {
        Self
    }
}

impl EffectValidator for ZkEvmEffectValidator {
    fn supports_effect_type(&self, effect_type: &str) -> bool {
        matches!(effect_type, 
            "deploy_contract" | "execute_function" | "transfer_eth" | "update_state")
    }
    
    fn validate_effect(&self, 
                      effect_type: &str, 
                      params: &serde_json::Value, 
                      context: &ValidationContext) -> ValidationResult {
        // Validation logic for each effect type
    }
}
```

## Factory

The `ZkEvmAdapterFactory` creates instances of the ZK-VM powered EVM adapter:

```rust
pub struct ZkEvmAdapterFactory;

impl ZkEvmAdapterFactory {
    pub fn new() -> Self {
        Self
    }
}

impl VmAdapterFactory for ZkEvmAdapterFactory {
    fn name(&self) -> String {
        "zk_evm".to_string()
    }
    
    fn supported_vm_types(&self) -> Vec<VmType> {
        vec![VmType::ZkVm]
    }
    
    fn create_adapter(&self, config: &serde_json::Value) -> Result<Box<dyn VmAdapter>> {
        // Factory implementation
    }
}
```

## Usage Examples

### Creating and Configuring the Adapter

```rust
let zkvm_config = ZkVmAdapterConfig {
    domain_id: DomainId::new("ethereum-mainnet"),
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
    chain_id: 1, // Ethereum mainnet
    rpc_endpoints: vec!["https://eth-mainnet.provider.com".to_string()],
    gas_price: Some("auto".to_string()),
    verifier_contract: None,
    private_key: None,
};

// Create adapter
let adapter = ZkEvmAdapter::new(evm_config);
```

### Deploying a Contract with Verification

```rust
// Contract bytecode and constructor arguments
let deploy_params = serde_json::json!({
    "bytecode": "0x608060405234801561001057600080fd5b5061017f806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c8063a6f9dae11461003b578063e79a198f14610057575b600080fd5b610055600480360381019061005091906100f9565b610073565b005b61005f6100b3565b60405161006e9190610135565b60405180910390f35b...",
    "constructor_args": "0x",
});

// Private deployment parameters
let private_inputs = serde_json::json!({
    "wallet_address": "0x1234567890123456789012345678901234567890",
    "nonce": 42,
    "gas_price": "5000000000",
    "gas_limit": 3000000,
    "chain_id": 1,
});

// Generate proof
let proof = adapter.generate_proof(
    ZkEvmEffectType::DeployContract.as_str(),
    &deploy_params,
    &private_inputs,
)?;

// Verify proof locally
let is_valid = adapter.verify_proof(&proof)?;
assert!(is_valid);

// Submit proof on-chain
let tx_hash = adapter.verify_proof_on_chain(&proof, None)?;
println!("Verification transaction: {}", tx_hash);
```

### Executing a Contract Function

```rust
// Function call parameters
let execute_params = serde_json::json!({
    "contract": "0x1234567890123456789012345678901234567890",
    "function": "transfer(address,uint256)",
    "args": [
        "0x2222222222222222222222222222222222222222",
        "1000000000000000000"
    ],
});

// Private execution parameters
let private_inputs = serde_json::json!({
    "wallet_address": "0x1234567890123456789012345678901234567890",
    "nonce": 43,
    "gas_price": "5000000000",
    "gas_limit": 1000000,
    "chain_id": 1,
});

// Generate proof
let proof = adapter.generate_proof(
    ZkEvmEffectType::ExecuteFunction.as_str(),
    &execute_params,
    &private_inputs,
)?;

// Verify proof
let is_valid = adapter.verify_proof(&proof)?;
assert!(is_valid);
```

## Testing

The adapter includes extensive tests for validation and verification:

```rust
#[test]
fn test_zkvm_evm_adapter_creation() -> Result<()> {
    // Create ZK-VM EVM adapter configuration
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
    
    // Verify adapter properties
    assert_eq!(adapter.domain_id().as_ref(), "ethereum-test");
    assert_eq!(adapter.vm_type(), VmType::ZkVm);
    assert_eq!(adapter.target_vm_type(), VmType::Evm);
    assert_eq!(adapter.zkvm_backend(), &ZkVmBackend::RiscZero);
    
    Ok(())
}
```

## Future Enhancements

1. **Multi-Chain Support**: Extend the adapter to support multiple EVM-compatible chains
2. **Recursive Proofs**: Implement recursive proof composition for complex operations
3. **Performance Optimization**: Optimize guest programs for faster proof generation
4. **Aggregation**: Support batch verification of multiple EVM operations
5. **State Witnesses**: Implement compact state witnesses for efficient verification 