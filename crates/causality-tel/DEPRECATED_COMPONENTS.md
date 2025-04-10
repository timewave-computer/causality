# Removed Components from causality-tel

This document lists components that have been removed as part of the TEG implementation. It provides migration guidance for users of these components.

## Removed Components

### TelHandlerAdapter (REMOVED)

**Previous Location**: `causality-tel/src/compiler.rs`

**Removal Reason**: 
The `TelHandlerAdapter` has been completely removed as it's been superseded by the more robust TEG-based implementation.

**Migration Path**: 
- Use `causality-engine::effect::tel::TelEffectAdapter` instead
- The new TEG-based approach provides a more robust implementation with better integration with the effect system

### execute_tel_effect (REMOVED)

**Previous Location**: `causality-tel/src/types/effect.rs`  

**Removal Reason**:
The direct execution of TEL effects has been completely replaced by the TEG-based execution pipeline.

**Migration Path**:
- Use `causality-ir` and `causality-engine::effect::tel::TelEffectExecutor` instead
- First convert TEL programs to TEG using `Program.to_teg()` method
- Then execute the TEG using the `TelEffectExecutor` from the engine

### to_core_effect Method (REPLACED)

**Previous Location**: `TelEffect` method in `causality-tel/src/types/effect.rs`  

**Replacement Details**:
The method has been replaced with a direct implementation in `causality-engine/src/effect/tel/executor.rs` that uses `TelEffectAdapter` instead of the deprecated method.

**Migration Path**:
- Use the TEG-based execution pipeline instead:
  1. Convert TEL programs to TEG using `Program.to_teg()`
  2. Execute using `TegExecutor` from `causality-engine/src/effect/tel/teg_executor.rs`
- If you need to convert a TelEffect to a CoreEffect, use:
  ```rust
  let adapter = TelEffectAdapter::new(&effect.name, effect.combinator.clone());
  let core_effect = adapter_to_core_effect(adapter);
  ```

## Preferred TEG-based Execution Flow

The new execution flow is:

1. Parse TEL code to create a `Program` object
2. Convert the `Program` to a `TemporalEffectGraph` using the `to_teg()` method
3. Execute the TEG using the `TegExecutor` from the engine

Example:

```rust
// Parse TEL code
let program = tel_parser::parse_program(tel_code)?;

// Convert to TEG
let teg = program.to_teg()?;

// Execute with TEG executor
let teg_executor = TegExecutor::new(core_executor, resource_manager);
let result = teg_executor.execute(&teg).await?;
```

This approach provides better performance, optimizability, and cleaner semantics compared to the removed direct execution approach. 