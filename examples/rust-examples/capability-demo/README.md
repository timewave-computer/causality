# Capability Demo

## Overview

This example demonstrates **type-level capabilities** and **session types** in the Causality system. It shows how effect constraints can be expressed as row types and enforced at compile time, eliminating the need for runtime constraint checking.

## What it demonstrates

- **Session Types as Capabilities**: Capabilities are session types, not a separate system
- **Effect Row Types**: Effect constraints expressed as row types in the type system
- **Compile-time Enforcement**: Type system enforces constraints without runtime overhead
- **Capability Composition**: How multiple capabilities can be combined
- **Rate Limiting**: API rate limiting through type-level constraints
- **Data Access Control**: Fine-grained access control for different data types

## Key Concepts

1. **Rate-Limited API Capability**: Demonstrates how to limit API calls through types
2. **Data Access Capability**: Shows read/write permissions for different data types  
3. **Capability Composition**: Combining multiple capabilities together
4. **Session Type Protocols**: Using session types to define capability lifecycles

## How to run

```bash
# From the capability-demo directory
rustc capability_demo.rs --edition 2021
./capability_demo
```

## Expected Output

The demo will show:
- Rate-limited API capability allowing specific operations
- Data access capability with granular permissions
- Composed capabilities combining multiple constraint sets
- Session type protocols for capability management

## Architecture Notes

This example showcases the **Layer 2** capability system that compiles down to **Layer 1** tensor operations and ultimately **Layer 0** register machine instructions. The type-level constraints ensure that invalid operations are caught at compile time rather than runtime.
