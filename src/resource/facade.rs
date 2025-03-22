// Facade layer for Resource-Register unification
//
// This module implements a facade that presents a unified ResourceRegister API
// while delegating to the new lifecycle manager and relationship tracker.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::resource::{
    ResourceId, ResourceState, RegisterState, ResourceRegister,
    TransitionReason, RelationshipType, RelationshipDirection,
    ResourceRelationship
};
use crate::resource::lifecycle_manager::ResourceRegisterLifecycleManager;
use crate::resource::relationship_tracker::RelationshipTracker;
use crate::error::{Error, Result};

/// Facade service that presents a simplified API using the unified resource register services
/// This is a temporary facade to ease transition to the new system for existing code
pub struct ResourceRegisterFacade {
    // New unified components
    lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
    relationship_tracker: Arc<RelationshipTracker>,
    
    // Cache to avoid redundant lookups
    cache: Mutex<HashMap<ResourceId, ResourceRegister>>,
}

impl ResourceRegisterFacade {
    /// Create a new ResourceRegister facade using the unified components
    pub fn new(
        lifecycle_manager: Arc<ResourceRegisterLifecycleManager>,
        relationship_tracker: Arc<RelationshipTracker>,
    ) -> Self {
        Self {
            lifecycle_manager,
            relationship_tracker,
            cache: Mutex::new(HashMap::new()),
        }
    }
    
    /// Get a ResourceRegister by ID
    pub fn get_resource_register(&self, id: &ResourceId) -> Result<ResourceRegister> {
        // Check cache first
        {
            let cache = self.cache.lock().unwrap();
            if let Some(register) = cache.get(id) {
                return Ok(register.clone());
            }
        }
        
        // Get register from the lifecycle manager
        let resource_register = self.lifecycle_manager.get_register(id)?;
        
        // Cache the result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(id.clone(), resource_register.clone());
        }
        
        Ok(resource_register)
    }
    
    /// Create a new ResourceRegister
    pub fn create_resource_register(&self, resource_register: ResourceRegister) -> Result<ResourceId> {
        // Use the lifecycle manager to create the register
        let id = self.lifecycle_manager.create_register(resource_register.clone())?;
        
        // Cache the result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(id.clone(), resource_register);
        }
        
        Ok(id)
    }
    
    /// Update a ResourceRegister's state
    pub fn update_resource_register(&self, id: &ResourceId, new_state: ResourceState) -> Result<()> {
        // Transition the resource register to the new state
        self.lifecycle_manager.transition_state(
            id, 
            new_state, 
            TransitionReason::UserInitiated
        )?;
        
        // Update cache
        let updated_register = self.lifecycle_manager.get_register(id)?;
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(id.clone(), updated_register);
        }
        
        Ok(())
    }
    
    /// Delete a ResourceRegister 
    pub fn delete_resource_register(&self, id: &ResourceId) -> Result<()> {
        // Mark the register as consumed (soft delete)
        self.lifecycle_manager.transition_state(
            id,
            ResourceState::Consumed,
            TransitionReason::UserInitiated
        )?;
        
        // Remove from cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.remove(id);
        }
        
        Ok(())
    }
    
    /// Create a relationship between two resources
    pub fn create_relationship(
        &self,
        source_id: &ResourceId,
        target_id: &ResourceId,
        relationship_type: RelationshipType,
        direction: RelationshipDirection,
    ) -> Result<()> {
        self.relationship_tracker.add_relationship(
            ResourceRelationship {
                source_id: source_id.clone(),
                target_id: target_id.clone(),
                relationship_type,
                direction,
                metadata: HashMap::new(),
            }
        )
    }
    
    /// Find resources related to the given resource
    pub fn find_related_resources(
        &self,
        resource_id: &ResourceId,
        relationship_type: Option<RelationshipType>,
        direction: Option<RelationshipDirection>,
    ) -> Result<Vec<ResourceId>> {
        self.relationship_tracker.find_related_resources(
            resource_id,
            relationship_type,
            direction
        )
    }
    
    /// Check if a relationship exists between two resources
    pub fn has_relationship(
        &self,
        source_id: &ResourceId,
        target_id: &ResourceId,
        relationship_type: Option<RelationshipType>,
        direction: Option<RelationshipDirection>,
    ) -> Result<bool> {
        self.relationship_tracker.has_relationship(
            source_id,
            target_id,
            relationship_type,
            direction
        )
    }
}

// Implementation of the ResourceRegistryAdapter trait
impl crate::resource::ResourceRegistryAdapter for ResourceRegisterFacade {
    fn get_register(&self, id: &ResourceId) -> Result<ResourceRegister> {
        self.get_resource_register(id)
    }
    
    fn create_register(&self, register: ResourceRegister) -> Result<ResourceId> {
        self.create_resource_register(register)
    }
    
    fn update_state(&self, id: &ResourceId, new_state: ResourceState) -> Result<()> {
        self.update_resource_register(id, new_state)
    }
    
    fn delete_register(&self, id: &ResourceId) -> Result<()> {
        self.delete_resource_register(id)
    }
} 