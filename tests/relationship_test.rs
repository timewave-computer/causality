#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;
    
    use causality::error::Result;
    use causality::resource::relationship::{
        CrossDomainRelationship,
        CrossDomainRelationshipType,
        CrossDomainMetadata,
        CrossDomainRelationshipManager,
        SyncStrategy,
    };
    
    #[test]
    fn test_cross_domain_relationship_creation() -> Result<()> {
        // Create relationship manager
        let manager = CrossDomainRelationshipManager::new();
        
        // Create test relationships
        let mirror_rel = create_test_relationship(
            "resource1",
            "domain1",
            "resource1-mirror",
            "domain2",
            CrossDomainRelationshipType::Mirror,
            true,
            SyncStrategy::Periodic(Duration::from_secs(3600)),
            false,
        );
        
        let reference_rel = create_test_relationship(
            "resource2",
            "domain1",
            "resource3",
            "domain2",
            CrossDomainRelationshipType::Reference,
            false,
            SyncStrategy::EventDriven,
            true,
        );
        
        // Add relationships to manager
        manager.add_relationship(mirror_rel.clone())?;
        manager.add_relationship(reference_rel.clone())?;
        
        // Verify relationships were added
        let all_relationships = manager.get_all_relationships()?;
        assert_eq!(all_relationships.len(), 2);
        
        // Get specific relationship
        let retrieved_mirror = manager.get_relationship(&mirror_rel.id)?;
        assert_eq!(retrieved_mirror.source_resource, "resource1");
        assert_eq!(retrieved_mirror.target_resource, "resource1-mirror");
        assert_eq!(retrieved_mirror.relationship_type, CrossDomainRelationshipType::Mirror);
        
        // Test filtering by source domain
        let domain1_rels = manager.get_relationships_by_source_domain("domain1".to_string())?;
        assert_eq!(domain1_rels.len(), 2);
        
        // Test filtering by target domain
        let domain2_rels = manager.get_relationships_by_target_domain("domain2".to_string())?;
        assert_eq!(domain2_rels.len(), 2);
        
        // Test filtering by source resource
        let resource1_rels = manager.get_relationships_by_source_resource("resource1".to_string())?;
        assert_eq!(resource1_rels.len(), 1);
        assert_eq!(resource1_rels[0].relationship_type, CrossDomainRelationshipType::Mirror);
        
        // Test removal
        manager.remove_relationship(&reference_rel.id)?;
        let remaining = manager.get_all_relationships()?;
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, mirror_rel.id);
        
        Ok(())
    }
    
    #[test]
    fn test_relationship_type_compatibility() -> Result<()> {
        // Test Mirror relationship
        let mirror = create_test_relationship(
            "r1", "d1", "r2", "d2", 
            CrossDomainRelationshipType::Mirror, true, 
            SyncStrategy::Periodic(Duration::from_secs(60)), false
        );
        
        assert!(mirror.metadata.requires_sync);
        assert!(matches!(mirror.metadata.sync_strategy, SyncStrategy::Periodic(_)));
        
        // Test Reference relationship
        let reference = create_test_relationship(
            "r1", "d1", "r2", "d2", 
            CrossDomainRelationshipType::Reference, false, 
            SyncStrategy::EventDriven, true
        );
        
        assert!(!reference.metadata.requires_sync);
        assert!(matches!(reference.metadata.sync_strategy, SyncStrategy::EventDriven));
        assert!(reference.bidirectional);
        
        // Test Custom relationship
        let custom = create_test_relationship(
            "r1", "d1", "r2", "d2", 
            CrossDomainRelationshipType::Custom("test-type".to_string()), true, 
            SyncStrategy::Hybrid(Duration::from_secs(3600)), false
        );
        
        if let CrossDomainRelationshipType::Custom(type_name) = &custom.relationship_type {
            assert_eq!(type_name, "test-type");
        } else {
            panic!("Expected Custom relationship type");
        }
        
        assert!(matches!(custom.metadata.sync_strategy, SyncStrategy::Hybrid(_)));
        
        Ok(())
    }
    
    #[test]
    fn test_relationship_bidirectionality() -> Result<()> {
        let manager = CrossDomainRelationshipManager::new();
        
        // Create a bidirectional relationship
        let bidir_rel = create_test_relationship(
            "r1", "d1", "r2", "d2", 
            CrossDomainRelationshipType::Reference, false, 
            SyncStrategy::EventDriven, true
        );
        
        manager.add_relationship(bidir_rel.clone())?;
        
        // Test that we can find the relationship by either source or target
        let by_source = manager.get_relationships_by_source_resource("r1".to_string())?;
        assert_eq!(by_source.len(), 1);
        
        let by_target = manager.get_relationships_by_target_resource("r2".to_string())?;
        assert_eq!(by_target.len(), 1);
        
        // For bidirectional relationships, we should be able to find them regardless of direction
        let r1_relationships = manager.get_relationships_for_resource("r1".to_string())?;
        assert_eq!(r1_relationships.len(), 1);
        
        let r2_relationships = manager.get_relationships_for_resource("r2".to_string())?;
        assert_eq!(r2_relationships.len(), 1);
        
        Ok(())
    }
    
    // Helper to create test relationships
    fn create_test_relationship(
        source_resource: &str,
        source_domain: &str,
        target_resource: &str,
        target_domain: &str,
        rel_type: CrossDomainRelationshipType,
        requires_sync: bool,
        sync_strategy: SyncStrategy,
        bidirectional: bool,
    ) -> CrossDomainRelationship {
        let metadata = CrossDomainMetadata {
            origin_domain: source_domain.to_string(),
            target_domain: target_domain.to_string(),
            requires_sync,
            sync_strategy,
        };
        
        CrossDomainRelationship::new(
            source_resource.to_string(),
            source_domain.to_string(),
            target_resource.to_string(),
            target_domain.to_string(),
            rel_type,
            metadata,
            bidirectional,
        )
    }
} 