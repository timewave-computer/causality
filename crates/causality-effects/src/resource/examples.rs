// Resource management examples
//
// This file contains examples demonstrating how to use the resource management
// integration systems, including access control, lifecycle management,
// locking, dependencies, and capabilities.

use std::sync::Arc;
use async_trait::async_trait;

use causality_types::{Result, ContentId};
use causality_domain::domain::DomainId;
use crate::effect_id::EffectId;
use crate::resource::access::{ResourceAccessType, ResourceAccessManager};
use crate::resource::lifecycle::{ResourceLifecycleEvent, EffectResourceLifecycle};
use crate::resource::locking::{CrossDomainLockManager, CrossDomainLockType, LockStatus};
use crate::resource::dependency::{ResourceDependencyManager, DependencyType};
use crate::capability::{UnifiedCapabilityManager, UnifiedCapabilityContext};

/// Example of using the resource access system
pub fn resource_access_example() -> Result<()> {
    println!("Resource Access Example");
    
    // Create resource IDs
    let resource_id = ContentId::from_string("example-resource-1");
    let effect_id = EffectId::from_string("example-effect-1");
    let domain_id = DomainId::from_string("example-domain-1");
    
    // Create resource access manager
    let access_manager = ResourceAccessManager::new();
    
    // Record different types of access
    println!("Recording Read access");
    access_manager.record_access(
        &resource_id,
        ResourceAccessType::Read,
        Some(&effect_id),
        Some(&domain_id)
    );
    
    println!("Recording Write access");
    access_manager.record_access(
        &resource_id,
        ResourceAccessType::Write,
        Some(&effect_id),
        Some(&domain_id)
    );
    
    // Get and print all accesses
    let accesses = access_manager.get_resource_accesses(&resource_id);
    println!("Resource has {} recorded accesses", accesses.len());
    
    for (i, access) in accesses.iter().enumerate() {
        println!("Access {}: {:?} by effect {:?} in domain {:?}", 
            i + 1, 
            access.access_type,
            access.effect_id.as_ref().map(|id| id.to_string()),
            access.domain_id.as_ref().map(|id| id.to_string())
        );
    }
    
    // Check if resource is locked
    let is_locked = access_manager.is_resource_locked(&resource_id);
    println!("Resource is locked: {}", is_locked);
    
    // Lock the resource
    println!("Locking resource");
    access_manager.record_access(
        &resource_id,
        ResourceAccessType::Lock,
        Some(&effect_id),
        Some(&domain_id)
    );
    
    // Check again if resource is locked
    let is_locked = access_manager.is_resource_locked(&resource_id);
    println!("Resource is locked: {}", is_locked);
    
    Ok(())
}

/// Example of using the resource lifecycle system
pub fn resource_lifecycle_example() -> Result<()> {
    println!("Resource Lifecycle Example");
    
    // Create resource IDs
    let resource_id = ContentId::from_string("example-resource-2");
    let effect_id = EffectId::from_string("example-effect-2");
    let domain_id = DomainId::from_string("example-domain-2");
    
    // Create resource lifecycle manager
    let lifecycle_manager = EffectResourceLifecycle::new();
    
    // Register a new resource
    println!("Registering new resource");
    lifecycle_manager.register_resource(&resource_id, Some(&effect_id), Some(&domain_id));
    
    // Print current state
    let state = lifecycle_manager.get_resource_state(&resource_id);
    println!("Resource state after registration: {:?}", state);
    
    // Activate the resource
    println!("Activating resource");
    lifecycle_manager.activate_resource(&resource_id, Some(&effect_id), Some(&domain_id));
    
    // Print updated state
    let state = lifecycle_manager.get_resource_state(&resource_id);
    println!("Resource state after activation: {:?}", state);
    
    // Lock the resource
    println!("Locking resource");
    lifecycle_manager.lock_resource(&resource_id, Some(&effect_id), Some(&domain_id));
    
    // Print updated state
    let state = lifecycle_manager.get_resource_state(&resource_id);
    println!("Resource state after locking: {:?}", state);
    
    // Unlock the resource
    println!("Unlocking resource");
    lifecycle_manager.unlock_resource(&resource_id, Some(&effect_id), Some(&domain_id));
    
    // Print updated state
    let state = lifecycle_manager.get_resource_state(&resource_id);
    println!("Resource state after unlocking: {:?}", state);
    
    // Consume the resource
    println!("Consuming resource");
    lifecycle_manager.consume_resource(&resource_id, Some(&effect_id), Some(&domain_id));
    
    // Print final state
    let state = lifecycle_manager.get_resource_state(&resource_id);
    println!("Resource state after consumption: {:?}", state);
    
    // Get all lifecycle events for the resource
    let events = lifecycle_manager.get_lifecycle_events(&resource_id);
    println!("Resource has {} lifecycle events", events.len());
    
    for (i, event) in events.iter().enumerate() {
        println!("Event {}: {:?} at {}", 
            i + 1, 
            event.event_type,
            event.timestamp
        );
    }
    
    Ok(())
}

/// Example of using the cross-domain locking system
pub fn resource_locking_example() -> Result<()> {
    println!("Resource Locking Example");
    
    // Create resource IDs
    let resource_id = ContentId::from_string("example-resource-3");
    let effect_id = EffectId::from_string("example-effect-3");
    let domain_id = DomainId::from_string("example-domain-3");
    
    // Create cross-domain lock manager
    let lock_manager = CrossDomainLockManager::new();
    
    // Acquire exclusive lock
    println!("Acquiring exclusive lock");
    let lock_result = lock_manager.acquire_lock(
        &resource_id,
        CrossDomainLockType::Exclusive,
        &domain_id,
        &effect_id,
        None,
        None
    );
    
    println!("Lock result: {:?}", lock_result);
    
    // Check if resource is locked
    let is_locked = lock_manager.is_resource_locked(&resource_id);
    println!("Resource is locked: {}", is_locked);
    
    // Try to acquire shared lock while exclusive lock is held
    println!("Trying to acquire shared lock");
    let lock_result = lock_manager.acquire_lock(
        &resource_id,
        CrossDomainLockType::Shared,
        &domain_id,
        &effect_id,
        None,
        None
    );
    
    println!("Lock result: {:?}", lock_result);
    
    // Release the lock
    println!("Releasing lock");
    lock_manager.release_lock(&resource_id, &effect_id);
    
    // Check if resource is still locked
    let is_locked = lock_manager.is_resource_locked(&resource_id);
    println!("Resource is locked: {}", is_locked);
    
    // Acquire shared lock
    println!("Acquiring shared lock");
    let lock_result = lock_manager.acquire_lock(
        &resource_id,
        CrossDomainLockType::Shared,
        &domain_id,
        &effect_id,
        None,
        None
    );
    
    println!("Lock result: {:?}", lock_result);
    
    // Try to acquire another shared lock
    println!("Acquiring another shared lock");
    let another_effect_id = EffectId::from_string("another-effect");
    let lock_result = lock_manager.acquire_lock(
        &resource_id,
        CrossDomainLockType::Shared,
        &domain_id,
        &another_effect_id,
        None,
        None
    );
    
    println!("Lock result: {:?}", lock_result);
    
    // Release all locks
    println!("Releasing all locks");
    lock_manager.release_all_locks(&resource_id);
    
    // Check if resource is still locked
    let is_locked = lock_manager.is_resource_locked(&resource_id);
    println!("Resource is locked: {}", is_locked);
    
    Ok(())
}

/// Example of using the resource dependency system
pub fn resource_dependency_example() -> Result<()> {
    println!("Resource Dependency Example");
    
    // Create resource IDs
    let source_id = ContentId::from_string("source-resource");
    let target_id = ContentId::from_string("target-resource");
    let effect_id = EffectId::from_string("example-effect-4");
    let domain_id = DomainId::from_string("example-domain-4");
    
    // Create dependency manager
    let dependency_manager = ResourceDependencyManager::new();
    
    // Add strong dependency
    println!("Adding strong dependency");
    dependency_manager.add_dependency(
        &source_id,
        &target_id,
        DependencyType::Strong,
        Some(&domain_id),
        Some(&effect_id),
        None
    );
    
    // Check if dependency exists
    let has_dependency = dependency_manager.has_dependency(&source_id, &target_id);
    println!("Has dependency: {}", has_dependency);
    
    // Get dependencies for source
    let deps = dependency_manager.get_dependencies_for_source(&source_id);
    println!("Source has {} dependencies", deps.len());
    
    for (i, dep) in deps.iter().enumerate() {
        println!("Dependency {}: {:?} from {} to {}", 
            i + 1, 
            dep.dependency_type,
            dep.source_id.to_string(),
            dep.target_id.to_string()
        );
    }
    
    // Add more dependencies
    println!("Adding weak dependency");
    let another_target = ContentId::from_string("another-target");
    dependency_manager.add_dependency(
        &source_id,
        &another_target,
        DependencyType::Weak,
        Some(&domain_id),
        Some(&effect_id),
        None
    );
    
    // Get dependencies for source again
    let deps = dependency_manager.get_dependencies_for_source(&source_id);
    println!("Source now has {} dependencies", deps.len());
    
    // Get target dependencies
    let target_deps = dependency_manager.get_dependencies_for_target(&target_id);
    println!("Target has {} reverse dependencies", target_deps.len());
    
    // Remove dependency
    println!("Removing dependency");
    dependency_manager.remove_dependency(&source_id, &target_id);
    
    // Check if dependency still exists
    let has_dependency = dependency_manager.has_dependency(&source_id, &target_id);
    println!("Has dependency: {}", has_dependency);
    
    Ok(())
}

/// Example of coordinating all resource management systems
pub fn integrated_resource_management_example() -> Result<()> {
    println!("Integrated Resource Management Example");
    
    // Create resource IDs
    let resource_id = ContentId::from_string("example-resource-5");
    let effect_id = EffectId::from_string("example-effect-5");
    let domain_id = DomainId::from_string("example-domain-5");
    
    // Create all managers
    let access_manager = ResourceAccessManager::new();
    let lifecycle_manager = EffectResourceLifecycle::new();
    let lock_manager = CrossDomainLockManager::new();
    let dependency_manager = ResourceDependencyManager::new();
    
    println!("1. Creating and registering resource");
    // Register resource in lifecycle manager
    lifecycle_manager.register_resource(&resource_id, Some(&effect_id), Some(&domain_id));
    
    println!("2. Recording access");
    // Record creation access
    access_manager.record_access(
        &resource_id,
        ResourceAccessType::Write,
        Some(&effect_id),
        Some(&domain_id)
    );
    
    println!("3. Activating resource");
    // Activate resource
    lifecycle_manager.activate_resource(&resource_id, Some(&effect_id), Some(&domain_id));
    
    println!("4. Acquiring lock");
    // Acquire lock
    let lock_result = lock_manager.acquire_lock(
        &resource_id,
        CrossDomainLockType::Exclusive,
        &domain_id,
        &effect_id,
        None,
        None
    );
    println!("Lock result: {:?}", lock_result);
    
    println!("5. Recording lock access");
    // Record lock access
    access_manager.record_access(
        &resource_id,
        ResourceAccessType::Lock,
        Some(&effect_id),
        Some(&domain_id)
    );
    
    println!("6. Updating resource lifecycle state");
    // Update lifecycle state
    lifecycle_manager.lock_resource(&resource_id, Some(&effect_id), Some(&domain_id));
    
    println!("7. Creating dependency");
    // Create dependency with another resource
    let dependent_id = ContentId::from_string("dependent-resource");
    dependency_manager.add_dependency(
        &resource_id,
        &dependent_id,
        DependencyType::Strong,
        Some(&domain_id),
        Some(&effect_id),
        None
    );
    
    println!("8. Checking states");
    // Check various states
    let is_locked_access = access_manager.is_resource_locked(&resource_id);
    let is_locked_manager = lock_manager.is_resource_locked(&resource_id);
    let lifecycle_state = lifecycle_manager.get_resource_state(&resource_id);
    let has_dependency = dependency_manager.has_dependency(&resource_id, &dependent_id);
    
    println!("Access manager shows locked: {}", is_locked_access);
    println!("Lock manager shows locked: {}", is_locked_manager);
    println!("Lifecycle state: {:?}", lifecycle_state);
    println!("Has dependency: {}", has_dependency);
    
    println!("9. Releasing and unlocking");
    // Release lock
    lock_manager.release_lock(&resource_id, &effect_id);
    
    // Update lifecycle state
    lifecycle_manager.unlock_resource(&resource_id, Some(&effect_id), Some(&domain_id));
    
    println!("10. Final check");
    // Final checks
    let is_locked_access = access_manager.is_resource_locked(&resource_id);
    let is_locked_manager = lock_manager.is_resource_locked(&resource_id);
    
    println!("Access manager shows locked: {}", is_locked_access);
    println!("Lock manager shows locked: {}", is_locked_manager);
    
    Ok(())
}

/// Runs all resource management examples
pub fn run_all_examples() -> Result<()> {
    println!("Running all resource management examples");
    
    resource_access_example()?;
    println!("\n--------------------------------\n");
    
    resource_lifecycle_example()?;
    println!("\n--------------------------------\n");
    
    resource_locking_example()?;
    println!("\n--------------------------------\n");
    
    resource_dependency_example()?;
    println!("\n--------------------------------\n");
    
    integrated_resource_management_example()?;
    
    println!("\nAll examples completed successfully");
    Ok(())
} 