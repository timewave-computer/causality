//! Integration tests for the Sparse Merkle Tree implementation

use causality_core::{MemorySmt, Blake3Hasher, Hasher};

#[test]
fn test_basic_smt_operations() {
    // Create a new SMT
    let tree = MemorySmt::default();
    let mut root = MemorySmt::empty_tree_root();
    
    // Test data
    let key1 = Blake3Hasher::hash(b"key1");
    let data1 = b"Hello, SMT!";
    
    let key2 = Blake3Hasher::hash(b"key2");
    let data2 = b"Another value";
    
    // Insert first value
    root = tree.insert(root, &key1, data1).unwrap();
    
    // Verify proof for first value
    let proof1 = tree.get_opening(root, &key1).unwrap().unwrap();
    assert!(MemorySmt::verify(&proof1, &root, &key1, data1));
    
    // Insert second value
    root = tree.insert(root, &key2, data2).unwrap();
    
    // Verify both proofs still work
    let proof1_updated = tree.get_opening(root, &key1).unwrap().unwrap();
    let proof2 = tree.get_opening(root, &key2).unwrap().unwrap();
    
    assert!(MemorySmt::verify(&proof1_updated, &root, &key1, data1));
    assert!(MemorySmt::verify(&proof2, &root, &key2, data2));
    
    println!("✅ SMT implementation is working correctly!");
}

#[test]
fn test_smt_with_blake3() {
    let tree = MemorySmt::default();
    let data = b"test data for Blake3";
    let key = Blake3Hasher::hash(data);
    
    let root = MemorySmt::empty_tree_root();
    let new_root = tree.insert(root, &key, data).unwrap();
    
    assert_ne!(root, new_root);
    
    let proof = tree.get_opening(new_root, &key).unwrap().unwrap();
    assert!(MemorySmt::verify(&proof, &new_root, &key, data));
    
    println!("✅ Blake3 hashing is working correctly in SMT!");
} 