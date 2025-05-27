# Causality Toolkit

Standard effects, handlers, and utilities for building applications with the Causality Resource Model framework. This crate provides reusable components that simplify common patterns in Resource-based systems.

## Overview

The `causality-toolkit` crate serves as a high-level toolkit that builds on the core types and runtime capabilities of the Causality framework. It provides:

- **Standard Effects**: Common effect implementations for typical application needs
- **Effect Handlers**: Production-ready handlers for standard effects
- **Type-Safe Resource Management**: Utilities for managing Resource lifecycle and state
- **Capability System**: Tools for building permission and authorization systems
- **Schema Integration**: Type schema support for content-addressed type definitions

All components maintain consistency with the Resource Model's content-addressed, SSZ-serialized architecture.

## Core Components

### Effect System

The toolkit provides a comprehensive effect system built on top of the core Causality types:

```rust
use causality_toolkit::core::ToolkitEffect;

pub trait ToolkitEffect: Send + Sync + AsValueExpr + Debug + 'static {
    fn effect_type_str(&self) -> Str;
    fn effect_logic_id(&self) -> EffectId;
    fn as_any(&self) -> &dyn Any;
}
```

#### Effect Composition

```rust
use causality_toolkit::core::EffectExpr;

let effect1 = MyEffect::new("param1");
let effect2 = MyEffect::new("param2");

let workflow = EffectExpr::single(effect1)
    .then(effect2)
    .then(MyOtherEffect::new());

workflow.execute(&handler).await?;
```

### Standard Effects

#### LogMessage Effect

```rust
use causality_toolkit::effects::{LogMessageEffect, LogMessageEffectInput, LogMessageHandler};

let log_input = LogMessageEffectInput {
    level: "info".to_string(),
    message: "Resource validation completed".to_string(),
    context: Some("ResourceValidator".to_string()),
};

let handler = LogMessageHandler::default();
let result = handler.handle(log_input).await?;
```

#### Effect Input/Output Schema

```rust
use causality_types::effects_core::{EffectInput, EffectOutput};

impl EffectInput for LogMessageEffectInput {
    fn from_value_expr(value: ValueExpr) -> Result<Self, ConversionError> {
        // Convert from ValueExpr to typed input
    }
    
    fn schema() -> TypeExpr {
        TypeExpr::Record(/* field definitions */)
    }
}
```

### Resource Management

#### Type-Safe Resource References

```rust
use causality_toolkit::core::{TypedResource, ConsumedResource, ResourceState};

let resource: TypedResource<TokenData, ResourceState> = 
    TypedResource::new(resource_id);

let consumed: ConsumedResource<TokenData> = 
    ConsumedResource::consume(resource);

let nullifier = consumed.nullifier();
```

#### Resource State Management

```rust
use causality_toolkit::core::ResourceState;

#[derive(Debug, Clone, Copy)]
pub enum ResourceState {
    Active,    // Resource exists and can be used
    Consumed,  // Resource has been consumed
    Created,   // Resource created but not committed
}
```

### Type Schema System

```rust
use causality_toolkit::AsTypeSchema;

pub trait AsTypeSchema {
    fn type_schema(&self) -> TypeExpr;
    fn schema_id(&self) -> TypeExprId;
    fn effect_type_name(&self) -> &'static str;
}
```

### Capability System

```rust
use causality_toolkit::capability::{CapabilitySystem, CapabilityCheck};

let capability_check = CapabilityCheck::new()
    .requires_capability("token.transfer")
    .with_resource_constraint("source_balance >= transfer_amount")
    .with_domain_scope(domain_id);

let authorized = capability_system.check_capability(
    &capability_check,
    &user_capabilities,
    &context
).await?;
```

## Usage Examples

### Defining Custom Effects

```rust
use causality_toolkit::core::{ToolkitEffect, ToolkitTelEffectData};

#[derive(Debug, Clone)]
pub struct TransferTokenEffect {
    pub from_resource: ResourceId,
    pub to_resource: ResourceId,
    pub amount: u64,
}

impl ToolkitEffect for TransferTokenEffect {
    fn effect_type_str(&self) -> Str {
        Str::from("token.transfer")
    }
    
    fn effect_logic_id(&self) -> EffectId {
        EffectId::new([/* hash of effect logic */])
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}
```

### Implementing Effect Handlers

```rust
use causality_toolkit::core::Handles;

pub struct TransferTokenHandler {
    // Handler state and dependencies
}

impl Handles<TransferTokenEffect> for TransferTokenHandler {
    fn handle(&self, effect: &TransferTokenEffect) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.validate_transfer(effect)?;
        self.execute_transfer(effect)?;
        self.update_resource_states(effect)?;
        Ok(())
    }
}
```

### Building Effect Workflows

```rust
use causality_toolkit::core::EffectExpr;

let token_transfer_workflow = EffectExpr::sequence(vec![
    EffectExpr::single(ValidateBalanceEffect::new(from_resource)),
    EffectExpr::single(CheckPermissionEffect::new(user_id, "transfer")),
    EffectExpr::single(TransferTokenEffect::new(from_resource, to_resource, amount)),
    EffectExpr::single(LogMessageEffect::new("info", "Transfer completed")),
]);

token_transfer_workflow.execute(&handler_registry).await?;
```

## Integration with Core Crates

### causality-types Integration

- Uses all core Resource Model types
- Implements effect traits from `effects_core`
- Maintains content-addressed consistency
- Supports SSZ serialization throughout

### causality-runtime Integration

- Provides handlers for runtime execution
- Integrates with host function system
- Supports async effect execution
- Enables Resource state management

### causality-lisp Integration

- Effect logic can be expressed in Lisp
- Supports constraint validation expressions
- Enables ZK-compatible effect definitions
- Provides schema-to-expression conversion

## Feature Flags

- **default**: Standard toolkit features
- **testing**: Additional testing utilities
- **capability-system-lisp-generator**: Capability system Lisp code generation
- **async**: Asynchronous effect handling

## Module Structure

```
src/
├── lib.rs                    # Main library interface and re-exports
├── core.rs                   # Core traits and resource management
├── effects.rs                # Standard effect implementations
├── capability.rs             # Capability system components
├── registry.rs               # Effect and handler registry
├── meta.rs                   # Metadata and schema utilities
├── control_flow.rs           # Control flow effects and patterns
└── capability_system_lisp/   # Lisp code generation for capabilities
```

## Testing Support

```rust
use causality_toolkit::core::testing::RecordingHandler;

let recording_handler = RecordingHandler::new();
effect.handle(&recording_handler)?;
let recorded_effects = recording_handler.get_recorded_effects();
assert_eq!(recorded_effects.len(), 1);
```

This toolkit enables rapid development of Resource-based applications by providing battle-tested components that handle common patterns while maintaining the verifiable and deterministic properties of the Causality framework.