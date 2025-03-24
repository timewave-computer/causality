// Tests for the unified ResourceRegister model, lifecycle management, and relationship tracking
//
// This module tests the integration of ResourceRegister with lifecycle management and
// relationship tracking to ensure they work together properly.

use std::sync::Arc;
use std::collections::HashMap;

use crate::resource::{
    ResourceRegister, ResourceLogic, FungibilityDomain, Quantity, StorageStrategy,
    ResourceRegisterLifecycleManager, RegisterOperationType, RegisterState,
    RelationshipTracker, RelationshipType, RelationshipDirection, ResourceRelationship
};
use crate::resource::resource_register::StateVisibility;
use crate::resource::lifecycle::TransitionReason;
use crate::tel::types::Metadata;
use crate::time::TimeMapSnapshot;
use crate::crypto::hash::{ContentId, ContentAddressed};
use crate::error::Result;
use crate::types::DomainId;

// Helper function to create a test resource register
fn create_test_register(name: &str, amount: u128) -> ResourceRegister {
    ResourceRegister::new(
        ResourceLogic::Fungible,
        FungibilityDomain(format!("domain-{}", name)),
        Quantity(amount),
        Metadata::new(),
        StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
    )
}

#[cfg(test)]
mod resource_register_integration_tests {
    use std::collections::HashMap;
    use crate::error::Result;
    use crate::types::{RegisterState, DomainId};
    use crate::crypto::hash::{ContentId, ContentAddressed};
    use crate::resource::{
        ResourceRegister, 
        ResourceRegisterLifecycleManager, 
        RelationshipTracker,
        RelationshipType,
        RelationshipDirection
    };

    // Helper function to create a test register
    fn create_test_register(id: &str) -> ResourceRegister {
        let mut register = ResourceRegister::new_minimal();
        register.domain_id = DomainId("test-domain".to_string());
        register.metadata = HashMap::new();
        register
    }

    #[test]
    fn test_lifecycle_and_relationship_integration() -> Result<()> {
        // Create the lifecycle manager and relationship tracker
        let lifecycle_manager = ResourceRegisterLifecycleManager::new();
        let relationship_tracker = RelationshipTracker::new();

        // Create test resources
        let register_a = create_test_register("token-a");
        let register_b = create_test_register("token-b");
        let register_c = create_test_register("token-c");
        
        // Get content IDs
        let token_a = register_a.content_id();
        let token_b = register_b.content_id();
        let token_c = register_c.content_id();

        // Register resources
        lifecycle_manager.register_resource(&token_a, register_a)?;
        lifecycle_manager.register_resource(&token_b, register_b)?;
        lifecycle_manager.register_resource(&token_c, register_c)?;

        // Activate resources
        lifecycle_manager.activate(&token_a)?;
        lifecycle_manager.activate(&token_b)?;
        lifecycle_manager.activate(&token_c)?;

        // Record parent-child relationships
        relationship_tracker.record_parent_child_relationship(token_a.clone(), token_b.clone(), None)?;
        relationship_tracker.record_parent_child_relationship(token_a.clone(), token_c.clone(), None)?;

        // Record dependency relationship
        relationship_tracker.record_dependency_relationship(token_b.clone(), token_c.clone(), None)?;

        // Lock child resources
        lifecycle_manager.lock(&token_b, Some(&token_a.to_string()))?;
        lifecycle_manager.lock(&token_c, Some(&token_a.to_string()))?;

        // Check state
        assert_eq!(lifecycle_manager.get_state(&token_a)?, RegisterState::Active);
        assert_eq!(lifecycle_manager.get_state(&token_b)?, RegisterState::Locked);
        assert_eq!(lifecycle_manager.get_state(&token_c)?, RegisterState::Locked);

        // Get locked resources
        let locked_resources = lifecycle_manager.get_locked_resources(&token_a)?;
        assert_eq!(locked_resources.len(), 2);
        assert!(locked_resources.contains(&token_b));
        assert!(locked_resources.contains(&token_c));

        // Get child resources
        let children = relationship_tracker.get_related_resources(
            &token_a,
            &RelationshipType::ParentChild,
            None
        )?;
        assert_eq!(children.len(), 2);
        assert!(children.contains(&token_b));
        assert!(children.contains(&token_c));

        // Check parent-child relationship
        let has_relationship = relationship_tracker.are_resources_related(
            &token_a, &token_b, Some(RelationshipType::ParentChild))?;
        assert!(has_relationship);

        // Check dependency relationship
        let has_dependency = relationship_tracker.are_resources_related(
            &token_b, &token_c, Some(RelationshipType::Dependency))?;
        assert!(has_dependency);

        // Unlock and consume resources
        lifecycle_manager.unlock(&token_b, Some(&token_a.to_string()))?;
        lifecycle_manager.unlock(&token_c, Some(&token_a.to_string()))?;
        lifecycle_manager.consume(&token_b)?;
        lifecycle_manager.consume(&token_c)?;

        // Verify final states
        assert_eq!(lifecycle_manager.get_state(&token_a)?, RegisterState::Active);
        assert_eq!(lifecycle_manager.get_state(&token_b)?, RegisterState::Consumed);
        assert_eq!(lifecycle_manager.get_state(&token_c)?, RegisterState::Consumed);

        // Verify transition history 
        let b_history = lifecycle_manager.get_transition_history(&token_b)?;
        assert_eq!(b_history.len(), 3); // Initial->Active, Active->Locked, Locked->Consumed

        Ok(())
    }

    #[test]
    fn test_complex_resource_graph() -> Result<()> {
        let lifecycle_manager = ResourceRegisterLifecycleManager::new();
        let relationship_tracker = RelationshipTracker::new();

        // Create token hierarchy
        // token_root
        //   |-- token_a
        //   |     |-- token_a1
        //   |     `-- token_a2
        //   `-- token_b
        //         |-- token_b1
        //         `-- token_b2

        let token_root = ContentId::new("root".to_string());
        let token_a = ContentId::new("a".to_string());
        let token_b = ContentId::new("b".to_string());
        let token_a1 = ContentId::new("a1".to_string());
        let token_a2 = ContentId::new("a2".to_string());
        let token_b1 = ContentId::new("b1".to_string());
        let token_b2 = ContentId::new("b2".to_string());

        // Register and activate all tokens
        for token in [&token_root, &token_a, &token_b, &token_a1, &token_a2, &token_b1, &token_b2] {
            lifecycle_manager.register_resource(token)?;
            lifecycle_manager.activate(token)?;
        }

        // Build the hierarchy
        relationship_tracker.add_parent_child(token_root.clone(), token_a.clone())?;
        relationship_tracker.add_parent_child(token_root.clone(), token_b.clone())?;
        relationship_tracker.add_parent_child(token_a.clone(), token_a1.clone())?;
        relationship_tracker.add_parent_child(token_a.clone(), token_a2.clone())?;
        relationship_tracker.add_parent_child(token_b.clone(), token_b1.clone())?;
        relationship_tracker.add_parent_child(token_b.clone(), token_b2.clone())?;

        // Add dependencies between branches
        relationship_tracker.add_dependency(token_a1.clone(), token_b1.clone())?;
        relationship_tracker.add_dependency(token_a2.clone(), token_b2.clone())?;

        // Lock token_a and all its children
        lifecycle_manager.lock(&token_a, Some(&token_root))?;
        lifecycle_manager.lock(&token_a1, Some(&token_a))?;
        lifecycle_manager.lock(&token_a2, Some(&token_a))?;

        // Verify states
        assert_eq!(lifecycle_manager.get_state(&token_a)?, RegisterState::Locked);
        assert_eq!(lifecycle_manager.get_state(&token_a1)?, RegisterState::Locked);
        assert_eq!(lifecycle_manager.get_state(&token_a2)?, RegisterState::Locked);

        // Check relationships
        let root_children = relationship_tracker.get_child_resources(&token_root)?;
        assert_eq!(root_children.len(), 2);
        assert!(root_children.contains(&token_a));
        assert!(root_children.contains(&token_b));

        let a_children = relationship_tracker.get_child_resources(&token_a)?;
        assert_eq!(a_children.len(), 2);
        assert!(a_children.contains(&token_a1));
        assert!(a_children.contains(&token_a2));

        // Check b is not locked
        assert_eq!(lifecycle_manager.get_state(&token_b)?, RegisterState::Active);
        assert_eq!(lifecycle_manager.get_state(&token_b1)?, RegisterState::Active);
        assert_eq!(lifecycle_manager.get_state(&token_b2)?, RegisterState::Active);

        // Check dependency relationships
        let a1_deps = relationship_tracker.get_dependencies(&token_a1)?;
        assert_eq!(a1_deps.len(), 1);
        assert_eq!(a1_deps[0], token_b1);

        // Unlock everything in the a branch
        lifecycle_manager.unlock(&token_a, Some(&token_root))?;
        lifecycle_manager.unlock(&token_a1, Some(&token_a))?;
        lifecycle_manager.unlock(&token_a2, Some(&token_a))?;

        // All should be active again
        assert_eq!(lifecycle_manager.get_state(&token_a)?, RegisterState::Active);
        assert_eq!(lifecycle_manager.get_state(&token_a1)?, RegisterState::Active);
        assert_eq!(lifecycle_manager.get_state(&token_a2)?, RegisterState::Active);

        Ok(())
    }

    #[test]
    fn test_relationship_changes_over_time() -> Result<()> {
        let lifecycle_manager = ResourceRegisterLifecycleManager::new();
        let relationship_tracker = RelationshipTracker::new();

        // Create test resources
        let token_main = ContentId::new("main".to_string());
        let token_dep1 = ContentId::new("dep1".to_string());
        let token_dep2 = ContentId::new("dep2".to_string());
        let token_dep3 = ContentId::new("dep3".to_string());

        // Register all tokens
        for token in [&token_main, &token_dep1, &token_dep2, &token_dep3] {
            lifecycle_manager.register_resource(token)?;
            lifecycle_manager.activate(token)?;
        }

        // Initially, token_main depends on token_dep1
        relationship_tracker.add_dependency(token_main.clone(), token_dep1.clone())?;

        // Check initial dependency
        let dependencies = relationship_tracker.get_dependencies(&token_main)?;
        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0], token_dep1);

        // Later, token_main depends on token_dep2 instead
        relationship_tracker.remove_relationship(&token_main, &token_dep1)?;
        relationship_tracker.add_dependency(token_main.clone(), token_dep2.clone())?;

        // Check updated dependency
        let dependencies = relationship_tracker.get_dependencies(&token_main)?;
        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0], token_dep2);

        // Finally, token_main depends on both token_dep2 and token_dep3
        relationship_tracker.add_dependency(token_main.clone(), token_dep3.clone())?;

        // Check final dependencies
        let dependencies = relationship_tracker.get_dependencies(&token_main)?;
        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.contains(&token_dep2));
        assert!(dependencies.contains(&token_dep3));

        Ok(())
    }

    #[test]
    fn test_relationship_metadata() -> Result<()> {
        let relationship_tracker = RelationshipTracker::new();

        let token_a = ContentId::new("token-a".to_string());
        let token_b = ContentId::new("token-b".to_string());

        // Create a relationship with custom metadata
        let mut rel = crate::resource::ResourceRelationship::new(
            token_a.clone(),
            token_b.clone(),
            RelationshipType::Custom("TestRelation".to_string()),
            RelationshipDirection::Bidirectional,
        );
        
        // Add metadata
        rel.metadata.insert("created_by", "test-user");
        rel.metadata.insert("priority", "high");
        
        // Add relationship
        relationship_tracker.add_relationship(rel)?;
        
        // Retrieve and check
        let relationships = relationship_tracker.get_relationships(&token_a)?;
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].metadata.get("created_by"), Some(&"test-user".to_string()));
        assert_eq!(relationships[0].metadata.get("priority"), Some(&"high".to_string()));
        
        // Update metadata
        let mut updated_metadata = crate::types::Metadata::new();
        updated_metadata.insert("created_by", "test-user");
        updated_metadata.insert("priority", "low");
        updated_metadata.insert("last_modified", "today");
        
        relationship_tracker.update_relationship_metadata(
            &token_a,
            &token_b,
            updated_metadata
        )?;
        
        // Check updated metadata
        let relationships = relationship_tracker.get_relationships(&token_a)?;
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].metadata.get("priority"), Some(&"low".to_string()));
        assert_eq!(relationships[0].metadata.get("last_modified"), Some(&"today".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_custom_relationship_types() -> Result<()> {
        let relationship_tracker = RelationshipTracker::new();
        
        let token_a = ContentId::new("a".to_string());
        let token_b = ContentId::new("b".to_string());
        let token_c = ContentId::new("c".to_string());
        
        // Add various custom relationships
        relationship_tracker.add_custom_relationship(
            token_a.clone(),
            token_b.clone(),
            "Authorizes".to_string(),
            RelationshipDirection::ParentToChild
        )?;
        
        relationship_tracker.add_custom_relationship(
            token_a.clone(),
            token_c.clone(),
            "Validates".to_string(),
            RelationshipDirection::ParentToChild
        )?;
        
        relationship_tracker.add_custom_relationship(
            token_b.clone(),
            token_c.clone(),
            "References".to_string(),
            RelationshipDirection::Bidirectional
        )?;
        
        // Check relationships by type
        let relationships = relationship_tracker.get_relationships_by_type(
            &token_a,
            &RelationshipType::Custom("Authorizes".to_string())
        )?;
        
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].to_id, token_b);
        
        // Check bidirectional relationship
        let has_relationship = relationship_tracker.has_relationship_of_type(
            &token_c,
            &token_b,
            &RelationshipType::Custom("References".to_string())
        )?;
        
        assert!(has_relationship);
        
        Ok(())
    }
} 
