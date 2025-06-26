# Layer 2 Pipeline (OCaml)

## Overview

This example demonstrates the **complete Layer 2 → Layer 1 → Layer 0 compilation pipeline** using OCaml. It shows how high-level concepts like intents and effects are compiled down to register machine instructions.

## What it demonstrates

- **3-Layer Architecture**: Layer 2 (intents) → Layer 1 (linear lambda calculus) → Layer 0 (register machine)
- **Content Addressing**: Deterministic ID generation for entities and domains
- **Intent Structure**: High-level declarative programming constructs
- **Effect Composition**: Building complex operations from simple effects
- **Pipeline Validation**: Ensuring correctness at each compilation stage
- **Register Machine Compilation**: Final compilation to executable instructions

## Key Concepts

1. **Layer 2 Intents**: High-level user-facing programming constructs
2. **Layer 1 Linear Lambda Calculus**: Mathematical foundation for computation
3. **Layer 0 Register Machine**: 5-instruction execution target
4. **Content Addressing**: Deterministic entity and domain identification
5. **Effect Pipelines**: Composing effects into complex operations

## How to run

```bash
# From the layer2-pipeline directory
cd /Users/hxrts/projects/timewave/reverse-causality/ocaml_causality
dune exec -- ocaml ../examples/ocaml-examples/layer2-pipeline/ocaml_layer2_example_working.ml
```

## Expected Output

The demo will show:
- Content addressing examples
- Entity and domain ID management
- Layer 2 intent structures
- Effect composition pipelines
- Pipeline validation results
- Layer 0 compilation output

## Architecture Notes

This demonstrates the complete **Causality compilation pipeline** from high-level declarative programming down to efficient register machine execution, showing how the 3-layer architecture provides both expressiveness and performance.
