# Private Payment (Lisp)

## Overview

This example demonstrates **privacy-preserving payment operations** using Causality Lisp with zero-knowledge proof integration. It shows how to make payments while keeping transaction details private.

## What it demonstrates

- **Private Transactions**: Payments that don't reveal amounts or participants
- **Zero-Knowledge Proofs**: Proving payment validity without revealing details
- **Privacy-Preserving Logic**: Maintaining privacy throughout the transaction
- **ZK Circuit Integration**: How privacy features compile to ZK circuits

## Key Concepts

1. **Transaction Privacy**: Keeping payment details confidential
2. **Zero-Knowledge Validation**: Proving correctness without revealing data
3. **Private State Management**: Managing private account states
4. **ZK Proof Generation**: Creating proofs for private operations

## How to run

```bash
# From the private-payment directory
cd /Users/hxrts/projects/timewave/reverse-causality
# Remove comments first (Lisp parser doesn't handle ;; comments yet)
grep -v "^;;" examples/lisp-examples/private-payment/private_payment.lisp | grep -v "^$" > /tmp/clean_private.lisp
cargo run --bin causality -- compile --input /tmp/clean_private.lisp --output /tmp/private_payment.out
cat /tmp/private_payment.out
```

## Expected Output

The compilation will produce register machine instructions that implement the private payment logic with proper zero-knowledge proof integration.

## Architecture Notes

This demonstrates how **privacy-preserving applications** can be built using **Layer 2** Lisp that compiles to **ZK circuits** and **Layer 0** execution while maintaining strong privacy guarantees.
