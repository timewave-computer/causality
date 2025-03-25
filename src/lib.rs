// Cryptographic primitives and utilities

// Re-export types from causality-types
pub use causality_types::{
    HashOutput, HashAlgorithm, HashError, ContentId, ContentAddressed, ContentHash
};

// Define the Blake3 implementation for hash functions
use std::sync::Arc;

/// Trait for a hash function implementation
pub trait HashFunction: Send + Sync {
    /// Get the algorithm this function implements
    fn algorithm(&self) -> HashAlgorithm;
    
    /// Hash the given data
    fn hash(&self, data: &[u8]) -> HashOutput;
    
    /// Verify a hash
    fn verify(&self, data: &[u8], hash: &HashOutput) -> bool {
        let computed = self.hash(data);
        computed == *hash
    }
}

/// Blake3 hash function implementation
#[derive(Debug, Clone, Copy)]
pub struct Blake3HashFunction;

impl HashFunction for Blake3HashFunction {
    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Blake3
    }
    
    fn hash(&self, data: &[u8]) -> HashOutput {
        let hash = blake3::hash(data);
        let mut output = [0u8; 32];
        output.copy_from_slice(hash.as_bytes());
        HashOutput::new(output, HashAlgorithm::Blake3)
    }
}

/// Factory for creating hash functions
#[derive(Clone)]
pub struct HashFactory {
    /// Default hash function to use
    default_fn: Arc<dyn HashFunction>,
}

impl HashFactory {
    /// Create a new hash factory with the default hash function
    pub fn new() -> Self {
        Self {
            default_fn: Arc::new(Blake3HashFunction),
        }
    }
    
    /// Create a hash function for the given algorithm
    pub fn create(&self, algorithm: HashAlgorithm) -> Arc<dyn HashFunction> {
        match algorithm {
            HashAlgorithm::Blake3 => Arc::new(Blake3HashFunction),
            HashAlgorithm::Poseidon => {
                // Always fall back to Blake3 for now
                Arc::new(Blake3HashFunction)
            }
        }
    }
    
    /// Hash data with the default function
    pub fn hash(&self, data: &[u8]) -> HashOutput {
        self.default_fn.hash(data)
    }
} 