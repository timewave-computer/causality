# Simple Transfer Effect

## Overview

This example demonstrates a **simple token transfer effect** using the Causality Layer 2 effect system. It shows how to create basic financial primitives with linear resource management and effect composition.

## What it demonstrates

- **Token Transfer Logic**: Basic transfer operations with validation
- **Linear Resource Safety**: Ensuring tokens can't be double-spent
- **Effect Composition**: Chaining validation, transfer, and update effects
- **Resource Constraints**: Type-level guarantees for resource management
- **Financial Primitives**: Building blocks for DeFi applications

## Key Concepts

1. **Token Resources**: Linear resources representing fungible tokens
2. **Transfer Validation**: Checking balances and permissions
3. **Atomic Operations**: Ensuring transfers are all-or-nothing
4. **Resource Tracking**: Maintaining resource linearity

## How to run

```bash
# From the simple-transfer directory
rustc simple_transfer_effect.rs --edition 2021
./simple_transfer_effect
```

## Expected Output

The demo will show:
- Token creation and initialization
- Transfer validation logic
- Successful transfer execution
- Resource consumption tracking

## Architecture Notes

This example shows how **Layer 2** financial primitives can be built with strong safety guarantees that compile down to efficient **Layer 0** execution.
