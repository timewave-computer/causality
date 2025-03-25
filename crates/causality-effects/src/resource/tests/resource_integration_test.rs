// Integration tests for the resource management system
//
// This tests the core functionality of the resource management system,
// including:
// - Resource access control
// - Resource lifecycle management
// - Cross-domain resource locking
// - Resource dependency tracking
// - Implementation through effect interfaces

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use async_trait::async_trait;

use causality_common::identity::ContentId;
use causality_resource::interface::{
    ResourceState, ResourceAccessType, LockType, DependencyType, LockStatus,
    ResourceAccess, ResourceLifecycle, ResourceLocking, ResourceDependency,
    ResourceContext, ResourceAccessRecord, ResourceLockInfo, ResourceDependencyInfo,
    BasicResourceContext
};

use crate::effect::{EffectId, EffectRegistry, EffectContext};
use crate::resource::{
    access::ResourceAccessManager,
    lifecycle::EffectResourceLifecycle,
    locking::CrossDomainLockManager,
    dependency::ResourceDependencyManager,
    implementation::EffectResourceImplementation,
    implementation::create_effect_context
};

// Helper function to create a test resource ID
fn create_test_resource_id(name: &str) -> ContentId {
    ContentId::from_string(name).unwrap_or_else(|_| panic!("Failed to create content ID for {}", name))
}

#[tokio::test]
async fn test_resource_lifecycle_management() -> Result<()> {
    // Create the components
    let effect_registry = Arc::new(EffectRegistry::new());
    let access_manager = Arc::new(ResourceAccessManager::new());
    let lifecycle_manager = Arc::new(EffectResourceLifecycle::new());
    let lock_manager = Arc::new(CrossDomainLockManager::new());
    let dependency_manager = Arc::new(ResourceDependencyManager::new());
    
    // Create the resource implementation
    let resource_impl = EffectResourceImplementation::new(
        effect_registry.clone(),
        access_manager.clone(),
        lifecycle_manager.clone(),
        lock_manager.clone(),
        dependency_manager.clone()
    );
    
    // Create a resource context
    let effect_id = EffectId::new();
    let context = create_effect_context(effect_id.clone(), None);
    
    // Create a resource and verify it exists
    let resource_id = create_test_resource_id("test-resource-1");
    resource_impl.register_resource(
        resource_id.clone(),
        ResourceState::Created,
        &context
    ).await?;
    
    // Check that the resource exists
    assert!(resource_impl.resource_exists(&resource_id).await?);
    
    // Check the state
    let state = resource_impl.get_resource_state(&resource_id).await?;
    assert_eq!(state, ResourceState::Created);
    
    // Update the state
    resource_impl.update_resource_state(
        &resource_id,
        ResourceState::Active,
        &context
    ).await?;
    
    // Verify the update
    let state = resource_impl.get_resource_state(&resource_id).await?;
    assert_eq!(state, ResourceState::Active);
    
    Ok(())
}

#[tokio::test]
async fn test_resource_access_control() -> Result<()> {
    // Create the components
    let effect_registry = Arc::new(EffectRegistry::new());
    let access_manager = Arc::new(ResourceAccessManager::new());
    let lifecycle_manager = Arc::new(EffectResourceLifecycle::new());
    let lock_manager = Arc::new(CrossDomainLockManager::new());
    let dependency_manager = Arc::new(ResourceDependencyManager::new());
    
    // Create the resource implementation
    let resource_impl = EffectResourceImplementation::new(
        effect_registry.clone(),
        access_manager.clone(),
        lifecycle_manager.clone(),
        lock_manager.clone(),
        dependency_manager.clone()
    );
    
    // Create a resource context
    let effect_id = EffectId::new();
    let context = create_effect_context(effect_id.clone(), None);
    
    // Create a resource 
    let resource_id = create_test_resource_id("test-resource-2");
    resource_impl.register_resource(
        resource_id.clone(),
        ResourceState::Created,
        &context
    ).await?;
    
    // Check access allowed initially
    let allowed = resource_impl.is_access_allowed(
        &resource_id,
        ResourceAccessType::Read,
        &context
    ).await?;
    assert!(allowed, "Access should be allowed initially");
    
    // Record some accesses
    resource_impl.record_access(
        &resource_id,
        ResourceAccessType::Read,
        &context
    ).await?;
    
    resource_impl.record_access(
        &resource_id,
        ResourceAccessType::Write,
        &context
    ).await?;
    
    // Verify we can get the access records
    let accesses = resource_impl.get_resource_accesses(&resource_id).await?;
    assert_eq!(accesses.len(), 2, "Should have recorded 2 accesses");
    
    // Verify access types
    let mut read_count = 0;
    let mut write_count = 0;
    
    for access in accesses {
        match access.access_type {
            ResourceAccessType::Read => read_count += 1,
            ResourceAccessType::Write => write_count += 1,
            _ => {}
        }
    }
    
    assert_eq!(read_count, 1, "Should have 1 read access");
    assert_eq!(write_count, 1, "Should have 1 write access");
    
    Ok(())
}

#[tokio::test]
async fn test_resource_locking() -> Result<()> {
    // Create the components
    let effect_registry = Arc::new(EffectRegistry::new());
    let access_manager = Arc::new(ResourceAccessManager::new());
    let lifecycle_manager = Arc::new(EffectResourceLifecycle::new());
    let lock_manager = Arc::new(CrossDomainLockManager::new());
    let dependency_manager = Arc::new(ResourceDependencyManager::new());
    
    // Create the resource implementation
    let resource_impl = EffectResourceImplementation::new(
        effect_registry.clone(),
        access_manager.clone(),
        lifecycle_manager.clone(),
        lock_manager.clone(),
        dependency_manager.clone()
    );
    
    // Create a resource context
    let effect_id = EffectId::new();
    let context = create_effect_context(effect_id.clone(), None);
    
    // Create a resource 
    let resource_id = create_test_resource_id("test-resource-3");
    resource_impl.register_resource(
        resource_id.clone(),
        ResourceState::Active,
        &context
    ).await?;
    
    // Create a holder ID
    let holder_id = create_test_resource_id("test-holder-1");
    
    // Acquire a lock
    let lock_status = resource_impl.acquire_lock(
        &resource_id,
        LockType::Exclusive,
        &holder_id,
        None, // No timeout
        &context
    ).await?;
    
    assert_eq!(lock_status, LockStatus::Acquired, "Lock should be acquired");
    
    // Check if the resource is locked
    let is_locked = resource_impl.is_locked(&resource_id).await?;
    assert!(is_locked, "Resource should be locked");
    
    // Get lock info
    let lock_info = resource_impl.get_lock_info(&resource_id).await?;
    assert!(lock_info.is_some(), "Lock info should be available");
    
    if let Some(info) = lock_info {
        assert_eq!(info.resource_id, resource_id, "Lock should be for the right resource");
        assert_eq!(info.lock_type, LockType::Exclusive, "Lock should be exclusive");
        assert_eq!(info.holder_id, holder_id, "Lock should be held by the right holder");
    }
    
    // Try to acquire the same lock with another holder - should fail
    let another_holder = create_test_resource_id("test-holder-2");
    let lock_status2 = resource_impl.acquire_lock(
        &resource_id,
        LockType::Exclusive,
        &another_holder,
        None,
        &context
    ).await?;
    
    assert_eq!(lock_status2, LockStatus::Unavailable, "Lock should be unavailable");
    
    // Release the lock
    let released = resource_impl.release_lock(
        &resource_id,
        &holder_id,
        &context
    ).await?;
    
    assert!(released, "Lock should be released");
    
    // Verify the resource is unlocked
    let is_locked_after = resource_impl.is_locked(&resource_id).await?;
    assert!(!is_locked_after, "Resource should be unlocked after release");
    
    Ok(())
}

#[tokio::test]
async fn test_resource_dependencies() -> Result<()> {
    // Create the components
    let effect_registry = Arc::new(EffectRegistry::new());
    let access_manager = Arc::new(ResourceAccessManager::new());
    let lifecycle_manager = Arc::new(EffectResourceLifecycle::new());
    let lock_manager = Arc::new(CrossDomainLockManager::new());
    let dependency_manager = Arc::new(ResourceDependencyManager::new());
    
    // Create the resource implementation
    let resource_impl = EffectResourceImplementation::new(
        effect_registry.clone(),
        access_manager.clone(),
        lifecycle_manager.clone(),
        lock_manager.clone(),
        dependency_manager.clone()
    );
    
    // Create a resource context
    let effect_id = EffectId::new();
    let context = create_effect_context(effect_id.clone(), None);
    
    // Create two resources
    let source_id = create_test_resource_id("test-resource-source");
    let target_id = create_test_resource_id("test-resource-target");
    
    resource_impl.register_resource(
        source_id.clone(),
        ResourceState::Active,
        &context
    ).await?;
    
    resource_impl.register_resource(
        target_id.clone(),
        ResourceState::Active,
        &context
    ).await?;
    
    // Add a dependency
    resource_impl.add_dependency(
        &source_id,
        &target_id,
        DependencyType::Strong,
        &context
    ).await?;
    
    // Check if resource has dependencies
    let has_deps = resource_impl.has_dependencies(&source_id).await?;
    assert!(has_deps, "Source should have dependencies");
    
    // Check if resource has dependents
    let has_dependents = resource_impl.has_dependents(&target_id).await?;
    assert!(has_dependents, "Target should have dependents");
    
    // Get the dependencies
    let deps = resource_impl.get_dependencies(&source_id).await?;
    assert_eq!(deps.len(), 1, "Should have 1 dependency");
    
    if let Some(dep) = deps.first() {
        assert_eq!(dep.source_id, source_id, "Dependency source should match");
        assert_eq!(dep.target_id, target_id, "Dependency target should match");
        assert_eq!(dep.dependency_type, DependencyType::Strong, "Dependency type should match");
    }
    
    // Get the dependents
    let dependents = resource_impl.get_dependents(&target_id).await?;
    assert_eq!(dependents.len(), 1, "Should have 1 dependent");
    
    if let Some(dep) = dependents.first() {
        assert_eq!(dep.source_id, source_id, "Dependent source should match");
        assert_eq!(dep.target_id, target_id, "Dependent target should match");
    }
    
    // Remove the dependency
    let removed = resource_impl.remove_dependency(
        &source_id,
        &target_id,
        DependencyType::Strong,
        &context
    ).await?;
    
    assert!(removed, "Dependency should be removed");
    
    // Verify dependencies are gone
    let has_deps_after = resource_impl.has_dependencies(&source_id).await?;
    assert!(!has_deps_after, "Source should not have dependencies after removal");
    
    let has_dependents_after = resource_impl.has_dependents(&target_id).await?;
    assert!(!has_dependents_after, "Target should not have dependents after removal");
    
    Ok(())
}

#[tokio::test]
async fn test_integrated_resource_management() -> Result<()> {
    // Create the components
    let effect_registry = Arc::new(EffectRegistry::new());
    let access_manager = Arc::new(ResourceAccessManager::new());
    let lifecycle_manager = Arc::new(EffectResourceLifecycle::new());
    let lock_manager = Arc::new(CrossDomainLockManager::new());
    let dependency_manager = Arc::new(ResourceDependencyManager::new());
    
    // Create the resource implementation
    let resource_impl = EffectResourceImplementation::new(
        effect_registry.clone(),
        access_manager.clone(),
        lifecycle_manager.clone(),
        lock_manager.clone(),
        dependency_manager.clone()
    );
    
    // Create a resource context
    let effect_id = EffectId::new();
    let context = create_effect_context(effect_id.clone(), None);
    
    // Create three related resources
    let parent_id = create_test_resource_id("parent-resource");
    let child1_id = create_test_resource_id("child-resource-1");
    let child2_id = create_test_resource_id("child-resource-2");
    
    // Register all resources
    resource_impl.register_resource(
        parent_id.clone(),
        ResourceState::Active,
        &context
    ).await?;
    
    resource_impl.register_resource(
        child1_id.clone(),
        ResourceState::Active,
        &context
    ).await?;
    
    resource_impl.register_resource(
        child2_id.clone(),
        ResourceState::Active,
        &context
    ).await?;
    
    // Record access to resources
    resource_impl.record_access(
        &parent_id,
        ResourceAccessType::Read,
        &context
    ).await?;
    
    resource_impl.record_access(
        &child1_id,
        ResourceAccessType::Write,
        &context
    ).await?;
    
    resource_impl.record_access(
        &child2_id,
        ResourceAccessType::Read,
        &context
    ).await?;
    
    // Add dependencies
    resource_impl.add_dependency(
        &child1_id,
        &parent_id,
        DependencyType::Strong,
        &context
    ).await?;
    
    resource_impl.add_dependency(
        &child2_id,
        &parent_id,
        DependencyType::Strong,
        &context
    ).await?;
    
    // Lock the parent
    let holder_id = create_test_resource_id("holder-id");
    
    let lock_status = resource_impl.acquire_lock(
        &parent_id,
        LockType::Exclusive,
        &holder_id,
        None,
        &context
    ).await?;
    
    assert_eq!(lock_status, LockStatus::Acquired, "Lock should be acquired");
    
    // Verify state changes
    assert_eq!(
        resource_impl.get_resource_state(&parent_id).await?,
        ResourceState::Locked,
        "Parent should be locked"
    );
    
    // Release lock
    resource_impl.release_lock(
        &parent_id,
        &holder_id,
        &context
    ).await?;
    
    // Verify state after release
    assert_eq!(
        resource_impl.get_resource_state(&parent_id).await?,
        ResourceState::Active,
        "Parent should be active after lock release"
    );
    
    // Freeze a resource
    resource_impl.update_resource_state(
        &child1_id,
        ResourceState::Frozen,
        &context
    ).await?;
    
    assert_eq!(
        resource_impl.get_resource_state(&child1_id).await?,
        ResourceState::Frozen,
        "Child1 should be frozen"
    );
    
    // Consume a resource
    resource_impl.update_resource_state(
        &child2_id,
        ResourceState::Consumed,
        &context
    ).await?;
    
    assert_eq!(
        resource_impl.get_resource_state(&child2_id).await?,
        ResourceState::Consumed,
        "Child2 should be consumed"
    );
    
    // Verify dependencies
    let dependents = resource_impl.get_dependents(&parent_id).await?;
    assert_eq!(dependents.len(), 2, "Parent should have 2 dependents");
    
    Ok(())
} 