// Hashing functions and types
// Original file: src/crypto/hash.rs

// Hash functionality for cryptographic operations
//
// This module provides hash functions used throughout the system,
// with a focus on cryptographic properties needed for secure operations.

use std::fmt;
use std::sync::Arc;
use thiserror::Error;
use borsh::{BorshSerialize, BorshDeserialize};
use std::str::FromStr;
use rand;

// Define our own types that would normally come from causality-types
// Note: Remove when causality-types dependency is restored

/// Error that can occur during hashing operations
#[derive(Debug, Error)]
pub enum HashError {
    /// Hash algorithm not supported
    #[error("Unsupported hash algorithm: {0}")]
    UnsupportedAlgorithm(String),
    
    /// Invalid hash format
    #[error("Invalid hash format")]
    InvalidFormat,
    
    /// Invalid hash length
    #[error("Invalid hash length")]
    InvalidLength,
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// I/O error 
    #[error("I/O error: {0}")]
    IoError(String),
    
    /// Internal error during hashing
    #[error("Internal hash error: {0}")]
    InternalError(String),
}

/// Hash algorithm options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
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
            Self::Blake3 => write!(f, "blake3"),
            Self::Poseidon => write!(f, "poseidon"),
        }
    }
}

impl FromStr for HashAlgorithm {
    type Err = HashError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "blake3" => Ok(Self::Blake3),
            "poseidon" => Ok(Self::Poseidon),
            _ => Err(HashError::UnsupportedAlgorithm(s.to_string())),
        }
    }
}

/// Output of a hash function with algorithm awareness
#[derive(Clone, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize, Debug)]
pub struct HashOutput {
    /// The raw bytes of the hash
    data: [u8; 32],
    /// The algorithm used to generate this hash
    algorithm: HashAlgorithm,
}

/// Content hash with algorithm information
#[derive(Debug, Clone, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
pub struct ContentHash {
    /// The algorithm used for hashing
    pub algorithm: String,
    /// The raw hash bytes
    pub bytes: Vec<u8>,
}

impl ContentHash {
    /// Create a new content hash with the specified algorithm and bytes
    pub fn new(algorithm: &str, bytes: Vec<u8>) -> Self {
        Self {
            algorithm: algorithm.to_string(),
            bytes,
        }
    }
    
    /// Create a ContentHash from a HashOutput
    pub fn from_hash_output(hash_output: &HashOutput) -> Self {
        let algorithm = hash_output.algorithm().to_string();
        let bytes = hash_output.as_bytes().to_vec();
        Self::new(&algorithm, bytes)
    }
    
    /// Convert to a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }
    
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.algorithm.to_lowercase(), self.to_hex())
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl HashOutput {
    /// Create a new hash output from raw bytes with the specified algorithm
    pub fn new(data: [u8; 32], algorithm: HashAlgorithm) -> Self {
        Self { data, algorithm }
    }
    
    /// Get the raw bytes of the hash
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    
    /// Get the algorithm used to generate this hash
    pub fn algorithm(&self) -> HashAlgorithm {
        self.algorithm
    }
    
    /// Convert the hash output to a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.data)
    }
    
    /// Create a hash output from a hex string with the specified algorithm
    pub fn from_hex(hex_str: &str, algorithm: HashAlgorithm) -> Result<Self, HashError> {
        let bytes = hex::decode(hex_str)
            .map_err(|_| HashError::InvalidFormat)?;
        
        if bytes.len() != 32 {
            return Err(HashError::InvalidLength);
        }
        
        let mut data = [0u8; 32];
        data.copy_from_slice(&bytes);
        Ok(Self::new(data, algorithm))
    }
    
    /// Convert the hash output to a hex string with algorithm prefix
    pub fn to_hex_string(&self) -> String {
        format!("{}:{}", self.algorithm.to_string().to_lowercase(), self.to_hex())
    }
    
    /// Create a hash output from a hex string with algorithm prefix
    pub fn from_hex_string(s: &str) -> Result<Self, HashError> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(HashError::InvalidFormat);
        }
        
        let algorithm = HashAlgorithm::from_str(parts[0])
            .map_err(|_| HashError::UnsupportedAlgorithm(parts[0].to_string()))?;
        
        Self::from_hex(parts[1], algorithm)
    }
}

impl fmt::Display for HashOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex_string())
    }
}

/// Interface for hash functions
pub trait HashFunction: Send + Sync {
    /// Compute a hash of the given bytes
    fn hash(&self, data: &[u8]) -> HashOutput;
    
    /// Get the algorithm used by this hash function
    fn algorithm(&self) -> HashAlgorithm;
    
    /// Create a Hasher that can incrementally build a hash
    fn create_hasher(&self) -> Box<dyn ContentHasher>;
}

/// Interface for content hashers that can build hashes incrementally
pub trait ContentHasher: Send + Sync {
    /// Update the hasher with more data
    fn update(&mut self, data: &[u8]);
    
    /// Finalize and get the hash output
    fn finalize(&self) -> HashOutput;
    
    /// Get the algorithm used by this hasher
    fn algorithm(&self) -> HashAlgorithm;
    
    /// Reset the hasher to its initial state
    fn reset(&mut self);
    
    /// Convenience method to hash data in one step
    fn hash(&self, data: &[u8]) -> HashOutput {
        let mut hasher = self.reset_copy();
        hasher.update(data);
        hasher.finalize()
    }
    
    /// Create a copy of this hasher in its initial state
    fn reset_copy(&self) -> Box<dyn ContentHasher>;
}

/// Factory for creating hash functions and hashers
pub struct HashFactory {
    /// The default algorithm to use
    default_algorithm: HashAlgorithm,
}

impl HashFactory {
    /// Create a new HashFactory with the specified default algorithm
    pub fn new(default_algorithm: HashAlgorithm) -> Self {
        Self { default_algorithm }
    }
    
    /// Create a hash function for the specified algorithm
    pub fn create_hash_function(&self, algorithm: HashAlgorithm) -> Result<Box<dyn HashFunction>, HashError> {
        match algorithm {
            HashAlgorithm::Blake3 => Ok(Box::new(Blake3HashFunction)),
            #[cfg(feature = "poseidon")]
            HashAlgorithm::Poseidon => Ok(Box::new(PoseidonHashFunction::new())),
            #[cfg(not(feature = "poseidon"))]
            HashAlgorithm::Poseidon => Err(HashError::UnsupportedAlgorithm("poseidon".to_string())),
        }
    }
    
    /// Create a content hasher for the default algorithm
    pub fn create_hasher(&self) -> Result<Box<dyn ContentHasher>, HashError> {
        self.create_hasher_with_algorithm(self.default_algorithm)
    }
    
    /// Create a content hasher for the specified algorithm
    pub fn create_hasher_with_algorithm(&self, algorithm: HashAlgorithm) -> Result<Box<dyn ContentHasher>, HashError> {
        Ok(self.create_hash_function(algorithm)?.create_hasher())
    }
}

impl Default for HashFactory {
    fn default() -> Self {
        Self::new(HashAlgorithm::Blake3)
    }
}

/// BLAKE3 implementation of HashFunction
#[derive(Clone, Copy, Debug)]
pub struct Blake3HashFunction;

impl Blake3HashFunction {
    /// Create a new Blake3HashFunction
    pub fn new() -> Self {
        Self
    }
}

impl HashFunction for Blake3HashFunction {
    fn hash(&self, data: &[u8]) -> HashOutput {
        let hash = blake3::hash(data);
        let mut output = [0u8; 32];
        output.copy_from_slice(hash.as_bytes());
        HashOutput::new(output, HashAlgorithm::Blake3)
    }
    
    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Blake3
    }
    
    fn create_hasher(&self) -> Box<dyn ContentHasher> {
        Box::new(Blake3Hasher::new())
    }
}

/// BLAKE3 implementation of ContentHasher
#[derive(Clone)]
pub struct Blake3Hasher {
    hasher: blake3::Hasher,
}

impl Blake3Hasher {
    /// Create a new Blake3Hasher
    pub fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
        }
    }
}

impl Default for Blake3Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentHasher for Blake3Hasher {
    fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }
    
    fn finalize(&self) -> HashOutput {
        let hash = self.hasher.finalize();
        let mut output = [0u8; 32];
        output.copy_from_slice(hash.as_bytes());
        HashOutput::new(output, HashAlgorithm::Blake3)
    }
    
    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Blake3
    }
    
    fn reset(&mut self) {
        self.hasher = blake3::Hasher::new();
    }
    
    fn reset_copy(&self) -> Box<dyn ContentHasher> {
        Box::new(Self::new())
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
        
        HashOutput::new(result, HashAlgorithm::Poseidon)
    }
    
    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Poseidon
    }
    
    fn create_hasher(&self) -> Box<dyn ContentHasher> {
        Box::new(PoseidonHasher::new())
    }
}

/// Checksum output type for non-cryptographic hash functions
#[derive(Clone, PartialEq, Eq)]
pub struct ChecksumOutput {
    /// The raw bytes of the checksum
    data: Vec<u8>,
}

impl ChecksumOutput {
    /// Create a new checksum output from raw bytes
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
    
    /// Get the raw bytes of the checksum
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    
    /// Convert the checksum output to a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.data)
    }
    
    /// Create a checksum output from a hex string
    pub fn from_hex(hex_str: &str) -> Result<Self, HashError> {
        let bytes = hex::decode(hex_str)
            .map_err(|_| HashError::InvalidFormat)?;
        
        Ok(Self::new(bytes))
    }
}

impl fmt::Debug for ChecksumOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChecksumOutput({})", self.to_hex())
    }
}

impl fmt::Display for ChecksumOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Checksum algorithm options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumAlgorithm {
    /// MD5 checksum algorithm
    Md5,
}

impl Default for ChecksumAlgorithm {
    fn default() -> Self {
        Self::Md5
    }
}

impl fmt::Display for ChecksumAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Md5 => write!(f, "MD5"),
        }
    }
}

/// Interface for checksum functions
pub trait ChecksumFunction: Send + Sync {
    /// Compute the checksum of the provided data
    fn checksum(&self, data: &[u8]) -> ChecksumOutput;
    
    /// Get the algorithm used by this checksum function
    fn algorithm(&self) -> ChecksumAlgorithm;
}

/// A concrete checksum implementation
pub struct Checksum {
    function: Arc<dyn ChecksumFunction>,
}

impl Checksum {
    /// Create a new checksum with the given function
    pub fn new(function: Arc<dyn ChecksumFunction>) -> Self {
        Self { function }
    }
    
    /// Compute the checksum of the provided data
    pub fn checksum(&self, data: &[u8]) -> ChecksumOutput {
        self.function.checksum(data)
    }
    
    /// Get the algorithm used by this checksum
    pub fn algorithm(&self) -> ChecksumAlgorithm {
        self.function.algorithm()
    }
}

/// Factory for creating checksum functions
#[derive(Clone)]
pub struct ChecksumFactory {
    default_algorithm: ChecksumAlgorithm,
}

impl ChecksumFactory {
    /// Create a new checksum factory with the specified default algorithm
    pub fn new(default_algorithm: ChecksumAlgorithm) -> Self {
        Self { default_algorithm }
    }
    
    /// Create a new checksum factory with the default algorithm
    pub fn default() -> Self {
        Self::new(ChecksumAlgorithm::default())
    }
    
    /// Create a checksum using the default algorithm
    pub fn create_checksum(&self) -> Result<Checksum, HashError> {
        self.create_checksum_with_algorithm(self.default_algorithm)
    }
    
    /// Create a checksum with the specified algorithm
    pub fn create_checksum_with_algorithm(&self, algorithm: ChecksumAlgorithm) -> Result<Checksum, HashError> {
        match algorithm {
            ChecksumAlgorithm::Md5 => {
                #[cfg(feature = "md5")]
                {
                    let function = Arc::new(Md5ChecksumFunction::new());
                    Ok(Checksum::new(function))
                }
                #[cfg(not(feature = "md5"))]
                {
                    Err(HashError::UnsupportedAlgorithm("MD5 not enabled".to_string()))
                }
            },
        }
    }
}

/// MD5 checksum function implementation
#[cfg(feature = "md5")]
pub struct Md5ChecksumFunction;

#[cfg(feature = "md5")]
impl Md5ChecksumFunction {
    /// Create a new MD5 checksum function
    pub fn new() -> Self {
        Self
    }
    
    /// Compute the MD5 hash directly without creating an instance
    pub fn compute(data: &[u8]) -> ChecksumOutput {
        let digest = md5::compute(data);
        let mut output = Vec::with_capacity(16);
        output.extend_from_slice(digest.as_ref());
        ChecksumOutput::new(output)
    }
}

#[cfg(feature = "md5")]
impl ChecksumFunction for Md5ChecksumFunction {
    fn checksum(&self, data: &[u8]) -> ChecksumOutput {
        Self::compute(data)
    }
    
    fn algorithm(&self) -> ChecksumAlgorithm {
        ChecksumAlgorithm::Md5
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
        let hash = HashOutput::new(data, HashAlgorithm::Blake3);
        
        // Convert to hex
        let hex = hash.to_hex();
        
        // Check hex length
        assert_eq!(hex.len(), 64);
        
        // Recreate from hex
        let recreated = HashOutput::from_hex(&hex, HashAlgorithm::Blake3).unwrap();
        
        // Should be the same as the original
        assert_eq!(hash, recreated);
    }

    #[test]
    #[cfg(feature = "md5")]
    fn test_md5_checksum() {
        let checksum_fn = Md5ChecksumFunction::new();
        let data = b"test data for checksum";
        let checksum = checksum_fn.checksum(data);
        
        // MD5 should be 16 bytes
        assert_eq!(checksum.as_bytes().len(), 16);
        
        // Checksumming the same data twice should produce the same result
        let checksum2 = checksum_fn.checksum(data);
        assert_eq!(checksum, checksum2);
        
        // Checksumming different data should produce different checksums
        let different_data = b"different data";
        let different_checksum = checksum_fn.checksum(different_data);
        assert_ne!(checksum, different_checksum);
    }

    #[test]
    fn test_checksum_factory() {
        let factory = ChecksumFactory::default();
        
        // Default algorithm should be MD5
        assert_eq!(factory.default_algorithm, ChecksumAlgorithm::Md5);
        
        #[cfg(feature = "md5")]
        {
            // Create an MD5 checksum
            let md5_checksum = factory.create_checksum_with_algorithm(ChecksumAlgorithm::Md5).unwrap();
            assert_eq!(md5_checksum.algorithm(), ChecksumAlgorithm::Md5);
            
            // Create a default checksum (should be MD5)
            let default_checksum = factory.create_checksum().unwrap();
            assert_eq!(default_checksum.algorithm(), ChecksumAlgorithm::Md5);
        }
    }

    #[test]
    fn test_checksum_output_hex() {
        let data = vec![1u8; 16];
        let checksum = ChecksumOutput::new(data);
        
        // Convert to hex
        let hex = checksum.to_hex();
        
        // Check hex length (MD5 is 16 bytes = 32 hex chars)
        assert_eq!(hex.len(), 32);
        
        // Recreate from hex
        let recreated = ChecksumOutput::from_hex(&hex).unwrap();
        
        // Should be the same as the original
        assert_eq!(checksum, recreated);
    }

    #[test]
    #[cfg(feature = "md5")]
    fn test_md5_direct_compute() {
        let data = b"test data";
        
        // Using the function instance
        let checksum1 = Md5ChecksumFunction::new().checksum(data);
        
        // Using the static compute method
        let checksum2 = Md5ChecksumFunction::compute(data);
        
        // Both should be identical
        assert_eq!(checksum1, checksum2);
    }
} 