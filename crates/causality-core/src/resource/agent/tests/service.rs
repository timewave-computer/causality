// service.rs - Integration tests for the ServiceStatus system
//
// This file contains integration tests for the service status system that allows agents
// to advertise services they offer to other agents in the system.

use crate::resource_types::{ResourceId, ResourceType};
use crate::capability::Capability;
use crate::crypto::ContentHash;
use crate::resource::agent::types::{AgentId, AgentType, AgentState, AgentError};
use crate::resource::agent::agent::{Agent, AgentImpl, AgentBuilder};
use crate::resource::agent::service::{
    ServiceStatus, ServiceState, ServiceVersion, ServiceStatusBuilder,
    ServiceStatusManager, ServiceStatusResult, ServiceStatusError
};

use std::collections::HashMap;
use tokio;

#[tokio::test]
async fn test_service_registration_and_discovery() {
    // Create agent IDs for testing
    let agent1_id = AgentId::from_content_hash(ContentHash::calculate(b"agent1"));
    let agent2_id = AgentId::from_content_hash(ContentHash::calculate(b"agent2"));
    
    // Create a service status manager
    let manager = ServiceStatusManager::new();
    
    // Create service for agent1
    let service1 = ServiceStatusBuilder::new(agent1_id.clone(), "database")
        .version(1, 0, 0)
        .state(ServiceState::Available)
        .require_capability("database.read")
        .require_capability("database.write")
        .endpoint("localhost:5432")
        .description("PostgreSQL Database")
        .with_metadata("region", "us-east")
        .build();
    
    // Create service for agent2
    let service2 = ServiceStatusBuilder::new(agent2_id.clone(), "api")
        .version(2, 1, 0)
        .state(ServiceState::Available)
        .require_capability("api.read")
        .endpoint("https://api.example.com")
        .description("REST API Service")
        .with_metadata("region", "us-west")
        .build();
    
    // Create another service for agent1
    let service3 = ServiceStatusBuilder::new(agent1_id.clone(), "cache")
        .version(1, 5, 2)
        .state(ServiceState::Available)
        .require_capability("cache.read")
        .require_capability("cache.write")
        .endpoint("localhost:6379")
        .description("Redis Cache")
        .with_metadata("region", "us-east")
        .build();
    
    // Register the services
    let service1_id = manager.register_service(service1).await.unwrap();
    let service2_id = manager.register_service(service2).await.unwrap();
    let service3_id = manager.register_service(service3).await.unwrap();
    
    // Test getting all services
    let all_services = manager.get_all_services().await.unwrap();
    assert_eq!(all_services.len(), 3);
    
    // Test getting services by agent
    let agent1_services = manager.get_agent_services(&agent1_id).await.unwrap();
    assert_eq!(agent1_services.len(), 2);
    
    let agent2_services = manager.get_agent_services(&agent2_id).await.unwrap();
    assert_eq!(agent2_services.len(), 1);
    
    // Test getting services by type
    let database_services = manager.get_services_by_type("database").await.unwrap();
    assert_eq!(database_services.len(), 1);
    
    let api_services = manager.get_services_by_type("api").await.unwrap();
    assert_eq!(api_services.len(), 1);
    
    let cache_services = manager.get_services_by_type("cache").await.unwrap();
    assert_eq!(cache_services.len(), 1);
    
    // Test finding available services
    let available_db = manager.find_available_services("database").await.unwrap();
    assert_eq!(available_db.len(), 1);
    
    // Test finding services requiring a capability
    let db_write_services = manager.find_services_requiring_capability("database.write").await.unwrap();
    assert_eq!(db_write_services.len(), 1);
    
    // Test updating service state
    manager.update_service_state(&service1_id, ServiceState::Degraded {
        reason: "High load".to_string(),
    }).await.unwrap();
    
    // Verify state was updated
    let updated_service = manager.get_service(&service1_id).await.unwrap();
    if let ServiceState::Degraded { reason } = updated_service.state() {
        assert_eq!(reason, "High load");
    } else {
        panic!("Expected Degraded state");
    }
    
    // Now available services should not include the database
    let available_db = manager.find_available_services("database").await.unwrap();
    assert_eq!(available_db.len(), 0);
    
    // Test unregistering a service
    manager.unregister_service(&service3_id).await.unwrap();
    
    // Verify service was removed
    let agent1_services = manager.get_agent_services(&agent1_id).await.unwrap();
    assert_eq!(agent1_services.len(), 1);
    
    let all_services = manager.get_all_services().await.unwrap();
    assert_eq!(all_services.len(), 2);
}

#[tokio::test]
async fn test_service_access_control() {
    // Create a service status manager
    let manager = ServiceStatusManager::new();
    
    // Create agents
    let agent1 = AgentBuilder::new()
        .agent_type(AgentType::Operator)
        .state(AgentState::Active)
        .build();
    
    let agent2 = AgentBuilder::new()
        .agent_type(AgentType::User)
        .state(AgentState::Active)
        .build();
    
    // Add capabilities to agents
    let mut agent1_mut = agent1.clone();
    let mut agent2_mut = agent2.clone();
    
    // Add database capabilities to agent1
    let db_read = Capability::new("database.read", None);
    let db_write = Capability::new("database.write", None);
    agent1_mut.add_capability(db_read).await.unwrap();
    agent1_mut.add_capability(db_write).await.unwrap();
    
    // Add api capabilities to agent2
    let api_read = Capability::new("api.read", None);
    agent2_mut.add_capability(api_read).await.unwrap();
    
    // Create services
    let service1 = ServiceStatusBuilder::new(agent1.agent_id().clone(), "database")
        .state(ServiceState::Available)
        .require_capability("database.read")
        .require_capability("database.write")
        .build();
    
    let service2 = ServiceStatusBuilder::new(agent2.agent_id().clone(), "api")
        .state(ServiceState::Available)
        .require_capability("api.read")
        .build();
    
    // Register services
    let service1_id = manager.register_service(service1).await.unwrap();
    let service2_id = manager.register_service(service2).await.unwrap();
    
    // Test service access
    
    // Agent1 should be able to access both services
    let can_access_db = manager.can_access_service(&agent1_mut, &service1_id).await.unwrap();
    assert!(can_access_db);
    
    let can_access_api = manager.can_access_service(&agent1_mut, &service2_id).await.unwrap();
    assert!(!can_access_api);
    
    // Agent2 should only be able to access the API service
    let can_access_db = manager.can_access_service(&agent2_mut, &service1_id).await.unwrap();
    assert!(!can_access_db);
    
    let can_access_api = manager.can_access_service(&agent2_mut, &service2_id).await.unwrap();
    assert!(can_access_api);
}

#[tokio::test]
async fn test_service_state_transitions() {
    // Create a service status manager
    let manager = ServiceStatusManager::new();
    
    // Create agent
    let agent_id = AgentId::from_content_hash(ContentHash::calculate(b"agent"));
    
    // Create service
    let service = ServiceStatusBuilder::new(agent_id.clone(), "webserver")
        .state(ServiceState::Available)
        .build();
    
    // Register service
    let service_id = manager.register_service(service).await.unwrap();
    
    // Test state transitions
    
    // Available -> Degraded
    manager.update_service_state(&service_id, ServiceState::Degraded {
        reason: "High CPU usage".to_string(),
    }).await.unwrap();
    
    let service = manager.get_service(&service_id).await.unwrap();
    if let ServiceState::Degraded { reason } = service.state() {
        assert_eq!(reason, "High CPU usage");
    } else {
        panic!("Expected Degraded state");
    }
    
    // Degraded -> Maintenance
    let now = chrono::Utc::now().timestamp() as u64;
    manager.update_service_state(&service_id, ServiceState::Maintenance {
        reason: "Scheduled maintenance".to_string(),
        expected_end: Some(now + 3600), // 1 hour from now
    }).await.unwrap();
    
    let service = manager.get_service(&service_id).await.unwrap();
    if let ServiceState::Maintenance { reason, expected_end } = service.state() {
        assert_eq!(reason, "Scheduled maintenance");
        assert!(expected_end.is_some());
    } else {
        panic!("Expected Maintenance state");
    }
    
    // Maintenance -> Unavailable
    manager.update_service_state(&service_id, ServiceState::Unavailable)
        .await.unwrap();
    
    let service = manager.get_service(&service_id).await.unwrap();
    assert!(matches!(service.state(), ServiceState::Unavailable));
    
    // Unavailable -> Available
    manager.update_service_state(&service_id, ServiceState::Available)
        .await.unwrap();
    
    let service = manager.get_service(&service_id).await.unwrap();
    assert!(matches!(service.state(), ServiceState::Available));
}

#[tokio::test]
async fn test_service_metadata() {
    // Create a service status manager
    let manager = ServiceStatusManager::new();
    
    // Create agent
    let agent_id = AgentId::from_content_hash(ContentHash::calculate(b"agent"));
    
    // Create service with metadata
    let mut service = ServiceStatusBuilder::new(agent_id.clone(), "database")
        .state(ServiceState::Available)
        .with_metadata("region", "us-east")
        .with_metadata("instance-type", "t3.large")
        .build();
    
    // Add more metadata
    service.set_metadata("cluster", "prod-cluster-1");
    
    // Register service
    let service_id = manager.register_service(service).await.unwrap();
    
    // Retrieve and check metadata
    let service = manager.get_service(&service_id).await.unwrap();
    assert_eq!(service.get_metadata("region"), Some(&"us-east".to_string()));
    assert_eq!(service.get_metadata("instance-type"), Some(&"t3.large".to_string()));
    assert_eq!(service.get_metadata("cluster"), Some(&"prod-cluster-1".to_string()));
    
    // Non-existent metadata should return None
    assert_eq!(service.get_metadata("non-existent"), None);
}

#[tokio::test]
async fn test_service_versioning() {
    // Create a service status manager
    let manager = ServiceStatusManager::new();
    
    // Create agent
    let agent_id = AgentId::from_content_hash(ContentHash::calculate(b"agent"));
    
    // Create service with version information
    let service1 = ServiceStatusBuilder::new(agent_id.clone(), "api")
        .version(1, 0, 0)
        .state(ServiceState::Available)
        .build();
    
    let service2 = ServiceStatusBuilder::new(agent_id.clone(), "api")
        .version(1, 1, 0)
        .pre_release("alpha")
        .state(ServiceState::Available)
        .build();
    
    let service3 = ServiceStatusBuilder::new(agent_id.clone(), "api")
        .version(2, 0, 0)
        .state(ServiceState::Available)
        .build();
    
    // Register services
    let service1_id = manager.register_service(service1).await.unwrap();
    let service2_id = manager.register_service(service2).await.unwrap();
    let service3_id = manager.register_service(service3).await.unwrap();
    
    // Get services by type
    let api_services = manager.get_services_by_type("api").await.unwrap();
    assert_eq!(api_services.len(), 3);
    
    // Check versions
    let service1 = manager.get_service(&service1_id).await.unwrap();
    assert_eq!(service1.version().major, 1);
    assert_eq!(service1.version().minor, 0);
    assert_eq!(service1.version().patch, 0);
    assert_eq!(service1.version().pre_release, None);
    
    let service2 = manager.get_service(&service2_id).await.unwrap();
    assert_eq!(service2.version().major, 1);
    assert_eq!(service2.version().minor, 1);
    assert_eq!(service2.version().patch, 0);
    assert_eq!(service2.version().pre_release, Some("alpha".to_string()));
    
    let service3 = manager.get_service(&service3_id).await.unwrap();
    assert_eq!(service3.version().major, 2);
    assert_eq!(service3.version().minor, 0);
    assert_eq!(service3.version().patch, 0);
    assert_eq!(service3.version().pre_release, None);
} 