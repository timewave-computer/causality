use causality::crypto::{
    ContentAddressed, ContentId, HashOutput, HashAlgorithm, HashFactory
};
use causality::content_addressed_storage::{
    ContentAddressedStorage, StorageError, StorageFactory, InMemoryStorage
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq)]
struct TestResource {
    id: String,
    name: String,
    data: HashMap<String, String>,
    value: u64,
}

impl ContentAddressed for TestResource {
    fn content_hash(&self) -> HashOutput {
        let hasher = HashFactory::default().create_hasher().unwrap();
        
        // Serialize the content to create a deterministic hash
        let serialized = borsh::to_vec(self).unwrap();
        hasher.hash(&serialized)
    }
    
    fn verify(&self) -> bool {
        // Hash the current state and compare with stored hash
        let current_hash = self.content_hash();
        
        // For test purposes, verification is simple
        // In a real implementation, this would compare against a stored hash
        let expected_hash = self.content_hash();
        
        current_hash == expected_hash
    }
}

#[test]
fn test_storage_basics() {
    // Create in-memory storage
    let storage = StorageFactory::create_memory_storage();
    
    // Create test resource
    let resource = TestResource {
        id: "res1".to_string(),
        name: "Test Resource".to_string(),
        data: HashMap::from([
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ]),
        value: 42,
    };
    
    // Store the resource
    let content_id = storage.store(&resource).unwrap();
    
    // Verify the resource exists
    assert!(storage.exists(&content_id));
    
    // Retrieve the resource
    let retrieved: TestResource = storage.get(&content_id).unwrap();
    
    // Verify the retrieved resource matches the original
    assert_eq!(resource, retrieved);
    
    // Verify count
    assert_eq!(storage.count(), 1);
    
    // Remove the resource
    storage.remove(&content_id).unwrap();
    
    // Verify it's removed
    assert!(!storage.exists(&content_id));
    assert_eq!(storage.count(), 0);
}

#[test]
fn test_storage_errors() {
    // Create in-memory storage
    let storage = StorageFactory::create_memory_storage();
    
    // Create test resource
    let resource = TestResource {
        id: "res2".to_string(),
        name: "Error Test Resource".to_string(),
        data: HashMap::new(),
        value: 100,
    };
    
    // Get content ID without storing
    let content_id = resource.content_id();
    
    // Attempt to get non-existent resource
    let result: Result<TestResource, StorageError> = storage.get(&content_id);
    assert!(matches!(result, Err(StorageError::ObjectNotFound(_))));
    
    // Store the resource
    storage.store(&resource).unwrap();
    
    // Attempt to store the same resource again
    let result = storage.store(&resource);
    assert!(matches!(result, Err(StorageError::DuplicateObject(_))));
    
    // Clear storage
    storage.clear();
    assert_eq!(storage.count(), 0);
}

#[test]
fn test_storage_with_multiple_types() {
    let storage = StorageFactory::create_memory_storage();
    
    // First type
    let resource1 = TestResource {
        id: "res-multi-1".to_string(),
        name: "Multi-type Resource 1".to_string(),
        data: HashMap::new(),
        value: 1,
    };
    
    // Second type (same struct but different content)
    let resource2 = TestResource {
        id: "res-multi-2".to_string(),
        name: "Multi-type Resource 2".to_string(),
        data: HashMap::from([
            ("test".to_string(), "value".to_string()),
        ]),
        value: 2,
    };
    
    // Store both resources
    let id1 = storage.store(&resource1).unwrap();
    let id2 = storage.store(&resource2).unwrap();
    
    // Verify different content IDs
    assert_ne!(id1, id2);
    
    // Retrieve and verify
    let retrieved1: TestResource = storage.get(&id1).unwrap();
    let retrieved2: TestResource = storage.get(&id2).unwrap();
    
    assert_eq!(resource1, retrieved1);
    assert_eq!(resource2, retrieved2);
    assert_eq!(storage.count(), 2);
}

#[test]
fn test_storage_factory() {
    // Test factory creation methods
    let memory_storage1 = StorageFactory::create_memory_storage();
    let memory_storage2 = StorageFactory::create_memory_storage();
    
    assert_eq!(memory_storage1.count(), 0);
    assert_eq!(memory_storage2.count(), 0);
    
    // Store in first storage
    let resource = TestResource {
        id: "factory-test".to_string(),
        name: "Factory Test".to_string(),
        data: HashMap::new(),
        value: 999,
    };
    
    let id = memory_storage1.store(&resource).unwrap();
    
    // Verify only in first storage
    assert!(memory_storage1.exists(&id));
    assert!(!memory_storage2.exists(&id));
    
    // Test with custom configuration
    let custom_storage = StorageFactory::with_configuration(
        "test-config".to_string(), 
        HashMap::from([
            ("max_size".to_string(), "1000".to_string()),
            ("cache_enabled".to_string(), "true".to_string()),
        ])
    );
    
    assert_eq!(custom_storage.count(), 0);
} 