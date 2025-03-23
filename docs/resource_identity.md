# Resource Identity

This document outlines the resource identity system in Causality, detailing how resources are uniquely identified, referenced, and resolved across domains.

## Overview

Resource identity in Causality provides a robust framework for uniquely identifying and tracking resources throughout their lifecycle, regardless of location, state, or domain boundary crossings. The identity system ensures consistency, traceability, and security in resource management while supporting cross-domain operations.

## Identity Structure

### Resource Identifier

```rust
/// Uniquely identifies a resource within the system
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId {
    /// Base identifier - typically a UUID or similar unique value
    pub base_id: String,
    
    /// Domain where the resource was originally created
    pub origin_domain: DomainId,
    
    /// Resource type identifier
    pub resource_type: ResourceType,
    
    /// Optional namespace to categorize resources
    pub namespace: Option<String>,
    
    /// Version information for the resource
    pub version: VersionInfo,
}
```

### Version Information

```rust
/// Tracks the version information for a resource
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Semantic version of the resource
    pub semantic_version: String,
    
    /// State transition counter
    pub state_transition_count: u64,
    
    /// Timestamp of last modification
    pub last_modified: TimeSnapshot,
}
```

### Domain Identifier

```rust
/// Uniquely identifies a domain in the system
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DomainId {
    /// Domain identifier - typically a registered namespace
    pub id: String,
    
    /// Verification key for the domain
    pub verification_key: Option<PublicKey>,
}
```

## Identity Generation

Resources are assigned identifiers at creation time:

```rust
/// Generates a new unique resource identifier
pub fn generate_resource_id(
    resource_type: ResourceType,
    domain_id: &DomainId,
    namespace: Option<String>,
) -> ResourceId {
    let base_id = generate_uuid();
    
    ResourceId {
        base_id,
        origin_domain: domain_id.clone(),
        resource_type,
        namespace,
        version: VersionInfo {
            semantic_version: "1.0.0".to_string(),
            state_transition_count: 0,
            last_modified: get_current_time(),
        },
    }
}
```

## Identity Resolution

The identity resolution system translates identifiers across domains:

```rust
/// Resolves a resource ID to its current location and state
pub async fn resolve_resource_id(
    resource_id: &ResourceId,
    context: &ResolutionContext,
) -> Result<ResourceLocation> {
    // Check if the resource is in the local domain
    if let Some(location) = local_lookup(resource_id, context).await? {
        return Ok(location);
    }
    
    // If not found locally, check the global registry
    if let Some(location) = global_registry_lookup(resource_id, context).await? {
        return Ok(location);
    }
    
    // If not in registry, query the origin domain
    origin_domain_lookup(resource_id, context).await
}
```

### Cross-Domain Resolution

When resources cross domain boundaries, their identities are preserved while their locations are tracked:

```rust
/// Creates a cross-domain reference for a resource
pub fn create_cross_domain_reference(
    resource_id: &ResourceId,
    target_domain: &DomainId,
) -> CrossDomainReference {
    CrossDomainReference {
        original_id: resource_id.clone(),
        target_domain: target_domain.clone(),
        reference_type: ReferenceType::Direct,
        timestamp: get_current_time(),
    }
}
```

## Identity Verification

Resource identities include mechanisms for verification:

```rust
/// Verifies a resource identity against a signature
pub fn verify_resource_identity(
    resource_id: &ResourceId,
    signature: &Signature,
    verification_context: &VerificationContext,
) -> Result<bool> {
    // Verify the signature matches the resource ID
    let verification_result = verify_signature(
        resource_id_to_bytes(resource_id),
        signature,
        &verification_context.verification_key,
    )?;
    
    // Check additional verification rules specific to resource type
    if verification_result {
        verify_resource_type_specific_rules(resource_id, verification_context)?
    } else {
        Ok(false)
    }
}
```

## Resource References

References allow resources to be linked without direct coupling:

```rust
/// Reference to another resource
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceReference {
    /// Target resource ID
    pub target_id: ResourceId,
    
    /// Reference type
    pub reference_type: ReferenceType,
    
    /// Capabilities needed to access the resource
    pub required_capabilities: Vec<Capability>,
    
    /// Relationship type with the referenced resource
    pub relationship_type: Option<RelationshipType>,
}
```

### Reference Types

```rust
/// Types of resource references
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceType {
    /// Direct reference to a concrete resource
    Direct,
    
    /// Reference that can be satisfied by any resource meeting criteria
    Indirect,
    
    /// Reference that can resolve to one of several resources
    MultiOption,
    
    /// Reference that resolves based on runtime conditions
    Conditional(Box<dyn ResolutionCondition>),
}
```

## Identity Persistence

Resource identities are persisted in multiple ways:

1. **Local Storage**: Each domain maintains a local registry of resource identities
2. **Global Registry**: A shared registry maintains cross-domain identity mappings
3. **Blockchain Anchoring**: Critical identities can be anchored in blockchain for immutability

```rust
/// Persists a resource identity to storage
pub async fn persist_resource_identity(
    resource_id: &ResourceId,
    storage_context: &StorageContext,
) -> Result<()> {
    // Store in local registry
    store_in_local_registry(resource_id, storage_context).await?;
    
    // If configured, also store in global registry
    if storage_context.use_global_registry {
        store_in_global_registry(resource_id, storage_context).await?;
    }
    
    // If critical resource, anchor in blockchain
    if is_critical_resource(resource_id) {
        anchor_in_blockchain(resource_id, storage_context).await?;
    }
    
    Ok(())
}
```

## Identity Evolution

Resource identities evolve over time:

1. **Version Updates**: Semantic versioning for significant changes
2. **State Transitions**: State change tracking through transition counts
3. **Temporal Snapshots**: Time-based identity snapshots at key points

```rust
/// Updates the version information for a resource
pub fn update_resource_version(
    resource_id: &mut ResourceId,
    update_type: VersionUpdateType,
) {
    match update_type {
        VersionUpdateType::Major => {
            let version_parts: Vec<&str> = resource_id.version.semantic_version.split('.').collect();
            if version_parts.len() >= 3 {
                let major = version_parts[0].parse::<u32>().unwrap_or(0) + 1;
                resource_id.version.semantic_version = format!("{}.0.0", major);
            }
        },
        VersionUpdateType::Minor => {
            let version_parts: Vec<&str> = resource_id.version.semantic_version.split('.').collect();
            if version_parts.len() >= 3 {
                let major = version_parts[0].parse::<u32>().unwrap_or(0);
                let minor = version_parts[1].parse::<u32>().unwrap_or(0) + 1;
                resource_id.version.semantic_version = format!("{}.{}.0", major, minor);
            }
        },
        VersionUpdateType::Patch => {
            let version_parts: Vec<&str> = resource_id.version.semantic_version.split('.').collect();
            if version_parts.len() >= 3 {
                let major = version_parts[0].parse::<u32>().unwrap_or(0);
                let minor = version_parts[1].parse::<u32>().unwrap_or(0);
                let patch = version_parts[2].parse::<u32>().unwrap_or(0) + 1;
                resource_id.version.semantic_version = format!("{}.{}.{}", major, minor, patch);
            }
        },
        VersionUpdateType::StateTransition => {
            resource_id.version.state_transition_count += 1;
        },
    }
    
    resource_id.version.last_modified = get_current_time();
}
```

## Identity Namespaces

Namespaces provide logical grouping of resources:

```rust
/// Registers a new namespace
pub async fn register_namespace(
    namespace: &str,
    owner: &IdentityId,
    registration_context: &RegistrationContext,
) -> Result<NamespaceInfo> {
    // Verify the namespace is available
    check_namespace_availability(namespace, registration_context).await?;
    
    // Create the namespace information
    let namespace_info = NamespaceInfo {
        name: namespace.to_string(),
        owner: owner.clone(),
        registration_time: get_current_time(),
        permissions: default_namespace_permissions(),
    };
    
    // Register the namespace
    persist_namespace(&namespace_info, registration_context).await?;
    
    Ok(namespace_info)
}
```

## Identity Security

Security measures protect resource identities:

1. **Cryptographic Verification**: Digital signatures verify authenticity
2. **Capability-Based Access**: Capabilities control access to identities
3. **Temporal Validation**: Time-based validation of identity claims

```rust
/// Validates access to a resource identity
pub fn validate_identity_access(
    resource_id: &ResourceId,
    accessor: &IdentityId,
    requested_operation: &Operation,
    security_context: &SecurityContext,
) -> Result<bool> {
    // Check if the accessor has the required capabilities
    let required_capabilities = get_required_capabilities_for_operation(resource_id, requested_operation);
    
    if !has_sufficient_capabilities(accessor, &required_capabilities, security_context)? {
        return Ok(false);
    }
    
    // Check temporal validity
    if !is_temporally_valid(resource_id, security_context.current_time)? {
        return Ok(false);
    }
    
    // Check additional security policies
    validate_against_security_policies(resource_id, accessor, requested_operation, security_context)
}
```

## Integration with Resource Lifecycle

Resource identity is tightly integrated with the resource lifecycle:

```rust
// Lifecycle integration pseudocode
pub fn lifecycle_transition(
    resource_id: &mut ResourceId,
    from_state: &RegisterState,
    to_state: &RegisterState,
    transition_context: &TransitionContext,
) -> Result<()> {
    // Update the resource identity for the state transition
    update_resource_version(resource_id, VersionUpdateType::StateTransition);
    
    // Update the identity registry with the new version
    update_identity_registry(resource_id, from_state, to_state, transition_context)?;
    
    // If crossing domain boundaries, create cross-domain references
    if transition_context.is_cross_domain {
        create_and_register_cross_domain_reference(
            resource_id, 
            &transition_context.target_domain,
            transition_context,
        )?;
    }
    
    Ok(())
}
```

## Global Identity Resolution

The global identity resolution system provides consistent identity resolution across all domains:

```rust
/// Global identity resolution system
pub struct GlobalIdentityResolver {
    /// Registry connection for identity lookups
    registry_connection: RegistryConnection,
    
    /// Cache for resolved identities
    identity_cache: IdentityCache,
    
    /// Resolution policies
    resolution_policies: Vec<ResolutionPolicy>,
}

impl GlobalIdentityResolver {
    /// Resolves a resource ID globally
    pub async fn resolve(&self, resource_id: &ResourceId) -> Result<ResolvedIdentity> {
        // Check cache first
        if let Some(cached) = self.identity_cache.get(resource_id) {
            if !is_cache_stale(&cached, self.cache_policy) {
                return Ok(cached);
            }
        }
        
        // Query the global registry
        let resolved = self.registry_connection.query_identity(resource_id).await?;
        
        // Update cache
        self.identity_cache.insert(resource_id.clone(), resolved.clone());
        
        Ok(resolved)
    }
}
```

## Conclusion

The resource identity system provides a comprehensive framework for managing resource identification throughout the Causality ecosystem. By combining unique identifiers, cryptographic verification, cross-domain resolution, and lifecycle integration, it ensures that resources can be tracked, referenced, and accessed consistently and securely across domain boundaries, while maintaining the temporal integrity of identity information. 