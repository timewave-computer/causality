# Layer 2 Effects Demo

## Overview

This example demonstrates **Layer 2 algebraic effects** and how they compile down to the lower layers of the Causality system. It showcases effect composition, linear resource management, and the compilation pipeline from high-level effects to register machine instructions.

## What it demonstrates

- **Algebraic Effects**: Direct-style programming with effects
- **Linear Resource Management**: Resources that can only be consumed once
- **Effect Composition**: Combining multiple effects in a pipeline
- **Layer 2 → Layer 1 → Layer 0 Compilation**: Complete compilation pipeline
- **Resource Lifecycle**: Creation, transformation, and consumption of linear resources
- **Type Safety**: Compile-time guarantees for resource usage

## Key Concepts

1. **Linear Resources**: Resources with single-use semantics
2. **Effect Handlers**: How effects are interpreted and executed
3. **Resource Transformation**: Converting resources through effects
4. **Compilation Pipeline**: From effects to register machine code

## How to run

```bash
# From the layer2-effects directory
rustc layer2_effect_demo.rs --edition 2021
./layer2_effect_demo
```

## Expected Output

The demo will show:
- Linear resource creation and consumption
- Effect composition and chaining
- Resource transformation through effects
- Compilation to lower layers

## Architecture Notes

This demonstrates the **Layer 2** effect system that provides high-level abstractions for resource management while compiling down to efficient **Layer 0** register machine code.
