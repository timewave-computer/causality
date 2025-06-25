# Token Transfer (Lisp)

## Overview

This example demonstrates **token transfer operations** using Causality Lisp with linear resource management. It shows how to safely transfer tokens between accounts while maintaining linearity constraints.

## What it demonstrates

- **Linear Resource Management**: Tokens as linear resources that can't be double-spent
- **Transfer Logic**: Safe transfer operations with validation
- **Account Management**: Working with sender and receiver accounts
- **Resource Consumption**: How linear resources are consumed in transfers

## Key Concepts

1. **Linear Token Resources**: Tokens that can only be used once
2. **Transfer Validation**: Checking sender balance and permissions
3. **Atomic Operations**: Ensuring transfers are all-or-nothing
4. **Resource Linearity**: Preventing double-spending through types

## How to run

```bash
# From the token-transfer directory
cd /Users/hxrts/projects/timewave/reverse-causality
# Remove comments first (Lisp parser doesn't handle ;; comments yet)
grep -v "^;;" examples/lisp-examples/token-transfer/token_transfer.lisp | grep -v "^$" > /tmp/clean_transfer.lisp
cargo run --bin causality -- compile --input /tmp/clean_transfer.lisp --output /tmp/token_transfer.out
cat /tmp/token_transfer.out
```

## Expected Output

The compilation will produce register machine instructions that implement the token transfer logic with proper linear resource management.

## Architecture Notes

This example shows how **Layer 2** financial primitives can express complex transfer logic that compiles down to efficient and safe **Layer 0** execution while maintaining strong linearity guarantees.
