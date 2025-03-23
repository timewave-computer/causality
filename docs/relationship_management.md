# Relationship Management

This document describes the relationship management system in Causality, focusing on how relationships between resources are tracked, maintained, and queried efficiently across domains.

## Overview

The relationship management system enables tracking and querying relationships between resources, both within a single domain and across multiple domains. Relationships provide a structured way to define and enforce connections between resources, enabling complex data models and operations that span multiple resources.

## Key Components

### Relationship Tracker

The core of the relationship management system is the `RelationshipTracker`, which is responsible for:

- Recording relationships between resources
- Indexing relationships for efficient querying
- Managing relationship metadata
- Tracking relationship types and directions
- Supporting traversal across relationships

```rust
pub struct RelationshipTracker {
    relationships: RwLock<HashMap<String, ResourceRelationship>>,
    source_index: RwLock<HashMap<ResourceId, HashSet<String>>>,
    target_index: RwLock<HashMap<ResourceId, HashSet<String>>>,
    type_index: RwLock<HashMap<RelationshipType, HashSet<String>>>,
}
```

### Relationship Types

The system supports various relationship types that express different semantic meanings:

```rust
pub enum RelationshipType {
    // Parent-child hierarchical relationship
    ParentChild,
    
    // Dependency relationship where one resource depends on another
    Dependency,
    
    // Consumption relationship where one resource consumes another
    Consumption,
    
    // Reference relationship where one resource references another
    Reference,
    
    // Custom relationship type with a user-defined name
    Custom(String),
}
```

### Relationship Direction

Relationships can have different directions:

```rust
pub enum RelationshipDirection {
    // From source to target
    ParentToChild,
    
    // From target to source
    ChildToParent,
    
    // Both ways between source and target
    Bidirectional,
}
```

### Cross-Domain Relationships

Cross-domain relationships extend the base relationship model to support connections between resources in different domains:

```rust
pub struct CrossDomainRelationship {
    // Resource in the source domain
    pub source_resource: ResourceId,
    
    // Source domain identifier
    pub source_domain: DomainId,
    
    // Resource in the target domain
    pub target_resource: ResourceId,
    
    // Target domain identifier
    pub target_domain: DomainId,
    
    // Type of cross-domain relationship
    pub relationship_type: CrossDomainRelationshipType,
    
    // Additional metadata for the relationship
    pub metadata: CrossDomainMetadata,
    
    // Whether the relationship is bidirectional
    pub bidirectional: bool,
}
```

## Relationship Query System

The relationship query system enables efficient traversal and discovery of resource relationships, especially across domains. It provides path-finding capabilities, relationship caching, and efficient indexing.

### Key Features

1. **Efficient Path Finding**: Uses breadth-first search algorithm to find paths between resources
2. **Relationship Caching**: Caches query results for improved performance
3. **Domain-Aware Indexing**: Efficiently indexes resources by domain for fast cross-domain queries
4. **Relationship Query Language**: Provides a structured way to query relationships
5. **Cross-Domain Paths**: Specialized algorithms for discovering paths that span multiple domains

### Usage Example

```rust
// Create a tracker and query executor
let tracker = Arc::new(RelationshipTracker::new());
let query_executor = RelationshipQueryExecutor::new(tracker.clone());

// Find paths between two resources
let query = RelationshipQuery::new(
    source_id.clone(), 
    target_id.clone()
)
.with_relationship_type(RelationshipType::Dependency)
.with_max_depth(5);

let paths = query_executor.execute(&query)?;

// Find cross-domain paths
let cross_domain_paths = query_executor.find_cross_domain_path(
    &source_id,
    &target_id,
    &source_domain,
    &target_domain,
)?;
```

## Domain-Aware Relationship Management

The relationship management system is domain-aware, meaning it understands and respects domain boundaries while still enabling cross-domain operations:

1. **Domain Indexing**: Resources are indexed by domain for efficient querying
2. **Domain-Specific Validation**: Relationships can be validated differently based on domain rules
3. **Domain Boundary Traversal**: Special handling for relationships that cross domain boundaries
4. **Domain-Specific Permissions**: Capability checks when traversing between domains

## Relationship Path Representation

When querying relationships, the system returns paths that represent the sequence of relationships between resources:

```rust
pub struct RelationshipPath {
    // Source resource ID
    pub source_id: ResourceId,
    
    // Target resource ID
    pub target_id: ResourceId,
    
    // Ordered list of relationships in the path
    pub relationships: Vec<ResourceRelationship>,
    
    // Total path length (number of hops)
    pub length: usize,
    
    // Domains traversed in the path
    pub domains: HashSet<DomainId>,
    
    // When this path was calculated
    pub calculated_at: Timestamp,
}
```

## Relationship Validation

The system supports validation of relationships to ensure they adhere to business rules and constraints:

1. **Type Validation**: Ensuring relationship types are appropriate for the connected resources
2. **Direction Validation**: Verifying that relationship directions make semantic sense
3. **Cross-Domain Validation**: Specialized validation for relationships that cross domain boundaries
4. **Lifecycle Validation**: Ensuring resources are in appropriate lifecycle states for the relationship
5. **Capability Validation**: Checking that the operation has the capabilities required to establish relationships

## Performance Considerations

The relationship management system is designed for performance:

1. **Efficient Indexing**: Multiple indices for fast relationship lookup
2. **Query Caching**: Caching of query results to avoid repeated complex traversals
3. **Bounded Traversal**: Maximum depth limits to prevent excessive resource consumption
4. **Incremental Results**: Ability to return partial results for complex queries
5. **Parallel Query Execution**: Support for executing queries in parallel where appropriate

## Integration with Other Systems

The relationship management system integrates with other core systems:

1. **Resource System**: Relationships are established between resources
2. **Capability System**: Capabilities control the ability to create and traverse relationships
3. **Lifecycle Management**: Resource lifecycle affects relationship validity
4. **Time Map**: Temporal consistency of relationships across domains
5. **Verification Framework**: Verification of relationship validity and consistency

## Conclusion

The relationship management system provides a powerful mechanism for modeling and querying connections between resources, both within and across domains. With efficient indexing, caching, and path-finding capabilities, it enables complex resource graphs while maintaining performance and scalability. 