# Effects Demo (OCaml)

## Overview

This example demonstrates **OCaml native algebraic effects** integration with the Causality system. It shows how algebraic effects provide direct-style programming without monadic composition overhead.

## What it demonstrates

- **Native Algebraic Effects**: OCaml 5.x algebraic effects system
- **Direct-Style Programming**: Natural control flow without monadic composition
- **Linear Resource Management**: Resources with single-use semantics
- **Effect Composition**: Combining effects naturally
- **Linearity Enforcement**: Preventing resource misuse at runtime
- **ZK Integration**: Zero-knowledge proof generation from effects

## Key Concepts

1. **Algebraic Effects**: Native OCaml effect system
2. **Linear Resources**: Resources that can only be consumed once
3. **Effect Handlers**: How effects are interpreted and executed
4. **Direct-Style Programming**: Natural control flow without >>= operators
5. **Linearity Violations**: Runtime detection of resource misuse

## How to run

```bash
# From the effects-demo directory
cd /Users/hxrts/projects/timewave/reverse-causality/ocaml_causality
dune exec test/test_effects.exe
```

## Expected Output

The demo will show:
- Linear resource creation and consumption
- Effect composition without monads
- Natural control flow with effects
- Linearity enforcement examples
- ZK witness generation
- Complex computation examples

## Architecture Notes

This demonstrates how **OCaml's native algebraic effects** provide a clean programming model for the **Causality Layer 2 system** while maintaining strong safety guarantees and efficient execution.
