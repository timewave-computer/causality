<!-- Architecture for VM -->
<!-- Original file: docs/src/vm_architecture.md -->

# Virtual Machine Architecture

## Overview

The Causality system employs a virtual machine (VM) architecture to provide a secure, controlled execution environment for resource operations and cross-domain interactions. The VM system is a foundational component that supports the execution of operations while maintaining content addressing, capability-based security, and temporal consistency guarantees.

The VM architecture in Causality follows a layered design that integrates with the content-addressed storage system, the unified operation model, and the three-layer effect architecture. This architecture enables the system to execute operations safely and deterministically while providing a consistent programming model across different domains.

## Core Components

### VM Subsystems

The VM architecture consists of several key subsystems:

1. **VM Runtime**: Manages the execution environment, instruction processing, and state transitions
2. **Memory Management**: Controls access to VM memory segments and provides isolation
3. **Register System**: Manages one-time use registers for resource operations
4. **Resource Integration**: Bridges the VM with the unified ResourceRegister system
5. **ZK Integration**: Supports zero-knowledge proof generation for operations
6. **Deferred Hashing**: Optimizes cryptographic operations for ZK workflow efficiency

### Virtual Machine Types

The Causality system supports several types of virtual machines:

1. **RISC-V VM**: A general-purpose VM for executing TEL operations
2. **Domain-Specific VMs**: Specialized VMs for specific domains like EVM, CosmWasm
3. **ZK-VM**: Specialized VM for zero-knowledge proof generation
4. **TEL Resource VM**: VM for TEL-based resource management operations

## Architecture Diagram

```
┌───────────────────────────────────────────────────────────────────┐
│                         Application Layer                         │
└───────────────────────────────────┬───────────────────────────────┘
                                    │
┌───────────────────────────────────▼───────────────────────────────┐
│                    Three-Layer Effect Architecture                │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────────┐   │
│  │ Algebraic      │  │ Effect         │  │ Domain             │   │
│  │ Effect Layer   │  │ Constraints    │  │ Implementation     │   │
│  │ (Rust)         │  │ Layer (Rust)   │  │ Layer (TEL)        │   │
│  └────────────────┘  └────────────────┘  └────────────────────┘   │
└───────────────────────────────────┬───────────────────────────────┘
                                    │
┌───────────────────────────────────▼───────────────────────────────┐
│                      VM Execution Environment                     │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────────┐   │
│  │ VM Runtime     │  │ Memory         │  │ Register           │   │
│  │ Services       │  │ Management     │  │ System             │   │
│  └────────────────┘  └────────────────┘  └────────────────────┘   │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────────┐   │
│  │ ResourceVM     │  │ ZK Integration │  │ Deferred Hashing   │   │
│  │ Integration    │  │                │  │ System             │   │
│  └────────────────┘  └────────────────┘  └────────────────────┘   │
└───────────────────────────────────┬───────────────────────────────┘
                                    │
┌───────────────────────────────────▼───────────────────────────────┐
│                    Content-Addressed Storage                      │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────────┐   │
│  │ Content        │  │ Content        │  │ Storage            │   │
│  │ Addressed      │  │ References     │  │ Commitment         │   │
│  │ Objects        │  │                │  │ Management         │   │
│  └────────────────┘  └────────────────┘  └────────────────────┘   │
└───────────────────────────────────────────────────────────────────┘
```

## Detailed Component Descriptions

### VM Runtime

The VM Runtime provides the core execution environment for operations. It includes:

1. **Instruction Processing**: Executes VM instructions
2. **Register Management**: Tracks register state and transitions
3. **Program Counter**: Manages program execution flow
4. **Exception Handling**: Handles runtime errors and exceptions
5. **Breakpoint Support**: Allows debugging and step-by-step execution

```rust
/// The virtual machine state
pub enum VmState {
    Ready,
    Running,
    Paused,
    Error,
    Completed,
}

/// A virtual machine for executing operations
pub struct VirtualMachine {
    state: VmState,
    memory: MemoryMapper,
    registers: [u64; 32],
    pc: u64,
    breakpoints: HashSet<u64>,
}
```

### Memory Management

The Memory Management subsystem controls access to memory segments within the VM:

1. **Memory Segments**: Organizes memory into logical segments
2. **Access Control**: Enforces memory access permissions
3. **Resource Mapping**: Maps resources to VM memory
4. **Isolation**: Ensures memory isolation between operations

```rust
/// Memory manager for VM operations
pub struct MemoryManager {
    registers: HashMap<VmRegId, VmRegister>,
    sections: HashMap<String, Vec<VmRegId>>,
}
```

### Register System

The Register System implements the one-time use register pattern for resource operations:

1. **VmRegister**: Represents a register in the VM
2. **Register ID**: Uniquely identifies registers using content addressing
3. **Register Operations**: Create, read, update, and consume operations
4. **Register State Tracking**: Tracks register lifecycle states

```rust
/// A VM register ID
pub struct VmRegId(pub ContentId);

/// A VM register
pub struct VmRegister {
    pub id: VmRegId,
    pub section: String,
    pub data: Vec<u8>,
}
```

### Resource Integration

The Resource Integration subsystem bridges the VM with the unified ResourceRegister system:

1. **Resource Mapping**: Maps ResourceRegisters to VM registers
2. **Access Control**: Enforces capability-based access control
3. **State Transfer**: Manages state transfers between VM and resources
4. **Execution Context**: Provides context for resource operations

```rust
/// Integrates resource management with the VM
pub struct ResourceVmIntegration {
    resource_manager: Arc<ResourceManager>,
    memory_manager: MemoryManager,
    config: VmIntegrationConfig,
    register_mappings: HashMap<(String, ContentId), VmRegId>,
}
```

### ZK Integration

The ZK Integration subsystem supports zero-knowledge proof generation:

1. **Proof Generation**: Generates ZK proofs for operations
2. **Proof Verification**: Verifies ZK proofs
3. **Circuit Integration**: Integrates with ZK circuits
4. **Deferred Hashing**: Implements hash computation deferral

```rust
/// ZK proof manager for register operations
pub struct RegisterZkProofManager {
    adapter: SuccinctAdapter,
    consumption_program_id: String,
    verified_proofs: HashMap<Hash256, bool>,
}
```

### Deferred Hashing

The Deferred Hashing subsystem optimizes cryptographic operations:

1. **Hash Deferral**: Defers hash computations until after VM execution
2. **ZK-Friendly Hashing**: Uses Poseidon hash for ZK-friendly operations
3. **Commitment Verification**: Verifies commitments instead of recomputing hashes
4. **Optimized Circuits**: Uses specialized circuits for verification

```rust
/// Execution context for the zkVM with deferred hashing
pub struct ZkVmExecutionContext {
    // Content to be hashed after execution
    deferred_hash_inputs: Vec<DeferredHashInput>,
    // Hash outputs (filled after execution)
    hash_outputs: HashMap<DeferredHashId, ContentHash>,
}
```

## VM Execution Flow

The VM execution follows a structured flow:

1. **Preparation**:
   - Load program into VM memory
   - Set up initial register state
   - Create execution context

2. **Execution**:
   - Execute instructions
   - Manage register transitions
   - Handle memory operations
   - Defer cryptographic operations

3. **Finalization**:
   - Complete deferred hash computations
   - Generate execution proofs
   - Commit state changes

4. **Verification**:
   - Verify execution results
   - Validate state transitions
   - Check temporal consistency

## VM Integration with the TEL System

The VM architecture integrates with the Temporal Effect Language (TEL) system:

1. **TEL Compilation**: TEL scripts are compiled to VM instructions
2. **Effect Execution**: Effects are executed within the VM
3. **Resource Access**: Resources are accessed through VM registers
4. **Domain Integration**: Domain-specific operations use specialized VMs

```rust
/// Execute an effect in the VM
pub async fn execute_effect<E: Effect>(
    effect: &E,
    context: &ExecutionContext,
) -> Result<E::Output, EffectError> {
    // Get the domain VM for the effect
    let domain = effect.domains().first().ok_or(EffectError::NoDomain)?;
    let vm = get_vm_for_domain(domain)?;
    
    // Prepare VM state
    prepare_vm_for_effect(vm, effect, context)?;
    
    // Execute in VM
    let result = vm.execute()?;
    
    // Extract result
    let output = extract_result_from_vm(vm, effect)?;
    
    Ok(output)
}
```

## Cross-Domain VM Operations

The VM architecture supports cross-domain operations through:

1. **Domain-Specific VMs**: Each domain may have its own specialized VM
2. **VM Proxying**: Operations can be proxied between VMs
3. **Unified Register Model**: Common register model across domains
4. **Cross-Domain Proofs**: ZK proofs verify cross-domain operations

## Optimization Strategies

The VM architecture implements several optimization strategies:

1. **Register Caching**: Frequently accessed registers are cached
2. **Instruction Optimization**: Common instruction sequences are optimized
3. **Deferred Hashing**: Cryptographic operations are deferred
4. **Parallel Execution**: Independent operations execute in parallel
5. **Just-In-Time Compilation**: TEL code is compiled just-in-time for specific domains

## VM Security Model

The VM security model includes:

1. **Capability-Based Access**: Resources are accessed through capabilities
2. **Memory Isolation**: Operations are isolated in memory
3. **Deterministic Execution**: Operations execute deterministically
4. **Proof Verification**: ZK proofs verify operation correctness
5. **Temporal Consistency**: Operations maintain temporal consistency

## Future Enhancements

Future enhancements to the VM architecture include:

1. **VM Specialization**: More domain-specific VMs
2. **Enhanced JIT Compilation**: Improved TEL compilation
3. **VM Sandboxing**: Advanced isolation techniques
4. **Parallel Execution**: Improved parallel execution
5. **Hardware Acceleration**: Support for specialized hardware

## Implementation Status

The VM architecture is currently implemented with the following components:

1. **ResourceVmIntegration**: Fully implemented for TEL resource operations
2. **ZK Integration**: Partially implemented with deferred hashing
3. **Domain-Specific VMs**: Implemented for EVM and CosmWasm
4. **Register System**: Fully implemented with content addressing
5. **Deferred Hashing**: Implemented following ADR-030

Additional implementation work is ongoing to enhance performance, security, and cross-domain capabilities.

## Related Documentation

- [VM Capability Management](vm_capability_management.md)
- [VM Execution](vm_execution.md)
- [VM Interface](vm_interface.md)
- [VM Sandbox](vm_sandbox.md)
- [Deferred Hashing in VM](storage_commitment.md)
- [Three-Layer Effect Architecture](unified_effect_model.md)
- [Unified Operation Model](unified_operation_model.md)
- [Content Addressing](content_addressing.md)
- [Resource Register Unified Model](resource_register_unified_model.md)