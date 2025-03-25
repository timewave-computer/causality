// Tests for the UnifiedRegistry implementation
//
// This module contains tests for the UnifiedRegistry class, which unifies
// functionality from multiple registry classes for ResourceRegister instances.

use std::sync::Arc;

use crate::crypto::hash::ContentId;
use crate::error::Result;
use crate::resource::{
    ResourceRegister, 
    ResourceLogic,
    FungibilityDomain,
    Quantity,
    StorageStrategy,
    StateVisibility,
    RegisterState,
    UnifiedRegistry,
    relationship_tracker::RelationshipTracker,
    lifecycle_manager::ResourceRegisterLifecycleManager
};
use crate::tel::types::Metadata;

#[test]
fn test_registry_lifecycle_integration() -> Result<()> {
    // Create a lifecycle manager
    let lifecycle_manager = Arc::new(ResourceRegisterLifecycleManager::new());
    
    // Create a registry with the lifecycle manager
    let mut registry = UnifiedRegistry::with_lifecycle_manager(lifecycle_manager.clone());
    
    // Create a ResourceRegister
    let register = ResourceRegister::new(
        ContentId::new("test-lifecycle-register"),
        ResourceLogic::Fungible,
        FungibilityDomain("TEST".to_string()),
        Quantity(100),
        Metadata::default(),
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    
    // Register it
    let id = registry.register(register.clone())?;
    
    // Verify it's in the registry
    assert!(registry.contains(&id)?);
    
    // Verify it's in the lifecycle manager
    let state = lifecycle_manager.get_state(&id)?;
    assert_eq!(state, register.state);
    
    // Update the register
    registry.update(&id, |register| {
        register.quantity = Quantity(200);
        Ok(())
    })?;
    
    // Verify the update was applied
    let updated = registry.get(&id)?.unwrap();
    assert_eq!(updated.quantity, Quantity(200));
    
    // Consume the register
    registry.consume(&id)?;
    
    // Verify it's consumed in the registry
    let consumed = registry.get(&id)?.unwrap();
    assert_eq!(consumed.state, crate::resource::RegisterState::Consumed);
    
    // Verify it's consumed in the lifecycle manager
    let state = lifecycle_manager.get_state(&id)?;
    assert_eq!(state, crate::resource::RegisterState::Consumed);
    
    // Remove the register
    registry.remove(&id)?;
    
    // Verify it's gone from the registry
    assert!(!registry.contains(&id)?);
    
    Ok(())
}

#[test]
fn test_registry_relationship_integration() -> Result<()> {
    // Create a relationship tracker
    let relationship_tracker = Arc::new(RelationshipTracker::new());
    
    // Create a registry with the relationship tracker
    let registry = UnifiedRegistry::with_relationship_tracker(relationship_tracker.clone());
    
    // Create two ResourceRegisters
    let register1 = ResourceRegister::new(
        ContentId::new("test-relationship-register-1"),
        ResourceLogic::Fungible,
        FungibilityDomain("TEST".to_string()),
        Quantity(100),
        Metadata::default(),
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    
    let register2 = ResourceRegister::new(
        ContentId::new("test-relationship-register-2"),
        ResourceLogic::NonFungible,
        FungibilityDomain("NFT".to_string()),
        Quantity(1),
        Metadata::default(),
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    
    // Register them
    let mut registry_mut = UnifiedRegistry::with_relationship_tracker(relationship_tracker.clone());
    let id1 = registry_mut.register(register1.clone())?;
    let id2 = registry_mut.register(register2.clone())?;
    
    // Add a relationship
    registry.add_relationship(&id1, &id2, "owns")?;
    
    // Verify the relationship in the registry
    let relationships = registry.get_relationships(&id1)?;
    assert_eq!(relationships.len(), 1);
    assert_eq!(relationships[0].0, id2);
    assert_eq!(relationships[0].1, "owns");
    
    // Verify the relationship in the tracker
    let relationships = relationship_tracker.get_relationships(&id1)?;
    assert_eq!(relationships.len(), 1);
    assert_eq!(relationships[0].0, id2);
    assert_eq!(relationships[0].1, "owns");
    
    // Remove the relationship
    registry.remove_relationship(&id1, &id2, "owns")?;
    
    // Verify it's gone from the registry
    let relationships = registry.get_relationships(&id1)?;
    assert_eq!(relationships.len(), 0);
    
    // Verify it's gone from the tracker
    let relationships = relationship_tracker.get_relationships(&id1)?;
    assert_eq!(relationships.len(), 0);
    
    Ok(())
}

#[test]
fn test_registry_shared() -> Result<()> {
    // Create a shared registry
    let registry = UnifiedRegistry::shared();
    
    // Create a ResourceRegister
    let register = ResourceRegister::new(
        ContentId::new("test-shared-register"),
        ResourceLogic::Fungible,
        FungibilityDomain("TEST".to_string()),
        Quantity(100),
        Metadata::default(),
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    
    // Register it in a write lock scope
    let id = {
        let mut write_guard = registry.write().unwrap();
        write_guard.register(register.clone())?
    };
    
    // Verify it's in the registry using a read lock
    {
        let read_guard = registry.read().unwrap();
        assert!(read_guard.contains(&id)?);
        
        let retrieved = read_guard.get(&id)?.unwrap();
        assert_eq!(retrieved.id, register.id);
    }
    
    // Update it in a write lock scope
    {
        let mut write_guard = registry.write().unwrap();
        write_guard.update(&id, |register| {
            register.quantity = Quantity(200);
            Ok(())
        })?;
    }
    
    // Verify the update using a read lock
    {
        let read_guard = registry.read().unwrap();
        let updated = read_guard.get(&id)?.unwrap();
        assert_eq!(updated.quantity, Quantity(200));
    }
    
    // Test concurrent access with clone
    let registry_clone = registry.clone();
    
    // Verify the clone can access the same data
    {
        let read_guard = registry_clone.read().unwrap();
        assert!(read_guard.contains(&id)?);
        
        let retrieved = read_guard.get(&id)?.unwrap();
        assert_eq!(retrieved.quantity, Quantity(200));
    }
    
    Ok(())
}

#[test]
fn test_registry_adapter_implementation() -> Result<()> {
    // Create a registry
    let mut registry = UnifiedRegistry::new();
    
    // Create a ResourceRegister
    let register = ResourceRegister::new(
        ContentId::new("test-adapter-register"),
        ResourceLogic::Fungible,
        FungibilityDomain("TEST".to_string()),
        Quantity(100),
        Metadata::default(),
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    
    // Test adapter methods
    
    // create_register
    let id = registry.register(register.clone())?;
    
    // get_register
    let retrieved = registry.get(&id)?.unwrap();
    assert_eq!(retrieved.id, register.id);
    
    // update_state
    registry.update(&id, |register| {
        register.state = crate::resource::RegisterState::Consumed;
        Ok(())
    })?;
    
    let updated = registry.get(&id)?.unwrap();
    assert_eq!(updated.state, crate::resource::RegisterState::Consumed);
    
    // delete_register (mark as consumed)
    registry.consume(&id)?;
    
    let consumed = registry.get(&id)?.unwrap();
    assert_eq!(consumed.state, crate::resource::RegisterState::Consumed);
    
    Ok(())
}

#[test]
fn test_registry_find_operations() -> Result<()> {
    // Create a registry
    let mut registry = UnifiedRegistry::new();
    
    // Create some ResourceRegisters with different states
    let active_register = ResourceRegister::new(
        ContentId::new("test-active-register"),
        ResourceLogic::Fungible,
        FungibilityDomain("TEST".to_string()),
        Quantity(100),
        Metadata::default(),
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    
    let mut consumed_register = ResourceRegister::new(
        ContentId::new("test-consumed-register"),
        ResourceLogic::Fungible,
        FungibilityDomain("TEST".to_string()),
        Quantity(200),
        Metadata::default(),
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    consumed_register.state = crate::resource::RegisterState::Consumed;
    
    let mut pending_register = ResourceRegister::new(
        ContentId::new("test-pending-register"),
        ResourceLogic::Fungible,
        FungibilityDomain("TEST".to_string()),
        Quantity(300),
        Metadata::default(),
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    );
    pending_register.state = crate::resource::RegisterState::Pending;
    
    // Register them
    registry.register(active_register.clone())?;
    registry.register(consumed_register.clone())?;
    registry.register(pending_register.clone())?;
    
    // Test find methods
    
    // Find by state
    let active = registry.find_by_state(crate::resource::RegisterState::Active);
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, active_register.id);
    
    let consumed = registry.find_by_state(crate::resource::RegisterState::Consumed);
    assert_eq!(consumed.len(), 1);
    assert_eq!(consumed[0].id, consumed_register.id);
    
    let pending = registry.find_by_state(crate::resource::RegisterState::Pending);
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, pending_register.id);
    
    // Find active
    let active = registry.find_active();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, active_register.id);
    
    // Find consumed
    let consumed = registry.find_consumed();
    assert_eq!(consumed.len(), 1);
    assert_eq!(consumed[0].id, consumed_register.id);
    
    // Find with predicate
    let large_quantity = registry.find_all(|reg| reg.quantity.0 > 150);
    assert_eq!(large_quantity.len(), 2);
    
    // Verify the registry length
    assert_eq!(registry.len(), 3);
    assert!(!registry.is_empty());
    
    Ok(())
} 