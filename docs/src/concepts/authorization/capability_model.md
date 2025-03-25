<!-- Model for capabilities in the system -->
<!-- Original file: docs/src/capability_model.md -->

# Capability-Based Authorization Model

This document outlines the capability-based authorization model in Causality, which provides rigorous, fine-grained control over resource access and operations.

## Core Concepts

### Capabilities

A **Capability** is an unforgeable token of authority that grants specific rights to perform operations on specific targets, potentially with constraints. Capabilities have three key components:

1. **Rights**: What operations are allowed
2. **Targets**: What resources can be operated on
3. **Constraints**: Limitations on how the capability can be used

This model shifts from identity-based permissions ("who you are") to capability-based permissions ("what you can do"), providing clearer security boundaries and more precise access control.

### Structure

```rust
/// A capability that grants specific rights over specific targets
struct Capability {
    /// Unique identifier for the capability
    id: CapabilityId,
    
    /// The rights this capability grants
    rights: Rights,
    
    /// The targets this capability applies to
    targets: Targets,
    
    /// Optional constraints on how this capability can be used
    constraints: Option<CapabilityConstraints>,
    
    /// Who created this capability
    issuer: Identity,
    
    /// When this capability was created
    created_at: Timestamp,
    
    /// When this capability expires (if ever)
    expires_at: Option<Timestamp>,
    
    /// Whether this capability can be delegated
    delegable: bool,
    
    /// Metadata for the capability
    metadata: Metadata,
}

/// Rights that can be granted by a capability
struct Rights {
    /// The set of allowed operations
    allowed_operations: HashSet<Right>,
}

/// Specific rights that can be granted
enum Right {
    /// View the resource
    View,
    
    /// Update the resource
    Update,
    
    /// Transfer ownership of the resource
    Transfer,
    
    /// Lock the resource
    Lock,
    
    /// Unlock the resource
    Unlock,
    
    /// Freeze the resource
    Freeze,
    
    /// Unfreeze the resource
    Unfreeze,
    
    /// Consume the resource
    Consume,
    
    /// Create a new resource
    Create,
    
    /// Define relationships with other resources
    DefineRelationship,
    
    /// Delegate this capability to others
    Delegate,
    
    /// Custom rights for domain-specific operations
    Custom(String),
}

/// Targets that capabilities can apply to
enum Targets {
    /// A specific resource
    Resource(ResourceId),
    
    /// A group of resources
    ResourceGroup(ResourceGroupId),
    
    /// All resources of a particular type
    ResourceType(ResourceType),
    
    /// All resources in a domain
    Domain(DomainId),
    
    /// A specific operation
    Operation(OperationType),
    
    /// A specific program
    Program(ProgramId),
    
    /// A custom target
    Custom(String, Value),
}

/// Constraints on how a capability can be used
struct CapabilityConstraints {
    /// Maximum quantity that can be affected by operations
    max_quantity: Option<u64>,
    
    /// Maximum number of times this capability can be used
    max_uses: Option<u32>,
    
    /// When this capability becomes valid
    valid_from: Option<Timestamp>,
    
    /// When this capability expires
    valid_until: Option<Timestamp>,
    
    /// Specific contexts where this capability is valid
    valid_contexts: Option<HashSet<String>>,
    
    /// Custom constraints as key-value pairs
    custom_constraints: HashMap<String, Value>,
}
```

## Authorization Process

Capabilities are used in the authorization process as follows:

1. **Creation**: Capabilities are created by authorities (e.g., resource owners)
2. **Distribution**: Capabilities are distributed to agents who need to perform operations
3. **Presentation**: Agents present capabilities when invoking operations
4. **Validation**: The system validates that the presented capabilities authorize the requested operation
5. **Execution**: If validation succeeds, the operation is executed

## Capability Validation

When an operation is attempted, the authorization system validates:

1. **Presence**: Does the invoker have a capability for this operation?
2. **Rights**: Does the capability grant the required rights?
3. **Target**: Does the capability apply to the target resource?
4. **Constraints**: Are all constraints satisfied?
5. **Expiration**: Has the capability expired?
6. **Delegation**: If delegated, was the delegation valid?

## Capability Delegation

Capabilities can be delegated to enable secure capability transfer between entities:

```rust
/// Create a delegated capability
let delegated_capability = capability.delegate(
    recipient,
    DelegationConstraints::new()
        .with_expiration(time::now() + Duration::hours(1))
        .with_max_uses(1)
)?;
```

Key aspects of delegation:

1. **Attenuation**: Delegated capabilities can be more restricted but never less restricted than the original
2. **Chain of Trust**: Delegation creates an auditable chain of trust
3. **Revocation**: Delegations can be revoked, invalidating all derived capabilities
4. **Constraints**: Delegations can add constraints to limit usage

## Integration with Resource System

Capabilities integrate with the resource system through:

1. **Resource Operations**: All resource operations require appropriate capabilities
2. **Lifecycle Management**: State transitions require specific capabilities
3. **Relationship Management**: Establishing relationships requires capabilities for both resources
4. **Resource Logic**: Resource-specific logic can define custom capability requirements

## Usage Examples

### Basic Capability Creation

```rust
// Create a capability to transfer a resource
let transfer_capability = Capability::new(
    Rights::from([Right::Transfer]),
    Targets::Resource(resource.id.clone()),
    Some(CapabilityConstraints::new()
        .with_max_quantity(100)
        .with_expiration(time::now() + Duration::hours(24)))
);

// Create an operation with this capability
let operation = Operation::new(OperationType::Transfer)
    .with_input(source_resource.clone())
    .with_output(destination_resource.clone())
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::with_capabilities(
        invoker.clone(),
        vec![transfer_capability]
    ));

// Validate and execute the operation
let validation_result = validator.validate(&operation)?;
if validation_result.is_valid {
    let result = execute_operation(operation, &context).await?;
    // Process the result
}
```

### Resource Owner Capabilities

```rust
// When creating a resource, the creator automatically gets owner capabilities
let resource_operation = Operation::new(OperationType::Create)
    .with_output(ResourceRegister::new(
        "resource1".to_string(),
        ResourceProperties::new()
            .with_fungibility_domain("token")
            .with_quantity(100),
        FungibleTokenLogic::new(),
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    ))
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(creator.clone()));

let result = execute_operation(resource_operation, &context).await?;

// The creator now automatically has full capabilities for the resource
// Let's get those capabilities
let owner_capabilities = capability_registry.get_capabilities_for_owner(
    creator.id(), 
    result.outputs[0].id()
)?;

// These can be used for any operation on the resource
```

### Capability Delegation with Constraints

```rust
// Delegate transfer capability to another user with constraints
let delegated_capability = owner_capabilities
    .get(Right::Transfer)
    .delegate(
        recipient.clone(),
        DelegationConstraints::new()
            .with_max_quantity(10)
            .with_max_uses(1)
            .with_expiration(time::now() + Duration::hours(1))
    )?;

// The recipient can now use this capability for a transfer
let transfer_operation = Operation::new(OperationType::Transfer)
    .with_input(resource.clone())
    .with_output(resource.with_owner(new_owner.clone()))
    .with_parameter("amount", 10)
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::with_capabilities(
        recipient.clone(),
        vec![delegated_capability]
    ));

// This operation will succeed within the constraints
let result = execute_operation(transfer_operation, &context).await?;

// A second attempt would fail due to max_uses=1
let second_attempt = execute_operation(transfer_operation, &context).await;
assert!(second_attempt.is_err());

// An attempt to transfer more than 10 would also fail
let excessive_transfer = Operation::new(OperationType::Transfer)
    .with_input(resource.clone())
    .with_output(resource.with_owner(new_owner.clone()))
    .with_parameter("amount", 20)
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::with_capabilities(
        recipient.clone(),
        vec![delegated_capability]
    ));

let excessive_result = execute_operation(excessive_transfer, &context).await;
assert!(excessive_result.is_err());
```

### Group-Based Capabilities

```rust
// Create a resource group
let resource_group = ResourceGroup::new("fungible_tokens");
resource_group.add_resource(token1.id())?;
resource_group.add_resource(token2.id())?;
resource_group.add_resource(token3.id())?;

// Create a capability for the entire group
let group_capability = Capability::new(
    Rights::from([Right::View, Right::Transfer]),
    Targets::ResourceGroup(resource_group.id()),
    None
);

// An operation targeting any resource in the group can use this capability
let operation = Operation::new(OperationType::Transfer)
    .with_input(token2.clone())
    .with_output(token2.with_owner(recipient.clone()))
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::with_capabilities(
        invoker.clone(),
        vec![group_capability]
    ));

// This will succeed since token2 is in the group
let result = execute_operation(operation, &context).await?;
```

## Capability Verification

Operations involving capabilities undergo multi-stage verification:

```rust
// Verify capabilities for an operation
fn verify_capabilities(operation: &Operation, capability_verifier: &CapabilityVerifier) -> Result<VerificationResult> {
    // 1. Extract capabilities from the operation
    let capabilities = operation.authorization().capabilities();
    
    // 2. Verify each capability applies to the operation
    for capability in capabilities {
        // Check rights match the operation type
        if !capability_verifier.has_right_for_operation(capability, operation)? {
            return Ok(VerificationResult::failed("Missing required rights"));
        }
        
        // Check targets match the operation resources
        if !capability_verifier.targets_match_resources(capability, operation)? {
            return Ok(VerificationResult::failed("Capability doesn't target these resources"));
        }
        
        // Check constraints are satisfied
        if !capability_verifier.constraints_satisfied(capability, operation)? {
            return Ok(VerificationResult::failed("Capability constraints not satisfied"));
        }
        
        // Check capability hasn't expired
        if capability_verifier.is_expired(capability)? {
            return Ok(VerificationResult::failed("Capability has expired"));
        }
    }
    
    // All capability checks passed
    Ok(VerificationResult::success())
}
```

## Best Practices

1. **Least Privilege**: Grant the minimum capabilities needed for each agent.

2. **Capability Attenuation**: When delegating, always add constraints to reduce scope.

3. **Short Lifetimes**: Use short expiration times for sensitive capabilities.

4. **Capability Composition**: Compose multiple specific capabilities rather than using broad ones.

5. **Revocation Strategy**: Plan for capability revocation in your security model.

6. **Audit Trail**: Maintain an audit trail of capability issuance and use.

7. **Secure Distribution**: Ensure secure mechanisms for distributing capabilities.

8. **Validation First**: Always validate capabilities before executing operations.

9. **Delegation Control**: Carefully control which capabilities can be delegated.

10. **Context-Aware Validation**: Consider execution context when validating capabilities.

## Implementation Status

The capability-based authorization model is fully implemented in the Causality system:

- ✅ Core `Capability` data structure
- ✅ Rights and targets model
- ✅ Constraints system
- ✅ Delegation functionality
- ✅ Integration with the Operation model
- ✅ Verification framework

## Future Enhancements

1. **Capability Revocation**: More sophisticated revocation mechanisms
2. **Compound Capabilities**: Capabilities that represent combinations of other capabilities
3. **Capability Inference**: Automatic inference of required capabilities from operations
4. **Threshold Capabilities**: Capabilities that require multiple holders to approve
5. **Capability Negotiation**: Protocols for secure capability negotiation between entities 