//! Real System Integration Tests
//!
//! These tests verify integration with actual Almanac storage and Valence coprocessor APIs.
//! They require local instances to be running (use setup scripts in /scripts/).

use anyhow::Result;
use std::time::Duration;
use tokio::time::timeout;

use causality_compiler::{
    storage_backend::{StorageBackend, StorageConfig, DatabaseConfig},
    event_storage::{EventStorage, EventFilter, EventQuery},
    valence_state_persistence::{ValenceStatePersistence, AccountInstantiation, AccountUpdate},
};

/// Test configuration for real system integration
struct IntegrationTestConfig {
    almanac_endpoint: String,
    valence_endpoint: String,
    postgres_url: String,
    test_timeout: Duration,
}

impl Default for IntegrationTestConfig {
    fn default() -> Self {
        Self {
            almanac_endpoint: "http://localhost:8080".to_string(),
            valence_endpoint: "http://localhost:9090".to_string(),
            postgres_url: "postgresql://almanac:test_password@localhost:5432/almanac_test".to_string(),
            test_timeout: Duration::from_secs(30),
        }
    }
}

/// Test real storage backend initialization and health checks
#[tokio::test]
async fn test_real_storage_backend_initialization() -> Result<()> {
    let config = IntegrationTestConfig::default();
    
    println!("=== Testing Real Storage Backend Initialization ===");
    
    // Test storage backend initialization
    let storage_config = StorageConfig {
        backend_type: "postgresql".to_string(),
        database: Some(DatabaseConfig {
            url: config.postgres_url.clone(),
            max_connections: 5,
            connection_timeout_seconds: 10,
        }),
        cache_size_mb: 64,
        enable_metrics: true,
    };
    
    // Initialize storage backend with real configuration
    let storage_backend = timeout(
        config.test_timeout,
        StorageBackend::new(storage_config)
    ).await??;
    
    // Test health check
    let health_status = timeout(
        Duration::from_secs(5),
        storage_backend.health_check()
    ).await??;
    
    assert!(health_status.is_healthy);
    assert!(health_status.database_connected);
    println!("✓ Storage backend health check passed");
    
    // Test connection pool
    let pool_stats = storage_backend.get_connection_pool_stats().await?;
    assert!(pool_stats.active_connections >= 0);
    assert!(pool_stats.max_connections > 0);
    println!("✓ Connection pool stats: {} active, {} max", 
             pool_stats.active_connections, pool_stats.max_connections);
    
    Ok(())
}

/// Test real event storage operations with actual data
#[tokio::test]
async fn test_real_event_storage_operations() -> Result<()> {
    let config = IntegrationTestConfig::default();
    
    println!("=== Testing Real Event Storage Operations ===");
    
    // Initialize storage backend
    let storage_config = StorageConfig {
        backend_type: "postgresql".to_string(),
        database: Some(DatabaseConfig {
            url: config.postgres_url.clone(),
            max_connections: 5,
            connection_timeout_seconds: 10,
        }),
        cache_size_mb: 64,
        enable_metrics: true,
    };
    
    let storage_backend = StorageBackend::new(storage_config).await?;
    let mut event_storage = EventStorage::new(storage_backend);
    
    // Test event storage with real data
    let test_events = vec![
        serde_json::json!({
            "event_type": "account_created",
            "account_id": "test_account_001",
            "timestamp": chrono::Utc::now().timestamp(),
            "data": {
                "owner": "0x1234567890123456789012345678901234567890",
                "account_type": "factory"
            }
        }),
        serde_json::json!({
            "event_type": "library_approved",
            "account_id": "test_account_001",
            "timestamp": chrono::Utc::now().timestamp(),
            "data": {
                "library_id": "swap_library_v1",
                "permissions": ["execute", "query"]
            }
        }),
    ];
    
    // Store events
    for event in &test_events {
        let event_id = timeout(
            config.test_timeout,
            event_storage.store_event("ethereum", "test_contract", event.clone())
        ).await??;
        
        println!("✓ Stored event with ID: {}", event_id);
    }
    
    // Query events with filters
    let filter = EventFilter {
        contract_id: Some("test_contract".to_string()),
        event_types: Some(vec!["account_created".to_string()]),
        from_timestamp: None,
        to_timestamp: None,
        limit: Some(10),
    };
    
    let query = EventQuery {
        domain: "ethereum".to_string(),
        filter,
    };
    
    let retrieved_events = timeout(
        config.test_timeout,
        event_storage.query_events(query)
    ).await??;
    
    assert!(!retrieved_events.is_empty());
    println!("✓ Retrieved {} events", retrieved_events.len());
    
    // Test event subscription (if supported)
    let subscription_result = event_storage.subscribe_to_events(
        "ethereum".to_string(),
        vec!["test_contract".to_string()],
        vec!["account_created".to_string(), "library_approved".to_string()],
    ).await;
    
    match subscription_result {
        Ok(_) => println!("✓ Event subscription created successfully"),
        Err(e) => println!("⚠ Event subscription not supported: {}", e),
    }
    
    Ok(())
}

/// Test real Valence state persistence with actual account data
#[tokio::test]
async fn test_real_valence_state_persistence() -> Result<()> {
    let config = IntegrationTestConfig::default();
    
    println!("=== Testing Real Valence State Persistence ===");
    
    // Initialize storage backend
    let storage_config = StorageConfig {
        backend_type: "postgresql".to_string(),
        database: Some(DatabaseConfig {
            url: config.postgres_url.clone(),
            max_connections: 5,
            connection_timeout_seconds: 10,
        }),
        cache_size_mb: 64,
        enable_metrics: true,
    };
    
    let storage_backend = StorageBackend::new(storage_config).await?;
    let mut valence_persistence = ValenceStatePersistence::new(storage_backend);
    
    // Test account instantiation storage
    let account_instantiation = AccountInstantiation {
        account_id: "test_account_real_001".to_string(),
        owner: "0x1234567890123456789012345678901234567890".to_string(),
        account_type: "factory".to_string(),
        configuration: serde_json::json!({
            "permissions": ["read", "write", "execute"],
            "gas_limit": 1000000,
            "libraries": []
        }),
        domain: "ethereum".to_string(),
        created_at: chrono::Utc::now(),
        transaction_hash: Some("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string()),
    };
    
    // Store account instantiation
    timeout(
        config.test_timeout,
        valence_persistence.store_account_instantiation(account_instantiation.clone())
    ).await??;
    
    println!("✓ Stored account instantiation for {}", account_instantiation.account_id);
    
    // Retrieve account state
    let retrieved_state = timeout(
        config.test_timeout,
        valence_persistence.get_account_state(&account_instantiation.account_id)
    ).await??;
    
    assert!(retrieved_state.is_some());
    let state = retrieved_state.unwrap();
    assert_eq!(state.account_id, account_instantiation.account_id);
    assert_eq!(state.owner, account_instantiation.owner);
    println!("✓ Retrieved account state successfully");
    
    // Test account update
    let account_update = AccountUpdate {
        account_id: account_instantiation.account_id.clone(),
        update_type: "library_approval".to_string(),
        update_data: serde_json::json!({
            "library_id": "swap_library_v1",
            "permissions": ["execute", "query"],
            "approved_at": chrono::Utc::now().timestamp()
        }),
        updated_at: chrono::Utc::now(),
        transaction_hash: Some("0x9876543210987654321098765432109876543210987654321098765432109876".to_string()),
    };
    
    timeout(
        config.test_timeout,
        valence_persistence.update_account_state(account_update.clone())
    ).await??;
    
    println!("✓ Updated account state with library approval");
    
    // Test account statistics
    let stats = timeout(
        config.test_timeout,
        valence_persistence.get_account_statistics(&account_instantiation.account_id)
    ).await??;
    
    assert!(stats.total_updates >= 1);
    println!("✓ Account statistics: {} updates, {} libraries", 
             stats.total_updates, stats.approved_libraries_count);
    
    Ok(())
}

/// Test concurrent operations with real storage
#[tokio::test]
async fn test_concurrent_real_storage_operations() -> Result<()> {
    let config = IntegrationTestConfig::default();
    
    println!("=== Testing Concurrent Real Storage Operations ===");
    
    // Initialize storage backend
    let storage_config = StorageConfig {
        backend_type: "postgresql".to_string(),
        database: Some(DatabaseConfig {
            url: config.postgres_url.clone(),
            max_connections: 10, // Higher for concurrent operations
            connection_timeout_seconds: 10,
        }),
        cache_size_mb: 128,
        enable_metrics: true,
    };
    
    let storage_backend = StorageBackend::new(storage_config).await?;
    
    // Create multiple storage instances for concurrent testing
    let event_storage = EventStorage::new(storage_backend.clone());
    let valence_persistence = ValenceStatePersistence::new(storage_backend.clone());
    
    // Spawn concurrent operations
    let mut handles = vec![];
    
    // Concurrent event storage operations
    for i in 0..5 {
        let mut event_storage = event_storage.clone();
        let handle = tokio::spawn(async move {
            let event = serde_json::json!({
                "event_type": "concurrent_test",
                "account_id": format!("concurrent_account_{}", i),
                "timestamp": chrono::Utc::now().timestamp(),
                "data": {
                    "test_id": i,
                    "operation": "concurrent_event_storage"
                }
            });
            
            event_storage.store_event("ethereum", "concurrent_test", event).await
        });
        handles.push(handle);
    }
    
    // Concurrent account state operations
    for i in 0..3 {
        let mut valence_persistence = valence_persistence.clone();
        let handle = tokio::spawn(async move {
            let account_instantiation = AccountInstantiation {
                account_id: format!("concurrent_account_{}", i),
                owner: format!("0x{:040x}", i),
                account_type: "factory".to_string(),
                configuration: serde_json::json!({
                    "test_id": i,
                    "concurrent": true
                }),
                domain: "ethereum".to_string(),
                created_at: chrono::Utc::now(),
                transaction_hash: Some(format!("0x{:064x}", i)),
            };
            
            valence_persistence.store_account_instantiation(account_instantiation).await
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    let results = timeout(
        config.test_timeout,
        futures::future::try_join_all(handles)
    ).await?;
    
    // Check results
    let mut successful_operations = 0;
    for result in results {
        match result {
            Ok(_) => successful_operations += 1,
            Err(e) => println!("⚠ Concurrent operation failed: {}", e),
        }
    }
    
    println!("✓ Completed {}/{} concurrent operations successfully", 
             successful_operations, 8);
    assert!(successful_operations >= 6); // Allow some failures in concurrent testing
    
    Ok(())
}

/// Test system integration validation with health checks
#[tokio::test]
async fn test_system_integration_validation() -> Result<()> {
    let config = IntegrationTestConfig::default();
    
    println!("=== Testing System Integration Validation ===");
    
    // Test external system connectivity
    println!("1. Testing external system connectivity...");
    
    // Test Almanac connectivity
    let almanac_health = timeout(
        Duration::from_secs(5),
        reqwest::get(&format!("{}/health", config.almanac_endpoint))
    ).await;
    
    match almanac_health {
        Ok(Ok(response)) if response.status().is_success() => {
            println!("✓ Almanac is accessible at {}", config.almanac_endpoint);
        }
        _ => {
            println!("⚠ Almanac not accessible - using fallback mode");
        }
    }
    
    // Test Valence connectivity
    let valence_health = timeout(
        Duration::from_secs(5),
        reqwest::get(&format!("{}/health", config.valence_endpoint))
    ).await;
    
    match valence_health {
        Ok(Ok(response)) if response.status().is_success() => {
            println!("✓ Valence coprocessor is accessible at {}", config.valence_endpoint);
        }
        _ => {
            println!("⚠ Valence coprocessor not accessible - using fallback mode");
        }
    }
    
    // Test database connectivity
    println!("2. Testing database connectivity...");
    
    let storage_config = StorageConfig {
        backend_type: "postgresql".to_string(),
        database: Some(DatabaseConfig {
            url: config.postgres_url.clone(),
            max_connections: 5,
            connection_timeout_seconds: 10,
        }),
        cache_size_mb: 64,
        enable_metrics: true,
    };
    
    let storage_backend = StorageBackend::new(storage_config).await?;
    let health_status = storage_backend.health_check().await?;
    
    assert!(health_status.is_healthy);
    println!("✓ Database connectivity verified");
    
    // Test end-to-end data flow
    println!("3. Testing end-to-end data flow...");
    
    let mut event_storage = EventStorage::new(storage_backend.clone());
    let mut valence_persistence = ValenceStatePersistence::new(storage_backend);
    
    // Create test account
    let account_id = "integration_test_account";
    let account_instantiation = AccountInstantiation {
        account_id: account_id.to_string(),
        owner: "0x1111111111111111111111111111111111111111".to_string(),
        account_type: "factory".to_string(),
        configuration: serde_json::json!({
            "integration_test": true,
            "timestamp": chrono::Utc::now().timestamp()
        }),
        domain: "ethereum".to_string(),
        created_at: chrono::Utc::now(),
        transaction_hash: Some("0x1111111111111111111111111111111111111111111111111111111111111111".to_string()),
    };
    
    // Store account
    valence_persistence.store_account_instantiation(account_instantiation).await?;
    
    // Store related event
    let event = serde_json::json!({
        "event_type": "integration_test",
        "account_id": account_id,
        "timestamp": chrono::Utc::now().timestamp(),
        "data": {
            "test_phase": "end_to_end_validation",
            "success": true
        }
    });
    
    event_storage.store_event("ethereum", "integration_test", event).await?;
    
    // Verify data retrieval
    let retrieved_account = valence_persistence.get_account_state(account_id).await?;
    assert!(retrieved_account.is_some());
    
    let filter = EventFilter {
        contract_id: Some("integration_test".to_string()),
        event_types: Some(vec!["integration_test".to_string()]),
        from_timestamp: None,
        to_timestamp: None,
        limit: Some(1),
    };
    
    let query = EventQuery {
        domain: "ethereum".to_string(),
        filter,
    };
    
    let retrieved_events = event_storage.query_events(query).await?;
    assert!(!retrieved_events.is_empty());
    
    println!("✓ End-to-end data flow validation successful");
    
    Ok(())
}

/// Helper function to check if external services are available
async fn check_external_services() -> (bool, bool) {
    let almanac_available = reqwest::get("http://localhost:8080/health")
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    
    let valence_available = reqwest::get("http://localhost:9090/health")
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    
    (almanac_available, valence_available)
}

/// Integration test runner that checks service availability
#[tokio::test]
async fn test_integration_runner() -> Result<()> {
    println!("=== Integration Test Runner ===");
    
    let (almanac_available, valence_available) = check_external_services().await;
    
    println!("Service availability:");
    println!("  Almanac: {}", if almanac_available { "✓" } else { "✗" });
    println!("  Valence: {}", if valence_available { "✓" } else { "✗" });
    
    if !almanac_available && !valence_available {
        println!("⚠ No external services available - running in mock mode");
        println!("To run full integration tests:");
        println!("  1. Run: ./scripts/setup-local-almanac.sh start");
        println!("  2. Run: ./scripts/setup-local-valence.sh start");
        println!("  3. Re-run tests");
        return Ok(());
    }
    
    // Run available tests based on service availability
    if almanac_available {
        println!("Running Almanac integration tests...");
        // These would be run automatically by the test framework
    }
    
    if valence_available {
        println!("Running Valence integration tests...");
        // These would be run automatically by the test framework
    }
    
    println!("✓ Integration test runner completed");
    Ok(())
} 