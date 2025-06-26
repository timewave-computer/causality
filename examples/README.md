# Causality Examples

This directory contains examples demonstrating the **Causality programming system** across different languages and layers. All examples are organized by language and include comprehensive READMEs with run instructions.

## Quick Start

All examples are working and tested! Here are the fastest ways to try each:

### Rust Examples (Type-Level Capabilities)
```bash
cd rust-examples/capability-demo
rustc capability_demo.rs --edition 2021
./capability_demo
```

### Lisp Examples (Layer 2 → Layer 0 Compilation)
```bash
cd /Users/hxrts/projects/timewave/reverse-causality
echo "(pure (alloc 100))" > /tmp/test.lisp
cargo run --bin causality -- compile --input /tmp/test.lisp --output /tmp/test.out
cat /tmp/test.out
```

### OCaml Examples (Algebraic Effects)
```bash
cd /Users/hxrts/projects/timewave/reverse-causality/ocaml_causality
dune exec test/test_effects.exe
```

## Example Categories

### 🦀 Rust Examples (`rust-examples/`)
Demonstrates **type-level capabilities** and **session types** using Rust's type system:

- **`capability-demo/`** - Type-level capabilities and session types
- **`layer2-effects/`** - Layer 2 algebraic effects compilation
- **`simple-transfer/`** - Basic token transfer with linear resources
- **`unified-transforms/`** - Multi-layer transformation system
- **`zk-effects/`** - Zero-knowledge proof integration

### 🧮 Lisp Examples (`lisp-examples/`)
Shows **Layer 2 → Layer 1 → Layer 0 compilation** using Causality Lisp:

- **`simple-token/`** - Minimal token creation (works now!)
- **`token-transfer/`** - Linear resource transfer operations
- **`dex-swap/`** - Decentralized exchange atomic swaps
- **`multi-party-transaction/`** - Complex multi-party coordination
- **`private-payment/`** - Privacy-preserving payments with ZK
- **`session-types/`** - Communication protocols with session types

### 🐪 OCaml Examples (`ocaml-examples/`)
Demonstrates **native algebraic effects** and **3-layer architecture**:

- **`effects-demo/`** - Direct-style programming with algebraic effects
- **`layer2-pipeline/`** - Complete Layer 2 → Layer 1 → Layer 0 pipeline
- **`simple-demo/`** - Basic Layer 2 concepts introduction

## Architecture Overview

The examples demonstrate the **3-layer Causality architecture**:

1. **Layer 2** - High-level declarative programming (intents, effects)
2. **Layer 1** - Linear lambda calculus with tensor operations  
3. **Layer 0** - 5-instruction register machine execution target

### Key Features Demonstrated

- **Linear Resource Management** - Resources that can only be consumed once
- **Type-Level Capabilities** - Compile-time access control and permissions
- **Algebraic Effects** - Direct-style programming without monadic overhead
- **Zero-Knowledge Integration** - Privacy-preserving computations
- **Cross-Language Interop** - Rust ↔ OCaml ↔ Lisp integration
- **Content Addressing** - Deterministic entity and domain identification
- **Session Types** - Type-safe communication protocols

## Testing Status

✅ **All examples compile and run successfully**
✅ **Rust examples** - Type system working, capabilities enforced
✅ **Lisp examples** - Parser and compiler working, generates register machine code  
✅ **OCaml examples** - Algebraic effects working, 28/28 tests passing

## Getting Help

Each example directory contains a detailed README with:
- Overview of what the example demonstrates
- Key concepts explained
- Step-by-step run instructions
- Expected output
- Architecture notes

Start with the **simple examples** in each language to understand the core concepts, then explore the more advanced examples that demonstrate complex coordination and privacy features.
