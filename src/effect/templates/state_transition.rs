// Resource State Transition Helper
//
// This module provides a helper for validating resource state transitions
// against relationship constraints between resources.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

use crate::error::{Error, Result};
use crate::resource::{
    ResourceId,
    RegisterState,
    RelationshipTracker,
    RelationshipType,
    ResourceRegisterLifecycleManager,
};

/// Filter for querying relationships
pub struct RelationshipFilter {
    /// Types of relationships to include (None = all types)
    pub relationship_types: Option<Vec<RelationshipType>>,
    /// Maximum number of results to return
    pub max_results: Option<usize>,
    /// Whether to include deleted relationships
    pub include_deleted: bool,
}

/// Helper struct to validate state transitions against relationship constraints
pub struct ResourceStateTransitionHelper {
    /// Resource lifecycle manager
    lifecycle_manager: Option<Arc<ResourceRegisterLifecycleManager>>,
    /// Relationship tracker
    relationship_tracker: Arc<RelationshipTracker>,
}

impl ResourceStateTransitionHelper {
    /// Create a new helper with a resource lifecycle manager and relationship tracker
    pub fn new(
        lifecycle_manager: Option<Arc<ResourceRegisterLifecycleManager>>,
        relationship_tracker: Arc<RelationshipTracker>,
    ) -> Self {
        Self {
            lifecycle_manager,
            relationship_tracker,
        }
    }
    
    /// Get the lifecycle manager
    pub fn get_lifecycle_manager(&self) -> Option<&Arc<ResourceRegisterLifecycleManager>> {
        self.lifecycle_manager.as_ref()
    }
    
    /// Set the lifecycle manager
    pub fn set_lifecycle_manager(&mut self, lifecycle_manager: Arc<ResourceRegisterLifecycleManager>) {
        self.lifecycle_manager = Some(lifecycle_manager);
    }
    
    /// Validate that a state transition doesn't violate relationship constraints
    pub async fn validate_relationships_for_transition(
        &self,
        resource_id: &ResourceId,
        from_state: &str,
        to_state: &str,
    ) -> Result<bool> {
        // Special case: if transitioning to Archived or Consumed
        if to_state == "Archived" || to_state == "Consumed" {
            // Check if this resource is a parent of any active resources
            let children = self.relationship_tracker.get_child_resources(resource_id)?;
            if !children.is_empty() {
                // Check if any children are still active
                if let Some(lifecycle_manager) = &self.lifecycle_manager {
                    for child_id in &children {
                        if let Ok(state) = lifecycle_manager.get_state(child_id) {
                            // Can't archive/consume if active children exist
                            if state == RegisterState::Active || 
                               state == RegisterState::Locked || 
                               state == RegisterState::Frozen {
                                return Ok(false);
                            }
                        }
                    }
                }
            }
            
            // Check if any resources depend on this resource
            let dependents = self.relationship_tracker.get_dependents(resource_id)?;
            if !dependents.is_empty() {
                // Check if any dependents are still active
                if let Some(lifecycle_manager) = &self.lifecycle_manager {
                    for dependent_id in &dependents {
                        if let Ok(state) = lifecycle_manager.get_state(dependent_id) {
                            // Can't archive/consume if active dependents exist
                            if state == RegisterState::Active || 
                               state == RegisterState::Locked || 
                               state == RegisterState::Frozen {
                                return Ok(false);
                            }
                        }
                    }
                }
            }
        }
        
        // If transitioning to Frozen, check if this resource is locked by another
        if to_state == "Frozen" && from_state != "Frozen" {
            // Get locking relationships
            let filter = RelationshipFilter {
                relationship_types: Some(vec![RelationshipType::Lock]),
                max_results: None,
                include_deleted: false,
            };
            
            let relationships = self.relationship_tracker.get_resource_relationships(
                resource_id, 
                &filter
            )?;
            
            // Can't freeze if locked by another resource
            if !relationships.is_empty() {
                return Ok(false);
            }
        }
        
        // All other transitions are allowed from a relationship perspective
        Ok(true)
    }
    
    /// Update relationships after a state transition
    pub async fn update_relationships_after_transition(
        &self,
        resource_id: &ResourceId,
        from_state: &str,
        to_state: &str,
    ) -> Result<()> {
        // Example: when a resource is consumed, remove all its relationships
        if to_state == "Consumed" {
            self.relationship_tracker.remove_all_relationships(resource_id)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_validate_parent_child_relationship() -> Result<()> {
        // Create the helper components
        let lifecycle_manager = Arc::new(ResourceRegisterLifecycleManager::new());
        let relationship_tracker = Arc::new(RelationshipTracker::new());
        
        // Create the helper
        let helper = ResourceStateTransitionHelper::new(
            Some(lifecycle_manager.clone()),
            relationship_tracker.clone(),
        );
        
        // Set up parent-child relationship
        let parent_id = ResourceId::from("parent");
        let child_id = ResourceId::from("child");
        
        // Register resources
        Arc::get_mut(&mut lifecycle_manager.clone()).unwrap().register_resource(parent_id.clone())?;
        Arc::get_mut(&mut lifecycle_manager.clone()).unwrap().register_resource(child_id.clone())?;
        Arc::get_mut(&mut lifecycle_manager.clone()).unwrap().activate(&parent_id)?;
        Arc::get_mut(&mut lifecycle_manager.clone()).unwrap().activate(&child_id)?;
        
        // Add relationship
        Arc::get_mut(&mut relationship_tracker.clone()).unwrap().add_relationship(
            parent_id.clone(),
            child_id.clone(),
            RelationshipType::ParentChild,
            None,
        )?;
        
        // Try to archive parent - should fail because child is active
        let valid = helper.validate_relationships_for_transition(
            &parent_id,
            "Active",
            "Archived",
        ).await?;
        
        assert!(!valid, "Should not allow archiving parent with active child");
        
        // Archive the child first
        Arc::get_mut(&mut lifecycle_manager.clone()).unwrap().archive(&child_id)?;
        
        // Now try again to archive parent - should succeed
        let valid = helper.validate_relationships_for_transition(
            &parent_id,
            "Active",
            "Archived",
        ).await?;
        
        assert!(valid, "Should allow archiving parent when child is archived");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_validate_dependency_relationship() -> Result<()> {
        // Create the helper components
        let lifecycle_manager = Arc::new(ResourceRegisterLifecycleManager::new());
        let relationship_tracker = Arc::new(RelationshipTracker::new());
        
        // Create the helper
        let helper = ResourceStateTransitionHelper::new(
            Some(lifecycle_manager.clone()),
            relationship_tracker.clone(),
        );
        
        // Set up dependency relationship
        let dependent_id = ResourceId::from("dependent");
        let dependency_id = ResourceId::from("dependency");
        
        // Register resources
        Arc::get_mut(&mut lifecycle_manager.clone()).unwrap().register_resource(dependent_id.clone())?;
        Arc::get_mut(&mut lifecycle_manager.clone()).unwrap().register_resource(dependency_id.clone())?;
        Arc::get_mut(&mut lifecycle_manager.clone()).unwrap().activate(&dependent_id)?;
        Arc::get_mut(&mut lifecycle_manager.clone()).unwrap().activate(&dependency_id)?;
        
        // Add relationship
        Arc::get_mut(&mut relationship_tracker.clone()).unwrap().add_relationship(
            dependent_id.clone(),
            dependency_id.clone(),
            RelationshipType::Dependency,
            None,
        )?;
        
        // Try to consume dependency - should fail because it has an active dependent
        let valid = helper.validate_relationships_for_transition(
            &dependency_id,
            "Active",
            "Consumed",
        ).await?;
        
        assert!(!valid, "Should not allow consuming a dependency with active dependents");
        
        // Consume the dependent first
        Arc::get_mut(&mut lifecycle_manager.clone()).unwrap().consume(&dependent_id)?;
        
        // Now try again to consume dependency - should succeed
        let valid = helper.validate_relationships_for_transition(
            &dependency_id,
            "Active",
            "Consumed",
        ).await?;
        
        assert!(valid, "Should allow consuming dependency when dependent is consumed");
        
        Ok(())
    }
} 
