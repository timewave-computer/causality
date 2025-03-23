# Resource Relationships in Causality

## Overview

This document details the resource relationship model within the Causality architecture. Resource relationships define the connections, dependencies, and associations between resources. They enable complex network structures to be modeled, traversed, and validated throughout the system.

## Core Concepts

### Relationship Definition

A relationship represents a directed connection between two resources:

```rust
pub struct ResourceRelationship {
    /// Unique identifier for the relationship
    id: RelationshipId,
    
    /// Source resource (relationship origin)
    source_id: ResourceId,
    
    /// Target resource (relationship destination)
    target_id: ResourceId,
    
    /// Type of relationship
    relationship_type: RelationshipType,
    
    /// Creation timestamp
    created_at: Timestamp,
    
    /// Optional expiration timestamp
    expires_at: Option<Timestamp>,
    
    /// Relationship attributes (metadata)
    attributes: HashMap<String, Value>,
    
    /// Cryptographic proof of the relationship
    proof: Option<RelationshipProof>,
}
```

### Relationship Types

Relationships can have various semantics defined by their type:

```rust
pub enum RelationshipType {
    /// Ownership relationship (has direct ownership)
    Owns,
    
    /// Containment relationship (physically or logically contains)
    Contains,
    
    /// Dependency relationship (depends on functionality or state)
    DependsOn,
    
    /// Authorization relationship (has specific authorization)
    CanAccess {
        /// Specific access capabilities granted
        capabilities: Vec<CapabilityId>,
    },
    
    /// Reference relationship (simple reference without specific semantics)
    References,
    
    /// Derivation relationship (derived from or based on)
    DerivedFrom,
    
    /// Replacement relationship (replaces or supersedes)
    Replaces,
    
    /// Custom relationship with domain-specific semantics
    Custom {
        /// Type identifier
        type_id: String,
        /// Custom relationship data
        data: Vec<u8>,
    },
}
```

### Relationship Paths

Resources can be connected through paths of relationships:

```rust
pub struct RelationshipPath {
    /// Sequence of relationships forming the path
    relationships: Vec<RelationshipId>,
    
    /// Resources in sequence along the path
    resources: Vec<ResourceId>,
    
    /// Path properties (e.g., length, aggregated capabilities)
    properties: HashMap<String, Value>,
}
```

## Relationship Registry

Relationships are stored and managed in a dedicated registry:

```rust
pub struct RelationshipRegistry {
    /// All relationships indexed by ID
    relationships: HashMap<RelationshipId, ResourceRelationship>,
    
    /// Source-indexed lookup table
    source_index: HashMap<ResourceId, Vec<RelationshipId>>,
    
    /// Target-indexed lookup table
    target_index: HashMap<ResourceId, Vec<RelationshipId>>,
    
    /// Type-indexed lookup table
    type_index: HashMap<RelationshipType, Vec<RelationshipId>>,
}

impl RelationshipRegistry {
    /// Create a new relationship registry
    pub fn new() -> Self {
        Self {
            relationships: HashMap::new(),
            source_index: HashMap::new(),
            target_index: HashMap::new(),
            type_index: HashMap::new(),
        }
    }
    
    /// Register a new relationship
    pub fn register_relationship(
        &mut self,
        relationship: ResourceRelationship,
    ) -> Result<(), RelationshipError> {
        let id = relationship.id;
        
        // Check if relationship already exists
        if self.relationships.contains_key(&id) {
            return Err(RelationshipError::AlreadyExists);
        }
        
        // Update indices
        self.source_index.entry(relationship.source_id)
            .or_insert_with(Vec::new)
            .push(id);
        
        self.target_index.entry(relationship.target_id)
            .or_insert_with(Vec::new)
            .push(id);
        
        self.type_index.entry(relationship.relationship_type.clone())
            .or_insert_with(Vec::new)
            .push(id);
        
        // Store relationship
        self.relationships.insert(id, relationship);
        
        Ok(())
    }
    
    /// Get relationships by source
    pub fn get_relationships_by_source(
        &self,
        source_id: ResourceId,
    ) -> Vec<&ResourceRelationship> {
        self.source_index.get(&source_id)
            .map(|ids| ids.iter()
                .filter_map(|id| self.relationships.get(id))
                .collect())
            .unwrap_or_default()
    }
    
    /// Get relationships by target
    pub fn get_relationships_by_target(
        &self,
        target_id: ResourceId,
    ) -> Vec<&ResourceRelationship> {
        self.target_index.get(&target_id)
            .map(|ids| ids.iter()
                .filter_map(|id| self.relationships.get(id))
                .collect())
            .unwrap_or_default()
    }
    
    /// Get relationships by type
    pub fn get_relationships_by_type(
        &self,
        relationship_type: &RelationshipType,
    ) -> Vec<&ResourceRelationship> {
        self.type_index.get(relationship_type)
            .map(|ids| ids.iter()
                .filter_map(|id| self.relationships.get(id))
                .collect())
            .unwrap_or_default()
    }
    
    /// Remove a relationship
    pub fn remove_relationship(
        &mut self,
        relationship_id: RelationshipId,
    ) -> Result<ResourceRelationship, RelationshipError> {
        let relationship = self.relationships.remove(&relationship_id)
            .ok_or(RelationshipError::NotFound)?;
        
        // Update indices
        if let Some(source_rels) = self.source_index.get_mut(&relationship.source_id) {
            source_rels.retain(|id| *id != relationship_id);
        }
        
        if let Some(target_rels) = self.target_index.get_mut(&relationship.target_id) {
            target_rels.retain(|id| *id != relationship_id);
        }
        
        if let Some(type_rels) = self.type_index.get_mut(&relationship.relationship_type) {
            type_rels.retain(|id| *id != relationship_id);
        }
        
        Ok(relationship)
    }
}
```

## Creating and Managing Relationships

### Establishing Relationships

Relationships between resources are created through an authorization process:

```rust
/// Create a relationship between two resources
pub fn create_relationship(
    source_id: ResourceId,
    target_id: ResourceId,
    relationship_type: RelationshipType,
    attributes: HashMap<String, Value>,
    expiration: Option<Timestamp>,
    auth_context: AuthContext,
) -> Result<RelationshipId, RelationshipError> {
    // Verify both resources exist
    let source = registry.get_resource(source_id)?;
    let target = registry.get_resource(target_id)?;
    
    // Verify authorization to create this relationship
    if !auth_system.authorize_relationship_creation(
        source_id,
        target_id,
        &relationship_type,
        &auth_context,
    ) {
        return Err(RelationshipError::Unauthorized);
    }
    
    // Check any relationship-specific constraints
    relationship_validator.validate_relationship(
        source_id,
        target_id,
        &relationship_type,
        &attributes,
    )?;
    
    // Generate relationship ID
    let relationship_id = RelationshipId::generate();
    
    // Create relationship proof if required
    let proof = if is_proof_required(&relationship_type) {
        Some(create_relationship_proof(
            relationship_id,
            source_id,
            target_id,
            &relationship_type,
            &auth_context,
        )?)
    } else {
        None
    };
    
    // Create the relationship
    let relationship = ResourceRelationship {
        id: relationship_id,
        source_id,
        target_id,
        relationship_type,
        created_at: system.current_time(),
        expires_at: expiration,
        attributes,
        proof,
    };
    
    // Register the relationship
    relationship_registry.register_relationship(relationship)?;
    
    // Log relationship creation event
    event_log.record(RelationshipEvent::Created {
        relationship_id,
        source_id,
        target_id,
        relationship_type: relationship_type.to_string(),
        timestamp: system.current_time(),
    });
    
    Ok(relationship_id)
}
```

### Terminating Relationships

Relationships can be explicitly removed:

```rust
/// Remove an existing relationship
pub fn remove_relationship(
    relationship_id: RelationshipId,
    auth_context: AuthContext,
) -> Result<(), RelationshipError> {
    // Get the relationship
    let relationship = relationship_registry.get_relationship(relationship_id)?;
    
    // Verify authorization to remove this relationship
    if !auth_system.authorize_relationship_removal(
        relationship.source_id,
        relationship.target_id,
        &relationship.relationship_type,
        &auth_context,
    ) {
        return Err(RelationshipError::Unauthorized);
    }
    
    // Check if removing this relationship would violate any constraints
    relationship_validator.validate_relationship_removal(relationship_id)?;
    
    // Remove the relationship
    relationship_registry.remove_relationship(relationship_id)?;
    
    // Log relationship removal event
    event_log.record(RelationshipEvent::Removed {
        relationship_id,
        source_id: relationship.source_id,
        target_id: relationship.target_id,
        relationship_type: relationship.relationship_type.to_string(),
        timestamp: system.current_time(),
    });
    
    Ok(())
}
```

## Relationship Constraints and Validation

Relationships can be constrained and validated:

```rust
pub struct RelationshipConstraint {
    /// Type of relationship constraint
    constraint_type: RelationshipConstraintType,
    
    /// Error message if constraint is violated
    error_message: String,
    
    /// Constraint parameters
    parameters: HashMap<String, Value>,
}

pub enum RelationshipConstraintType {
    /// Source resource must be of specific type
    SourceType(ResourceType),
    
    /// Target resource must be of specific type
    TargetType(ResourceType),
    
    /// Limit on relationships of this type per source
    MaxOutgoing(usize),
    
    /// Limit on relationships of this type per target
    MaxIncoming(usize),
    
    /// Relationship must have specific attribute
    RequiredAttribute {
        /// Name of required attribute
        name: String,
        /// Type of the attribute
        attr_type: AttributeType,
    },
    
    /// Custom constraint logic
    Custom {
        /// Constraint identifier
        id: String,
        /// Constraint data
        data: Vec<u8>,
    },
}

/// Relationship validator ensures relationships meet defined constraints
pub struct RelationshipValidator {
    constraints: HashMap<RelationshipType, Vec<RelationshipConstraint>>,
    registry: RelationshipRegistry,
    resource_registry: ResourceRegistry,
}

impl RelationshipValidator {
    /// Validate a potential relationship before creation
    pub fn validate_relationship(
        &self,
        source_id: ResourceId,
        target_id: ResourceId,
        relationship_type: &RelationshipType,
        attributes: &HashMap<String, Value>,
    ) -> Result<(), RelationshipError> {
        // Get the source and target resources
        let source = self.resource_registry.get_resource(source_id)?;
        let target = self.resource_registry.get_resource(target_id)?;
        
        // Check constraints for this relationship type
        if let Some(constraints) = self.constraints.get(relationship_type) {
            for constraint in constraints {
                match &constraint.constraint_type {
                    RelationshipConstraintType::SourceType(required_type) => {
                        if source.resource_type() != *required_type {
                            return Err(RelationshipError::ConstraintViolation(
                                constraint.error_message.clone(),
                            ));
                        }
                    }
                    RelationshipConstraintType::TargetType(required_type) => {
                        if target.resource_type() != *required_type {
                            return Err(RelationshipError::ConstraintViolation(
                                constraint.error_message.clone(),
                            ));
                        }
                    }
                    RelationshipConstraintType::MaxOutgoing(max) => {
                        let outgoing_count = self.registry
                            .get_relationships_by_source(source_id)
                            .iter()
                            .filter(|r| r.relationship_type == *relationship_type)
                            .count();
                        
                        if outgoing_count >= *max {
                            return Err(RelationshipError::ConstraintViolation(
                                constraint.error_message.clone(),
                            ));
                        }
                    }
                    RelationshipConstraintType::MaxIncoming(max) => {
                        let incoming_count = self.registry
                            .get_relationships_by_target(target_id)
                            .iter()
                            .filter(|r| r.relationship_type == *relationship_type)
                            .count();
                        
                        if incoming_count >= *max {
                            return Err(RelationshipError::ConstraintViolation(
                                constraint.error_message.clone(),
                            ));
                        }
                    }
                    RelationshipConstraintType::RequiredAttribute { name, attr_type } => {
                        if let Some(value) = attributes.get(name) {
                            if !value.matches_type(attr_type) {
                                return Err(RelationshipError::ConstraintViolation(
                                    format!("Attribute {} has incorrect type", name),
                                ));
                            }
                        } else {
                            return Err(RelationshipError::ConstraintViolation(
                                format!("Required attribute {} is missing", name),
                            ));
                        }
                    }
                    RelationshipConstraintType::Custom { id, data } => {
                        // Call custom validation logic
                        if !self.execute_custom_validation(
                            id, 
                            data, 
                            source_id, 
                            target_id, 
                            relationship_type, 
                            attributes,
                        )? {
                            return Err(RelationshipError::ConstraintViolation(
                                constraint.error_message.clone(),
                            ));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate removal of a relationship
    pub fn validate_relationship_removal(
        &self,
        relationship_id: RelationshipId,
    ) -> Result<(), RelationshipError> {
        // Logic to verify if removing this relationship would violate system integrity
        
        // For example, check if any critical dependencies would be broken
        
        Ok(())
    }
}
```

## Relationship Traversal and Queries

Relationships can be queried and traversed to discover resource networks:

```rust
/// Find all resources related to a given resource
pub fn find_related_resources(
    resource_id: ResourceId,
    relationship_type: Option<RelationshipType>,
    direction: RelationshipDirection,
    auth_context: AuthContext,
) -> Result<Vec<ResourceId>, RelationshipError> {
    // Verify access to the resource
    if !auth_system.authorize_resource_view(resource_id, &auth_context) {
        return Err(RelationshipError::Unauthorized);
    }
    
    let mut related_resources = Vec::new();
    
    match direction {
        RelationshipDirection::Outgoing => {
            let relationships = relationship_registry.get_relationships_by_source(resource_id);
            
            for relationship in relationships {
                if relationship_type.is_none() || relationship_type.as_ref() == Some(&relationship.relationship_type) {
                    if auth_system.authorize_resource_view(relationship.target_id, &auth_context) {
                        related_resources.push(relationship.target_id);
                    }
                }
            }
        }
        RelationshipDirection::Incoming => {
            let relationships = relationship_registry.get_relationships_by_target(resource_id);
            
            for relationship in relationships {
                if relationship_type.is_none() || relationship_type.as_ref() == Some(&relationship.relationship_type) {
                    if auth_system.authorize_resource_view(relationship.source_id, &auth_context) {
                        related_resources.push(relationship.source_id);
                    }
                }
            }
        }
        RelationshipDirection::Both => {
            // Combine both directions
            let outgoing = find_related_resources(
                resource_id,
                relationship_type.clone(),
                RelationshipDirection::Outgoing,
                auth_context.clone(),
            )?;
            
            let incoming = find_related_resources(
                resource_id,
                relationship_type,
                RelationshipDirection::Incoming,
                auth_context,
            )?;
            
            related_resources.extend(outgoing);
            related_resources.extend(incoming);
            related_resources.sort();
            related_resources.dedup();
        }
    }
    
    Ok(related_resources)
}

/// Find a path between two resources
pub fn find_relationship_path(
    start_id: ResourceId,
    end_id: ResourceId,
    relationship_types: Option<Vec<RelationshipType>>,
    max_depth: usize,
    auth_context: AuthContext,
) -> Result<Option<RelationshipPath>, RelationshipError> {
    // Verify access to the start resource
    if !auth_system.authorize_resource_view(start_id, &auth_context) {
        return Err(RelationshipError::Unauthorized);
    }
    
    // Use breadth-first search to find a path
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut parent_map = HashMap::new();
    
    queue.push_back(start_id);
    visited.insert(start_id);
    
    while let Some(current_id) = queue.pop_front() {
        // Check if we've reached the target
        if current_id == end_id {
            return Ok(Some(construct_path(start_id, end_id, parent_map)?));
        }
        
        // Check if we've reached the maximum depth
        let current_path_len = get_path_length(start_id, current_id, &parent_map);
        if current_path_len >= max_depth {
            continue;
        }
        
        // Get all related resources
        let related = find_related_resources(
            current_id,
            None,
            RelationshipDirection::Outgoing,
            auth_context.clone(),
        )?;
        
        for related_id in related {
            if !visited.contains(&related_id) {
                // Check if relationship type is acceptable
                let relationship = get_relationship(current_id, related_id)?;
                
                if let Some(allowed_types) = &relationship_types {
                    if !allowed_types.contains(&relationship.relationship_type) {
                        continue;
                    }
                }
                
                // Add to the queue
                queue.push_back(related_id);
                visited.insert(related_id);
                parent_map.insert(related_id, current_id);
            }
        }
    }
    
    // No path found
    Ok(None)
}
```

## Cross-Domain Relationships

Relationships can span across domains:

```rust
/// Create a cross-domain relationship
pub fn create_cross_domain_relationship(
    source_id: ResourceId,
    target_domain: DomainId,
    target_remote_id: ResourceId,
    relationship_type: RelationshipType,
    attributes: HashMap<String, Value>,
    auth_context: AuthContext,
) -> Result<RelationshipId, RelationshipError> {
    // Verify source resource exists
    let source = registry.get_resource(source_id)?;
    
    // Verify authorization
    if !auth_system.authorize_cross_domain_relationship(
        source_id,
        target_domain,
        target_remote_id,
        &relationship_type,
        &auth_context,
    ) {
        return Err(RelationshipError::Unauthorized);
    }
    
    // Create cross-domain resource mapping if it doesn't exist
    if !cross_domain_registry.has_mapping(source_id, target_remote_id, target_domain) {
        cross_domain_registry.create_resource_mapping(
            source_id,
            target_remote_id,
            target_domain,
        )?;
    }
    
    // Generate relationship ID
    let relationship_id = RelationshipId::generate();
    
    // Create the cross-domain relationship message
    let relationship_msg = CrossDomainMessage::CreateRelationship {
        origin_domain: system.domain_id(),
        relationship_id,
        source_id,
        target_id: target_remote_id,
        relationship_type: relationship_type.clone(),
        attributes: attributes.clone(),
        timestamp: system.current_time(),
    };
    
    // Send the message to the target domain
    cross_domain_messenger.send_message(target_domain, relationship_msg)?;
    
    // Create a local representation of the cross-domain relationship
    let local_target_id = cross_domain_registry.get_local_mapping(
        target_remote_id,
        target_domain,
    )?;
    
    let cross_domain_attrs = attributes.clone();
    cross_domain_attrs.insert(
        "target_domain".to_string(),
        Value::String(target_domain.to_string()),
    );
    cross_domain_attrs.insert(
        "remote_target_id".to_string(),
        Value::String(target_remote_id.to_string()),
    );
    
    // Create the local relationship
    create_relationship(
        source_id,
        local_target_id,
        RelationshipType::Custom {
            type_id: format!("cross_domain:{}", relationship_type.to_string()),
            data: Vec::new(),
        },
        cross_domain_attrs,
        None,
        auth_context,
    )
}
```

## Relationship-Based Authorization

Relationships can be used for authorization decisions:

```rust
/// Check if a resource has access to another resource via relationships
pub fn check_relationship_based_access(
    accessor_id: ResourceId,
    target_id: ResourceId,
    required_capability: CapabilityId,
    auth_context: AuthContext,
) -> Result<bool, RelationshipError> {
    // First check direct CanAccess relationships
    let direct_relationships = relationship_registry.get_relationships_by_source(accessor_id)
        .into_iter()
        .filter(|r| r.target_id == target_id)
        .collect::<Vec<_>>();
    
    for relationship in direct_relationships {
        if let RelationshipType::CanAccess { capabilities } = &relationship.relationship_type {
            if capabilities.contains(&required_capability) {
                return Ok(true);
            }
        }
    }
    
    // Next, check for paths with capabilities
    let max_path_length = 5; // Configurable maximum length for capability delegation
    
    let path = find_capability_delegation_path(
        accessor_id,
        target_id,
        required_capability,
        max_path_length,
        auth_context,
    )?;
    
    Ok(path.is_some())
}

/// Find a path that delegates a specific capability
fn find_capability_delegation_path(
    start_id: ResourceId,
    end_id: ResourceId,
    capability: CapabilityId,
    max_depth: usize,
    auth_context: AuthContext,
) -> Result<Option<RelationshipPath>, RelationshipError> {
    // Implementation details...
    // This would use a modified breadth-first search that tracks capability delegation
    
    Ok(None) // Placeholder
}
```

## Usage Examples

### Creating Basic Relationships

```rust
// Create an ownership relationship between a user and a token
let user_id = get_user_resource_id();
let token_id = get_token_resource_id();

let ownership_rel_id = create_relationship(
    user_id,
    token_id,
    RelationshipType::Owns,
    HashMap::new(), // No additional attributes
    None,           // No expiration
    auth_context,
)?;

println!("Created ownership relationship: {}", ownership_rel_id);

// Create a dependency relationship between two resources
let service_id = get_service_resource_id();
let database_id = get_database_resource_id();

let dependency_attrs = HashMap::from([
    ("reason".to_string(), Value::String("Stores user data".to_string())),
    ("criticality".to_string(), Value::Integer(5)),
]);

let dependency_rel_id = create_relationship(
    service_id,
    database_id,
    RelationshipType::DependsOn,
    dependency_attrs,
    None, // No expiration
    auth_context,
)?;

println!("Created dependency relationship: {}", dependency_rel_id);
```

### Finding Related Resources

```rust
// Find all resources owned by a user
let owned_resources = find_related_resources(
    user_id,
    Some(RelationshipType::Owns),
    RelationshipDirection::Outgoing,
    auth_context,
)?;

println!("User owns {} resources:", owned_resources.len());
for resource_id in owned_resources {
    let resource = registry.get_resource(resource_id)?;
    println!("- {} ({})", resource.name(), resource.resource_type());
}

// Find all dependencies of a service
let dependencies = find_related_resources(
    service_id,
    Some(RelationshipType::DependsOn),
    RelationshipDirection::Outgoing,
    auth_context,
)?;

println!("Service depends on {} resources:", dependencies.len());
for dep_id in dependencies {
    let dep = registry.get_resource(dep_id)?;
    println!("- {}", dep.name());
}
```

### Relationship-Based Access Control

```rust
// Grant specific access capabilities through a relationship
let admin_id = get_admin_resource_id();
let system_id = get_system_resource_id();

let admin_capabilities = vec![
    CapabilityId::from_str("system.read"),
    CapabilityId::from_str("system.configure"),
    CapabilityId::from_str("system.restart"),
];

let access_rel_id = create_relationship(
    admin_id,
    system_id,
    RelationshipType::CanAccess { capabilities: admin_capabilities },
    HashMap::new(),
    Some(system.current_time() + Duration::days(30)), // 30-day access
    auth_context,
)?;

println!("Created admin access relationship: {}", access_rel_id);

// Later, check if admin can restart the system
let can_restart = check_relationship_based_access(
    admin_id,
    system_id,
    CapabilityId::from_str("system.restart"),
    auth_context,
)?;

if can_restart {
    println!("Admin can restart the system");
} else {
    println!("Admin cannot restart the system");
}
```

### Creating Cross-Domain Relationships

```rust
// Create a cross-domain relationship between local and remote resources
let local_service_id = get_local_service_id();
let remote_domain = DomainId::from_str("remote-domain-1");
let remote_service_id = get_remote_service_id();

let cross_domain_rel_id = create_cross_domain_relationship(
    local_service_id,
    remote_domain,
    remote_service_id,
    RelationshipType::DependsOn,
    HashMap::from([
        ("service_type".to_string(), Value::String("authentication".to_string())),
        ("priority".to_string(), Value::Integer(1)),
    ]),
    auth_context,
)?;

println!("Created cross-domain relationship: {}", cross_domain_rel_id);
```

## Implementation Status

The following components of the resource relationship system have been implemented:

- ✅ Core relationship model
- ✅ Basic relationship registry
- ✅ Relationship creation and termination
- ✅ Basic relationship validation
- ⚠️ Relationship traversal and path finding (partially implemented)
- ⚠️ Cross-domain relationships (partially implemented)
- ❌ Relationship-based authorization (not yet implemented)
- ❌ Advanced relationship constraints (not yet implemented)

## Future Enhancements

Future enhancements to the relationship system include:

1. **Graph-Based Queries**: Advanced graph query language for relationship traversal and analysis
2. **Typed Relationships**: Type-safe relationships with schema validation
3. **Relationship Composition**: Rules for composing relationships into higher-level structures
4. **Relationship Inference**: Inference of implicit relationships from explicit ones
5. **Bidirectional Relationships**: First-class support for bidirectional relationships with enforced consistency
6. **Relationship Triggers**: Event triggers on relationship creation, modification, and removal
7. **Relationship Analytics**: Analytics on relationship patterns and usage
8. **Visual Relationship Explorer**: Tools for visualizing and exploring resource relationships
</rewritten_file> 