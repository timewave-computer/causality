# ZK Effects Demo

## Overview

This example demonstrates **zero-knowledge proof integration** with the Causality effect system. It shows how ZK proofs can be generated and verified as part of effect execution, enabling privacy-preserving computations.

## What it demonstrates

- **ZK Proof Generation**: Creating zero-knowledge proofs from effects
- **ZK Proof Verification**: Validating proofs within the effect system
- **Privacy-Preserving Effects**: Effects that don't reveal sensitive data
- **ZK Circuit Integration**: How effects compile to ZK circuits
- **Proof Composition**: Combining multiple proofs

## Key Concepts

1. **ZK Effect Primitives**: Basic building blocks for ZK computations
2. **Proof Generation**: Creating proofs from effect execution
3. **Proof Verification**: Validating proofs in the effect system
4. **Privacy Preservation**: Keeping sensitive data private

## How to run

```bash
# From the zk-effects directory
rustc zk_effect_demo.rs --edition 2021
./zk_effect_demo
```

## Expected Output

The demo will show:
- ZK proof generation from effects
- Proof verification processes
- Privacy-preserving computations
- ZK circuit compilation

## Architecture Notes

This demonstrates how the **Layer 2** effect system integrates with **zero-knowledge proof systems** to enable privacy-preserving applications while maintaining the performance benefits of the **Layer 0** execution model.
