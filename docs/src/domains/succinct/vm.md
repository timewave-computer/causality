<!-- Virtual machine for ZK -->
<!-- Original file: docs/src/zk_vm.md -->

# ZK-VM Integration Documentation

## Overview

The Causality system now includes ZK-VM integration, focusing on the Succinct ZK-VM backend. This integration allows domain-specific adapters to generate and verify zero-knowledge proofs for operations, enhancing privacy and security across domains.

## Key Components

- **Main Documentation**: [ZK Register System](./zk_register_system.md)
- **Implementation Directory**: `src/domain_adapters/succinct/`
- **Demo Script**: `src/bin/zk_demo.rs`

## Getting Started

To run the ZK-VM demo:

```bash
cargo run --bin zk_demo [optional-api-key]
```

The demo showcases:
1. Creating a Succinct ZK-VM adapter
2. Compiling a simple Rust program for the ZK-VM
3. Generating a zero-knowledge proof of execution
4. Verifying the proof
5. Reading output values from the proof journal

## Architecture

The ZK-VM integration has a streamlined architecture:

```
┌─────────────────────────────────────┐
│         Causality Core System       │
└───────────────┬─────────────────────┘
                │
┌───────────────▼─────────────────────┐
│      Domain & Effect Adapters       │
└───────────────┬─────────────────────┘
                │
┌───────────────▼─────────────────────┐
│     ZkVirtualMachine Bridge         │
└───────────────┬─────────────────────┘
                │
┌───────────────▼─────────────────────┐
│       Succinct Adapter              │
└─────────────────────────────────────┘
```

## Implementation Structure

The integration consists of several key files:

- `src/domain_adapters/succinct/mod.rs` - Main module exports
- `src/domain_adapters/succinct/types.rs` - Common data types
- `src/domain_adapters/succinct/adapter.rs` - Implementation of the Succinct adapter
- `src/domain_adapters/succinct/bridge.rs` - Bridge between ZkVirtualMachine and SuccinctAdapter
- `scripts/succinct_demo.rs` - Demo script showcasing the functionality

## Completed Tasks

- ✅ Created type definitions for ZK-VM interaction
- ✅ Implemented Succinct adapter with compile/prove/verify functionality
- ✅ Built bridge layer to implement ZkVirtualMachine trait
- ✅ Created demo script showcasing the integration
- ✅ Added documentation

## Next Steps

- Phase 13.2: Domain Adapter Refactoring for ZK-VM Targets
- Phase 13.3: Effect Implementation in ZK-VM Environments
- Phase 13.4: Code Generation for ZK-VM Targets
- Phase 13.5: Register Operations via ZK-VM
- Phase 13.6: Cross-Domain Proof Propagation
- Phase 13.7: Adapter Testing and Benchmarking
- Phase 13.8: Migration Strategy

## References

- [Succinct Documentation](https://docs.succinct.xyz/)
- [ZK Proof Systems Overview](./zk_register_system.md#proof-systems)
- [Causality Architecture](./architecture.md) 