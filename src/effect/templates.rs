// Effect Templates for Common Operations
//
// This module provides templates for commonly used effect patterns,
// making it easier to create common operations without having to manually
// compose multiple effects.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde::{Serialize, Deserialize};
use async_trait::async_trait;

use crate::address::Address;
use crate::domain::DomainType;
use crate::effect::{
    Effect, EffectContext, EffectOutcome, EffectResult, EffectError,
    create_composite_effect, ExecutionBoundary,
};
use crate::effect::storage::{
    create_store_on_chain_effect,
    create_read_from_chain_effect,
    create_store_commitment_effect,
    create_store_nullifier_effect,
};
use crate::error::{Error, Result};
use crate::resource::{
    ResourceId, 
    ResourceRegister, 
    RegisterState,
    RegisterOperationType,
    ResourceRegisterLifecycleManager,
    RelationshipTracker,
    RelationshipType,
};
use crate::types::{DomainId, Metadata};

/// Template for creating a new resource
pub fn create_resource_effect(
    resource: &ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
) -> Result<Arc<dyn Effect>> {
    // Default fields to store for a new resource
    let fields = HashSet::from([
        "id".to_string(),
        "owner".to_string(),
        "state".to_string(),
        "metadata".to_string(),
    ]);
    
    // Create a storage effect based on the resource's storage strategy
    let storage_effect = match resource.storage_strategy {
        crate::resource::StorageStrategy::OnChain => {
            create_store_on_chain_effect(
                resource.id.clone(),
                fields,
                domain_id,
                invoker,
            )
        },
        crate::resource::StorageStrategy::Commitment => {
            // Generate a commitment for the resource
            let commitment = resource.generate_commitment()?;
            
            create_store_commitment_effect(
                resource.id.clone(),
                commitment,
                domain_id,
                invoker,
            )
        },
        crate::resource::StorageStrategy::Nullifier => {
            // Generate a nullifier for the resource
            let nullifier = resource.generate_nullifier()?;
            
            create_store_nullifier_effect(
                resource.id.clone(),
                nullifier,
                domain_id,
                invoker,
            )
        },
        crate::resource::StorageStrategy::Hybrid => {
            // For hybrid storage, store both on-chain and a commitment
            let on_chain_effect = create_store_on_chain_effect(
                resource.id.clone(),
                fields.clone(),
                domain_id.clone(),
                invoker,
            );
            
            let commitment = resource.generate_commitment()?;
            let commitment_effect = create_store_commitment_effect(
                resource.id.clone(),
                commitment,
                domain_id,
                invoker,
            );
            
            // Combine the effects
            create_composite_effect(
                vec![on_chain_effect, commitment_effect],
                "Hybrid Storage".to_string(),
            )
        },
    };
    
    Ok(storage_effect)
}

/// Template for updating a resource's metadata
pub fn update_resource_effect(
    resource: &mut ResourceRegister,
    fields: HashSet<String>,
    domain_id: DomainId,
    invoker: Address,
) -> Result<Arc<dyn Effect>> {
    // Ensure resource is in a valid state for updates
    if resource.state != RegisterState::Active && 
       resource.state != RegisterState::Pending {
        return Err(Error::InvalidOperation(
            format!("Resource {} cannot be updated in state {:?}", 
                resource.id, resource.state)
        ));
    }
    
    // Create a storage effect based on the resource's storage strategy
    let storage_effect = match resource.storage_strategy {
        crate::resource::StorageStrategy::OnChain => {
            create_store_on_chain_effect(
                resource.id.clone(),
                fields,
                domain_id,
                invoker,
            )
        },
        crate::resource::StorageStrategy::Commitment => {
            // Generate a new commitment for the updated resource
            let commitment = resource.generate_commitment()?;
            
            create_store_commitment_effect(
                resource.id.clone(),
                commitment,
                domain_id,
                invoker,
            )
        },
        crate::resource::StorageStrategy::Nullifier => {
            // For nullifier-based resources, updates create a new nullifier
            let nullifier = resource.generate_nullifier()?;
            
            create_store_nullifier_effect(
                resource.id.clone(),
                nullifier,
                domain_id,
                invoker,
            )
        },
        crate::resource::StorageStrategy::Hybrid => {
            // For hybrid storage, update both on-chain and commitment
            let on_chain_effect = create_store_on_chain_effect(
                resource.id.clone(),
                fields.clone(),
                domain_id.clone(),
                invoker,
            );
            
            let commitment = resource.generate_commitment()?;
            let commitment_effect = create_store_commitment_effect(
                resource.id.clone(),
                commitment,
                domain_id,
                invoker,
            );
            
            // Combine the effects
            create_composite_effect(
                vec![on_chain_effect, commitment_effect],
                "Hybrid Update".to_string(),
            )
        },
    };
    
    Ok(storage_effect)
}

/// Template for locking a resource
pub fn lock_resource_effect(
    resource: &mut ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
) -> Result<Arc<dyn Effect>> {
    // Lock the resource in the lifecycle manager
    resource.lifecycle_manager.lock(&resource.id, Some(&invoker))?;
    
    // If a relationship tracker is provided and we have a locker,
    // create a lock relationship between the resources
    if let Some(tracker) = resource.relationship_tracker.as_mut() {
        tracker.add_relationship(
            invoker.clone(),
            resource.id.clone(),
            RelationshipType::Lock,
            None,
        )?;
    }
    
    Ok(Arc::new(EmptyEffect::new()))
}

/// Template for unlocking a resource
pub fn unlock_resource_effect(
    resource: &mut ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
) -> Result<Arc<dyn Effect>> {
    // Unlock the resource in the lifecycle manager
    resource.lifecycle_manager.unlock(&resource.id, Some(&invoker))?;
    
    // If a relationship tracker is provided and we have an unlocker,
    // remove the lock relationship between the resources
    if let Some(tracker) = resource.relationship_tracker.as_mut() {
        tracker.remove_relationship_between(
            &invoker,
            &resource.id,
            Some(RelationshipType::Lock),
        )?;
    }
    
    Ok(Arc::new(EmptyEffect::new()))
}

/// Template for resource consumption
pub fn consume_resource_effect(
    resource: &mut ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
    relationship_tracker: &mut RelationshipTracker,
) -> Result<Arc<dyn Effect>> {
    // Consume the resource in the lifecycle manager
    resource.lifecycle_manager.consume(&resource.id)?;
    
    // If a relationship tracker is provided and we have a consumer,
    // create a consumption relationship between the resources
    if let Some(consumer) = resource.relationship_tracker.as_mut() {
        consumer.add_relationship(
            invoker.clone(),
            resource.id.clone(),
            RelationshipType::Consumption,
            None,
        )?;
    }
    
    Ok(Arc::new(EmptyEffect::new()))
}

/// Template for transferring resource ownership
pub fn transfer_resource_effect(
    resource: &mut ResourceRegister,
    from: Address,
    to: Address,
    domain_id: DomainId,
) -> Result<Arc<dyn Effect>> {
    // Ensure resource is in a valid state for transfer
    if resource.state != RegisterState::Active {
        return Err(Error::InvalidOperation(
            format!("Resource {} cannot be transferred in state {:?}", 
                resource.id, resource.state)
        ));
    }
    
    // Update the owner
    resource.update_owner(to.clone())?;
    
    // Create a fields set for updating
    let fields = HashSet::from([
        "owner".to_string(),
        "metadata".to_string(), // Include metadata for transfer record
    ]);
    
    // Create a storage effect based on the resource's storage strategy
    let storage_effect = match resource.storage_strategy {
        crate::resource::StorageStrategy::OnChain => {
            create_store_on_chain_effect(
                resource.id.clone(),
                fields,
                domain_id,
                from,
            )
        },
        crate::resource::StorageStrategy::Commitment => {
            // Generate a new commitment for the transferred resource
            let commitment = resource.generate_commitment()?;
            
            create_store_commitment_effect(
                resource.id.clone(),
                commitment,
                domain_id,
                from,
            )
        },
        crate::resource::StorageStrategy::Nullifier => {
            // For nullifier-based resources, transfers create a new nullifier
            let nullifier = resource.generate_nullifier()?;
            
            create_store_nullifier_effect(
                resource.id.clone(),
                nullifier,
                domain_id,
                from,
            )
        },
        crate::resource::StorageStrategy::Hybrid => {
            // For hybrid storage, update both on-chain and commitment
            let on_chain_effect = create_store_on_chain_effect(
                resource.id.clone(),
                fields.clone(),
                domain_id.clone(),
                from.clone(),
            );
            
            let commitment = resource.generate_commitment()?;
            let commitment_effect = create_store_commitment_effect(
                resource.id.clone(),
                commitment,
                domain_id,
                from,
            );
            
            // Combine the effects
            create_composite_effect(
                vec![on_chain_effect, commitment_effect],
                "Hybrid Transfer".to_string(),
            )
        },
    };
    
    Ok(storage_effect)
}

/// Template for creating a parent-child relationship between resources
pub fn create_parent_child_effect(
    parent_id: ResourceId,
    child_id: ResourceId,
    relationship_tracker: &mut RelationshipTracker,
    metadata: Option<Metadata>,
) -> Result<Arc<dyn Effect>> {
    // Add a parent-child relationship
    relationship_tracker.add_parent_child_relationship(
        parent_id,
        child_id,
        metadata,
    )?;
    
    Ok(Arc::new(EmptyEffect::new()))
}

/// Template for creating a dependency relationship between resources
pub fn create_dependency_effect(
    dependent_id: ResourceId,
    dependency_id: ResourceId,
    relationship_tracker: &mut RelationshipTracker,
    metadata: Option<Metadata>,
) -> Result<Arc<dyn Effect>> {
    // Add a dependency relationship
    relationship_tracker.add_dependency_relationship(
        dependent_id,
        dependency_id,
        metadata,
    )?;
    
    Ok(Arc::new(EmptyEffect::new()))
}

/// Template for freezing a resource
pub fn freeze_resource_effect(
    resource: &mut ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
) -> Result<Arc<dyn Effect>> {
    // Freeze the resource in the lifecycle manager
    resource.lifecycle_manager.freeze(&resource.id)?;
    
    Ok(Arc::new(EmptyEffect::new()))
}

/// Template for unfreezing a resource
pub fn unfreeze_resource_effect(
    resource: &mut ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
) -> Result<Arc<dyn Effect>> {
    // Unfreeze the resource in the lifecycle manager
    resource.lifecycle_manager.unfreeze(&resource.id)?;
    
    Ok(Arc::new(EmptyEffect::new()))
}

/// Template for archiving a resource
pub fn archive_resource_effect(
    resource: &mut ResourceRegister,
    domain_id: DomainId,
    invoker: Address,
) -> Result<Arc<dyn Effect>> {
    // Archive the resource in the lifecycle manager
    resource.lifecycle_manager.archive(&resource.id)?;
    
    Ok(Arc::new(EmptyEffect::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Metadata;
    
    // Helper function to create a test resource
    fn create_test_resource(id: &str) -> ResourceRegister {
        let mut metadata = Metadata::default();
        metadata.insert("test".to_string(), "value".to_string());
        
        ResourceRegister {
            id: id.to_string(),
            owner: Address::random(),
            state: RegisterState::Initial,
            metadata,
            storage_strategy: crate::resource::StorageStrategy::OnChain,
            // ... other fields would be initialized here
        }
    }
    
    #[test]
    fn test_lock_unlock_resource_effect() {
        let mut lifecycle_manager = ResourceRegisterLifecycleManager::new();
        let mut relationship_tracker = RelationshipTracker::new();
        
        let resource_id = "resource1".to_string();
        let locker_id = "locker1".to_string();
        
        // Register and activate resources
        lifecycle_manager.register_resource(resource_id.clone()).unwrap();
        lifecycle_manager.activate(&resource_id).unwrap();
        
        lifecycle_manager.register_resource(locker_id.clone()).unwrap();
        lifecycle_manager.activate(&locker_id).unwrap();
        
        // Test lock effect
        lock_resource_effect(
            &mut create_test_resource("resource1"),
            DomainId::new(0),
            Address::random(),
        ).unwrap();
        
        // Verify resource is locked
        assert_eq!(
            lifecycle_manager.get_state(&resource_id).unwrap(),
            RegisterState::Locked
        );
        
        // Verify relationship is created
        let relationships = relationship_tracker.get_relationships_between(
            &locker_id,
            &resource_id,
        ).unwrap();
        
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].relationship_type, RelationshipType::Lock);
        
        // Test unlock effect
        unlock_resource_effect(
            &mut create_test_resource("resource1"),
            DomainId::new(0),
            Address::random(),
        ).unwrap();
        
        // Verify resource is unlocked
        assert_eq!(
            lifecycle_manager.get_state(&resource_id).unwrap(),
            RegisterState::Active
        );
        
        // Verify relationship is removed
        let relationships = relationship_tracker.get_relationships_between(
            &locker_id,
            &resource_id,
        ).unwrap();
        
        assert_eq!(relationships.len(), 0);
    }
    
    #[test]
    fn test_parent_child_relationship() {
        let mut relationship_tracker = RelationshipTracker::new();
        
        let parent_id = "parent1".to_string();
        let child_id = "child1".to_string();
        
        // Test creating parent-child relationship
        create_parent_child_effect(
            parent_id.clone(),
            child_id.clone(),
            &mut relationship_tracker,
            None,
        ).unwrap();
        
        // Verify relationship is created
        let relationships = relationship_tracker.get_relationships_between(
            &parent_id,
            &child_id,
        ).unwrap();
        
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].relationship_type, RelationshipType::ParentChild);
        
        // Test getting children
        let children = relationship_tracker.get_child_resources(&parent_id).unwrap();
        assert_eq!(children.len(), 1);
        assert!(children.contains(&child_id));
        
        // Test getting parent
        let parents = relationship_tracker.get_parent_resources(&child_id).unwrap();
        assert_eq!(parents.len(), 1);
        assert!(parents.contains(&parent_id));
    }
    
    #[test]
    fn test_dependency_relationship() {
        let mut relationship_tracker = RelationshipTracker::new();
        
        let dependent_id = "dependent1".to_string();
        let dependency_id = "dependency1".to_string();
        
        // Test creating dependency relationship
        create_dependency_effect(
            dependent_id.clone(),
            dependency_id.clone(),
            &mut relationship_tracker,
            None,
        ).unwrap();
        
        // Verify relationship is created
        let relationships = relationship_tracker.get_relationships_between(
            &dependent_id,
            &dependency_id,
        ).unwrap();
        
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].relationship_type, RelationshipType::Dependency);
        
        // Test getting dependencies
        let dependencies = relationship_tracker.get_dependencies(&dependent_id).unwrap();
        assert_eq!(dependencies.len(), 1);
        assert!(dependencies.contains(&dependency_id));
        
        // Test getting dependents
        let dependents = relationship_tracker.get_dependents(&dependency_id).unwrap();
        assert_eq!(dependents.len(), 1);
        assert!(dependents.contains(&dependent_id));
    }
} 