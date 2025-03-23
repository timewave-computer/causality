// Hash module providing a unified interface for different hash functions
// Currently supports Blake3, with a placeholder for Poseidon (future integration with Valence)

use std::fmt;
use std::sync::Arc;
use std::str::FromStr;
use std::hash::{Hash, Hasher as StdHasher};

mod blake3_impl;
mod factory;
mod poseidon_placeholder;

pub use blake3_impl::Blake3Hash32;
pub use factory::{HashFactory, HashAlgorithm};

/// Fixed-size hash output (32 bytes, 256 bits)
#[derive(Clone, PartialEq, Eq)]
pub struct HashOutput {
    /// The raw bytes of the hash output
    bytes: [u8; 32],
}

impl HashOutput {
    /// Create a new hash output from the provided bytes
    pub fn new(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }
    
    /// Get the raw bytes of the hash output
    pub fn as_bytes(&self) -> [u8; 32] {
        self.bytes
    }
    
    /// Convert the hash output to a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.bytes)
    }
    
    /// Create a hash output from a hex string
    pub fn from_hex(hex_str: &str) -> Result<Self, HashError> {
        if hex_str.len() != 64 {
            return Err(HashError::InvalidLength);
        }
        
        let bytes = hex::decode(hex_str)
            .map_err(|_| HashError::InvalidFormat)?;
            
        if bytes.len() != 32 {
            return Err(HashError::InvalidLength);
        }
        
        let mut result = [0u8; 32];
        result.copy_from_slice(&bytes);
        Ok(Self::new(result))
    }
}

// Implement Hash trait for HashOutput so it can be used as a key in HashMap
impl Hash for HashOutput {
    fn hash<H: StdHasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl fmt::Debug for HashOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({})", self.to_hex())
    }
}

impl fmt::Display for HashOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Error type for hash operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashError {
    /// The hash output has an invalid length
    InvalidLength,
    /// The hash input is in an invalid format
    InvalidFormat,
    /// The requested hash algorithm is not supported
    UnsupportedAlgorithm,
    /// Internal error in the hash implementation
    InternalError,
}

impl fmt::Display for HashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength => write!(f, "Invalid hash length"),
            Self::InvalidFormat => write!(f, "Invalid hash format"),
            Self::UnsupportedAlgorithm => write!(f, "Unsupported hash algorithm"),
            Self::InternalError => write!(f, "Internal hash error"),
        }
    }
}

impl std::error::Error for HashError {}

/// Hash function interface
pub trait HashFunction: Send + Sync {
    /// Hash the given data
    fn hash(&self, data: &[u8]) -> HashOutput;
    
    /// Hash multiple inputs together
    fn multi_hash(&self, inputs: &[&[u8]]) -> HashOutput;
    
    /// Create a new incremental hasher
    fn new_hasher(&self) -> Box<dyn Hasher>;
    
    /// Verify that the hash of the given data matches the expected hash
    fn verify(&self, data: &[u8], expected: &HashOutput) -> bool;
}

/// Incremental hasher interface
pub trait Hasher: Send + Sync {
    /// Update the hasher with more data
    fn update(&mut self, data: &[u8]);
    
    /// Finalize the hash and get the result
    fn finalize(&self) -> HashOutput;
    
    /// Reset the hasher to its initial state
    fn reset(&mut self);
}

impl Clone for Box<dyn Hasher> {
    fn clone(&self) -> Self {
        // This is a workaround since we can't directly clone a Box<dyn Trait>
        // Instead, we'll just create a new hasher with a different state
        // This should only be used for testing, not for production code
        unimplemented!("Cannot clone a Box<dyn Hasher> directly")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_output_hex() {
        let bytes = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 
                     16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
        let hash = HashOutput::new(bytes);
        
        // Convert to hex and back
        let hex = hash.to_hex();
        assert_eq!(hex, "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        
        let from_hex = HashOutput::from_hex(&hex).unwrap();
        assert_eq!(hash, from_hex);
    }
    
    #[test]
    fn test_hash_factory() {
        let factory = HashFactory::default();
        let hasher = factory.create_hasher().unwrap();
        
        let data = b"test data";
        let hash = hasher.hash(data);
        
        // Should be deterministic
        let hash2 = hasher.hash(data);
        assert_eq!(hash, hash2);
    }
    
    #[test]
    fn test_hash_trait() {
        use std::collections::HashMap;
        
        let bytes1 = [1; 32];
        let bytes2 = [2; 32];
        let hash1 = HashOutput::new(bytes1);
        let hash2 = HashOutput::new(bytes2);
        
        let mut map = HashMap::new();
        map.insert(hash1.clone(), "value1");
        map.insert(hash2.clone(), "value2");
        
        assert_eq!(map.get(&hash1), Some(&"value1"));
        assert_eq!(map.get(&hash2), Some(&"value2"));
    }
}