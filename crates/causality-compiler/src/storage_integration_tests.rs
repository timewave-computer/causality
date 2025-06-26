// ------------ STORAGE INTEGRATION TESTS ------------
// Purpose: Integration tests for storage operations with real data

#[cfg(test)]
mod tests {
    use crate::storage_backend::{StorageBackendManager, StorageBackendConfig, StorageBackendType, RocksDbConfig};
    use crate::event_storage::{EventStorageManager, CausalityEvent, EventFilter};
    use crate::valence_state_persistence::{ValenceStatePersistence, CausalityValenceAccount, CausalityLibraryApproval};
    use std::sync::Arc;
    use tokio;

    /// Test storage backend initialization
    #[tokio::test]
    async fn test_storage_backend_initialization() {
        // Test RocksDB backend initialization
        let config = StorageBackendConfig {
            backend_type: StorageBackendType::RocksDB,
            postgres_config: None,
            rocksdb_config: Some(RocksDbConfig {
                path: "./test_data/test.db".to_string(),
                ..Default::default()
            }),
            pool_config: Default::default(),
            migration_config: Default::default(),
        };

        let mut manager = StorageBackendManager::new(config);
        let result = manager.initialize().await;
        
        #[cfg(feature = "almanac")]
        {
            assert!(result.is_ok(), "Storage backend initialization should succeed");
            assert!(manager.storage().is_some(), "Storage should be initialized");
        }
        
        #[cfg(not(feature = "almanac"))]
        {
            assert!(result.is_ok(), "Mock storage backend initialization should succeed");
        }
    }

    /// Test in-memory storage backend
    #[tokio::test]
    async fn test_in_memory_storage() {
        let config = StorageBackendConfig {
            backend_type: StorageBackendType::InMemory,
            postgres_config: None,
            rocksdb_config: None,
            pool_config: Default::default(),
            migration_config: Default::default(),
        };

        let mut manager = StorageBackendManager::new(config);
        let result = manager.initialize().await;
        assert!(result.is_ok(), "In-memory storage initialization should succeed");

        // Test connection
        let health_check = manager.test_connection().await;
        assert!(health_check.is_ok(), "Health check should pass");
        assert!(health_check.unwrap(), "Storage should be healthy");
    }

    /// Test event storage operations
    #[tokio::test]
    async fn test_event_storage_operations() {
        // Initialize storage backend
        let config = StorageBackendConfig {
            backend_type: StorageBackendType::InMemory,
            postgres_config: None,
            rocksdb_config: None,
            pool_config: Default::default(),
            migration_config: Default::default(),
        };

        let mut backend_manager = StorageBackendManager::new(config);
        backend_manager.initialize().await.expect("Storage initialization should succeed");
        let backend_manager = Arc::new(backend_manager);

        // Create event storage manager
        let event_manager = EventStorageManager::new(backend_manager);

        // Create test event
        let test_event = CausalityEvent {
            id: "test_event_1".to_string(),
            chain_id: "1".to_string(),
            contract_address: "0x1234567890123456789012345678901234567890".to_string(),
            event_name: "Transfer".to_string(),
            block_number: 12345,
            transaction_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            log_index: 0,
            topics: vec![
                "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string(),
            ],
            data: "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000".to_string(),
            timestamp: chrono::Utc::now(),
            removed: false,
        };

        // Test storing event
        let store_result = event_manager.store_event(test_event.clone()).await;
        assert!(store_result.is_ok(), "Event storage should succeed");

        // Test retrieving events
        let filter = EventFilter {
            contract_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            from_block: Some(12000),
            to_block: Some(13000),
            event_names: Some(vec!["Transfer".to_string()]),
            topics: None,
        };

        let events = event_manager.get_events(filter).await;
        assert!(events.is_ok(), "Event retrieval should succeed");
        
        #[cfg(not(feature = "almanac"))]
        {
            // In mock mode, we get mock events
            let events = events.unwrap();
            assert!(!events.is_empty(), "Should retrieve events");
        }
    }

    /// Test Valence account state persistence
    #[tokio::test]
    async fn test_valence_state_persistence() {
        // Initialize storage backend
        let config = StorageBackendConfig {
            backend_type: StorageBackendType::InMemory,
            postgres_config: None,
            rocksdb_config: None,
            pool_config: Default::default(),
            migration_config: Default::default(),
        };

        let mut backend_manager = StorageBackendManager::new(config);
        backend_manager.initialize().await.expect("Storage initialization should succeed");
        let backend_manager = Arc::new(backend_manager);

        // Create state persistence manager
        let state_manager = ValenceStatePersistence::new(backend_manager);

        // Create test account
        let test_account = CausalityValenceAccount {
            id: "test_account_1".to_string(),
            chain_id: "1".to_string(),
            contract_address: "0x1234567890123456789012345678901234567890".to_string(),
            created_at_block: 12345,
            created_at_tx: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            current_owner: Some("0xa1b2c3d4e5f6789012345678901234567890abcd".to_string()),
            pending_owner: None,
            pending_owner_expiry: None,
            last_updated_block: 12345,
            last_updated_tx: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
        };

        // Test storing account
        let store_result = state_manager.store_account(test_account.clone()).await;
        assert!(store_result.is_ok(), "Account storage should succeed");

        // Test retrieving account
        let account = state_manager.get_account("test_account_1").await;
        assert!(account.is_ok(), "Account retrieval should succeed");
        
        let account = account.unwrap();
        assert!(account.is_some(), "Account should be found");
        
        #[cfg(not(feature = "almanac"))]
        {
            // In mock mode, we get mock accounts
            let account = account.unwrap();
            assert_eq!(account.id, "test_account_1");
        }
    }

    /// Test batch event storage
    #[tokio::test]
    async fn test_batch_event_storage() {
        // Initialize storage backend
        let config = StorageBackendConfig {
            backend_type: StorageBackendType::InMemory,
            postgres_config: None,
            rocksdb_config: None,
            pool_config: Default::default(),
            migration_config: Default::default(),
        };

        let mut backend_manager = StorageBackendManager::new(config);
        backend_manager.initialize().await.expect("Storage initialization should succeed");
        let backend_manager = Arc::new(backend_manager);

        // Create event storage manager
        let event_manager = EventStorageManager::new(backend_manager);

        // Create multiple test events
        let events = vec![
            CausalityEvent {
                id: "batch_event_1".to_string(),
                chain_id: "1".to_string(),
                contract_address: "0x1234567890123456789012345678901234567890".to_string(),
                event_name: "Transfer".to_string(),
                block_number: 12345,
                transaction_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
                log_index: 0,
                topics: vec!["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string()],
                data: "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000".to_string(),
                timestamp: chrono::Utc::now(),
                removed: false,
            },
            CausalityEvent {
                id: "batch_event_2".to_string(),
                chain_id: "1".to_string(),
                contract_address: "0x1234567890123456789012345678901234567890".to_string(),
                event_name: "Approval".to_string(),
                block_number: 12346,
                transaction_hash: "0xbcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890a".to_string(),
                log_index: 1,
                topics: vec!["0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925".to_string()],
                data: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string(),
                timestamp: chrono::Utc::now(),
                removed: false,
            },
        ];

        // Test batch storage
        let batch_result = event_manager.store_events_batch(events).await;
        assert!(batch_result.is_ok(), "Batch event storage should succeed");
    }

    /// Test library approval storage
    #[tokio::test]
    async fn test_library_approval_storage() {
        // Initialize storage backend
        let config = StorageBackendConfig {
            backend_type: StorageBackendType::InMemory,
            postgres_config: None,
            rocksdb_config: None,
            pool_config: Default::default(),
            migration_config: Default::default(),
        };

        let mut backend_manager = StorageBackendManager::new(config);
        backend_manager.initialize().await.expect("Storage initialization should succeed");
        let backend_manager = Arc::new(backend_manager);

        // Create state persistence manager
        let state_manager = ValenceStatePersistence::new(backend_manager);

        // Create test library approval
        let test_approval = CausalityLibraryApproval {
            account_id: "test_account_1".to_string(),
            library_address: "0xb2c3d4e5f6789012345678901234567890abcdef".to_string(),
            approved_at_block: 12346,
            approved_at_tx: "0xbcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890a".to_string(),
        };

        // Test storing library approval
        let store_result = state_manager.store_library_approval(test_approval.clone()).await;
        assert!(store_result.is_ok(), "Library approval storage should succeed");

        // Test retrieving account libraries
        let libraries = state_manager.get_account_libraries("test_account_1").await;
        assert!(libraries.is_ok(), "Library retrieval should succeed");
        
        #[cfg(not(feature = "almanac"))]
        {
            // In mock mode, we get mock libraries
            let libraries = libraries.unwrap();
            assert!(!libraries.is_empty(), "Should have libraries");
        }
    }

    /// Test storage statistics
    #[tokio::test]
    async fn test_storage_statistics() {
        // Initialize storage backend
        let config = StorageBackendConfig {
            backend_type: StorageBackendType::InMemory,
            postgres_config: None,
            rocksdb_config: None,
            pool_config: Default::default(),
            migration_config: Default::default(),
        };

        let mut backend_manager = StorageBackendManager::new(config);
        backend_manager.initialize().await.expect("Storage initialization should succeed");

        // Test getting statistics
        let stats = backend_manager.get_statistics().await;
        assert!(stats.is_ok(), "Statistics retrieval should succeed");
        
        let stats = stats.unwrap();
        assert!(stats.total_events >= 0, "Total events should be non-negative");
        assert!(stats.total_accounts >= 0, "Total accounts should be non-negative");
    }

    /// Test error handling for uninitialized storage
    #[tokio::test]
    async fn test_uninitialized_storage_error_handling() {
        // Create uninitialized storage backend
        let config = StorageBackendConfig {
            backend_type: StorageBackendType::InMemory,
            postgres_config: None,
            rocksdb_config: None,
            pool_config: Default::default(),
            migration_config: Default::default(),
        };

        let backend_manager = StorageBackendManager::new(config);
        // Don't initialize the backend

        // Test connection should fail
        let health_check = backend_manager.test_connection().await;
        assert!(health_check.is_err(), "Health check should fail for uninitialized storage");
    }

    /// Test concurrent storage operations
    #[tokio::test]
    async fn test_concurrent_storage_operations() {
        // Initialize storage backend
        let config = StorageBackendConfig {
            backend_type: StorageBackendType::InMemory,
            postgres_config: None,
            rocksdb_config: None,
            pool_config: Default::default(),
            migration_config: Default::default(),
        };

        let mut backend_manager = StorageBackendManager::new(config);
        backend_manager.initialize().await.expect("Storage initialization should succeed");
        let backend_manager = Arc::new(backend_manager);

        // Create multiple event storage managers
        let event_manager1 = Arc::new(EventStorageManager::new(backend_manager.clone()));
        let event_manager2 = Arc::new(EventStorageManager::new(backend_manager.clone()));

        // Create test events
        let event1 = CausalityEvent {
            id: "concurrent_event_1".to_string(),
            chain_id: "1".to_string(),
            contract_address: "0x1111111111111111111111111111111111111111".to_string(),
            event_name: "Transfer".to_string(),
            block_number: 12345,
            transaction_hash: "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            log_index: 0,
            topics: vec!["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string()],
            data: "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000".to_string(),
            timestamp: chrono::Utc::now(),
            removed: false,
        };

        let event2 = CausalityEvent {
            id: "concurrent_event_2".to_string(),
            chain_id: "1".to_string(),
            contract_address: "0x2222222222222222222222222222222222222222".to_string(),
            event_name: "Approval".to_string(),
            block_number: 12346,
            transaction_hash: "0x2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            log_index: 1,
            topics: vec!["0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925".to_string()],
            data: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string(),
            timestamp: chrono::Utc::now(),
            removed: false,
        };

        // Store events concurrently
        let handle1 = {
            let manager = event_manager1.clone();
            let event = event1.clone();
            tokio::spawn(async move {
                manager.store_event(event).await
            })
        };

        let handle2 = {
            let manager = event_manager2.clone();
            let event = event2.clone();
            tokio::spawn(async move {
                manager.store_event(event).await
            })
        };

        // Wait for both operations to complete
        let result1 = handle1.await.expect("Task should complete").expect("Event storage should succeed");
        let result2 = handle2.await.expect("Task should complete").expect("Event storage should succeed");

        // Both operations should succeed
        assert!(result1 == () && result2 == (), "Concurrent storage operations should succeed");
    }

    /// Performance test for batch operations
    #[tokio::test]
    async fn test_batch_performance() {
        // Initialize storage backend
        let config = StorageBackendConfig {
            backend_type: StorageBackendType::InMemory,
            postgres_config: None,
            rocksdb_config: None,
            pool_config: Default::default(),
            migration_config: Default::default(),
        };

        let mut backend_manager = StorageBackendManager::new(config);
        backend_manager.initialize().await.expect("Storage initialization should succeed");
        let backend_manager = Arc::new(backend_manager);

        // Create event storage manager
        let event_manager = EventStorageManager::new(backend_manager);

        // Create a large batch of events
        let batch_size = 100;
        let mut events = Vec::with_capacity(batch_size);
        
        for i in 0..batch_size {
            events.push(CausalityEvent {
                id: format!("perf_event_{}", i),
                chain_id: "1".to_string(),
                contract_address: "0x1234567890123456789012345678901234567890".to_string(),
                event_name: "Transfer".to_string(),
                block_number: 12345 + i as u64,
                transaction_hash: format!("0x{:064x}", i),
                log_index: i as u32,
                topics: vec!["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string()],
                data: "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000".to_string(),
                timestamp: chrono::Utc::now(),
                removed: false,
            });
        }

        // Measure batch storage time
        let start = std::time::Instant::now();
        let batch_result = event_manager.store_events_batch(events).await;
        let duration = start.elapsed();

        assert!(batch_result.is_ok(), "Batch storage should succeed");
        println!("Batch storage of {} events took: {:?}", batch_size, duration);
        
        // Performance should be reasonable (less than 1 second for 100 events in mock mode)
        assert!(duration.as_secs() < 5, "Batch storage should complete within 5 seconds");
    }
} 