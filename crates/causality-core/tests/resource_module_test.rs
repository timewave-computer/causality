// Basic test for the resource module alone
// This helps verify if resource module is self-contained and can be compiled

use causality_core::resource::agent::types::{AgentId, AgentType, AgentState};
use causality_core::resource::types::{ResourceId, ResourceTypeId};
use causality_types::ContentHash;

// Simple test to confirm that AgentId can be created from ContentHash
#[test]
fn test_agent_id_from_content_hash() {
    let bytes = [1u8; 32];
    let content_hash = ContentHash::from_bytes(&bytes).expect("Failed to create ContentHash");
    let agent_id = AgentId::from_content_hash(content_hash.as_bytes(), AgentType::User);
    
    assert!(agent_id.to_string().len() > 0);
}

// Test to confirm that agent types can be correctly instantiated
#[test]
fn test_agent_types() {
    let user_type = AgentType::User;
    let committee_type = AgentType::Committee;
    let operator_type = AgentType::Operator;
    
    assert_ne!(user_type, committee_type);
    assert_ne!(user_type, operator_type);
    assert_ne!(committee_type, operator_type);
}

// Test to confirm that agent state works correctly
#[test]
fn test_agent_state() {
    let active_state = AgentState::Active;
    let inactive_state = AgentState::Inactive;
    let suspended_state = AgentState::Suspended { 
        reason: "Test suspension".to_string(), 
        timestamp: 12345 
    };
    
    assert_ne!(active_state, suspended_state);
    assert_ne!(active_state, inactive_state);
    assert_ne!(suspended_state, inactive_state);
}

// Test resource IDs
#[test]
fn test_resource_id() {
    let bytes = [2u8; 32];
    let content_hash = ContentHash::from_bytes(&bytes).expect("Failed to create ContentHash");
    let resource_id = ResourceId::new(content_hash);
    
    assert!(resource_id.to_string().len() > 0);
}

// Test resource type IDs
#[test]
fn test_resource_type_id() {
    let type_id = ResourceTypeId::new("TestResource");
    
    assert_eq!(type_id.name(), "TestResource");
    assert_eq!(type_id.namespace(), None);
} 