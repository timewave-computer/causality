# Simple Token (Lisp)

## Overview

This example demonstrates the **simplest possible token implementation** using Causality Lisp. It shows how to create a basic linear resource using only the core primitives that are currently implemented.

## What it demonstrates

- **Basic Resource Creation**: Using `alloc` to create a token resource
- **Pure Computations**: Wrapping resource creation in pure effects
- **Lisp Syntax**: Basic Causality Lisp syntax and structure
- **Layer 2 → Layer 0 Compilation**: How Lisp compiles to register machine code

## Key Concepts

1. **Linear Resources**: Resources that can only be consumed once
2. **Pure Effects**: Effects that don't have side effects
3. **Resource Allocation**: Creating new resources in the system

## Code

```lisp
(pure (alloc 100))
```

This creates a pure effect that allocates a resource with value 100.

## How to run

```bash
# From the simple-token directory
cd /Users/hxrts/projects/timewave/reverse-causality
cargo run --bin causality -- compile --input examples/lisp-examples/simple-token/simple_token.lisp --output /tmp/simple_token.out
cat /tmp/simple_token.out
```

## Expected Output

The compilation will produce JSON-formatted register machine instructions:

```json
{
  "format": "causality-ir-v1",
  "instructions": [
    "Alloc { type_reg: RegisterId(1), init_reg: RegisterId(2), output_reg: RegisterId(0) }",
    "Alloc { type_reg: RegisterId(5), init_reg: RegisterId(6), output_reg: RegisterId(4) }",
    "Alloc { type_reg: RegisterId(4), init_reg: RegisterId(0), output_reg: RegisterId(3) }"
  ],
  "metadata": {
    "compilation_time": "N/A",
    "instruction_count": 3
  }
}
```

## Architecture Notes

This demonstrates the complete **Layer 2 → Layer 1 → Layer 0** compilation pipeline, from high-level Lisp syntax down to register machine instructions.
