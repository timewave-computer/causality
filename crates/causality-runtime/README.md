# Causality Runtime

Execution environment for the Causality Resource Model framework. This crate provides the runtime system that orchestrates Resource logic evaluation, ProcessDataflowBlock execution, and cross-domain operations through the TEL (Temporal Effect Logic) interpreter.

## Overview

The `causality-runtime` crate serves as the execution environment for the Causality system, providing:

- **TEL Interpreter**: Orchestrates Resource logic evaluation and ProcessDataflowBlock execution
- **Host Functions**: Bridge between Lisp expressions and runtime capabilities
- **State Management**: Manages Resource states and system-wide state transitions
- **ProcessDataflowBlock Orchestration**: Executes complex multi-step, multi-domain workflows
- **Domain-Aware Execution**: Handles both VerifiableDomain and ServiceDomain operations

All runtime operations maintain consistency with the Resource Model's content-addressed, SSZ-serialized architecture.

## Core Components

### TEL Interpreter

The Temporal Effect Logic interpreter orchestrates all Resource operations:

```rust
use causality_runtime::{TelInterpreter, RuntimeConfig};

let config = RuntimeConfig {
    max_execution_steps: 10000,
    enable_tracing: true,
    domain_timeout: Duration::from_secs(30),
};

let interpreter = TelInterpreter::new(config);
let result = interpreter.evaluate_resource_logic(&resource, &context).await?;
```

### Host Functions

Bridge between Lisp expressions and runtime capabilities:

```rust
use causality_runtime::host_functions::{HostFunctionRegistry, HostFunction};

let mut registry = HostFunctionRegistry::new();

// Register custom host function
registry.register("get-current-time", HostFunction::new(|_args| {
    Ok(ValueExpr::Integer(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64))
}));

// Use in TEL interpreter
let interpreter = TelInterpreter::with_host_functions(registry);
```

### State Management

Manages Resource states and transitions:

```rust
use causality_runtime::state::{StateManager, StateTransition};

let mut state_manager = StateManager::new();

// Apply state transition
let transition = StateTransition::new(
    resource_id,
    old_state,
    new_state,
    effect_id
);

state_manager.apply_transition(transition).await?;
let current_state = state_manager.get_resource_state(&resource_id).await?;
```

### ProcessDataflowBlock Orchestration

Executes complex multi-step workflows:

```rust
use causality_runtime::dataflow::{DataflowOrchestrator, DataflowInstance};

let orchestrator = DataflowOrchestrator::new();

// Create dataflow instance
let instance = orchestrator.create_instance(&dataflow_block, &initial_context).await?;

// Execute dataflow step
let completed_effect = Effect::new(/* ... */);
let result = orchestrator.execute_step(&instance, &completed_effect).await?;

// Check if dataflow is complete
if result.is_complete {
    println!("Dataflow completed successfully");
}
```

### Domain-Aware Execution

Handles operations across different domain types:

```rust
use causality_runtime::domain::{DomainExecutor, DomainType};

let executor = DomainExecutor::new();

// Execute in VerifiableDomain
let verifiable_result = executor.execute_in_domain(
    &DomainType::Verifiable,
    &resource_operation
).await?;

// Execute in ServiceDomain
let service_result = executor.execute_in_domain(
    &DomainType::Service,
    &external_service_call
).await?;
```

## Runtime Context

### Execution Context

Provides context for Resource logic evaluation:

```rust
use causality_runtime::context::{RuntimeContext, ContextBuilder};

let context = ContextBuilder::new()
    .with_resource_state(&resource_id, &current_state)
    .with_domain_config(&domain_config)
    .with_capability_grants(&user_capabilities)
    .build();

let result = interpreter.evaluate_with_context(&expression, &context).await?;
```

### Resource Access

Provides access to Resource states during evaluation:

```rust
use causality_runtime::access::{ResourceAccessor, AccessPermissions};

let accessor = ResourceAccessor::new(state_manager);

// Get resource field with permissions check
let balance = accessor.get_resource_field(
    &resource_id,
    "balance",
    &AccessPermissions::Read
).await?;
```

## Host Function Library

### Core Functions

Built-in host functions for common operations:

- `get-current-timestamp`: Get current system timestamp
- `get-resource-field`: Access Resource field values
- `compute-hash`: Compute cryptographic hashes
- `verify-signature`: Verify digital signatures
- `emit-effect`: Emit new Effects for processing

### Custom Functions

Register custom host functions for domain-specific operations:

```rust
use causality_runtime::host_functions::*;

// Register domain-specific function
registry.register("validate-token-transfer", HostFunction::new(|args| {
    let from_balance = args[0].as_integer()?;
    let transfer_amount = args[1].as_integer()?;
    
    Ok(ValueExpr::Boolean(from_balance >= transfer_amount))
}));
```

## Error Handling

Comprehensive error handling throughout the runtime:

```rust
use causality_runtime::error::{RuntimeError, ErrorContext};

match interpreter.evaluate(&expression, &context).await {
    Ok(result) => process_result(result),
    Err(RuntimeError::ExecutionTimeout) => {
        eprintln!("Expression evaluation timed out");
    }
    Err(RuntimeError::ResourceNotFound { resource_id }) => {
        eprintln!("Resource {} not found", resource_id);
    }
    Err(RuntimeError::PermissionDenied { operation }) => {
        eprintln!("Permission denied for operation: {}", operation);
    }
}
```

## Performance Optimization

### Execution Optimization

- **Expression Caching**: Cache compiled expressions for reuse
- **State Caching**: Cache frequently accessed Resource states
- **Parallel Execution**: Execute independent operations in parallel
- **Lazy Evaluation**: Defer expensive operations until needed

### Resource Management

- **Memory Pooling**: Reuse memory allocations for better performance
- **Connection Pooling**: Pool connections to external services
- **Batch Operations**: Batch multiple operations for efficiency

## Feature Flags

- **default**: Standard runtime features
- **tracing**: Execution tracing and debugging
- **metrics**: Performance metrics collection
- **async**: Asynchronous execution support
- **persistence**: State persistence capabilities

## Module Structure

```
src/
├── lib.rs                    # Main library interface
├── interpreter.rs            # TEL interpreter implementation
├── host_functions.rs         # Host function registry and implementations
├── state.rs                  # State management
├── dataflow.rs               # ProcessDataflowBlock orchestration
├── domain.rs                 # Domain-aware execution
├── context.rs                # Runtime context management
├── access.rs                 # Resource access control
├── error.rs                  # Error handling
└── config.rs                 # Runtime configuration
```

This crate provides the execution environment for the Causality Resource Model, enabling deterministic and verifiable execution of Resource logic while maintaining the content-addressed and SSZ-serialized properties of the framework.
