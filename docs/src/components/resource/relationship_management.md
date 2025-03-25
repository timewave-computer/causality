<!-- Management of relationships -->
<!-- Original file: docs/src/relationship_management.md -->

# Relationship Management

This document describes the relationship management system in Causality, focusing on how relationships between ResourceRegisters are tracked, maintained, and queried efficiently across domains.

## Overview

The relationship management system enables tracking and querying relationships between ResourceRegisters, both within a single domain and across multiple domains. Relationships provide a structured way to define and enforce connections between resources, enabling complex data models and operations that span multiple resources.

## Key Components

### Relationship Tracker

The core of the relationship management system is the `RelationshipTracker`, which is responsible for:

- Recording relationships between ResourceRegisters
- Indexing relationships for efficient querying
- Managing relationship metadata
- Tracking relationship types and directions
- Supporting traversal across relationships

```rust
pub struct RelationshipTracker {
    // Content-addressed relationships for immutability and verification
    relationships: RwLock<HashMap<ContentHash, ResourceRelationship>>,
    // Indices for efficient lookup
    source_index: RwLock<HashMap<RegisterId, HashSet<ContentHash>>>,
    target_index: RwLock<HashMap<RegisterId, HashSet<ContentHash>>>,
    type_index: RwLock<HashMap<RelationshipType, HashSet<ContentHash>>>,
    // Domain-specific indices for cross-domain queries
    domain_index: RwLock<HashMap<DomainId, HashSet<ContentHash>>>,
}
```

### Relationship Types

The system supports various relationship types that express different semantic meanings:

```rust
pub enum RelationshipType {
    // Mirror relationship (identical resources across domains)
    Mirror,
    
    // Reference relationship (resource references)
    Reference,
    
    // Ownership relationship (hierarchical ownership)
    Ownership,
    
    // Derived relationship (derived data)
    Derived,
    
    // Bridge relationship (domain-spanning connections)
    Bridge,
    
    // Custom relationship type with a user-defined name
    Custom(String),
}
```

### Relationship Direction

Relationships can have different directions:

```rust
pub enum RelationshipDirection {
    // From source to target
    SourceToTarget,
    
    // From target to source
    TargetToSource,
    
    // Both ways between source and target
    Bidirectional,
}
```

### Cross-Domain Relationships

Cross-domain relationships extend the base relationship model to support connections between ResourceRegisters in different domains:

```rust
pub struct CrossDomainRelationship {
    // ResourceRegister in the source domain
    pub source_register: RegisterId,
    
    // Source domain identifier
    pub source_domain: DomainId,
    
    // ResourceRegister in the target domain
    pub target_register: RegisterId,
    
    // Target domain identifier
    pub target_domain: DomainId,
    
    // Type of cross-domain relationship
    pub relationship_type: RelationshipType,
    
    // Additional metadata for the relationship
    pub metadata: HashMap<String, Value>,
    
    // Direction of the relationship
    pub direction: RelationshipDirection,
    
    // Temporal context when this relationship was established
    pub temporal_context: TemporalContext,
    
    // Content hash of this relationship (for content addressing)
    pub content_hash: ContentHash,
}
```

## Unified Operation Model for Relationship Management

With the unified operation model, relationship operations are represented using the same `Operation<C>` structure as other system operations:

```rust
// Create a relationship operation
let relationship_op = Operation::new(OperationType::CreateRelationship)
    .with_input(source_register_ref)
    .with_output(target_register_ref)
    .with_parameter("type", RelationshipType::Ownership)
    .with_parameter("direction", RelationshipDirection::SourceToTarget)
    .with_context(AbstractContext::new());

// Execute the operation
let result = execute_operation(relationship_op, relationship_executor).await?;
```

## Relationship Query System

The relationship query system enables efficient traversal and discovery of resource relationships, especially across domains. It provides path-finding capabilities, relationship caching, and efficient indexing.

### Key Features

1. **Efficient Path Finding**: Uses optimized graph traversal algorithms to find paths between resources
2. **Content-Addressed Caching**: Caches query results with content-addressed keys for verification
3. **Domain-Aware Indexing**: Efficiently indexes ResourceRegisters by domain for fast cross-domain queries
4. **Relationship Query Language**: Provides a structured way to query relationships
5. **Cross-Domain Paths**: Specialized algorithms for discovering paths that span multiple domains

### Usage Example

```rust
// Create a tracker and query executor
let tracker = Arc::new(RelationshipTracker::new());
let query_executor = RelationshipQueryExecutor::new(tracker.clone());

// Find paths between two registers
let query = RelationshipQuery::new(
    source_register.id.clone(), 
    target_register.id.clone()
)
.with_relationship_type(RelationshipType::Ownership)
.with_max_depth(5);

let paths = query_executor.execute(&query)?;

// Find cross-domain paths
let cross_domain_paths = query_executor.find_cross_domain_path(
    &source_register.id,
    &target_register.id,
    &source_domain,
    &target_domain,
)?;
```

## Storage as an Effect

With the unified model, relationship storage operations are represented as storage effects:

```rust
// Store a relationship with appropriate storage strategy
effect_system.execute_effect(StorageEffect::StoreRelationship {
    relationship_id: relationship.content_hash,
    source_register: relationship.source_register,
    target_register: relationship.target_register,
    domains: vec![relationship.source_domain, relationship.target_domain],
    storage_strategy: StorageStrategy::CommitmentBased {
        commitment: Some(relationship_commitment),
        nullifier: None,
    },
    continuation: Box::new(|result| {
        println!("Relationship storage result: {:?}", result)
    }),
}).await?;
```

## Domain-Aware Relationship Management

The relationship management system is domain-aware, meaning it understands and respects domain boundaries while still enabling cross-domain operations:

1. **Domain Indexing**: ResourceRegisters are indexed by domain for efficient querying
2. **Domain-Specific Validation**: Relationships can be validated differently based on domain rules
3. **Domain Boundary Traversal**: Special handling for relationships that cross domain boundaries
4. **Capability-Based Domain Access**: Capability checks when traversing between domains

## Relationship Path Representation

When querying relationships, the system returns content-addressed paths that represent the sequence of relationships between resources:

```rust
pub struct RelationshipPath {
    // Source register ID
    pub source_id: RegisterId,
    
    // Target register ID
    pub target_id: RegisterId,
    
    // Ordered list of relationships in the path
    pub relationships: Vec<ContentRef<ResourceRelationship>>,
    
    // Total path length (number of hops)
    pub length: usize,
    
    // Domains traversed in the path
    pub domains: HashSet<DomainId>,
    
    // Temporal context when this path was calculated
    pub temporal_context: TemporalContext,
    
    // Content hash of this path for verification
    pub content_hash: ContentHash,
}
```

## Relationship Validation through Unified Verification Framework

The system supports validation of relationships using the unified verification framework:

```rust
impl Verifiable for Relationship {
    type Proof = UnifiedProof;
    type Subject = RelationshipValidity;
    
    fn prove(&self, context: &VerificationContext) -> Result<Self::Proof, VerificationError> {
        // Generate relationship validity proof
        let logical_proof = generate_logical_proof(self, context)?;
        let temporal_proof = generate_temporal_proof(self, &context.time_map)?;
        
        // Create unified proof with multiple components
        let proof = UnifiedProof {
            zk_components: None,
            temporal_components: Some(temporal_proof),
            ancestral_components: None,
            logical_components: Some(logical_proof),
            cross_domain_components: Some(generate_cross_domain_proof(self, context)?),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            signature: None,
        };
        
        Ok(proof)
    }
    
    fn verify(&self, proof: &Self::Proof, context: &VerificationContext) -> Result<bool, VerificationError> {
        // Verify logical component
        let logical_valid = if let Some(logical_proof) = &proof.logical_components {
            verify_logical_consistency(self, logical_proof, context)?
        } else {
            return Err(VerificationError::MissingProofComponent("logical_components"));
        };
        
        // Verify temporal component
        let temporal_valid = if let Some(temporal_proof) = &proof.temporal_components {
            verify_temporal_consistency(self, temporal_proof, &context.time_map)?
        } else {
            return Err(VerificationError::MissingProofComponent("temporal_components"));
        };
        
        // Verify cross-domain component
        let cross_domain_valid = if let Some(cross_domain_proof) = &proof.cross_domain_components {
            verify_cross_domain_validity(self, cross_domain_proof, context)?
        } else {
            return Err(VerificationError::MissingProofComponent("cross_domain_components"));
        };
        
        // All aspects must be valid
        Ok(logical_valid && temporal_valid && cross_domain_valid)
    }
}
```

## Performance Considerations

The relationship management system is designed for performance:

1. **Content-Addressed Caching**: Perfect caching of query results using content hashes
2. **Bounded Traversal**: Maximum depth limits to prevent excessive resource consumption
3. **Parallel Query Execution**: Support for executing queries in parallel with explicit resource guards
4. **Incremental Results**: Ability to return partial results for complex queries
5. **Storage Optimization**: Multiple storage strategies based on relationship importance and query patterns

## Integration with Other Systems

The relationship management system integrates with the unified architecture:

1. **Unified ResourceRegister System**: Relationships are established between ResourceRegisters
2. **Capability-Based Authorization**: Capabilities control the ability to create and traverse relationships
3. **Three-Layer Effect Architecture**: Relationship operations flow through the unified effect pipeline
4. **Unified Verification Framework**: Comprehensive verification of relationship validity
5. **Universal Content-Addressing**: All relationships are content-addressed for verification

## Conclusion

The relationship management system provides a powerful mechanism for modeling and querying connections between ResourceRegisters, both within and across domains. With content-addressed relationships, capability-based access control, and integration with the unified verification framework, it enables complex resource graphs while maintaining security, consistency, and scalability. 