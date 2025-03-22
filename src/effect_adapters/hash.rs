// Hash module for Causality Effect Adapters
//
// This module provides functionality for calculating content hashes
// using Blake3 initially, with future support for Poseidon hash planned.

use std::fmt;
use std::convert::TryFrom;
use std::str::FromStr;

use serde::{Serialize, Deserialize, Serializer, Deserializer};
use blake3::Hasher as Blake3Hasher;
use hex;

/// The type of hash algorithm used
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    /// Blake3 hash algorithm
    Blake3,
    /// Poseidon hash algorithm (not yet implemented)
    #[allow(dead_code)]
    Poseidon,
}

impl std::hash::Hash for HashAlgorithm {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl HashAlgorithm {
    /// Get the digest size of the algorithm in bytes
    pub fn digest_size(&self) -> usize {
        match self {
            HashAlgorithm::Blake3 => 32, // 256 bits
            HashAlgorithm::Poseidon => 32, // 256 bits
        }
    }
    
    /// Convert to a string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            HashAlgorithm::Blake3 => "blake3",
            HashAlgorithm::Poseidon => "poseidon",
        }
    }
}

impl FromStr for HashAlgorithm {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "blake3" => Ok(HashAlgorithm::Blake3),
            "poseidon" => Ok(HashAlgorithm::Poseidon),
            _ => Err(format!("Unknown hash algorithm: {}", s)),
        }
    }
}

/// A content hash with algorithm information
#[derive(Clone, PartialEq, Eq)]
pub struct Hash {
    /// The hash algorithm used
    pub algorithm: HashAlgorithm,
    /// The hash bytes
    pub bytes: Vec<u8>,
}

impl std::hash::Hash for Hash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.algorithm.as_str().hash(state);
        self.bytes.hash(state);
    }
}

impl Hash {
    /// Create a new hash value
    pub fn new(algorithm: HashAlgorithm, bytes: Vec<u8>) -> Self {
        assert_eq!(bytes.len(), algorithm.digest_size(), 
                   "Hash size mismatch for algorithm {:?}", algorithm);
        Hash { algorithm, bytes }
    }
    
    /// Create a Blake3 hash from bytes
    pub fn blake3(bytes: Vec<u8>) -> Self {
        assert_eq!(bytes.len(), HashAlgorithm::Blake3.digest_size());
        Hash {
            algorithm: HashAlgorithm::Blake3,
            bytes,
        }
    }
    
    /// Convert to a hex string
    pub fn to_hex(&self) -> String {
        let mut result = String::with_capacity(2 + 2 * self.bytes.len());
        result.push_str("0x");
        for byte in &self.bytes {
            result.push_str(&format!("{:02x}", byte));
        }
        result
    }
    
    /// Check if this is a Blake3 hash
    pub fn is_blake3(&self) -> bool {
        self.algorithm == HashAlgorithm::Blake3
    }
    
    /// Check if this is a Poseidon hash
    pub fn is_poseidon(&self) -> bool {
        self.algorithm == HashAlgorithm::Poseidon
    }

    /// Create a Hash from an Option<String>
    pub fn from_option_string(hash_str: &Option<String>) -> Option<Self> {
        match hash_str {
            Some(hash) => Hash::from_str(hash).ok(),
            None => None,
        }
    }
    
    /// Convert a Hash to an Option<String>
    pub fn to_option_string(&self) -> Option<String> {
        Some(self.to_string())
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm.as_str(), self.to_hex())
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm.as_str(), self.to_hex())
    }
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}:{}", self.algorithm.as_str(), self.to_hex()))
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Hash::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for Hash {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Check format
        if !s.contains(':') {
            return Err(format!("Invalid hash format: {}", s));
        }
        
        // Split algorithm and hash
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid hash format: {}", s));
        }
        
        // Parse algorithm
        let algorithm = match parts[0] {
            "blake3" => HashAlgorithm::Blake3,
            "poseidon" => HashAlgorithm::Poseidon,
            _ => return Err(format!("Unknown hash algorithm: {}", parts[0])),
        };
        
        // Parse hash
        let hash_hex = parts[1];
        let hash_bytes = match hex::decode(hash_hex) {
            Ok(bytes) => bytes,
            Err(e) => return Err(format!("Invalid hex in hash: {}", e)),
        };
        
        Ok(Hash {
            algorithm,
            bytes: hash_bytes,
        })
    }
}

/// A hasher trait for calculating content hashes
pub trait ContentHasher: Send + Sync {
    /// Calculate a hash of the given bytes
    fn hash_bytes(&self, bytes: &[u8]) -> Hash;
    
    /// Get the algorithm used by this hasher
    fn algorithm(&self) -> HashAlgorithm;
}

/// Extended hasher trait for serializable objects
pub trait ObjectHasher: ContentHasher {
    /// Calculate a hash of a serializable object
    fn hash_object<T: Serialize>(&self, object: &T) -> Hash;
}

/// Blake3 hasher implementation
#[derive(Debug, Clone)]
pub struct Blake3ContentHasher;

impl Blake3ContentHasher {
    /// Create a new Blake3 hasher
    pub fn new() -> Self {
        Blake3ContentHasher
    }
}

impl Default for Blake3ContentHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentHasher for Blake3ContentHasher {
    fn hash_bytes(&self, bytes: &[u8]) -> Hash {
        let mut hasher = Blake3Hasher::new();
        hasher.update(bytes);
        let hash_bytes = hasher.finalize().as_bytes().to_vec();
        Hash::blake3(hash_bytes)
    }
    
    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Blake3
    }
}

impl ObjectHasher for Blake3ContentHasher {
    fn hash_object<T: Serialize>(&self, object: &T) -> Hash {
        let bytes = bincode::serialize(object)
            .expect("Failed to serialize object for hashing");
        self.hash_bytes(&bytes)
    }
}

/// Factory for creating appropriate hashers
#[derive(Debug, Clone)]
pub struct HasherFactory;

impl HasherFactory {
    /// Create a new hasher factory
    pub fn new() -> Self {
        HasherFactory
    }
    
    /// Create a hasher for the specified algorithm
    pub fn create_hasher(&self, algorithm: HashAlgorithm) -> Box<dyn ContentHasher> {
        match algorithm {
            HashAlgorithm::Blake3 => Box::new(Blake3ContentHasher::new()),
            HashAlgorithm::Poseidon => {
                // Poseidon not yet implemented, fall back to Blake3
                Box::new(Blake3ContentHasher::new())
            }
        }
    }
    
    /// Create a default hasher (currently Blake3)
    pub fn default_hasher(&self) -> Box<dyn ContentHasher> {
        Box::new(Blake3ContentHasher::new())
    }
}

impl Default for HasherFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blake3_hash() {
        let hasher = Blake3ContentHasher::new();
        let hash = hasher.hash_bytes(b"hello world");
        
        assert_eq!(hash.algorithm, HashAlgorithm::Blake3);
        assert_eq!(hash.bytes.len(), 32);
    }
    
    #[test]
    fn test_hash_serialization() {
        let hasher = Blake3ContentHasher::new();
        let hash = hasher.hash_bytes(b"test serialization");
        
        let serialized = serde_json::to_string(&hash).unwrap();
        let deserialized: Hash = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(hash, deserialized);
    }
    
    #[test]
    fn test_hash_from_str() {
        let s = "blake3:0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let hash = Hash::from_str(s).unwrap();
        
        assert_eq!(hash.algorithm, HashAlgorithm::Blake3);
        assert_eq!(hash.to_string(), s);
    }
} 