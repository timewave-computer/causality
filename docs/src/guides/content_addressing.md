<!-- Guide for content addressing -->
<!-- Original file: docs/src/content_addressing_guide.md -->

# Content Addressing Guide

This guide explains the content addressing system implemented in the Causality project and provides instructions for correctly using it in future development.

## Overview

The Universal Content Addressing system replaces traditional UUID-based identifiers with content-derived identifiers. This approach offers several advantages:

1. **Deterministic Identity**: Objects with the same content always have the same identifier
2. **Data Verification**: Automatic verification of data integrity using cryptographic hashes
3. **Deduplication**: Natural deduplication of identical content
4. **Distributed Storage**: Better compatibility with distributed and decentralized storage systems

## Core Components

The content addressing system consists of the following key components:

### 1. Content Hashing (`crypto/hash.rs`)

- `HashOutput`: A struct containing the raw hash bytes and algorithm information
- `ContentId`: A content-derived identifier based on a hash
- `HashFunction`: Trait for hash function implementations
- `ContentHasher`: Extension trait for content addressing support
- `ContentAddressed`: Trait for objects that can be content-addressed

### 2. Content Storage (`crypto/content_addressed_storage.rs`)

- `ContentAddressedStorage`: Interface for storing and retrieving content-addressed objects
- `InMemoryStorage`: Reference implementation of content-addressed storage
- `StorageFactory`: Factory for creating storage instances

### 3. SMT Integration (`crypto/smt.rs`)

- Sparse Merkle Tree integration for efficient key-value storage
- `MerkleSmt`: Implementation of a Sparse Merkle Tree
- `ContentAddressedSmt`: Trait for content-addressed SMT operations

### 4. Nullifier Tracking (`crypto/nullifier.rs`)

- System for tracking one-time use objects
- `Nullifier`: Represents a nullifier for a content-addressed object
- `NullifierTracking`: Interface for nullifier operations

### 5. Deferred Hashing (`crypto/deferred.rs`)

- Support for deferred hash computation
- `DeferredHashingContext`: Context for deferred hashing operations
- `DeferredHashBatchProcessor`: Batch processor for deferred hash operations

## Using Content Addressing

### Implementing ContentAddressed Trait

To make a struct content-addressable, implement the `ContentAddressed` trait:

```rust
use borsh::{BorshSerialize, BorshDeserialize};
use crate::crypto::hash::{ContentAddressed, HashOutput, HashError, HashFactory};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct MyObject {
    pub name: String,
    pub value: u64,
    pub data: Vec<u8>,
}

impl ContentAddressed for MyObject {
    fn content_hash(&self) -> HashOutput {
        // Get the configured hasher
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        
        // Create a canonical serialization
        let data = self.try_to_vec().unwrap();
        
        // Compute hash with configured hasher
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}
```

### Storing and Retrieving Content-Addressed Objects

```rust
use crate::crypto::content_addressed_storage::{ContentAddressedStorage, InMemoryStorage};
use crate::crypto::hash::ContentId;

// Create storage
let storage = InMemoryStorage::new();

// Store an object
let my_object = MyObject {
    name: "example".to_string(),
    value: 42,
    data: vec![1, 2, 3],
};

let content_id: ContentId = storage.store(&my_object)?;

// Retrieve the object
let retrieved: MyObject = storage.get(&content_id)?;

// Verify the object
assert!(retrieved.verify());
```

### Using the SMT for Content Addressed Storage

```rust
use crate::crypto::smt::{SmtFactory, ContentAddressedSmt};

// Create an SMT
let factory = SmtFactory::default();
let smt = factory.create_default_smt();

// Store an object with proof
let (hash, proof) = smt.store_with_proof(&my_object)?;

// Retrieve an object with proof
let (retrieved, retrieval_proof): (MyObject, _) = smt.get_with_proof(&hash)?;

// Verify inclusion
assert!(smt.verify_inclusion(&smt.root(), &hash, &proof));
```

### Replacing UUID-based Identifiers

When replacing UUID-based identifiers, follow these principles:

1. **Determine Content**: Identify the meaningful content that should determine the identity
2. **Canonical Serialization**: Use a deterministic serialization method (Borsh is recommended)
3. **Hash Computation**: Use the configured hasher to compute the hash
4. **Content ID Creation**: Create a ContentId from the hash

Example of replacing a UUID-based ID:

```rust
// Before
let message_id = Uuid::new_v4().to_string();

// After
let message_id = message.content_id().to_string();
```

For cases where the object doesn't exist yet:

```rust
// Before
let resource_id = ResourceId::from_str(&format!("asset-{}", Uuid::new_v4()));

// After
let properties = AssetProperties {
    name: "my_asset".to_string(),
    value: 100,
    // other properties
};

// Create a ContentId from the properties
let hash_factory = HashFactory::default();
let hasher = hash_factory.create_hasher().unwrap();
let data = properties.try_to_vec().unwrap();
let hash = hasher.hash(&data);
let content_id = ContentId::from(hash);

let resource_id = ResourceId::from_content_id("asset", &content_id);
```

## Best Practices

1. **Immutable Content**: Content-addressed objects should be immutable. If an object changes, its identity changes.
2. **Canonical Serialization**: Always use a deterministic serialization method (like Borsh).
3. **Hash Algorithm**: Use the configured hash algorithm from HashFactory rather than hardcoding a specific algorithm.
4. **Error Handling**: Handle errors properly in hash computation, serialization, and storage operations.
5. **Testing**: Write tests to verify that content addressing works correctly for your objects.
6. **Performance**: For performance-critical code paths, consider using deferred hashing.

## Debugging Tips

1. **Hash Mismatch**: If hashes don't match, check serialization consistency and ensure no fields are excluded.
2. **Storage Errors**: Check that the storage implementation is correctly returning errors and not silently failing.
3. **Content Not Found**: Verify that the ContentId is correctly computed and that objects are stored before retrieval.

## Future Directions

The content addressing system is designed to be extensible. Future enhancements may include:

1. **Advanced SMT Features**: Additional SMT capabilities like range proofs and batch operations
2. **External Storage Backends**: Implementations for various storage backends (IPFS, databases, etc.)
3. **Cross-Domain Verification**: Enhanced cross-domain content verification mechanisms

## References

- [ADR-011: Universal Content Addressing](work/011.md) - The full implementation plan
- [Sparse Merkle Tree Specification](https://github.com/nervosnetwork/sparse-merkle-tree) - Details on the SMT implementation used
- [Borsh Serialization](https://borsh.io/) - The canonical serialization format used 