<!-- Interfaces for database hash SMTs -->
<!-- Original file: docs/src/db_hash_smt_interfaces.md -->

# Database, Hash, Commitment, and SMT Interfaces

This document describes the core interfaces and implementations for the database, hash function, commitment scheme, and Sparse Merkle Tree (SMT) components of the Causality system.

## Overview

The system is designed with clean trait abstractions to allow for multiple implementations and easy interoperability with external systems like the Valence coprocessor project. The key components are:

1. **Hash Interface**: Provides traits for cryptographic hash functions with initial support for Blake3 and a placeholder for Poseidon
2. **Commitment Interface**: Builds on the hash interface to provide commitment schemes 
3. **Database Interface**: Defines a common interface for key-value storage with implementations for in-memory (testing) and RocksDB
4. **Sparse Merkle Tree (SMT)**: Implements an efficient data structure for Merkle proofs using the other interfaces

## Hash Interface

The hash interface (`src/hash/mod.rs`) provides a unified approach to cryptographic hashing operations:

### Core Components

* `HashOutput`: Fixed-size (32-byte) hash result with serialization/deserialization support
* `HashFunction`: Trait defining the common operations for hash functions
* `Hasher`: Trait for incremental (streaming) hash operations
* `HashFactory`: Factory for creating hash function instances based on algorithm type

### Implementations

* **Blake3**: Default implementation using the Blake3 cryptographic hash function
* **Poseidon**: Placeholder for future integration with the Valence coprocessor's ZK-friendly Poseidon implementation

## Commitment Interface

The commitment interface (`src/commitment/mod.rs`) builds on the hash interface to provide cryptographic commitments:

### Core Components

* `Commitment`: Data structure representing a binding to a piece of data
* `CommitmentScheme`: Trait defining operations for creating and verifying commitments
* `CommitmentFactory`: Factory for creating commitment scheme instances

### Implementations

* **Hash-based Commitments**: Default implementation using cryptographic hashes
* **Merkle Tree**: (Planned) Implementation using Sparse Merkle Trees

## Database Interface

The database interface (`src/db/mod.rs`) provides a unified interface for key-value storage:

### Core Components

* `Database`: Trait defining CRUD operations, batch operations, and iterators
* `DbIterator`: Trait for iterating over database entries
* `DbConfig`: Configuration options for database instances
* `DbFactory`: Factory for creating database instances

### Implementations

* **MemoryDb**: In-memory implementation for testing and small datasets
* **RocksDb**: Persistent storage implementation using RocksDB (enabled with `rocksdb` feature)

## Sparse Merkle Tree (SMT)

The SMT implementation (`src/smt/mod.rs`) provides an efficient data structure for Merkle proofs:

### Core Components

* `SparseMerkleTree`: Trait defining operations for a Sparse Merkle Tree
* `Key`: Data structure for SMT keys (32-byte fixed-size)
* `Value`: Data structure for SMT values (arbitrary-size)
* `Node`: Tree node representation (internal, leaf, or empty)
* `Path`: Representation of a path through the tree
* `Proof`: Data structure for inclusion/non-inclusion proofs
* `StorageBackend`: Trait for SMT node storage
* `SmtFactory`: Factory for creating SMT instances

### Implementations

* **MemoryStorage**: In-memory storage backend for nodes
* **RocksDbStorage**: (Placeholder) Persistent storage using RocksDB

## Integration with Valence Coprocessor

The interfaces are designed to be compatible with the Valence coprocessor project:

1. **Hash Integration**: The Poseidon placeholder provides the interface structure to be filled with the actual Valence implementation in the future.

2. **Database Integration**: The database interface can be implemented by the Valence project to use its storage backend.

3. **SMT Integration**: The SMT interface allows the Valence project to provide its own implementation or use our implementation.

## Future Extensions

1. **Additional Hash Algorithms**: Support for other ZK-friendly hash algorithms beyond Poseidon
2. **Additional Database Backends**: Support for other database systems
3. **Performance Optimizations**: Optimized implementations for specific use cases
4. **Complete SMT Implementation**: Finish the SMT implementation with full proof generation and verification

## Usage Examples

### Using the Hash Interface

```rust
use causality::hash::{HashFactory, HashAlgorithm};

fn hash_example() {
    // Create a hash factory with default algorithm (Blake3)
    let factory = HashFactory::default();
    
    // Create a hasher
    let hasher = factory.create_hasher().unwrap();
    
    // Hash some data
    let data = b"Hello, world!";
    let hash = hasher.hash(data);
    
    println!("Hash: {}", hash);
}
```

### Using the Database Interface

```rust
use causality::db::{DbFactory, DbConfig, BatchOp};

fn database_example() {
    // Create an in-memory database
    let db = DbFactory::create_memory_db().unwrap();
    
    // Basic operations
    db.put(b"key1", b"value1").unwrap();
    let value = db.get(b"key1").unwrap().unwrap();
    
    // Batch operations
    let batch = vec![
        BatchOp::Put(b"key2".to_vec(), b"value2".to_vec()),
        BatchOp::Put(b"key3".to_vec(), b"value3".to_vec()),
        BatchOp::Delete(b"key1".to_vec()),
    ];
    db.write_batch(&batch).unwrap();
    
    // Iteration
    let mut iter = db.iterator().unwrap();
    while let Some(result) = iter.next() {
        let (key, value) = result.unwrap();
        println!("Key: {:?}, Value: {:?}", key, value);
    }
}
```

### Using the SMT Interface

```rust
use causality::smt::{SmtFactory, Key, Value};

fn smt_example() {
    // Create an SMT factory
    let factory = SmtFactory::default();
    
    // Create an in-memory SMT
    let mut smt = factory.create_memory_smt().unwrap();
    
    // Insert key-value pairs
    let key = Key::from_slice(b"test key");
    let value = Value::new(b"test value".to_vec());
    smt.insert(&key, &value).unwrap();
    
    // Generate and verify a proof
    let proof = smt.generate_proof(&key).unwrap();
    let result = smt.verify_proof(&key, Some(&value), &proof).unwrap();
    assert!(result);
} 