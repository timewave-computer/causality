use std::sync::Arc;
use serde::{Serialize, Deserialize};

use causality_types::{ContentAddressed, ContentHash, ContentId};
use causality_types::crypto_primitives::{HashError, HashOutput};
use crate::resource::storage::{InMemoryContentAddressedStorage, ResourceStorageError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestData {
    field1: String,
    field2: i32,
}

impl ContentAddressed for TestData {
    fn content_hash(&self) -> Result<HashOutput, HashError> {
        // Create a deterministic content hash for testing
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.field1.as_bytes());
        hasher.update(&self.field2.to_le_bytes());
        let hash = hasher.finalize();
        
        // Create a HashOutput with Blake3 algorithm
        let mut data = [0u8; 32];
        data.copy_from_slice(hash.as_bytes());
        Ok(HashOutput::new(data, causality_types::crypto_primitives::HashAlgorithm::Blake3))
    }

    fn to_bytes(&self) -> Result<Vec<u8>, HashError> {
        // Serialize the resource to bytes
        match serde_json::to_vec(self) {
            Ok(bytes) => Ok(bytes),
            Err(err) => Err(HashError::SerializationError(err.to_string())),
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        // Deserialize from bytes
        match serde_json::from_slice(bytes) {
            Ok(resource) => Ok(resource),
            Err(err) => Err(HashError::SerializationError(err.to_string())),
        }
    }
}

#[test]
fn test_inmemory_content_addressed_storage_basic() {
    // Create a new InMemoryContentAddressedStorage
    let storage = InMemoryContentAddressedStorage::new();
    
    // Create test data
    let test_data = TestData {
        field1: "test".to_string(),
        field2: 42,
    };
    
    // Test basic store and retrieve
    let bytes = test_data.to_bytes().unwrap();
    let content_id = storage.store_bytes(&bytes).unwrap();
    assert!(storage.contains(&content_id));
    
    // Test retrieve bytes
    let retrieved_bytes = storage.get_bytes(&content_id).unwrap();
    assert_eq!(retrieved_bytes, bytes);
} 