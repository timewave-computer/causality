//! Utility functions for cryptographic operations
//!
//! This module provides utility functions for cryptographic operations.

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Compute a simple non-cryptographic hash of a string
/// Useful for non-security-critical situations like stable ID generation
pub fn simple_hash(input: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

/// Compute a simple non-cryptographic hash of bytes
/// Useful for non-security-critical situations like stable ID generation
pub fn simple_hash_bytes(input: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

/// Generate a random 64-bit number
pub fn random_u64() -> u64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen()
}

/// Truncate a slice to the specified length
pub fn truncate_bytes(bytes: &[u8], length: usize) -> Vec<u8> {
    bytes.iter().take(length).cloned().collect()
}

/// Pad a slice to the specified length with zeros
pub fn pad_bytes(bytes: &[u8], length: usize) -> Vec<u8> {
    let mut result = bytes.to_vec();
    if result.len() < length {
        result.resize(length, 0);
    }
    result
}

/// Convert bytes to a hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Convert a hex string to bytes
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(hex)
}

/// Generate a deterministic u64 hash from an arbitrary object that can be serialized
pub fn hash_object<T: serde::Serialize>(obj: &T) -> Result<u64, serde_json::Error> {
    let json = serde_json::to_string(obj)?;
    Ok(simple_hash(&json))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_hash() {
        // Check that the same input produces the same hash
        let hash1 = simple_hash("hello world");
        let hash2 = simple_hash("hello world");
        assert_eq!(hash1, hash2);
        
        // Check that different inputs produce different hashes
        let hash3 = simple_hash("hello world!");
        assert_ne!(hash1, hash3);
    }
    
    #[test]
    fn test_simple_hash_bytes() {
        let bytes1 = vec![1, 2, 3, 4];
        let bytes2 = vec![1, 2, 3, 4];
        let bytes3 = vec![5, 6, 7, 8];
        
        let hash1 = simple_hash_bytes(&bytes1);
        let hash2 = simple_hash_bytes(&bytes2);
        let hash3 = simple_hash_bytes(&bytes3);
        
        // Same input should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different input should produce different hash
        assert_ne!(hash1, hash3);
    }
    
    #[test]
    fn test_hash_object() {
        #[derive(serde::Serialize)]
        struct TestStruct {
            field1: String,
            field2: i32,
        }
        
        let obj1 = TestStruct {
            field1: "test".to_string(),
            field2: 42,
        };
        
        let obj2 = TestStruct {
            field1: "test".to_string(),
            field2: 42,
        };
        
        let obj3 = TestStruct {
            field1: "test".to_string(),
            field2: 43,
        };
        
        // Same data should produce same hash
        let hash1 = hash_object(&obj1).unwrap();
        let hash2 = hash_object(&obj2).unwrap();
        assert_eq!(hash1, hash2);
        
        // Different data should produce different hash
        let hash3 = hash_object(&obj3).unwrap();
        assert_ne!(hash1, hash3);
    }
}
