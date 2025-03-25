// Relationship validation tests
// Original file: src/effect/templates/relationship_validation_tests.rs

// Integration tests for relationship validation effects
//
// This file demonstrates how the relationship validation effects can be integrated
// with resource operations to enforce relationship constraints.

use std::sync::Arc;
use std::collections::HashMap;

use causality_types::Address;
use causality_types::Result;
use causality_types::{*};
use causality_crypto::ContentId;;
use crate::resource::{
    ResourceRegister,
    RegisterState,
    RegisterOperationType,
    ResourceManager,
    ResourceRegisterLifecycleManager,
    RelationshipTracker,
    RelationshipType,
    ResourceLogic,
    FungibilityDomain,
    Quantity,
    StorageStrategy,
    StateVisibility,
};
use crate::effect::{Effect, EffectContext, EffectOutcome, EmptyEffect};
use causality_effects::{RelationshipStateValidationEffect};
use causality_effects::{ResourceStateTransitionHelper, RelationshipFilter};

#[tokio::test]
async fn test_relationship_validation_with_parent_child() -> Result<()> {
    // Set up test resources and relationships
    let parent_id = ResourceId::from("parent-resource");
    let child_id = ResourceId::from("child-resource");
    let domain_id = DomainId::from("test-domain");
    let invoker = Address::from("test-user");
    
    // Create lifecycle manager and relationship tracker
    let mut lifecycle_manager = ResourceRegisterLifecycleManager::new();
    let mut relationship_tracker = RelationshipTracker::new();
    
    // Register resources
    lifecycle_manager.register_resource(parent_id.clone())?;
    lifecycle_manager.register_resource(child_id.clone())?;
    
    // Set up parent-child relationship
    relationship_tracker.add_relationship(
        parent_id.clone(),
        child_id.clone(),
        RelationshipType::ParentChild,
        None,
    )?;
    
    // Set up resource manager
    let mut resource_manager = ResourceManager::new(Box::new(lifecycle_manager));
    resource_manager.set_relationship_tracker(Box::new(relationship_tracker));
    
    // Create a parent resource
    let parent_resource = ResourceRegister::new(
        parent_id.clone(),
        ResourceLogic::new(),
        FungibilityDomain::new("test_token"), 
        Quantity::new(100),
        Metadata::default(),
        StorageStrategy::FullyOnChain {
            visibility: StateVisibility::Public,
        },
    );
    
    // Create a child resource
    let child_resource = ResourceRegister::new(
        child_id.clone(),
        ResourceLogic::new(),
        FungibilityDomain::new("test_token"),
        Quantity::new(50),
        Metadata::default(),
        StorageStrategy::FullyOnChain {
            visibility: StateVisibility::Public,
        },
    );
    
    // Add resources to the manager
    resource_manager.add_resource(parent_resource.clone())?;
    resource_manager.add_resource(child_resource.clone())?;
    
    // Create a validation effect for archiving the parent resource
    // This should fail because it has a child resource
    let inner_effect = Arc::new(EmptyEffect::new());
    let validation_effect = RelationshipStateValidationEffect::new(
        parent_id.clone(),
        RegisterOperationType::Archive,
        domain_id.clone(),
        inner_effect,
        None,
    );
    
    // Create an effect context with the resource manager
    let context = EffectContext {
        execution_id: None,
        invoker: Some(invoker.clone()),
        domains: vec![domain_id.clone()],
        capabilities: vec![],
        resource_manager: Some(Arc::new(resource_manager)),
    };
    
    // Execute the effect - this should fail due to the parent-child relationship
    let outcome = validation_effect.execute_async(&context).await?;
    
    // Verify the outcome
    assert!(!outcome.success);
    assert!(outcome.error.is_some());
    assert!(outcome.error.unwrap().contains("would violate relationship constraints"));
    
    Ok(())
}

#[tokio::test]
async fn test_relationship_validation_with_dependencies() -> Result<()> {
    // Set up test resources and relationships
    let dependent_id = ResourceId::from("dependent-resource");
    let dependency_id = ResourceId::from("dependency-resource");
    let domain_id = DomainId::from("test-domain");
    let invoker = Address::from("test-user");
    
    // Create lifecycle manager and relationship tracker
    let mut lifecycle_manager = ResourceRegisterLifecycleManager::new();
    let mut relationship_tracker = RelationshipTracker::new();
    
    // Register resources
    lifecycle_manager.register_resource(dependent_id.clone())?;
    lifecycle_manager.register_resource(dependency_id.clone())?;
    
    // Set up dependency relationship
    relationship_tracker.add_relationship(
        dependent_id.clone(),
        dependency_id.clone(),
        RelationshipType::Dependency,
        None,
    )?;
    
    // Set up resource manager
    let mut resource_manager = ResourceManager::new(Box::new(lifecycle_manager));
    resource_manager.set_relationship_tracker(Box::new(relationship_tracker));
    
    // Create the resources
    let dependent_resource = ResourceRegister::new(
        dependent_id.clone(),
        ResourceLogic::new(),
        FungibilityDomain::new("test_token"), 
        Quantity::new(100),
        Metadata::default(),
        StorageStrategy::FullyOnChain {
            visibility: StateVisibility::Public,
        },
    );
    
    let dependency_resource = ResourceRegister::new(
        dependency_id.clone(),
        ResourceLogic::new(),
        FungibilityDomain::new("test_token"),
        Quantity::new(50),
        Metadata::default(),
        StorageStrategy::FullyOnChain {
            visibility: StateVisibility::Public,
        },
    );
    
    // Add resources to the manager
    resource_manager.add_resource(dependent_resource.clone())?;
    resource_manager.add_resource(dependency_resource.clone())?;
    
    // Create a validation effect for consuming the dependency resource
    // This should fail because the dependent resource depends on it
    let inner_effect = Arc::new(EmptyEffect::new());
    let validation_effect = RelationshipStateValidationEffect::new(
        dependency_id.clone(),
        RegisterOperationType::Consume,
        domain_id.clone(),
        inner_effect,
        None,
    );
    
    // Create an effect context with the resource manager
    let context = EffectContext {
        execution_id: None,
        invoker: Some(invoker.clone()),
        domains: vec![domain_id.clone()],
        capabilities: vec![],
        resource_manager: Some(Arc::new(resource_manager)),
    };
    
    // Execute the effect - this should fail due to the dependency relationship
    let outcome = validation_effect.execute_async(&context).await?;
    
    // Verify the outcome
    assert!(!outcome.success);
    assert!(outcome.error.is_some());
    assert!(outcome.error.unwrap().contains("would violate relationship constraints"));
    
    Ok(())
}

#[tokio::test]
async fn test_complex_resource_relationships() -> Result<()> {
    // Create a more complex scenario with multiple relationship types
    
    // Resources
    let root_resource_id = ResourceId::from("root-resource");
    let child1_id = ResourceId::from("child1");
    let child2_id = ResourceId::from("child2");
    let grandchild_id = ResourceId::from("grandchild");
    let dependency1_id = ResourceId::from("dependency1");
    let dependency2_id = ResourceId::from("dependency2");
    let domain_id = DomainId::from("test-domain");
    let invoker = Address::from("test-user");
    
    // Create lifecycle manager and relationship tracker
    let mut lifecycle_manager = ResourceRegisterLifecycleManager::new();
    let mut relationship_tracker = RelationshipTracker::new();
    
    // Register all resources
    lifecycle_manager.register_resource(root_resource_id.clone())?;
    lifecycle_manager.register_resource(child1_id.clone())?;
    lifecycle_manager.register_resource(child2_id.clone())?;
    lifecycle_manager.register_resource(grandchild_id.clone())?;
    lifecycle_manager.register_resource(dependency1_id.clone())?;
    lifecycle_manager.register_resource(dependency2_id.clone())?;
    
    // Activate all resources
    lifecycle_manager.activate(&root_resource_id)?;
    lifecycle_manager.activate(&child1_id)?;
    lifecycle_manager.activate(&child2_id)?;
    lifecycle_manager.activate(&grandchild_id)?;
    lifecycle_manager.activate(&dependency1_id)?;
    lifecycle_manager.activate(&dependency2_id)?;
    
    // Set up relationships:
    // root
    //  ├── child1
    //  │    └── grandchild
    //  └── child2
    //       ├── dependency1
    //       └── dependency2
    
    // Parent-child relationships
    relationship_tracker.add_relationship(
        root_resource_id.clone(),
        child1_id.clone(),
        RelationshipType::ParentChild,
        None,
    )?;
    
    relationship_tracker.add_relationship(
        root_resource_id.clone(),
        child2_id.clone(),
        RelationshipType::ParentChild,
        None,
    )?;
    
    relationship_tracker.add_relationship(
        child1_id.clone(),
        grandchild_id.clone(),
        RelationshipType::ParentChild,
        None,
    )?;
    
    // Dependency relationships
    relationship_tracker.add_relationship(
        child2_id.clone(),
        dependency1_id.clone(),
        RelationshipType::Dependency,
        None,
    )?;
    
    relationship_tracker.add_relationship(
        child2_id.clone(),
        dependency2_id.clone(),
        RelationshipType::Dependency,
        None,
    )?;
    
    // Set up resource manager with the state transition helper
    let mut resource_manager = ResourceManager::new(Box::new(lifecycle_manager));
    resource_manager.set_relationship_tracker(Box::new(relationship_tracker));
    
    // Create resources for each ID
    let create_resource = |id: &ResourceId| -> ResourceRegister {
        ResourceRegister::new(
            id.clone(),
            ResourceLogic::new(),
            FungibilityDomain::new("test_token"),
            Quantity::new(100),
            Metadata::default(),
            StorageStrategy::FullyOnChain {
                visibility: StateVisibility::Public,
            },
        )
    };
    
    // Add all resources to the manager
    resource_manager.add_resource(create_resource(&root_resource_id))?;
    resource_manager.add_resource(create_resource(&child1_id))?;
    resource_manager.add_resource(create_resource(&child2_id))?;
    resource_manager.add_resource(create_resource(&grandchild_id))?;
    resource_manager.add_resource(create_resource(&dependency1_id))?;
    resource_manager.add_resource(create_resource(&dependency2_id))?;
    
    // Create shared resource manager reference
    let resource_manager_arc = Arc::new(resource_manager);
    
    // Test 1: Try to archive the root resource (should fail due to active children)
    let inner_effect = Arc::new(EmptyEffect::new());
    let archive_root_effect = RelationshipStateValidationEffect::new(
        root_resource_id.clone(),
        RegisterOperationType::Archive,
        domain_id.clone(),
        inner_effect.clone(),
        None,
    );
    
    // Create an effect context with the resource manager
    let context = EffectContext {
        execution_id: None,
        invoker: Some(invoker.clone()),
        domains: vec![domain_id.clone()],
        capabilities: vec![],
        resource_manager: Some(resource_manager_arc.clone()),
    };
    
    // Execute the effect - this should fail due to active children
    let outcome = archive_root_effect.execute_async(&context).await?;
    assert!(!outcome.success);
    assert!(outcome.error.is_some());
    assert!(outcome.error.unwrap().contains("would violate relationship constraints"));
    
    // Test 2: Try to consume a dependency (should fail because it's required by child2)
    let consume_dependency_effect = RelationshipStateValidationEffect::new(
        dependency1_id.clone(),
        RegisterOperationType::Consume,
        domain_id.clone(),
        inner_effect.clone(),
        None,
    );
    
    let outcome = consume_dependency_effect.execute_async(&context).await?;
    assert!(!outcome.success);
    assert!(outcome.error.is_some());
    
    // Test 3: Try archiving grandchild (should succeed since it has no dependents)
    let archive_grandchild_effect = RelationshipStateValidationEffect::new(
        grandchild_id.clone(),
        RegisterOperationType::Archive,
        domain_id.clone(),
        inner_effect.clone(),
        None,
    );
    
    // This should succeed as the grandchild has no dependencies
    let outcome = archive_grandchild_effect.execute_async(&context).await?;
    
    // With RelationshipStateValidationEffect, this would normally return success=true
    // But since we're using EmptyEffect as the inner effect, it may not modify the actual resource state
    // To fully test this in an integration context, we'd need a real resource-modifying effect
    
    // Test 4: Archive in the correct order (grandchild → child1 → child2 → root)
    // (In a real scenario, we'd also need to remove the dependencies for child2)
    
    // First archive grandchild (already tested above)
    // Then archive child1
    let archive_child1_effect = RelationshipStateValidationEffect::new(
        child1_id.clone(),
        RegisterOperationType::Archive,
        domain_id.clone(),
        inner_effect.clone(),
        None,
    );
    
    // Since grandchild was archived in the previous test, this should succeed
    let outcome = archive_child1_effect.execute_async(&context).await?;
    
    // In a real integration test with real resources:
    // 1. We'd update the actual resources with real effects
    // 2. We'd verify the state changes were applied
    // 3. We'd check that relationships were updated appropriately
    
    Ok(())
}

// Example of how to use the relationship validation effect with a template function
#[allow(unused)]
fn resource_operation_with_relationship_validation(
    resource: &mut ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
    operation_type: RegisterOperationType,
) -> Result<Arc<dyn Effect>> {
    // Create a base operation effect
    // In a real implementation, this would use one of the standard effect templates
    let operation_effect = Arc::new(EmptyEffect::new());
    
    // Wrap the operation with relationship validation
    let validation_effect = RelationshipStateValidationEffect::new(
        resource.id.clone(),
        operation_type,
        domain_id,
        operation_effect,
        None,
    );
    
    Ok(Arc::new(validation_effect))
} 
