# Multi-Party Transaction (Lisp)

## Overview

This example demonstrates **complex multi-party transactions** using Causality Lisp. It shows how to coordinate multiple participants in a single atomic transaction with proper resource management and validation.

## What it demonstrates

- **Multi-Party Coordination**: Managing multiple participants in one transaction
- **Complex Resource Flows**: Resources moving between multiple parties
- **Atomic Execution**: Ensuring all parts of the transaction succeed or fail together
- **Permission Management**: Different participants with different capabilities

## Key Concepts

1. **Multi-Party Atomicity**: All participants succeed or the entire transaction fails
2. **Resource Coordination**: Managing complex resource flows between parties
3. **Permission Validation**: Ensuring each party has required permissions
4. **Transaction Ordering**: Proper sequencing of multi-party operations

## How to run

```bash
# From the multi-party-transaction directory
cd /Users/hxrts/projects/timewave/reverse-causality
# Remove comments first (Lisp parser doesn't handle ;; comments yet)
grep -v "^;;" examples/lisp-examples/multi-party-transaction/multi_party_transaction.lisp | grep -v "^$" > /tmp/clean_multiparty.lisp
cargo run --bin causality -- compile --input /tmp/clean_multiparty.lisp --output /tmp/multi_party.out
cat /tmp/multi_party.out
```

## Expected Output

The compilation will produce register machine instructions that implement the multi-party transaction logic with proper atomicity and resource coordination.

## Architecture Notes

This demonstrates how **complex coordination patterns** can be expressed in **Layer 2** Lisp and compiled to efficient **Layer 0** execution while maintaining strong atomicity and safety guarantees.
