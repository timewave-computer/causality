# Capability System

*This document is derived from [ADR-003](../../../spec/adr_003_resource.md), [ADR-022](../../../spec/adr_022_permissioning_unification.md), [ADR-032](../../../spec/adr_032_capability_refinement.md), and the [System Specification](../../../spec/spec.md).*

*Last updated: 2023-03-26*

## Overview

The Capability System provides a secure authorization model for Causality that governs access to resources through unforgeable capability tokens with explicit delegation paths. It ensures that operations on resources are properly authorized while enabling fine-grained access control and verifiable delegation chains.

## Core Concepts

### Capability Model

A capability is an unforgeable token that grants specific rights to perform operations on resources. Capabilities in Causality follow these core principles:

1. **Unforgeable**: Each capability has a cryptographically secure content hash
2. **Delegatable**: Capabilities can be delegated to create capability chains
3. **Revocable**: Capabilities can be revoked without affecting other capability chains
4. **Attenuable**: Capabilities can be restricted when delegated

```
┌─────────────────────────────────────────────────────────┐
│                 Capability System Model                 │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────────┐                                    │
│  │   Capability    │                                    │
│  │                 │                                    │
│  │ • Target        │                                    │
│  │ • Type          │◄───────┐                           │
│  │ • Constraints   │        │                           │
│  │ • Expiration    │        │                           │
│  │ • Content Hash  │        │                           │
│  └─────────────────┘        │                           │
│                             │                           │
│  ┌─────────────────┐        │    ┌─────────────────┐    │
│  │  Authorization  │        │    │   Delegation    │    │
│  │                 │        └────┤                 │    │
│  │ • Effects       │             │ • Delegator     │    │
│  │ • Resources     │◄────────────┤ • Delegatee     │    │
│  │ • Operations    │             │ • Source        │    │
│  │ • Contexts      │             │ • Derived       │    │
│  └─────────────────┘             │ • Signature     │    │
│                                  └─────────────────┘    │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Capability Types

Capabilities are categorized based on the type of access they grant:

- **Read**: Allows reading resource data
- **Write**: Allows modifying resource data
- **Execute**: Allows executing operations on resources
- **Admin**: Allows administrative actions, including delegation
- **Owner**: Grants full ownership rights, including transfer

### Capability Constraints

Capabilities can be constrained in various ways:

- **Time Constraints**: Limit when a capability can be used
- **Field Constraints**: Restrict access to specific fields of a resource
- **Operation Constraints**: Limit the operations that can be performed
- **Quantity Constraints**: Restrict the amount of a resource that can be affected
- **Custom Constraints**: Allow for domain-specific restrictions

## Components

### Capability Structure

The core structure of a capability contains:

```rust
/// Capability token that grants access to a resource
pub struct Capability {
    /// Target resource
    target: ResourceId,
    /// Capability type (read, write, etc.)
    capability_type: CapabilityType,
    /// Constraints on this capability
    constraints: Vec<CapabilityConstraint>,
    /// Expiration time (if any)
    expires_at: Option<DateTime<Utc>>,
    /// Content hash of this capability
    content_hash: ContentHash,
}
```

### Capability Delegation

Capabilities can be delegated to create verifiable authorization chains:

```rust
/// Capability delegation
pub struct CapabilityDelegation {
    /// Delegator identity
    delegator: Identity,
    /// Delegatee identity
    delegatee: Identity,
    /// Source capability
    source: ContentHash,
    /// Derived capability
    derived: Capability,
    /// Signature from the delegator
    signature: Signature,
    /// Content hash of this delegation
    content_hash: ContentHash,
}
```

### Capability Store

Capabilities and their delegations are persisted in a capability store:

```rust
/// Storage for capabilities and delegations
pub trait CapabilityStore: Send + Sync + 'static {
    /// Store a capability delegation
    async fn store_delegation(&self, delegation: &CapabilityDelegation) 
        -> Result<(), CapabilityError>;
    
    /// Find delegations for an identity
    async fn find_delegations_for_identity(&self, identity: &Identity) 
        -> Result<Vec<CapabilityDelegation>, CapabilityError>;
    
    /// Find a delegation chain
    async fn find_delegation_chain(&self, delegation_hash: &ContentHash) 
        -> Result<Vec<CapabilityDelegation>, CapabilityError>;
    
    /// Revoke a capability
    async fn revoke_capability(&self, capability_hash: &ContentHash) 
        -> Result<(), CapabilityError>;
    
    /// Check if a capability is revoked
    async fn is_revoked(&self, capability_hash: &ContentHash) 
        -> Result<bool, CapabilityError>;
}
```

### Capability Manager

The Capability Manager provides high-level functions for working with capabilities:

```rust
/// Manages capabilities in the system
pub struct CapabilityManager {
    /// Store for capabilities and delegations
    store: Arc<dyn CapabilityStore>,
    /// Crypto service for signing and verification
    crypto: Arc<dyn CryptoService>,
    /// System key pair for root capabilities
    system_key: Arc<KeyPair>,
    /// System identity
    system_identity: Identity,
}
```

## Capability Lifecycle

### Creation

Capabilities are created by authorized entities, typically starting with system root capabilities:

1. The system creates root capabilities for resources
2. Resource owners receive owner capabilities for their resources
3. Owners can delegate more restricted capabilities to others

### Delegation

Capabilities can be delegated to create authorization chains:

1. A delegator creates a derived capability with potentially more constraints
2. The delegator signs the delegation with their private key
3. The delegation is stored in the capability store
4. The delegatee can use the derived capability to access resources

### Verification

When a capability is used, it is verified through several steps:

1. Verify the content hash of the capability
2. Check if the capability has been revoked
3. Verify that the capability has not expired
4. Validate that all constraints are satisfied
5. Verify the delegation chain back to a root capability
6. Check signatures in the delegation chain

### Revocation

Capabilities can be revoked by authorized entities:

1. The capability hash is added to the revoked set
2. All derived capabilities are implicitly revoked
3. Revocation does not affect parallel capability chains

## Integration with Other Systems

### Resource System Integration

The Capability System integrates closely with the Resource System:

```
┌─────────────────────────────────────────┐
│           Resource Operation            │
│                                         │
│  ┌───────────────┐    ┌───────────────┐ │
│  │   Resource    │    │   Operation   │ │
│  │   System      │───▶│   Execution   │ │
│  └───────┬───────┘    └───────────────┘ │
│          │                    ▲         │
│          │                    │         │
│          ▼                    │         │
│  ┌───────────────┐    ┌───────────────┐ │
│  │  Capability   │    │   Resource    │ │
│  │ Verification  │───▶│  Authorization│ │
│  └───────────────┘    └───────────────┘ │
│                                         │
└─────────────────────────────────────────┘
```

1. Resources are associated with capabilities
2. Operations on resources require the appropriate capabilities
3. Resource transfers involve capability delegation
4. Resource access is controlled through capability verification

### Effect System Integration

The Capability System authorizes effect execution:

```rust
/// Execute an effect with capability verification
pub async fn execute_effect_with_capabilities(
    effect: &dyn Effect,
    context: &EffectContext,
    executor: &EffectEngine,
    capability_manager: &CapabilityManager,
) -> Result<EffectOutcome, EffectError> {
    // Get the identity
    let identity = context.identity();
    
    // Verify capabilities for each resource
    for resource in effect.resources() {
        // Determine the required capability type based on the effect
        let capability_type = determine_capability_type(effect);
        
        // Verify the capability
        let has_capability = capability_manager.verify_capability(
            &identity,
            &resource,
            capability_type,
        ).await?;
        
        if !has_capability {
            return Err(EffectError::InsufficientCapabilities(
                format!("Missing {:?} capability for resource {}", capability_type, resource)
            ));
        }
    }
    
    // Execute the effect
    executor.execute_effect(effect, context).await
}
```

### Time System Integration

The Capability System works with the Time System to enforce temporal constraints:

1. Capabilities can have time-based constraints
2. Capability expiration is verified using the Time System
3. Temporal constraints are evaluated against the current time
4. Time-based delegation can be implemented using temporal constraints

## Examples

### Creating a Capability

```rust
/// Create a capability
let capability = Capability::new(
    ResourceId::from_parts("account", "user1"),
    CapabilityType::Write,
)?
.with_constraint(
    CapabilityConstraint::Time(TimeConstraint {
        start: Some(Utc::now()),
        end: Some(Utc::now() + chrono::Duration::days(30)),
    }),
)?
.with_constraint(
    CapabilityConstraint::Operation(OperationConstraint {
        allowed_operations: vec!["update_balance".to_string()],
    }),
)?;
```

### Delegating a Capability

```rust
/// Create a derived capability with constraints
let derived = Capability::new(
    resource_id.clone(),
    CapabilityType::Write,
)?.with_constraint(
    CapabilityConstraint::Time(TimeConstraint {
        start: Some(Utc::now()),
        end: Some(Utc::now() + chrono::Duration::days(30)),
    }),
)?.with_constraint(
    CapabilityConstraint::Operation(OperationConstraint {
        allowed_operations: vec!["update_balance".to_string()],
    }),
)?;

/// Delegate the capability
let delegation = capability_manager.delegate_capability(
    &capability,
    derived,
    delegator_identity,
    delegatee_identity,
    &delegator_key,
).await?;
```

### Verifying a Capability

```rust
/// Verify a capability
let is_authorized = capability_manager.verify_capability(
    &identity,
    &resource_id,
    CapabilityType::Write,
).await?;

if is_authorized {
    // Perform the operation
} else {
    // Return an error
}
```

## Where Implemented

The Capability System is implemented in the following crates and modules:

| Component | Crate | Module |
|-----------|-------|--------|
| Capability Types | `causality-types` | `causality_types::capability::types` |
| Capability Store | `causality-core` | `causality_core::capability::store` |
| Capability Manager | `causality-core` | `causality_core::capability::manager` |
| Capability Verification | `causality-core` | `causality_core::capability::verification` |
| Capability Delegation | `causality-core` | `causality_core::capability::delegation` |
| Resource Authorization | `causality-core` | `causality_core::resource::authorization` |
| Effect Authorization | `causality-core` | `causality_core::effects::authorization` |

## Security Considerations

The Capability System is designed with security as a primary concern:

1. **Principle of Least Privilege**: Capabilities should grant the minimum privileges needed
2. **Unforgeable References**: Capabilities cannot be forged due to cryptographic verification
3. **Delegation Control**: Delegation chains are explicitly verified
4. **Confinement**: Capabilities can only be used for their intended purpose
5. **Revocation**: Capabilities can be revoked to remove access
6. **Attenuation**: Derived capabilities can only restrict, never expand privileges
7. **Content Addressing**: All capabilities are content-addressed for integrity

## References

- [ADR-003: Resource System](../../../spec/adr_003_resource.md)
- [ADR-022: Rigorous Resource and Capability Model](../../../spec/adr_022_permissioning_unification.md)
- [ADR-032: Capability Refinement](../../../spec/adr_032_capability_refinement.md)
- [Implementing the Capability System](../../guides/implementation/capability-system.md)
