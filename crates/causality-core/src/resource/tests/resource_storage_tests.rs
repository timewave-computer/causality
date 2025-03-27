use std::collections::HashMap;
use std::sync::Arc;

use serde::{Serialize, Deserialize};

use crate::content::{ContentAddressed, ContentAddressingError, ContentHash};
use crate::resource::{ResourceId, ResourceTypeId};
use crate::resource::storage::{
    ResourceStorage, InMemoryResourceStorage, ResourceStorageConfig, create_resource_storage
};
use crate::storage::InMemoryContentAddressedStorage;

// Test resource implementation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestResource {
    name: String,
    value: i32,
    data: Vec<u8>,
}

impl ContentAddressed for TestResource {
    fn content_hash(&self) -> Result<ContentHash, ContentAddressingError> {
        // Create a deterministic content hash for testing
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.name.as_bytes());
        hasher.update(&self.value.to_le_bytes());
        hasher.update(&self.data);
        let hash = hasher.finalize();
        Ok(ContentHash::new(hash.as_bytes().to_vec()))
    }
}

// Helper to create a test resource
fn create_test_resource(name: &str, value: i32, data_size: usize) -> TestResource {
    let mut data = Vec::with_capacity(data_size);
    for i in 0..data_size {
        data.push((i % 256) as u8);
    }
    
    TestResource {
        name: name.to_string(),
        value,
        data,
    }
}

#[tokio::test]
async fn test_basic_resource_storage_operations() {
    // Create storage
    let cas_storage = Arc::new(InMemoryContentAddressedStorage::new());
    let storage = InMemoryResourceStorage::new(cas_storage);
    
    // Create test resources
    let resource1 = create_test_resource("test1", 42, 100);
    let resource2 = create_test_resource("test2", 84, 200);
    
    let resource_type1 = ResourceTypeId::new("test_type_1");
    let resource_type2 = ResourceTypeId::new("test_type_2");
    
    // Test storing resources
    let resource_id1 = storage.store_resource(
        resource1.clone(),
        resource_type1.clone(),
        None
    ).await.unwrap();
    
    let resource_id2 = storage.store_resource(
        resource2.clone(),
        resource_type2.clone(),
        Some(HashMap::from([
            ("creator".to_string(), "test".to_string()),
            ("purpose".to_string(), "testing".to_string()),
        ]))
    ).await.unwrap();
    
    // Test existence check
    assert!(storage.has_resource(&resource_id1).await.unwrap());
    assert!(storage.has_resource(&resource_id2).await.unwrap());
    
    // Test non-existence
    let fake_id = ResourceId::new();
    assert!(!storage.has_resource(&fake_id).await.unwrap());
    
    // Test retrieval
    let retrieved1: TestResource = storage.get_resource(&resource_id1).await.unwrap();
    let retrieved2: TestResource = storage.get_resource(&resource_id2).await.unwrap();
    
    assert_eq!(retrieved1, resource1);
    assert_eq!(retrieved2, resource2);
    
    // Test metadata
    let metadata = storage.get_resource_metadata(&resource_id2).await.unwrap();
    assert_eq!(metadata.get("creator").unwrap(), "test");
    assert_eq!(metadata.get("purpose").unwrap(), "testing");
    
    // Test resource type indexing
    let resources_of_type1 = storage.find_resources_by_type(&resource_type1).await.unwrap();
    let resources_of_type2 = storage.find_resources_by_type(&resource_type2).await.unwrap();
    
    assert_eq!(resources_of_type1.len(), 1);
    assert_eq!(resources_of_type1[0], resource_id1);
    
    assert_eq!(resources_of_type2.len(), 1);
    assert_eq!(resources_of_type2[0], resource_id2);
}

#[tokio::test]
async fn test_resource_versioning() {
    // Create storage
    let cas_storage = Arc::new(InMemoryContentAddressedStorage::new());
    let storage = InMemoryResourceStorage::new(cas_storage);
    
    // Create initial resource
    let resource = create_test_resource("versioned", 1, 50);
    let resource_type = ResourceTypeId::new("versioned_type");
    
    let resource_id = storage.store_resource(
        resource.clone(),
        resource_type.clone(),
        None
    ).await.unwrap();
    
    // Create multiple versions
    for i in 2..=5 {
        let updated = create_test_resource("versioned", i, 50 + i as usize);
        let new_version = storage.update_resource(
            &resource_id,
            updated,
            Some(HashMap::from([
                ("version".to_string(), i.to_string())
            ]))
        ).await.unwrap();
        
        assert_eq!(new_version, i as u64);
    }
    
    // Check version history
    let history = storage.get_version_history(&resource_id).await.unwrap();
    assert_eq!(history.len(), 5);
    
    // Verify version 1
    let v1: TestResource = storage.get_resource_version(&resource_id, 1).await.unwrap();
    assert_eq!(v1.name, "versioned");
    assert_eq!(v1.value, 1);
    
    // Verify version 3
    let v3: TestResource = storage.get_resource_version(&resource_id, 3).await.unwrap();
    assert_eq!(v3.name, "versioned");
    assert_eq!(v3.value, 3);
    
    // Verify latest version
    let latest: TestResource = storage.get_resource(&resource_id).await.unwrap();
    assert_eq!(latest.name, "versioned");
    assert_eq!(latest.value, 5);
    
    // Verify metadata
    let metadata = storage.get_resource_metadata(&resource_id).await.unwrap();
    assert_eq!(metadata.get("version").unwrap(), "5");
}

#[tokio::test]
async fn test_resource_tagging() {
    // Create storage
    let cas_storage = Arc::new(InMemoryContentAddressedStorage::new());
    let storage = InMemoryResourceStorage::new(cas_storage);
    
    // Create test resources
    let resources = vec![
        ("resource1", 1, "type_a"),
        ("resource2", 2, "type_a"),
        ("resource3", 3, "type_b"),
        ("resource4", 4, "type_b"),
        ("resource5", 5, "type_c"),
    ];
    
    let mut resource_ids = Vec::new();
    
    // Store resources
    for (name, value, type_name) in resources {
        let resource = create_test_resource(name, value, 100);
        let resource_type = ResourceTypeId::new(type_name);
        
        let id = storage.store_resource(
            resource,
            resource_type,
            None
        ).await.unwrap();
        
        resource_ids.push(id);
    }
    
    // Add tags
    storage.add_tag(&resource_ids[0], "important").await.unwrap();
    storage.add_tag(&resource_ids[1], "important").await.unwrap();
    storage.add_tag(&resource_ids[2], "important").await.unwrap();
    
    storage.add_tag(&resource_ids[0], "urgent").await.unwrap();
    storage.add_tag(&resource_ids[3], "urgent").await.unwrap();
    
    storage.add_tag(&resource_ids[4], "special").await.unwrap();
    
    // Test finding by tag
    let important_resources = storage.find_resources_by_tag("important").await.unwrap();
    assert_eq!(important_resources.len(), 3);
    assert!(important_resources.contains(&resource_ids[0]));
    assert!(important_resources.contains(&resource_ids[1]));
    assert!(important_resources.contains(&resource_ids[2]));
    
    let urgent_resources = storage.find_resources_by_tag("urgent").await.unwrap();
    assert_eq!(urgent_resources.len(), 2);
    assert!(urgent_resources.contains(&resource_ids[0]));
    assert!(urgent_resources.contains(&resource_ids[3]));
    
    let special_resources = storage.find_resources_by_tag("special").await.unwrap();
    assert_eq!(special_resources.len(), 1);
    assert!(special_resources.contains(&resource_ids[4]));
    
    // Test removal of tags
    storage.remove_tag(&resource_ids[0], "urgent").await.unwrap();
    
    let updated_urgent = storage.find_resources_by_tag("urgent").await.unwrap();
    assert_eq!(updated_urgent.len(), 1);
    assert!(updated_urgent.contains(&resource_ids[3]));
    
    // Resource should still have the other tag
    let updated_important = storage.find_resources_by_tag("important").await.unwrap();
    assert_eq!(updated_important.len(), 3);
    assert!(updated_important.contains(&resource_ids[0]));
}

#[tokio::test]
async fn test_resource_storage_config() {
    // Create storage with custom config
    let cas_storage = Arc::new(InMemoryContentAddressedStorage::new());
    let config = ResourceStorageConfig {
        enable_versioning: true,
        enable_caching: true,
        max_versions_per_resource: Some(5),
        cache_size: Some(100),
    };
    
    let storage = create_resource_storage(cas_storage, config);
    
    // Create initial resource
    let resource = create_test_resource("config_test", 1, 50);
    let resource_type = ResourceTypeId::new("config_test_type");
    
    let resource_id = storage.store_resource(
        resource.clone(),
        resource_type.clone(),
        None
    ).await.unwrap();
    
    // Basic operations should still work
    assert!(storage.has_resource(&resource_id).await.unwrap());
    
    let retrieved: TestResource = storage.get_resource(&resource_id).await.unwrap();
    assert_eq!(retrieved, resource);
} 