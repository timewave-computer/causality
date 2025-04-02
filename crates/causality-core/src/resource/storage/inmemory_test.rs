use std::sync::Arc;
use causality_types::ContentId;
use crate::resource::storage::storage::InMemoryContentAddressedStorage;

// This is a simple manual test that doesn't use any assert macros
// to avoid issues with other parts of the codebase
pub fn test_inmemory_storage_manual() -> bool {
    // Create new storage
    let storage = InMemoryContentAddressedStorage::new();
    
    // Test storing bytes
    let test_data = b"hello world".to_vec();
    let content_id = match storage.store_bytes(&test_data) {
        Ok(id) => id,
        Err(_) => return false,
    };
    
    // Verify contains works
    if !storage.contains(&content_id) {
        return false;
    }
    
    // Verify retrieval works
    let retrieved_data = match storage.get_bytes(&content_id) {
        Ok(data) => data,
        Err(_) => return false,
    };
    
    if retrieved_data != test_data {
        return false;
    }
    
    // Check length
    if storage.len() != 1 {
        return false;
    }
    
    // Test removing content
    if let Err(_) = storage.remove(&content_id) {
        return false;
    }
    
    if storage.contains(&content_id) {
        return false;
    }
    
    if storage.len() != 0 {
        return false;
    }
    
    // Test storing multiple items
    let id1 = match storage.store_bytes(b"data1".to_vec()) {
        Ok(id) => id,
        Err(_) => return false,
    };
    
    let id2 = match storage.store_bytes(b"data2".to_vec()) {
        Ok(id) => id,
        Err(_) => return false,
    };
    
    if storage.len() != 2 {
        return false;
    }
    
    // Test clear
    storage.clear();
    
    if storage.len() != 0 {
        return false;
    }
    
    println!("InMemoryContentAddressedStorage implementation passes all manual tests");
    true
}

#[test]
fn manual_test_runner() {
    assert!(test_inmemory_storage_manual());
} 