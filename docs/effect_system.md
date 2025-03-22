# Effect System

The Effect System in Causality is a powerful abstraction that implements algebraic effects for representing state changes and operations across system boundaries. This document explains how the effect system works, compares it to other effect systems, and demonstrates its practical usage.

## Core Concepts

### Algebraic Effects

Algebraic effects are a way to model computational effects like I/O, state manipulation, exceptions, and non-determinism in a compositional manner. The key aspects of algebraic effects are:

1. **Effects are operations**: They represent actions a program can perform that aren't pure computations
2. **Effects are delimited**: The scope of an effect is clearly bounded
3. **Effects are handled**: Handlers determine what happens when an effect is invoked
4. **Effects support continuation passing**: Control flow is managed explicitly

Causality implements a structured algebraic effect system with the following characteristics:

- **Closed Effect Set**: Effects are represented as a sealed enumeration with explicit continuation types
- **Static Handler Resolution**: Effect handlers use static dispatch with composition capabilities
- **Resource-Scoped Concurrency**: Resource locks are managed explicitly with deterministic wait queues
- **RISC-V Compilation**: The system compiles to RISC-V code compatible with ZK VM execution

### Effects

An effect in Causality is a discrete operation that may cause state changes within the system or across system boundaries. Effects:

- Have a name and description
- Require specific capabilities to execute
- Can execute at specific system boundaries
- Produce an outcome with results or errors
- May cause resource changes

### Execution Boundaries

The system defines two primary execution boundaries:

- **Inside**: Effects execute within the Causality system
- **Outside**: Effects execute outside the system (e.g., on a blockchain or external service)

Additionally, effects can cross chain boundaries, which include:
- EVM (Ethereum)
- SVM (Solana)
- MoveVM
- CosmWasm
- Local
- Custom

### Effect Context

Each effect executes within a context that provides:

- Execution ID for tracing
- Boundary information
- Invoker address
- Capabilities for authorization
- Parameters for configuration

### Boundary Crossing

When effects cross system boundaries, they are wrapped in a `BoundaryCrossing` that includes:

- Context information
- Payload data
- Authentication
- Timestamp
- Origin/destination information

## Algebraic Effects Implementation

Causality implements algebraic effects as a structured system with explicit continuations and handlers. This section details the implementation approach and how it enables powerful compositional programming.

### Effect Representation

In Causality, effects are represented as values that conform to the `Effect` trait:

```rust
pub trait Effect: Send + Sync {
    /// The output type of this effect
    type Output;
    
    /// Get the type of this effect
    fn get_type(&self) -> EffectType;
    
    /// Get a debug representation of this effect
    fn as_debug(&self) -> &dyn std::fmt::Debug;
    
    /// Clone this effect
    fn clone_box(&self) -> Box<dyn Effect<Output = Self::Output>>;
    
    /// Get the resources affected by this effect
    fn resources(&self) -> Vec<ResourceId>;
    
    /// Get the domains involved in this effect
    fn domains(&self) -> Vec<DomainId>;
    
    /// Execute this effect using the given handler
    fn execute(self, handler: &dyn EffectHandler) -> Self::Output;
    
    /// Get the fact dependencies for this effect
    fn fact_dependencies(&self) -> Vec<FactDependency> {
        Vec::new()
    }
    
    /// Get the fact snapshot for this effect
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        None
    }
}
```

### Continuations

Continuations represent "what happens next" after an effect completes. They take the result of an effect and transform it into the desired output:

```rust
pub trait Continuation<I, O>: Debug + Send + 'static 
where
    I: Debug + Send + 'static,
    O: Debug + Send + 'static,
{
    /// Apply this continuation to the input value
    fn apply(self: Box<Self>, input: I) -> O;
    
    /// Compute a content hash for this continuation
    fn content_hash(&self) -> [u8; 32];
}
```

Continuations can be chained together to create complex computations that interleave effects with pure code:

```rust
// Chain two continuations together
pub fn and_then<A, B, C>(
    first: Box<dyn Continuation<A, B>>,
    second: Box<dyn Continuation<B, C>>,
) -> Box<dyn Continuation<A, C>> {
    Box::new(ChainContinuation::new(first, second))
}

// Map a value through a function
pub fn map<I, O, F>(f: F) -> Box<dyn Continuation<I, O>>
where
    F: FnOnce(I) -> O + Send + 'static,
{
    Box::new(FnContinuation::new(f))
}
```

### Effect Handlers

Effect handlers determine how effects are executed. The core handler interface is:

```rust
pub trait EffectHandler: Send + Sync {
    fn handle_deposit(&self, resource: ResourceId, domain: DomainId, amount: TokenAmount) -> bool;
    
    fn handle_withdrawal(&self, resource: ResourceId, domain: DomainId, amount: TokenAmount) -> bool;
    
    fn handle_observation(&self, resource: ResourceId, domain: DomainId) -> TokenAmount;
    
    // Other handler methods...
}
```

Handlers can be composed using helper functions:

```rust
// Compose two handlers, with the second used as fallback
pub fn compose<A, B>(primary: A, fallback: B) -> impl EffectHandler
where
    A: EffectHandler,
    B: EffectHandler,
{
    CompositeHandler { primary, fallback }
}

// Create a shared handler from a handler instance
pub fn shared<H: EffectHandler>(handler: H) -> SharedHandler {
    SharedHandler::new(handler)
}
```

### ZK VM Compilation

A unique aspect of Causality's effect system is its ability to compile to RISC-V code for zero-knowledge VM execution. This enables cryptographic proofs of effect execution:

```rust
pub trait RiscVContinuation: Debug + Send + 'static {
    /// Generate RISC-V code for this continuation
    fn to_risc_v(&self, writer: &mut dyn RiscVWriter) -> Result<()>;
}
```

## RISC-V Integration Details

The connection between Causality's algebraic effects and RISC-V compilation is a key innovation that enables zero-knowledge provable computation. This section explains how effects are translated to RISC-V code and executed in a ZK VM.

### Compilation Pipeline

The compilation process follows these steps:

1. **Effect Decomposition**: Each effect is broken down into its core operations and continuations
2. **Continuation Analysis**: The continuations are analyzed for data flow and dependencies
3. **Resource Tracking**: Resources affected by the effect are tracked for concurrency control
4. **RISC-V Code Generation**: The effect and its continuations are translated to RISC-V instructions
5. **Optimization Passes**: The generated code undergoes optimization passes specific to ZK VMs
6. **Boundary Crossing Instrumentation**: Special instrumentation is added for boundary crossings
7. **Proof-Specific Annotations**: Annotations are added to facilitate zero-knowledge proof generation

```rust
// A simplified version of the compilation pipeline
pub fn compile_effect_to_risc_v<R>(
    effect: &dyn Effect<Output = R>,
    config: &CompilationConfig,
) -> Result<RiscVProgram> {
    // Create a program builder
    let mut builder = RiscVProgramBuilder::new(config.clone());
    
    // Generate the main section
    builder.create_section(".text")?;
    builder.set_entry_point("effect_main")?;
    
    // Write the effect execution code
    let writer = builder.get_writer();
    effect.to_risc_v(writer)?;
    
    // Apply optimization passes
    for optimizer in &config.optimizers {
        optimizer.optimize(&mut builder)?;
    }
    
    // Build the final program
    builder.build()
}
```

### Effect Representation in RISC-V

Effects are represented in RISC-V code as a series of instructions that:

1. **Set Up Context**: Load the effect context and parameters into registers
2. **Check Capabilities**: Verify that required capabilities are available
3. **Perform Operation**: Execute the core effect logic
4. **Handle Boundary Crossings**: Marshal data across system boundaries if needed
5. **Apply Continuation**: Call the continuation with the effect result
6. **Clean Up**: Release resources and restore state

For example, a deposit effect might be compiled to RISC-V code like this:

```assembly
# Deposit effect implementation
deposit_effect:
    # Load the resource ID into a0-a1
    ld a0, 0(sp)
    ld a1, 8(sp)
    
    # Load the domain ID into a2-a3
    ld a2, 16(sp)
    ld a3, 24(sp)
    
    # Load the amount into a4
    ld a4, 32(sp)
    
    # Check capabilities
    jal ra, check_deposit_capability
    beqz a0, capability_error
    
    # Execute the deposit
    jal ra, perform_deposit
    
    # Store the result
    sd a0, 40(sp)
    
    # Apply the continuation
    ld a0, 40(sp)  # Load the result
    jal ra, continuation_entry
    
    # Return
    ret

capability_error:
    # Handle capability error
    li a0, -1
    ret
```

### Register Allocation and Memory Management

The RISC-V code generator must carefully manage registers and memory:

1. **Register Allocation**: RISC-V has a limited set of registers (x0-x31), with specific conventions for function calls
2. **Stack Management**: Complex effects and continuations require stack frames for local variables and saved registers
3. **Heap Allocation**: Dynamic memory allocation is tightly controlled for deterministic execution
4. **Resource Tracking**: Resources used by effects are explicitly tracked in memory

```rust
pub struct RiscVWriter {
    pub register_allocator: RegisterAllocator,
    pub stack_manager: StackManager,
    pub heap_manager: Option<HeapManager>,
}

impl RiscVWriter {
    // Allocate a register for a value
    pub fn allocate_register(&mut self, value_type: ValueType) -> Result<Register> {
        self.register_allocator.allocate(value_type)
    }
    
    // Reserve stack space for local variables
    pub fn reserve_stack(&mut self, size: usize) -> Result<StackSlot> {
        self.stack_manager.reserve(size)
    }
}
```

### Boundary Crossings in RISC-V

When effects cross system boundaries, special handling is required:

1. **Marshaling**: Data is serialized and deserialized at boundaries
2. **Validation**: Input and output data are validated for correctness
3. **External Calls**: Calls to external systems are abstracted through a special interface
4. **State Verification**: State changes are verified cryptographically
5. **Proof Generation**: Special annotations facilitate proof generation for boundary crossings

```rust
// A boundary crossing in RISC-V
pub fn compile_boundary_crossing(
    writer: &mut RiscVWriter,
    crossing: &BoundaryCrossing,
) -> Result<()> {
    // Write a label for this crossing
    writer.write_label("boundary_crossing")?;
    
    // Serialize crossing data to a buffer
    writer.write_comment("Serialize boundary crossing data")?;
    // ... serialization instructions ...
    
    // Call the boundary service
    writer.write_comment("Call boundary service")?;
    writer.write_instruction(&RiscVInstruction::Jal {
        rd: 1, // ra (return address)
        offset: writer.get_label_offset("boundary_service_call")?,
    })?;
    
    // Deserialize the result
    writer.write_comment("Deserialize result")?;
    // ... deserialization instructions ...
    
    // Return with the result in a0-a1
    writer.write_instruction(&RiscVInstruction::Jalr {
        rd: 0, // Don't save return address
        rs1: 1, // ra (return address)
        offset: 0,
    })?;
    
    Ok(())
}
```

### ZK Proof Generation

The compiled RISC-V code is compatible with zero-knowledge virtual machines like Risc0, Succinct, or other ZK VM implementations. These VMs execute the code and generate cryptographic proofs that:

1. **Verify Execution**: Prove that the code executed correctly according to the RISC-V specification
2. **Validate State Transitions**: Prove that state changes follow the rules defined by the effect
3. **Ensure Capability Compliance**: Prove that capability checks were enforced
4. **Maintain Confidentiality**: Optionally hide specific inputs and intermediate values
5. **Prove Resource Conservation**: Ensure that resources are properly managed

```rust
// Generate a ZK proof for an effect execution
pub async fn prove_effect_execution<R>(
    effect: &dyn Effect<Output = R>,
    handler: &dyn EffectHandler,
    zk_vm: &dyn ZkVm,
) -> Result<ZkProof> {
    // Compile the effect to RISC-V
    let program = compile_effect_to_risc_v(effect, &zk_vm.get_compilation_config())?;
    
    // Execute the program in the ZK VM
    let execution_result = zk_vm.execute(&program).await?;
    
    // Generate the proof
    let proof = zk_vm.generate_proof(&program, &execution_result).await?;
    
    Ok(proof)
}
```

### Integration with Causality's Runtime

The RISC-V integration connects with the rest of Causality through:

1. **Runtime Bridging**: The runtime system bridges between native Rust and RISC-V execution
2. **Effect Handlers**: Effect handlers are implemented in both native Rust and RISC-V
3. **Proof Verification**: The system verifies proofs generated by the ZK VM
4. **Logging and Auditing**: All RISC-V executions are logged in the unified log
5. **Time Map Integration**: RISC-V executions are tracked in the time map for causal ordering

```rust
// Bridge between native and RISC-V execution
pub struct RuntimeBridge {
    zk_vm: Arc<dyn ZkVm>,
    effect_registry: Arc<EffectRegistry>,
    proof_verifier: Arc<ProofVerifier>,
}

impl RuntimeBridge {
    // Execute an effect either natively or in the ZK VM
    pub async fn execute_effect<R>(
        &self,
        effect: &dyn Effect<Output = R>,
        execution_mode: ExecutionMode,
    ) -> Result<ExecutionResult<R>> {
        match execution_mode {
            ExecutionMode::Native => {
                // Execute natively
                let handler = self.effect_registry.get_handler(effect.get_type())?;
                let result = effect.execute(handler);
                Ok(ExecutionResult::Native(result))
            }
            ExecutionMode::ZkVm => {
                // Compile and execute in ZK VM
                let program = compile_effect_to_risc_v(effect, &self.zk_vm.get_compilation_config())?;
                let execution = self.zk_vm.execute(&program).await?;
                let proof = self.zk_vm.generate_proof(&program, &execution).await?;
                
                // Verify the proof
                self.proof_verifier.verify(&proof)?;
                
                // Extract the result
                let result = extract_result_from_execution::<R>(&execution)?;
                
                Ok(ExecutionResult::Proved(result, proof))
            }
        }
    }
}
```

## Comparison with Haskell's Polysemy

Haskell's Polysemy is a popular effect system in the functional programming world. Here's how Causality compares:

| Feature | Causality | Polysemy |
|---------|-----------|----------|
| **Effect Definition** | Closed set via sealed traits | Open via type-level lists |
| **Effect Dispatch** | Static dispatch | Dynamic via higher-rank types |
| **Effect Composition** | Explicit via continuation chains | Implicit via monad transformers |
| **Resource Management** | Explicit resource locks | Via higher-order functions |
| **Compilation** | RISC-V + native Rust | Haskell native |
| **Concurrency Model** | Resource-scoped with wait queues | Pure with explicit threading |
| **Error Handling** | Typed results with explicit errors | Either or exceptions |
| **Optimization** | Static optimization + RISC-V specialization | GHC optimization |

### Key Differences

1. **Open vs. Closed Effects**
   - Polysemy allows for an open set of effects via type-level programming
   - Causality uses a closed set for better static analysis and RISC-V compilation

2. **Effect Handling**
   - Polysemy uses higher-rank polymorphism for effect elimination
   - Causality uses explicit handlers with static dispatch for better performance

3. **Continuation Representation**
   - Polysemy uses implicit continuations via the underlying monad
   - Causality uses explicit continuation objects for better control over memory layout

4. **Compilation Strategy**
   - Polysemy compiles to native Haskell with GHC optimizations
   - Causality compiles to both native Rust and RISC-V for ZK VM execution

5. **Concurrency Model**
   - Polysemy provides effect interpreters for concurrency primitives
   - Causality integrates with resource locks and deterministic scheduling

## Defining and Executing Effects

### Defining a New Effect

To define a new effect, create a struct that implements the `Effect` trait:

```rust
#[derive(Debug, Clone)]
pub struct ExternalAPIEffect {
    /// The API endpoint to call
    pub endpoint: String,
    /// The request payload
    pub payload: Vec<u8>,
    /// Fact dependencies
    pub dependencies: Vec<FactDependency>,
    /// Fact snapshot
    pub snapshot: Option<FactSnapshot>,
}

impl Effect for ExternalAPIEffect {
    type Output = Vec<u8>;
    
    fn get_type(&self) -> EffectType {
        EffectType::ExternalAPI
    }
    
    fn as_debug(&self) -> &dyn fmt::Debug {
        self
    }
    
    fn clone_box(&self) -> Box<dyn Effect<Output = Self::Output>> {
        Box::new(self.clone())
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        // This effect doesn't use specific resources
        Vec::new()
    }
    
    fn domains(&self) -> Vec<DomainId> {
        // This effect is outside any specific domain
        Vec::new()
    }
    
    fn execute(self, handler: &dyn EffectHandler) -> Self::Output {
        handler.handle_external_api(self.endpoint, self.payload)
    }
    
    fn fact_dependencies(&self) -> Vec<FactDependency> {
        self.dependencies.clone()
    }
    
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        self.snapshot.clone()
    }
}
```

### Implementing an Effect Handler

To handle the effect, implement the corresponding method in an `EffectHandler`:

```rust
pub struct ExternalAPIHandler {
    // Handler configuration
    client: HttpClient,
    timeout: Duration,
}

impl EffectHandler for ExternalAPIHandler {
    // Other handler methods...
    
    fn handle_external_api(&self, endpoint: String, payload: Vec<u8>) -> Vec<u8> {
        // Make the API call with the configured client
        match self.client.post(&endpoint, &payload).timeout(self.timeout).send() {
            Ok(response) => response.body().to_vec(),
            Err(_) => Vec::new(), // Empty response on error
        }
    }
}
```

### Executing an Effect

To execute an effect, create it and pass it to a handler:

```rust
// Create the effect
let effect = ExternalAPIEffect {
    endpoint: "https://api.example.com/data".to_string(),
    payload: serde_json::to_vec(&request_data).unwrap(),
    dependencies: Vec::new(),
    snapshot: None,
};

// Create a handler
let handler = ExternalAPIHandler {
    client: HttpClient::new(),
    timeout: Duration::from_secs(5),
};

// Execute the effect
let response = effect.execute(&handler);

// Process the response
let result = serde_json::from_slice::<ResponseType>(&response).unwrap();
```

### Handling External System Effects

For effects that interact with external systems (like blockchain or APIs), use boundary crossings:

```rust
pub struct EthereumCallEffect {
    /// The contract address
    pub contract: Address,
    /// The function to call
    pub function: String,
    /// The arguments to pass
    pub args: Vec<Value>,
    /// Gas limit for the call
    pub gas_limit: u64,
}

impl Effect for EthereumCallEffect {
    type Output = EthereumCallResult;
    
    // Implementation...
    
    fn execute(self, handler: &dyn EffectHandler) -> Self::Output {
        handler.handle_ethereum_call(self.contract, self.function, self.args, self.gas_limit)
    }
}

// Handler implementation
impl EffectHandler for EthereumHandler {
    fn handle_ethereum_call(
        &self, 
        contract: Address, 
        function: String, 
        args: Vec<Value>, 
        gas_limit: u64
    ) -> EthereumCallResult {
        // Create a boundary crossing
        let crossing = BoundaryCrossing {
            context: self.context.clone(),
            payload: serialize_ethereum_call(contract, function, args, gas_limit),
            authentication: self.auth_token.clone(),
            timestamp: Utc::now(),
            origin: SystemBoundary::Internal,
            destination: SystemBoundary::Ethereum,
        };
        
        // Send the boundary crossing
        let result = self.boundary_service.send_crossing(crossing)?;
        
        // Parse and return the result
        parse_ethereum_result(result)
    }
}
```

## Program Account Effects Integration

Program accounts can leverage the effect system through the `ProgramAccountEffectAdapter`, which:

1. Maps program accounts to effects
2. Manages effect capabilities for accounts
3. Filters available effects based on account types
4. Executes effects with the appropriate context
5. Provides a consistent interface for UI integration

### Using Program Account Effects

To use effects with program accounts:

1. Implement the `ProgramAccountEffect` trait for your effects
2. Create a `ProgramAccountEffectAdapterImpl` instance
3. Register accounts and their capabilities
4. Get available effects for accounts
5. Execute effects with parameters

### Example: Token Transfer Effect

The example demonstrates a token transfer effect that:

1. Transfers tokens between resources
2. Verifies capabilities for authorization
3. Can execute across different boundaries
4. Provides rich metadata for UI display
5. Integrates with program accounts

## Security Considerations

- Effects require appropriate capabilities to execute
- Boundary crossings include authentication
- Effects validate their inputs and execution environment
- Each effect specifies its required capabilities
- The effect system maintains an audit trail of boundary crossings

## Example Code

See the following examples for detailed implementations:

- `examples/program_account_effect.rs` - Basic effect usage with program accounts

### Basic Effect Execution

```rust
// Example of executing an effect on a program account
let mut params = HashMap::new();
params.insert("source_resource_id".to_string(), source_id.to_string());
params.insert("destination_resource_id".to_string(), dest_id.to_string());
params.insert("amount".to_string(), "50".to_string());

let outcome = effect_adapter.execute_effect(
    &account_id,
    "transfer",
    params,
).await?;
```
