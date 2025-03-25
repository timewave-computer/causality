<!-- Documentation on capability-based authorization -->
<!-- Original file: docs/src/capability_based_authorization.md -->

# Capability-Based Authorization Model

This document outlines the capability-based authorization model within the unified resource architecture, focusing on how capabilities are used to control access to resources and operations in the Causality system.

## Core Concepts

### Capability Model

Capabilities in the Causality system are unforgeable tokens that grant specific rights to perform operations on specific targets. Key characteristics include:

1. **Unforgeable**: Capabilities cannot be fabricated, only delegated
2. **Revocable**: Capabilities can be revoked by their issuer
3. **Transferable**: Capabilities can be delegated to other entities
4. **Fine-grained**: Capabilities provide precise control over permitted actions
5. **Contextual**: Capabilities can include conditions for their use

This capability model provides stronger security guarantees than traditional role-based access control by following the principle of least privilege and reducing the attack surface.

### Rights and Targets

Capabilities combine rights with targets:

1. **Rights**: Specific actions that can be performed (e.g., Create, Read, Update, Delete)
2. **Targets**: Resources or groups of resources that the rights apply to

### Capability Registry

The `CapabilityRegistry` manages the issuance, validation, and revocation of capabilities:

1. **Issuance**: Registering capabilities for entities
2. **Validation**: Verifying that an entity possesses required capabilities
3. **Revocation**: Removing capabilities when no longer needed
4. **Delegation**: Managing the transfer of capabilities between entities

## Structure

```rust
/// A capability granting rights to a target
pub struct Capability {
    /// Unique identifier for the capability
    id: CapabilityId,
    
    /// The rights granted by this capability
    rights: Rights,
    
    /// The target of this capability
    target: Targets,
    
    /// Optional constraints on when/how this capability can be used
    constraints: Option<CapabilityConstraints>,
    
    /// Metadata associated with this capability
    metadata: Option<MetadataMap>,
}

/// Rights that can be granted
pub struct Rights {
    /// The set of rights
    rights: HashSet<Right>,
}

/// Individual right types
pub enum Right {
    /// Right to create a resource
    Create,
    
    /// Right to read a resource
    Read,
    
    /// Right to update a resource
    Update,
    
    /// Right to delete a resource
    Delete,
    
    /// Right to transfer a resource
    Transfer,
    
    /// Right to freeze a resource
    Freeze,
    
    /// Right to unfreeze a resource
    Unfreeze,
    
    /// Right to lock a resource
    Lock,
    
    /// Right to unlock a resource
    Unlock,
    
    /// Right to consume a resource
    Consume,
    
    /// Right to archive a resource
    Archive,
    
    /// Right to delegate capabilities
    Delegate,
    
    /// Right to revoke capabilities
    Revoke,
    
    /// Right to administer capabilities
    Admin,
    
    /// Custom right
    Custom(String),
}

/// Targets for capabilities
pub enum Targets {
    /// Single resource target
    Resource(ResourceId),
    
    /// Resource type target (applies to all resources of a type)
    ResourceType(ResourceType),
    
    /// Domain target (applies to all resources in a domain)
    Domain(DomainId),
    
    /// Resource group target
    ResourceGroup(ResourceGroupId),
    
    /// Operation type target
    Operation(OperationType),
    
    /// Template target (applies to all resources created from a template)
    Template(TemplateId),
    
    /// Custom target
    Custom { type_id: String, target_id: String },
}

/// Constraints on capability usage
pub struct CapabilityConstraints {
    /// Time constraints on when the capability can be used
    time_constraints: Option<TimeConstraints>,
    
    /// Resource state constraints
    state_constraints: Option<StateConstraints>,
    
    /// Network constraints
    network_constraints: Option<NetworkConstraints>,
    
    /// Custom constraints expressed as a predicate
    custom_constraints: Option<Box<dyn CapabilityPredicate>>,
}

/// Registry for capabilities
pub struct CapabilityRegistry {
    /// Map of entity IDs to their capabilities
    entity_capabilities: HashMap<ResourceId, HashMap<CapabilityId, Capability>>,
    
    /// Capability delegations
    delegations: HashMap<CapabilityId, Vec<Delegation>>,
    
    /// Capability revocations
    revocations: HashSet<CapabilityId>,
    
    /// Configuration for the registry
    config: CapabilityRegistryConfig,
}
```

## Integration with Resource System

The capability model integrates with the unified resource system:

1. **Resource Lifecycle Manager**: Uses capabilities to validate state transitions
2. **Operation Validation**: Validates operations against required capabilities
3. **Effect Templates**: Templates embed capability requirements for operations
4. **Resource Register**: Applies capability checks during resource operations

## Usage Examples

### Basic Capability Management

```rust
// Create a capability registry
let mut capability_registry = CapabilityRegistry::new(
    CapabilityRegistryConfig::default()
);

// Create a capability for resource creation
let create_capability = Capability::new(
    Rights::from([Right::Create]),
    Targets::ResourceType(ResourceType::FungibleToken),
    None
);

// Register the capability for an entity
capability_registry.register_capability(
    create_capability.clone(),
    entity_id.clone()
)?;

// Check if entity has a capability
let has_capability = capability_registry.has_capability(
    &entity_id,
    &Right::Create,
    &Targets::ResourceType(ResourceType::FungibleToken)
)?;
assert!(has_capability);

// Revoke the capability
capability_registry.revoke_capability(
    &create_capability.id,
    &admin_id
)?;

// Verification should now fail
let has_capability = capability_registry.has_capability(
    &entity_id,
    &Right::Create,
    &Targets::ResourceType(ResourceType::FungibleToken)
)?;
assert!(!has_capability);
```

### Capability-Based Operations

```rust
// Create capabilities for resource operations
let transfer_capability = Capability::new(
    Rights::from([Right::Transfer]),
    Targets::Resource(resource_id.clone()),
    None
);

// Register the capability
capability_registry.register_capability(
    transfer_capability.clone(),
    sender_id.clone()
)?;

// Create an operation with the capability
let operation = Operation::new(OperationType::TransferResource)
    .with_input(resource.clone())
    .with_output(resource.with_owner(recipient_id.clone()))
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::with_capabilities(
        sender_id.clone(),
        vec![transfer_capability]
    ));

// Validate the operation
let validation_result = operation_validator.validate(&operation)?;
if validation_result.is_valid {
    // Proceed with operation
    execute_operation(operation, &context).await?;
}
```

### Constrained Capabilities

```rust
// Create a time-constrained capability
let time_constraints = TimeConstraints {
    valid_from: Some(time::now()),
    valid_until: Some(time::now() + Duration::days(7)),
    allowed_times: Some(vec![
        TimeRange::new(
            time::Time::from_hms(9, 0, 0).unwrap(),
            time::Time::from_hms(17, 0, 0).unwrap()
        ),
    ]),
};

let state_constraints = StateConstraints {
    required_states: vec![RegisterState::Active],
    forbidden_states: vec![RegisterState::Frozen, RegisterState::Locked],
};

let constrained_capability = Capability::new(
    Rights::from([Right::Update]),
    Targets::Resource(resource_id.clone()),
    Some(CapabilityConstraints {
        time_constraints: Some(time_constraints),
        state_constraints: Some(state_constraints),
        network_constraints: None,
        custom_constraints: None,
    })
);

// Register the constrained capability
capability_registry.register_capability(
    constrained_capability.clone(),
    entity_id.clone()
)?;

// Validation will now check constraints
let is_valid = capability_registry.validate_capability_use(
    &constrained_capability,
    &entity_id,
    &ValidationContext::new()
        .with_current_time(time::now())
        .with_resource_state(&resource_id, RegisterState::Active)
)?;

assert!(is_valid);
```

### Capability Delegation

```rust
// Create a capability with delegation rights
let delegatable_capability = Capability::new(
    Rights::from([Right::Read, Right::Delegate]),
    Targets::Resource(resource_id.clone()),
    None
);

// Register the capability
capability_registry.register_capability(
    delegatable_capability.clone(),
    delegator_id.clone()
)?;

// Delegate a subset of the capability to another entity
let delegated_capability = delegatable_capability
    .derive_with_rights(Rights::from([Right::Read]));

capability_registry.delegate_capability(
    &delegated_capability,
    &delegator_id,
    &delegate_id,
    Some(Duration::days(3)) // Delegation expires after 3 days
)?;

// The delegate now has read access
let has_read = capability_registry.has_capability(
    &delegate_id,
    &Right::Read,
    &Targets::Resource(resource_id.clone())
)?;
assert!(has_read);

// But does not have delegation rights
let has_delegate = capability_registry.has_capability(
    &delegate_id,
    &Right::Delegate,
    &Targets::Resource(resource_id.clone())
)?;
assert!(!has_delegate);

// Revoke delegated capability
capability_registry.revoke_delegation(
    &delegated_capability.id,
    &delegator_id
)?;

// The delegate no longer has access
let has_read = capability_registry.has_capability(
    &delegate_id,
    &Right::Read,
    &Targets::Resource(resource_id.clone())
)?;
assert!(!has_read);
```

### Compound Capabilities

```rust
// Create a capability that applies to multiple resources
let resource_group_id = ResourceGroupId::new();

// Add resources to the group
resource_group_registry.add_to_group(
    &resource_group_id,
    &resource1_id
)?;
resource_group_registry.add_to_group(
    &resource_group_id,
    &resource2_id
)?;

// Create a capability for the group
let group_capability = Capability::new(
    Rights::from([Right::Read, Right::Update]),
    Targets::ResourceGroup(resource_group_id.clone()),
    None
);

// Register the capability
capability_registry.register_capability(
    group_capability.clone(),
    entity_id.clone()
)?;

// Entity now has capabilities for all resources in the group
let has_capability1 = capability_registry.has_capability(
    &entity_id,
    &Right::Read,
    &Targets::Resource(resource1_id.clone())
)?;
assert!(has_capability1);

let has_capability2 = capability_registry.has_capability(
    &entity_id,
    &Right::Update,
    &Targets::Resource(resource2_id.clone())
)?;
assert!(has_capability2);
```

### Capability Amplification

```rust
// Define a custom capability predicate for amplification
struct AmplificationPredicate;

impl CapabilityPredicate for AmplificationPredicate {
    fn evaluate(
        &self,
        capability: &Capability,
        context: &ValidationContext,
    ) -> Result<bool> {
        // Example: Amplify capabilities when a specific condition is met
        if let Some(resource_id) = context.get_current_resource_id() {
            if let Some(metadata) = context.get_resource_metadata(resource_id)? {
                if let Some(trust_level) = metadata.get::<u32>("trust_level") {
                    // If trust level is high, amplify capabilities
                    if *trust_level >= 8 {
                        return Ok(true);
                    }
                }
            }
        }
        
        // Otherwise, just check if the capability natively covers the request
        Ok(false)
    }
}

// Create a capability with the custom predicate
let amplifiable_capability = Capability::new(
    Rights::from([Right::Read]),
    Targets::Resource(resource_id.clone()),
    Some(CapabilityConstraints {
        time_constraints: None,
        state_constraints: None,
        network_constraints: None,
        custom_constraints: Some(Box::new(AmplificationPredicate)),
    })
);

// Register the capability
capability_registry.register_capability(
    amplifiable_capability.clone(),
    entity_id.clone()
)?;

// In a high-trust context, the capability will be amplified
let context = ValidationContext::new()
    .with_current_resource_id(resource_id.clone())
    .with_resource_metadata(
        &resource_id,
        metadata_map! {
            "trust_level" => 9u32,
        }
    );

// Check for a capability that isn't directly granted
let has_update = capability_registry.validate_capability_use_with_context(
    &entity_id,
    &Right::Update,
    &Targets::Resource(resource_id.clone()),
    &context
)?;

// The capability is amplified in this context
assert!(has_update);
```

### Batch Capability Operations

```rust
// Create multiple capabilities
let capabilities = vec![
    Capability::new(
        Rights::from([Right::Read]),
        Targets::Resource(resource1_id.clone()),
        None
    ),
    Capability::new(
        Rights::from([Right::Update]),
        Targets::Resource(resource2_id.clone()),
        None
    ),
    Capability::new(
        Rights::from([Right::Delete]),
        Targets::Resource(resource3_id.clone()),
        None
    ),
];

// Register capabilities in batch
capability_registry.register_capabilities(
    capabilities.clone(),
    entity_id.clone()
)?;

// Validate multiple capabilities at once
let validation_results = capability_registry.validate_capabilities(
    &entity_id,
    &[
        (Right::Read, Targets::Resource(resource1_id.clone())),
        (Right::Update, Targets::Resource(resource2_id.clone())),
        (Right::Delete, Targets::Resource(resource3_id.clone())),
    ]
)?;

// All validations should succeed
assert!(validation_results.iter().all(|r| *r));
```

### Capability-Based Architecture

```rust
// Example of a capability-based architecture pattern

// 1. Create a domain with capability-based access
let domain = Domain::new(DomainType::FungibleTokens);

// 2. Setup capability bootstrap
let bootstrap_capability = Capability::new(
    Rights::from([Right::Admin]),
    Targets::Domain(domain.id().clone()),
    None
);

// Register the bootstrap capability to the admin
capability_registry.register_capability(
    bootstrap_capability.clone(),
    admin_id.clone()
)?;

// 3. Create domain-specific capabilities
let domain_capabilities = vec![
    Capability::new(
        Rights::from([Right::Create, Right::Read]),
        Targets::ResourceType(ResourceType::FungibleToken),
        None
    ),
    Capability::new(
        Rights::from([Right::Transfer]),
        Targets::Operation(OperationType::TransferToken),
        None
    ),
];

// 4. Delegate capabilities to domain users
for capability in domain_capabilities {
    capability_registry.delegate_capability(
        &capability,
        &admin_id,
        &user_id,
        None // No expiration
    )?;
}

// 5. User performs operations with capabilities
let token_operation = Operation::new(OperationType::CreateToken)
    .with_authorization(Authorization::from_entity_with_context(
        user_id.clone(),
        &capability_registry
    ));

// Operation will be validated against user's capabilities
let validation_result = operation_validator.validate(&token_operation)?;
assert!(validation_result.is_valid);
```

## Capability Chains

Capabilities can form chains of delegation that establish provenance:

```rust
// Get the delegation chain for a capability
let delegation_chain = capability_registry.get_delegation_chain(
    &capability_id
)?;

// Visualize the delegation chain
println!("Capability delegation chain:");
for (level, delegation) in delegation_chain.iter().enumerate() {
    println!("Level {}: {} -> {}", 
             level,
             delegation.delegator, 
             delegation.delegate);
}

// Validate the entire chain
let is_chain_valid = capability_registry.validate_delegation_chain(
    &delegation_chain
)?;
assert!(is_chain_valid);
```

## Best Practices

1. **Principle of Least Privilege**: Grant only the minimal set of capabilities needed.

2. **Temporal Constraints**: Use time-based constraints to limit capability validity periods.

3. **Avoid Ambient Authority**: Never rely on "who you are" but rather "what capabilities you have."

4. **Revocation Planning**: Design capability systems with revocation in mind.

5. **Capability Composition**: Build complex behaviors by composing simple capabilities.

6. **Avoid Capability Leakage**: Ensure capabilities cannot be inadvertently exposed.

7. **Delegation Control**: Carefully control delegation rights to prevent unwanted capability spread.

8. **Attenuation Not Amplification**: Prefer to restrict capabilities over amplifying them.

9. **Context-Aware Validation**: Include relevant context when validating capabilities.

10. **Audit Capability Usage**: Keep comprehensive logs of capability issuance and usage.

## Capability-Based Security Principles

1. **Unforgeable References**: Capabilities must be unforgeable tokens.

2. **Attenuation**: Delegated capabilities should be equal to or more restrictive than their parent.

3. **Delegation**: Capability holders should be able to delegate their capabilities to others.

4. **Revocation**: Capability issuers should be able to revoke capabilities.

5. **Confinement**: Capabilities should be contained within their designated security domains.

6. **No Ambient Authority**: All authority must be explicitly granted via capabilities.

7. **Universal Access**: Every resource access must go through capability validation.

## Implementation Status

The capability-based authorization system is fully implemented in the Causality system:

- ✅ Core `CapabilityRegistry` structure
- ✅ All capability types and operations
- ✅ Constraint-based capabilities
- ✅ Capability delegation
- ✅ Integration with the operation model
- ✅ Capability validation framework
- ✅ Batch operations
- ✅ Audit logging

## Future Enhancements

1. **Distributed Capabilities**: Support for cross-system capability recognition
2. **Capability Attenuation DSL**: Domain-specific language for attenuating capabilities
3. **Zero-Knowledge Proofs**: Capability verification without revealing capability details
4. **Capability Federation**: Federation of capability systems across domains
5. **Formal Verification**: Formal verification of capability security properties
6. **Capability Analysis Tools**: Tools for analyzing capability graphs and security properties
7. **Capability Templates**: Standardized templates for common capability patterns
8. **Capability Monitoring**: Real-time monitoring of capability usage and anomalies 