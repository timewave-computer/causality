# Effect Templates

This document describes effect templates in the Causality system, which provide standardized patterns for common resource operations within the unified operation model.

## Core Concepts

### Effect Templates

**Effect Templates** are reusable patterns that encapsulate the logic for common operations on resources. They provide:

1. **Standardization**: Consistent implementation of common operations
2. **Validation**: Built-in validation of operations against constraints
3. **Integration**: Seamless integration with the resource lifecycle manager
4. **Abstraction**: Hiding implementation details behind a clean API

Effect templates bridge the gap between abstract operations (what to do) and concrete implementations (how to do it), while ensuring that all operations adhere to the system's constraints and validation rules.

### Integration with the Operation Model

Effect templates work with the unified operation model by:

1. **Creating Operations**: Helping construct valid operations with proper context
2. **Validation**: Ensuring operations respect resource states and capabilities
3. **Transformation**: Converting between abstract and concrete representations
4. **Execution**: Handling the detailed execution logic across contexts

## Template Types

The system provides several categories of effect templates:

### Resource Lifecycle Templates

```rust
/// Create a new resource
struct CreateResourceTemplate;

/// Update an existing resource
struct UpdateResourceTemplate;

/// Lock a resource temporarily
struct LockResourceTemplate;

/// Unlock a previously locked resource
struct UnlockResourceTemplate;

/// Freeze a resource (more restrictive than locking)
struct FreezeResourceTemplate;

/// Unfreeze a previously frozen resource
struct UnfreezeResourceTemplate;

/// Consume a resource (terminal operation)
struct ConsumeResourceTemplate;

/// Archive a resource for historical reference
struct ArchiveResourceTemplate;
```

### Resource Transfer Templates

```rust
/// Transfer ownership of a resource
struct TransferResourceTemplate;

/// Transfer a partial quantity from a fungible resource
struct TransferQuantityTemplate;

/// Escrow a resource to a third party
struct EscrowResourceTemplate;

/// Release a resource from escrow
struct ReleaseEscrowTemplate;
```

### Resource Relationship Templates

```rust
/// Create a relationship between resources
struct CreateRelationshipTemplate;

/// Update an existing relationship
struct UpdateRelationshipTemplate;

/// Remove a relationship between resources
struct RemoveRelationshipTemplate;
```

### Cross-Domain Templates

```rust
/// Send a resource to another domain
struct SendCrossDomainTemplate;

/// Receive a resource from another domain
struct ReceiveCrossDomainTemplate;

/// Synchronize a resource state across domains
struct SynchronizeResourceTemplate;
```

## Template Implementation

Each template implements a common interface:

```rust
/// Common interface for effect templates
trait EffectTemplate {
    /// Create an operation from the template
    fn create_operation(&self, params: TemplateParams) -> Result<Operation>;
    
    /// Validate that an operation can be applied
    fn validate(&self, operation: &Operation) -> Result<ValidationResult>;
    
    /// Transform an abstract operation to a concrete implementation
    fn transform(&self, operation: &Operation) -> Result<Operation>;
    
    /// Execute the operation
    fn execute(&self, operation: &Operation, context: &ExecutionContext) -> Result<OperationResult>;
}
```

## Integration with Resources

Effect templates integrate with the resource system through:

1. **ResourceRegister**: Templates create operations that work with the unified ResourceRegister
2. **Lifecycle Manager**: Templates respect and update resource lifecycle states
3. **Relationship Tracker**: Templates maintain proper relationships between resources
4. **Capability System**: Templates validate operations against required capabilities

## Usage Examples

### Basic Resource Creation

```rust
// Create a resource using the CreateResourceTemplate
let create_template = CreateResourceTemplate::new();

// Create operation parameters
let params = TemplateParams::new()
    .with_value("resource_id", "resource1")
    .with_value("properties", ResourceProperties::new()
        .with_fungibility_domain("token")
        .with_quantity(100))
    .with_value("logic", FungibleTokenLogic::new())
    .with_value("storage_strategy", StorageStrategy::FullyOnChain { 
        visibility: StateVisibility::Public 
    })
    .with_value("invoker", invoker.clone());

// Create the operation
let operation = create_template.create_operation(params)?;

// Validate and execute
let validation_result = create_template.validate(&operation)?;
if validation_result.is_valid {
    let result = create_template.execute(&operation, &context)?;
    let resource_id = result.get_value::<ResourceId>("resource_id")?;
    // Use the resource_id
}
```

### Resource Transfer with Capabilities

```rust
// Create a transfer template
let transfer_template = TransferResourceTemplate::new();

// Set up parameters with capabilities
let params = TemplateParams::new()
    .with_value("source_resource", source_resource.clone())
    .with_value("destination_resource", destination_resource.clone())
    .with_value("invoker", invoker.clone())
    .with_value("capabilities", vec![
        Capability::new(
            Rights::from([Right::Transfer]),
            Targets::Resource(source_resource.id.clone()),
            Some(CapabilityConstraints::new()
                .with_max_quantity(amount)
                .with_expiration(time::now() + Duration::hours(1)))
        )
    ]);

// Create and execute the operation
let operation = transfer_template.create_operation(params)?;
let validation_result = transfer_template.validate(&operation)?;
if validation_result.is_valid {
    let result = transfer_template.execute(&operation, &context)?;
    // Process the result
}
```

### Cross-Domain Resource Transfer

```rust
// Create a cross-domain transfer template
let cross_domain_template = SendCrossDomainTemplate::new();

// Set up parameters including temporal facts
let params = TemplateParams::new()
    .with_value("source_resource", source_resource.clone())
    .with_value("destination_domain", destination_domain)
    .with_value("recipient", recipient_address)
    .with_value("invoker", invoker.clone())
    .with_value("temporal_facts", vec![source_domain_fact.id.clone()])
    .with_value("capabilities", vec![
        Capability::new(
            Rights::from([Right::Transfer]),
            Targets::Resource(source_resource.id.clone()),
            None
        )
    ]);

// Create the operation
let operation = cross_domain_template.create_operation(params)?;

// Validate and execute
let validation_result = cross_domain_template.validate(&operation)?;
if validation_result.is_valid {
    let result = cross_domain_template.execute(&operation, &context)?;
    let proof = result.get_value::<CrossDomainProof>("proof")?;
    // Use the proof for the receiving chain
}
```

## Template Constraints

Effect templates enforce various constraints to ensure operations remain valid:

### Lifecycle State Constraints

```rust
// Check if a resource is in a valid state for the operation
fn check_resource_state(resource: &ResourceRegister, allowed_states: &[RegisterState]) -> Result<bool> {
    let current_state = resource.state();
    Ok(allowed_states.contains(&current_state))
}

// Example validation in a template
fn validate_unlock_operation(&self, operation: &Operation) -> Result<ValidationResult> {
    let resource = operation.inputs()[0].as_resource()?;
    
    // Only locked resources can be unlocked
    if !check_resource_state(resource, &[RegisterState::Locked])? {
        return Ok(ValidationResult::invalid("Resource must be in Locked state to unlock"));
    }
    
    // Validate other constraints...
    Ok(ValidationResult::valid())
}
```

### Capability Constraints

```rust
// Validate capabilities for a transfer operation
fn validate_transfer_capabilities(&self, operation: &Operation) -> Result<ValidationResult> {
    let capabilities = operation.authorization().capabilities();
    let resource = operation.inputs()[0].as_resource()?;
    
    // Check for Transfer right
    let has_transfer_right = capabilities.iter().any(|cap| {
        // Check if capability grants Transfer right
        cap.rights().contains(&Right::Transfer) &&
        // Check if capability targets this resource
        match cap.targets() {
            Targets::Resource(id) => id == resource.id(),
            Targets::ResourceGroup(group_id) => resource_group_contains(group_id, resource.id()),
            Targets::ResourceType(type_id) => resource.is_type(type_id),
            Targets::Domain(domain_id) => resource.domain_id() == domain_id,
            _ => false,
        }
    });
    
    if !has_transfer_right {
        return Ok(ValidationResult::invalid("Missing Transfer capability for this resource"));
    }
    
    // Validate other capability constraints...
    Ok(ValidationResult::valid())
}
```

### Relationship Constraints

```rust
// Validate relationships for a consume operation
fn validate_consumption_relationships(&self, operation: &Operation) -> Result<ValidationResult> {
    let resource = operation.inputs()[0].as_resource()?;
    let relationship_tracker = self.context.get_relationship_tracker()?;
    
    // Check if resource has dependent resources
    let dependents = relationship_tracker.get_relationships(
        resource.id(),
        Some(RelationshipType::Dependency),
        RelationshipDirection::Incoming
    )?;
    
    if !dependents.is_empty() {
        return Ok(ValidationResult::invalid(
            "Cannot consume a resource that has dependent resources"
        ));
    }
    
    // Validate other relationship constraints...
    Ok(ValidationResult::valid())
}
```

### Temporal Constraints

```rust
// Validate temporal dependencies for an operation
fn validate_temporal_facts(&self, operation: &Operation) -> Result<ValidationResult> {
    let temporal_dependencies = operation.temporal_dependencies();
    
    // Skip validation if no temporal dependencies
    if temporal_dependencies.is_empty() {
        return Ok(ValidationResult::valid());
    }
    
    let fact_validator = self.context.get_fact_validator()?;
    
    // Check each temporal fact
    for fact_id in temporal_dependencies {
        if !fact_validator.fact_exists(&fact_id)? {
            return Ok(ValidationResult::invalid(
                format!("Required temporal fact {} does not exist", fact_id)
            ));
        }
        
        // Check if fact is in the right temporal order
        if !fact_validator.is_temporally_valid(&fact_id)? {
            return Ok(ValidationResult::invalid(
                format!("Temporal fact {} violates causal ordering", fact_id)
            ));
        }
    }
    
    Ok(ValidationResult::valid())
}
```

## Effect Template Registry

The system provides a registry of available templates:

```rust
// Get a template from the registry
let create_template = template_registry.get_template(TemplateType::CreateResource)?;
let transfer_template = template_registry.get_template(TemplateType::TransferResource)?;

// Or use the specialized getters
let create_template = template_registry.get_create_resource_template()?;
let transfer_template = template_registry.get_transfer_resource_template()?;
```

## Creating Custom Templates

Custom templates can be created for domain-specific operations:

```rust
// Create a custom template for a specialized resource operation
struct CustomResourceOperationTemplate {
    // Template-specific fields
}

impl EffectTemplate for CustomResourceOperationTemplate {
    fn create_operation(&self, params: TemplateParams) -> Result<Operation> {
        // Custom implementation
    }
    
    fn validate(&self, operation: &Operation) -> Result<ValidationResult> {
        // Custom validation
    }
    
    fn transform(&self, operation: &Operation) -> Result<Operation> {
        // Custom transformation
    }
    
    fn execute(&self, operation: &Operation, context: &ExecutionContext) -> Result<OperationResult> {
        // Custom execution
    }
}

// Register the custom template
template_registry.register_template(
    "custom.resource.operation",
    Box::new(CustomResourceOperationTemplate::new())
)?;
```

## Best Practices

1. **Use Provided Templates**: Prefer using the standard templates for common operations.

2. **Validate Before Execution**: Always validate operations before executing them.

3. **Check All Constraints**: Consider lifecycle, capability, relationship, and temporal constraints.

4. **Handle Errors Gracefully**: Provide clear error messages when validation fails.

5. **Test With Edge Cases**: Verify templates work correctly with boundary conditions.

6. **Respect Template Pipeline**: Follow the create → validate → execute pattern.

7. **Custom Templates**: Create custom templates for specialized operations that need specific logic.

8. **Document Templates**: Clearly document what each template does and its requirements.

9. **Compose Templates**: Build complex operations by composing multiple templates.

10. **Maintain Atomicity**: Ensure templates maintain atomicity for related operations.

## Implementation Status

Effect templates are fully implemented in the Causality system:

- ✅ Core template infrastructure
- ✅ Resource lifecycle templates
- ✅ Resource transfer templates
- ✅ Relationship templates
- ✅ Cross-domain templates
- ✅ Template registry
- ✅ Integration with the unified operation model

## Future Enhancements

1. **Template Composition**: More powerful composition of templates
2. **Conditional Templates**: Templates with conditional logic
3. **Template Optimization**: Performance optimization for high-throughput scenarios
4. **Template Versioning**: Support for template versioning and upgrades
5. **Template Analytics**: Telemetry and analytics for template usage 