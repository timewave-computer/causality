<!-- Tracking relationships -->
<!-- Original file: docs/src/relationship_tracker.md -->

# Resource Relationship Tracker

This document outlines the resource relationship tracking system within the unified resource architecture, focusing on how the `RelationshipTracker` manages relationships between resources and enforces relationship constraints.

## Core Concepts

### Resource Relationships

Resources in the Causality system can have different types of relationships:

1. **Ownership**: One resource owns another resource, implying privileges and control
2. **Dependency**: One resource depends on another resource for its functionality
3. **Reference**: One resource references another without strict dependency
4. **Composition**: One resource is composed of other resources
5. **Authorization**: One resource has authorization over another resource
6. **Derivation**: One resource is derived from another resource
7. **Temporal**: Resources linked with temporal constraints or sequences

These relationships allow the system to model complex interconnections between resources while enforcing appropriate constraints during resource lifecycle operations.

### The RelationshipTracker

The `RelationshipTracker` is responsible for:

1. **Relationship Registration**: Tracking relationships between resources
2. **Constraint Enforcement**: Ensuring relationships follow system rules
3. **Query Capabilities**: Providing methods to query resource relationships
4. **Validation**: Validating operations against relationship constraints
5. **Lifecycle Integration**: Cooperating with the lifecycle manager to enforce relationship rules

## Structure

```rust
/// Tracks relationships between resources
pub struct RelationshipTracker {
    /// Forward relationships: resource_id -> target_id -> relationship_type -> metadata
    forward_relationships: HashMap<ResourceId, HashMap<ResourceId, HashMap<RelationshipType, Option<MetadataMap>>>>,
    
    /// Reverse relationships: target_id -> resource_id -> relationship_type -> metadata
    reverse_relationships: HashMap<ResourceId, HashMap<ResourceId, HashMap<RelationshipType, Option<MetadataMap>>>>,
    
    /// Configured constraints for relationship validation
    constraints: Vec<Box<dyn RelationshipConstraint>>,
    
    /// Configuration for the relationship tracker
    config: RelationshipTrackerConfig,
}

/// Configuration for the relationship tracker
pub struct RelationshipTrackerConfig {
    /// Whether to enforce constraints during relationship operations
    enforce_constraints: bool,
    
    /// Whether to validate operations based on relationships
    validate_operations: bool,
    
    /// Maximum depth for recursive relationship queries
    max_query_depth: usize,
}

/// Types of relationships between resources
pub enum RelationshipType {
    /// Resource owns another resource
    Ownership,
    
    /// Resource depends on another resource
    Dependency,
    
    /// Resource references another resource
    Reference,
    
    /// Resource is composed of other resources
    Composition,
    
    /// Resource has authorization over another resource
    Authorization,
    
    /// Resource is derived from another resource
    Derivation,
    
    /// Resources have a temporal relationship
    Temporal {
        /// The type of temporal relationship
        temporal_type: TemporalRelationshipType,
    },
    
    /// Custom relationship type
    Custom(String),
}

/// Types of temporal relationships between resources
pub enum TemporalRelationshipType {
    /// Resource precedes another resource
    Precedes,
    
    /// Resource follows another resource
    Follows,
    
    /// Resources are concurrent
    Concurrent,
    
    /// Resources are synchronized
    Synchronized,
}
```

## Integration with Resource System

The relationship tracker integrates with the unified resource system:

1. **Resource Lifecycle Manager**: Coordinates with the lifecycle manager to prevent operations that would break relationship constraints
2. **Operation Validation**: Validates operations against relationship constraints
3. **Effect Templates**: Provides relationship information to effect templates
4. **Resource Register**: Tracks relationships as resources are registered and deregistered

## Relationship Graphs

The relationship tracker maintains a bi-directional graph of relationships:

```
   Resource A ─────> Resource B
      │   ↑            │   ↑ 
      │   │            │   │
      ▼   │            ▼   │
   Resource C <───── Resource D
```

This bi-directional tracking enables:
- Forward queries: "What resources does A have relationships with?"
- Reverse queries: "What resources have relationships with B?"
- Transitive queries: "What is the relationship path from A to D?"

## Usage Examples

### Basic Relationship Management

```rust
// Create a relationship tracker
let mut relationship_tracker = RelationshipTracker::new(
    RelationshipTrackerConfig::default()
        .with_enforce_constraints(true)
        .with_validate_operations(true)
        .with_max_query_depth(10)
);

// Register an ownership relationship
relationship_tracker.add_relationship(
    &owner_id,
    &resource_id,
    RelationshipType::Ownership,
    None
)?;

// Register a dependency relationship
relationship_tracker.add_relationship(
    &dependent_id,
    &dependency_id,
    RelationshipType::Dependency,
    Some(metadata)
)?;

// Check if a relationship exists
let has_relationship = relationship_tracker.has_relationship(
    &owner_id,
    &resource_id,
    Some(RelationshipType::Ownership)
)?;
assert!(has_relationship);

// Get all relationships for a resource
let relationships = relationship_tracker.get_relationships(&resource_id)?;
for (related_id, relationship_types) in relationships {
    for (relationship_type, metadata) in relationship_types {
        println!("Relationship: {:?} -> {:?} with metadata: {:?}",
                 related_id, relationship_type, metadata);
    }
}

// Remove a relationship
relationship_tracker.remove_relationship(
    &owner_id,
    &resource_id,
    RelationshipType::Ownership
)?;
```

### Recursive Relationship Queries

```rust
// Get all resources owned by a resource, recursively
let owned_resources = relationship_tracker.get_related_resources_recursive(
    &owner_id,
    RelationshipType::Ownership,
    QueryDirection::Outgoing,
    5 // Maximum depth
)?;

// Check for circular dependencies
let has_circular = relationship_tracker.has_circular_relationship(
    &resource_id,
    RelationshipType::Dependency
)?;
if has_circular {
    println!("Warning: Circular dependency detected");
}

// Find all resources that transitively depend on a resource
let dependent_resources = relationship_tracker.get_related_resources_recursive(
    &dependency_id,
    RelationshipType::Dependency,
    QueryDirection::Incoming,
    5 // Maximum depth
)?;

// Get a dependency graph as an adjacency list
let dependency_graph = relationship_tracker.get_relationship_graph(
    &root_id,
    RelationshipType::Dependency,
    QueryDirection::Both,
    5 // Maximum depth
)?;
```

### Integration with the Lifecycle Manager

```rust
// Attempt to consume a resource with dependencies
let resource_id = resource.id();

// Check if the resource can be consumed based on relationships
let can_consume = relationship_tracker.can_perform_operation(
    &resource_id,
    OperationType::ConsumeResource
)?;

if !can_consume {
    // Get resources that prevent the operation
    let blocking_resources = relationship_tracker
        .get_related_resources(
            &resource_id,
            RelationshipType::Dependency,
            QueryDirection::Incoming
        )?;
    
    println!("Cannot consume resource because it has dependencies: {:?}",
             blocking_resources);
}
```

### Relationship-Based Authorization

```rust
// Create an authorization relationship
relationship_tracker.add_relationship(
    &authorizer_id,
    &resource_id,
    RelationshipType::Authorization,
    Some(metadata_map! {
        "rights" => Rights::from([Right::Read, Right::Update]),
        "expires_at" => time::now() + Duration::days(30),
    })
)?;

// Check if an entity has authorization over a resource
let has_auth = relationship_tracker.has_relationship_with_metadata(
    &authorizer_id,
    &resource_id,
    RelationshipType::Authorization,
    |metadata| {
        if let Some(metadata) = metadata {
            if let (Some(rights), Some(expires_at)) = 
                (metadata.get::<Rights>("rights"), metadata.get::<Timestamp>("expires_at")) {
                return rights.contains(Right::Update) && *expires_at > time::now();
            }
        }
        false
    }
)?;

if has_auth {
    // Proceed with authorized operation
    println!("Entity is authorized to update the resource");
}
```

### Relationship Constraints

```rust
// Define a custom relationship constraint
struct OwnershipConstraint;

impl RelationshipConstraint for OwnershipConstraint {
    fn validate(
        &self,
        relationship_tracker: &RelationshipTracker,
        source_id: &ResourceId,
        target_id: &ResourceId,
        relationship_type: &RelationshipType,
        metadata: &Option<MetadataMap>,
    ) -> Result<ValidationResult> {
        // Example: A resource cannot own more than 5 other resources
        if *relationship_type == RelationshipType::Ownership {
            let existing_ownership = relationship_tracker.count_relationships(
                source_id,
                Some(RelationshipType::Ownership),
                QueryDirection::Outgoing
            )?;
            
            if existing_ownership >= 5 {
                return Ok(ValidationResult::invalid(
                    "A resource cannot own more than 5 other resources"
                ));
            }
        }
        
        Ok(ValidationResult::valid())
    }
}

// Add the constraint to the relationship tracker
relationship_tracker.add_constraint(Box::new(OwnershipConstraint));

// Now adding a relationship will validate against this constraint
let result = relationship_tracker.add_relationship(
    &owner_id,
    &new_resource_id,
    RelationshipType::Ownership,
    None
);

// If owner already has 5 ownership relationships, this will fail
assert!(result.is_err());
```

### Batch Operations

```rust
// Create a batch of relationship operations
let batch = vec![
    RelationshipOperation::Add {
        source: resource1_id.clone(),
        target: resource2_id.clone(),
        relationship_type: RelationshipType::Ownership,
        metadata: None,
    },
    RelationshipOperation::Add {
        source: resource1_id.clone(),
        target: resource3_id.clone(),
        relationship_type: RelationshipType::Composition,
        metadata: Some(metadata1),
    },
    RelationshipOperation::Remove {
        source: resource1_id.clone(),
        target: resource4_id.clone(),
        relationship_type: RelationshipType::Reference,
    },
];

// Apply the batch atomically
relationship_tracker.apply_batch(batch)?;
```

### Metadata-Enhanced Relationships

```rust
// Add a relationship with rich metadata
let metadata = metadata_map! {
    "created_at" => time::now(),
    "priority" => 5u32,
    "tags" => vec!["important", "critical"],
    "properties" => hashmap! {
        "weight" => 0.75f64,
        "threshold" => 100u32,
    },
};

relationship_tracker.add_relationship(
    &resource1_id,
    &resource2_id,
    RelationshipType::Composition,
    Some(metadata)
)?;

// Query relationships based on metadata criteria
let high_priority_relationships = relationship_tracker
    .find_relationships_by_metadata(
        &resource1_id,
        None, // Any relationship type
        QueryDirection::Outgoing,
        |metadata| {
            if let Some(metadata) = metadata {
                if let Some(priority) = metadata.get::<u32>("priority") {
                    return *priority >= 3;
                }
            }
            false
        }
    )?;
```

## Relationship Events

The relationship tracker can emit events for relationship changes:

```rust
// Subscribe to relationship events
let subscription = event_system.subscribe(
    EventFilter::new()
        .with_event_type(EventType::RelationshipChange)
        .with_resource_id(resource_id.clone())
);

// Perform a relationship operation
relationship_tracker.add_relationship(
    &resource1_id,
    &resource2_id,
    RelationshipType::Dependency,
    None
)?;

// Check for the event
let events = subscription.collect_events()?;
assert_eq!(events.len(), 1);
let event = &events[0];
assert_eq!(event.source_id(), &resource1_id);
assert_eq!(event.target_id(), &resource2_id);
assert_eq!(
    event.get_value::<RelationshipType>("relationship_type")?,
    RelationshipType::Dependency
);
```

## Time-Based Relationships

```rust
// Add a temporal relationship
relationship_tracker.add_relationship(
    &resource1_id,
    &resource2_id,
    RelationshipType::Temporal {
        temporal_type: TemporalRelationshipType::Precedes,
    },
    Some(metadata_map! {
        "min_gap" => Duration::minutes(5),
        "max_gap" => Duration::hours(1),
    })
)?;

// Query for temporal sequences
let sequence = relationship_tracker.get_temporal_sequence(
    &start_resource_id,
    TemporalRelationshipType::Precedes,
    10 // Maximum length
)?;

// For each resource in the sequence
for resource_id in sequence {
    println!("Resource in sequence: {:?}", resource_id);
}
```

## Best Practices

1. **Avoid Circular Dependencies**: Always check for circular dependencies when adding dependency relationships.

2. **Use Appropriate Relationship Types**: Choose the most appropriate relationship type to accurately model the relationship semantics.

3. **Leverage Metadata**: Use relationship metadata to store important contextual information about relationships.

4. **Set Constraints**: Define and enforce relationship constraints to maintain system integrity.

5. **Batch Related Changes**: Use batch operations to ensure relationship changes are atomic.

6. **Consider Performance**: Be mindful of recursive queries that might traverse large portions of the relationship graph.

7. **Clean Up Relationships**: Always clean up relationships when resources are removed or significantly changed.

8. **Validate Before Critical Operations**: Always validate against relationships before performing destructive operations.

9. **Use Temporal Relationships**: Use temporal relationships for sequence-dependent resources.

10. **Leverage the Relationship Graph**: Use graph algorithms to analyze complex relationship structures.

## Implementation Status

The relationship tracking system is fully implemented in the Causality system:

- ✅ Core `RelationshipTracker` structure
- ✅ All relationship types and operations
- ✅ Recursive relationship queries
- ✅ Integration with the lifecycle manager
- ✅ Constraint validation system
- ✅ Metadata-enhanced relationships
- ✅ Batch operations
- ✅ Event system

## Future Enhancements

1. **Graph Algorithms**: Additional graph algorithms for analyzing resource relationships
2. **Visualization Tools**: Tools for visualizing resource relationship graphs
3. **Relationship Statistics**: Metrics and statistics about the relationship graph
4. **Performance Optimizations**: Optimizations for large relationship graphs
5. **Pattern Matching**: Support for identifying common relationship patterns
6. **Schema Validation**: Validation of relationship metadata against schemas
7. **Declarative Constraints**: Declarative language for defining relationship constraints
8. **Relationship Versioning**: Tracking changes to relationships over time 