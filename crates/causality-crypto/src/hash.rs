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

// Import types from causality-types crate
use causality_types::{
    HashOutput, HashAlgorithm, HashError, ContentId, 
    ContentAddressed, ContentHash
};

/// Output of a hash function with algorithm awareness
#[derive(Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct HashOutput {
    /// The raw bytes of the hash
    data: [u8; 32],
    /// The algorithm used to generate this hash
    algorithm: HashAlgorithm,
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
    
    /// Get a deterministic unique identifier for this hash
    pub fn to_content_id(&self) -> ContentId {
        ContentId::from(*self)
    }
}

impl fmt::Debug for HashOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HashOutput({}, {})", self.algorithm, self.to_hex())
    }
}

impl fmt::Display for HashOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex_string())
    }
}

/// A content-derived identifier replacing UUID for object identification
#[derive(Clone, Debug, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
pub struct ContentId(HashOutput);

impl ContentId {
    /// Create a new ContentId from a hash output
    pub fn from(hash: HashOutput) -> Self {
        Self(hash)
    }
    
    /// Create a zero-value ContentId for use as a placeholder
    pub fn nil() -> Self {
        let zero_bytes = [0u8; 32];
        Self(HashOutput::new(zero_bytes, HashAlgorithm::default()))
    }
    
    /// Get the underlying hash output
    pub fn hash(&self) -> &HashOutput {
        &self.0
    }
    
    /// Get raw bytes from the content id
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
    
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("cid:{}", self.0.to_hex_string())
    }
    
    /// Create a new ContentId from a string
    pub fn new(id: impl Into<String>) -> Self {
        // Convert to string first
        let id_str = id.into();
        
        // If it looks like a properly formatted ContentId, try to parse it
        if id_str.starts_with("cid:") {
            if let Ok(content_id) = Self::parse(&id_str) {
                return content_id;
            }
        }
        
        // Otherwise, treat it as raw data and hash it
        Self::from(id_str.as_bytes())
    }
    
    /// Parse from string
    pub fn parse(s: &str) -> Result<Self, HashError> {
        if let Some(hex) = s.strip_prefix("cid:") {
            let hash = HashOutput::from_hex_string(hex)?;
            Ok(Self(hash))
        } else {
            Err(HashError::InvalidFormat)
        }
    }
}

impl fmt::Display for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<&[u8]> for ContentId {
    fn from(data: &[u8]) -> Self {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().expect("Failed to create hasher");
        Self(hasher.hash(data))
    }
}

/// Hash algorithm options
#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
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
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Interface for hash functions
pub trait HashFunction: Send + Sync {
    /// Hash the provided data
    fn hash(&self, data: &[u8]) -> HashOutput;
    
    /// Get the algorithm used by this hash function
    fn algorithm(&self) -> HashAlgorithm;
}

/// Extension trait for content addressing support
pub trait ContentHasher: HashFunction {
    /// Hash a content-addressed object
    fn hash_object<T: ContentAddressed>(&self, object: &T) -> HashOutput {
        self.hash(&object.to_bytes())
    }
    
    /// Verify a content hash against an object
    fn verify_object<T: ContentAddressed>(&self, object: &T, hash: &HashOutput) -> bool {
        let computed = self.hash_object(object);
        computed == *hash
    }
}

/// Implement ContentHasher for all HashFunction implementations
impl<T: HashFunction + ?Sized> ContentHasher for T {}

/// Trait for content-addressed objects
pub trait ContentAddressed {
    /// Get the content hash of this object
    fn content_hash(&self) -> HashOutput;
    
    /// Get a deterministic identifier derived from content
    fn content_id(&self) -> ContentId {
        ContentId::from(self.content_hash())
    }
    
    /// Verify that the object matches its hash
    fn verify(&self) -> bool;
    
    /// Convert to a serialized form for storage using Borsh
    fn to_bytes(&self) -> Vec<u8>;
    
    /// Create from serialized form using Borsh
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> where Self: Sized;
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
    
    /// Hash a content-addressed object
    pub fn hash_object<T: ContentAddressed>(&self, object: &T) -> HashOutput {
        self.function.hash_object(object)
    }
    
    /// Verify a content hash against an object
    pub fn verify_object<T: ContentAddressed>(&self, object: &T, hash: &HashOutput) -> bool {
        self.function.verify_object(object, hash)
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
        HashOutput::new(output, HashAlgorithm::Blake3)
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
        
        HashOutput::new(result, HashAlgorithm::Poseidon)
    }
    
    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Poseidon
    }
}

/// Interface for deferred hash computation
pub trait DeferredHashing {
    /// Request a hash computation (creates a placeholder)
    fn request_hash(
        &mut self, 
        data: &[u8], 
        algorithm: HashAlgorithm
    ) -> DeferredHashId;
    
    /// Check if a deferred hash result is available
    fn has_hash_result(&self, id: &DeferredHashId) -> bool;
    
    /// Get the result of a deferred hash operation
    fn get_hash_result(&self, id: &DeferredHashId) -> Option<HashOutput>;
    
    /// Perform all deferred hash computations
    fn compute_deferred_hashes(&mut self);
}

/// A deferred hash ID for content that will be hashed later
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeferredHashId(String);

impl DeferredHashId {
    /// Create a new deferred hash ID
    pub fn new() -> Self {
        // Generate a content ID with a random nonce
        let content = DeferredIdContent {
            creation_time: chrono::Utc::now().timestamp_millis(),
            nonce: rand::random::<[u8; 16]>(),
        };
        
        let content_id = content.content_id();
        Self(format!("deferred:{}", content_id))
    }
    
    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<ContentId> for DeferredHashId {
    fn from(content_id: ContentId) -> Self {
        Self(format!("deferred:{}", content_id))
    }
}

/// Content type for deferred hash IDs
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
struct DeferredIdContent {
    /// Creation timestamp
    creation_time: i64,
    /// Random nonce for uniqueness
    nonce: [u8; 16],
}

impl ContentAddressed for DeferredIdContent {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
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