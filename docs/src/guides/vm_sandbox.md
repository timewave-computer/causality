<!-- Guide for using the VM sandbox -->
<!-- Original file: docs/src/vm_sandbox.md -->

# VM Sandbox

## Overview

The VM Sandbox in Causality provides a secure execution environment that isolates VM operations from the host system and other VMs. It ensures that operations execute with appropriate security boundaries, preventing unauthorized access to system resources and containing potential security vulnerabilities. The sandbox is a critical component of the Causality security architecture, enabling safe execution of untrusted or third-party code while maintaining the integrity of the overall system.

The VM Sandbox integrates with the capability-based security model, the unified ResourceRegister model, and the three-layer effect system to provide comprehensive security guarantees while enabling efficient resource access and cross-domain operations.

## Core Components

### Sandbox Architecture

The VM Sandbox consists of several key components:

1. **Resource Isolation**: Isolates ResourceRegisters and their operations
2. **Memory Isolation**: Provides isolated memory regions for VM execution
3. **Capability Enforcement**: Enforces capability-based access control
4. **Effect Sandboxing**: Constrains effect handlers to operate within boundaries
5. **Cross-Domain Boundary**: Manages secure cross-domain interactions

```
┌─────────────────────────────────────────────────────────────────┐
│                         Host System                             │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                  VM Sandbox Manager                     │    │
│  │                                                         │    │
│  │  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   │    │
│  │  │ Capability  │   │ Memory      │   │ Resource    │   │    │
│  │  │ Enforcement │   │ Management  │   │ Access      │   │    │
│  │  └─────────────┘   └─────────────┘   └─────────────┘   │    │
│  │                                                         │    │
│  │  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   │    │
│  │  │ Effect      │   │ Cross-Domain│   │ Proof       │   │    │
│  │  │ Constraints │   │ Gateway     │   │ Verification│   │    │
│  │  └─────────────┘   └─────────────┘   └─────────────┘   │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   VM Instance   │  │   VM Instance   │  │   VM Instance   │ │
│  │                 │  │                 │  │                 │ │
│  │  ┌───────────┐ │  │  ┌───────────┐  │  │  ┌───────────┐  │ │
│  │  │ Isolated  │ │  │  │ Isolated  │  │  │  │ Isolated  │  │ │
│  │  │ Memory    │ │  │  │ Memory    │  │  │  │ Memory    │  │ │
│  │  └───────────┘ │  │  └───────────┘  │  │  └───────────┘  │ │
│  │                 │  │                 │  │                 │ │
│  │  ┌───────────┐ │  │  ┌───────────┐  │  │  ┌───────────┐  │ │
│  │  │ Operation │ │  │  │ Operation │  │  │  │ Operation │  │ │
│  │  │ Context   │ │  │  │ Context   │  │  │  │ Context   │  │ │
│  │  └───────────┘ │  │  └───────────┘  │  │  └───────────┘  │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Security Boundaries

VM Sandbox enforces several types of security boundaries:

1. **Memory Boundaries**: Restrict memory access to allocated regions
2. **Resource Boundaries**: Limit resource access based on capabilities
3. **Effect Boundaries**: Constrain effects to authorized domains
4. **Domain Boundaries**: Control cross-domain operations
5. **Time Boundaries**: Limit execution time and resource consumption

## Implementation Strategies

### Memory Isolation

Memory isolation is implemented through several techniques:

```rust
/// Memory access rights for sandbox regions
pub enum MemoryAccessRights {
    /// No access
    None,
    /// Read-only access
    Read,
    /// Read and write access
    ReadWrite,
    /// Read, write, and execute access
    ReadWriteExecute,
}

/// A memory region in the sandbox
pub struct MemoryRegion {
    /// Base address of the region
    pub base_address: usize,
    /// Size of the region in bytes
    pub size: usize,
    /// Access rights for the region
    pub access_rights: MemoryAccessRights,
    /// Owner of the region
    pub owner_id: VmInstanceId,
}

/// Memory manager for sandbox instances
pub struct SandboxMemoryManager {
    /// Regions allocated to sandbox instances
    regions: Vec<MemoryRegion>,
    /// Page size used for memory allocation
    page_size: usize,
    /// Total memory available
    total_memory: usize,
}

impl SandboxMemoryManager {
    /// Allocate a new memory region
    pub fn allocate_region(&mut self, 
                          size: usize, 
                          access_rights: MemoryAccessRights, 
                          owner_id: VmInstanceId) 
        -> Result<MemoryRegion, SandboxError> {
        // Check if enough memory is available
        // Find a suitable location
        // Allocate the region with proper access rights
        // Return the new region
    }
    
    /// Validate memory access
    pub fn validate_access(&self, 
                          address: usize, 
                          size: usize, 
                          required_rights: MemoryAccessRights, 
                          instance_id: VmInstanceId) 
        -> Result<(), SandboxError> {
        // Find the region containing the address
        // Check if the access rights are sufficient
        // Ensure the access doesn't cross region boundaries
    }
}
```

### Capability Enforcement

The sandbox enforces capability-based access control:

```rust
/// Sandbox capability validator
pub struct SandboxCapabilityValidator {
    /// Capability manager for checking capabilities
    capability_manager: Arc<CapabilityManager>,
}

impl SandboxCapabilityValidator {
    /// Validate an operation against capabilities
    pub fn validate_operation(&self, 
                            operation: &ResourceOperation, 
                            context: &ExecutionContext) 
        -> Result<(), SandboxError> {
        // Get the required capabilities for the operation
        let required_capabilities = self.get_required_capabilities(operation);
        
        // Check if the context has the required capabilities
        if !self.capability_manager.has_capabilities(
            &context.capability_token, 
            &required_capabilities
        ) {
            return Err(SandboxError::InsufficientCapabilities);
        }
        
        Ok(())
    }
    
    /// Get the capabilities required for an operation
    fn get_required_capabilities(&self, operation: &ResourceOperation) 
        -> Vec<Capability> {
        // Determine required capabilities based on operation type and target
    }
}
```

### Effect Sandboxing

Effects are constrained within the sandbox:

```rust
/// Sandboxed effect executor
pub struct SandboxedEffectExecutor {
    /// Effect manager for handling effects
    effect_manager: Arc<EffectManager>,
    /// Constraints for effect execution
    constraints: Vec<EffectConstraint>,
}

impl SandboxedEffectExecutor {
    /// Execute an effect within sandbox constraints
    pub fn execute_effect(&self, 
                         effect: &Effect, 
                         context: &ExecutionContext) 
        -> Result<EffectResult, SandboxError> {
        // Validate the effect against constraints
        self.validate_effect(effect, context)?;
        
        // Execute the effect
        let result = self.effect_manager.handle_effect(effect, context)
            .map_err(|err| SandboxError::EffectError(err))?;
        
        // Validate the result
        self.validate_effect_result(&result, effect, context)?;
        
        Ok(result)
    }
    
    /// Validate an effect against sandbox constraints
    fn validate_effect(&self, 
                      effect: &Effect, 
                      context: &ExecutionContext) 
        -> Result<(), SandboxError> {
        // Check effect against all constraints
        for constraint in &self.constraints {
            if !constraint.validate(effect, context) {
                return Err(SandboxError::ConstraintViolation);
            }
        }
        
        Ok(())
    }
}
```

### Cross-Domain Gateway

The sandbox includes a secure gateway for cross-domain operations:

```rust
/// Cross-domain gateway for sandbox
pub struct CrossDomainGateway {
    /// Domain manager for handling cross-domain operations
    domain_manager: Arc<DomainManager>,
    /// Authorization manager for cross-domain requests
    auth_manager: Arc<AuthorizationManager>,
}

impl CrossDomainGateway {
    /// Execute a cross-domain operation
    pub fn execute_cross_domain(&self, 
                              operation: &ResourceOperation, 
                              source_domain: &DomainId, 
                              target_domain: &DomainId, 
                              context: &ExecutionContext) 
        -> Result<CrossDomainResult, SandboxError> {
        // Validate cross-domain authorization
        self.auth_manager.validate_cross_domain(
            source_domain, 
            target_domain, 
            &operation.operation_type, 
            &context.capability_token
        )?;
        
        // Create a new context for the target domain
        let target_context = self.create_target_context(
            context, 
            target_domain
        )?;
        
        // Execute the operation in the target domain
        let result = self.domain_manager.execute_in_domain(
            operation, 
            target_domain, 
            &target_context
        )?;
        
        Ok(result)
    }
}
```

## Sandbox Management

### VM Instance Management

The sandbox manages VM instances throughout their lifecycle:

```rust
/// VM instance within a sandbox
pub struct SandboxVmInstance {
    /// Unique ID of the VM instance
    pub id: VmInstanceId,
    /// Type of VM
    pub vm_type: VmType,
    /// Current state of the VM
    pub state: VmState,
    /// Allocated resources
    pub resources: VmResources,
    /// Operation context
    pub context: ExecutionContext,
}

/// Sandbox VM manager
pub struct SandboxVmManager {
    /// Active VM instances
    instances: HashMap<VmInstanceId, SandboxVmInstance>,
    /// Memory manager
    memory_manager: SandboxMemoryManager,
    /// Resource quotas
    resource_quotas: ResourceQuotas,
}

impl SandboxVmManager {
    /// Create a new VM instance
    pub fn create_instance(&mut self, 
                          vm_type: VmType, 
                          config: &VmConfig, 
                          context: &ExecutionContext) 
        -> Result<VmInstanceId, SandboxError> {
        // Allocate resources for the VM
        let resources = self.allocate_vm_resources(vm_type, config)?;
        
        // Create the VM instance
        let instance_id = VmInstanceId::new();
        let instance = SandboxVmInstance {
            id: instance_id.clone(),
            vm_type,
            state: VmState::Initialized,
            resources,
            context: context.clone(),
        };
        
        // Store the instance
        self.instances.insert(instance_id.clone(), instance);
        
        Ok(instance_id)
    }
    
    /// Execute an operation in a VM instance
    pub fn execute_in_instance(&self, 
                             instance_id: &VmInstanceId, 
                             operation: &ResourceOperation, 
                             context: &ExecutionContext) 
        -> Result<ExecutionResult, SandboxError> {
        // Get the VM instance
        let instance = self.instances.get(instance_id)
            .ok_or(SandboxError::InstanceNotFound)?;
        
        // Validate the context against the instance
        self.validate_context_for_instance(context, instance)?;
        
        // Execute the operation
        let vm_adapter = self.create_vm_adapter(instance);
        vm_adapter.execute_operation(operation, context)
            .map_err(|err| SandboxError::ExecutionError(err))
    }
}
```

### Resource Quotas and Limits

The sandbox enforces resource quotas:

```rust
/// Resource quotas for VM instances
pub struct ResourceQuotas {
    /// Maximum memory per instance
    pub max_memory_per_instance: usize,
    /// Maximum CPU time per operation
    pub max_cpu_time_per_operation: Duration,
    /// Maximum storage operations per instance
    pub max_storage_operations: usize,
    /// Maximum network operations per instance
    pub max_network_operations: usize,
}

/// Resource usage tracking
pub struct ResourceUsage {
    /// Memory currently in use
    pub memory_usage: usize,
    /// CPU time used
    pub cpu_time: Duration,
    /// Storage operations performed
    pub storage_operations: usize,
    /// Network operations performed
    pub network_operations: usize,
}

/// Resources allocated to a VM instance
pub struct VmResources {
    /// Memory regions allocated to the VM
    pub memory_regions: Vec<MemoryRegion>,
    /// Resource usage statistics
    pub usage: ResourceUsage,
    /// Resource quota assigned to the VM
    pub quota: ResourceQuotas,
}
```

## Sandbox Security Mechanisms

### Isolation Techniques

The VM Sandbox employs several isolation techniques:

1. **Address Space Isolation**: Each VM gets an isolated address space
2. **Resource Capability Isolation**: Resources are only accessible with capabilities
3. **Effect Handler Isolation**: Effect handlers are constrained by sandbox policies
4. **Domain Isolation**: Operations across domains pass through secure gateways
5. **Time Isolation**: Execution time is limited and monitored

### Security Policy Enforcement

Security policies are strictly enforced:

```rust
/// Sandbox security policy
pub struct SandboxSecurityPolicy {
    /// Allowed effect types
    pub allowed_effects: HashSet<EffectType>,
    /// Allowed resource operations
    pub allowed_operations: HashSet<OperationType>,
    /// Allowed cross-domain targets
    pub allowed_cross_domain: HashSet<DomainId>,
    /// Memory limits
    pub memory_limits: MemoryLimits,
    /// Execution time limits
    pub time_limits: ExecutionLimits,
}

/// Sandbox policy enforcer
pub struct SandboxPolicyEnforcer {
    /// Security policy
    policy: SandboxSecurityPolicy,
}

impl SandboxPolicyEnforcer {
    /// Enforce security policy for an operation
    pub fn enforce_operation_policy(&self, 
                                   operation: &ResourceOperation, 
                                   context: &ExecutionContext) 
        -> Result<(), SandboxError> {
        // Check if operation type is allowed
        if !self.policy.allowed_operations.contains(&operation.operation_type) {
            return Err(SandboxError::OperationNotAllowed);
        }
        
        // Additional policy checks
        self.check_cross_domain_policy(operation, context)?;
        self.check_memory_limits(operation, context)?;
        self.check_time_limits(context)?;
        
        Ok(())
    }
}
```

## Integration with Unified ResourceRegister Model

The VM Sandbox integrates with the ResourceRegister unified model:

```rust
/// Sandboxed resource manager
pub struct SandboxedResourceManager {
    /// Underlying resource manager
    resource_manager: Arc<ResourceManager>,
    /// Security policy enforcer
    policy_enforcer: SandboxPolicyEnforcer,
}

impl SandboxedResourceManager {
    /// Access a ResourceRegister in the sandbox
    pub fn access_resource_register(&self, 
                                   register_id: &RegisterId, 
                                   access_intent: AccessIntent, 
                                   context: &ExecutionContext) 
        -> Result<ResourceRegister, SandboxError> {
        // Enforce security policy
        self.policy_enforcer.enforce_resource_access_policy(
            register_id, 
            access_intent, 
            context
        )?;
        
        // Access the resource through the resource manager
        let register = self.resource_manager.get_resource_register(
            register_id, 
            context
        )?;
        
        Ok(register)
    }
    
    /// Execute a ResourceRegister operation in the sandbox
    pub fn execute_register_operation(&self, 
                                     operation: &ResourceOperation, 
                                     context: &ExecutionContext) 
        -> Result<ExecutionResult, SandboxError> {
        // Enforce security policy
        self.policy_enforcer.enforce_operation_policy(
            operation, 
            context
        )?;
        
        // Execute the operation through the resource manager
        let result = self.resource_manager.execute_operation(
            operation, 
            context
        )?;
        
        Ok(result)
    }
}
```

## Proof Verification in Sandbox

The sandbox includes secure proof verification:

```rust
/// Sandboxed proof verifier
pub struct SandboxedProofVerifier {
    /// Underlying proof verifier
    proof_verifier: Arc<ProofVerifier>,
}

impl SandboxedProofVerifier {
    /// Verify a proof within the sandbox
    pub fn verify_proof(&self, 
                       operation: &ResourceOperation, 
                       proof: &Proof, 
                       context: &ExecutionContext) 
        -> Result<bool, SandboxError> {
        // Verify the proof using the proof verifier
        let valid = self.proof_verifier.verify(
            operation, 
            proof, 
            context
        )?;
        
        Ok(valid)
    }
}
```

## Error Handling

The sandbox includes comprehensive error handling:

```rust
/// Errors that can occur in the VM sandbox
pub enum SandboxError {
    /// Insufficient capabilities for an operation
    InsufficientCapabilities,
    /// Memory access violation
    MemoryAccessViolation,
    /// Resource quota exceeded
    ResourceQuotaExceeded,
    /// Execution time limit exceeded
    TimeoutExceeded,
    /// Effect constraint violation
    ConstraintViolation,
    /// Cross-domain operation not allowed
    CrossDomainNotAllowed,
    /// Operation type not allowed
    OperationNotAllowed,
    /// VM instance not found
    InstanceNotFound,
    /// Resource not accessible
    ResourceNotAccessible,
    /// Execution error from VM
    ExecutionError(ExecutionError),
    /// Effect handling error
    EffectError(EffectError),
}
```

## Sandbox Performance Considerations

The sandbox implementation addresses several performance considerations:

1. **Minimal Overhead**: Optimized VM boundary crossing
2. **Efficient Capability Checks**: Fast capability validation
3. **Memory Management**: Efficient memory allocation and isolation
4. **Resource Pooling**: Reuse of VM instances and resources
5. **Optimized Cross-Domain Operations**: Streamlined cross-domain gateway

## Debugging and Monitoring

The sandbox includes tools for debugging and monitoring:

```rust
/// Sandbox monitoring interface
pub trait SandboxMonitoringInterface {
    /// Get current resource usage for a VM instance
    fn get_resource_usage(&self, instance_id: &VmInstanceId) 
        -> Result<ResourceUsage, SandboxError>;
    
    /// Get execution logs for a VM instance
    fn get_execution_logs(&self, 
                         instance_id: &VmInstanceId, 
                         filter: LogFilter) 
        -> Result<Vec<ExecutionLog>, SandboxError>;
    
    /// Get security events for a VM instance
    fn get_security_events(&self, 
                          instance_id: &VmInstanceId, 
                          filter: SecurityEventFilter) 
        -> Result<Vec<SecurityEvent>, SandboxError>;
    
    /// Get performance metrics for a VM instance
    fn get_performance_metrics(&self, 
                              instance_id: &VmInstanceId) 
        -> Result<PerformanceMetrics, SandboxError>;
}
```

## Related Documentation

- [VM Architecture](vm_architecture.md)
- [VM Execution](vm_execution.md)
- [VM Interface](vm_interface.md)
- [VM Capability Management](vm_capability_management.md)
- [ResourceRegister Unified Model](resource_register_unified_model.md)
- [Unified Operation Model](unified_operation_model.md)
- [Three-Layer Effect Architecture](effect_templates.md)
``` 