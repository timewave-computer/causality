<!-- Ownership of resources -->
<!-- Original file: docs/src/resource_ownership.md -->

# Resource Ownership Model in Causality

## Overview

This document details the ownership model for resources within the Causality architecture. Ownership represents the relationship between entities (accounts, programs, or other resources) and the resources they control. The ownership model defines how resources are possessed, transferred, shared, and controlled throughout their lifecycle.

## Core Concepts

### Ownership Types

Causality supports multiple ownership types to accommodate different use cases:

```rust
pub enum OwnershipType {
    /// Single entity has exclusive ownership
    Exclusive,
    
    /// Multiple entities share ownership with equal rights
    Shared {
        /// List of co-owners
        owners: Vec<OwnerId>,
        /// Required signatures for operations (e.g., M-of-N)
        signature_threshold: u32,
    },
    
    /// Resource is owned by a program and governed by its logic
    Programmatic {
        /// Program that controls the resource
        program_id: ProgramId,
        /// Additional control parameters
        parameters: Vec<u8>,
    },
    
    /// Resource is bound to another resource and shares its ownership
    Bound {
        /// ID of the parent resource
        parent_resource_id: ResourceId,
    },
    
    /// Custom ownership model with specialized rules
    Custom {
        /// Type identifier for the custom ownership model
        model_type: String,
        /// Arbitrary data for the custom model
        data: Vec<u8>,
    },
}
```

### Owner Identifiers

Owners in Causality can be various types of entities:

```rust
/// Represents any entity that can own resources
pub enum OwnerId {
    /// Standard user account
    Account(AccountId),
    
    /// Smart contract or program
    Program(ProgramId),
    
    /// Another resource
    Resource(ResourceId),
    
    /// An organization or group identity
    Organization(OrganizationId),
    
    /// Custom owner type
    Custom {
        /// Type identifier
        owner_type: String,
        /// Owner identifier
        id: Vec<u8>,
    },
}
```

### Ownership Claims

Ownership is established and verified through cryptographic claims:

```rust
pub struct OwnershipClaim {
    /// The resource being claimed
    resource_id: ResourceId,
    
    /// The entity claiming ownership
    owner_id: OwnerId,
    
    /// Type of ownership being claimed
    ownership_type: OwnershipType,
    
    /// When the ownership was established
    established_at: Timestamp,
    
    /// Optional expiration of the ownership
    expires_at: Option<Timestamp>,
    
    /// Cryptographic proof of the claim
    proof: OwnershipProof,
}
```

## Ownership Establishment

### Initial Ownership Assignment

When a resource is first created, initial ownership is assigned:

```rust
/// Register a new resource with initial ownership
pub fn register_resource_with_ownership(
    resource_type: ResourceType,
    owner_id: OwnerId,
    ownership_type: OwnershipType,
    initial_attributes: HashMap<String, Value>,
) -> Result<ResourceId, ResourceError> {
    // Generate a unique resource ID
    let resource_id = ResourceId::generate();
    
    // Create resource entry in the registry
    registry.create_resource(resource_id, resource_type, initial_attributes)?;
    
    // Establish initial ownership
    ownership_manager.establish_ownership(
        resource_id,
        owner_id,
        ownership_type,
        None, // No expiration for initial ownership
        system.current_time(),
    )?;
    
    // Log the registration event with ownership information
    event_log.record(ResourceLifecycleEvent::Registered {
        resource_id,
        resource_type,
        owner: owner_id,
        timestamp: system.current_time(),
    });
    
    Ok(resource_id)
}
```

### Ownership Proofs

Ownership is proven through cryptographic means:

```rust
pub enum OwnershipProof {
    /// Standard signature-based proof
    Signature {
        /// Signature data
        signature: Vec<u8>,
        /// Public key that can verify the signature
        public_key: PublicKey,
    },
    
    /// Multi-signature proof
    MultiSignature {
        /// Multiple signatures
        signatures: Vec<(PublicKey, Vec<u8>)>,
        /// Signature verification threshold
        threshold: u32,
    },
    
    /// Zero-knowledge proof of ownership
    ZkProof {
        /// ZK proof data
        proof: ZkProof,
    },
    
    /// Proof by delegation from another entity
    Delegation {
        /// Entity that delegated ownership
        delegator: OwnerId,
        /// Proof of delegation
        delegation_proof: Box<OwnershipProof>,
    },
    
    /// Ownership established by system rule
    SystemRule {
        /// Rule identifier
        rule_id: String,
    },
}
```

## Ownership Operations

### Transferring Ownership

Ownership can be transferred between entities:

```rust
/// Transfer exclusive ownership of a resource
pub fn transfer_exclusive_ownership(
    resource_id: ResourceId,
    new_owner: OwnerId,
    auth_context: AuthContext,
) -> Result<(), OwnershipError> {
    // Verify the resource exists
    let resource = registry.get_resource(resource_id)?;
    
    // Get current ownership information
    let current_ownership = ownership_manager.get_ownership(resource_id)?;
    
    // Verify this is exclusive ownership
    if !matches!(current_ownership.ownership_type, OwnershipType::Exclusive) {
        return Err(OwnershipError::NotExclusiveOwnership);
    }
    
    // Verify the requester is the current owner
    if !ownership_manager.verify_ownership(
        resource_id, 
        auth_context.origin(), 
        &current_ownership.ownership_type
    ) {
        return Err(OwnershipError::NotOwner);
    }
    
    // Verify authorization
    if !auth_system.authorize_ownership_transfer(resource_id, new_owner, &auth_context) {
        return Err(OwnershipError::Unauthorized);
    }
    
    // Revoke current ownership
    ownership_manager.revoke_ownership(resource_id, current_ownership.owner_id)?;
    
    // Establish new ownership
    ownership_manager.establish_ownership(
        resource_id,
        new_owner,
        OwnershipType::Exclusive,
        None, // No expiration
        system.current_time(),
    )?;
    
    // Log ownership transfer event
    event_log.record(OwnershipEvent::Transferred {
        resource_id,
        from: current_ownership.owner_id,
        to: new_owner,
        timestamp: system.current_time(),
    });
    
    Ok(())
}
```

### Converting Ownership Type

The ownership model can be changed:

```rust
/// Convert exclusive ownership to shared ownership
pub fn convert_to_shared_ownership(
    resource_id: ResourceId,
    co_owners: Vec<OwnerId>,
    signature_threshold: u32,
    auth_context: AuthContext,
) -> Result<(), OwnershipError> {
    // Verify the resource exists
    let resource = registry.get_resource(resource_id)?;
    
    // Get current ownership information
    let current_ownership = ownership_manager.get_ownership(resource_id)?;
    
    // Verify this is exclusive ownership
    if !matches!(current_ownership.ownership_type, OwnershipType::Exclusive) {
        return Err(OwnershipError::NotExclusiveOwnership);
    }
    
    // Verify the requester is the current owner
    if !ownership_manager.verify_ownership(
        resource_id, 
        auth_context.origin(), 
        &current_ownership.ownership_type
    ) {
        return Err(OwnershipError::NotOwner);
    }
    
    // Verify valid co-owners list and threshold
    if co_owners.is_empty() || signature_threshold == 0 || signature_threshold > co_owners.len() as u32 {
        return Err(OwnershipError::InvalidOwnershipParameters);
    }
    
    // Verify authorization
    if !auth_system.authorize_ownership_conversion(resource_id, &co_owners, &auth_context) {
        return Err(OwnershipError::Unauthorized);
    }
    
    // Create the new ownership type
    let new_ownership_type = OwnershipType::Shared {
        owners: co_owners,
        signature_threshold,
    };
    
    // Revoke current ownership
    ownership_manager.revoke_ownership(resource_id, current_ownership.owner_id)?;
    
    // Establish new ownership
    ownership_manager.establish_ownership(
        resource_id,
        OwnerId::Custom { // Special case for shared ownership
            owner_type: "SharedOwnership".to_string(),
            id: resource_id.to_bytes(),
        },
        new_ownership_type,
        None, // No expiration
        system.current_time(),
    )?;
    
    // Log ownership conversion event
    event_log.record(OwnershipEvent::TypeChanged {
        resource_id,
        from_type: "Exclusive".to_string(),
        to_type: "Shared".to_string(),
        timestamp: system.current_time(),
    });
    
    Ok(())
}
```

## Programmatic Ownership

Programmatic ownership allows smart contracts to control resources:

```rust
/// Set a resource to be owned by a program
pub fn set_programmatic_ownership(
    resource_id: ResourceId,
    program_id: ProgramId,
    parameters: Vec<u8>,
    auth_context: AuthContext,
) -> Result<(), OwnershipError> {
    // Verify the resource exists
    let resource = registry.get_resource(resource_id)?;
    
    // Get current ownership information
    let current_ownership = ownership_manager.get_ownership(resource_id)?;
    
    // Verify the requester is the current owner
    if !ownership_manager.verify_ownership(
        resource_id, 
        auth_context.origin(), 
        &current_ownership.ownership_type
    ) {
        return Err(OwnershipError::NotOwner);
    }
    
    // Verify the program exists and can own resources
    if !program_registry.can_own_resources(program_id) {
        return Err(OwnershipError::ProgramCannotOwnResources);
    }
    
    // Verify authorization
    if !auth_system.authorize_ownership_transfer(
        resource_id, 
        OwnerId::Program(program_id), 
        &auth_context
    ) {
        return Err(OwnershipError::Unauthorized);
    }
    
    // Create the new ownership type
    let new_ownership_type = OwnershipType::Programmatic {
        program_id,
        parameters,
    };
    
    // Revoke current ownership
    ownership_manager.revoke_ownership(resource_id, current_ownership.owner_id)?;
    
    // Establish new ownership
    ownership_manager.establish_ownership(
        resource_id,
        OwnerId::Program(program_id),
        new_ownership_type,
        None, // No expiration
        system.current_time(),
    )?;
    
    // Log ownership change event
    event_log.record(OwnershipEvent::ProgrammaticOwnershipSet {
        resource_id,
        program_id,
        timestamp: system.current_time(),
    });
    
    Ok(())
}
```

## Delegated Ownership

Ownership can be delegated to other entities temporarily:

```rust
/// Delegate limited ownership rights to another entity
pub fn delegate_ownership(
    resource_id: ResourceId,
    delegate: OwnerId,
    capabilities: Vec<CapabilityId>,
    expiration: Timestamp,
    auth_context: AuthContext,
) -> Result<DelegationId, OwnershipError> {
    // Verify the resource exists
    let resource = registry.get_resource(resource_id)?;
    
    // Get current ownership information
    let current_ownership = ownership_manager.get_ownership(resource_id)?;
    
    // Verify the requester is the current owner
    if !ownership_manager.verify_ownership(
        resource_id, 
        auth_context.origin(), 
        &current_ownership.ownership_type
    ) {
        return Err(OwnershipError::NotOwner);
    }
    
    // Verify authorization for delegation
    if !auth_system.authorize_ownership_delegation(
        resource_id, 
        delegate, 
        &capabilities, 
        &auth_context
    ) {
        return Err(OwnershipError::Unauthorized);
    }
    
    // Verify expiration is in the future
    if expiration <= system.current_time() {
        return Err(OwnershipError::InvalidExpirationTime);
    }
    
    // Generate a unique delegation ID
    let delegation_id = DelegationId::generate();
    
    // Create the delegation
    let delegation = OwnershipDelegation {
        id: delegation_id,
        resource_id,
        owner_id: current_ownership.owner_id,
        delegate,
        capabilities,
        created_at: system.current_time(),
        expires_at: expiration,
        revoked: false,
    };
    
    // Register the delegation
    delegation_registry.register_delegation(delegation)?;
    
    // Log delegation event
    event_log.record(OwnershipEvent::Delegated {
        resource_id,
        owner: current_ownership.owner_id,
        delegate,
        delegation_id,
        expiration,
        timestamp: system.current_time(),
    });
    
    Ok(delegation_id)
}
```

## Collective Ownership

Shared ownership models allow multiple entities to control a resource:

```rust
/// Perform an operation on a shared-ownership resource
pub fn operate_shared_resource(
    resource_id: ResourceId,
    operation: ResourceOperation,
    signatures: Vec<(OwnerId, Vec<u8>)>,
) -> Result<OperationResult, OwnershipError> {
    // Verify the resource exists
    let resource = registry.get_resource(resource_id)?;
    
    // Get current ownership information
    let current_ownership = ownership_manager.get_ownership(resource_id)?;
    
    // Extract shared ownership details
    let (owners, threshold) = match &current_ownership.ownership_type {
        OwnershipType::Shared { owners, signature_threshold } => (owners, signature_threshold),
        _ => return Err(OwnershipError::NotSharedOwnership),
    };
    
    // Verify sufficient valid signatures
    let mut valid_signature_count = 0;
    
    for (signer, signature) in signatures {
        // Verify signer is a co-owner
        if !owners.contains(&signer) {
            continue;
        }
        
        // Verify signature for this operation
        if auth_system.verify_signature(
            resource_id,
            &operation,
            &signer,
            &signature,
        ) {
            valid_signature_count += 1;
        }
    }
    
    // Check if threshold is met
    if valid_signature_count < *threshold {
        return Err(OwnershipError::InsufficientSignatures {
            required: *threshold,
            provided: valid_signature_count,
        });
    }
    
    // Process the operation
    resource_manager.execute_operation(resource_id, operation)
}
```

## Cross-Domain Ownership

Resources can be owned across different domains:

```rust
/// Register a resource in a remote domain with the same ownership
pub fn register_cross_domain_ownership(
    resource_id: ResourceId,
    target_domain: DomainId,
    auth_context: AuthContext,
) -> Result<ResourceId, OwnershipError> {
    // Verify the resource exists in the local domain
    let resource = registry.get_resource(resource_id)?;
    
    // Get current ownership information
    let current_ownership = ownership_manager.get_ownership(resource_id)?;
    
    // Verify the requester is the current owner
    if !ownership_manager.verify_ownership(
        resource_id, 
        auth_context.origin(), 
        &current_ownership.ownership_type
    ) {
        return Err(OwnershipError::NotOwner);
    }
    
    // Verify cross-domain capabilities
    if !cross_domain_registry.can_register_resources(target_domain) {
        return Err(OwnershipError::DomainCannotRegisterResources);
    }
    
    // Create cross-domain registration message
    let registration = CrossDomainMessage::ResourceRegistration {
        origin_domain: system.domain_id(),
        origin_resource_id: resource_id,
        resource_type: resource.resource_type(),
        owner_id: current_ownership.owner_id,
        ownership_type: current_ownership.ownership_type.clone(),
        attributes: resource.attributes().clone(),
        timestamp: system.current_time(),
    };
    
    // Send registration message to target domain
    let remote_resource_id = cross_domain_messenger.send_registration(
        target_domain, 
        registration
    )?;
    
    // Create local-to-remote resource mapping
    cross_domain_registry.map_resources(
        resource_id,
        remote_resource_id,
        target_domain,
    )?;
    
    // Log cross-domain registration event
    event_log.record(CrossDomainEvent::ResourceRegistered {
        local_resource_id: resource_id,
        remote_resource_id,
        target_domain,
        timestamp: system.current_time(),
    });
    
    Ok(remote_resource_id)
}
```

## Ownership Management

The Ownership Manager provides a central service for managing resource ownership:

```rust
pub struct OwnershipManager {
    registry: ResourceRegistry,
    auth_system: AuthorizationSystem,
    delegation_registry: DelegationRegistry,
    event_log: EventLog,
}

impl OwnershipManager {
    pub fn new(
        registry: ResourceRegistry,
        auth_system: AuthorizationSystem,
        delegation_registry: DelegationRegistry,
        event_log: EventLog,
    ) -> Self {
        Self {
            registry,
            auth_system,
            delegation_registry,
            event_log,
        }
    }
    
    /// Establish ownership of a resource
    pub fn establish_ownership(
        &self,
        resource_id: ResourceId,
        owner_id: OwnerId,
        ownership_type: OwnershipType,
        expires_at: Option<Timestamp>,
        established_at: Timestamp,
    ) -> Result<(), OwnershipError> {
        // Implementation details...
    }
    
    /// Revoke current ownership
    pub fn revoke_ownership(
        &self,
        resource_id: ResourceId,
        owner_id: OwnerId,
    ) -> Result<(), OwnershipError> {
        // Implementation details...
    }
    
    /// Get current ownership information
    pub fn get_ownership(
        &self,
        resource_id: ResourceId,
    ) -> Result<OwnershipClaim, OwnershipError> {
        // Implementation details...
    }
    
    /// Verify if an entity owns a resource
    pub fn verify_ownership(
        &self,
        resource_id: ResourceId,
        owner_id: OwnerId,
        ownership_type: &OwnershipType,
    ) -> bool {
        // Implementation details...
    }
    
    /// Check if an entity can operate on a resource through ownership or delegation
    pub fn can_operate(
        &self,
        resource_id: ResourceId,
        operator: OwnerId,
        operation: &ResourceOperation,
    ) -> bool {
        // Implementation details...
    }
}
```

## Ownership Verification

Ownership is verified during operations:

```rust
/// Verify ownership for a resource operation
pub fn verify_resource_operation_authorization(
    resource_id: ResourceId,
    operation: &ResourceOperation,
    auth_context: &AuthContext,
) -> Result<bool, OwnershipError> {
    // Get current ownership information
    let ownership = ownership_manager.get_ownership(resource_id)?;
    
    // Check direct ownership
    let direct_ownership = ownership_manager.verify_ownership(
        resource_id,
        auth_context.origin(),
        &ownership.ownership_type,
    );
    
    if direct_ownership {
        return Ok(true);
    }
    
    // Check delegated ownership
    let has_delegation = delegation_registry.has_valid_delegation(
        resource_id,
        auth_context.origin(),
        operation,
    );
    
    if has_delegation {
        return Ok(true);
    }
    
    // Check capability-based authorization
    let has_capability = auth_system.has_capability(
        auth_context.origin(),
        resource_id,
        operation,
    );
    
    if has_capability {
        return Ok(true);
    }
    
    // No valid authorization found
    Ok(false)
}
```

## Usage Examples

### Creating a Resource with Exclusive Ownership

```rust
// Register a new resource with exclusive ownership
let token_attributes = HashMap::from([
    ("name".to_string(), Value::String("GoldToken".to_string())),
    ("symbol".to_string(), Value::String("GLD".to_string())),
    ("decimals".to_string(), Value::Integer(18)),
    ("total_supply".to_string(), Value::Integer(1000000)),
]);

let token_id = resource_lifecycle_manager.register_resource_with_ownership(
    ResourceType::Token,
    OwnerId::Account(my_account_id),
    OwnershipType::Exclusive,
    token_attributes,
)?;

println!("Token resource created with exclusive ownership: {}", token_id);
```

### Transferring Resource Ownership

```rust
// Transfer ownership to another account
let recipient = OwnerId::Account(recipient_account_id);

let auth_context = AuthContext::with_signature(
    my_account_id,
    my_signature,
    system.current_time(),
);

ownership_manager.transfer_exclusive_ownership(
    resource_id,
    recipient,
    auth_context,
)?;

println!("Resource {} transferred to account {}", resource_id, recipient_account_id);
```

### Setting Up Shared Ownership

```rust
// Convert exclusive ownership to shared ownership
let co_owners = vec![
    OwnerId::Account(my_account_id),
    OwnerId::Account(partner_account_id),
    OwnerId::Account(third_party_id),
];

let auth_context = AuthContext::with_signature(
    my_account_id,
    my_signature,
    system.current_time(),
);

ownership_manager.convert_to_shared_ownership(
    resource_id,
    co_owners,
    2, // 2-of-3 signature threshold
    auth_context,
)?;

println!("Resource {} now has shared ownership (2-of-3)", resource_id);
```

### Delegating Ownership Rights

```rust
// Delegate limited ownership capabilities to another account
let capabilities = vec![
    CapabilityId::from_str("resource.use"),
    CapabilityId::from_str("resource.view"),
];

let one_week_from_now = system.current_time() + Duration::days(7);

let auth_context = AuthContext::with_signature(
    my_account_id,
    my_signature,
    system.current_time(),
);

let delegation_id = ownership_manager.delegate_ownership(
    resource_id,
    OwnerId::Account(delegate_account_id),
    capabilities,
    one_week_from_now,
    auth_context,
)?;

println!("Ownership delegated with ID: {} (expires in 7 days)", delegation_id);
```

## Implementation Status

The following components of the ownership model have been implemented:

- ✅ Basic exclusive ownership (registration, transfer, verification)
- ✅ Ownership claims and proofs
- ✅ Integration with the authorization system
- ⚠️ Shared ownership (partially implemented)
- ⚠️ Delegation system (partially implemented)
- ⚠️ Cross-domain ownership (partially implemented)
- ❌ Programmatic ownership (not yet implemented)
- ❌ Custom ownership models (not yet implemented)

## Future Enhancements

Future enhancements to the ownership model include:

1. **Hierarchical Ownership**: Support for ownership hierarchies where parent-child relationships between resources define ownership inheritance
2. **Time-bound Ownership**: Allow ownership that is valid only during specific time windows
3. **Conditional Ownership**: Ownership that depends on specific conditions being met
4. **Gradual Ownership Transfer**: Enable staged transfers of ownership over time
5. **Ownership Recovery Mechanisms**: Secure methods for recovering ownership of lost resources
6. **Hybrid Ownership Models**: Combinations of different ownership types
7. **On-chain Governance for Ownership**: Allow community-based decisions for certain ownership operations 