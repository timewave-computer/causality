// Cross-Domain Relationship Scheduler Tests
//
// This file tests the functionality of the cross-domain relationship scheduler,
// which manages automated synchronization of relationships between domains.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    
    use causality::error::Result;
    use causality::resource::relationship::{
        CrossDomainRelationship,
        CrossDomainRelationshipType,
        CrossDomainMetadata,
        CrossDomainRelationshipManager,
        SyncStrategy,
        SchedulerConfig,
        RetryBackoff,
        SchedulerStatus,
        CrossDomainSyncScheduler,
        SyncResult,
        CrossDomainSyncManager,
    };
    
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
    fn test_scheduler_configuration() -> Result<()> {
        // Create default config
        let default_config = SchedulerConfig::default();
        assert!(default_config.enabled);
        assert_eq!(default_config.max_concurrent_tasks, 10);
        
        // Create custom config
        let custom_config = SchedulerConfig {
            enabled: true,
            max_concurrent_tasks: 5,
            periodic_check_interval: Duration::from_secs(30),
            sync_timeout: Duration::from_secs(120),
            retry_failed: true,
            max_retry_attempts: 5,
            retry_backoff: RetryBackoff::Exponential {
                initial: Duration::from_secs(10),
                max: Duration::from_secs(600),
                multiplier: 1.5,
            },
        };
        
        assert_eq!(custom_config.max_concurrent_tasks, 5);
        assert_eq!(custom_config.periodic_check_interval, Duration::from_secs(30));
        
        // Test backoff calculation
        match &custom_config.retry_backoff {
            RetryBackoff::Exponential { initial, max, multiplier } => {
                assert_eq!(*initial, Duration::from_secs(10));
                assert_eq!(*max, Duration::from_secs(600));
                assert_eq!(*multiplier, 1.5);
                
                // Calculate backoff duration for first retry
                let first_retry = custom_config.retry_backoff.calculate(1);
                assert!(first_retry > Duration::from_secs(10)); // Should be more than initial
            },
            _ => panic!("Expected Exponential backoff"),
        }
        
        Ok(())
    }
    
    #[test]
    fn test_scheduler_lifecycle() -> Result<()> {
        // Create test components
        let relationship_manager = Arc::new(CrossDomainRelationshipManager::new());
        
        // We would normally need an OperationManager here, but for testing we can use a mock
        // or rely on the fact that we won't actually execute operations
        let operation_manager = Arc::new(mock_operation_manager());
        let sync_manager = Arc::new(CrossDomainSyncManager::new(operation_manager));
        
        // Create scheduler
        let scheduler = CrossDomainSyncScheduler::new(
            relationship_manager.clone(),
            sync_manager.clone(),
        );
        
        // Set up a minimal configuration to prevent long test times
        let minimal_config = SchedulerConfig {
            enabled: true,
            max_concurrent_tasks: 2,
            periodic_check_interval: Duration::from_millis(100),
            sync_timeout: Duration::from_millis(500),
            retry_failed: false,
            max_retry_attempts: 1,
            retry_backoff: RetryBackoff::Fixed(Duration::from_millis(100)),
        };
        scheduler.set_config(minimal_config.clone())?;
        
        // Add a test relationship
        let test_rel = create_test_relationship(
            "resource1", "domain1", "resource2", "domain2",
            CrossDomainRelationshipType::Mirror, true,
            SyncStrategy::Periodic(Duration::from_millis(100)), true,
        );
        relationship_manager.add_relationship(test_rel.clone())?;
        
        // Start the scheduler
        scheduler.start()?;
        assert_eq!(scheduler.get_status()?, SchedulerStatus::Running);
        
        // Small wait to let scheduler initialize
        thread::sleep(Duration::from_millis(50));
        
        // Pause the scheduler
        scheduler.pause()?;
        assert_eq!(scheduler.get_status()?, SchedulerStatus::Paused);
        
        // Resume the scheduler
        scheduler.resume()?;
        assert_eq!(scheduler.get_status()?, SchedulerStatus::Running);
        
        // Stop the scheduler
        scheduler.stop()?;
        assert_eq!(scheduler.get_status()?, SchedulerStatus::Stopped);
        
        Ok(())
    }
    
    // Mock functions for testing
    
    // Mock operation manager for testing
    fn mock_operation_manager() -> impl std::any::Any + Send + Sync {
        struct MockOperationManager;
        MockOperationManager
    }
} 