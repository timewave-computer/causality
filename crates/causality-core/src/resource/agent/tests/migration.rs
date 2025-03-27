// migration.rs - Tests to verify agent resource system can replace actor system functionality
//
// This file contains tests that validate the agent resource system can successfully
// replace the actor system for all key functionality.

use crate::resource::{
    agent::{
        agent::{Agent, AgentImpl},
        types::{AgentId, AgentType, AgentState, AgentRelationship},
        messaging::{Message, MessageFactory, MessageType, MessageRouter},
        registry::AgentRegistry,
        service::{ServiceStatus, ServiceState},
        operation::{Operation, OperationBuilder, OperationType},
        capability::CapabilityBundle
    },
    ResourceId, ResourceType, Resource
};
use crate::capability::Capability;
use crate::effect::{Effect, EffectContext, EffectOutcome};
use crate::crypto::ContentHash;

use std::collections::HashMap;
use std::sync::Arc;

/// Test helper to create a test agent
fn create_test_agent(name: &str, agent_type: AgentType) -> AgentImpl {
    let metadata = HashMap::from([
        ("name".to_string(), name.to_string())
    ]);
    
    AgentImpl::new(
        agent_type,
        Some(AgentState::Active),
        Some(vec![Capability::new("test", vec!["read", "write"])]),
        None,
        Some(metadata),
    ).unwrap()
}

#[tokio::test]
async fn test_agent_replaces_actor_identity() {
    // Create agents with different types
    let user_agent = create_test_agent("user1", AgentType::User);
    let committee_agent = create_test_agent("committee1", AgentType::Committee);
    let operator_agent = create_test_agent("operator1", AgentType::Operator);
    
    // Verify each agent has a unique identity
    assert_ne!(user_agent.agent_id(), committee_agent.agent_id());
    assert_ne!(user_agent.agent_id(), operator_agent.agent_id());
    assert_ne!(committee_agent.agent_id(), operator_agent.agent_id());
    
    // Verify agent types are correctly assigned
    assert_eq!(user_agent.agent_type(), &AgentType::User);
    assert_eq!(committee_agent.agent_type(), &AgentType::Committee);
    assert_eq!(operator_agent.agent_type(), &AgentType::Operator);
    
    // Verify content-addressed identity
    let original_id = user_agent.agent_id().clone();
    let mut user_agent_clone = user_agent.clone();
    
    // When agent state changes, its content-based identity should change
    user_agent_clone.set_state(AgentState::Inactive).await.unwrap();
    assert_ne!(user_agent_clone.agent_id(), &original_id);
}

#[tokio::test]
async fn test_agent_replaces_actor_messaging() {
    // Create agents for messaging
    let sender = create_test_agent("sender", AgentType::User);
    let recipient = create_test_agent("recipient", AgentType::User);
    
    // Create message factory and router
    let factory = MessageFactory::new();
    let router = MessageRouter::new();
    
    // Register agents with the message router
    router.register_agent(sender.agent_id().clone()).await.unwrap();
    router.register_agent(recipient.agent_id().clone()).await.unwrap();
    
    // Create a message
    let message = factory.create_message(
        sender.agent_id().clone(),
        recipient.agent_id().clone(),
        MessageType::Request,
        "Hello, this is a test message".as_bytes().to_vec(),
        HashMap::from([("priority".to_string(), "high".to_string())]),
    ).unwrap();
    
    // Send the message
    router.send_message(message.clone()).await.unwrap();
    
    // Check that the message was delivered
    let recipient_messages = router.get_messages_for_agent(recipient.agent_id()).await.unwrap();
    assert_eq!(recipient_messages.len(), 1);
    assert_eq!(recipient_messages[0].content(), &message.content());
}

#[tokio::test]
async fn test_agent_registry_replaces_actor_discovery() {
    // Create an in-memory agent registry
    let registry = crate::resource::agent::registry::InMemoryAgentRegistry::new();
    
    // Create test agents
    let user_agent = create_test_agent("user1", AgentType::User);
    let committee_agent = create_test_agent("committee1", AgentType::Committee);
    
    // Register agents
    registry.register_agent(Box::new(user_agent.clone())).await.unwrap();
    registry.register_agent(Box::new(committee_agent.clone())).await.unwrap();
    
    // Find an agent by ID
    let found = registry.get_agent_by_id(user_agent.agent_id()).await.unwrap();
    assert_eq!(found.agent_id(), user_agent.agent_id());
    
    // Find agents by type
    let users = registry.get_agents_by_type(&AgentType::User).await.unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].agent_id(), user_agent.agent_id());
    
    // Find agents by capability
    let with_capability = registry.get_agents_with_capability("test").await.unwrap();
    assert_eq!(with_capability.len(), 2); // Both agents have this capability
}

#[tokio::test]
async fn test_agent_service_replaces_actor_service_discovery() {
    // Create test agents
    let service_provider = create_test_agent("provider", AgentType::Operator);
    let service_consumer = create_test_agent("consumer", AgentType::User);
    
    // Create service status
    let service = ServiceStatus::new(
        service_provider.agent_id().clone(),
        ResourceId::new(ResourceType::Service, vec![1, 2, 3]),
        "data_processing",
        ServiceState::Available,
        None,
    );
    
    // Create service registry
    let service_registry = crate::resource::agent::service::ServiceStatusManager::new();
    
    // Register service
    service_registry.register_service(service.clone()).await.unwrap();
    
    // Discover service
    let services = service_registry.find_services_by_type("data_processing").await.unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].agent_id(), service_provider.agent_id());
    assert_eq!(services[0].service_type(), "data_processing");
    assert_eq!(services[0].state(), &ServiceState::Available);
}

#[tokio::test]
async fn test_agent_operations_replace_actor_actions() {
    // Create a test agent
    let mut agent = create_test_agent("test-agent", AgentType::User);
    
    // Create a target resource
    let target_id = ResourceId::new(ResourceType::Document, vec![4, 5, 6]);
    
    // Create a test effect
    struct TestEffect {
        resource_id: ResourceId,
        was_executed: bool,
    }
    
    impl TestEffect {
        fn new(resource_id: ResourceId) -> Self {
            Self {
                resource_id,
                was_executed: false,
            }
        }
    }
    
    impl Effect for TestEffect {
        fn effect_type(&self) -> String {
            "test-effect".to_string()
        }
        
        fn resource_id(&self) -> Option<&ResourceId> {
            Some(&self.resource_id)
        }
        
        fn dependencies(&self) -> Vec<ContentHash> {
            Vec::new()
        }
        
        fn clone_effect(&self) -> Box<dyn Effect> {
            Box::new(Self {
                resource_id: self.resource_id.clone(),
                was_executed: self.was_executed,
            })
        }
    }
    
    // Create operation
    let operation_builder = OperationBuilder::new();
    let operation = operation_builder
        .agent_id(agent.agent_id().clone())
        .target_resource(target_id.clone())
        .operation_type(OperationType::Read)
        .add_effect(Box::new(TestEffect::new(target_id.clone())))
        .with_parameter("field", "content")
        .build()
        .unwrap();
    
    // Add required capability
    agent.add_capability(Capability::new("document:read", vec!["read"])).await.unwrap();
    
    // Verify operation properties
    assert_eq!(operation.agent_id(), agent.agent_id());
    assert_eq!(operation.target_resource_id(), &target_id);
    assert_eq!(operation.operation_type(), &OperationType::Read);
    assert_eq!(operation.parameters().get("field"), Some(&"content".to_string()));
}

#[tokio::test]
async fn test_capability_bundle_replaces_actor_roles() {
    use crate::resource::agent::capability::{
        CapabilityBundleManager, StandardBundleType, CapabilityBundleScope
    };
    
    // Create a capability bundle manager
    let mut manager = CapabilityBundleManager::new();
    
    // Create test agents
    let admin = create_test_agent("admin", AgentType::Operator);
    let user = create_test_agent("user", AgentType::User);
    
    // Register standard bundles
    let admin_bundle_id = manager.register_standard_bundle(
        StandardBundleType::Admin,
        None,
    ).unwrap();
    
    let user_bundle_id = manager.register_standard_bundle(
        StandardBundleType::UserBasic,
        None,
    ).unwrap();
    
    // Delegate bundles to agents
    let current_time = 1000;
    manager.delegate_bundle(&admin_bundle_id, &AgentId::from_content_hash(ContentHash::calculate("system".as_bytes())), admin.agent_id(), current_time).unwrap();
    manager.delegate_bundle(&user_bundle_id, admin.agent_id(), user.agent_id(), current_time).unwrap();
    
    // Check capabilities
    let doc_id = ResourceId::new(ResourceType::Document, vec![7, 8, 9]);
    let doc_type = ResourceType::Document;
    
    // Admin should have admin capabilities
    let admin_caps = manager.get_agent_capabilities_for_resource(
        admin.agent_id(),
        &doc_id,
        &doc_type,
        current_time,
    );
    
    assert!(admin_caps.iter().any(|c| c.id() == "admin"));
    assert!(admin_caps.iter().any(|c| c.id() == "read"));
    assert!(admin_caps.iter().any(|c| c.id() == "write"));
    
    // User should have basic user capabilities
    let user_caps = manager.get_agent_capabilities_for_resource(
        user.agent_id(),
        &doc_id,
        &doc_type,
        current_time,
    );
    
    assert!(user_caps.iter().any(|c| c.id() == "resource.list"));
    assert!(user_caps.iter().any(|c| c.id() == "resource.describe"));
}

#[tokio::test]
async fn test_agent_relationships_replace_actor_supervision() {
    // Create test agents
    let mut parent = create_test_agent("parent", AgentType::Operator);
    let child1 = create_test_agent("child1", AgentType::User);
    let child2 = create_test_agent("child2", AgentType::User);
    
    // Create parent-child relationships
    let rel1 = AgentRelationship::new(
        crate::resource::agent::types::RelationshipType::Parent,
        child1.id().clone(),
        vec![Capability::new("supervise", vec!["restart", "stop"])],
        HashMap::new(),
    );
    
    let rel2 = AgentRelationship::new(
        crate::resource::agent::types::RelationshipType::Parent,
        child2.id().clone(),
        vec![Capability::new("supervise", vec!["restart", "stop"])],
        HashMap::new(),
    );
    
    // Add relationships
    parent.add_relationship(rel1).await.unwrap();
    parent.add_relationship(rel2).await.unwrap();
    
    // Get all children
    let children = parent.relationships().iter()
        .filter(|r| matches!(r.relationship_type(), crate::resource::agent::types::RelationshipType::Parent))
        .map(|r| r.target_resource_id().clone())
        .collect::<Vec<_>>();
    
    assert_eq!(children.len(), 2);
    assert!(children.contains(child1.id()));
    assert!(children.contains(child2.id()));
    
    // Check supervision capabilities
    let child1_rel = parent.get_relationship(child1.id()).unwrap();
    assert!(child1_rel.capabilities().iter().any(|c| c.id() == "supervise"));
} 