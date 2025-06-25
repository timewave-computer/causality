# DEX Swap (Lisp)

## Overview

This example demonstrates **decentralized exchange (DEX) swap operations** using Causality Lisp. It shows how to implement atomic token swaps with proper liquidity management and price calculation.

## What it demonstrates

- **Atomic Swaps**: All-or-nothing token exchange operations
- **Liquidity Pool Management**: Working with DEX liquidity pools
- **Price Calculation**: Computing exchange rates and slippage
- **Multi-token Operations**: Managing multiple token types in one transaction

## Key Concepts

1. **Liquidity Pools**: Reserves of tokens available for trading
2. **Atomic Swaps**: Ensuring either both sides of trade complete or neither does
3. **Price Discovery**: Calculating fair exchange rates
4. **Slippage Protection**: Preventing unfavorable price movements

## How to run

```bash
# From the dex-swap directory
cd /Users/hxrts/projects/timewave/reverse-causality
# Remove comments first (Lisp parser doesn't handle ;; comments yet)
grep -v "^;;" examples/lisp-examples/dex-swap/dex_swap.lisp | grep -v "^$" > /tmp/clean_dex.lisp
cargo run --bin causality -- compile --input /tmp/clean_dex.lisp --output /tmp/dex_swap.out
cat /tmp/dex_swap.out
```

## Expected Output

The compilation will produce register machine instructions that implement the DEX swap logic with proper atomic execution and liquidity management.

## Architecture Notes

This demonstrates how complex **DeFi primitives** can be expressed in **Layer 2** Lisp and compiled to efficient **Layer 0** execution while maintaining strong safety guarantees for financial operations.
