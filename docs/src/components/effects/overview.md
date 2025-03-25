<!-- Overview of the effect system -->
<!-- Original file: docs/src/effect_system.md -->

# Effect System API Documentation

This document provides detailed information about Causality's effect system, including proper usage patterns and examples.

## Overview

The effect system is a core component of Causality that provides a structured way to perform operations with side effects. It ensures these operations can be:

- Properly validated
- Executed in the correct boundary (inside or outside the system)
- Tracked and monitored
- Composed into higher-level operations

The effect system implements algebraic effects, a powerful concept from functional programming that allows structured handling of side effects. In Causality, effects represent discrete operations that may cause state changes within the system or across system boundaries.

## Algebraic Effects Foundation

Algebraic effects in Causality have the following characteristics:

1. **Closed Effect Set**: Effects are represented through a well-defined set of operations
2. **Static Handler Resolution**: Effect handlers use static dispatch with composition capabilities
3. **Resource-Scoped Concurrency**: Resource locks are managed explicitly with deterministic wait queues
4. **RISC-V Compilation**: The system can compile to RISC-V code compatible with ZK VM execution

This algebraic foundation enables powerful compositional programming while maintaining strong reasoning about effects and their boundaries.

## Key Components

### Effect Trait

The `Effect` trait defines the core interface for all effects:

```rust
pub trait Effect: Send + Sync + fmt::Debug {
    fn id(&self) -> &EffectId;
    fn name(&self) -> &str;
    fn display_name(&self) -> String;
    fn description(&self) -> String;
    fn execute(&self, context: &EffectContext) -> Result<EffectOutcome>;
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool;
    fn preferred_boundary(&self) -> ExecutionBoundary;
    fn display_parameters(&self) -> HashMap<String, String>;
    fn as_any(&self) -> &dyn std::any::Any;
    
    // Methods with default implementations
    fn requires_authorization(&self) -> bool { true }
    fn required_capabilities(&self) -> Vec<(ResourceId, Right)> { Vec::new() }
    fn dependencies(&self) -> Vec<EffectId> { Vec::new() }
    fn fact_dependencies(&self) -> Vec<FactDependency> { Vec::new() }
    fn fact_snapshot(&self) -> Option<FactSnapshot> { None }
    fn validate_fact_dependencies(&self) -> Result<()> { Ok(()) }
}
```

This trait defines the fundamental interface for all effects in the system. Each method serves a specific purpose:
- `id()`: Returns a unique identifier for this specific effect instance
- `name()` and `display_name()`: Provide human-readable identifiers for the effect
- `description()`: Returns a detailed description of what the effect does
- `execute()`: Performs the actual effect operation with the given context
- `can_execute_in()` and `preferred_boundary()`: Define execution boundary constraints
- `display_parameters()`: Returns a human-readable representation of effect parameters
- `as_any()`: Enables downcasting to concrete effect types when needed
- Other methods provide capabilities, dependencies, and fact management information

### AsyncEffect Trait

For asynchronous execution, effects can implement the `AsyncEffect` trait:

```rust
#[async_trait]
pub trait AsyncEffect: Effect {
    async fn execute_async(&self, context: &EffectContext) -> EffectResult<EffectOutcome>;
}
```

This extension trait enables asynchronous execution of effects. It builds on the base `Effect` trait and adds an async execution method. This separation allows for a clear distinction between synchronous and asynchronous effects while maintaining object safety.

### EffectOutcome

All effects produce an `EffectOutcome` structure:

```rust
pub struct EffectOutcome {
    pub id: EffectId,
    pub success: bool,
    pub data: HashMap<String, String>,
    pub error: Option<String>,
    pub execution_id: Option<uuid::Uuid>,
    pub resource_changes: Vec<ResourceChange>,
    pub metadata: HashMap<String, String>,
}
```

The `EffectOutcome` represents the result of executing an effect. It contains key information:
- `id`: The effect ID that produced this outcome
- `success`: Whether the effect succeeded
- `data`: Key-value data produced by the effect
- `error`: Optional error message if the effect failed
- `execution_id`: Optional ID of the execution context
- `resource_changes`: Record of all resources modified by the effect
- `metadata`: Additional metadata about the execution

### EffectContext

The execution context for effects:

```rust
pub struct EffectContext {
    pub started_at: DateTime<Utc>,
    pub caller: Option<String>,
    pub params: HashMap<String, String>,
}
```

The `EffectContext` provides information about when and how an effect is being executed. It includes:
- `started_at`: The timestamp when the effect execution started
- `caller`: Optional identifier of who/what is executing the effect
- `params`: Key-value parameters for the effect execution

## Execution Boundaries

Effects specify their execution boundary to control where they can execute:

- `ExecutionBoundary::InsideSystem`: The effect runs within the Causality system
- `ExecutionBoundary::OutsideSystem`: The effect runs outside the system boundary
- `ExecutionBoundary::External`: The effect runs in an external system

Additionally, effects can cross chain boundaries, including:
- EVM (Ethereum)
- SVM (Solana)
- MoveVM
- CosmWasm
- Local
- Custom

When effects cross system boundaries, they are wrapped in a boundary crossing structure that includes context information, payload data, authentication, timestamp, and origin/destination information.

## Implementation Patterns

### Creating a Basic Effect

Here's how to implement a simple effect:

```rust
#[derive(Debug)]
pub struct MyEffect {
    id: EffectId,
    param1: String,
    param2: i32,
    boundary: ExecutionBoundary,
}

impl MyEffect {
    pub fn new(param1: String, param2: i32) -> Self {
        Self {
            id: EffectId::new_unique(),
            param1,
            param2,
            boundary: ExecutionBoundary::InsideSystem,
        }
    }
}

impl Effect for MyEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn name(&self) -> &str {
        "my_effect"
    }
    
    fn display_name(&self) -> String {
        format!("My Effect: {}", self.param1)
    }
    
    fn description(&self) -> String {
        "An example effect implementation".to_string()
    }
    
    fn execute(&self, context: &EffectContext) -> Result<EffectOutcome> {
        // Perform the effect operation
        let result = format!("Executed with param1={} and param2={}", 
                            self.param1, self.param2);
        
        // Create a successful outcome
        Ok(EffectOutcome::success(self.id.clone())
            .with_data("result", result))
    }
    
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        boundary == self.boundary
    }
    
    fn preferred_boundary(&self) -> ExecutionBoundary {
        self.boundary
    }
    
    fn display_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("param1".to_string(), self.param1.clone());
        params.insert("param2".to_string(), self.param2.to_string());
        params
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
```

This example demonstrates a basic effect implementation:
1. The effect struct stores all necessary state: a unique ID, parameters, and boundary information
2. A constructor method creates new effect instances with the required parameters
3. The `Effect` trait implementation provides all the necessary methods
4. The `execute` method implements the core logic, performing the effect and returning an outcome
5. Boundary methods control where this effect can execute
6. Utility methods provide information about the effect's parameters and type

### Implementing AsyncEffect

To make your effect asynchronous:

```rust
#[async_trait]
impl AsyncEffect for MyEffect {
    async fn execute_async(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Perform async operations
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Create a successful outcome
        Ok(EffectOutcome::success(self.id.clone())
            .with_data("result", format!("Async execution with param1={}", self.param1)))
    }
}
```

This async implementation builds on the previous example:
1. It uses the `#[async_trait]` attribute to support async methods in traits
2. The `execute_async` method performs asynchronous operations using `async/await`
3. The implementation can leverage Tokio or other async runtimes for operations
4. It returns the same `EffectOutcome` type as the synchronous version, but allows asynchronous execution

### Using Effects with the EffectManager

The `EffectManager` provides a high-level API for registering and executing effects:

```rust
// Create a new effect manager
let mut manager = EffectManager::new();

// Register an effect
let effect = Arc::new(MyEffect::new("test".to_string(), 42));
manager.register_effect(effect);

// Create an execution context
let context = EffectContext::new()
    .with_param("caller", "user123");

// Execute the effect
let outcome = manager.execute_effect("my_effect", context).await?;

// Check the outcome
if outcome.success {
    println!("Effect executed successfully!");
    println!("Result: {}", outcome.data.get("result").unwrap_or(&"<no result>".to_string()));
} else {
    println!("Effect failed: {}", outcome.error.unwrap_or_else(|| "Unknown error".to_string()));
}
```

This example shows the high-level effect execution pattern:
1. Create an `EffectManager` instance to manage effect registration and execution
2. Register concrete effect instances with the manager
3. Create an `EffectContext` with necessary parameters
4. Execute the effect by name using the manager's API
5. Process the outcome by checking success status and extracting data

### Using the Interpreter

The `Interpreter` provides lower-level control over effect execution:

```rust
// Create a handler
let handler = MyHandler::new();

// Create an interpreter with the handler
let interpreter = Interpreter::new(handler);

// Create an effect
let effect = MyEffect::new("test".to_string(), 42);

// Execute synchronously
let sync_result = interpreter.execute(&effect)?;

// Execute asynchronously (if the effect implements AsyncEffect)
let async_result = interpreter.execute_async(&effect).await?;
```

The Interpreter pattern provides more direct control over effect execution:
1. Create a handler that implements the `EffectHandler` trait
2. Construct an `Interpreter` with the handler
3. Create effect instances directly
4. Execute effects synchronously or asynchronously through the interpreter
5. Process the results as needed

## Error Handling

Effects can return various error types through `EffectError`:

```rust
pub enum EffectError {
    AuthenticationFailed(String),
    AuthorizationFailed(String),
    CapabilityError(String),
    ResourceError(String),
    ExecutionError(String),
    BoundaryError(BoundaryError),
    InvalidParameter(String),
    PermissionDenied(String),
    InvalidState(String),
    NotFound(String),
    Timeout(String),
    UnsupportedOperation(String),
    ExternalError(String),
    NotImplemented,
}
```

This error enum provides structured error reporting:
- Authentication and authorization failures
- Capability and resource access errors
- Execution and boundary errors
- Parameter validation and state errors
- External system errors and timeouts
- Each variant can include a descriptive message

## Effect Handlers

Effect handlers determine how effects are executed. The core handler interface is:

```rust
pub trait EffectHandler: Send + Sync {
    fn execution_boundary(&self) -> ExecutionBoundary;
    
    fn can_handle(&self, effect: &dyn Effect) -> bool {
        effect.can_execute_in(self.execution_boundary())
    }
    
    fn handle(&self, effect: &dyn Effect, context: &EffectContext) -> EffectResult<EffectOutcome>;
    
    #[cfg(feature = "async")]
    async fn handle_async(&self, effect: &dyn Effect, context: &EffectContext) -> EffectResult<EffectOutcome>;
}
```

Effect handlers can be composed using utility functions that combine multiple handlers:

```rust
// Create a composite handler that tries primary first, then falls back to secondary
pub fn compose_handlers(primary: Arc<dyn EffectHandler>, secondary: Arc<dyn EffectHandler>) -> Arc<dyn EffectHandler> {
    Arc::new(CompositeHandler::new(primary, secondary))
}

// Create a handler that enforces a specific boundary
pub fn boundary_handler(boundary: ExecutionBoundary, inner: Arc<dyn EffectHandler>) -> Arc<dyn EffectHandler> {
    Arc::new(BoundaryHandler::new(boundary, inner))
}
```

## RISC-V Integration

A unique aspect of Causality's effect system is its ability to compile to RISC-V code for zero-knowledge VM execution. This enables cryptographic proofs of effect execution:

```rust
// Compile an effect to RISC-V bytecode
pub fn compile_effect_to_risc_v(
    effect: &dyn Effect,
    config: &CompilationConfig
) -> Result<RiscVProgram> {
    let mut compiler = RiscVCompiler::new(config);
    compiler.compile_effect(effect)
}

// Execute effect in a ZK VM and generate proof
pub async fn prove_effect_execution(
    effect: &dyn Effect,
    context: &EffectContext,
    zk_vm: &dyn ZkVm
) -> Result<ZkProof> {
    // Compile the effect
    let program = compile_effect_to_risc_v(effect, &zk_vm.get_compilation_config())?;
    
    // Execute in the ZK VM
    let execution = zk_vm.execute(&program, context).await?;
    
    // Generate the proof
    zk_vm.generate_proof(&program, &execution).await
}
```

The RISC-V integration enables:
1. Compiling effects to RISC-V instructions
2. Executing those instructions in a zero-knowledge virtual machine
3. Generating cryptographic proofs of effect execution
4. Verifying those proofs to confirm effect outcomes
5. Cross-chain verification of effect execution

## Best Practices

1. **Keep Effects Focused**: Each effect should do one thing well
2. **Use Descriptive Names**: Make effect names and descriptions clear
3. **Proper Error Handling**: Return meaningful error messages
4. **Resource Changes**: Track all resource changes in the outcome
5. **Boundary Awareness**: Be explicit about which boundary an effect can run in
6. **Capabilities**: Specify required capabilities for proper authorization
7. **Reuse Common Patterns**: Use effect templates for common operations

## Examples

### Resource Creation Effect

```rust
#[derive(Debug)]
pub struct CreateResourceEffect {
    id: EffectId,
    resource_id: ResourceId,
    resource_type: String,
    owner: String,
    data: Vec<u8>,
}

impl CreateResourceEffect {
    pub fn new(
        resource_id: ResourceId,
        resource_type: String,
        owner: String,
        data: Vec<u8>,
    ) -> Self {
        Self {
            id: EffectId::new_unique(),
            resource_id,
            resource_type,
            owner,
            data,
        }
    }
}

impl Effect for CreateResourceEffect {
    // Implementation details...
    
    fn execute(&self, context: &EffectContext) -> Result<EffectOutcome> {
        // Create the resource...
        
        // Return outcome with resource change
        Ok(EffectOutcome::success(self.id.clone())
            .with_resource_change(ResourceChange {
                resource_id: self.resource_id.clone(),
                change_type: ResourceChangeType::Created,
                previous_state_hash: None,
                new_state_hash: "hash_of_new_state".to_string(),
            }))
    }
}
```

This example demonstrates a resource creation effect:
1. It encapsulates all necessary data to create a resource: ID, type, owner, and data
2. The constructor provides a clean API for creating effect instances
3. The `execute` method creates the resource and records the change in the outcome
4. Resource changes are tracked explicitly for auditing and consistency

### Composite Effect

```rust
#[derive(Debug)]
pub struct CompositeEffect {
    id: EffectId,
    effects: Vec<Box<dyn Effect>>,
}

impl CompositeEffect {
    pub fn new(effects: Vec<Box<dyn Effect>>) -> Self {
        Self {
            id: EffectId::new_unique(),
            effects,
        }
    }
}

impl Effect for CompositeEffect {
    // Implementation details...
    
    fn execute(&self, context: &EffectContext) -> Result<EffectOutcome> {
        let mut outcome = EffectOutcome::success(self.id.clone());
        
        // Execute each sub-effect
        for effect in &self.effects {
            match effect.execute(context) {
                Ok(sub_outcome) => {
                    // Merge resource changes
                    for change in sub_outcome.resource_changes {
                        outcome = outcome.with_resource_change(change);
                    }
                    
                    // Merge data
                    for (k, v) in sub_outcome.data {
                        outcome.data.insert(k, v);
                    }
                },
                Err(e) => {
                    return Ok(EffectOutcome::failure(self.id.clone(), 
                                                   format!("Sub-effect failed: {}", e)));
                }
            }
        }
        
        Ok(outcome)
    }
}
```

This example shows how effects can be composed:
1. The composite effect contains a collection of sub-effects
2. When executed, it runs each sub-effect in sequence
3. Results from sub-effects are merged into the composite outcome
4. If any sub-effect fails, the entire composite effect fails
5. This pattern enables building complex operations from simpler ones

### External API Effect

```rust
#[derive(Debug)]
pub struct ExternalAPIEffect {
    id: EffectId,
    endpoint: String,
    payload: Vec<u8>,
    timeout: Duration,
}

impl ExternalAPIEffect {
    pub fn new(endpoint: String, payload: Vec<u8>, timeout: Duration) -> Self {
        Self {
            id: EffectId::new_unique(),
            endpoint,
            payload,
            timeout,
        }
    }
}

impl Effect for ExternalAPIEffect {
    // Implementation details...
    
    fn preferred_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::OutsideSystem
    }
    
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        boundary == ExecutionBoundary::OutsideSystem
    }
}

#[async_trait]
impl AsyncEffect for ExternalAPIEffect {
    async fn execute_async(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Create HTTP client
        let client = reqwest::Client::new();
        
        // Set up request with timeout
        let request = client
            .post(&self.endpoint)
            .body(self.payload.clone())
            .timeout(self.timeout);
            
        // Execute the request
        match request.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let body = response.bytes().await?;
                    Ok(EffectOutcome::success(self.id.clone())
                        .with_data("status_code", response.status().as_u16().to_string())
                        .with_data("response_body", base64::encode(&body)))
                } else {
                    Ok(EffectOutcome::failure(
                        self.id.clone(),
                        format!("API returned error status: {}", response.status())
                    ))
                }
            },
            Err(e) => {
                Ok(EffectOutcome::failure(
                    self.id.clone(),
                    format!("API request failed: {}", e)
                ))
            }
        }
    }
}
```

This example demonstrates an effect that interacts with external systems:
1. It's designed to make API calls outside the Causality system
2. It specifies that it can only execute in the OutsideSystem boundary
3. It implements the AsyncEffect trait for asynchronous execution
4. The implementation handles timeouts, errors, and response processing
5. The outcome includes detailed information about the API call result

## Program Account Effects Integration

Program accounts can leverage the effect system through adapters, which:

1. Map program accounts to effects
2. Manage effect capabilities for accounts
3. Filter available effects based on account types
4. Execute effects with the appropriate context
5. Provide a consistent interface for UI integration

Example of using effects with program accounts:

```rust
// Create an effect adapter for program accounts
let effect_adapter = ProgramAccountEffectAdapter::new();

// Register an account type with available effects
effect_adapter.register_account_type(
    "asset_account",
    vec!["transfer", "deposit", "withdraw"]
);

// Set up parameters for execution
let mut params = HashMap::new();
params.insert("destination".to_string(), "account123".to_string());
params.insert("amount".to_string(), "100".to_string());

// Execute an effect on a program account
let outcome = effect_adapter.execute_effect(
    "my_account",
    "transfer",
    params
).await?;
```

This pattern enables:
1. Decoupling program account logic from effect implementation
2. Enforcing capability requirements for different account types
3. Providing a consistent API for effect execution
4. Managing execution boundaries appropriately

## Summary

The effect system provides a structured way to perform operations with side effects in Causality. By implementing the `Effect` trait (and optionally `AsyncEffect`), components can integrate with the system and benefit from built-in capabilities like boundary checking, authorization, and resource change tracking.

The system's algebraic foundation and RISC-V integration provide powerful capabilities for provable computation, while the clear separation of synchronous and asynchronous execution paths ensures good performance and object safety.
