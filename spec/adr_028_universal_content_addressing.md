# ADR-028: Universal Content-Addressing for System State

## Status

Accepted

## Implementation Status

This ADR has been fully implemented. Universal content addressing is now the standard approach for all state in Causality. Key implementation components include:

- ContentAddressed trait implementation across all state objects
- ContentReference type for safe cross-domain references
- ContentAddressedStorage interfaces for different storage backends
- Integration with the cryptographic subsystem
- Support for deferred hashing for performance optimization
- Comprehensive verification mechanisms for content integrity

The implementation provides a consistent, tamper-evident foundation for all system state, enabling efficient verification and cross-domain references. Documentation is available in [docs/src/content_addressing.md](/docs/src/content_addressing.md) and [docs/src/content_addressing_guide.md](/docs/src/content_addressing_guide.md).

## Context

Causality currently implements content-addressing for code through the content-addressable code system described in ADR-007. This approach has demonstrated significant benefits in immutability, verifiability, and composability for code definitions. However, the current implementation is limited to code, while many other stateful objects in the system could benefit from the same properties.

We have the opportunity to extend content-addressing as a universal architectural principle across all stateful objects in the system. This would transform Causality into a fully content-addressed system with powerful verification, caching, and distribution properties.

## Decision

We will adopt content-addressing as a universal principle for all stateful objects in Causality. This means that:

1. Every stateful object will have a unique content hash derived from its data.
2. Objects will be referenced by their content hash rather than by identity or location.
3. Content hashes will be calculated using a consistent, secure hashing algorithm.
4. Objects will be immutable - any change creates a new object with a new hash.
5. Storage and retrieval systems will be content-addressed.

## Objects to Be Content-Addressed

The following objects will be converted to a content-addressed representation:

### Core Execution Objects
1. **Effects** - Operations that cause state transitions
2. **Effect Handlers** - Components that process specific effect types
3. **Fact Observations** - External state observations from Domains
4. **Fact Snapshots** - Collections of facts observed at a point in time
5. **Time Map Snapshots** - Observations of Domain states at specific times
6. **Execution Context** - State of a running program
7. **Execution Sequence** - Ordered sequence of execution steps
8. **Execution Snapshots** - Checkpoints of execution state

### Resource System Objects
9. **Resources** - Logical representations of assets and capabilities
10. **Resource Definitions** - Type definitions for resources
11. **Resource Deltas** - Changes to resource quantities
12. **Resource Transformations** - Operations that convert resources
13. **Controller Labels** - Provenance tracking for resources
14. **Resource Conservation Proofs** - Proofs that resources are conserved

### Register System Objects
15. **Registers** - On-chain storage units
16. **Register Operations** - Create, update, transfer operations
17. **Register Contents** - Data stored in registers
18. **Nullifiers** - Markers for spent registers
19. **Register Witnesses** - Private data for ZK proofs
20. **Register Commitments** - Cryptographic commitments to register state

### Program Objects
21. **Program Memory** - Program state and variables
22. **Program Safe States** - States where upgrades are permitted
23. **Program Schema** - Structure of program state
24. **Program Evolution Rules** - Rules for schema evolution
25. **Program Capabilities** - Rights granted to a program

### Domain Objects
26. **Domain Adapters** - Interfaces to external Domains
27. **Domain Descriptors** - Metadata about Domains
28. **Adapter Schemas** - Specifications for generating adapters
29. **Fact Rules** - Rules for extracting facts from Domains
30. **Boundary Crossings** - Messages between system boundaries

### ZK Proof System Objects
31. **ZK Circuits** - Circuit definitions for ZK proofs
32. **ZK Proofs** - Generated zero-knowledge proofs
33. **Verification Keys** - Keys for verifying ZK proofs
34. **Public Inputs** - Public inputs to ZK circuits
35. **Circuit Constraints** - Constraints defining circuit behavior

### Log System Objects
36. **Log Entries** - Individual entries in the unified log
37. **Log Segments** - Groups of log entries
38. **Fact Logs** - Logs of observed facts
39. **Effect Logs** - Logs of applied effects
40. **Event Logs** - Logs of system events

### User-Facing Objects
41. **Program Account Interfaces** - User-visible interfaces
42. **Program Account Policies** - Rules for account behavior
43. **Available Actions** - Actions a user can take
44. **Resource Presentations** - User-visible resource representations
45. **Activity Records** - Records of user activity

### Temporal Effect Language (TEL) Objects
46. **TEL Programs** - Programs written in TEL
47. **TEL Expressions** - Individual expressions in TEL
48. **TEL Effect Expressions** - Effect-specific expressions
49. **TEL Time Expressions** - Time-related expressions
50. **TEL Resource Expressions** - Resource-related expressions

### Dual Validation Objects
51. **Dual Validation Proofs** - Combined temporal and ancestral proofs
52. **Temporal Validation Components** - Parts of temporal validation
53. **Ancestral Validation Components** - Parts of ancestral validation
54. **Validation Rules** - Rules for validation
55. **Validation Results** - Results of validation checks

## Implementation Approach

The implementation will follow these steps:

1. **Define Universal Traits**: Create a `ContentAddressed` trait that all stateful objects will implement.

```rust
pub trait ContentAddressed {
    /// Get the content hash of this object
    fn content_hash(&self) -> ContentHash;
    
    /// Verify that the object matches its hash
    fn verify(&self) -> bool;
    
    /// Convert to a serialized form for storage
    fn to_bytes(&self) -> Vec<u8>;
    
    /// Create from serialized form
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> where Self: Sized;
}
```

2. **Create Content-Addressed Storage Interface**: Implement a unified storage system for content-addressed objects.

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
```

3. **Prioritize Critical Components**: Implement content-addressing for high-impact components first:
   - Effects and Effect Handlers
   - Resources and Resource Transformations
   - Register Operations
   - Fact Observations
   - Time Map Snapshots

4. **Adapt Existing Systems**: Modify existing systems to work with content-addressed objects:
   - Update the effect pipeline to process content-addressed effects
   - Modify the resource system to handle content-addressed resources
   - Adapt the register system for content-addressed operations
   - Update the log system to store content-addressed entries

5. **System Evolution**: Extend content-addressing to remaining objects over time, with the goal of achieving full coverage.

## Implementation Details

### Content Hash Calculation

```rust
pub fn calculate_content_hash<T: Serialize>(object: &T) -> Result<ContentHash, HashError> {
    // Consistently serialize the object
    let serialized = serialize_canonical(object)?;
    
    // Apply the hash function
    let hash = Blake3::hash(&serialized);
    
    // Return the content hash
    Ok(ContentHash::from_bytes(hash.as_bytes()))
}
```

### Content Normalization

To ensure consistent hashing, we'll implement canonical serialization:

```rust
pub fn serialize_canonical<T: Serialize>(object: &T) -> Result<Vec<u8>, SerializationError> {
    // Use a deterministic serialization format
    let mut serializer = CanonicalSerializer::new();
    
    // Serialize the object
    object.serialize(&mut serializer)?;
    
    // Return the serialized bytes
    Ok(serializer.into_bytes())
}
```

### Content-Addressed References

To reference content-addressed objects:

```rust
pub struct ContentRef<T> {
    /// The content hash
    pub hash: ContentHash,
    /// Phantom type to indicate what this references
    phantom: PhantomData<T>,
}

impl<T: ContentAddressed> ContentRef<T> {
    /// Create a new content reference
    pub fn new(object: &T) -> Self {
        Self {
            hash: object.content_hash(),
            phantom: PhantomData,
        }
    }
    
    /// Resolve this reference to an object
    pub fn resolve(&self, storage: &impl ContentAddressedStorage) -> Result<T, StorageError> {
        storage.get(&self.hash)
    }
}
```

## Benefits and Risks

### Benefits

1. **Intrinsic Verification**: Objects are verified by their content hash, eliminating many classes of verification logic.
2. **Guaranteed Immutability**: Once created, objects cannot be modified without changing their hash.
3. **Perfect Caching**: Identical operations always produce identical results, enabling perfect caching.
4. **Simplified Distribution**: Objects can be fetched from any source that has them, verified by hash.
5. **Algebraic Composition**: Operations compose based on their content hashes, creating a clean algebraic structure.
6. **Cryptographic Auditability**: System behavior can be cryptographically verified from inputs to outputs.
7. **Minimized Trust Requirements**: Components need only trust the content verification mechanism, not each other.
8. **Natural De-duplication**: Identical objects are automatically unified by their content hashes.
9. **Simplified Cross-Domain Operations**: Cross-domain operations become cryptographically verifiable without requiring trust between domains.
10. **Enhanced ZK Integration**: ZK proofs operate naturally on clearly defined, content-addressed inputs and outputs.

### Risks

1. **Performance Overhead**: Content addressing adds some overhead for hash calculation and verification.
2. **Complexity**: The system becomes more complex initially as we transition to content-addressing.
3. **Storage Growth**: Immutable objects may increase storage requirements compared to mutable ones.
4. **Implementation Consistency**: Must ensure consistent hash calculation across all components.
5. **Hash Collision Risks**: Must use a hash function with sufficient collision resistance.

## Mitigation Strategies

1. **Gradual Implementation**: Implement content-addressing incrementally, starting with high-value components.
2. **Caching Optimization**: Implement aggressive caching of content hashes to minimize recalculation.
3. **Storage Optimization**: Use structural sharing and garbage collection for efficient storage.
4. **Consistent Libraries**: Develop shared libraries for content addressing to ensure consistency.
5. **Modern Hash Functions**: Use modern hash functions (Blake3) with extremely low collision probability.
6. **Verification Tooling**: Build tools to verify content-addressed properties across the system.

## Alternate Approaches Considered

1. **Partial Content-Addressing**: Only applying content-addressing to selected high-value components.
   - **Pros**: Simpler implementation, lower migration effort.
   - **Cons**: Loses system-wide verification properties, creates inconsistent architectural patterns.

2. **Identity-Based + Content-Verification**: Keeping identity-based addressing but adding content verification.
   - **Pros**: Maintains compatibility with existing patterns, adds some verification.
   - **Cons**: More complex than pure content-addressing, loses algebraic composition properties.

3. **Hybrid Mutable/Immutable Model**: Content-addressing for immutable objects, traditional addressing for mutable state.
   - **Pros**: Potentially better performance for frequently changing state.
   - **Cons**: Introduces two parallel systems with different semantics, raises complexity.

## Conclusion

Adopting universal content-addressing for all stateful objects in Causality represents a transformative architectural decision with far-reaching benefits. By making this change, we will create a system where correctness is cryptographically verifiable at every level, from individual operations to entire execution histories.

The migration will be challenging but manageable, especially when approached incrementally. The resulting architecture will provide powerful verification, caching, and distribution properties that significantly enhance the system's security, performance, and correctness guarantees.

This change aligns perfectly with Causality's cross-domain nature, where verifiable operations across trust boundaries are essential. It also complements our existing approaches to zero-knowledge proofs, resource conservation, and dual validation by providing a consistent verification framework across all components.

## References

1. ADR-007: Content-addressable Code and Execution in Rust
2. ADR-002: Effect Adapters
3. ADR-006: ZK-Based Register System