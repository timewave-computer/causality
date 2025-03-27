# Resource Operation Patterns Guide

This guide explains common patterns for working with resources in the Causality system, focusing on operations, effects, and best practices.

## Overview

Resources in Causality represent state and functionality encapsulated behind a well-defined interface. Resource operations provide a structured way to interact with resources, ensuring proper authorization, validation, and effect handling.

## Key Concepts

### Resources

A resource in Causality has the following characteristics:

- Identified by a unique `ResourceId`
- Associated with a specific domain (`DomainId`)
- Protected by capabilities
- Manipulated through operations
- Connected to facts for provenance and causal ordering

### Operations

An operation is a well-defined action performed on a resource:

- Has a specific action type (e.g., "read", "write", "update")
- Targets one or more resources
- Requires specific capabilities
- May produce effects
- Results in an outcome (success or failure)

### Effects

Effects represent the side effects of operations:

- Encapsulate changes to system state
- Can be composed and validated
- Provide a mechanism for authorization and validation
- Enable operations across domain boundaries

## Basic Resource Operation Pattern

The most common pattern for resource operations follows this structure:

```rust
use causality_core::{
    resource::{ResourceId, ResourceReference},
    effect::{Effect, EffectContext, EffectRegistry},
    resource::agent::{Authorization, Agent, Operation},
};

async fn perform_resource_operation(
    resource_id: ResourceId,
    domain_id: DomainId,
    action: &str,
    parameters: &[u8],
    agent: &Agent,
    registry: &EffectRegistry,
) -> Result<EffectOutcome, Error> {
    // 1. Create an operation
    let operation = Operation::new()
        .in_domain(domain_id.clone())
        .target_resource(resource_id.clone())
        .with_action(action)
        .with_parameters(parameters);
    
    // 2. Verify the operation against the agent's capabilities
    let capability_manager = agent.capability_manager();
    let is_authorized = capability_manager
        .verify_operation(&operation)
        .await?;
    
    if !is_authorized {
        return Err(Error::Unauthorized);
    }
    
    // 3. Convert the operation to an effect
    let effect = operation.to_effect();
    
    // 4. Create an effect context with the agent's authorization
    let authorization = agent.create_authorization()?;
    let context = EffectContext::new()
        .with_authorization(authorization);
    
    // 5. Execute the effect
    let outcome = registry.execute(&effect, &context).await?;
    
    // 6. Return the outcome
    Ok(outcome)
}
```

## Common Resource Operation Patterns

### CRUD Operations

Create, Read, Update, Delete (CRUD) operations form the basis of resource management:

```rust
// Create a resource
async fn create_resource(
    resource_type: &str,
    content: &[u8],
    domain_id: &DomainId,
    agent: &Agent,
    registry: &EffectRegistry,
) -> Result<ResourceId, Error> {
    // Generate a new resource ID
    let resource_id = ResourceId::new_random();
    
    // Create an operation
    let operation = Operation::new()
        .in_domain(domain_id.clone())
        .target_resource(resource_id.clone())
        .with_action("create")
        .with_resource_type(resource_type)
        .with_content(content);
    
    // Convert to effect and execute
    let effect = operation.to_effect();
    let context = EffectContext::new()
        .with_authorization(agent.create_authorization()?);
    
    let outcome = registry.execute(&effect, &context).await?;
    
    // Extract the resource ID from the outcome
    let result = outcome.result();
    let created_id = result.get("resource_id")
        .ok_or(Error::MissingResourceId)?;
    
    Ok(ResourceId::from_str(created_id)?)
}

// Read a resource
async fn read_resource(
    resource_id: &ResourceId,
    domain_id: &DomainId,
    agent: &Agent,
    registry: &EffectRegistry,
) -> Result<Vec<u8>, Error> {
    let operation = Operation::new()
        .in_domain(domain_id.clone())
        .target_resource(resource_id.clone())
        .with_action("read");
    
    let effect = operation.to_effect();
    let context = EffectContext::new()
        .with_authorization(agent.create_authorization()?);
    
    let outcome = registry.execute(&effect, &context).await?;
    
    // Extract the content from the outcome
    let result = outcome.result();
    let content = result.get("content")
        .ok_or(Error::MissingContent)?;
    
    Ok(content.as_bytes().to_vec())
}

// Update a resource
async fn update_resource(
    resource_id: &ResourceId,
    domain_id: &DomainId,
    content: &[u8],
    agent: &Agent,
    registry: &EffectRegistry,
) -> Result<(), Error> {
    let operation = Operation::new()
        .in_domain(domain_id.clone())
        .target_resource(resource_id.clone())
        .with_action("update")
        .with_content(content);
    
    let effect = operation.to_effect();
    let context = EffectContext::new()
        .with_authorization(agent.create_authorization()?);
    
    let outcome = registry.execute(&effect, &context).await?;
    
    // Check if the operation was successful
    if outcome.is_success() {
        Ok(())
    } else {
        Err(Error::UpdateFailed)
    }
}

// Delete a resource
async fn delete_resource(
    resource_id: &ResourceId,
    domain_id: &DomainId,
    agent: &Agent,
    registry: &EffectRegistry,
) -> Result<(), Error> {
    let operation = Operation::new()
        .in_domain(domain_id.clone())
        .target_resource(resource_id.clone())
        .with_action("delete");
    
    let effect = operation.to_effect();
    let context = EffectContext::new()
        .with_authorization(agent.create_authorization()?);
    
    let outcome = registry.execute(&effect, &context).await?;
    
    // Check if the operation was successful
    if outcome.is_success() {
        Ok(())
    } else {
        Err(Error::DeleteFailed)
    }
}
```

### Resource Query Operations

Query operations allow for complex searches and filtering of resources:

```rust
// Query resources by type
async fn query_resources_by_type(
    resource_type: &str,
    domain_id: &DomainId,
    agent: &Agent,
    registry: &EffectRegistry,
) -> Result<Vec<ResourceId>, Error> {
    let operation = Operation::new()
        .in_domain(domain_id.clone())
        .with_action("query")
        .with_parameters(json!({
            "type": resource_type,
        }).to_string().as_bytes());
    
    let effect = operation.to_effect();
    let context = EffectContext::new()
        .with_authorization(agent.create_authorization()?);
    
    let outcome = registry.execute(&effect, &context).await?;
    
    // Extract the resources from the outcome
    let result = outcome.result();
    let resources = result.get("resources")
        .ok_or(Error::MissingResources)?
        .as_array()
        .ok_or(Error::InvalidFormat)?;
    
    // Convert the resources to ResourceIds
    let mut resource_ids = Vec::new();
    for resource in resources {
        let id_str = resource.as_str().ok_or(Error::InvalidFormat)?;
        resource_ids.push(ResourceId::from_str(id_str)?);
    }
    
    Ok(resource_ids)
}
```

### Batch Operations

Batch operations allow multiple resources to be operated on atomically:

```rust
// Batch update resources
async fn batch_update_resources(
    updates: &[(ResourceId, Vec<u8>)],
    domain_id: &DomainId,
    agent: &Agent,
    registry: &EffectRegistry,
) -> Result<(), Error> {
    // Create an effect builder for a composite effect
    let mut builder = EffectBuilder::new("batch_update");
    
    // Add an effect for each resource update
    for (resource_id, content) in updates {
        let operation = Operation::new()
            .in_domain(domain_id.clone())
            .target_resource(resource_id.clone())
            .with_action("update")
            .with_content(content);
        
        let effect = operation.to_effect();
        builder.add_effect(effect);
    }
    
    // Build the composite effect
    let composite_effect = builder.build();
    
    // Execute the composite effect
    let context = EffectContext::new()
        .with_authorization(agent.create_authorization()?);
    
    let outcome = registry.execute(&composite_effect, &context).await?;
    
    // Check if the operation was successful
    if outcome.is_success() {
        Ok(())
    } else {
        Err(Error::BatchUpdateFailed)
    }
}
```

### Resource Relationship Operations

Operations that establish or query relationships between resources:

```rust
// Link two resources
async fn link_resources(
    source_id: &ResourceId,
    target_id: &ResourceId,
    relationship_type: &str,
    domain_id: &DomainId,
    agent: &Agent,
    registry: &EffectRegistry,
) -> Result<(), Error> {
    let operation = Operation::new()
        .in_domain(domain_id.clone())
        .target_resource(source_id.clone())
        .with_action("link")
        .with_parameters(json!({
            "target_id": target_id.to_string(),
            "relationship_type": relationship_type,
        }).to_string().as_bytes());
    
    let effect = operation.to_effect();
    let context = EffectContext::new()
        .with_authorization(agent.create_authorization()?);
    
    let outcome = registry.execute(&effect, &context).await?;
    
    if outcome.is_success() {
        Ok(())
    } else {
        Err(Error::LinkFailed)
    }
}

// Query related resources
async fn get_related_resources(
    resource_id: &ResourceId,
    relationship_type: &str,
    domain_id: &DomainId,
    agent: &Agent,
    registry: &EffectRegistry,
) -> Result<Vec<ResourceId>, Error> {
    let operation = Operation::new()
        .in_domain(domain_id.clone())
        .target_resource(resource_id.clone())
        .with_action("get_related")
        .with_parameters(json!({
            "relationship_type": relationship_type,
        }).to_string().as_bytes());
    
    let effect = operation.to_effect();
    let context = EffectContext::new()
        .with_authorization(agent.create_authorization()?);
    
    let outcome = registry.execute(&effect, &context).await?;
    
    // Extract the related resources from the outcome
    let result = outcome.result();
    let resources = result.get("related_resources")
        .ok_or(Error::MissingResources)?
        .as_array()
        .ok_or(Error::InvalidFormat)?;
    
    // Convert to ResourceIds
    let mut resource_ids = Vec::new();
    for resource in resources {
        let id_str = resource.as_str().ok_or(Error::InvalidFormat)?;
        resource_ids.push(ResourceId::from_str(id_str)?);
    }
    
    Ok(resource_ids)
}
```

### Resource Version Operations

Operations for managing resource versions and history:

```rust
// Create a new version of a resource
async fn create_resource_version(
    resource_id: &ResourceId,
    content: &[u8],
    domain_id: &DomainId,
    agent: &Agent,
    registry: &EffectRegistry,
) -> Result<VersionId, Error> {
    let operation = Operation::new()
        .in_domain(domain_id.clone())
        .target_resource(resource_id.clone())
        .with_action("create_version")
        .with_content(content);
    
    let effect = operation.to_effect();
    let context = EffectContext::new()
        .with_authorization(agent.create_authorization()?);
    
    let outcome = registry.execute(&effect, &context).await?;
    
    // Extract the version ID from the outcome
    let result = outcome.result();
    let version_id = result.get("version_id")
        .ok_or(Error::MissingVersionId)?;
    
    Ok(VersionId::from_str(version_id)?)
}

// Get a specific version of a resource
async fn get_resource_version(
    resource_id: &ResourceId,
    version_id: &VersionId,
    domain_id: &DomainId,
    agent: &Agent,
    registry: &EffectRegistry,
) -> Result<Vec<u8>, Error> {
    let operation = Operation::new()
        .in_domain(domain_id.clone())
        .target_resource(resource_id.clone())
        .with_action("get_version")
        .with_parameters(json!({
            "version_id": version_id.to_string(),
        }).to_string().as_bytes());
    
    let effect = operation.to_effect();
    let context = EffectContext::new()
        .with_authorization(agent.create_authorization()?);
    
    let outcome = registry.execute(&effect, &context).await?;
    
    // Extract the content from the outcome
    let result = outcome.result();
    let content = result.get("content")
        .ok_or(Error::MissingContent)?;
    
    Ok(content.as_bytes().to_vec())
}
```

## Advanced Resource Operation Patterns

### Resource Operation with Time Effects

Combining resource operations with time effects:

```rust
use causality_core::time::integration::TimeEffectIntegration;

// Create a resource with time effects
async fn create_resource_with_time(
    resource_type: &str,
    content: &[u8],
    domain_id: &DomainId,
    agent: &Agent,
    registry: &EffectRegistry,
    time_integration: &TimeEffectIntegration,
) -> Result<ResourceId, Error> {
    // Generate a new resource ID
    let resource_id = ResourceId::new_random();
    
    // Create a resource creation operation
    let create_operation = Operation::new()
        .in_domain(domain_id.clone())
        .target_resource(resource_id.clone())
        .with_action("create")
        .with_resource_type(resource_type)
        .with_content(content);
    
    let create_effect = create_operation.to_effect();
    
    // Create a causal time effect
    let causal_effect = time_integration
        .create_causal_time_effect(
            domain_id.clone(),
            vec![], // No dependencies for a new resource
        )
        .await?;
    
    // Create a clock time effect
    let clock_effect = time_integration
        .create_clock_time_effect(
            domain_id.clone(),
        )
        .await?;
    
    // Create a composite effect
    let composite_effect = EffectBuilder::new("create_with_time")
        .add_effect(create_effect)
        .add_causal_time_effect(causal_effect)
        .add_clock_time_effect(clock_effect)
        .build();
    
    // Execute the composite effect
    let context = EffectContext::new()
        .with_authorization(agent.create_authorization()?);
    
    let outcome = registry.execute(&composite_effect, &context).await?;
    
    // Extract the resource ID from the outcome
    let result = outcome.result();
    let created_id = result.get("resource_id")
        .ok_or(Error::MissingResourceId)?;
    
    Ok(ResourceId::from_str(created_id)?)
}
```

### Resource Operation with Capability Verification

Explicitly verifying capabilities for an operation:

```rust
use causality_core::resource::agent::capability::{
    CapabilityManager, CapabilityVerification, CapabilityBundle,
};

// Perform a resource operation with explicit capability verification
async fn perform_verified_operation(
    resource_id: &ResourceId,
    action: &str,
    domain_id: &DomainId,
    agent: &Agent,
    registry: &EffectRegistry,
    capability_manager: &CapabilityManager,
) -> Result<EffectOutcome, Error> {
    // Create an operation
    let operation = Operation::new()
        .in_domain(domain_id.clone())
        .target_resource(resource_id.clone())
        .with_action(action);
    
    // Create a capability verifier
    let verifier = CapabilityVerification::new(domain_id.clone());
    
    // Check if the agent has the required capability
    let has_capability = verifier
        .verify_operation(&operation, capability_manager)
        .await?;
    
    if !has_capability {
        return Err(Error::Unauthorized);
    }
    
    // Convert to effect and execute
    let effect = operation.to_effect();
    let context = EffectContext::new()
        .with_authorization(agent.create_authorization()?)
        .with_capabilities(capability_manager.get_agent_capabilities(agent.id())?);
    
    let outcome = registry.execute(&effect, &context).await?;
    
    Ok(outcome)
}
```

### Resource Operation with Conditional Execution

Operations that only execute if certain conditions are met:

```rust
// Update a resource only if it matches a specified version
async fn conditional_update_resource(
    resource_id: &ResourceId,
    expected_version: &VersionId,
    content: &[u8],
    domain_id: &DomainId,
    agent: &Agent,
    registry: &EffectRegistry,
) -> Result<(), Error> {
    let operation = Operation::new()
        .in_domain(domain_id.clone())
        .target_resource(resource_id.clone())
        .with_action("conditional_update")
        .with_parameters(json!({
            "expected_version": expected_version.to_string(),
        }).to_string().as_bytes())
        .with_content(content);
    
    let effect = operation.to_effect();
    let context = EffectContext::new()
        .with_authorization(agent.create_authorization()?);
    
    let outcome = registry.execute(&effect, &context).await?;
    
    if outcome.is_success() {
        Ok(())
    } else {
        // Check for version mismatch
        let result = outcome.result();
        if let Some(error) = result.get("error") {
            if error == "version_mismatch" {
                return Err(Error::VersionMismatch);
            }
        }
        
        Err(Error::UpdateFailed)
    }
}
```

## Best Practices for Resource Operations

### 1. Use Structured Operations

Always use the `Operation` struct to create well-defined operations:

```rust
// Good: Structured operation
let operation = Operation::new()
    .in_domain(domain_id.clone())
    .target_resource(resource_id.clone())
    .with_action("read");

// Bad: Ad-hoc effect creation
let effect = Effect::new("read_resource")
    .add_attribute("resource_id", resource_id.to_string());
```

### 2. Always Verify Capabilities

Ensure operations are authorized through capability verification:

```rust
// Verify the operation against capabilities
let is_authorized = capability_manager
    .verify_operation(&operation)
    .await?;

if !is_authorized {
    return Err(Error::Unauthorized);
}
```

### 3. Use Composite Effects for Complex Operations

Compose multiple effects for complex operations:

```rust
// Create a composite effect for a complex operation
let composite_effect = EffectBuilder::new("complex_operation")
    .add_effect(first_effect)
    .add_effect(second_effect)
    .add_causal_time_effect(causal_effect)
    .build();
```

### 4. Include Time Effects for Proper Causality

Add time effects to establish causal ordering:

```rust
// Add a causal time effect
let effect_with_time = EffectBuilder::new("operation_with_time")
    .add_effect(operation.to_effect())
    .add_causal_time_effect(causal_effect)
    .build();
```

### 5. Use Domain-Specific Operations

Create domain-specific operation helpers:

```rust
// Domain-specific operation helper
async fn create_document(
    title: &str,
    content: &str,
    domain_id: &DomainId,
    agent: &Agent,
    registry: &EffectRegistry,
) -> Result<ResourceId, Error> {
    // Create document-specific parameters
    let parameters = json!({
        "title": title,
        "content": content,
        "created_at": chrono::Utc::now().to_rfc3339(),
    }).to_string().as_bytes().to_vec();
    
    // Use the generic create_resource function
    create_resource(
        "document",
        &parameters,
        domain_id,
        agent,
        registry,
    ).await
}
```

### 6. Handle Errors Properly

Provide detailed error information in operation outcomes:

```rust
// Return detailed error information
if resource_not_found {
    return EffectOutcome::error("resource_not_found")
        .with_attribute("resource_id", resource_id.to_string());
}
```

### 7. Log Operations for Audit Trails

Log operations for accountability and debugging:

```rust
// Log the operation before execution
log::info!(
    "Executing operation: domain={}, resource={}, action={}",
    operation.domain(),
    operation.resource(),
    operation.action(),
);
```

## Example: Complete Resource Management System

This example shows a complete resource management system:

```rust
use causality_core::{
    resource::{ResourceId, ResourceManager},
    effect::{Effect, EffectContext, EffectRegistry},
    resource::agent::{Agent, Operation, Authorization},
    time::integration::TimeEffectIntegration,
};

struct ResourceService {
    resource_manager: ResourceManager,
    effect_registry: EffectRegistry,
    time_integration: TimeEffectIntegration,
}

impl ResourceService {
    fn new(
        resource_manager: ResourceManager,
        effect_registry: EffectRegistry,
        time_integration: TimeEffectIntegration,
    ) -> Self {
        Self {
            resource_manager,
            effect_registry,
            time_integration,
        }
    }
    
    async fn create_resource(
        &self,
        resource_type: &str,
        content: &[u8],
        domain_id: &DomainId,
        agent: &Agent,
    ) -> Result<ResourceId, Error> {
        // Generate a new resource ID
        let resource_id = ResourceId::new_random();
        
        // Create an operation
        let operation = Operation::new()
            .in_domain(domain_id.clone())
            .target_resource(resource_id.clone())
            .with_action("create")
            .with_resource_type(resource_type)
            .with_content(content);
        
        // Create a causal time effect
        let causal_effect = self.time_integration
            .create_causal_time_effect(
                domain_id.clone(),
                vec![],
            )
            .await?;
        
        // Create a composite effect
        let effect = EffectBuilder::new("create_resource")
            .add_effect(operation.to_effect())
            .add_causal_time_effect(causal_effect)
            .build();
        
        // Execute the effect
        let context = EffectContext::new()
            .with_authorization(agent.create_authorization()?);
        
        let outcome = self.effect_registry.execute(&effect, &context).await?;
        
        // Extract the resource ID from the outcome
        let result = outcome.result();
        let created_id = result.get("resource_id")
            .ok_or(Error::MissingResourceId)?;
        
        Ok(ResourceId::from_str(created_id)?)
    }
    
    async fn read_resource(
        &self,
        resource_id: &ResourceId,
        domain_id: &DomainId,
        agent: &Agent,
    ) -> Result<Vec<u8>, Error> {
        let operation = Operation::new()
            .in_domain(domain_id.clone())
            .target_resource(resource_id.clone())
            .with_action("read");
        
        let effect = operation.to_effect();
        let context = EffectContext::new()
            .with_authorization(agent.create_authorization()?);
        
        let outcome = self.effect_registry.execute(&effect, &context).await?;
        
        // Extract the content from the outcome
        let result = outcome.result();
        let content = result.get("content")
            .ok_or(Error::MissingContent)?;
        
        Ok(content.as_bytes().to_vec())
    }
    
    async fn update_resource(
        &self,
        resource_id: &ResourceId,
        content: &[u8],
        domain_id: &DomainId,
        agent: &Agent,
    ) -> Result<(), Error> {
        // First, get the current version of the resource for causal dependency
        let resource = self.resource_manager.get_resource(resource_id).await?;
        let fact_id = resource.fact_id();
        
        // Create an update operation
        let operation = Operation::new()
            .in_domain(domain_id.clone())
            .target_resource(resource_id.clone())
            .with_action("update")
            .with_content(content);
        
        // Create a causal time effect with dependency on the current version
        let causal_effect = self.time_integration
            .create_causal_time_effect(
                domain_id.clone(),
                vec![fact_id.clone()],
            )
            .await?;
        
        // Create a composite effect
        let effect = EffectBuilder::new("update_resource")
            .add_effect(operation.to_effect())
            .add_causal_time_effect(causal_effect)
            .build();
        
        // Execute the effect
        let context = EffectContext::new()
            .with_authorization(agent.create_authorization()?);
        
        let outcome = self.effect_registry.execute(&effect, &context).await?;
        
        if outcome.is_success() {
            Ok(())
        } else {
            Err(Error::UpdateFailed)
        }
    }
    
    async fn delete_resource(
        &self,
        resource_id: &ResourceId,
        domain_id: &DomainId,
        agent: &Agent,
    ) -> Result<(), Error> {
        let operation = Operation::new()
            .in_domain(domain_id.clone())
            .target_resource(resource_id.clone())
            .with_action("delete");
        
        let effect = operation.to_effect();
        let context = EffectContext::new()
            .with_authorization(agent.create_authorization()?);
        
        let outcome = self.effect_registry.execute(&effect, &context).await?;
        
        if outcome.is_success() {
            Ok(())
        } else {
            Err(Error::DeleteFailed)
        }
    }
    
    async fn query_resources(
        &self,
        query: &ResourceQuery,
        domain_id: &DomainId,
        agent: &Agent,
    ) -> Result<Vec<ResourceId>, Error> {
        let query_params = serde_json::to_vec(query)?;
        
        let operation = Operation::new()
            .in_domain(domain_id.clone())
            .with_action("query")
            .with_parameters(&query_params);
        
        let effect = operation.to_effect();
        let context = EffectContext::new()
            .with_authorization(agent.create_authorization()?);
        
        let outcome = self.effect_registry.execute(&effect, &context).await?;
        
        // Extract resources from the outcome
        let result = outcome.result();
        let resources = result.get("resources")
            .ok_or(Error::MissingResources)?
            .as_array()
            .ok_or(Error::InvalidFormat)?;
        
        // Convert to ResourceIds
        let mut resource_ids = Vec::new();
        for resource in resources {
            let id_str = resource.as_str().ok_or(Error::InvalidFormat)?;
            resource_ids.push(ResourceId::from_str(id_str)?);
        }
        
        Ok(resource_ids)
    }
}
```

## Conclusion

Resource operations in Causality provide a structured way to interact with resources, ensuring proper authorization, validation, and effect handling. By following the patterns and best practices outlined in this guide, you can create robust resource management systems that maintain security, consistency, and causal ordering.

Key takeaways:
- Use structured operations via the `Operation` struct
- Always verify capabilities for operations
- Compose effects for complex operations
- Include time effects for proper causality
- Create domain-specific operation helpers
- Handle errors with detailed information
- Log operations for audit trails

By adhering to these patterns and best practices, you can create resource operations that are secure, maintainable, and scalable. 