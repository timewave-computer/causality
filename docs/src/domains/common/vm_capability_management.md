<!-- Management of VM capabilities -->
<!-- Original file: docs/src/vm_capability_management.md -->

# VM Capability Management

## Overview

The capability management system within the Causality VM architecture provides a secure, principled approach to controlling access to resources and operations. Built on capability-based security principles, it ensures that only authorized entities can access or modify resources within the VM environment and across domain boundaries.

This capability system integrates with the unified resource register model and the three-layer effect architecture to provide consistent, fine-grained access control across all aspects of the system. By enforcing the principle of least privilege, it minimizes the attack surface and creates a robust security foundation.

## Core Concepts

### Capabilities in VM Context

In the Causality VM system, a capability is an unforgeable token that grants specific permissions to access or modify resources. Key properties include:

1. **Unforgeable References**: Capabilities serve as unforgeable references to protected resources
2. **Attenuable Rights**: Capabilities specify precise rights and constraints on what operations can be performed
3. **Delegatable Control**: Capabilities can be delegated to other entities with equal or fewer rights
4. **Contextual Validation**: Capability validation considers execution context, temporal constraints, and domain boundaries

### Capability Types

The VM system supports several capability types:

1. **Resource Capabilities**: Control access to resource registers
2. **Operation Capabilities**: Control who can perform specific operations
3. **Domain Capabilities**: Control interactions with specific domains
4. **VM Capabilities**: Control access to VM features and memory regions

### Rights and Intents

Capabilities grant specific rights for different access intents:

```rust
/// Intent for accessing a resource
pub enum AccessIntent {
    /// Read access
    Read,
    /// Write access
    Write,
    /// Execute access
    Execute,
}

/// Rights that can be granted by a capability
pub enum Right {
    /// Read access to a resource
    Read,
    /// Write access to a resource
    Write,
    /// Execute operations on a resource
    Execute,
    /// Delegate capabilities to others
    Delegate,
    /// Administer a resource
    Admin,
}
```

## Architecture

### Capability Management Components

The VM capability management system consists of several key components:

1. **CapabilityManager**: Central service for capability creation, validation, and revocation
2. **DomainCapabilityManager**: Manages domain-specific capabilities and permission mappings
3. **ResourceVmIntegration**: Enforces capability-based access to resources in VM memory
4. **ExecutionContext**: Provides context for capability validation during operation execution
5. **CapabilityConstraint**: Defines constraints on when and how capabilities can be used

### Integration with VM Memory Model

VM capability management integrates with the memory model through controlled access to VM registers:

```
┌─────────────────────────────┐
│   Application Layer         │
└───────────────┬─────────────┘
                │
┌───────────────▼─────────────┐
│   VM Capability Manager     │
│                             │
│  ┌─────────────────────┐    │
│  │ Capability Registry │    │
│  └─────────────────────┘    │
│                             │
│  ┌─────────────────────┐    │
│  │ Access Validation   │    │
│  └─────────────────────┘    │
└───────────────┬─────────────┘
                │
┌───────────────▼─────────────┐
│   Resource VM Integration   │
│                             │
│  ┌─────────────────────┐    │
│  │ VM Register Mapping │    │
│  └─────────────────────┘    │
│                             │
│  ┌─────────────────────┐    │
│  │ Access Control      │    │
│  └─────────────────────┘    │
└───────────────┬─────────────┘
                │
┌───────────────▼─────────────┐
│   VM Memory Manager         │
│                             │
│  ┌─────────────────────┐    │
│  │ Memory Sections     │    │
│  └─────────────────────┘    │
│                             │
│  ┌─────────────────────┐    │
│  │ Register Storage    │    │
│  └─────────────────────┘    │
└─────────────────────────────┘
```

## Resource Access Control in VM

### Access Check Workflow

When a resource is accessed in the VM, the following workflow ensures proper capability validation:

1. **Resource Identification**: The resource is identified by its ContentId
2. **Capability Lookup**: The system looks up relevant capabilities for the resource
3. **Context Validation**: The capabilities are validated against the current execution context
4. **Access Decision**: Based on the validation, access is granted or denied
5. **Audit Logging**: Resource access attempts are logged for audit purposes

```rust
/// Result of a resource access operation
pub enum ResourceAccessResult {
    /// Resource access was successful
    Success,
    /// Resource access was denied
    AccessDenied(String),
    /// Resource does not exist
    NotFound,
    /// Resource is in an invalid state for the operation
    InvalidState(String),
    /// Internal error during resource access
    InternalError(String),
}
```

### Resource Loading and Storage

The ResourceVmIntegration component handles resource access control during VM operations:

```rust
/// Load a resource into VM memory
pub fn load_resource(
    &mut self,
    resource_id: &ContentId,
    ctx: &ExecutionContext,
    initiator: &Address,
) -> TelResult<VmRegId> {
    // Get the resource register
    let resource_register = self.resource_manager.get_resource_register(resource_id)?;
    
    // Check access permissions using capabilities
    let access_result = self.check_resource_access(
        &resource_register,
        ctx,
        initiator,
        AccessIntent::Read,
    );
    
    if let ResourceAccessResult::Success = access_result {
        // Map the resource to a VM register
        let vm_reg_id = self.map_resource_to_vm(&resource_register, ctx)?;
        Ok(vm_reg_id)
    } else {
        // Access denied
        let error_msg: Result<(), String> = access_result.into();
        Err(TelError::AccessDenied(error_msg.unwrap_err()))
    }
}
```

## Domain Capability Management

The domain capability system manages permissions for cross-domain operations within the VM:

### Domain Capability Types

The VM supports various domain capabilities:

```rust
/// Standard domain capabilities that can be supported by domain adapters
pub enum DomainCapability {
    // Transaction capabilities
    SendTransaction,
    SignTransaction,
    BatchTransactions,
    
    // Smart contract capabilities
    DeployContract,
    ExecuteContract,
    QueryContract,
    
    // State capabilities
    ReadState,
    WriteState,
    
    // Cryptographic capabilities
    VerifySignature,
    GenerateProof,
    VerifyProof,
    
    // ZK capabilities
    ZkProve,
    ZkVerify,
    
    // Consensus capabilities
    Stake,
    Validate,
    Vote,
    
    // Governance capabilities
    ProposeUpgrade,
    VoteOnProposal,
    
    // Cross-domain capabilities
    BridgeAssets,
    VerifyBridgeTransaction,
    
    // Custom capability (with name)
    Custom(String)
}
```

### Domain-Specific Capability Sets

Different domain types provide different capability sets:

| Capability | EVM | CosmWasm | Solana | TEL |
|------------|-----|----------|--------|-----|
| SendTransaction | ✓ | ✓ | ✓ | ✓ |
| DeployContract | ✓ | ✓ | ✓ | - |
| ExecuteContract | ✓ | ✓ | ✓ | ✓ |
| QueryContract | - | ✓ | - | ✓ |
| WriteState | ✓ | ✓ | ✓ | - |
| VerifySignature | ✓ | ✓ | ✓ | - |
| ZkProve | - | - | - | ✓ |
| ZkVerify | - | - | - | ✓ |
| Stake | - | ✓ | ✓ | - |
| Validate | - | ✓ | ✓ | - |

### Domain Capability Creation

The system creates domain-specific capabilities as follows:

```rust
/// Create a capability for using a domain
pub async fn create_domain_capability(
    &self,
    domain_id: &DomainId,
    resource_id: &ContentId,
    owner: &Address,
    issuer: &Address,
    capabilities: &[DomainCapability],
    delegatable: bool,
) -> Result<CapabilityId> {
    // Convert domain capabilities to capability constraints
    let operations: Vec<String> = capabilities
        .iter()
        .map(|cap| cap.to_string())
        .collect();
        
    let domains_constraint = CapabilityConstraint::Domains(vec![domain_id.to_string()]);
    let operations_constraint = CapabilityConstraint::Operations(operations);
    
    // Create the capability constraints
    let constraints = vec![domains_constraint, operations_constraint];
    
    // Create rights for the capability
    let mut rights = HashSet::new();
    rights.insert(Right::Execute);
    
    // Create the capability
    let capability = RigorousCapability {
        id: CapabilityId::new_random(),
        resource_id: resource_id.clone(),
        rights,
        delegated_from: None,
        issuer: issuer.clone(),
        owner: owner.clone(),
        expires_at: None, // No expiration
        revocation_id: Some(format!("domain_capability_{}", domain_id)),
        delegatable,
        constraints,
        proof: None, // No proof required for system-created capabilities
    };
    
    // Create the capability in the capability system
    self.capability_system.create_capability(capability).await
}
```

## VM Execution Context and Capabilities

Different VM execution contexts require different capabilities:

### Execution Environments

The VM supports different execution environments, each with specific capability requirements:

```rust
/// Execution environments
pub enum ExecutionEnvironment {
    /// Abstract environment (logical operations)
    Abstract,
    
    /// Program execution environment
    Program,
    
    /// Register-based environment
    Register,
    
    /// Physical on-chain environment
    OnChain(DomainId),
    
    /// ZK verification environment
    ZkVm,
}
```

### Execution Phases and Capability Requirements

Each execution phase requires specific capabilities:

```rust
/// Execution phases for operations
pub enum ExecutionPhase {
    /// Planning phase (intent formation)
    Planning,
    
    /// Validation phase (checking preconditions)
    Validation,
    
    /// Authorization phase (verifying permissions)
    Authorization,
    
    /// Execution phase (applying changes)
    Execution,
    
    /// Verification phase (confirming effects)
    Verification,
    
    /// Finalization phase (recording outcomes)
    Finalization,
}
```

For example, during the Execution phase in an OnChain environment, capabilities for WriteState and ExecuteContract are typically required, whereas during the Validation phase, only ReadState capabilities might be needed.

## VM Integration Configuration

The VM can be configured with different capability settings:

```rust
/// Configuration for VM resource integration
pub struct VmIntegrationConfig {
    /// Maximum registers per execution context
    pub max_registers_per_context: usize,
    /// Whether to auto-commit changes on context exit
    pub auto_commit_on_exit: bool,
    /// Whether to validate resource access against time system
    pub validate_time_access: bool,
    /// Memory section for resource data
    pub resource_memory_section: String,
}
```

## Delegation and Attenuation

The capability system supports delegation and attenuation of capabilities:

### Capability Delegation

Capabilities can be delegated to other entities with equal or fewer permissions:

1. **Parent-Child Relationship**: Delegated capabilities maintain a link to their parent
2. **Rights Reduction**: Delegated capabilities can have fewer rights than the parent, never more
3. **Constraint Addition**: Additional constraints can be added during delegation
4. **Revocation Chain**: Revoking a parent capability automatically revokes all delegated children

### Capability Attenuation

Capabilities can be attenuated along several dimensions:

1. **Right Reduction**: Removing specific rights (e.g., removing Write while keeping Read)
2. **Temporal Constraints**: Adding time-based limitations (e.g., expiration dates)
3. **Domain Constraints**: Limiting to specific domains
4. **Operation Constraints**: Limiting to specific operations
5. **Parameter Constraints**: Limiting parameter values for operations

## Cross-Domain Capability Management

For operations spanning multiple domains, the VM uses a coordinated capability system:

### Cross-Domain Capability Chains

1. **Capability Linking**: Capabilities across domains are linked by a common identifier
2. **Proof Verification**: ZK proofs verify capability validity across domains
3. **Atomic Validation**: Capabilities are validated atomically for cross-domain operations
4. **Chain Integrity**: Cryptographic techniques ensure the integrity of capability chains

### Verification in ZK-VM

For operations requiring zero-knowledge proofs:

1. **Capability Encoding**: Capabilities are encoded in circuit-friendly formats
2. **In-Circuit Verification**: Capability validation occurs within ZK circuits
3. **Witness Data**: Capability data is provided as witness data to circuits
4. **Proof Generation**: ZK proofs attest to capability validation

## Security Considerations

### Protection Against Common Attacks

The VM capability system protects against:

1. **Confused Deputy**: By using unforgeable object capabilities instead of ambient authority
2. **Privilege Escalation**: Through fine-grained permission control and attenuation
3. **Capability Forgery**: Through cryptographic verification of capability provenance
4. **Replay Attacks**: By incorporating nonces or temporal constraints
5. **Covert Channels**: By isolating capability domains and execution contexts

### Audit and Monitoring

The system includes extensive audit and monitoring features:

1. **Access Logging**: All capability uses are logged for audit
2. **Capability Tracking**: Capability provenance and delegation chains are tracked
3. **Usage Analytics**: Patterns of capability usage are analyzed for anomalies
4. **Revocation Monitoring**: Revocation propagation is monitored for completeness

## Implementation Status

The VM capability management system is currently implemented with the following components:

1. **ResourceVmIntegration**: Fully implemented for TEL resource operations
2. **DomainCapabilityManager**: Fully implemented for domain capability management
3. **Execution Contexts**: Fully implemented with capability requirements
4. **Cross-Domain Capabilities**: Partially implemented, with ongoing enhancements
5. **ZK-VM Capability Verification**: Partially implemented, with ongoing development

## Related Documentation

- [VM Architecture](vm_architecture.md)
- [VM Execution](vm_execution.md)
- [VM Interface](vm_interface.md)
- [VM Sandbox](vm_sandbox.md)
- [Capability Based Authorization](capability_based_authorization.md)
- [Security Capability Model](security_capability_model.md)
- [Unified Operation Model](unified_operation_model.md)
- [Resource Register Unified Model](resource_register_unified_model.md)