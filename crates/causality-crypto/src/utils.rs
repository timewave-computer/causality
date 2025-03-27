//! Utility functions for cryptographic operations
//!
//! This module provides utility functions for cryptographic operations.

/// Generate a simple hash from a string (non-cryptographic)
/// 
/// This is a simple non-cryptographic hash function, useful for basic hashing needs.
/// For cryptographic purposes, use the proper hash functions from the hash module.
pub fn simple_hash(input: &str) -> u64 {
    let mut hash: u64 = 0;
    for byte in input.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

/// Generate a simple hash from a byte slice (non-cryptographic)
pub fn simple_hash_bytes(input: &[u8]) -> u64 {
    let mut hash: u64 = 0;
    for &byte in input {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
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
        // Check that the same input produces the same hash
        let hash1 = simple_hash_bytes(b"hello world");
        let hash2 = simple_hash_bytes(b"hello world");
        assert_eq!(hash1, hash2);
        
        // Check that different inputs produce different hashes
        let hash3 = simple_hash_bytes(b"hello world!");
        assert_ne!(hash1, hash3);
        
        // Check that string and byte versions are equivalent
        let hash_str = simple_hash("test string");
        let hash_bytes = simple_hash_bytes(b"test string");
        assert_eq!(hash_str, hash_bytes);
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
