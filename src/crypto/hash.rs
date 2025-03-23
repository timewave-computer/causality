// Hash functionality for cryptographic operations
//
// This module provides hash functions used throughout the system,
// with a focus on cryptographic properties needed for secure operations.

use std::fmt;
use std::sync::Arc;
use thiserror::Error;

/// Output of a hash function
#[derive(Clone, PartialEq, Eq)]
pub struct HashOutput {
    /// The raw bytes of the hash
    data: [u8; 32],
}

impl HashOutput {
    /// Create a new hash output from raw bytes
    pub fn new(data: [u8; 32]) -> Self {
        Self { data }
    }
    
    /// Get the raw bytes of the hash
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    
    /// Convert the hash output to a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.data)
    }
    
    /// Create a hash output from a hex string
    pub fn from_hex(hex_str: &str) -> Result<Self, HashError> {
        let bytes = hex::decode(hex_str)
            .map_err(|_| HashError::InvalidFormat)?;
        
        if bytes.len() != 32 {
            return Err(HashError::InvalidLength);
        }
        
        let mut data = [0u8; 32];
        data.copy_from_slice(&bytes);
        Ok(Self::new(data))
    }
}

impl fmt::Debug for HashOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HashOutput({})", self.to_hex())
    }
}

impl fmt::Display for HashOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Hash algorithm options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// BLAKE3 cryptographic hash function
    Blake3,
    /// Poseidon hash function (ZK-friendly)
    Poseidon,
}

impl Default for HashAlgorithm {
    fn default() -> Self {
        Self::Blake3
    }
}

impl fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blake3 => write!(f, "Blake3"),
            Self::Poseidon => write!(f, "Poseidon"),
        }
    }
}

/// Error type for hash operations
#[derive(Debug, Error)]
pub enum HashError {
    /// Invalid hash format
    #[error("Invalid hash format")]
    InvalidFormat,
    
    /// Invalid hash length
    #[error("Invalid hash length")]
    InvalidLength,
    
    /// Unsupported hash algorithm
    #[error("Unsupported hash algorithm: {0}")]
    UnsupportedAlgorithm(String),
    
    /// Internal error during hashing
    #[error("Internal hash error: {0}")]
    InternalError(String),
}

/// Interface for hash functions
pub trait HashFunction: Send + Sync {
    /// Hash the provided data
    fn hash(&self, data: &[u8]) -> HashOutput;
    
    /// Get the algorithm used by this hash function
    fn algorithm(&self) -> HashAlgorithm;
}

/// A concrete hasher implementation
pub struct Hasher {
    function: Arc<dyn HashFunction>,
}

impl Hasher {
    /// Create a new hasher with the given hash function
    pub fn new(function: Arc<dyn HashFunction>) -> Self {
        Self { function }
    }
    
    /// Hash the provided data
    pub fn hash(&self, data: &[u8]) -> HashOutput {
        self.function.hash(data)
    }
    
    /// Get the algorithm used by this hasher
    pub fn algorithm(&self) -> HashAlgorithm {
        self.function.algorithm()
    }
}

/// Factory for creating hash functions
#[derive(Clone)]
pub struct HashFactory {
    default_algorithm: HashAlgorithm,
}

impl HashFactory {
    /// Create a new hash factory with the specified default algorithm
    pub fn new(default_algorithm: HashAlgorithm) -> Self {
        Self { default_algorithm }
    }
    
    /// Create a new hash factory with the default algorithm
    pub fn default() -> Self {
        Self::new(HashAlgorithm::default())
    }
    
    /// Create a hasher using the default algorithm
    pub fn create_hasher(&self) -> Result<Hasher, HashError> {
        self.create_hasher_with_algorithm(self.default_algorithm)
    }
    
    /// Create a hasher with the specified algorithm
    pub fn create_hasher_with_algorithm(&self, algorithm: HashAlgorithm) -> Result<Hasher, HashError> {
        match algorithm {
            HashAlgorithm::Blake3 => {
                let function = Arc::new(Blake3HashFunction::new());
                Ok(Hasher::new(function))
            },
            HashAlgorithm::Poseidon => {
                #[cfg(feature = "poseidon")]
                {
                    let function = Arc::new(PoseidonHashFunction::new());
                    Ok(Hasher::new(function))
                }
                #[cfg(not(feature = "poseidon"))]
                {
                    Err(HashError::UnsupportedAlgorithm("Poseidon not enabled".to_string()))
                }
            },
        }
    }
}

/// BLAKE3 hash function implementation
pub struct Blake3HashFunction;

impl Blake3HashFunction {
    /// Create a new BLAKE3 hash function
    pub fn new() -> Self {
        Self
    }
}

impl HashFunction for Blake3HashFunction {
    fn hash(&self, data: &[u8]) -> HashOutput {
        let hash = blake3::hash(data);
        let mut output = [0u8; 32];
        output.copy_from_slice(hash.as_bytes());
        HashOutput::new(output)
    }
    
    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Blake3
    }
}

/// Poseidon hash function implementation
#[cfg(feature = "poseidon")]
pub struct PoseidonHashFunction;

#[cfg(feature = "poseidon")]
impl PoseidonHashFunction {
    /// Create a new Poseidon hash function
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "poseidon")]
impl HashFunction for PoseidonHashFunction {
    fn hash(&self, data: &[u8]) -> HashOutput {
        // This is a placeholder implementation since we don't have a real Poseidon implementation yet
        // In a real implementation, this would use the actual Poseidon algorithm
        
        // For now, we'll use a simple algorithm that provides at least some different outputs
        let mut result = [0u8; 32];
        
        // Simple hash: XOR data bytes into the result
        for (i, byte) in data.iter().enumerate() {
            result[i % 32] ^= *byte;
        }
        
        // Add a marker to distinguish from Blake3
        result[0] = 0xAA;
        
        HashOutput::new(result)
    }
    
    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Poseidon
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blake3_hash() {
        let hasher = Blake3HashFunction::new();
        let data = b"test data for hashing";
        let hash = hasher.hash(data);
        
        // The hash should be 32 bytes
        assert_eq!(hash.as_bytes().len(), 32);
        
        // Hashing the same data twice should produce the same hash
        let hash2 = hasher.hash(data);
        assert_eq!(hash, hash2);
        
        // Hashing different data should produce different hashes
        let different_data = b"different data";
        let different_hash = hasher.hash(different_data);
        assert_ne!(hash, different_hash);
    }
    
    #[test]
    #[cfg(feature = "poseidon")]
    fn test_poseidon_hash() {
        let hasher = PoseidonHashFunction::new();
        let data = b"test data for hashing";
        let hash = hasher.hash(data);
        
        // The hash should be 32 bytes
        assert_eq!(hash.as_bytes().len(), 32);
        
        // Hashing the same data twice should produce the same hash
        let hash2 = hasher.hash(data);
        assert_eq!(hash, hash2);
        
        // Hashing different data should produce different hashes
        let different_data = b"different data";
        let different_hash = hasher.hash(different_data);
        assert_ne!(hash, different_hash);
    }
    
    #[test]
    fn test_hash_factory() {
        let factory = HashFactory::default();
        
        // Default algorithm should be Blake3
        assert_eq!(factory.default_algorithm, HashAlgorithm::Blake3);
        
        // Create a Blake3 hasher
        let blake3_hasher = factory.create_hasher_with_algorithm(HashAlgorithm::Blake3).unwrap();
        assert_eq!(blake3_hasher.algorithm(), HashAlgorithm::Blake3);
        
        // Create a default hasher (should be Blake3)
        let default_hasher = factory.create_hasher().unwrap();
        assert_eq!(default_hasher.algorithm(), HashAlgorithm::Blake3);
    }
    
    #[test]
    fn test_hash_output_hex() {
        let data = [1u8; 32];
        let hash = HashOutput::new(data);
        
        // Convert to hex
        let hex = hash.to_hex();
        
        // Check hex length
        assert_eq!(hex.len(), 64);
        
        // Recreate from hex
        let recreated = HashOutput::from_hex(&hex).unwrap();
        
        // Should be the same as the original
        assert_eq!(hash, recreated);
    }
} 