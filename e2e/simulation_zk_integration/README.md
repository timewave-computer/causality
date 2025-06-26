# Simulation ZK Integration E2E Test

This test suite verifies the integration between the Causality simulation engine and zero-knowledge proof systems, ensuring that simulated executions can be proven and verified cryptographically.

## What is Tested

### Simulation Engine Core
- **Effect Execution**: Running Layer 2 effects in simulation mode
- **State Transitions**: Tracking state changes through effect execution
- **Resource Management**: Linear resource handling in simulated environments
- **Cross-Chain Operations**: Multi-domain effect execution and coordination

### Zero-Knowledge Integration  
- **Proof Generation**: Creating ZK proofs for simulated executions
- **Circuit Compilation**: Converting effects to ZK circuits
- **Witness Generation**: Extracting execution traces for proof input
- **Verification**: Validating proofs against public parameters

### End-to-End Workflows
- **Simulate → Prove**: Complete workflow from effect execution to proof
- **Cross-Domain Proofs**: Proving multi-chain effect executions
- **Batch Processing**: Proving multiple effects together
- **Verification Pipeline**: Complete proof verification workflow

## How to Run

### Run All Simulation ZK Tests
```bash
cargo test --test simulation_zk_integration_e2e
```

### Run Individual Test Categories

#### Basic Simulation ZK Integration
```bash
cargo test --test simulation_zk_integration_e2e test_basic_simulation_zk_integration
```

#### Cross-Domain Effect Proving
```bash
cargo test --test simulation_zk_integration_e2e test_cross_domain_effect_proving
```

#### Batch Effect Verification
```bash
cargo test --test simulation_zk_integration_e2e test_batch_effect_verification
```

### Run with Verbose Output
```bash
cargo test --test simulation_zk_integration_e2e -- --nocapture
```

## Test Structure

The test suite covers three main integration scenarios:

### 1. Basic Simulation ZK Integration
- Creates a simple transfer effect
- Executes it in the simulation engine
- Generates a ZK proof of the execution
- Verifies the proof cryptographically

### 2. Cross-Domain Effect Proving
- Sets up multi-chain environment (Ethereum + Solana)
- Executes cross-chain bridge effect
- Proves the cross-domain state transitions
- Verifies domain-specific constraints

### 3. Batch Effect Verification
- Executes multiple effects in sequence
- Generates batch proof covering all effects
- Verifies the combined execution proof
- Validates state consistency across effects

## Dependencies

This test requires:
- **causality-simulation**: Simulation engine
- **causality-zk**: Zero-knowledge proof system
- **causality-core**: Core type system and effects
- **Mock ZK Backend**: Test implementation of ZK circuits

## Expected Results

All 3 tests should pass, verifying:
- ✅ Simulation engine produces valid execution traces
- ✅ ZK circuits correctly represent effect semantics
- ✅ Proof generation works for all effect types
- ✅ Verification accepts valid proofs and rejects invalid ones
- ✅ Cross-domain proofs maintain security properties
- ✅ Batch proving preserves individual effect guarantees

## Performance Notes

These tests involve cryptographic operations and may take longer than unit tests:
- Proof generation: ~100ms per simple effect
- Verification: ~10ms per proof
- Cross-domain proofs: ~500ms due to multi-chain setup
- Batch proofs: Time scales sub-linearly with batch size 