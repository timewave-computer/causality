use std::collections::HashMap;

use crate::address::Address;
use crate::resource::{
    MemoryResourceAPI, ResourceId, ResourceQuery, Right, ResourceApiError, ResourceState,
};

#[tokio::test]
async fn test_create_resource() {
    // Create addresses
    let admin = Address::from("admin:0x1234");
    let alice = Address::from("user:alice");
    
    // Create API
    let api = MemoryResourceAPI::new(admin.clone());
    let root_cap = api.root_capability();
    
    // Create resource
    let data = "Hello, world!".as_bytes().to_vec();
    let mut metadata = HashMap::new();
    metadata.insert("title".to_string(), "Test Document".to_string());
    
    let (resource_id, capability) = api.create_resource(
        &root_cap,
        "document",
        &alice,
        data.clone(),
        Some(metadata),
    ).await.expect("Failed to create resource");
    
    // Verify resource was created
    let exists = api.resource_exists(&root_cap, &resource_id)
        .await
        .expect("Failed to check existence");
    assert!(exists);
    
    // Check resource can be retrieved
    let resource = api.get_resource(&capability, &resource_id)
        .await
        .expect("Failed to get resource");
    
    assert_eq!(resource.id(), resource_id);
    assert_eq!(resource.resource_type(), "document");
    assert_eq!(resource.data(), &data);
    assert_eq!(resource.state(), &ResourceState::Active);
    assert_eq!(resource.metadata().resource_type, "document");
    assert_eq!(resource.metadata().owner, alice);
    assert_eq!(resource.metadata().custom.get("title").unwrap(), "Test Document");
}

#[tokio::test]
async fn test_update_resource() {
    // Create addresses
    let admin = Address::from("admin:0x1234");
    let alice = Address::from("user:alice");
    
    // Create API
    let api = MemoryResourceAPI::new(admin.clone());
    let root_cap = api.root_capability();
    
    // Create resource
    let data = "Initial content".as_bytes().to_vec();
    let (resource_id, capability) = api.create_resource(
        &root_cap,
        "document",
        &alice,
        data.clone(),
        None,
    ).await.expect("Failed to create resource");
    
    // Update the resource
    let new_data = "Updated content".as_bytes().to_vec();
    api.update_resource(&capability, &resource_id, Some(new_data.clone()), None)
        .await
        .expect("Failed to update resource");
    
    // Verify update
    let resource = api.get_resource(&capability, &resource_id)
        .await
        .expect("Failed to get resource");
    
    assert_eq!(resource.data(), &new_data);
    
    // Update metadata
    let mut update_options = crate::resource::ResourceUpdateOptions {
        resource_type: None,
        owner: None,
        domain: Some("test-domain".to_string()),
        metadata: HashMap::new(),
        override_metadata: false,
    };
    update_options.metadata.insert("version".to_string(), "2.0".to_string());
    
    api.update_resource(&capability, &resource_id, None, Some(update_options))
        .await
        .expect("Failed to update resource metadata");
    
    // Verify metadata update
    let resource = api.get_resource(&capability, &resource_id)
        .await
        .expect("Failed to get resource");
    
    assert_eq!(resource.metadata().domain.as_deref(), Some("test-domain"));
    assert_eq!(resource.metadata().custom.get("version").unwrap(), "2.0");
}

#[tokio::test]
async fn test_delete_resource() {
    // Create addresses
    let admin = Address::from("admin:0x1234");
    let alice = Address::from("user:alice");
    
    // Create API
    let api = MemoryResourceAPI::new(admin.clone());
    let root_cap = api.root_capability();
    
    // Create resource
    let data = "Test content".as_bytes().to_vec();
    let (resource_id, capability) = api.create_resource(
        &root_cap,
        "document",
        &alice,
        data.clone(),
        None,
    ).await.expect("Failed to create resource");
    
    // Delete the resource
    api.delete_resource(&capability, &resource_id)
        .await
        .expect("Failed to delete resource");
    
    // Resource should still exist but be in deleted state
    let exists = api.resource_exists(&root_cap, &resource_id)
        .await
        .expect("Failed to check existence");
    assert!(exists);
    
    // Check the state
    let resource = api.get_resource(&root_cap, &resource_id)
        .await
        .expect("Failed to get resource");
    
    assert_eq!(resource.state(), &ResourceState::Deleted);
}

#[tokio::test]
async fn test_capability_delegation() {
    // Create addresses
    let admin = Address::from("admin:0x1234");
    let alice = Address::from("user:alice");
    let bob = Address::from("user:bob");
    
    // Create API
    let api = MemoryResourceAPI::new(admin.clone());
    let root_cap = api.root_capability();
    
    // Create resource
    let data = "Alice's document".as_bytes().to_vec();
    let (resource_id, alice_capability) = api.create_resource(
        &root_cap,
        "document",
        &alice,
        data.clone(),
        None,
    ).await.expect("Failed to create resource");
    
    // Alice delegates read access to Bob
    let bob_capability = api.create_capability(
        &alice_capability,
        &resource_id,
        vec![Right::Read],
        &bob,
    ).await.expect("Failed to create capability");
    
    // Bob should be able to read
    let resource = api.get_resource(&bob_capability, &resource_id)
        .await
        .expect("Bob should be able to read");
    
    assert_eq!(resource.data(), &data);
    
    // Bob should not be able to write
    let new_data = "Bob's modification".as_bytes().to_vec();
    let result = api.update_resource(&bob_capability, &resource_id, Some(new_data), None).await;
    
    assert!(result.is_err());
    if let Err(err) = result {
        match err {
            ResourceApiError::AccessDenied(_) => {}, // This is expected
            _ => panic!("Expected AccessDenied error, got: {:?}", err),
        }
    }
}

#[tokio::test]
async fn test_capability_revocation() {
    // Create addresses
    let admin = Address::from("admin:0x1234");
    let alice = Address::from("user:alice");
    let bob = Address::from("user:bob");
    
    // Create API
    let api = MemoryResourceAPI::new(admin.clone());
    let root_cap = api.root_capability();
    
    // Create resource
    let data = "Alice's document".as_bytes().to_vec();
    let (resource_id, alice_capability) = api.create_resource(
        &root_cap,
        "document",
        &alice,
        data.clone(),
        None,
    ).await.expect("Failed to create resource");
    
    // Alice delegates read access to Bob
    let bob_capability = api.create_capability(
        &alice_capability,
        &resource_id,
        vec![Right::Read],
        &bob,
    ).await.expect("Failed to create capability");
    
    // Bob can read before revocation
    let result = api.get_resource(&bob_capability, &resource_id).await;
    assert!(result.is_ok());
    
    // Alice revokes Bob's capability
    api.revoke_capability(&alice_capability, bob_capability.id())
        .await
        .expect("Failed to revoke capability");
    
    // Bob can no longer read after revocation
    let result = api.get_resource(&bob_capability, &resource_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_resource_query() {
    // Create addresses
    let admin = Address::from("admin:0x1234");
    let alice = Address::from("user:alice");
    
    // Create API
    let api = MemoryResourceAPI::new(admin.clone());
    let root_cap = api.root_capability();
    
    // Create resources of different types
    for i in 1..=5 {
        // Documents
        let data = format!("Document {}", i).as_bytes().to_vec();
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), if i % 2 == 0 { "work".to_string() } else { "personal".to_string() });
        
        api.create_resource(
            &root_cap,
            "document",
            &alice,
            data,
            Some(metadata),
        ).await.expect("Failed to create document");
        
        // Images
        let data = format!("Image {}", i).as_bytes().to_vec();
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), if i % 2 == 0 { "work".to_string() } else { "personal".to_string() });
        
        api.create_resource(
            &root_cap,
            "image",
            &alice,
            data,
            Some(metadata),
        ).await.expect("Failed to create image");
    }
    
    // Query all resources
    let query = ResourceQuery {
        resource_type: None,
        owner: Some(alice.clone()),
        domain: None,
        metadata: HashMap::new(),
        sort_by: None,
        ascending: true,
        offset: None,
        limit: None,
    };
    
    let resources = api.find_resources(&root_cap, query)
        .await
        .expect("Failed to query resources");
    
    assert_eq!(resources.len(), 10);
    
    // Query only documents
    let query = ResourceQuery {
        resource_type: Some("document".to_string()),
        owner: Some(alice.clone()),
        domain: None,
        metadata: HashMap::new(),
        sort_by: None,
        ascending: true,
        offset: None,
        limit: None,
    };
    
    let resources = api.find_resources(&root_cap, query)
        .await
        .expect("Failed to query resources");
    
    assert_eq!(resources.len(), 5);
    for resource in &resources {
        assert_eq!(resource.resource_type(), "document");
    }
    
    // Query with metadata filter
    let mut metadata_filter = HashMap::new();
    metadata_filter.insert("category".to_string(), "work".to_string());
    
    let query = ResourceQuery {
        resource_type: None,
        owner: Some(alice.clone()),
        domain: None,
        metadata: metadata_filter,
        sort_by: None,
        ascending: true,
        offset: None,
        limit: None,
    };
    
    let resources = api.find_resources(&root_cap, query)
        .await
        .expect("Failed to query resources");
    
    assert_eq!(resources.len(), 5); // Should have 5 resources with category=work
}

#[tokio::test]
async fn test_capability_composition() {
    // Create addresses
    let admin = Address::from("admin:0x1234");
    let alice = Address::from("user:alice");
    let bob = Address::from("user:bob");
    let charlie = Address::from("user:charlie");
    
    // Create API
    let api = MemoryResourceAPI::new(admin.clone());
    let root_cap = api.root_capability();
    
    // Create two resources
    let data1 = "Document 1".as_bytes().to_vec();
    let (resource_id1, alice_capability1) = api.create_resource(
        &root_cap,
        "document",
        &alice,
        data1.clone(),
        None,
    ).await.expect("Failed to create resource 1");
    
    let data2 = "Document 2".as_bytes().to_vec();
    let (resource_id2, alice_capability2) = api.create_resource(
        &root_cap,
        "document",
        &alice,
        data2.clone(),
        None,
    ).await.expect("Failed to create resource 2");
    
    // Alice delegates different rights to Bob for each resource
    let bob_capability1 = api.create_capability(
        &alice_capability1,
        &resource_id1,
        vec![Right::Read, Right::Write],
        &bob,
    ).await.expect("Failed to create capability 1 for Bob");
    
    let bob_capability2 = api.create_capability(
        &alice_capability2,
        &resource_id2,
        vec![Right::Read, Right::Delete],
        &bob,
    ).await.expect("Failed to create capability 2 for Bob");
    
    // Bob composes capabilities and delegates to Charlie
    let charlie_capability = api.compose_capabilities(
        &[bob_capability1.clone(), bob_capability2.clone()],
        &charlie,
    ).await.expect("Failed to compose capabilities");
    
    // Verify that Charlie has only the common rights (Read)
    let resource1 = api.get_resource(&charlie_capability, &resource_id1)
        .await
        .expect("Charlie should be able to read resource 1");
    
    let resource2 = api.get_resource(&charlie_capability, &resource_id2)
        .await
        .expect("Charlie should be able to read resource 2");
    
    assert_eq!(resource1.data(), &data1);
    assert_eq!(resource2.data(), &data2);
    
    // Charlie should not be able to write or delete
    let new_data = "Charlie's modification".as_bytes().to_vec();
    let result = api.update_resource(&charlie_capability, &resource_id1, Some(new_data.clone()), None).await;
    assert!(result.is_err());
    
    let result = api.delete_resource(&charlie_capability, &resource_id2).await;
    assert!(result.is_err());
} 