# Migration Guide: causality-effects to causality-core::effect

This document provides specific instructions for migrating from the `causality-effects` crate to the consolidated `causality-core::effect` module.

## Overview

As part of our architectural simplification efforts, the functionality from `causality-effects` has been consolidated into the `causality-core::effect` module. This guide will help you update your code to use the new API.

## Key Changes

| Legacy API (causality-effects) | New API (causality-core::effect) |
|--------------------------------|-----------------------------------|
| `Effect` trait | `Effect` trait (with updated methods) |
| `EffectOutcome` | `EffectResult` |
| `EffectContext` | `Context` |
| `EffectRegistry` | `EffectManager` |
| `capability::Capability` | `capability::Capability` (with updated methods) |
| `context::Context` | `context::ExecutionContext` |

## Migration Steps

### 1. Update Import Statements

Replace imports from `causality_effects` with imports from `causality_core::effect`:

```rust
// Old
use causality_effects::{
    Effect, EffectOutcome, EffectContext, EffectRegistry,
    capability::Capability,
    context::Context,
};

// New
use causality_core::effect::{
    Effect, EffectResult, Context, EffectManager,
    capability::Capability,
    context::ExecutionContext,
};
```

### 2. API Differences

#### Effect Implementation

```rust
// Old
#[derive(Debug)]
struct MyEffect {
    data: String,
}

impl Effect for MyEffect {
    fn execute(&self, context: &EffectContext) -> EffectOutcome {
        // Effect implementation
        EffectOutcome::Success
    }
}

// New
#[derive(Debug)]
struct MyEffect {
    data: String,
}

impl Effect for MyEffect {
    fn execute(&self, context: &Context) -> EffectResult {
        // Effect implementation
        EffectResult::success()
    }
}
```

#### Effect Registry/Manager

```rust
// Old
let mut registry = EffectRegistry::new();
registry.register::<MyEffect>();

// New
let mut manager = EffectManager::new();
manager.register::<MyEffect>();
```

#### Effect Context

```rust
// Old
let context = EffectContext::new();
let outcome = effect.execute(&context);

// New
let context = Context::new();
let result = effect.execute(&context);
```

### 3. Advanced Features

#### Capabilities

```rust
// Old
let capability = Capability::new("resource_access");
context.add_capability(capability);

// New
let capability = Capability::create("resource_access");
context.provide_capability(capability);
```

#### Context Handling

```rust
// Old
let parent_context = Context::new();
let child_context = parent_context.create_child();

// New
let parent_context = ExecutionContext::new();
let child_context = parent_context.spawn_child();
```

## Feature Parity

While most functionality has direct equivalents, some advanced features have been redesigned:

1. **Effect Chaining** - Now uses a more ergonomic builder pattern
2. **Effect Composition** - Enhanced support for composing effects
3. **Context Propagation** - Improved handling of context across effect boundaries

## Testing Your Migration

After migrating, ensure you:

1. Run all tests for your code that implements or uses effects
2. Verify that effect outcomes are correctly processed
3. Test effect execution in different contexts
4. Check that capabilities are properly managed

## Getting Help

If you encounter issues during migration:

1. Refer to the [general migration guide](./migration-guide.md)
2. Check the API documentation for `causality-core::effect`
3. Open an issue in the project repository

## Timeline

The `causality-effects` crate is now deprecated and will be removed in a future release. Please complete your migration as soon as possible to avoid disruption. 