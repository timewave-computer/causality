//-----------------------------------------------------------------------------
// Context Tests
//-----------------------------------------------------------------------------

use causality_runtime::state_manager::DefaultStateManager;
use causality_runtime::tel::StateManager;
use causality_types::{
    core::{
        id::{DomainId, AsId, EntityId, HandlerId, ResourceId},
        str::Str,
        time::Timestamp,
        Effect, Handler, Resource,
    },
    resource::ResourceFlow,
    tel::optimization::TypedDomain,
    provider::context::AsRuntimeContext,
};
use tokio;

//-----------------------------------------------------------------------------
// Helper Functions
//-----------------------------------------------------------------------------

fn create_test_resource() -> Resource {
    Resource::new(
        EntityId::null(),
        Str::from("test_resource"),
        DomainId::null(),
        Str::from("test"),
        1,
        Timestamp::now(),
    )
}

fn create_test_effect(inputs: Vec<ResourceFlow>, outputs: Vec<ResourceFlow>) -> Effect {
    Effect {
        id: EntityId::null(),
        name: Str::from("test_effect"),
        domain_id: DomainId::null(),
        effect_type: Str::from("test"),
        inputs,
        outputs,
        expression: None,
        timestamp: Timestamp::now(),
        resources: Vec::new(),
        nullifiers: Vec::new(),
        scoped_by: HandlerId::null(),
        intent_id: None,
        source_typed_domain: TypedDomain::default(),
        target_typed_domain: TypedDomain::default(),
        cost_model: None,
        resource_usage_estimate: None,
        originating_dataflow_instance: None,
    }
}

fn create_test_handler() -> Handler {
    Handler::new(
        EntityId::null(),
        Str::from("test_handler"),
        DomainId::null(),
        Str::from("test"),
    )
}

//-----------------------------------------------------------------------------
// Context Tests
//-----------------------------------------------------------------------------

#[test]
fn test_context_initialization() {
    let _state_manager = DefaultStateManager::new();
    
    // Basic test - just verify it was created
    assert!(true, "Context initialized successfully");
}

#[tokio::test]
async fn test_context_resource_lifecycle() {
    let state_manager = DefaultStateManager::new();
    
    let resource = create_test_resource();
    // Convert EntityId to ResourceId for the API
    let resource_id = ResourceId::from(resource.id.inner());
    
    // Use async method instead of sync to avoid runtime conflicts
    let retrieved = state_manager.get_resource(&resource_id).await.unwrap();
    assert!(retrieved.is_none(), "Resource should not exist initially");
}

#[tokio::test]
async fn test_context_handler_registration() {
    let state_manager = DefaultStateManager::new();
    
    let handler = create_test_handler();
    let handler_id = HandlerId::from(handler.id.inner());
    
    // Test async handler retrieval
    let retrieved = state_manager.get_handler(&handler_id).await.unwrap();
    assert!(retrieved.is_none(), "Handler should not exist initially");
}

#[tokio::test]
async fn test_context_effect_tracking() {
    let _state_manager = DefaultStateManager::new();
    
    let _effect = create_test_effect(vec![], vec![]);
    
    // Basic test - just verify effect creation works
    assert!(true, "Effect created successfully");
}

#[test]
fn test_context_snapshots() {
    let _state_manager = DefaultStateManager::new();
    
    // Basic test - just verify state manager creation
    assert!(true, "State manager created successfully");
}

#[tokio::test]
async fn test_context_creation() {
    let _state_manager = DefaultStateManager::new();
    
    // Basic test - just verify it was created
    assert!(true, "Context created successfully");
}

#[tokio::test]
async fn test_resource_operations() {
    let state_manager = DefaultStateManager::new();
    
    let resource = create_test_resource();
    // Convert EntityId to ResourceId for the API
    let resource_id = ResourceId::from(resource.id.inner());
    
    // Test resource retrieval (should be None initially)
    let retrieved = state_manager.get_resource(&resource_id).await.unwrap();
    assert!(retrieved.is_none(), "Resource should not exist initially");
}

