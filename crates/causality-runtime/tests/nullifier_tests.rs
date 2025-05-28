//-----------------------------------------------------------------------------
// Nullifier Registry Test
//-----------------------------------------------------------------------------

use causality_runtime::nullifier::NullifierRegistry;
use causality_types::{
    core::id::{ResourceId, TransactionId, AsId},
};

//-----------------------------------------------------------------------------
// Basic Nullifier Operation
//-----------------------------------------------------------------------------

#[test]
fn test_nullifier_registry_consume_resource() {
    let mut registry = NullifierRegistry::new();
    
    let resource_id = ResourceId::null();
    let tx_id = TransactionId::null();
    
    // Test consuming a resource
    registry.consume_resource(&resource_id, &tx_id).unwrap();
    
    // Verify the resource is consumed
    assert!(registry.is_consumed(&resource_id), "Resource should be consumed");
    
    // Test consuming the same resource again (should fail)
    let result = registry.consume_resource(&resource_id, &tx_id);
    assert!(result.is_err(), "Should not be able to consume the same resource twice");
}

#[test]
fn test_nullifier_registry_batch_operations() {
    let mut registry = NullifierRegistry::new();
    
    let resource_id1 = ResourceId::null();
    let resource_id2 = ResourceId::from([1u8; 32]);
    let tx_id = TransactionId::null();
    
    // Consume multiple resources
    registry.consume_resource(&resource_id1, &tx_id).unwrap();
    registry.consume_resource(&resource_id2, &tx_id).unwrap();
    
    // Verify both are consumed
    assert!(registry.is_consumed(&resource_id1), "Resource 1 should be consumed");
    assert!(registry.is_consumed(&resource_id2), "Resource 2 should be consumed");
    
    // Test getting consumed resources
    let consumed_resources = registry.get_consumed_resources();
    assert_eq!(consumed_resources.len(), 2, "Should have 2 consumed resources");
    assert!(consumed_resources.contains(&resource_id1), "Should contain resource 1");
    assert!(consumed_resources.contains(&resource_id2), "Should contain resource 2");
}

#[test]
fn test_nullifier_registry_as_registry() {
    let mut registry = NullifierRegistry::new();
    
    let resource_id = ResourceId::null();
    let tx_id = TransactionId::null();
    
    // Use registry methods
    registry.consume_resource(&resource_id, &tx_id).unwrap();
    assert!(registry.is_consumed(&resource_id), "Registry should contain resource");
    
    // Test getting nullifier
    let nullifier = registry.get_nullifier(&resource_id);
    assert!(nullifier.is_some(), "Should retrieve nullifier");
}

#[tokio::test]
async fn test_nullifier_registry_creation() {
    let registry = NullifierRegistry::new();
    assert!(!registry.is_consumed(&ResourceId::null()), "New registry should be empty");
}

#[tokio::test]
async fn test_nullifier_basic_operations() {
    let mut registry = NullifierRegistry::new();
    
    let resource_id = ResourceId::null();
    let tx_id = TransactionId::null();
    
    registry.consume_resource(&resource_id, &tx_id).unwrap();
    
    let is_consumed = registry.is_consumed(&resource_id);
    assert!(is_consumed, "Resource should be consumed");
}
