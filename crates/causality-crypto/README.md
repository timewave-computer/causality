# Causality Crypto

This crate provides the cryptographic primitives and implementations used within the Causality system.

## Purpose

Consolidates cryptographic operations, ensuring consistent and secure implementations across the Causality ecosystem. This crate serves as the central authority for all cryptographic operations, which helps maintain security and consistency.

## Core Functionality

- **Hashing**: Implementation of cryptographic hash functions (e.g., Blake3) and content addressable storage primitives.
  - `hash.rs`: Core hashing implementations and traits
  - `deferred.rs`: Deferred/batched hashing for performance optimization

- **Signatures**: Digital signature generation and verification.
  - `signatures.rs`: Core signature abstractions
  - `signature.rs`: Concrete signature implementations

- **Proof Systems**: Generation and verification of various types of cryptographic proofs.
  - `proof.rs`: Complex proof structures (unified proofs, ZK proofs, etc.)
  - `proofs.rs`: Abstractions for creating and verifying proofs
  - `zk.rs`: Zero-knowledge proof utilities

- **Merkle Trees**: Implementations for Merkle trees and related data structures.
  - `merkle.rs`: Merkle tree implementations
  - `sparse_merkle_tree.rs`: Sparse Merkle tree implementation

- **Nullifiers**: Implementation of nullifiers for privacy-preserving operations.
  - `nullifier.rs`: Nullifier generation and verification

## Architecture

The crate follows these architectural principles:

1. **Trait-based Design**: Core functionality is expressed through traits that define capabilities (e.g., `HashFunction`, `Signer`, `Prover`)
2. **Implementation Agnostic**: The abstractions allow for multiple backend implementations
3. **Type Safety**: Heavy use of generics and type parameters to ensure type safety
4. **Content Addressing**: All cryptographic objects can be content-addressed for integrity verification

## Usage Examples

```rust
// Hashing a value
use causality_crypto::hash::{HashFunction, Blake3Hasher};

let hasher = Blake3Hasher::new();
let hash = hasher.hash("Hello, world!".as_bytes());
println!("Hash: {}", hash);

// Creating and verifying a proof
use causality_crypto::proofs::{Prover, ProofVerifier};
// ... implementation details ...
```

## Dependencies

This crate depends on `causality-types` for core cryptographic data structures like `HashOutput`, `ContentHash`, etc., but provides the actual computational logic.

## Feature Flags

The crate provides optional features for different cryptographic algorithms:

- `poseidon`: Enable Poseidon hash function
- `ed25519`: Enable Ed25519 signature scheme
- `ecdsa`: Enable ECDSA signature scheme
- `bls`: Enable BLS signature scheme
- `md5`: Enable MD5 checksum (non-cryptographic, for compatibility)
- `groth16`: Enable Groth16 zero-knowledge proofs
- `plonk`: Enable PLONK zero-knowledge proofs

## Security Considerations

This crate implements cryptographic primitives that are critical for the security of the entire Causality system. Any changes to cryptographic implementations must be carefully reviewed and tested.

---

Refer to the main project [README.md](../../README.md) and [spec.md](../../spec/spec.md) for broader architectural context. 