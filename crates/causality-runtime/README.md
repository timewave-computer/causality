# Causality Engine

The `causality-engine` crate serves as the central coordinator and runtime implementation for the Causality system. It implements the interfaces defined by the `causality-effects` crate and orchestrates the execution of effects across the system.

## Core Responsibility

The Engine's primary responsibility is to be the **coordinator** between the different subsystems:

1. **Effect Runtime**: Implement the `EffectRuntime` interface defined in `causality-effects`
2. **Execution Management**: Handle effect execution, capability verification, and error propagation
3. **Handler Registration**: Manage registration and discovery of effect handlers
4. **Orchestration**: Coordinate between effects, resources, domains, and other subsystems
5. **Global Runtime**: Provide system-wide effect runtime

## Architecture

The engine is structured to fulfill its coordinator role:

```
causality-engine/
├── effect/             # Effect runtime implementation
│   ├── runtime.rs      # Implementation of EffectRuntime interface
│   ├── executor.rs     # Effect execution engine 
│   ├── registry.rs     # Handler registration and discovery
│   └── capability.rs   # Capability verification delegation
├── log/                # Unified log system
│   ├── storage.rs      # Log storage backends
│   ├── entry.rs        # Log entry definitions
│   └── replay.rs       # Log replay capabilities
├── execution/          # Execution infrastructure
│   ├── context.rs      # Execution context propagation
│   ├── scheduler.rs    # Task scheduling
│   └── deferred.rs     # Deferred execution
└── integration/        # Integration with other subsystems
    ├── resource.rs     # Resource system integration
    ├── domain.rs       # Domain system integration
    └── fact.rs         # Fact observation integration
```

## Relationship with Other Crates

### causality-effects

The engine crate implements the interfaces defined in the effects crate:

- Implements the `EffectRuntime` trait
- Implements the `CapabilityVerifier` interface
- Provides the concrete implementation for effect execution
- Adheres to the type definitions and error propagation

The engine never extends or modifies the interfaces, only implements them.

### causality-resource

The engine delegates resource-specific concerns to the resource crate:

- Uses the `ResourceManager` interface for capability verification
- Allows resources to register their effects through clean interfaces
- Never implements resource logic directly
- Maintains a clear separation between execution and resource concerns

### causality-domain

The engine integrates with domains through delegation:

- Uses domain adapter interfaces for cross-domain operations
- Allows domains to register their effects and handlers
- Maintains consistent context propagation across domains
- Coordinates fact observation across domain boundaries

## Core Components

### Effect Runtime Implementation

The engine implements the `EffectRuntime` interface as an orchestrator:

```rust
#[async_trait]
impl EffectRuntime for EngineEffectRuntime {
    async fn execute<E: Effect>(
        &self,
        effect: &E,
        param: E::Param,
        context: &Context,
    ) -> EffectResult<E::Outcome> {
        // Verify capabilities by delegating to appropriate subsystems
        self.verify_capabilities(effect, context)?;
        
        // Find the appropriate handler
        let handler = self.registry.get_handler(&effect.type_id())?;
        
        // Execute the effect through the handler
        let result = handler.handle(&effect.type_id(), param, context).await?;
        
        Ok(result)
    }
}
```

### Capability Verification

The engine delegates capability verification to the appropriate subsystems:

```rust
impl CapabilityVerifier for EngineCapabilityVerifier {
    fn verify_capabilities<E: Effect>(
        &self,
        effect: &E,
        context: &Context,
    ) -> EffectResult<()> {
        // Get required capabilities
        let capabilities = effect.required_capabilities();
        
        // Verify each capability by delegating to the appropriate manager
        for capability in capabilities {
            match capability.subsystem() {
                // Resource capabilities are verified by the resource manager
                Subsystem::Resource => {
                    self.resource_manager.verify_capability(capability, context)?;
                },
                // Domain capabilities are verified by the domain manager
                Subsystem::Domain => {
                    self.domain_manager.verify_capability(capability, context)?;
                },
                // Core capabilities are verified by the engine itself
                Subsystem::Core => {
                    self.verify_core_capability(capability, context)?;
                }
            }
        }
        
        Ok(())
    }
}
```

### Handler Registry

The engine provides a registry for managing effect handlers:

```rust
impl EffectRegistry {
    // Register handlers from different subsystems
    pub fn register_subsystem_handlers<S: SubsystemHandlerProvider>(
        &mut self,
        provider: &S,
    ) {
        provider.register_handlers(self);
    }
}
```

## Integration Points

The engine coordinates between different subsystems through these integration points:

1. **Resource Integration**
   - Delegates capability verification to resource manager
   - Allows resource-specific effects to be registered
   - Maintains clean abstraction over resource operations

2. **Domain Integration**
   - Coordinates domain adapter registration
   - Manages cross-domain effect execution
   - Handles domain context propagation

3. **Fact Observation**
   - Coordinates fact observation across domains
   - Manages fact verification and handling
   - Integrates fact log with effect execution

## Best Practices

When using or extending the engine:

1. **Respect separation of concerns**:
   - Engine orchestrates but delegates actual implementation
   - Resource logic belongs in the resource crate
   - Domain logic belongs in the domain crate

2. **Follow the interface-based approach**:
   - Define interfaces in effects crate
   - Implement interfaces in appropriate subsystem crates
   - Use interfaces for integration between subsystems

3. **Maintain clean component boundaries**:
   - Engine should not have deep knowledge of resource internals
   - Engine should not implement domain-specific logic
   - Each system should expose minimal interface surface

## Usage Examples

### Basic Integration

```rust
// Create the engine runtime
let mut runtime = EngineEffectRuntime::new();

// Integrate with the resource system
let resource_manager = ResourceManager::new();
runtime.with_resource_manager(Arc::new(resource_manager));

// Register domain handlers
let domain_registry = DomainRegistry::new();
domain_registry.register_domains(&mut runtime);

// Set as global runtime
set_effect_runtime(Arc::new(runtime));
```

### Executing Effects

```rust
// Get the runtime
let runtime = get_effect_runtime();

// Create and execute an effect
let effect = MyEffect::new();
let params = MyEffectParams { /* ... */ };
let context = Context::new();

let outcome = runtime.execute(&effect, params, &context).await?;
```

### Custom Handler Registration

```rust
// Create a custom effect handler
let handler = Arc::new(MyCustomHandler::new());

// Register with the engine
let mut runtime = get_effect_runtime_mut().unwrap();
runtime.register_handler(
    EffectTypeId::new("my.custom.effect"),
    handler
);
``` 