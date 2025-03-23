# Relationship Validation

This document outlines the relationship validation system in Causality, focusing on how relationships between resources are validated, enforced, and maintained to ensure consistency and correctness.

## Overview

The relationship validation system ensures that relationships between resources conform to defined rules and constraints. It provides mechanisms to validate relationships at creation time, during modifications, and as part of ongoing maintenance. The system integrates with the broader verification framework to ensure consistency both within and across domains.

## Validation Mechanisms

### Type Validation

Type validation ensures that the types of resources in a relationship are compatible with the relationship type:

```rust
// Example of type validation for a dependency relationship
fn validate_dependency_relationship(
    source_resource: &ResourceRegister,
    target_resource: &ResourceRegister,
) -> Result<()> {
    // Check that the resource types are compatible
    if !is_dependency_compatible(&source_resource.resource_type, &target_resource.resource_type) {
        return Err(Error::InvalidRelationship(format!(
            "Resource of type {} cannot depend on resource of type {}",
            source_resource.resource_type,
            target_resource.resource_type
        )));
    }
    
    Ok(())
}
```

### Lifecycle State Validation

Lifecycle state validation ensures that resources are in appropriate states to participate in a relationship:

```rust
// Example of lifecycle state validation
fn validate_resource_state_for_relationship(
    resource: &ResourceRegister,
    expected_states: &[RegisterState],
) -> Result<()> {
    if !expected_states.contains(&resource.state) {
        return Err(Error::InvalidState(format!(
            "Resource {} is in state {:?}, expected one of {:?}",
            resource.id, resource.state, expected_states
        )));
    }
    
    Ok(())
}
```

### Directionality Validation

Directionality validation ensures that the direction of a relationship is appropriate for the relationship type:

```rust
// Example of directionality validation
fn validate_relationship_direction(
    relationship_type: &RelationshipType,
    direction: &RelationshipDirection,
) -> Result<()> {
    match (relationship_type, direction) {
        (RelationshipType::ParentChild, RelationshipDirection::ChildToParent) => {
            Err(Error::InvalidDirection(
                "ParentChild relationship cannot have ChildToParent direction".to_string()
            ))
        },
        // Other cases...
        _ => Ok(()),
    }
}
```

### Cross-Domain Validation

Cross-domain validation ensures that relationships that span domain boundaries adhere to additional constraints:

```rust
// Example of cross-domain validation
fn validate_cross_domain_relationship(
    source_domain: &DomainId,
    target_domain: &DomainId,
    relationship_type: &CrossDomainRelationshipType,
) -> Result<()> {
    // Check if domains are compatible for the relationship type
    if !is_cross_domain_compatible(source_domain, target_domain, relationship_type) {
        return Err(Error::InvalidCrossDomainRelationship(format!(
            "Domains {} and {} cannot have a {:?} relationship",
            source_domain, target_domain, relationship_type
        )));
    }
    
    Ok(())
}
```

## Validation Rules

Validation rules define the constraints that relationships must satisfy to be considered valid. These rules can be domain-specific, relationship-type-specific, or general.

### Rule Types

```rust
// Example of validation rule types
pub enum ValidationRuleType {
    // Resource type compatibility
    TypeCompatibility,
    
    // Lifecycle state compatibility
    StateCompatibility,
    
    // Direction validity
    DirectionValidity,
    
    // Cross-domain compatibility
    CrossDomainCompatibility,
    
    // Capability requirements
    CapabilityRequirement,
    
    // Custom rule with specific logic
    Custom(String),
}
```

### Rule Implementation

```rust
// Example of a validation rule implementation
pub struct ValidationRule {
    // Type of the rule
    pub rule_type: ValidationRuleType,
    
    // Function that implements the validation logic
    pub validate: Box<dyn Fn(&ResourceRelationship, &ValidationContext) -> ValidationResult>,
    
    // Severity of the rule (error, warning, etc.)
    pub severity: ValidationSeverity,
    
    // Description of the rule
    pub description: String,
}
```

## Validation Context

The validation context provides information needed for validation:

```rust
// Example of validation context
pub struct ValidationContext {
    // Resources involved in the relationship
    pub resources: HashMap<ResourceId, ResourceRegister>,
    
    // Domains involved
    pub domains: HashMap<DomainId, DomainInfo>,
    
    // Capabilities available for the operation
    pub capabilities: Vec<Capability>,
    
    // Current time snapshot
    pub time_snapshot: TimeMapSnapshot,
    
    // Additional context-specific data
    pub metadata: HashMap<String, String>,
}
```

## Validation Pipeline

The validation pipeline orchestrates the validation process:

1. **Rule Collection**: Gather applicable validation rules
2. **Context Preparation**: Prepare the validation context
3. **Rule Execution**: Execute validation rules
4. **Result Aggregation**: Combine validation results
5. **Decision Making**: Determine if validation passed or failed

```rust
// Example of validation pipeline
fn validate_relationship(
    relationship: &ResourceRelationship,
    validator: &RelationshipValidator,
) -> ValidationResult {
    // 1. Collect applicable rules
    let rules = validator.get_applicable_rules(relationship);
    
    // 2. Prepare validation context
    let context = validator.prepare_context(relationship)?;
    
    // 3. Execute validation rules
    let mut results = Vec::new();
    for rule in rules {
        let result = (rule.validate)(relationship, &context);
        results.push(result);
    }
    
    // 4. Aggregate results
    let aggregated_result = aggregate_validation_results(results);
    
    // 5. Make decision
    if aggregated_result.has_errors() {
        return ValidationResult::Failed(aggregated_result.errors);
    } else {
        return ValidationResult::Passed;
    }
}
```

## Cross-Domain Relationship Validation

Cross-domain relationships have additional validation requirements:

1. **Domain Compatibility**: Ensuring the domains can have relationships of the specified type
2. **Domain-Specific Rules**: Applying domain-specific validation rules
3. **Boundary Crossing Validation**: Validating that boundary crossing is permitted
4. **Capability Verification**: Verifying capabilities for cross-domain operations
5. **Time Consistency**: Ensuring temporal consistency across domains

```rust
// Example of cross-domain relationship validation
fn validate_cross_domain_relationship(
    cross_domain_relationship: &CrossDomainRelationship,
    validator: &CrossDomainRelationshipValidator,
) -> ValidationResult {
    // 1. Validate domain compatibility
    let domain_result = validator.validate_domain_compatibility(
        &cross_domain_relationship.source_domain,
        &cross_domain_relationship.target_domain,
        &cross_domain_relationship.relationship_type,
    )?;
    
    // 2. Apply domain-specific rules
    let source_domain_result = validator.validate_source_domain_rules(cross_domain_relationship)?;
    let target_domain_result = validator.validate_target_domain_rules(cross_domain_relationship)?;
    
    // 3. Validate boundary crossing
    let boundary_result = validator.validate_boundary_crossing(cross_domain_relationship)?;
    
    // 4. Verify capabilities
    let capability_result = validator.validate_capabilities(cross_domain_relationship)?;
    
    // 5. Validate time consistency
    let time_result = validator.validate_time_consistency(cross_domain_relationship)?;
    
    // Aggregate all results
    aggregate_validation_results(vec![
        domain_result,
        source_domain_result,
        target_domain_result,
        boundary_result,
        capability_result,
        time_result,
    ])
}
```

## Query-Time Validation

In addition to validation at relationship creation time, the system also performs validation during relationship queries:

1. **Path Validity**: Ensuring paths between resources are valid
2. **Access Control**: Validating that the querying entity has appropriate access
3. **Time Consistency**: Ensuring temporal consistency in the query results
4. **Domain Boundary Rules**: Enforcing domain boundary rules during traversal

```rust
// Example of query-time validation
fn validate_query_path(
    path: &RelationshipPath,
    query_context: &QueryContext,
) -> Result<()> {
    // Validate each relationship in the path
    for relationship in &path.relationships {
        // Check access permissions
        if !has_access_permission(query_context.capabilities, relationship) {
            return Err(Error::AccessDenied(format!(
                "Access denied for relationship between {} and {}",
                relationship.source_id, relationship.target_id
            )));
        }
        
        // Check domain boundary rules if crossing domains
        if is_crossing_domain_boundary(relationship) {
            validate_domain_boundary_crossing(relationship, query_context)?;
        }
    }
    
    // Validate temporal consistency of the full path
    validate_path_temporal_consistency(path, query_context)?;
    
    Ok(())
}
```

## Relationship Constraint System

The relationship constraint system defines constraints that relationships must satisfy:

```rust
// Example of relationship constraints
pub enum RelationshipConstraint {
    // Maximum number of relationships of a type
    MaxRelationships(RelationshipType, usize),
    
    // Required relationships for a resource
    RequiredRelationship(RelationshipType),
    
    // Exclusive relationships (cannot have both types)
    ExclusiveRelationships(RelationshipType, RelationshipType),
    
    // Custom constraint with specific logic
    Custom(String, Box<dyn Fn(&ResourceId, &RelationshipTracker) -> Result<()>>),
}
```

## Integration with Resource Lifecycle

Relationships are integrated with the resource lifecycle:

1. **Creation Validation**: Validating relationships when resources are created
2. **State Transition Validation**: Ensuring relationships are valid during state transitions
3. **Consumption Validation**: Validating relationships when resources are consumed
4. **Archival Validation**: Ensuring relationship integrity during archival

```rust
// Example of lifecycle integration
fn validate_resource_state_transition(
    resource_id: &ResourceId,
    from_state: &RegisterState,
    to_state: &RegisterState,
    relationship_tracker: &RelationshipTracker,
) -> Result<()> {
    let relationships = relationship_tracker.get_resource_relationships(resource_id)?;
    
    // Check if the state transition is valid given the relationships
    for relationship in relationships {
        validate_relationship_for_state_transition(
            &relationship,
            resource_id,
            from_state,
            to_state,
        )?;
    }
    
    Ok(())
}
```

## Conclusion

The relationship validation system ensures that relationships between resources adhere to defined rules and constraints, maintaining consistency and correctness within the system. Through integration with the resource lifecycle, the capability system, and the cross-domain validation mechanisms, it provides a comprehensive approach to relationship validation that spans domain boundaries while maintaining robust security and consistency guarantees. 