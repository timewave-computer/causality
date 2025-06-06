//! Integration tests for the Sparse Merkle Tree implementation

use causality_core::{MemorySmt, Sha256Hasher, Hasher};

#[test]
fn test_empty_smt() {
    let smt = MemorySmt::default();
    let initial_root = [0u8; 32]; // Empty root
    
    let key1 = Sha256Hasher::hash(b"key1");
    let value1 = b"Hello, SMT!";
    
    let key2 = Sha256Hasher::hash(b"key2");
    let value2 = b"Another value";
    
    // Insert first value
    let root1 = smt.insert(initial_root, &key1, value1).unwrap();
    
    // Insert second value  
    let root2 = smt.insert(root1, &key2, value2).unwrap();
    
    // Verify values can be retrieved
    let proof1 = smt.get_opening(root2, &key1).unwrap().unwrap();
    let proof2 = smt.get_opening(root2, &key2).unwrap().unwrap();
    
    assert!(MemorySmt::verify(&proof1, &root2, &key1, value1));
    assert!(MemorySmt::verify(&proof2, &root2, &key2, value2));
    
    println!("✅ Basic SMT operations working correctly!");
}

#[test]
fn test_smt_with_sha256() {
    let smt = MemorySmt::default();
    let initial_root = [0u8; 32]; // Empty root
    let data = b"test data for SHA256";
    let key = Sha256Hasher::hash(data);
    
    let root = smt.insert(initial_root, &key, data).unwrap();
    
    assert_ne!(initial_root, root);
    
    let proof = smt.get_opening(root, &key).unwrap().unwrap();
    assert!(MemorySmt::verify(&proof, &root, &key, data));
    
    println!("✅ SHA256 hashing is working correctly in SMT!");
} 