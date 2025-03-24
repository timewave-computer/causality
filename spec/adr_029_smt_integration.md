# ADR-029: Content-Addressing with Sparse Merkle Tree Integration

## Status

Accepted

## Implementation Status

This ADR has been fully implemented. The Sparse Merkle Tree (SMT) integration is now a standard component in the Causality storage architecture. Key implementation components include:

- ContentAddressedStorage implementation backed by SMT
- Integration with the unified ResourceRegister model
- Support for efficient state proofs and verification
- Implementation of domain-specific storage strategies
- Cross-domain state verification with SMT proofs
- Optimization for ZK-circuit compatibility

The implementation provides efficient, verifiable storage that maintains content-addressing benefits while supporting diverse blockchain environments. Documentation is available in [docs/src/storage_models.md](/docs/src/storage_models.md) and [docs/src/storage_architecture.md](/docs/src/storage_architecture.md).

## Context

Following our decision to adopt content addressing as a universal principle for all stateful objects (ADR-019), we need to determine how this impacts our on-chain footprint. While content addressing provides powerful verification and composition properties, storing all content hashes on-chain would be prohibitively expensive and inefficient.

The Valence protocol team has been developing a Sparse Merkle Tree (SMT) implementation that aligns well with our goals. This ADR explores the integration of our content-addressing efforts with the Valence SMT to minimize on-chain storage while maximizing verification capabilities.

## Decision

We will implement a unified architecture where:

1. All Causality objects are content-addressed for off-chain organization and verification
2. Only Merkle roots and minimal verification data are stored on-chain
3. The Valence SMT will serve as the bridge between our content-addressed system and on-chain representation

This approach maintains all the benefits of content addressing while drastically reducing our on-chain footprint.

## Detailed Approach

### On-Chain / Off-Chain Division

```
┌───────────────────────────┐      ┌─────────────────────────┐
│  Content-Addressed World  │      │     On-Chain World      │
│  (Off-Chain)              │      │                         │
│                           │      │                         │
│  • Full objects           │      │  • Merkle roots         │
│  • Complete history       │ ───► │  • Nullifiers           │
│  • Effect definitions     │      │  • Commitments          │
│  • Resource definitions   │      │  • Verification results │
│  • ZK proofs              │      │                         │
│                           │      │                         │
└───────────────────────────┘      └─────────────────────────┘
```

### SMT Integration

The Valence SMT provides several features that align perfectly with our content-addressing model:

1. **Efficient Proofs**: Generates compact inclusion proofs for verification
2. **Flexible Backend**: Separates tree logic from data persistence
3. **Execution Context**: Provides necessary cryptographic primitives
4. **ZK Compatibility**: Designed for integration with ZK execution environments

### Content-Addressing Pattern

Each content-addressed object will:
1. Be identified by its content hash off-chain
2. Be included in the SMT with its content hash as a key
3. Be verifiable on-chain through SMT inclusion proofs

### Only Store What's Necessary On-Chain

The only data that will be stored on-chain are:
1. **State Root**: The current Merkle root of the SMT
2. **Nullifiers**: Markers for spent/consumed resources
3. **Verification Results**: Results of ZK proof verification
4. **Minimal Metadata**: Any essential data needed for on-chain operation

## Implementation

### Integration Components

1. **ContentAddressedStorage Implementation**:
```rust
struct SmtContentStore {
    smt: Arc<dyn SmtBackend>,
    root: Root,
}

impl ContentAddressedStorage for SmtContentStore {
    fn store<T: ContentAddressed>(&self, object: &T) -> Result<ContentHash, StorageError> {
        let hash = object.content_hash();
        let data = object.to_bytes();
        
        // Use content hash as key in SMT
        self.smt.insert(self.root, "causality_object", data)?;
        
        Ok(hash)
    }
    
    fn get<T: ContentAddressed>(&self, hash: &ContentHash) -> Result<T, StorageError> {
        // Generate proof of inclusion
        let proof = self.smt.get_opening(
            "causality_object", 
            self.root, 
            hash.as_bytes()
        )?.ok_or(StorageError::NotFound)?;
        
        // Reconstruct object from proof data
        T::from_bytes(&proof.data)
    }
}
```

2. **On-Chain Verification Contract**:
```solidity
contract ValenceCausalityVerifier {
    // Current state root
    bytes32 public stateRoot;
    
    // Nullifier set
    mapping(bytes32 => bool) public nullifiers;
    
    // Verify inclusion proof
    function verifyInclusion(
        bytes32 key,
        bytes calldata proof
    ) public view returns (bool) {
        return Smt.verify("causality_object", stateRoot, key, proof);
    }
    
    // Update state root (with appropriate authorization)
    function updateStateRoot(bytes32 newRoot) external onlyAuthorized {
        stateRoot = newRoot;
    }
    
    // Mark nullifier as spent
    function markNullifier(bytes32 nullifier) external onlyAuthorized {
        require(!nullifiers[nullifier], "Nullifier already spent");
        nullifiers[nullifier] = true;
    }
}
```

3. **ZK Circuit Integration**:
```rust
// Generate ZK proof of content-addressed object inclusion
fn generate_inclusion_proof<T: ContentAddressed>(
    object: &T,
    smt: &SmtContentStore,
    execution_context: &ExecutionContext
) -> Result<ZkProof, ProofError> {
    // Get object hash
    let hash = object.content_hash();
    
    // Get SMT inclusion proof
    let inclusion_proof = smt.get_opening(
        "causality_object",
        smt.root(),
        hash.as_bytes()
    )?.ok_or(ProofError::ObjectNotFound)?;
    
    // Generate ZK proof that:
    // 1. Object exists in SMT with hash as key
    // 2. Object has specific properties we want to prove
    execution_context.prove_circuit(
        "object_inclusion",
        [
            ("root", smt.root().to_bytes()),
            ("key", hash.as_bytes()),
            ("proof", inclusion_proof.to_bytes()),
            ("object_property", extract_public_property(object)?)
        ]
    )
}
```

## Consequences

### Benefits

1. **Minimal On-Chain Footprint**: Only roots and verification data on-chain
2. **Verifiable Off-Chain State**: All off-chain data is verifiable through SMT proofs
3. **Efficient Cross-Domain Operations**: Only proofs need to be provided for verification
4. **ZK-Friendly Design**: Perfect alignment with ZK proof generation
5. **Scalable Architecture**: Can handle millions of content-addressed objects with minimal on-chain cost

### Challenges

1. **Proof Generation Overhead**: Generating SMT proofs adds computational overhead
2. **Implementation Complexity**: Integrating content addressing with SMT adds complexity
3. **Synchronization Requirements**: Need to ensure SMT state is synchronized across nodes

### Important Considerations

1. **Data Availability**: Content-addressed objects must be available somewhere in the network
2. **Root Management**: State root updates must be carefully managed
3. **Nullifier Design**: Need efficient nullifier scheme for spent resources
4. **Performance Tuning**: SMT operations need optimization for high throughput

## Comparison to Alternatives

1. **Direct On-Chain Storage**:
   - **Pros**: Simplest implementation, direct chain verification
   - **Cons**: Prohibitively expensive for large state, limited scalability
   - **Our Decision**: Rejected due to cost and scalability concerns

2. **Hash-Only On-Chain**:
   - **Pros**: Reduced footprint, maintains verification
   - **Cons**: Still significant cost for many objects, no aggregation benefit
   - **Our Decision**: Rejected in favor of more efficient SMT approach

3. **General Merkle Trees**:
   - **Pros**: Efficient for some workloads, single root verification
   - **Cons**: Less efficient for sparse data, more complex proof generation
   - **Our Decision**: Rejected in favor of Sparse Merkle Tree specifically designed for our use case

## Implementation Roadmap

1. **Phase 1: Core SMT Integration**
   - Implement ContentAddressedStorage backed by Valence SMT
   - Define mapping from content hashes to SMT keys
   - Create basic proof generation and verification

2. **Phase 2: On-Chain Components**
   - Develop on-chain contracts for root management and verification
   - Implement nullifier tracking
   - Create ZK circuits for state transition verification

3. **Phase 3: Performance Optimization**
   - Optimize proof generation
   - Implement caching strategies
   - Tune SMT parameters for our workload

4. **Phase 4: Cross-Domain Verification**
   - Extend to multiple chains
   - Implement cross-chain root verification
   - Develop cross-domain nullifier protocol

## References

1. ADR-019: Universal Content-Addressing for System State
2. Valence SMT Implementation Documentation
3. ADR-007: Content-addressable Code and Execution in Rust

## Conclusion

By integrating our content-addressing efforts with the Valence SMT implementation, we create a system that achieves both the verification benefits of content addressing and the efficiency of minimal on-chain storage. This approach aligns perfectly with our cross-domain focus and ZK-based verification strategy.

This architecture gives us the best of both worlds: the rich, deterministic verification properties of content addressing with the scalability and efficiency of a minimal on-chain footprint.