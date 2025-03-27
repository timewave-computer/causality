# Implementing the Capability System

*This guide provides practical implementations for working with the [Capability System](../../architecture/core/capability-system.md).*

*Last updated: 2023-03-26*

## Overview

This guide covers the practical aspects of implementing and working with the Capability System in Causality. It provides code examples, best practices, and implementation patterns for creating, delegating, and verifying capabilities.

## Prerequisites

Before implementing capabilities in your code, make sure you're familiar with:

- The [Capability System Architecture](../../architecture/core/capability-system.md)
- The [Resource System](../../architecture/core/resource-system.md)
- The [Effect System](../../architecture/core/effect-system.md)

## Implementation Guide

### Required Crates and Imports

```rust
// Core capability types
use causality_types::{
    capability::{
        Capability, CapabilityType, CapabilityConstraint, 
        TimeConstraint, OperationConstraint, FieldConstraint
    },
    identity::Identity,
    resource::ResourceId,
    common::ContentHash,
};

// Capability services and managers
use causality_core::{
    capability::{
        CapabilityManager, CapabilityStore, MemoryCapabilityStore,
        CapabilityDelegation, CapabilityError
    },
    crypto::{CryptoService, KeyPair, Signature},
};
```

### Setting Up the Capability System

```rust
/// Set up the capability system
async fn setup_capability_system(
    crypto: Arc<dyn CryptoService>,
    system_keypair: Arc<KeyPair>,
) -> Result<Arc<CapabilityManager>, CapabilityError> {
    // Create a capability store
    let store = Arc::new(MemoryCapabilityStore::new());
    
    // Create the capability manager
    let system_identity = crypto.derive_identity(&system_keypair.public_key)?;
    let manager = Arc::new(CapabilityManager::new(
        store,
        crypto,
        system_keypair,
        system_identity,
    ));
    
    Ok(manager)
}
```

### Creating Root Capabilities

Root capabilities are the starting point of all capability chains. They are typically created by the system for resource owners:

```rust
/// Create a root capability for a resource owner
async fn create_root_capability(
    manager: &CapabilityManager,
    resource_id: ResourceId,
    owner_identity: Identity,
) -> Result<ContentHash, CapabilityError> {
    // Create the capability with owner rights
    let capability = Capability::new(
        resource_id,
        CapabilityType::Owner,
    )?;
    
    // Create and sign the root delegation
    let delegation = manager.create_root_delegation(
        capability,
        owner_identity,
    ).await?;
    
    // Return the capability's content hash
    Ok(delegation.derived.content_hash)
}
```

### Delegating Capabilities

Owners can delegate more restricted capabilities to other entities:

```rust
/// Delegate a capability to another identity
async fn delegate_capability(
    manager: &CapabilityManager,
    delegator_identity: &Identity,
    delegator_key: &KeyPair,
    delegatee_identity: &Identity,
    source_capability_hash: &ContentHash,
    resource_id: &ResourceId,
) -> Result<ContentHash, CapabilityError> {
    // Retrieve the source capability
    let source = manager.get_capability(source_capability_hash).await?;
    
    // Create a derived capability with more constraints
    let derived = Capability::new(
        resource_id.clone(),
        CapabilityType::Write,  // More restricted than Owner
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
    
    // Create and sign the delegation
    let delegation = manager.delegate_capability(
        &source,
        derived,
        *delegator_identity,
        *delegatee_identity,
        delegator_key,
    ).await?;
    
    // Return the new capability's content hash
    Ok(delegation.derived.content_hash)
}
```

### Adding Constraints to Capabilities

Constraints restrict what can be done with a capability:

```rust
/// Add different types of constraints to a capability
fn add_constraints_to_capability(
    resource_id: &ResourceId,
) -> Result<Capability, CapabilityError> {
    let capability = Capability::new(
        resource_id.clone(),
        CapabilityType::Write,
    )?
    
    // Time constraint
    .with_constraint(
        CapabilityConstraint::Time(TimeConstraint {
            start: Some(Utc::now()),
            end: Some(Utc::now() + chrono::Duration::days(30)),
        }),
    )?
    
    // Operation constraint
    .with_constraint(
        CapabilityConstraint::Operation(OperationConstraint {
            allowed_operations: vec!["update_balance".to_string(), "read_balance".to_string()],
        }),
    )?
    
    // Field constraint
    .with_constraint(
        CapabilityConstraint::Field(FieldConstraint {
            allowed_fields: vec!["balance".to_string()],
            denied_fields: vec!["personal_info".to_string()],
        }),
    )?;
    
    Ok(capability)
}
```

### Verifying Capabilities

When executing operations on resources, capabilities must be verified:

```rust
/// Check if an identity has a capability for a resource
async fn check_capability(
    manager: &CapabilityManager,
    identity: &Identity,
    resource_id: &ResourceId,
    capability_type: CapabilityType,
) -> Result<bool, CapabilityError> {
    // This checks the full delegation chain from the identity to the root
    let has_capability = manager.verify_capability(
        identity,
        resource_id,
        capability_type,
    ).await?;
    
    Ok(has_capability)
}
```

### Revoking Capabilities

Capabilities can be revoked to remove access:

```rust
/// Revoke a capability
async fn revoke_capability(
    manager: &CapabilityManager,
    revoker_identity: &Identity,
    revoker_key: &KeyPair,
    capability_hash: &ContentHash,
) -> Result<(), CapabilityError> {
    // This checks if the revoker has the right to revoke
    // and adds the capability to the revoked set
    manager.revoke_capability(
        capability_hash,
        revoker_identity,
        revoker_key,
    ).await
}
```

## Integrating with Resource Operations

When performing operations on resources, capabilities should be verified:

```rust
/// Example resource operation with capability check
async fn update_resource(
    manager: &CapabilityManager,
    identity: &Identity,
    resource_id: &ResourceId,
    field: &str,
    value: &str,
) -> Result<(), ResourceError> {
    // First, verify that the identity has the capability
    let has_capability = manager.verify_capability(
        identity,
        resource_id,
        CapabilityType::Write,
    ).await
    .map_err(|e| ResourceError::CapabilityError(e.to_string()))?;
    
    if !has_capability {
        return Err(ResourceError::InsufficientCapabilities(
            format!("Missing write capability for resource {}", resource_id)
        ));
    }
    
    // If capability check passes, perform the operation
    // ...resource update logic here...
    
    Ok(())
}
```

## Integrating with Effect Execution

Effects can be authorized using capabilities:

```rust
/// Execute an effect with capability verification
async fn execute_effect_with_capabilities<E: Effect>(
    effect: &E,
    context: &EffectContext,
    executor: &EffectExecutor,
    capability_manager: &CapabilityManager,
) -> Result<EffectOutcome, EffectError> {
    // Get the identity from the context
    let identity = context.identity();
    
    // Verify capabilities for each resource used by the effect
    for resource in effect.resources() {
        // Determine the required capability type based on the effect
        let capability_type = match effect.operation_type() {
            OperationType::Read => CapabilityType::Read,
            OperationType::Write => CapabilityType::Write,
            OperationType::Execute => CapabilityType::Execute,
            OperationType::Admin => CapabilityType::Admin,
        };
        
        // Verify the capability
        let has_capability = capability_manager.verify_capability(
            &identity,
            &resource,
            capability_type,
        ).await
        .map_err(|e| EffectError::CapabilityError(e.to_string()))?;
        
        if !has_capability {
            return Err(EffectError::InsufficientCapabilities(
                format!("Missing {:?} capability for resource {}", capability_type, resource)
            ));
        }
    }
    
    // Execute the effect
    executor.execute(effect, context).await
}
```

## Advanced Patterns

### Capability-Based Security for APIs

When exposing APIs, capability verification should be part of the request handling:

```rust
/// API endpoint with capability verification
async fn api_update_resource(
    req: HttpRequest,
    capability_manager: &CapabilityManager,
) -> HttpResponse {
    // Extract JWT token from request
    let token = match extract_token(&req) {
        Some(t) => t,
        None => return HttpResponse::Unauthorized().finish(),
    };
    
    // Verify token and extract identity
    let identity = match verify_token(token) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };
    
    // Extract resource ID from request
    let resource_id = match extract_resource_id(&req) {
        Some(id) => id,
        None => return HttpResponse::BadRequest().finish(),
    };
    
    // Verify capability
    let has_capability = match capability_manager.verify_capability(
        &identity,
        &resource_id,
        CapabilityType::Write,
    ).await {
        Ok(result) => result,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    
    if !has_capability {
        return HttpResponse::Forbidden().finish();
    }
    
    // Process the request
    // ...
    
    HttpResponse::Ok().finish()
}
```

### Capability Composition

Complex workflows can combine multiple capabilities:

```rust
/// Transfer resources between domains with capability verification
async fn transfer_resources(
    manager: &CapabilityManager,
    identity: &Identity,
    source_resource: &ResourceId,
    destination_resource: &ResourceId,
    amount: u64,
) -> Result<(), TransferError> {
    // Verify source capability (withdrawal)
    let has_source_capability = manager.verify_capability(
        identity,
        source_resource,
        CapabilityType::Write,
    ).await
    .map_err(|e| TransferError::CapabilityError(e.to_string()))?;
    
    if !has_source_capability {
        return Err(TransferError::InsufficientCapabilities(
            format!("Missing write capability for source resource {}", source_resource)
        ));
    }
    
    // Verify destination capability (deposit)
    let has_destination_capability = manager.verify_capability(
        identity,
        destination_resource,
        CapabilityType::Write,
    ).await
    .map_err(|e| TransferError::CapabilityError(e.to_string()))?;
    
    if !has_destination_capability {
        return Err(TransferError::InsufficientCapabilities(
            format!("Missing write capability for destination resource {}", destination_resource)
        ));
    }
    
    // If both capability checks pass, perform the transfer
    // ...transfer logic here...
    
    Ok(())
}
```

### Secure Capability Serialization

When capabilities need to be serialized for storage or transfer:

```rust
/// Serialize and deserialize capabilities securely
fn serialize_capability(capability: &Capability) -> Result<Vec<u8>, SerializationError> {
    // Serialize to bytes
    let bytes = bincode::serialize(capability)?;
    
    // Return the bytes
    Ok(bytes)
}

fn deserialize_capability(bytes: &[u8]) -> Result<Capability, SerializationError> {
    // Deserialize from bytes
    let capability: Capability = bincode::deserialize(bytes)?;
    
    // Verify the content hash
    let computed_hash = ContentHash::compute_for_bytes(bytes)?;
    if computed_hash != capability.content_hash {
        return Err(SerializationError::HashMismatch);
    }
    
    // Return the capability
    Ok(capability)
}
```

## Best Practices

### Security Considerations

1. **Principle of Least Privilege**: Always delegate the minimum capabilities needed.
   ```rust
   // GOOD: Specific constraint limiting operations
   let capability = Capability::new(resource_id, CapabilityType::Write)?
       .with_constraint(CapabilityConstraint::Operation(
           OperationConstraint {
               allowed_operations: vec!["update_balance".to_string()],
           }
       ))?;
   
   // BAD: No constraints, too permissive
   let capability = Capability::new(resource_id, CapabilityType::Write)?;
   ```

2. **Always Verify Delegation Chains**: Don't trust capabilities without verifying the full chain.
   ```rust
   // GOOD: Use the manager to verify the full chain
   let is_authorized = capability_manager.verify_capability(
       &identity,
       &resource_id,
       CapabilityType::Write,
   ).await?;
   
   // BAD: Checking only the capability without verifying the chain
   let is_authorized = capability.target == resource_id && 
                       capability.capability_type == CapabilityType::Write;
   ```

3. **Add Temporal Constraints**: Add expiration times to delegated capabilities.
   ```rust
   // GOOD: Adding an expiration time
   let capability = Capability::new(resource_id, CapabilityType::Write)?
       .with_constraint(CapabilityConstraint::Time(
           TimeConstraint {
               start: Some(Utc::now()),
               end: Some(Utc::now() + chrono::Duration::days(30)),
           }
       ))?;
   ```

4. **Revoke Unused Capabilities**: Implement regular checks to revoke unused capabilities.
   ```rust
   // Example of a regular capability cleanup task
   async fn cleanup_expired_capabilities(
       manager: &CapabilityManager,
   ) -> Result<(), CapabilityError> {
       let expired = manager.find_expired_capabilities().await?;
       for cap_hash in expired {
           manager.revoke_capability(
               &cap_hash,
               &manager.system_identity(),
               &manager.system_key(),
           ).await?;
       }
       Ok(())
   }
   ```

### Performance Optimization

1. **Cache Verification Results**: Cache capability verification results for frequently accessed resources.
   ```rust
   struct CapabilityCache {
       cache: HashMap<(Identity, ResourceId, CapabilityType), (bool, Instant)>,
       expiry: Duration,
   }
   
   impl CapabilityCache {
       fn new(expiry: Duration) -> Self {
           Self {
               cache: HashMap::new(),
               expiry,
           }
       }
       
       fn get(&self, identity: &Identity, resource: &ResourceId, cap_type: CapabilityType) -> Option<bool> {
           if let Some((result, timestamp)) = self.cache.get(&(*identity, resource.clone(), cap_type)) {
               if timestamp.elapsed() < self.expiry {
                   return Some(*result);
               }
           }
           None
       }
       
       fn set(&mut self, identity: Identity, resource: ResourceId, cap_type: CapabilityType, result: bool) {
           self.cache.insert((identity, resource, cap_type), (result, Instant::now()));
       }
   }
   ```

2. **Batch Capability Verifications**: When multiple capabilities need to be checked, batch them together.
   ```rust
   /// Verify multiple capabilities in a single pass
   async fn verify_multiple_capabilities(
       manager: &CapabilityManager,
       identity: &Identity,
       capability_checks: Vec<(ResourceId, CapabilityType)>,
   ) -> Result<Vec<bool>, CapabilityError> {
       let mut results = Vec::with_capacity(capability_checks.len());
       
       // Find all delegations for this identity first (one query)
       let delegations = manager.find_delegations_for_identity(identity).await?;
       
       // Verify each capability against the already fetched delegations
       for (resource_id, capability_type) in capability_checks {
           let result = manager.verify_capability_with_delegations(
               &delegations,
               identity,
               &resource_id,
               capability_type,
           ).await?;
           
           results.push(result);
       }
       
       Ok(results)
   }
   ```

## Implementation Examples

### Creating a Root Resource with Owner Capability

```rust
/// Create a new resource with owner capability for the creator
async fn create_resource_with_owner_capability(
    resource_manager: &ResourceManager,
    capability_manager: &CapabilityManager,
    creator_identity: &Identity,
    creator_key: &KeyPair,
    resource_type: &str,
    resource_data: &[u8],
) -> Result<ResourceId, ResourceError> {
    // Create the resource
    let resource_id = resource_manager.create_resource(
        resource_type,
        resource_data,
        creator_identity,
        creator_key,
    ).await?;
    
    // Create root owner capability
    let capability = Capability::new(
        resource_id.clone(),
        CapabilityType::Owner,
    )?;
    
    // Create and sign the root delegation
    capability_manager.create_root_delegation(
        capability,
        *creator_identity,
    ).await
    .map_err(|e| ResourceError::CapabilityError(e.to_string()))?;
    
    Ok(resource_id)
}
```

### Authorization Check in an Effect Handler

```rust
/// Effect handler that includes capability verification
struct ResourceEffectHandler {
    resource_manager: Arc<ResourceManager>,
    capability_manager: Arc<CapabilityManager>,
}

impl EffectHandler<ResourceEffect> for ResourceEffectHandler {
    async fn handle(&self, effect: &ResourceEffect, context: &EffectContext) 
        -> Result<EffectOutcome, EffectError> {
        // Get the identity from the context
        let identity = context.identity();
        
        // Get the resource ID from the effect
        let resource_id = effect.resource_id();
        
        // Determine the required capability type based on the effect
        let capability_type = match effect {
            ResourceEffect::Read(_) => CapabilityType::Read,
            ResourceEffect::Write(_) => CapabilityType::Write,
            ResourceEffect::Transfer(_) => CapabilityType::Owner,
        };
        
        // Verify the capability
        let has_capability = self.capability_manager.verify_capability(
            &identity,
            &resource_id,
            capability_type,
        ).await
        .map_err(|e| EffectError::CapabilityError(e.to_string()))?;
        
        if !has_capability {
            return Err(EffectError::InsufficientCapabilities(
                format!("Missing {:?} capability for resource {}", capability_type, resource_id)
            ));
        }
        
        // Handle the effect based on its type
        match effect {
            ResourceEffect::Read(read_effect) => {
                // Handle read operation
                let data = self.resource_manager.read_resource(&resource_id).await?;
                Ok(EffectOutcome::Success(json!({ "data": data })))
            },
            ResourceEffect::Write(write_effect) => {
                // Handle write operation
                self.resource_manager.write_resource(
                    &resource_id, 
                    &write_effect.data,
                    &identity,
                ).await?;
                Ok(EffectOutcome::Success(json!({ "status": "updated" })))
            },
            ResourceEffect::Transfer(transfer_effect) => {
                // Handle transfer operation
                self.resource_manager.transfer_resource(
                    &resource_id,
                    &transfer_effect.recipient,
                    &identity,
                ).await?;
                Ok(EffectOutcome::Success(json!({ "status": "transferred" })))
            },
        }
    }
}
```

## Testing Capability Implementations

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_capability_delegation() {
        // Set up crypto service
        let crypto = Arc::new(MockCryptoService::new());
        
        // Create key pairs
        let system_key = crypto.generate_key_pair().unwrap();
        let owner_key = crypto.generate_key_pair().unwrap();
        let user_key = crypto.generate_key_pair().unwrap();
        
        // Derive identities
        let system_identity = crypto.derive_identity(&system_key.public_key).unwrap();
        let owner_identity = crypto.derive_identity(&owner_key.public_key).unwrap();
        let user_identity = crypto.derive_identity(&user_key.public_key).unwrap();
        
        // Set up capability system
        let manager = setup_capability_system(
            crypto.clone(),
            Arc::new(system_key),
        ).await.unwrap();
        
        // Create a resource
        let resource_id = ResourceId::from_parts("account", "test");
        
        // Create a root capability for the owner
        let owner_cap_hash = create_root_capability(
            &manager,
            resource_id.clone(),
            owner_identity,
        ).await.unwrap();
        
        // Delegate a more restricted capability to the user
        let user_cap_hash = delegate_capability(
            &manager,
            &owner_identity,
            &owner_key,
            &user_identity,
            &owner_cap_hash,
            &resource_id,
        ).await.unwrap();
        
        // Verify owner capability (should have full access)
        let owner_has_capability = manager.verify_capability(
            &owner_identity,
            &resource_id,
            CapabilityType::Owner,
        ).await.unwrap();
        assert!(owner_has_capability);
        
        // Verify user capability (should have write but not owner)
        let user_has_write = manager.verify_capability(
            &user_identity,
            &resource_id,
            CapabilityType::Write,
        ).await.unwrap();
        assert!(user_has_write);
        
        let user_has_owner = manager.verify_capability(
            &user_identity,
            &resource_id,
            CapabilityType::Owner,
        ).await.unwrap();
        assert!(!user_has_owner);
        
        // Revoke user capability
        manager.revoke_capability(
            &user_cap_hash,
            &owner_identity,
            &owner_key,
        ).await.unwrap();
        
        // Verify user capability again (should no longer have access)
        let user_has_write_after_revoke = manager.verify_capability(
            &user_identity,
            &resource_id,
            CapabilityType::Write,
        ).await.unwrap();
        assert!(!user_has_write_after_revoke);
    }
}
```

## Troubleshooting

| Problem | Possible Cause | Solution |
|---------|---------------|----------|
| Capability verification fails | Delegation chain is broken | Ensure all delegations are correctly stored and linked |
| | Capability has expired | Check the time constraints on the capability |
| | Resource ID doesn't match | Verify the resource ID in the capability matches the target |
| | Capability has been revoked | Check if the capability has been revoked in the store |
| Cannot delegate capability | Missing source capability | Ensure the delegator has the source capability |
| | Insufficient rights | Only owner or admin capabilities can be used for delegation |
| | Invalid signature | Ensure the delegator's key is correct |
| Cannot create root capability | System identity mismatch | Only the system identity can create root capabilities |
| | Missing system key | Ensure the system key is available |
| Performance issues | Too many verification calls | Implement caching for frequently verified capabilities |
| | Long delegation chains | Minimize the length of delegation chains |

## References

- [Capability System Architecture](../../architecture/core/capability-system.md)
- [Resource System](../../architecture/core/resource-system.md)
- [Effect System](../../architecture/core/effect-system.md)
- [ADR-003: Resource System](../../../spec/adr_003_resource.md)
- [ADR-022: Rigorous Resource and Capability Model](../../../spec/adr_022_permissioning_unification.md)
- [ADR-032: Capability Refinement](../../../spec/adr_032_capability_refinement.md) 