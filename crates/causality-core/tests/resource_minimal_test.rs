// Minimal test for ContentHash in the resource module
use causality_types::ContentHash;

// We don't need to import anything from causality-core itself to test basic ContentHash functionality
#[test]
fn test_content_hash_creation() {
    let bytes = [1u8; 32];
    let content_hash = ContentHash::from_bytes(&bytes).expect("Failed to create ContentHash");
    
    // Basic checks to make sure ContentHash can be created and used
    assert_eq!(content_hash.as_bytes().len(), 32);
    // The hash format adds a prefix byte which is 149 (0x95) for blake3
    assert_eq!(content_hash.as_bytes()[0], 149);
}

// If this test passes, it means our ContentHash implementation from causality_types is working
#[test]
fn test_content_hash_equality() {
    let bytes1 = [1u8; 32];
    let bytes2 = [2u8; 32];
    
    let hash1 = ContentHash::from_bytes(&bytes1).expect("Failed to create ContentHash");
    let hash2 = ContentHash::from_bytes(&bytes1).expect("Failed to create ContentHash"); // Same as hash1
    let hash3 = ContentHash::from_bytes(&bytes2).expect("Failed to create ContentHash"); // Different from hash1
    
    assert_eq!(hash1, hash2);
    assert_ne!(hash1, hash3);
} 