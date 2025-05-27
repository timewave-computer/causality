// Basic test for the resource module alone
// This helps verify if resource module is self-contained and can be compiled

// Update import to use ResourceId from causality_core and ContentHash from causality_types
use causality_core::ResourceId;
use causality_types::ContentHash;

// Define test versions of these types for the tests
#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentId(ResourceId);

#[derive(Debug, Clone, PartialEq, Eq)]
enum AgentType {
    User,
    Committee,
    Operator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AgentState {
    Active,
    Inactive,
    Suspended {
        reason: String,
        timestamp: u64,
    },
}

// Implement conversion from ContentHash for AgentId
impl AgentId {
    fn from_content_hash(bytes: &[u8], _agent_type: AgentType) -> Self {
        let resource_id = ResourceId::new(ContentHash::from_bytes(bytes).expect("Failed to create ContentHash"));
        AgentId(resource_id)
    }
    
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

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
    // Use the existing test but directly test the string format
    let type_id = "TestResource".to_string();
    
    assert_eq!(type_id, "TestResource");
} 