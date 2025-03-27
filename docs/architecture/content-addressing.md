# Content Addressing System

*The Content Addressing System provides a universal identification mechanism for all stateful objects in Causality.*

*Last updated: 2023-03-26*

## Overview

The Content Addressing System is a foundational component of Causality, providing cryptographic guarantees for all stateful objects through content-derived identifiers. Instead of using arbitrary identifiers like UUIDs, all objects in Causality are identified by cryptographic hashes of their content, ensuring:

1. **Immutability**: Content-addressed objects cannot be changed without changing their identifier
2. **Verifiability**: Object integrity can be verified by recomputing its hash
3. **Tamper-resistance**: Any modification to the object will result in a different hash
4. **Universality**: Identification does not depend on central authorities or arbitrary assignment
5. **Determinism**: The same content always produces the same identifier

This approach is central to Causality's security and trust model, enabling verifiable state transitions and cryptographic proofs of data integrity across domains.

## Core Concepts

### Content Hash

A `ContentHash` is a cryptographic digest that uniquely identifies an object based on its content:

```rust
/// A cryptographic hash that uniquely identifies content
pub struct ContentHash {
    /// Raw hash bytes (32 bytes for Poseidon)
    bytes: [u8; 32],
}
```

Content hashes are computed using the Poseidon hash function, which is optimized for zero-knowledge proof systems and provides strong cryptographic guarantees.

### Content Addressed Trait

Objects that can be content-addressed implement the `ContentAddressed` trait:

```rust
/// Trait for objects that are content-addressed
pub trait ContentAddressed: Sized {
    /// Calculate the content hash of this object
    fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError>;
    
    /// Get the content hash of this object
    fn content_hash(&self) -> &ContentHash;
    
    /// Create a new instance with the given content hash
    fn with_content_hash(self, hash: ContentHash) -> Self;
    
    /// Verify that this object matches its content hash
    fn verify_content_hash(&self) -> bool;
}
```

This trait provides a standard interface for computing, storing, and verifying content hashes for any type of object.

### Content Addressed Storage

The `ContentAddressedStorage` trait defines storage operations for content-addressed objects:

```rust
/// Storage for content-addressed objects
pub trait ContentAddressedStorage: Send + Sync {
    /// Store an object by its content hash
    fn store<T: ContentAddressed>(&self, object: &T) -> Result<ContentHash, ContentAddressingError>;
    
    /// Retrieve an object by its content hash
    fn get<T: ContentAddressed>(&self, hash: &ContentHash) -> Result<T, ContentAddressingError>;
    
    /// Check if an object exists
    fn exists(&self, hash: &ContentHash) -> Result<bool, ContentAddressingError>;
}
```

This abstraction allows different storage backends to be used while maintaining the same content addressing semantics.

## Content Hash Calculation

Content hashes are calculated through a standardized process:

1. **Serialization**: Object is serialized using canonical serialization
2. **Normalization**: Serialized data is normalized to ensure consistent representation
3. **Hashing**: Poseidon hash function is applied to the normalized data

```rust
/// Calculate a content hash
pub fn calculate_content_hash<T: Serialize>(object: &T) -> Result<ContentHash, ContentHashError> {
    // Serialize object to bytes
    let serialized = serde_json::to_vec_pretty(object)
        .map_err(|e| ContentHashError::SerializationError(e.to_string()))?;
    
    // Create a Poseidon hasher
    let mut hasher = PoseidonHasher::new();
    
    // Update with serialized data
    hasher.update(&serialized);
    
    // Finalize and return the content hash
    Ok(ContentHash::from_bytes(hasher.finalize().as_bytes()))
}
```

For consistency and determinism, canonical serialization ensures that the same logical object always produces the same serialized form, regardless of implementation details.

## Advanced Features

### Deferred Hashing

For performance optimization in zero-knowledge contexts, Causality implements a deferred hashing mechanism that allows hash computation to be moved outside of zkVM execution:

```rust
/// Context for deferred hash operations
pub struct DeferredHashingContext {
    /// Queue of pending hash operations
    pending_operations: Vec<HashOperation>,
    /// Computed results
    results: HashMap<OperationId, ContentHash>,
}

/// Operation for calculating a content hash
struct HashOperation {
    /// Operation ID
    id: OperationId,
    /// Object to hash
    object: Box<dyn Serialize + Send + Sync>,
    /// Hash type to use
    hash_type: HashType,
}
```

In this approach:

1. During zkVM execution, a hash operation is registered but not executed
2. A placeholder hash is used for continued processing
3. After VM execution, the actual hash operations are performed
4. The results are verified against commitments made during VM execution

This significantly improves performance while maintaining security guarantees.

### Sparse Merkle Tree Integration

The Content Addressing System integrates with Sparse Merkle Trees (SMTs) for efficient verification:

```rust
/// Sparse Merkle Tree with content-addressed nodes
pub struct ContentAddressedSMT {
    /// Root hash of the tree
    root: ContentHash,
    /// Storage for content-addressed nodes
    storage: Arc<dyn ContentAddressedStorage>,
}

impl ContentAddressedSMT {
    /// Get a proof of inclusion for a key
    pub fn get_proof(&self, key: &[u8]) -> Result<SMTProof, SMTError>;
    
    /// Verify a proof of inclusion
    pub fn verify_proof(&self, key: &[u8], value: &[u8], proof: &SMTProof) -> Result<bool, SMTError>;
    
    /// Insert a key-value pair
    pub fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<ContentHash, SMTError>;
    
    /// Get a value by key
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, SMTError>;
}
```

SMTs provide efficient proofs of inclusion or exclusion for key-value pairs, making them ideal for verifying state in zero-knowledge applications.

## Implementation Patterns

### Implementing ContentAddressed

When implementing the `ContentAddressed` trait, follow these patterns:

```rust
impl ContentAddressed for MyType {
    fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError> {
        let mut hasher = ContentHasher::new();
        
        // Type name provides domain separation
        hasher.update("MyType");
        
        // Add fields in a deterministic order
        hasher.update(&self.field1);
        hasher.update(&self.field2);
        
        // If the type contains other content-addressed types, use their hashes
        hasher.update(self.content_addressed_field.content_hash().as_bytes());
        
        // Handle collections by iterating in a deterministic order
        for item in &self.items {
            hasher.update(&item);
        }
        
        Ok(hasher.finalize())
    }
    
    fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }
    
    fn with_content_hash(mut self, hash: ContentHash) -> Self {
        self.content_hash = hash;
        self
    }
    
    fn verify_content_hash(&self) -> bool {
        match self.calculate_content_hash() {
            Ok(hash) => &hash == self.content_hash(),
            Err(_) => false,
        }
    }
}
```

### Handling Recursion in Content Addressing

For recursive data structures, implement hashing carefully:

```rust
impl ContentAddressed for TreeNode {
    fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError> {
        let mut hasher = ContentHasher::new();
        hasher.update("TreeNode");
        hasher.update(&self.value);
        
        // Hash the children by their content hashes
        for child in &self.children {
            hasher.update(child.content_hash().as_bytes());
        }
        
        Ok(hasher.finalize())
    }
    
    // Other methods...
}
```

This approach ensures that hash computation terminates and avoids infinite recursion.

## Integration with Other Systems

### Resource System Integration

All resources in the Resource System are content-addressed:

```rust
/// A resource identified by its content hash
pub struct ResourceId {
    /// Resource type
    resource_type: String,
    /// Domain ID
    domain: DomainId,
    /// Unique identifier (content hash)
    id: ContentHash,
    /// Content hash of this resource ID
    content_hash: ContentHash,
}

impl ContentAddressed for ResourceId {
    // Implementation...
}
```

### Effect System Integration

Effects use content addressing for identification and verification:

```rust
/// An effect with a content hash
pub struct Effect {
    /// Effect type
    effect_type: EffectType,
    /// Effect parameters
    parameters: HashMap<String, Value>,
    /// Dependencies
    dependencies: Vec<FactId>,
    /// Content hash
    content_hash: ContentHash,
}

impl ContentAddressed for Effect {
    // Implementation...
}
```

### Capability System Integration

Capabilities are content-addressed for secure delegation and verification:

```rust
/// A capability with a content hash
pub struct Capability {
    /// Target resource
    target: ResourceId,
    /// Capability type
    capability_type: CapabilityType,
    /// Constraints
    constraints: Vec<CapabilityConstraint>,
    /// Expiration time
    expires_at: Option<DateTime<Utc>>,
    /// Content hash
    content_hash: ContentHash,
}

impl ContentAddressed for Capability {
    // Implementation...
}
```

## Affected Components and Location

The Content Addressing System touches every part of Causality, but its core components are located at:

| Component | Purpose | Location |
|-----------|---------|----------|
| Content Hash | Cryptographic identifier | `causality_types::hash::ContentHash` |
| Content Addressed Trait | Interface for content addressing | `causality_types::hash::ContentAddressed` |
| Content Addressed Storage | Storage abstraction | `causality_core::storage::ContentAddressedStorage` |
| Deferred Hashing | Performance optimization | `causality_vm::deferred::DeferredHashingContext` |
| SMT Integration | Efficient verification | `causality_crypto::smt::ContentAddressedSMT` |
| Poseidon Implementation | ZK-friendly hash function | `causality_crypto::poseidon::PoseidonHasher` |

## References

- [ADR-007: Content Addressing](../../adrs/adr_007_content_addressing.md)
- [ADR-028: Unified Hash Format](../../adrs/adr_028_unified_hash_format.md)
- [ADR-029: SMT Integration](../../adrs/adr_029_smt_integration.md)
- [ADR-030: Deferred Hashing](../../adrs/adr_030_deferred_hashing.md)
- [System Specification: Content Addressing System](../../../../spec/spec.md#1-content-addressing-system-adr-007-adr-028-adr-029-adr-030)
