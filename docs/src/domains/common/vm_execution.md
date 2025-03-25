<!-- Execution of VM -->
<!-- Original file: docs/src/vm_execution.md -->

# VM Execution Model

## Overview

The VM Execution Model in Causality defines how operations are executed within the virtual machine environment, ensuring deterministic, secure, and verifiable execution across different domains. This model is tightly integrated with the ResourceRegister unified model and the three-layer effect architecture to provide a consistent execution environment for all operations.

## Core Components

### Execution Cycle

The VM execution cycle follows a deterministic process:

1. **Initialization**: VM state is initialized with inputs and capability context
2. **Fetch**: Instructions are fetched from the content-addressed storage
3. **Decode**: Instructions are decoded into executable operations
4. **Execute**: Operations are executed with capability checks
5. **Memory Access**: Memory operations access VM memory through a capability-controlled interface
6. **Register Updates**: Register state is updated based on operation results
7. **Commit**: State changes are committed to the ResourceRegister
8. **Proof Generation**: Zero-knowledge proofs are generated for executed operations

### Execution Context

Each operation executes within a context that defines its security boundaries and available resources:

```rust
/// Execution context for VM operations
pub struct ExecutionContext {
    /// Domain ID where execution is taking place
    pub domain_id: DomainId,
    /// Trace ID for the current execution
    pub trace_id: TraceId,
    /// Capability token for the execution
    pub capability_token: CapabilityToken,
    /// Timestamp for the execution
    pub timestamp: Timestamp,
    /// Execution environment details
    pub environment: Environment,
}
```

## Execution Pipeline

The VM execution pipeline processes operations through several stages:

```
┌───────────────────┐    ┌───────────────────┐    ┌───────────────────┐
│ Operation Loading │ -> │ Capability Check  │ -> │ Input Validation  │
└─────────┬─────────┘    └─────────┬─────────┘    └─────────┬─────────┘
          │                        │                        │
          ▼                        ▼                        ▼
┌───────────────────┐    ┌───────────────────┐    ┌───────────────────┐
│ Memory Allocation │ -> │ Instruction       │ -> │ Effect Generation │
│                   │    │ Execution         │    │                   │
└─────────┬─────────┘    └─────────┬─────────┘    └─────────┬─────────┘
          │                        │                        │
          ▼                        ▼                        ▼
┌───────────────────┐    ┌───────────────────┐    ┌───────────────────┐
│ State Update      │ -> │ Proof Generation  │ -> │ Result Commitment │
└───────────────────┘    └───────────────────┘    └───────────────────┘
```

## Execution Modes

The VM supports different execution modes to accommodate various operational requirements:

1. **Standard Execution**: Full execution with all checks and validations
2. **Verification Mode**: Executes only verification-specific instructions
3. **Simulation Mode**: Simulated execution without state commitment
4. **Proof Generation Mode**: Execution optimized for ZK proof generation
5. **Debug Mode**: Extended logging and step-by-step execution support

## Integration with ResourceRegister Model

The VM execution model integrates with the ResourceRegister unified model in several key ways:

1. **Register Access**: VM operations access ResourceRegisters through capability-controlled interfaces
2. **State Transitions**: Operations result in well-defined state transitions in ResourceRegisters
3. **Cross-Domain Execution**: Supports execution across domain boundaries with appropriate permissions
4. **Unified Operation Model**: All operations follow the unified operation model 

```rust
/// Execute a resource operation within the VM
pub fn execute_resource_operation(
    operation: ResourceOperation,
    context: ExecutionContext,
    resource_manager: &ResourceManager,
) -> Result<ExecutionResult, ExecutionError> {
    // Validate operation against capabilities
    let capabilities = context.capability_token.capabilities();
    if !resource_manager.validate_operation_capabilities(&operation, &capabilities) {
        return Err(ExecutionError::InsufficientCapabilities);
    }
    
    // Execute the operation
    let result = match operation.operation_type {
        OperationType::Create => execute_create(operation, context, resource_manager),
        OperationType::Read => execute_read(operation, context, resource_manager),
        OperationType::Update => execute_update(operation, context, resource_manager),
        OperationType::Delete => execute_delete(operation, context, resource_manager),
        OperationType::Transfer => execute_transfer(operation, context, resource_manager),
        OperationType::Verify => execute_verify(operation, context, resource_manager),
        // Other operation types...
    }?;
    
    // Generate proof if required
    let proof = if context.requires_proof() {
        Some(generate_proof(&operation, &result, &context))
    } else {
        None
    };
    
    Ok(ExecutionResult {
        result_state: result,
        proof,
        execution_metadata: collect_execution_metadata(&context),
    })
}
```

## TEL Integration

The VM execution model is tightly integrated with the Temporal Effect Language (TEL):

1. **TEL Operations**: TEL operations are compiled into VM instructions
2. **Effect Handling**: Effects defined in TEL are processed by the VM effect system
3. **Constraint Validation**: TEL constraints are enforced during execution
4. **Cross-Domain Capabilities**: TEL capability model maps to VM capabilities

## Batch Operations

The VM supports batch operations for efficient processing:

```rust
/// Execute a batch of operations within a single VM context
pub fn execute_batch(
    operations: Vec<ResourceOperation>,
    context: ExecutionContext,
    resource_manager: &ResourceManager,
) -> Result<BatchExecutionResult, ExecutionError> {
    let mut results = Vec::with_capacity(operations.len());
    let mut success_count = 0;
    
    // Create a transaction context
    let tx_context = TransactionContext::from(context);
    
    for operation in operations {
        match execute_resource_operation(operation, tx_context.clone_for_operation(), resource_manager) {
            Ok(result) => {
                results.push(Ok(result));
                success_count += 1;
            }
            Err(err) => {
                results.push(Err(err));
                if tx_context.fail_fast() {
                    break;
                }
            }
        }
    }
    
    // Commit or rollback the transaction
    if success_count == operations.len() || tx_context.commit_partial() {
        tx_context.commit()?;
    } else {
        tx_context.rollback()?;
    }
    
    Ok(BatchExecutionResult {
        results,
        success_count,
        transaction_metadata: tx_context.metadata(),
    })
}
```

## Deferred Hashing

The VM employs a deferred hashing approach to optimize cryptographic operations:

1. **Hash Request Pool**: Collects hash requests during execution
2. **Batched Processing**: Processes hash requests in optimized batches
3. **Parallel Execution**: Leverages parallel processing for hash computations
4. **ZK Circuit Integration**: Optimizes hash operations for ZK circuit generation

## Error Handling

The VM execution model includes comprehensive error handling:

```rust
/// Errors that can occur during execution
pub enum ExecutionError {
    /// Insufficient capabilities for the operation
    InsufficientCapabilities,
    /// Memory access violation
    MemoryAccessViolation,
    /// Invalid instruction
    InvalidInstruction,
    /// Resource not found
    ResourceNotFound,
    /// Invalid resource state
    InvalidResourceState,
    /// Domain access error
    DomainAccessError,
    /// Execution timeout
    Timeout,
    /// Proof generation error
    ProofGenerationError,
    /// State commitment error
    StateCommitmentError,
    /// Cross-domain operation error
    CrossDomainError,
}
```

## Performance Considerations

The VM execution model addresses several performance considerations:

1. **Instruction Optimization**: Common instruction sequences are optimized
2. **Memory Management**: Efficient memory allocation and deallocation
3. **Register Usage**: Optimized register allocation for common operations
4. **Proof Generation**: Efficient zero-knowledge proof generation
5. **Parallelization**: Parallel execution of independent operations

## Related Documentation

- [VM Architecture](vm_architecture.md)
- [VM Interface](vm_interface.md)
- [VM Sandbox](vm_sandbox.md)
- [VM Capability Management](vm_capability_management.md)
- [ResourceRegister Unified Model](resource_register_unified_model.md)
- [Unified Operation Model](unified_operation_model.md)
- [Three-Layer Effect Architecture](effect_templates.md) 