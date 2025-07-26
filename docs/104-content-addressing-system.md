# 104: Content Addressing System

Content addressing is the foundational identity and data management system that permeates every layer of Causality. Unlike traditional systems that rely on location-based addressing (like memory addresses or file paths), Causality identifies all data by the cryptographic hash of its canonical representation. This design choice has important implications for the system's architecture, enabling properties like immutability, verifiability, and global deduplication.

## Overview

The content addressing system in Causality is built on several key principles:

1. **Universal Identity**: Every piece of data has a globally unique, content-derived identifier
2. **Immutability**: Content-addressed data cannot be modified without changing its identity
3. **Verifiability**: Any reference can be cryptographically verified against its content
4. **Deduplication**: Identical logical structures automatically share storage and identity
5. **Distribution**: Content can be referenced and shared across network boundaries

## Core Components

### EntityId: Universal Content Identifier

The `EntityId` is the fundamental identifier type throughout Causality:

```rust
pub struct EntityId([u8; 32]);  // 256-bit SHA-256 hash
```

Properties:
- **Content-Derived**: Computed as SHA-256 hash of SSZ-serialized data
- **Deterministic**: Same logical content always produces same EntityId
- **Cryptographically Secure**: Collision-resistant and tamper-evident
- **Fixed Size**: 32 bytes enables efficient indexing and storage

### Specialized Identifier Types

Different types of content use specialized identifier types, all based on EntityId:

#### Resource Identifiers
```rust
pub struct ResourceId(EntityId);    // References to linear resources
```
- Used for heap-allocated linear resources in Layer 0
- Enables resource sharing and linearity tracking
- Supports zero-copy resource passing

#### Expression Identifiers  
```rust
pub struct ExprId(EntityId);        // References to Layer 1 expressions
```
- Used for all AST nodes and compiled expressions
- Enables automatic memoization and structural sharing
- Supports global optimization across expression boundaries

#### Effect Identifiers
```rust
pub struct EffectId(EntityId);      // References to Layer 2 effects
```
- Used for all effects and handlers
- Enables effect deduplication and caching
- Supports distributed effect composition

#### Value Identifiers
```rust
pub struct ValueExprId(EntityId);   // References to computed values
```
- Used for memoized computation results
- Enables lazy evaluation and result sharing
- Supports distributed computation caching

#### Domain Identifiers
```rust
pub struct DomainId(EntityId);      // References to capability domains
```
- Used for organizing capabilities and access control
- Enables domain-based permission systems
- Supports hierarchical capability organization

## Content-Addressed Value System

Causality's value system is built entirely around content addressing. Instead of traditional primitive values, the system operates with a unified value model:

### LispValue Enumeration

```rust
pub enum LispValue {
    // Direct primitive values (small, immutable)
    CoreValue(CoreValue),           // Unit, Bool, Int, Symbol
    
    // Content-addressed references (large or complex data)
    ResourceRef(EntityId),          // Reference to linear resource
    ExprRef(EntityId),              // Reference to expression/AST
    EffectRef(EntityId),            // Reference to effect
    ValueRef(EntityId),             // Reference to computed value
}
```

### CoreValue for Primitives

Small, immutable values are embedded directly:

```rust
pub enum CoreValue {
    Unit,
    Bool(bool),
    Int(i64),
    Symbol(String),
}
```

This hybrid approach optimizes for:
- **Efficiency**: Small values don't require indirection
- **Consistency**: Large values benefit from content addressing
- **Performance**: Reduces hash computation overhead for primitives

## Storage Architecture

### Content Store Interface

The content store provides the foundation for all content-addressed operations:

```rust
pub trait ContentStore {
    fn store(&mut self, content: &[u8]) -> Result<EntityId, Error>;
    fn retrieve(&self, id: &EntityId) -> Result<Option<Vec<u8>>, Error>;
    fn contains(&self, id: &EntityId) -> Result<bool, Error>;
    fn verify(&self, id: &EntityId, content: &[u8]) -> Result<bool, Error>;
}
```

#### Store Operation
- Serializes data using SSZ canonical format
- Computes SHA-256 hash of serialized bytes
- Returns EntityId representing the content hash
- Automatically deduplicates identical content

#### Retrieve Operation
- Looks up content by EntityId
- Returns original serialized bytes if found
- Enables verification that content matches claimed hash

#### Verification
- Recomputes hash of retrieved content
- Ensures data integrity and authenticity
- Detects any tampering or corruption

### Layer-Specific Storage

Different layers utilize content addressing in specialized ways:

#### Layer 0: Resource Heap
```rust
pub struct ResourceHeap {
    store: ContentStore,
    consumed: HashSet<EntityId>,    // Track consumed resources
    metadata: HashMap<EntityId, ResourceMetadata>,
}
```

- Stores linear resources by content hash
- Tracks consumption state for linearity enforcement
- Enables resource deduplication across allocations

#### Layer 1: Expression Store
```rust
pub struct ExpressionStore {
    store: ContentStore,
    compiled: HashMap<ExprId, Vec<Instruction>>,  // Compilation cache
    types: HashMap<ExprId, Type>,                 // Type information cache
}
```

- Stores AST nodes and compiled expressions
- Enables structural sharing of subexpressions
- Supports global optimization through shared recognition

#### Layer 2: Effect Registry
```rust
pub struct EffectRegistry {
    store: ContentStore,
    handlers: HashMap<EffectId, Vec<HandlerId>>,  // Effect -> Handler mappings
    schemas: HashMap<EffectId, EffectSchema>,     // Effect type schemas
}
```

- Stores effect definitions and handlers
- Enables effect composition and reuse
- Supports distributed effect systems

## Serialization and Hashing

### SSZ Integration

Content addressing relies on SSZ (Simple Serialize) for canonical serialization:

```rust
pub fn compute_entity_id<T: Encode>(value: &T) -> EntityId {
    let serialized = value.as_ssz_bytes();
    let hash = Sha256::digest(&serialized);
    EntityId(hash.into())
}
```

Benefits of SSZ:
- **Deterministic**: Same logical structure always produces same bytes
- **Efficient**: Minimal serialization overhead
- **Merkleizable**: Natural tree structure for hash computation
- **Cross-Language**: Consistent across Rust and OCaml implementations

### Hash Computation Pipeline

1. **Value Construction**: Create data structure in memory
2. **SSZ Serialization**: Convert to canonical byte representation
3. **Hash Computation**: Apply SHA-256 to serialized bytes
4. **EntityId Creation**: Wrap hash in appropriate identifier type
5. **Storage**: Store mapping from EntityId to serialized bytes

## Benefits and Applications

### Automatic Deduplication

Content addressing provides automatic deduplication at all levels:

```rust
// These two expressions automatically share storage:
let expr1 = ExprId::from_content(&lambda_expr);
let expr2 = ExprId::from_content(&identical_lambda_expr);
assert_eq!(expr1, expr2);  // Same content, same ID
```

### Structural Sharing

Large data structures can efficiently share subcomponents:

```rust
// Complex expressions can share subexpressions
let shared_subexpr = ExprId::from_content(&common_pattern);
let complex_expr1 = Expr::Apply(func1_id, shared_subexpr);
let complex_expr2 = Expr::Apply(func2_id, shared_subexpr);
// shared_subexpr is stored only once
```

### Lazy Evaluation

Content addressing enables lazy computation patterns:

```rust
// Represent expensive computation by its content hash
let computation_id = ExprId::from_content(&expensive_expr);
// Actual computation deferred until value needed
let result = evaluate_when_needed(computation_id);
```

### Global Memoization

Results can be cached globally by input hash:

```rust
// Cache computation results by input content
let input_hash = EntityId::from_content(&input_data);
if let Some(cached) = memo_cache.get(&input_hash) {
    return cached;
}
let result = expensive_computation(&input_data);
memo_cache.insert(input_hash, result.clone());
```

### Distributed Systems

Content addressing enables seamless distribution:

- **Location Independence**: Data referenced by content, not location
- **Replication**: Content can be stored and cached anywhere
- **Integrity**: All transfers automatically verified
- **Bandwidth Optimization**: Transfer only missing content

### Zero-Knowledge Integration

Content addressing provides natural ZK integration:

- **Deterministic Circuits**: Same logical structure produces same circuit
- **Witness Efficiency**: Reference large data by hash in proofs
- **Verification Optimization**: Verify content hashes instead of full data
- **Circuit Caching**: Reuse compiled circuits for identical content

Content addressing is not just a storage mechanism in Causality—it's a fundamental architectural principle that enables immutability, verifiability, and global optimization throughout the entire system. By treating data identity as intrinsic to content rather than extrinsic to location, Causality achieves unprecedented levels of integrity, efficiency, and composability. 

## Implementation Considerations

### Hash Function

The system uses SHA256 as the hash function for content addressing. SHA256 provides:

- **Deterministic**: Same content always produces the same hash
- **Collision-resistant**: Practically impossible to find two different inputs with the same hash
- **Standard**: Widely adopted cryptographic hash function with extensive validation
- **ZK-compatible**: Suitable for zero-knowledge proof systems
- **Fixed-size**: Always produces 32-byte (256-bit) outputs

### Storage Backends

The ContentStore trait supports multiple storage backends:

#### In-Memory Store
```rust
pub struct MemoryStore {
    data: HashMap<EntityId, Vec<u8>>,
}
```
- Fast access for development and testing
- No persistence across process restarts
- Memory usage grows with content size

#### File System Store
```rust
pub struct FileStore {
    root_path: PathBuf,
}
```
- Content stored as files named by EntityId
- Simple persistence model
- Operating system handles caching

#### Database Store
```rust
pub struct DatabaseStore {
    connection: DatabaseConnection,
}
```
- SQL or NoSQL database backend
- ACID properties for consistency
- Query capabilities for content analysis

#### Distributed Store
```rust
pub struct DistributedStore {
    local: Box<dyn ContentStore>,
    network: NetworkClient,
}
```
- Content distributed across network nodes
- Automatic replication and redundancy
- Peer-to-peer content sharing

### Performance Optimizations

#### Hash Computation Caching
```rust
// Cache hash computation for frequently accessed data
let mut hash_cache = HashMap::new();
fn cached_hash<T: Encode>(value: &T, cache: &mut HashMap<*const T, EntityId>) -> EntityId {
    let ptr = value as *const T;
    if let Some(cached) = cache.get(&ptr) {
        return *cached;
    }
    let hash = compute_entity_id(value);
    cache.insert(ptr, hash);
    hash
}
```

#### Content Prefetching
```rust
// Prefetch related content to reduce latency
fn prefetch_related_content(store: &ContentStore, expr_id: ExprId) {
    if let Some(expr) = store.retrieve_typed::<Expr>(&expr_id) {
        // Prefetch all referenced subexpressions
        for sub_id in expr.referenced_exprs() {
            store.prefetch(&sub_id);
        }
    }
}
```

#### Compression
```rust
// Compress stored content to reduce storage overhead
pub struct CompressedStore<S: ContentStore> {
    inner: S,
    compression: CompressionAlgorithm,
}
```

## Integration with Causality Layers

### Layer 0 Integration

Content addressing is fundamental to Layer 0's register machine:

- **Register Values**: Can hold primitive values or content-addressed references
- **Resource Heap**: All allocated resources identified by content hash
- **Instruction Operands**: Reference data by EntityId rather than raw values
- **Linear Tracking**: Resource consumption tracked by EntityId

### Layer 1 Integration

Layer 1's lambda calculus operates entirely on content-addressed expressions:

- **AST Nodes**: Every expression node has a unique ExprId
- **Compilation**: Compilation results cached by input ExprId
- **Optimization**: Identical subexpressions automatically shared
- **Type Checking**: Type information associated with ExprId

### Layer 2 Integration

Layer 2's effect system leverages content addressing for composition:

- **Effect Definitions**: All effects identified by EffectId
- **Handler Registry**: Handlers mapped to effect content hashes
- **Intent System**: Intents reference effects and resources by ID
- **TEG Construction**: Effect graphs built from content-addressed nodes

## Future Directions

### Advanced Hash Functions

Exploring next-generation hash functions for improved performance and security:

- **SHA256**: Current implementation with excellent performance characteristics
- **Poseidon**: ZK-friendly hash function for circuit integration
- **Post-Quantum**: Resistance to quantum computing attacks

### Content-Addressed Networking

Extending content addressing to network protocols:

- **Peer-to-Peer**: Direct content exchange by hash
- **CDN Integration**: Global content distribution networks
- **Offline Capability**: Local content stores for disconnected operation

### Formal Verification

Leveraging content addressing for formal methods:

- **Proof Checking**: Verify proofs by content hash
- **Theorem Libraries**: Share formal theorems by content
- **Verification Caching**: Reuse verification results for identical content

Content addressing is not just a storage mechanism in Causality—it's a fundamental architectural principle that enables immutability, verifiability, and global optimization throughout the entire system. By treating data identity as intrinsic to content rather than extrinsic to location, Causality achieves unprecedented levels of integrity, efficiency, and composability. 

### Performance Considerations

- **SHA256**: Standard cryptographic function with good performance characteristics
- **Cache locality**: Related content often stored together
- **Batch operations**: Multiple content addressing operations can be batched
- **Lazy evaluation**: Content hashing only performed when needed 