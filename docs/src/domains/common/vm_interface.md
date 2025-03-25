<!-- Interface for VM -->
<!-- Original file: docs/src/vm_interface.md -->

# VM Interface

## Overview

The VM Interface in Causality provides a standardized way for system components to interact with virtual machine environments. It defines a set of consistent APIs, protocols, and abstractions that enable secure integration between the VM execution environment and other system components, particularly the ResourceRegister system, capability model, and effect system.

The VM Interface is designed to be domain-agnostic, supporting multiple VM implementations while providing a unified programming model across the entire Causality system. This interface layer is crucial for ensuring that operations can be executed consistently regardless of the underlying VM implementation.

## Core Interface Components

### ResourceVM Interface

The ResourceVM interface provides operations for interacting with ResourceRegisters within the VM environment:

```rust
/// Interface for resource operations within the VM
pub trait ResourceVmInterface {
    /// Create a new resource in the VM environment
    fn create_resource(&self, 
                      resource_data: ResourceData, 
                      context: &ExecutionContext) 
        -> Result<ResourceId, VmError>;
    
    /// Read a resource from the VM environment
    fn read_resource(&self, 
                    resource_id: &ResourceId, 
                    context: &ExecutionContext) 
        -> Result<Option<ResourceData>, VmError>;
    
    /// Update a resource in the VM environment
    fn update_resource(&self, 
                      resource_id: &ResourceId, 
                      resource_data: ResourceData, 
                      context: &ExecutionContext) 
        -> Result<(), VmError>;
    
    /// Delete a resource from the VM environment
    fn delete_resource(&self, 
                      resource_id: &ResourceId, 
                      context: &ExecutionContext) 
        -> Result<(), VmError>;
    
    /// Transfer a resource between domains
    fn transfer_resource(&self, 
                        resource_id: &ResourceId, 
                        target_domain: &DomainId, 
                        context: &ExecutionContext) 
        -> Result<(), VmError>;
}
```

### Operation Interface

The Operation interface handles execution of operations within the VM:

```rust
/// Interface for executing operations in the VM
pub trait OperationInterface {
    /// Execute a single operation
    fn execute_operation(&self, 
                        operation: &ResourceOperation, 
                        context: &ExecutionContext) 
        -> Result<ExecutionResult, VmError>;
    
    /// Execute a batch of operations
    fn execute_batch(&self, 
                    operations: &[ResourceOperation], 
                    context: &ExecutionContext) 
        -> Result<Vec<ExecutionResult>, VmError>;
    
    /// Verify an operation without executing it
    fn verify_operation(&self, 
                       operation: &ResourceOperation, 
                       context: &ExecutionContext) 
        -> Result<VerificationResult, VmError>;
}
```

### Memory Interface

The Memory interface controls access to VM memory:

```rust
/// Interface for memory management in the VM
pub trait MemoryInterface {
    /// Allocate memory in the VM
    fn allocate(&self, 
               size: usize, 
               access_rights: AccessRights, 
               context: &ExecutionContext) 
        -> Result<MemoryAddress, VmError>;
    
    /// Free allocated memory
    fn free(&self, 
           address: MemoryAddress, 
           context: &ExecutionContext) 
        -> Result<(), VmError>;
    
    /// Read data from VM memory
    fn read(&self, 
           address: MemoryAddress, 
           length: usize, 
           context: &ExecutionContext) 
        -> Result<Vec<u8>, VmError>;
    
    /// Write data to VM memory
    fn write(&self, 
            address: MemoryAddress, 
            data: &[u8], 
            context: &ExecutionContext) 
        -> Result<(), VmError>;
}
```

### Effect Interface

The Effect interface manages effects within the VM environment:

```rust
/// Interface for effect management in the VM
pub trait EffectInterface {
    /// Register an effect handler
    fn register_effect_handler(&self, 
                              effect_type: EffectType, 
                              handler: Box<dyn EffectHandler>, 
                              context: &ExecutionContext) 
        -> Result<(), VmError>;
    
    /// Handle an effect within the VM
    fn handle_effect(&self, 
                    effect: &Effect, 
                    context: &ExecutionContext) 
        -> Result<EffectResult, VmError>;
    
    /// Validate an effect against constraints
    fn validate_effect(&self, 
                      effect: &Effect, 
                      constraints: &[EffectConstraint], 
                      context: &ExecutionContext) 
        -> Result<bool, VmError>;
}
```

## Interface Integration Patterns

### Adapter Pattern

The VM system uses the adapter pattern to integrate different VM implementations:

```rust
/// Adapter for RISC-V VM implementation
pub struct RiscVAdapter {
    vm: RiscVVirtualMachine,
    resource_manager: Arc<ResourceManager>,
    capability_manager: Arc<CapabilityManager>,
}

impl ResourceVmInterface for RiscVAdapter {
    // Implementation for RISC-V VM
}

/// Adapter for EVM VM implementation
pub struct EvmAdapter {
    vm: EthereumVirtualMachine,
    resource_manager: Arc<ResourceManager>,
    capability_manager: Arc<CapabilityManager>,
}

impl ResourceVmInterface for EvmAdapter {
    // Implementation for EVM
}
```

### Factory Pattern

VM interfaces are instantiated through a factory pattern:

```rust
/// Factory for creating VM interfaces
pub struct VmInterfaceFactory {
    resource_manager: Arc<ResourceManager>,
    capability_manager: Arc<CapabilityManager>,
    effect_manager: Arc<EffectManager>,
}

impl VmInterfaceFactory {
    /// Create a ResourceVM interface for the specified VM type
    pub fn create_resource_vm_interface(&self, vm_type: VmType) -> Box<dyn ResourceVmInterface> {
        match vm_type {
            VmType::RiscV => Box::new(RiscVAdapter::new(
                self.resource_manager.clone(),
                self.capability_manager.clone(),
            )),
            VmType::Evm => Box::new(EvmAdapter::new(
                self.resource_manager.clone(),
                self.capability_manager.clone(),
            )),
            VmType::CosmWasm => Box::new(CosmWasmAdapter::new(
                self.resource_manager.clone(),
                self.capability_manager.clone(),
            )),
            VmType::ZkVm => Box::new(ZkVmAdapter::new(
                self.resource_manager.clone(),
                self.capability_manager.clone(),
            )),
        }
    }
}
```

## Cross-Domain Interface

The CrossDomainVmInterface enables operations across domain boundaries:

```rust
/// Interface for cross-domain VM operations
pub trait CrossDomainVmInterface {
    /// Execute an operation across domain boundaries
    fn execute_cross_domain_operation(&self, 
                                     operation: &ResourceOperation, 
                                     source_domain: &DomainId, 
                                     target_domain: &DomainId, 
                                     context: &ExecutionContext) 
        -> Result<CrossDomainExecutionResult, VmError>;
    
    /// Transfer a resource between domains
    fn transfer_resource_cross_domain(&self, 
                                     resource_id: &ResourceId, 
                                     source_domain: &DomainId, 
                                     target_domain: &DomainId, 
                                     context: &ExecutionContext) 
        -> Result<TransferResult, VmError>;
    
    /// Verify cross-domain operation permissions
    fn verify_cross_domain_permissions(&self, 
                                      operation: &ResourceOperation, 
                                      source_domain: &DomainId, 
                                      target_domain: &DomainId, 
                                      context: &ExecutionContext) 
        -> Result<bool, VmError>;
}
```

## TEL Integration Interface

The VM provides a specialized interface for TEL operations:

```rust
/// Interface for TEL integration with VM
pub trait TelVmInterface {
    /// Execute a TEL program in the VM
    fn execute_tel_program(&self, 
                          program: &TelProgram, 
                          context: &ExecutionContext) 
        -> Result<TelExecutionResult, VmError>;
    
    /// Register a TEL effect handler
    fn register_tel_effect_handler(&self, 
                                  effect_type: TelEffectType, 
                                  handler: Box<dyn TelEffectHandler>, 
                                  context: &ExecutionContext) 
        -> Result<(), VmError>;
    
    /// Compile TEL to VM instructions
    fn compile_tel_to_vm(&self, 
                        program: &TelProgram, 
                        optimization_level: OptimizationLevel, 
                        context: &ExecutionContext) 
        -> Result<CompiledVmProgram, VmError>;
}
```

## Zero-Knowledge Interface

The ZkVmInterface enables zero-knowledge operations:

```rust
/// Interface for ZK operations in the VM
pub trait ZkVmInterface {
    /// Generate a zero-knowledge proof for an operation
    fn generate_proof(&self, 
                     operation: &ResourceOperation, 
                     context: &ExecutionContext) 
        -> Result<ZkProof, VmError>;
    
    /// Verify a zero-knowledge proof
    fn verify_proof(&self, 
                   operation: &ResourceOperation, 
                   proof: &ZkProof, 
                   context: &ExecutionContext) 
        -> Result<bool, VmError>;
    
    /// Generate a ZK circuit for an operation type
    fn generate_circuit(&self, 
                       operation_type: OperationType, 
                       context: &ExecutionContext) 
        -> Result<ZkCircuit, VmError>;
}
```

## VM Lifecycle Interface

The VMLifecycleInterface manages VM instance lifecycle:

```rust
/// Interface for VM lifecycle management
pub trait VmLifecycleInterface {
    /// Initialize a new VM instance
    fn initialize(&self, 
                 vm_config: &VmConfig, 
                 context: &ExecutionContext) 
        -> Result<VmInstanceId, VmError>;
    
    /// Start a VM instance
    fn start(&self, 
            instance_id: &VmInstanceId, 
            context: &ExecutionContext) 
        -> Result<(), VmError>;
    
    /// Stop a VM instance
    fn stop(&self, 
           instance_id: &VmInstanceId, 
           context: &ExecutionContext) 
        -> Result<(), VmError>;
    
    /// Pause a VM instance
    fn pause(&self, 
            instance_id: &VmInstanceId, 
            context: &ExecutionContext) 
        -> Result<(), VmError>;
    
    /// Resume a paused VM instance
    fn resume(&self, 
             instance_id: &VmInstanceId, 
             context: &ExecutionContext) 
        -> Result<(), VmError>;
    
    /// Terminate and clean up a VM instance
    fn terminate(&self, 
                instance_id: &VmInstanceId, 
                context: &ExecutionContext) 
        -> Result<(), VmError>;
}
```

## Debug Interface

The DebugInterface provides tools for debugging VM execution:

```rust
/// Interface for VM debugging
pub trait DebugInterface {
    /// Set a breakpoint in the VM
    fn set_breakpoint(&self, 
                     instance_id: &VmInstanceId, 
                     address: MemoryAddress, 
                     context: &ExecutionContext) 
        -> Result<BreakpointId, VmError>;
    
    /// Remove a breakpoint
    fn remove_breakpoint(&self, 
                        instance_id: &VmInstanceId, 
                        breakpoint_id: BreakpointId, 
                        context: &ExecutionContext) 
        -> Result<(), VmError>;
    
    /// Step execution by one instruction
    fn step(&self, 
           instance_id: &VmInstanceId, 
           context: &ExecutionContext) 
        -> Result<DebugState, VmError>;
    
    /// Continue execution until next breakpoint
    fn continue_execution(&self, 
                         instance_id: &VmInstanceId, 
                         context: &ExecutionContext) 
        -> Result<DebugState, VmError>;
    
    /// Get current VM state
    fn get_state(&self, 
                instance_id: &VmInstanceId, 
                context: &ExecutionContext) 
        -> Result<VmState, VmError>;
    
    /// Get register values
    fn get_registers(&self, 
                    instance_id: &VmInstanceId, 
                    context: &ExecutionContext) 
        -> Result<RegisterState, VmError>;
}
```

## Resource Management Interface

The ResourceManagementInterface connects VM operations with the ResourceRegister system:

```rust
/// Interface for resource management within the VM
pub trait ResourceManagementInterface {
    /// Get a ResourceRegister by ID
    fn get_resource_register(&self, 
                            register_id: &RegisterId, 
                            context: &ExecutionContext) 
        -> Result<Option<ResourceRegister>, VmError>;
    
    /// List ResourceRegisters with optional filters
    fn list_resource_registers(&self, 
                              filter: Option<RegisterFilter>, 
                              context: &ExecutionContext) 
        -> Result<Vec<ResourceRegister>, VmError>;
    
    /// Update a ResourceRegister
    fn update_resource_register(&self, 
                               register: &ResourceRegister, 
                               context: &ExecutionContext) 
        -> Result<(), VmError>;
    
    /// Create a relationship between ResourceRegisters
    fn create_relationship(&self, 
                          source_id: &RegisterId, 
                          target_id: &RegisterId, 
                          relationship_type: RelationshipType, 
                          context: &ExecutionContext) 
        -> Result<RelationshipId, VmError>;
    
    /// Query relationships for a ResourceRegister
    fn query_relationships(&self, 
                          register_id: &RegisterId, 
                          direction: RelationshipDirection, 
                          context: &ExecutionContext) 
        -> Result<Vec<Relationship>, VmError>;
}
```

## Interface Versioning

The VM interfaces include versioning to ensure compatibility:

```rust
/// Interface version information
pub struct InterfaceVersion {
    /// Major version (breaking changes)
    pub major: u32,
    /// Minor version (non-breaking additions)
    pub minor: u32,
    /// Patch version (bug fixes)
    pub patch: u32,
}

/// Version-aware interface marker
pub trait VersionedInterface {
    /// Get the interface version
    fn version(&self) -> InterfaceVersion;
    
    /// Check compatibility with a required version
    fn is_compatible_with(&self, required: &InterfaceVersion) -> bool {
        self.version().major == required.major && 
        self.version().minor >= required.minor
    }
}
```

## Error Handling

VM interfaces use consistent error handling:

```rust
/// Errors that can occur in VM interfaces
pub enum VmError {
    /// Capability error
    CapabilityError(CapabilityError),
    /// Memory access error
    MemoryError(MemoryError),
    /// Resource operation error
    ResourceError(ResourceError),
    /// Execution error
    ExecutionError(ExecutionError),
    /// Cross-domain error
    CrossDomainError(CrossDomainError),
    /// TEL execution error
    TelError(TelError),
    /// ZK proof error
    ZkError(ZkError),
    /// VM lifecycle error
    LifecycleError(LifecycleError),
    /// Debug interface error
    DebugError(DebugError),
}
```

## Related Documentation

- [VM Architecture](vm_architecture.md)
- [VM Execution](vm_execution.md)
- [VM Sandbox](vm_sandbox.md)
- [VM Capability Management](vm_capability_management.md)
- [ResourceRegister Unified Model](resource_register_unified_model.md)
- [Unified Operation Model](unified_operation_model.md)
- [Three-Layer Effect Architecture](effect_templates.md) 