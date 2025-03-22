use timewave::effect_adapters::hash::{
    Hash, 
    HashAlgorithm, 
    ContentHasher, 
    ObjectHasher,
    Blake3ContentHasher,
    HasherFactory
};

#[test]
fn test_hash_implementation() {
    println!("Testing the migrated hash implementation...");
    
    // Create a hasher
    let hasher = Blake3ContentHasher::new();
    
    // Test hashing bytes
    let hash1 = hasher.hash_bytes(b"hello");
    let hash2 = hasher.hash_bytes(b"hello");
    let hash3 = hasher.hash_bytes(b"world");
    
    println!("Hash of 'hello': {}", hash1);
    println!("Hash of 'hello' again: {}", hash2);
    println!("Hash of 'world': {}", hash3);
    
    assert_eq!(hash1, hash2);
    assert_ne!(hash1, hash3);
    assert_eq!(hash1.algorithm, HashAlgorithm::Blake3);
    assert_eq!(hash1.bytes.len(), 32);
    
    // Test the hash factory
    let factory = HasherFactory::default();
    let blake3_hasher = factory.create_hasher(HashAlgorithm::Blake3);
    let default_hasher = factory.default_hasher();
    
    let hash4 = blake3_hasher.hash_bytes(b"test");
    let hash5 = default_hasher.hash_bytes(b"test");
    
    println!("Hash of 'test' using Blake3: {}", hash4);
    println!("Hash of 'test' using default: {}", hash5);
    
    assert_eq!(hash4, hash5);
    
    println!("All tests passed successfully!");
}
