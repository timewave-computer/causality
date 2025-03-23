# ADR-030: Poseidon-Based Content Addressing with Deferred Hashing

## Status

Proposed

## Context

Our architecture has adopted content addressing as a universal principle for all stateful objects to provide immutability, verification, and composition properties. However, we've identified a significant performance concern: computing cryptographic hashes inside a RISC-V ZK-VM is extremely expensive in terms of proof generation time and circuit complexity.

When implementing our SMT (Sparse Merkle Tree) integration with the Valence protocol, we need an approach that maintains the benefits of content addressing while addressing the performance challenges of ZK proof generation.

## Decision

We will:

1. **Switch completely from Blake3 to Poseidon** for all content addressing
2. **Defer hash computation** until after zkVM computation is complete
3. **Prove properties about the SMT outside of the zkVM** using specialized circuits
4. **Use commitment-based verification** inside the zkVM

This approach ensures ZK-friendly hashing throughout the system while minimizing the performance overhead of in-circuit hash computation.

## Detailed Approach

### 1. Poseidon as Universal Hash Function

We will use Poseidon as our standard hash function for all content addressing:

```rust
/// Content hash using Poseidon
pub struct ContentHash([u8; 32]);

impl ContentAddressed for T {
    fn content_hash(&self) -> ContentHash {
        // Serialize object
        let serialized = serialize_canonical(self);
        
        // Apply Poseidon hash OUTSIDE the zkVM
        let hash = poseidon_hash(&serialized);
        
        ContentHash(hash)
    }
}
```

Key properties of Poseidon:
- ZK-friendly design with minimal constraints
- Sufficient security for content addressing
- Optimized for efficient verification in zkSNARK circuits
- Significantly faster inside ZK circuits than general-purpose hashes

### 2. Deferred Hashing Architecture

Instead of computing hashes inside the zkVM, we'll defer hash computation:

```rust
/// Execution context for the zkVM
pub struct ZkVmExecutionContext {
    // Content to be hashed after execution
    deferred_hash_inputs: Vec<DeferredHashInput>,
    // Hash outputs (filled after execution)
    hash_outputs: HashMap<DeferredHashId, ContentHash>,
}

/// Input for deferred hashing
pub struct DeferredHashInput {
    id: DeferredHashId,
    data: Vec<u8>,
}

impl ZkVmExecutionContext {
    /// Request a hash computation (creates a placeholder)
    pub fn request_hash(&mut self, data: &[u8]) -> DeferredHashId {
        let id = DeferredHashId::new();
        self.deferred_hash_inputs.push(DeferredHashInput {
            id: id.clone(),
            data: data.to_vec(),
        });
        id
    }
    
    /// Perform all deferred hash computations after VM execution
    pub fn compute_deferred_hashes(&mut self) {
        for input in &self.deferred_hash_inputs {
            let hash = poseidon_hash(&input.data);
            self.hash_outputs.insert(input.id.clone(), ContentHash(hash));
        }
    }
}
```

This pattern:
- Creates placeholders during zkVM execution
- Computes actual hashes after VM execution completes
- Ensures deterministic results without in-circuit hashing

### 3. External SMT Proof Generation

The SMT proof system will operate outside the zkVM:

```rust
/// SMT proof generator
pub struct SmtProofGenerator {
    smt: Arc<dyn SmtBackend>,
    poseidon_params: PoseidonParams,
}

impl SmtProofGenerator {
    /// Generate SMT inclusion proof
    pub fn generate_inclusion_proof(
        &self,
        root: &Root,
        key: &[u8],
        value: &[u8],
    ) -> Result<SmtProof, ProofError> {
        // Generate proof using real SMT outside zkVM
        self.smt.get_opening("causality_object", *root, key)
            .map_err(|e| ProofError::SmtError(e))
    }
    
    /// Generate specialized circuit for SMT verification
    pub fn generate_verification_circuit(
        &self,
        proof: &SmtProof,
    ) -> Result<ZkCircuit, CircuitError> {
        // Create specialized Poseidon-based circuit
        // Much more efficient than general RISC-V execution
        SmtVerificationCircuit::new(proof, self.poseidon_params)
            .map_err(|e| CircuitError::CircuitGenerationError(e))
    }
}
```

This approach:
- Keeps SMT proof generation separate from general zkVM execution
- Uses specialized circuits optimized for SMT verification
- Takes advantage of Poseidon's efficiency in ZK settings

### 4. Commitment-Based Verification in zkVM

Inside the zkVM, we'll use commitment-based verification rather than recomputing hashes:

```rust
/// Inside zkVM code
pub fn verify_object(
    commitment: &Commitment,
    expected_commitment: &Commitment,
) -> bool {
    // Simple equality check instead of hash recomputation
    commitment == expected_commitment
}

/// Verify SMT proof inside zkVM
pub fn verify_smt_inclusion(
    root: &Root,
    key: &[u8],
    value_commitment: &Commitment,
    proof: &SmtProof,
) -> bool {
    // Verify the SMT proof using optimized circuit
    // Much cheaper than recomputing hashes
    verify_poseidon_merkle_proof(root, key, value_commitment, proof)
}
```

This pattern:
- Verifies commitments instead of recomputing hashes
- Uses specialized verification circuits optimized for Poseidon
- Minimizes in-circuit operations

## Architecture Diagram

```
┌─────────────────────────────┐      ┌───────────────────────────┐
│   Content-Addressed World   │      │      SMT Proof System     │
│   (Poseidon outside zkVM)   │      │                           │
│                             │      │   • Poseidon-based SMT    │
│   • Full objects            │      │   • Specialized circuits  │
│   • Content hashes          │  ──► │   • Proof generation      │
│   • Deferred hash requests  │      │                           │
│                             │      │                           │
└─────────────────────────────┘      └───────────┬───────────────┘
                                                  │
                                                  ▼
┌─────────────────────────────┐      ┌───────────────────────────┐
│      On-Chain World         │      │      ZK-VM World          │
│                             │      │                           │
│   • Merkle roots            │      │   • Commitment            │
│   • Nullifiers              │ ◄─── │   • Verification          │
│   • Verification results    │      │   • No hash computation!  │
│                             │      │                           │
└─────────────────────────────┘      └───────────────────────────┘
```

## ZK Circuit Optimization

The specialized SMT verification circuits will be much more efficient than general-purpose RISC-V circuits:

1. **Minimal in-circuit operations**: Only verify what's necessary
2. **Optimized for Poseidon**: Take advantage of Poseidon's ZK-friendly design
3. **Batch verification**: Verify multiple SMT proofs efficiently
4. **Custom constraint systems**: Tailored specifically for our SMT structure

## Integration with Content Addressing

This approach seamlessly integrates with our content addressing architecture:

1. **All objects are still content-addressed**: Using Poseidon instead of Blake3
2. **Full verification properties remain**: Immutability, composition, verification
3. **Efficient ZK implementation**: Avoiding expensive in-circuit hashing
4. **SMT integration preserved**: With optimized proof generation

## Concrete Example: Resource Operation

Here's how a typical resource operation would work:

```rust
// 1. Create a content-addressed resource
let resource = Resource::new(/* ... */);

// 2. Hash the resource OUTSIDE the zkVM
let resource_hash = resource.content_hash(); // Uses Poseidon

// 3. Insert into SMT (also outside zkVM)
let smt = SmtBackend::default();
let root = smt.insert(old_root, "resource", resource.to_bytes())?;

// 4. Generate SMT proof
let proof = smt.get_opening("resource", root, resource_hash.as_bytes())?;

// 5. Create specialized verification circuit
let circuit = SmtProofGenerator::generate_verification_circuit(&proof)?;

// 6. Inside zkVM, only verify commitment
// (This is the code that runs inside the zkVM)
fn verify_inside_zkvm(root: &Root, commitment: &Commitment, proof: &SmtProof) -> bool {
    // No hash computation, just verification
    verify_poseidon_merkle_proof(root, commitment, proof)
}
```

## Consequences

### Benefits

1. **ZK-Friendly Throughout**: Poseidon is optimized for ZK circuits
2. **Efficient Proof Generation**: Avoid expensive hashing inside zkVM
3. **Specialized SMT Circuits**: Much more efficient than general RISC-V
4. **Preserved Content Addressing**: All architectural benefits maintained
5. **Deferred Computation**: Move expensive operations outside critical path
6. **Minimal On-Chain Footprint**: Still only store roots and verification data on-chain

### Challenges

1. **Transition Cost**: Migration from Blake3 to Poseidon across the codebase
2. **Implementation Complexity**: Deferred hashing requires careful ordering
3. **Specialized Knowledge**: Requires ZK circuit optimization expertise
4. **Consistency Management**: Ensuring consistent hash computation
5. **Testing Requirements**: Specialized testing for deferred hashing

### Mitigations

1. **Clear Boundaries**: Define clear interfaces between systems
2. **Deterministic Wrappers**: Ensure deterministic behavior despite deferred computation
3. **Comprehensive Testing**: Verify hashing correctness and consistency
4. **Performance Monitoring**: Track and optimize ZK proof generation time

## Implementation Plan

1. **Phase 1: Hash Function Transition** (4 weeks)
   - Replace Blake3 with Poseidon across codebase
   - Implement deferred hashing architecture
   - Update content addressing system

2. **Phase 2: SMT Proof System** (3 weeks)
   - Implement Poseidon-based SMT
   - Create specialized verification circuits
   - Integrate with Valence SMT

3. **Phase 3: ZK-VM Integration** (3 weeks)
   - Implement commitment verification inside zkVM
   - Create external proof generation system
   - Connect verification results to on-chain contracts

4. **Phase 4: Performance Optimization** (2 weeks)
   - Optimize critical paths in proof generation
   - Benchmark and tune system
   - Document performance recommendations

## References

1. Valence SMT Implementation
2. ADR-019: Universal Content-Addressing for System State
3. ADR-020: Content-Addressing with Sparse Merkle Tree Integration

## Conclusion

By adopting Poseidon as our universal hash function, deferring hash computation, and proving properties about the SMT outside the zkVM, we create a system that maintains the benefits of content addressing while addressing the performance challenges of ZK proof generation. This approach creates a more efficient and scalable architecture for our content-addressed, zero-knowledge system.