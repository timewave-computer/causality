# Implementing the Content Addressing System

*This guide provides practical implementations for working with the [Content Addressing System](../../architecture/core/content-addressing.md).*

*Last updated: 2023-03-26*

## Overview

This guide covers the practical aspects of implementing and working with the Content Addressing System in Causality. It provides code examples, best practices, and implementation patterns for creating and using content-addressed objects.

## Prerequisites

Before implementing content addressing in your code, make sure you're familiar with:

- The [Content Addressing Architecture](../../architecture/core/content-addressing.md)
- General concepts of cryptographic hashing
- Serialization with Serde in Rust

## Implementation Guide

### Required Crates and Imports

```rust
// Core content addressing types
use causality_types::{
    hash::{
        ContentHash, ContentAddressed, ContentHasher,
        ContentHashError, ContentAddressingError
    },
    serialization::CanonicalSerialize,
};

// Storage for content-addressed objects
use causality_core::storage::{
    ContentAddressedStorage, MemoryContentStore,
    PersistentContentStore
};

// Poseidon hash implementation
use causality_crypto::poseidon::{
    PoseidonHasher, PoseidonParams
};

// Deferred hashing (for zkVM contexts)
use causality_vm::deferred::{
    DeferredHashingContext, HashOperation, 
    OperationId, HashType
};

// Merkle tree implementation
use causality_crypto::smt::{
    ContentAddressedSMT, SMTProof, SMTError
};

// Standard imports
use std::{
    collections::HashMap,
    sync::Arc,
};
use serde::{Serialize, Deserialize};
```

### Creating Content-Addressed Types

To make your types content-addressed, implement the `ContentAddressed` trait:

```rust
/// A simple content-addressed message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message sender
    pub sender: String,
    /// Message content
    pub content: String,
    /// Message timestamp
    pub timestamp: u64,
    /// Content hash (calculated from other fields)
    content_hash: ContentHash,
}

impl Message {
    /// Create a new message
    pub fn new(sender: String, content: String, timestamp: u64) -> Result<Self, ContentHashError> {
        let mut message = Self {
            sender,
            content,
            timestamp,
            content_hash: ContentHash::default(),
        };
        
        // Calculate and set the content hash
        message.content_hash = message.calculate_content_hash()?;
        
        Ok(message)
    }
}

impl ContentAddressed for Message {
    fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError> {
        let mut hasher = ContentHasher::new();
        
        // Type name provides domain separation
        hasher.update("Message");
        
        // Add fields in a deterministic order
        hasher.update(&self.sender);
        hasher.update(&self.content);
        hasher.update(&self.timestamp);
        
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
            Ok(hash) => &hash == &self.content_hash,
            Err(_) => false,
        }
    }
}
```

### Using Content-Addressed Storage

```rust
/// Store and retrieve content-addressed objects
async fn content_store_example() -> Result<(), ContentAddressingError> {
    // Create a memory-based content store
    let store = MemoryContentStore::new();
    
    // Create a message
    let message = Message::new(
        "alice".to_string(),
        "Hello, world!".to_string(),
        1234567890,
    )?;
    
    // Store the message
    let hash = store.store(&message)?;
    println!("Stored message with hash: {}", hash);
    
    // Retrieve the message by its hash
    let retrieved: Message = store.get(&hash)?;
    println!("Retrieved message: {:?}", retrieved);
    
    // Verify the content hash
    assert!(retrieved.verify_content_hash());
    
    // Check if a hash exists
    assert!(store.exists(&hash)?);
    
    // Try to retrieve a non-existent hash
    let random_hash = ContentHash::from_bytes(&[0u8; 32]);
    match store.get::<Message>(&random_hash) {
        Ok(_) => panic!("Should not retrieve non-existent hash"),
        Err(e) => println!("Expected error: {}", e),
    }
    
    Ok(())
}
```

### Persistent Content-Addressed Storage

```rust
/// Create a persistent content store
fn create_persistent_store() -> Result<PersistentContentStore, ContentAddressingError> {
    // Config for the store
    let config = PersistentStoreConfig {
        path: "/path/to/content/store".into(),
        cache_size_mb: 100,
        compression_level: 7,
    };
    
    // Create the store
    let store = PersistentContentStore::new(config)?;
    
    Ok(store)
}

/// Use a persistent content store
async fn persistent_store_example() -> Result<(), ContentAddressingError> {
    // Create or open a persistent store
    let store = create_persistent_store()?;
    
    // Create a message
    let message = Message::new(
        "bob".to_string(),
        "Stored persistently".to_string(),
        1234567890,
    )?;
    
    // Store the message
    let hash = store.store(&message)?;
    
    // Sync to ensure data is written
    store.sync()?;
    
    // Retrieve the message
    let retrieved: Message = store.get(&hash)?;
    assert_eq!(retrieved.content, "Stored persistently");
    
    Ok(())
}
```

### Working with Deferred Hashing

```rust
/// Use deferred hashing for better performance in zkVM contexts
async fn deferred_hashing_example() -> Result<(), ContentHashError> {
    // Create a deferred hashing context
    let mut context = DeferredHashingContext::new();
    
    // Register a hash operation but don't compute it yet
    let operation_id = context.register_hash_operation(
        "Message".to_string(),
        Box::new(Message::new(
            "alice".to_string(),
            "Deferred hashing".to_string(),
            1234567890,
        )?),
        HashType::Poseidon,
    );
    
    // Get a placeholder hash for zkVM execution
    let placeholder = context.get_placeholder_hash(operation_id)?;
    
    // Use the placeholder during zkVM execution
    // ...zkVM execution happens here...
    
    // After VM execution, compute the actual hashes
    context.compute_pending_hashes()?;
    
    // Get the computed hash
    let actual_hash = context.get_hash_result(operation_id)?;
    
    // Verify that the placeholder commitment matches the actual hash
    assert!(context.verify_commitment(operation_id, &actual_hash)?);
    
    Ok(())
}
```

### Using Content-Addressed Sparse Merkle Trees

```rust
/// Work with a content-addressed Sparse Merkle Tree
async fn smt_example() -> Result<(), SMTError> {
    // Create a content store for the SMT nodes
    let store = Arc::new(MemoryContentStore::new());
    
    // Create a new SMT
    let mut smt = ContentAddressedSMT::new(store.clone());
    
    // Insert key-value pairs
    let key1 = "user:alice".as_bytes();
    let value1 = serde_json::to_vec(&json!({
        "balance": 100,
        "nonce": 5
    }))?;
    
    smt.insert(key1, &value1)?;
    
    let key2 = "user:bob".as_bytes();
    let value2 = serde_json::to_vec(&json!({
        "balance": 50,
        "nonce": 3
    }))?;
    
    smt.insert(key2, &value2)?;
    
    // Get the current root hash
    let root_hash = smt.root();
    println!("SMT root hash: {}", root_hash);
    
    // Get a proof of inclusion
    let proof = smt.get_proof(key1)?;
    
    // Verify the proof
    assert!(smt.verify_proof(key1, &value1, &proof)?);
    
    // Try to verify with incorrect value
    let incorrect_value = serde_json::to_vec(&json!({"balance": 999}))?;
    assert!(!smt.verify_proof(key1, &incorrect_value, &proof)?);
    
    // Get a value
    let retrieved = smt.get(key1)?;
    assert_eq!(retrieved, Some(value1));
    
    Ok(())
}
```

## Best Practices

### Content Hash Calculation

1. **Include Type Information**
   ```rust
   // GOOD: Include type name for domain separation
   fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError> {
       let mut hasher = ContentHasher::new();
       hasher.update("MyType");  // Type name for domain separation
       // Rest of implementation...
   }
   
   // BAD: No type information
   fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError> {
       let mut hasher = ContentHasher::new();
       // Missing type information
       // Rest of implementation...
   }
   ```

2. **Deterministic Field Order**
   ```rust
   // GOOD: Fields in deterministic order
   fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError> {
       let mut hasher = ContentHasher::new();
       hasher.update("Record");
       hasher.update(&self.id);         // Always first field
       hasher.update(&self.name);       // Always second field
       hasher.update(&self.timestamp);  // Always third field
       Ok(hasher.finalize())
   }
   
   // BAD: Non-deterministic field order
   fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError> {
       let mut hasher = ContentHasher::new();
       
       // Order depends on HashMap iteration which is non-deterministic
       for (key, value) in &self.fields {
           hasher.update(key);
           hasher.update(value);
       }
       
       Ok(hasher.finalize())
   }
   ```

3. **Handle Nested Content-Addressed Types**
   ```rust
   // GOOD: Use content hash of nested types
   fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError> {
       let mut hasher = ContentHasher::new();
       hasher.update("Parent");
       hasher.update(self.child.content_hash().as_bytes());
       Ok(hasher.finalize())
   }
   
   // BAD: Recalculate nested type hash
   fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError> {
       let mut hasher = ContentHasher::new();
       hasher.update("Parent");
       
       // Inconsistent with child's own hash calculation
       hasher.update(&self.child.field1);
       hasher.update(&self.child.field2);
       
       Ok(hasher.finalize())
   }
   ```

### Storage Best Practices

1. **Error Handling**
   ```rust
   // GOOD: Handle specific errors
   match store.get::<Message>(&hash) {
       Ok(message) => {
           // Handle successful retrieval
       },
       Err(ContentAddressingError::NotFound(h)) => {
           println!("Message with hash {} not found", h);
       },
       Err(ContentAddressingError::DeserializationError(e)) => {
           println!("Failed to deserialize message: {}", e);
       },
       Err(e) => {
           println!("Unexpected error: {}", e);
       }
   }
   
   // BAD: Propagate generic errors
   let message = store.get::<Message>(&hash)?;
   ```

2. **Content Verification**
   ```rust
   // GOOD: Verify content hash after retrieval
   let message: Message = store.get(&hash)?;
   if !message.verify_content_hash() {
       return Err(ContentAddressingError::HashMismatch);
   }
   
   // BAD: Skip verification
   let message: Message = store.get(&hash)?;
   // Missing verification
   ```

3. **Batch Operations**
   ```rust
   // GOOD: Use batched operations for efficiency
   let messages = vec![message1, message2, message3];
   store.store_batch(&messages)?;
   
   // BAD: Individual operations
   store.store(&message1)?;
   store.store(&message2)?;
   store.store(&message3)?;
   ```

## Testing Content Addressing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_hash_calculation() {
        // Create two identical messages
        let message1 = Message::new(
            "alice".to_string(),
            "Hello, world!".to_string(),
            1234567890,
        ).unwrap();
        
        let message2 = Message::new(
            "alice".to_string(),
            "Hello, world!".to_string(),
            1234567890,
        ).unwrap();
        
        // Create a different message
        let message3 = Message::new(
            "alice".to_string(),
            "Different content".to_string(),
            1234567890,
        ).unwrap();
        
        // Identical messages should have identical hashes
        assert_eq!(message1.content_hash(), message2.content_hash());
        
        // Different messages should have different hashes
        assert_ne!(message1.content_hash(), message3.content_hash());
        
        // Verify content hashes
        assert!(message1.verify_content_hash());
        assert!(message2.verify_content_hash());
        assert!(message3.verify_content_hash());
    }
    
    #[test]
    fn test_content_storage() {
        // Create a memory store
        let store = MemoryContentStore::new();
        
        // Create a message
        let message = Message::new(
            "alice".to_string(),
            "Test message".to_string(),
            1234567890,
        ).unwrap();
        
        // Store the message
        let hash = store.store(&message).unwrap();
        
        // Retrieve the message
        let retrieved: Message = store.get(&hash).unwrap();
        
        // Verify content matches
        assert_eq!(retrieved.sender, "alice");
        assert_eq!(retrieved.content, "Test message");
        assert_eq!(retrieved.timestamp, 1234567890);
        
        // Verify hash matches
        assert_eq!(retrieved.content_hash(), message.content_hash());
    }
    
    #[tokio::test]
    async fn test_smt_operations() {
        // Create a store for the SMT
        let store = Arc::new(MemoryContentStore::new());
        
        // Create an SMT
        let mut smt = ContentAddressedSMT::new(store);
        
        // Add some values
        let key1 = "key1".as_bytes();
        let value1 = "value1".as_bytes();
        smt.insert(key1, value1).unwrap();
        
        // Get a proof
        let proof = smt.get_proof(key1).unwrap();
        
        // Verify the proof
        assert!(smt.verify_proof(key1, value1, &proof).unwrap());
        
        // Verify with wrong value fails
        let wrong_value = "wrong".as_bytes();
        assert!(!smt.verify_proof(key1, wrong_value, &proof).unwrap());
        
        // Get a value
        let retrieved = smt.get(key1).unwrap();
        assert_eq!(retrieved, Some(value1.to_vec()));
    }
}
```

## Troubleshooting

### Common Issues and Solutions

| Problem | Possible Cause | Solution |
|---------|---------------|----------|
| Hash mismatch | Object modified after hash calculation | Recalculate hash whenever object changes |
| | Inconsistent serialization | Use canonical serialization |
| | Hash calculation function changed | Maintain backward compatibility |
| Content not found | Hash calculated incorrectly | Verify hash calculation algorithm |
| | Object not stored yet | Check if store operation succeeded |
| | Wrong storage instance | Ensure using correct storage instance |
| Serialization errors | Type not serializable | Implement Serialize/Deserialize |
| | Custom serialization logic | Use canonical serialization format |
| Performance issues | Large objects | Consider using deferred hashing |
| | Many small operations | Use batch operations when possible |
| | Hash calculation in hot paths | Cache hashes when appropriate |

### Diagnosing Hash Calculation Issues

If you're having problems with hash calculation, try the following:

```rust
/// Debug hash calculation
fn debug_hash_calculation<T: ContentAddressed + std::fmt::Debug>(object: &T) {
    println!("Object: {:?}", object);
    
    match object.calculate_content_hash() {
        Ok(hash) => {
            println!("Calculated hash: {}", hash);
            println!("Stored hash: {}", object.content_hash());
            println!("Hashes match: {}", hash == *object.content_hash());
        },
        Err(e) => {
            println!("Error calculating hash: {}", e);
        }
    }
}
```

### Common Patterns for Fixing Hash Issues

```rust
// Fixing hashes in existing objects
fn fix_content_hashes<T: ContentAddressed>(objects: &mut Vec<T>) -> Result<(), ContentHashError> {
    for object in objects {
        let calculated_hash = object.calculate_content_hash()?;
        if calculated_hash != *object.content_hash() {
            *object = object.clone().with_content_hash(calculated_hash);
        }
    }
    
    Ok(())
}
```

## Advanced Usage

### Custom Hash Functions

```rust
/// Use a custom hash function
fn custom_hashing_example() -> Result<ContentHash, ContentHashError> {
    // Create custom Poseidon parameters
    let params = PoseidonParams::new(8, 57);
    
    // Create a custom hasher
    let mut hasher = PoseidonHasher::with_params(params);
    
    // Use the hasher
    hasher.update("MyType");
    hasher.update(&"field1");
    hasher.update(&42u64);
    
    // Finalize and get the hash
    let hash = ContentHash::from_bytes(hasher.finalize().as_bytes());
    
    Ok(hash)
}
```

### Integrating with zkVM

```rust
/// Integrate content addressing with zkVM
fn zk_integration_example() {
    // VM context
    let mut vm_context = ZkVmContext::new();
    
    // Create a deferred hashing context
    let mut hash_context = DeferredHashingContext::new();
    
    // Register hash operations
    let op1 = hash_context.register_hash_operation(
        "User".to_string(),
        Box::new(User { id: "alice", balance: 100 }),
        HashType::Poseidon,
    );
    
    let op2 = hash_context.register_hash_operation(
        "Transaction".to_string(),
        Box::new(Transaction { 
            from: "alice", 
            to: "bob", 
            amount: 50 
        }),
        HashType::Poseidon,
    );
    
    // Get placeholder hashes
    let user_hash = hash_context.get_placeholder_hash(op1).unwrap();
    let tx_hash = hash_context.get_placeholder_hash(op2).unwrap();
    
    // Use placeholders in VM execution
    vm_context.add_input("user_hash", user_hash);
    vm_context.add_input("tx_hash", tx_hash);
    
    // Execute VM
    vm_context.execute().unwrap();
    
    // After VM execution, compute actual hashes
    hash_context.compute_pending_hashes().unwrap();
    
    // Get actual hashes
    let actual_user_hash = hash_context.get_hash_result(op1).unwrap();
    let actual_tx_hash = hash_context.get_hash_result(op2).unwrap();
    
    // Verify hash commitments
    assert!(hash_context.verify_commitment(op1, &actual_user_hash).unwrap());
    assert!(hash_context.verify_commitment(op2, &actual_tx_hash).unwrap());
}
```

### Cross-Domain Content Verification

```rust
/// Verify content across domains
async fn cross_domain_verification(
    source_store: &dyn ContentAddressedStorage,
    target_store: &dyn ContentAddressedStorage,
    hash: &ContentHash,
) -> Result<bool, ContentAddressingError> {
    // Check if the object exists in the source domain
    if !source_store.exists(hash)? {
        return Ok(false);
    }
    
    // Get the object from the source domain
    let object: Message = source_store.get(hash)?;
    
    // Store in the target domain if it doesn't exist
    if !target_store.exists(hash)? {
        target_store.store(&object)?;
    }
    
    // Verify the object in the target domain
    let target_object: Message = target_store.get(hash)?;
    
    // Check content hashes match
    Ok(target_object.content_hash() == object.content_hash())
}
```

## References

- [Content Addressing Architecture](../../architecture/core/content-addressing.md)
- [ADR-007: Content Addressing](../../../spec/adr_007_content_addressing.md)
- [ADR-028: Unified Hash Format](../../../spec/adr_028_universal_content_addressing.md)
- [ADR-029: SMT Integration](../../../spec/adr_029_smt_integration.md)
- [ADR-030: Deferred Hashing](../../../spec/adr_030_deffered_hashing_out_of_vm.md)
- [System Specification: Content Addressing System](../../../spec/spec.md#1-content-addressing-system-adr-007-adr-028-adr-029-adr-030) 