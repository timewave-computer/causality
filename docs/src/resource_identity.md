# ResourceRegister Identity

This document outlines the unified identity system for ResourceRegisters in Causality, detailing how resources are uniquely identified, referenced, and resolved across domains using content addressing.

## Overview

The ResourceRegister identity system provides a robust framework for uniquely identifying and tracking resources throughout their lifecycle, regardless of location, state, or domain boundary crossings. With the unified ResourceRegister model and universal content addressing, identities are now cryptographically verifiable, immutable, and intrinsically linked to resource state.

## Identity Structure

### ResourceRegister Identifier

```rust
/// Uniquely identifies a ResourceRegister within the system
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegisterId {
    /// Content hash of the ResourceRegister - derived from its data
    pub content_hash: ContentHash,
    
    /// Domain where the register was originally created
    pub origin_domain: DomainId,
    
    /// Resource type identifier
    pub resource_type: ResourceType,
    
    /// Optional namespace to categorize resources
    pub namespace: Option<String>,
    
    /// Temporal context when this register was created
    pub creation_context: TemporalContext,
}
```

### Content Hash

```rust
/// A content hash derived from a resource's data
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash {
    /// The hash algorithm used
    pub algorithm: HashAlgorithm,
    
    /// The hash bytes
    pub bytes: Vec<u8>,
}

impl ContentHash {
    /// Verify that the provided data matches this hash
    pub fn verify(&self, data: &[u8]) -> bool {
        match self.algorithm {
            HashAlgorithm::Blake3 => {
                let computed_hash = Blake3::hash(data);
                self.bytes == computed_hash.as_bytes()
            },
            // Other algorithms...
        }
    }
}
```

### Temporal Context

```rust
/// Temporal context for a ResourceRegister
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemporalContext {
    /// Map of domain IDs to their time positions
    pub positions: HashMap<DomainId, TimePosition>,
    
    /// When this context was created
    pub observed_at: Timestamp,
    
    /// Hash of this temporal context
    pub context_hash: ContentHash,
}
```

## Content-Addressed Identity Generation

ResourceRegisters are now identified by their content hash, ensuring cryptographic verifiability:

```rust
/// Generates a ResourceRegister identifier using content addressing
pub fn generate_register_id<T: Serialize + ContentAddressed>(
    resource_register: &T,
    domain_id: &DomainId,
    resource_type: ResourceType,
    namespace: Option<String>,
) -> Result<RegisterId, HashError> {
    // Calculate content hash
    let content_hash = calculate_content_hash(resource_register)?;
    
    // Get current temporal context
    let temporal_context = get_current_temporal_context();
    
    RegisterId {
        content_hash,
        origin_domain: domain_id.clone(),
        resource_type,
        namespace,
        creation_context: temporal_context,
    }
}

/// Calculate content hash for any serializable object
pub fn calculate_content_hash<T: Serialize>(object: &T) -> Result<ContentHash, HashError> {
    // Consistently serialize the object
    let serialized = serialize_canonical(object)?;
    
    // Apply the hash function
    let hash = Blake3::hash(&serialized);
    
    // Return the content hash
    Ok(ContentHash {
        algorithm: HashAlgorithm::Blake3,
        bytes: hash.as_bytes().to_vec(),
    })
}
```

## Content-Addressed Identity Resolution

The identity resolution system uses content addressing for efficient resolution:

```rust
/// Resolves a register ID to its current location and state
pub async fn resolve_register_id(
    register_id: &RegisterId,
    context: &ResolutionContext,
) -> Result<RegisterLocation, ResolutionError> {
    // Try to resolve using content addressing first
    if let Some(location) = content_addressed_lookup(&register_id.content_hash, context).await? {
        return Ok(location);
    }
    
    // Check if the register is in the local domain
    if let Some(location) = local_lookup(register_id, context).await? {
        return Ok(location);
    }
    
    // If not found locally, check the global registry
    if let Some(location) = global_registry_lookup(register_id, context).await? {
        return Ok(location);
    }
    
    // If not in registry, query the origin domain
    origin_domain_lookup(register_id, context).await
}
```

### Cross-Domain Content-Addressed Resolution

When ResourceRegisters cross domain boundaries, their content hashes ensure integrity:

```rust
/// Creates a cross-domain reference for a ResourceRegister
pub fn create_cross_domain_reference(
    register_id: &RegisterId,
    target_domain: &DomainId,
) -> CrossDomainReference {
    CrossDomainReference {
        register_id: register_id.clone(),
        target_domain: target_domain.clone(),
        reference_type: ReferenceType::ContentAddressed,
        temporal_context: get_current_temporal_context(),
        content_hash: calculate_cross_domain_reference_hash(register_id, target_domain).unwrap(),
    }
}
```

## Identity Verification through Unified Verification Framework

ResourceRegister identities are verified using the unified verification framework:

```rust
impl Verifiable for RegisterId {
    type Proof = UnifiedProof;
    type Subject = IdentityValidity;
    
    fn prove(&self, context: &VerificationContext) -> Result<Self::Proof, VerificationError> {
        // Generate content verification proof
        let content_proof = generate_content_verification_proof(self, context)?;
        
        // Generate temporal proof
        let temporal_proof = generate_temporal_proof(self, &context.time_map)?;
        
        // Create unified proof
        let proof = UnifiedProof {
            zk_components: None,
            temporal_components: Some(temporal_proof),
            ancestral_components: None,
            logical_components: Some(content_proof),
            cross_domain_components: None,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            signature: None,
        };
        
        Ok(proof)
    }
    
    fn verify(&self, proof: &Self::Proof, context: &VerificationContext) -> Result<bool, VerificationError> {
        // Verify content proof
        let content_valid = if let Some(content_proof) = &proof.logical_components {
            verify_content_hash(self, content_proof, context)?
        } else {
            return Err(VerificationError::MissingProofComponent("logical_components"));
        };
        
        // Verify temporal proof
        let temporal_valid = if let Some(temporal_proof) = &proof.temporal_components {
            verify_temporal_consistency(self, temporal_proof, &context.time_map)?
        } else {
            return Err(VerificationError::MissingProofComponent("temporal_components"));
        };
        
        // All validations must pass
        Ok(content_valid && temporal_valid)
    }
}
```

## Content-Addressed ResourceRegister References

References use content addressing for immutable, verifiable references:

```rust
/// Reference to another ResourceRegister
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentRef<T> {
    /// The content hash
    pub hash: ContentHash,
    
    /// Reference type
    pub reference_type: ReferenceType,
    
    /// Required capabilities for access
    pub required_capabilities: HashSet<Capability>,
    
    /// Relationship type with the referenced resource
    pub relationship_type: Option<RelationshipType>,
    
    /// Domain where this reference is valid
    pub domain_id: DomainId,
    
    /// Phantom type to indicate what this references
    phantom: PhantomData<T>,
}

impl<T: ContentAddressed> ContentRef<T> {
    /// Create a new content reference
    pub fn new(object: &T) -> Self {
        Self {
            hash: object.content_hash(),
            reference_type: ReferenceType::Direct,
            required_capabilities: HashSet::new(),
            relationship_type: None,
            domain_id: DomainId::default(),
            phantom: PhantomData,
        }
    }
    
    /// Resolve this reference to an object
    pub fn resolve(&self, storage: &impl ContentAddressedStorage) -> Result<T, StorageError> {
        storage.get(&self.hash)
    }
}
```

### Reference Types

```rust
/// Types of ResourceRegister references
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceType {
    /// Direct reference to a concrete resource via content hash
    Direct,
    
    /// Content-addressed reference to a resource
    ContentAddressed,
    
    /// Reference that can be satisfied by any resource meeting criteria
    Indirect,
    
    /// Reference that can resolve to one of several resources
    MultiOption(Vec<ContentHash>),
    
    /// Reference that resolves based on runtime conditions
    Conditional(Box<dyn ResolutionCondition + Send + Sync>),
}
```

## Content-Addressed Storage Integration

Resource identities integrate seamlessly with content-addressed storage:

```rust
pub trait ContentAddressedStorage {
    /// Store an object by its content hash
    fn store<T: ContentAddressed>(&self, object: &T) -> Result<ContentHash, StorageError>;
    
    /// Retrieve an object by its content hash
    fn get<T: ContentAddressed>(&self, hash: &ContentHash) -> Result<T, StorageError>;
    
    /// Check if an object exists
    fn exists(&self, hash: &ContentHash) -> Result<bool, StorageError>;
    
    /// List objects matching a pattern
    fn list(&self, pattern: &Pattern) -> Result<Vec<ContentHash>, StorageError>;
}

/// Store a ResourceRegister in content-addressed storage
pub async fn store_resource_register(
    register: &ResourceRegister,
    storage: &impl ContentAddressedStorage,
) -> Result<RegisterId, StorageError> {
    // Store the register
    let content_hash = storage.store(register)?;
    
    // Create register ID
    let register_id = RegisterId {
        content_hash,
        origin_domain: register.domain_id.clone(),
        resource_type: register.resource_type.clone(),
        namespace: register.namespace.clone(),
        creation_context: get_current_temporal_context(),
    };
    
    // Store metadata about this register
    store_register_metadata(&register_id, storage)?;
    
    Ok(register_id)
}
```

## Unified Operation Model for Identity Management

Operations on ResourceRegister identities use the unified operation model:

```rust
// Create an operation to change namespace
let namespace_op = Operation::new(OperationType::UpdateNamespace)
    .with_input(register_ref.clone())
    .with_output(register_ref.with_namespace(new_namespace.clone()))
    .with_parameter("namespace", new_namespace)
    .with_context(AbstractContext::new())
    .with_authorization(auth);

// Execute the operation
let result = execute_operation(namespace_op, identity_executor).await?;

// The operation produces a new RegisterId with a new content hash
let new_register_id = result.outputs[0].id;
```

## Integration with Capability-Based Authorization

ResourceRegister identity now integrates with capability-based authorization:

```rust
/// Capability for accessing register identities
pub struct RegisterIdentityCapability {
    /// The capability ID
    pub id: CapabilityId,
    
    /// The rights this capability grants
    pub rights: HashSet<Right>,
    
    /// Targets this capability applies to
    pub targets: Vec<RegisterId>,
    
    /// Constraints on using this capability
    pub constraints: CapabilityConstraints,
    
    /// How this capability can be delegated
    pub delegation_rules: DelegationRules,
    
    /// When this capability expires (if ever)
    pub expiration: Option<Expiration>,
}

// Validates access to a register identity
pub fn validate_identity_access(
    register_id: &RegisterId,
    entity: &EntityId,
    operation: &Operation<C>,
    capabilities: &[Capability],
) -> Result<bool, AuthError> {
    // Authorize using the authorization service
    authorization_service.authorize(
        entity,
        operation,
        capabilities
    ).map(|result| result.is_authorized())
}
```

## Temporal Validation of Register Identity

Register identities include temporal validation via the unified verification framework:

```rust
/// Validate that a register identity is temporally consistent
pub fn validate_temporal_consistency(
    register_id: &RegisterId,
    current_context: &TemporalContext,
) -> Result<bool, ValidationError> {
    // Check that the register's creation context is not in the future
    for (domain_id, position) in &register_id.creation_context.positions {
        if let Some(current_position) = current_context.positions.get(domain_id) {
            if current_position < position {
                return Ok(false); // Register claims to be from the future
            }
        }
    }
    
    Ok(true)
}
```

## Content-Addressed Identity Evolution

ResourceRegister identity evolution produces new content hashes while maintaining lineage:

```rust
/// Create a new version of a ResourceRegister
pub fn create_register_version<T: ContentAddressed>(
    current_register: &T,
    modifications: impl FnOnce(&mut T) -> Result<(), VersionError>,
) -> Result<(T, RegisterId), VersionError> {
    // Clone the current register
    let mut new_register = current_register.clone();
    
    // Apply modifications
    modifications(&mut new_register)?;
    
    // Calculate new content hash
    let content_hash = calculate_content_hash(&new_register)?;
    
    // Create new register ID
    let register_id = RegisterId {
        content_hash,
        origin_domain: new_register.domain_id.clone(),
        resource_type: new_register.resource_type.clone(),
        namespace: new_register.namespace.clone(),
        creation_context: get_current_temporal_context(),
    };
    
    Ok((new_register, register_id))
}
```

## Content-Addressed Cross-Domain Identity Management

Cross-domain identity management benefits from content addressing:

```rust
/// Creates a cross-domain identity mapping
pub async fn create_cross_domain_identity_mapping(
    source_register_id: &RegisterId,
    target_domain: &DomainId,
    mapping_context: &MappingContext,
) -> Result<CrossDomainMapping, MappingError> {
    // Create the mapping
    let mapping = CrossDomainMapping {
        source_register_id: source_register_id.clone(),
        target_domain: target_domain.clone(),
        mapping_type: MappingType::Mirror,
        temporal_context: get_current_temporal_context(),
        content_hash: calculate_content_hash(&(source_register_id, target_domain))?,
    };
    
    // Store the mapping in content-addressed storage
    mapping_context.storage.store(&mapping)?;
    
    // Update the global registry
    update_global_registry(&mapping, mapping_context).await?;
    
    Ok(mapping)
}
```

## Conclusion

The ResourceRegister identity system leverages universal content addressing and the unified verification framework to provide cryptographically verifiable, immutable identities for resources throughout the Causality ecosystem. With content-addressed references, capability-based authorization, and integration with the unified operation model, it ensures that resources can be identified and tracked securely across domain boundaries while maintaining temporal consistency and provable correctness. 