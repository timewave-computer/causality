// Blake3 implementation of the HashFunction trait
//
// This module provides a concrete implementation of the HashFunction trait
// using the Blake3 hashing algorithm.

use super::{HashFunction, Hasher, HashOutput, HashError};
use blake3::{Hasher as Blake3Hasher, Hash as Blake3Hash};
use std::sync::Arc;

/// Blake3 hash implementation
#[derive(Clone, Default)]
pub struct Blake3Hash32;

impl HashFunction for Blake3Hash32 {
    fn hash(&self, data: &[u8]) -> HashOutput {
        let hash = blake3::hash(data);
        let bytes = hash.as_bytes();
        HashOutput::new(*bytes)
    }
    
    fn multi_hash(&self, inputs: &[&[u8]]) -> HashOutput {
        let mut hasher = blake3::Hasher::new();
        for input in inputs {
            hasher.update(input);
        }
        let hash = hasher.finalize();
        let bytes = hash.as_bytes();
        HashOutput::new(*bytes)
    }
    
    fn new_hasher(&self) -> Box<dyn Hasher> {
        Box::new(Blake3IncHasher::new())
    }
    
    fn verify(&self, data: &[u8], expected: &HashOutput) -> bool {
        let hash = self.hash(data);
        hash == *expected
    }
}

/// Blake3 incremental hasher implementation
pub struct Blake3IncHasher {
    hasher: Blake3Hasher,
}

impl Blake3IncHasher {
    /// Create a new Blake3 incremental hasher
    pub fn new() -> Self {
        Self {
            hasher: Blake3Hasher::new(),
        }
    }
}

impl Default for Blake3IncHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for Blake3IncHasher {
    fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }
    
    fn finalize(&self) -> HashOutput {
        let hash = self.hasher.finalize();
        let bytes = hash.as_bytes();
        HashOutput::new(*bytes)
    }
    
    fn reset(&mut self) {
        self.hasher = Blake3Hasher::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blake3_hash() {
        let hasher = Blake3Hash32::default();
        let data = b"test data";
        let hash = hasher.hash(data);
        
        // Verify the hash is deterministic
        let hash2 = hasher.hash(data);
        assert_eq!(hash, hash2);
        
        // Test serialization and deserialization
        let hash_hex = hash.to_hex();
        let hash_from_hex = HashOutput::from_hex(&hash_hex).unwrap();
        assert_eq!(hash, hash_from_hex);
    }
    
    #[test]
    fn test_blake3_incremental() {
        let hasher = Blake3Hash32::default();
        let inc_hasher = hasher.new_hasher();
        
        // Test single update
        let mut hasher1 = inc_hasher.clone();
        hasher1.update(b"test data");
        let hash1 = hasher1.finalize();
        
        // Test multiple updates
        let mut hasher2 = inc_hasher.clone();
        hasher2.update(b"test");
        hasher2.update(b" ");
        hasher2.update(b"data");
        let hash2 = hasher2.finalize();
        
        // Both should produce the same hash
        assert_eq!(hash1, hash2);
        
        // Both should match direct hash
        let direct_hash = hasher.hash(b"test data");
        assert_eq!(hash1, direct_hash);
    }
    
    #[test]
    fn test_blake3_multi_hash() {
        let hasher = Blake3Hash32::default();
        
        // Test multi_hash
        let hash1 = hasher.multi_hash(&[b"test", b" ", b"data"]);
        
        // Compare with incremental
        let mut inc_hasher = hasher.new_hasher();
        inc_hasher.update(b"test");
        inc_hasher.update(b" ");
        inc_hasher.update(b"data");
        let hash2 = inc_hasher.finalize();
        
        // Both should produce the same hash
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_blake3_verify() {
        let hasher = Blake3Hash32::default();
        let data = b"test data";
        let hash = hasher.hash(data);
        
        // Verify should return true for matching hash
        assert!(hasher.verify(data, &hash));
        
        // Verify should return false for non-matching hash
        let mut wrong_hash = hash.as_bytes();
        wrong_hash[0] ^= 1; // Flip a bit
        let wrong_hash = HashOutput::new(wrong_hash);
        assert!(!hasher.verify(data, &wrong_hash));
    }
} 