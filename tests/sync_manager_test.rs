// Cross-Domain Relationship Synchronization Manager Tests
//
// This file tests the functionality of the cross-domain relationship synchronization manager,
// which handles synchronizing resources across different domains.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    
    use causality::error::Result;
    use causality::resource::relationship::{
        CrossDomainRelationship,
        CrossDomainRelationshipType,
        CrossDomainMetadata,
        CrossDomainRelationshipManager,
        SyncStrategy,
        CrossDomainSyncManager,
        SyncDirection,
        SyncOptions,
        SyncResult,
        SyncError,
    };
    
    // Mock resource data for testing
    #[derive(Debug, Clone)]
    struct MockResource {
        id: String,
        domain: String,
        data: HashMap<String, String>,
        version: u64,
    }
    
    // Mock resource store for testing
    struct MockResourceStore {
        resources: Arc<Mutex<HashMap<String, MockResource>>>,
        sync_log: Arc<Mutex<Vec<(String, String, SyncDirection)>>>,
    }
    
    impl MockResourceStore {
        fn new() -> Self {
            Self {
                resources: Arc::new(Mutex::new(HashMap::new())),
                sync_log: Arc::new(Mutex::new(Vec::new())),
            }
        }
        
        fn add_resource(&self, resource: MockResource) {
            let mut resources = self.resources.lock().unwrap();
            resources.insert(resource.id.clone(), resource);
        }
        
        fn get_resource(&self, id: &str) -> Option<MockResource> {
            let resources = self.resources.lock().unwrap();
            resources.get(id).cloned()
        }
        
        fn update_resource(&self, id: &str, data: HashMap<String, String>) -> Result<()> {
            let mut resources = self.resources.lock().unwrap();
            
            if let Some(resource) = resources.get_mut(id) {
                resource.data = data;
                resource.version += 1;
                Ok(())
            } else {
                Err("Resource not found".into())
            }
        }
        
        fn log_sync(&self, source: &str, target: &str, direction: SyncDirection) {
            let mut log = self.sync_log.lock().unwrap();
            log.push((source.to_string(), target.to_string(), direction));
        }
        
        fn get_sync_log(&self) -> Vec<(String, String, SyncDirection)> {
            let log = self.sync_log.lock().unwrap();
            log.clone()
        }
    }
    
    // Helper to create a test relationship
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
    
    #[test]
    fn test_sync_manager_initialization() -> Result<()> {
        // We would normally need an OperationManager here, but for testing we can use a mock
        let operation_manager = Arc::new(mock_operation_manager());
        
        // Create sync manager
        let sync_manager = CrossDomainSyncManager::new(operation_manager);
        
        // Initially there should be no handlers registered
        assert_eq!(sync_manager.get_handler_count()?, 0);
        
        Ok(())
    }
    
    #[test]
    fn test_sync_handler_registration() -> Result<()> {
        // Create operation manager mock
        let operation_manager = Arc::new(mock_operation_manager());
        
        // Create sync manager
        let sync_manager = CrossDomainSyncManager::new(operation_manager);
        
        // Register a handler for domain1 -> domain2
        sync_manager.register_sync_handler(
            "domain1",
            "domain2",
            Box::new(|relationship, direction, options| {
                // Simple handler that returns success
                Ok(SyncResult::Success {
                    relationship_id: relationship.id.clone(),
                    direction: direction.clone(),
                    metadata: HashMap::new(),
                })
            }),
        )?;
        
        // Verify handler was registered
        assert_eq!(sync_manager.get_handler_count()?, 1);
        
        // Register another handler
        sync_manager.register_sync_handler(
            "domain2",
            "domain3",
            Box::new(|relationship, direction, options| {
                // Handler that always returns "skipped"
                Ok(SyncResult::Skipped {
                    relationship_id: relationship.id.clone(),
                    reason: "Not needed".to_string(),
                })
            }),
        )?;
        
        // Verify count increased
        assert_eq!(sync_manager.get_handler_count()?, 2);
        
        Ok(())
    }
    
    #[test]
    fn test_sync_relationship() -> Result<()> {
        // Create operation manager mock
        let operation_manager = Arc::new(mock_operation_manager());
        
        // Create sync manager
        let sync_manager = CrossDomainSyncManager::new(operation_manager);
        
        // Create a test relationship
        let test_rel = create_test_relationship(
            "resource1", "domain1", "resource2", "domain2",
            CrossDomainRelationshipType::Mirror, true,
            SyncStrategy::Periodic(Duration::from_secs(3600)), true,
        );
        
        // Register a handler that returns success
        sync_manager.register_sync_handler(
            "domain1",
            "domain2",
            Box::new(|relationship, direction, options| {
                Ok(SyncResult::Success {
                    relationship_id: relationship.id.clone(),
                    direction: direction.clone(),
                    metadata: HashMap::new(),
                })
            }),
        )?;
        
        // Synchronize the relationship
        let result = sync_manager.synchronize_relationship(
            &test_rel,
            SyncDirection::SourceToTarget,
            &SyncOptions::default(),
        )?;
        
        // Verify result
        match result {
            SyncResult::Success { relationship_id, direction, .. } => {
                assert_eq!(relationship_id, test_rel.id);
                assert!(matches!(direction, SyncDirection::SourceToTarget));
            },
            _ => panic!("Expected success result, got: {:?}", result),
        }
        
        Ok(())
    }
    
    #[test]
    fn test_sync_without_handler() -> Result<()> {
        // Create operation manager mock
        let operation_manager = Arc::new(mock_operation_manager());
        
        // Create sync manager
        let sync_manager = CrossDomainSyncManager::new(operation_manager);
        
        // Create a test relationship
        let test_rel = create_test_relationship(
            "resource1", "domain1", "resource2", "domain2",
            CrossDomainRelationshipType::Mirror, true,
            SyncStrategy::Periodic(Duration::from_secs(3600)), true,
        );
        
        // Try to synchronize without registered handler
        let result = sync_manager.synchronize_relationship(
            &test_rel,
            SyncDirection::SourceToTarget,
            &SyncOptions::default(),
        );
        
        // Should return an error
        assert!(result.is_err());
        
        // Verify the error is a NoSyncHandler error
        if let Err(err) = result {
            match err {
                SyncError::NoSyncHandler(source, target) => {
                    assert_eq!(source, "domain1");
                    assert_eq!(target, "domain2");
                },
                _ => panic!("Expected NoSyncHandler error, got: {:?}", err),
            }
        }
        
        Ok(())
    }
    
    #[test]
    fn test_bidirectional_sync() -> Result<()> {
        // Create operation manager mock
        let operation_manager = Arc::new(mock_operation_manager());
        
        // Create sync manager
        let sync_manager = CrossDomainSyncManager::new(operation_manager);
        
        // Create a bidirectional relationship
        let bidir_rel = create_test_relationship(
            "resource1", "domain1", "resource2", "domain2",
            CrossDomainRelationshipType::Reference, true,
            SyncStrategy::EventDriven, true, // bidirectional = true
        );
        
        // Track sync calls to verify both directions
        let mut sync_calls = Vec::new();
        
        // Register handlers for both directions
        sync_manager.register_sync_handler(
            "domain1",
            "domain2",
            Box::new(move |relationship, direction, _| {
                sync_calls.push(("domain1->domain2".to_string(), direction.clone()));
                Ok(SyncResult::Success {
                    relationship_id: relationship.id.clone(),
                    direction: direction.clone(),
                    metadata: HashMap::new(),
                })
            }),
        )?;
        
        sync_manager.register_sync_handler(
            "domain2",
            "domain1",
            Box::new(move |relationship, direction, _| {
                sync_calls.push(("domain2->domain1".to_string(), direction.clone()));
                Ok(SyncResult::Success {
                    relationship_id: relationship.id.clone(),
                    direction: direction.clone(),
                    metadata: HashMap::new(),
                })
            }),
        )?;
        
        // Synchronize bidirectionally
        let result = sync_manager.synchronize_relationship(
            &bidir_rel,
            SyncDirection::Bidirectional,
            &SyncOptions::default(),
        )?;
        
        // Verify both directions were called
        assert_eq!(sync_calls.len(), 2);
        
        // Verify the directions
        let has_source_to_target = sync_calls.iter().any(|(_, dir)| 
            matches!(dir, SyncDirection::SourceToTarget)
        );
        
        let has_target_to_source = sync_calls.iter().any(|(_, dir)| 
            matches!(dir, SyncDirection::TargetToSource)
        );
        
        assert!(has_source_to_target, "Missing source to target sync");
        assert!(has_target_to_source, "Missing target to source sync");
        
        Ok(())
    }
    
    // Mock functions for testing
    
    // Mock operation manager for testing
    fn mock_operation_manager() -> impl std::any::Any + Send + Sync {
        struct MockOperationManager;
        MockOperationManager
    }
} 