# Unified Transform Examples

## Overview

This example demonstrates **unified transformations** across different layers of the Causality system. It shows how high-level transformations can be expressed uniformly and compiled to different target layers.

## What it demonstrates

- **Unified Transformation API**: Common interface for different transformation types
- **Multi-layer Compilation**: Transformations targeting Layer 1 and Layer 0
- **Type-level Transformations**: Compile-time transformation validation
- **Resource Transformation**: Converting between different resource types
- **Optimization Passes**: How transformations can be optimized

## Key Concepts

1. **Transformation Abstraction**: Unified interface for different transform types
2. **Target Selection**: Choosing appropriate compilation targets
3. **Resource Conversion**: Transforming between resource types
4. **Optimization Pipeline**: How transformations are optimized

## How to run

```bash
# From the unified-transforms directory
rustc unified_transform_examples.rs --edition 2021
./unified_transform_examples
```

## Expected Output

The demo will show:
- Different types of transformations
- Compilation to multiple targets
- Resource type conversions
- Optimization examples

## Architecture Notes

This demonstrates the **unified transformation system** that allows the same high-level transformation to be compiled to different layers depending on performance and safety requirements.
