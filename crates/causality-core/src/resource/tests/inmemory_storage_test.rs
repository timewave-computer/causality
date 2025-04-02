use std::sync::Arc;
use causality_types::ContentId;
use crate::resource::storage::storage::InMemoryContentAddressedStorage;

#[test]
fn test_basic_inmemory_storage() {
    // Create a new in-memory storage
    let storage = InMemoryContentAddressedStorage::new();
    
    // Test storing and retrieving simple bytes
    let test_data = b"hello world".to_vec();
    let content_id = storage.store_bytes(&test_data).unwrap();
    
    // Verify the content is stored correctly
    assert!(storage.contains(&content_id));
    assert_eq!(storage.get_bytes(&content_id).unwrap(), test_data);
    
    // Test storage size
    assert_eq!(storage.len(), 1);
    
    // Test removing content
    storage.remove(&content_id).unwrap();
    assert!(!storage.contains(&content_id));
    assert_eq!(storage.len(), 0);
    
    // Test clearing storage
    let content_id1 = storage.store_bytes(b"data1".to_vec()).unwrap();
    let content_id2 = storage.store_bytes(b"data2".to_vec()).unwrap();
    assert_eq!(storage.len(), 2);
    
    storage.clear();
    assert_eq!(storage.len(), 0);
} 