use causality_types::content_addressing::storage::{ContentAddressedStorage, InMemoryStorage};

fn main() {
    println!("Testing InMemoryStorage implementation...");
    
    // Create new storage
    let storage = InMemoryStorage::new();
    
    // Test storing bytes
    let test_data = b"hello world".to_vec();
    let content_id = match storage.store_bytes(&test_data) {
        Ok(id) => {
            println!("✅ Successfully stored data with ID: {:?}", id);
            id
        },
        Err(e) => {
            println!("❌ Failed to store data: {:?}", e);
            return;
        }
    };
    
    // Verify contains works
    if storage.contains(&content_id) {
        println!("✅ Contains check passed");
    } else {
        println!("❌ Contains check failed");
        return;
    }
    
    // Verify retrieval works
    let retrieved_data = match storage.get_bytes(&content_id) {
        Ok(data) => {
            println!("✅ Successfully retrieved data");
            data
        },
        Err(e) => {
            println!("❌ Failed to retrieve data: {:?}", e);
            return;
        }
    };
    
    if retrieved_data == test_data {
        println!("✅ Retrieved data matches original data");
    } else {
        println!("❌ Retrieved data does not match original data");
        return;
    }
    
    // Check length
    if storage.len() == 1 {
        println!("✅ Storage length check passed");
    } else {
        println!("❌ Storage length check failed, length: {}", storage.len());
        return;
    }
    
    // Test removing content
    match storage.remove(&content_id) {
        Ok(_) => println!("✅ Successfully removed data"),
        Err(e) => {
            println!("❌ Failed to remove data: {:?}", e);
            return;
        }
    }
    
    if !storage.contains(&content_id) {
        println!("✅ Contains check after removal passed");
    } else {
        println!("❌ Contains check after removal failed");
        return;
    }
    
    if storage.len() == 0 {
        println!("✅ Storage length after removal check passed");
    } else {
        println!("❌ Storage length after removal check failed, length: {}", storage.len());
        return;
    }
    
    // Test storing multiple items
    let data1 = b"data1".to_vec();
    let id1 = match storage.store_bytes(&data1) {
        Ok(id) => {
            println!("✅ Successfully stored data1");
            id
        },
        Err(e) => {
            println!("❌ Failed to store data1: {:?}", e);
            return;
        }
    };
    
    let data2 = b"data2".to_vec();
    let id2 = match storage.store_bytes(&data2) {
        Ok(id) => {
            println!("✅ Successfully stored data2");
            id
        },
        Err(e) => {
            println!("❌ Failed to store data2: {:?}", e);
            return;
        }
    };
    
    if storage.len() == 2 {
        println!("✅ Storage length with multiple items check passed");
    } else {
        println!("❌ Storage length with multiple items check failed, length: {}", storage.len());
        return;
    }
    
    // Test clear
    storage.clear();
    
    if storage.len() == 0 {
        println!("✅ Storage clear check passed");
    } else {
        println!("❌ Storage clear check failed, length: {}", storage.len());
        return;
    }
    
    println!("🎉 All tests passed! InMemoryStorage implementation works correctly.");
} 