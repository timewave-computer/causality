# On-Chain Commitments with Merkle Trees

This document outlines our approach to handling on-chain commitments in the Causality system. Instead of posting individual object hashes to the blockchain, we leverage Merkle trees (specifically Sparse Merkle Trees) to create efficient and verifiable commitments.

## Overview

When dealing with blockchain systems, posting every piece of data or even individual hashes on-chain can be prohibitively expensive and inefficient. Instead, we use Merkle trees to create a single commitment that represents a collection of objects. This approach:

1. Reduces on-chain storage and transaction costs
2. Enables efficient verification through Merkle proofs
3. Maintains cryptographic integrity and security

## Sparse Merkle Trees (SMT)

We've chosen to use Sparse Merkle Trees (SMT) as our primary data structure for commitments because:

- They're optimized for sparse data sets (where most potential positions are empty)
- They allow for efficient proofs of inclusion and non-inclusion
- They're well-suited for ZK-friendly applications when using appropriate hash functions

## Implementation Approach

Our implementation in `src/commitment/mod.rs` provides:

1. A flexible trait-based interface (`CommitmentScheme`) that abstracts the details of different commitment methods
2. A concrete `MerkleTreeCommitmentScheme` implementation that uses a Sparse Merkle Tree
3. A `CommitmentFactory` that allows creating different types of commitment schemes

### Current Status

The implementation currently includes:

- A complete interface design for commitment schemes
- A working hash-based implementation for individual commitments
- A placeholder for the Merkle tree-based implementation

The exact SMT implementation is still being decided, but the interface is designed to accommodate various implementations without affecting the rest of the system.

## Using On-Chain Commitments

To use Merkle tree-based commitments in your code:

```rust
use causality::commitment::{CommitmentFactory, CommitmentType};

// Create a factory with Merkle tree support
let factory = CommitmentFactory::with_merkle_tree_support();

// Create a commitment scheme
let scheme = factory.create_scheme(CommitmentType::MerkleTree).unwrap();

// Add items to the tree (future API)
// scheme.add_item(key, value);

// Get the root commitment
let commitment = scheme.commit(data).unwrap();

// Generate and verify proofs (future API)
// let proof = scheme.generate_proof(key);
// let valid = scheme.verify_proof(key, value, &proof, &commitment);
```

## Future Work

1. **Complete SMT Implementation**: Finalize the SMT implementation with full proof generation and verification
2. **Integration with Valence Coprocessor**: Ensure compatibility with the Valence ZK-friendly hash functions
3. **Batched Operations**: Support for efficient batching of operations
4. **On-Chain Verification Contracts**: Templates for verifying Merkle proofs on different blockchain platforms

## Security Considerations

1. **Hash Function Choice**: The security of the Merkle tree depends on the underlying hash function
2. **Tree Depth**: The depth of the tree affects both security and performance
3. **Proof Verification**: All proofs should be verified before accepting any claims

By using Merkle tree-based commitments, we can maintain the security and verifiability of on-chain data while significantly reducing costs and improving efficiency. 