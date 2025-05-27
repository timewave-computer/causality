# TEG Migration Guide

## Overview

This guide helps developers migrate from the deprecated direct TEL execution model to the new Temporal Effect Graph (TEG) based execution model. The TEG approach offers several advantages:

1. More powerful optimization capabilities
2. Better composability of effects and resources
3. Enhanced debugging and visualization
4. Formal verification opportunities

## Key Changes

The main architectural change is the introduction of an intermediate representation (TEG) between TEL programs and execution:

- **Old approach**: TEL Program → Direct Execution
- **New approach**: TEL Program → TEG → Execution

## Migration Steps

### 1. Update Direct Effect Execution

#### Before:

```rust
// Direct execution of TEL effects
use causality_tel::types::effect::TelEffect;
use causality_engine::effect::tel::executor::TelEffectExecutor;

let effect = TelEffect::new("log", vec!["Hello World".into()]);
let executor = TelEffectExecutor::new();
let result = executor.execute_effect(&effect, context).await?;
```

#### After:

```rust
// TEG-based execution using the adapter
use causality_tel::combinators::Combinator;
use causality_engine::effect::tel::adapter::TelEffectAdapter;
use causality_engine::effect::executor::EffectExecutor;

// Create adapter
let adapter = TelEffectAdapter::new(
    "log",
    Combinator::Literal(causality_tel::combinators::Literal::String("Hello World".to_string()))
);

// Execute through core effect system
let effect_executor = EffectExecutor::new_with_registry(effect_registry.clone());
let context = effect_executor.create_context();
let effect = adapter.to_core_effect();
let result = effect_executor.execute_effect(effect, &*context).await?;
```

### 2. Update Program Execution

#### Before:

```rust
// Direct execution of a TEL program
use causality_tel::{Parser, Compiler};
use causality_engine::effect::tel::executor::TelProgramExecutor;

let parser = Parser::new();
let ast = parser.parse(program_src)?;
let compiler = Compiler::new();
let program = compiler.compile(&ast)?;

let executor = TelProgramExecutor::new(resource_manager, effect_registry);
let result = executor.execute_program(&program).await?;
```

#### After:

```rust
// TEG-based execution of a TEL program
use causality_tel::{Parser, Compiler};
use causality_engine::effect::tel::teg_executor::TegExecutor;
use causality_engine::effect::executor::EffectExecutor;

// Parse and compile TEL
let parser = Parser::new();
let ast = parser.parse(program_src)?;
let compiler = Compiler::new();
let program = compiler.compile(&ast)?;

// Convert to TEG (this is the key new step)
let teg = program.to_teg()?;

// Setup execution environment
let effect_executor = Arc::new(EffectExecutor::new_with_registry(effect_registry.clone()));
let executor = TegExecutor::new(effect_executor, resource_manager);

// Execute TEG
let result = executor.execute(&teg).await?;
```

### 3. Update Effect Handlers

Effect handlers remain largely the same, but are now accessed through the core effect system:

```rust
// Register an effect handler (same pattern for both old and new)
effect_registry.register_handler("effect_name", |params, context| {
    // Handler logic here
    Ok(CoreEffectOutcome::success(data))
}).expect("Failed to register handler");
```

### 4. Accessing Execution Results

The output format has slightly changed:

#### Before:

```rust
// Old TelProgramExecutor result format
let output_value = result.output;
let execution_metrics = result.metrics;
```

#### After:

```rust
// New TegExecutor result format
let outputs = result.outputs;
let execution_metrics = result.metrics;
let execution_trace = result.trace;  // New detailed execution trace
```

## Reference Examples

For complete reference examples, see:

1. `crates/causality-engine/src/tests/teg_execution_test.rs` - Contains the recommended patterns for TEG-based execution
2. `crates/causality-engine/src/effect/tel/tests/integration_tests.rs` - Contains additional examples and advanced use cases

## Troubleshooting

### Common Issues

1. **Missing execution trace**: Ensure you're using the TEG execution model that provides detailed traces
2. **Compatibility with older code**: Use the adapter pattern for transitioning specific effects
3. **Resource handling differences**: The TEG model handles resources more explicitly through resource nodes

If you encounter any issues during migration, please consult the reference tests or contact the platform team for assistance. 