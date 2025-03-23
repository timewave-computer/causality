use super::{HashFunction, HashError};
use super::blake3_impl::Blake3Hash32;
use std::sync::Arc;

/// Supported hash algorithm types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// Blake3 hash algorithm (default)
    Blake3,
    /// Poseidon hash algorithm (for ZK-friendly operations)
    Poseidon,
}

impl Default for HashAlgorithm {
    fn default() -> Self {
        Self::Blake3
    }
}

/// Factory for creating hash function instances
#[derive(Debug, Clone)]
pub struct HashFactory {
    /// Default algorithm to use
    default_algo: HashAlgorithm,
}

impl Default for HashFactory {
    fn default() -> Self {
        Self {
            default_algo: HashAlgorithm::default(),
        }
    }
}

impl HashFactory {
    /// Create a new hash factory with the specified default algorithm
    pub fn new(default_algo: HashAlgorithm) -> Self {
        Self { default_algo }
    }
    
    /// Get the default hash algorithm
    pub fn default_algorithm(&self) -> HashAlgorithm {
        self.default_algo
    }
    
    /// Create a new hash function with the default algorithm
    pub fn create_hasher(&self) -> Result<Box<dyn HashFunction>, HashError> {
        self.create_hasher_with_algorithm(self.default_algo)
    }
    
    /// Create a new hash function with the specified algorithm
    pub fn create_hasher_with_algorithm(&self, algo: HashAlgorithm) -> Result<Box<dyn HashFunction>, HashError> {
        match algo {
            HashAlgorithm::Blake3 => Ok(Box::new(Blake3Hash32::default())),
            HashAlgorithm::Poseidon => Err(HashError::UnsupportedAlgorithm),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::HashOutput;
    
    #[test]
    fn test_hash_factory() {
        // Create factory with default (Blake3)
        let factory = HashFactory::default();
        assert_eq!(factory.default_algorithm(), HashAlgorithm::Blake3);
        
        // Create a hasher
        let hasher = factory.create_hasher().unwrap();
        
        // Should successfully hash data
        let data = b"test data";
        let hash = hasher.hash(data);
        
        // Should be deterministic
        let hasher2 = factory.create_hasher().unwrap();
        let hash2 = hasher2.hash(data);
        assert_eq!(hash, hash2);
    }
    
    #[test]
    fn test_unsupported_algorithm() {
        let factory = HashFactory::default();
        
        // Poseidon is not yet supported
        let result = factory.create_hasher_with_algorithm(HashAlgorithm::Poseidon);
        assert!(result.is_err());
        
        if let Err(HashError::UnsupportedAlgorithm) = result {
            // Expected error
        } else {
            panic!("Expected UnsupportedAlgorithm error");
        }
    }
} 