// Hashing functions and types
// Original file: src/crypto/hash.rs

// Hash functionality for cryptographic operations
//
// This module provides hash functions used throughout the system,
// with a focus on cryptographic properties needed for secure operations.

use std::fmt;
use std::sync::Arc;
use thiserror::Error;
// Note: Borsh dependency might be removable if not used by remaining logic.
use borsh::{BorshSerialize, BorshDeserialize};
use std::str::FromStr;
use rand;
use lazy_static;

// Import core crypto types from causality-types
// These types define the structure and basic methods (like new, as_bytes, algorithm).
// The actual hashing logic (traits and implementations) resides in *this* crate.
use causality_types::crypto_primitives::{HashAlgorithm, HashError, HashOutput, ContentHash};

// --- Local definitions and impls for HashError, HashAlgorithm, HashOutput, ContentHash REMOVED --- 
// --- The canonical definitions and their impls are in causality-types::crypto_primitives --- 

/// Interface for cryptographic hash functions.
/// Implementors provide specific hashing algorithms (e.g., Blake3, Poseidon).
pub trait HashFunction: Send + Sync {
    /// Compute a hash of the given bytes using the specific algorithm.
    fn hash(&self, data: &[u8]) -> HashOutput;
    
    /// Get the algorithm identifier used by this hash function.
    fn algorithm(&self) -> HashAlgorithm;
    
    /// Create a Hasher instance for incremental hashing with this algorithm.
    fn create_hasher(&self) -> Box<dyn ContentHasher>;
}

/// Interface for content hashers that allow building hashes incrementally.
pub trait ContentHasher: Send + Sync {
    /// Update the hasher state with more data.
    fn update(&mut self, data: &[u8]);
    
    /// Finalize the hash computation and return the result.
    fn finalize(&self) -> HashOutput;
    
    /// Get the algorithm identifier used by this hasher.
    fn algorithm(&self) -> HashAlgorithm;
    
    /// Reset the hasher to its initial state, ready for a new hash computation.
    fn reset(&mut self);
    
    /// Convenience method to hash data in one step using this hasher's algorithm.
    fn hash(&self, data: &[u8]) -> HashOutput {
        let mut hasher = self.reset_copy();
        hasher.update(data);
        hasher.finalize()
    }
    
    /// Create a fresh copy of this hasher in its initial state.
    fn reset_copy(&self) -> Box<dyn ContentHasher>;
}

/// Factory for creating instances of hash functions and hashers based on algorithm.
pub struct HashFactory {
    /// The default algorithm to use when none is specified.
    default_algorithm: HashAlgorithm,
}

impl HashFactory {
    /// Create a new HashFactory with the specified default algorithm.
    pub fn new(default_algorithm: HashAlgorithm) -> Self {
        Self { default_algorithm }
    }
    
    /// Create a hash function instance for the specified algorithm.
    pub fn create_hash_function(&self, algorithm: HashAlgorithm) -> Result<Box<dyn HashFunction>, HashError> {
        match algorithm {
            HashAlgorithm::Blake3 => Ok(Box::new(Blake3HashFunction::new())),
            #[cfg(feature = "poseidon")]
            HashAlgorithm::Poseidon => Ok(Box::new(PoseidonHashFunction::new())), // Assuming PoseidonHashFunction::new() exists in poseidon_impl
            #[cfg(not(feature = "poseidon"))]
            HashAlgorithm::Poseidon => Err(HashError::UnsupportedAlgorithm("Poseidon feature not enabled".to_string())),
            // Note: This match might become non-exhaustive if HashAlgorithm enum changes in types crate
        }
    }
    
    /// Create a content hasher instance for the specified algorithm.
    pub fn create_content_hasher(&self, algorithm: HashAlgorithm) -> Result<Box<dyn ContentHasher>, HashError> {
        match algorithm {
            HashAlgorithm::Blake3 => Ok(Box::new(Blake3Hasher::new())),
            #[cfg(feature = "poseidon")]
            HashAlgorithm::Poseidon => Ok(Box::new(PoseidonHasher::new())), // Assuming PoseidonHasher::new() exists in poseidon_impl
            #[cfg(not(feature = "poseidon"))]
            HashAlgorithm::Poseidon => Err(HashError::UnsupportedAlgorithm("Poseidon feature not enabled".to_string())),
             // Note: This match might become non-exhaustive if HashAlgorithm enum changes in types crate
       }
    }
    
    /// Create a hash function instance using the factory's default algorithm.
    pub fn create_default_hash_function(&self) -> Result<Box<dyn HashFunction>, HashError> {
        self.create_hash_function(self.default_algorithm)
    }
    
    /// Create a content hasher instance using the factory's default algorithm.
    pub fn create_default_content_hasher(&self) -> Result<Box<dyn ContentHasher>, HashError> {
        self.create_content_hasher(self.default_algorithm)
    }
}

impl Default for HashFactory {
    fn default() -> Self {
        // Assumes the imported HashAlgorithm impls Default (which it does in crypto_primitives)
        Self::new(HashAlgorithm::default())
    }
}

// --- BLAKE3 Implementation ---

/// BLAKE3 implementation of the `HashFunction` trait.
#[derive(Clone, Copy, Debug, Default)] // Added Default
pub struct Blake3HashFunction;

impl Blake3HashFunction {
    /// Create a new Blake3HashFunction instance.
    pub fn new() -> Self {
        Self // Since it's a ZST
    }
}

impl HashFunction for Blake3HashFunction {
    fn hash(&self, data: &[u8]) -> HashOutput {
        let hash = blake3::hash(data);
        // Use the public constructor `::new()` from the imported `HashOutput` type.
        // The canonical HashOutput definition takes [u8; 32].
        HashOutput::new(*hash.as_bytes(), HashAlgorithm::Blake3)
    }
    
    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Blake3
    }
    
    fn create_hasher(&self) -> Box<dyn ContentHasher> {
        Box::new(Blake3Hasher::new())
    }
}

/// BLAKE3 implementation of the `ContentHasher` trait for incremental hashing.
#[derive(Clone)] // `blake3::Hasher` is Clone
pub struct Blake3Hasher {
    hasher: blake3::Hasher,
}

impl Blake3Hasher {
    /// Create a new Blake3Hasher instance.
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
        // Use the public constructor `::new()` from the imported `HashOutput` type.
        HashOutput::new(*hash.as_bytes(), HashAlgorithm::Blake3)
    }
    
    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::Blake3
    }
    
    fn reset(&mut self) {
        // Re-initialize the internal blake3 hasher state.
        self.hasher = blake3::Hasher::new();
    }
    
    fn reset_copy(&self) -> Box<dyn ContentHasher> {
        // Create a fresh Blake3Hasher instance.
        Box::new(Self::new())
    }
}


// --- Poseidon Implementation (Conditional) ---

#[cfg(feature = "poseidon")]
mod poseidon_impl {
    // Import necessary types from parent module and types crate
    use super::{HashFunction, ContentHasher}; // Import traits from parent
    use causality_types::crypto_primitives::{HashAlgorithm, HashError, HashOutput};
    // Assuming a Poseidon library/module exists, e.g., within crate::zk::poseidon
    // use crate::zk::poseidon::{RealPoseidonHash, RealPoseidonHasher, PoseidonParameters};
    use std::sync::Arc;

    /// Poseidon hash function implementation (Placeholder)
    #[derive(Debug, Clone, Default)] // Added derive Default
    pub struct PoseidonHashFunction {
        // Placeholder: In a real implementation, might hold parameters
        // params: Arc<PoseidonParameters>,
    }
    
    impl PoseidonHashFunction {
        /// Create a new Poseidon hash function instance.
        pub fn new() -> Self {
            // Placeholder: Load/initialize parameters if needed
            Self::default()
        }
    }
    
    // Implement the trait from the parent module
    impl super::HashFunction for PoseidonHashFunction {
        fn hash(&self, data: &[u8]) -> HashOutput {
            // Placeholder: Implement actual Poseidon hashing logic here.
            let placeholder_hash = blake3::hash(data); // Temporary
            HashOutput::new(*placeholder_hash.as_bytes(), HashAlgorithm::Poseidon)
        }
        
        fn algorithm(&self) -> HashAlgorithm {
            HashAlgorithm::Poseidon
        }
        
        fn create_hasher(&self) -> Box<dyn super::ContentHasher> {
            Box::new(PoseidonHasher::new())
        }
    }
    
    /// Poseidon content hasher implementation (Placeholder)
    #[derive(Debug, Clone)] // Added derive
    pub struct PoseidonHasher {
        // Placeholder: Internal state for Poseidon hashing
        state: Vec<u8>, // Example state, real implementation depends on Poseidon library
    }
    
    impl PoseidonHasher {
        /// Create a new Poseidon hasher instance.
        pub fn new() -> Self {
            Self { state: Vec::new() } // Initialize state as needed
        }
    }

    impl Default for PoseidonHasher {
        fn default() -> Self {
            Self::new()
        }
    }
    
    // Implement the trait from the parent module
    impl super::ContentHasher for PoseidonHasher {
        fn update(&mut self, data: &[u8]) {
            // Placeholder: Update internal Poseidon state based on `data`.
            self.state.extend_from_slice(data); // Simple placeholder logic
        }
        
        fn finalize(&self) -> HashOutput {
            // Placeholder: Finalize Poseidon hash based on internal state.
            let placeholder_hash = blake3::hash(&self.state); // Temporary
            HashOutput::new(*placeholder_hash.as_bytes(), HashAlgorithm::Poseidon)
        }
        
        fn algorithm(&self) -> HashAlgorithm {
            HashAlgorithm::Poseidon
        }
        
        fn reset(&mut self) {
            // Placeholder: Reset internal Poseidon state.
            self.state.clear(); // Simple placeholder logic
        }
        
        fn reset_copy(&self) -> Box<dyn super::ContentHasher> {
            // Create a fresh PoseidonHasher instance.
            Box::new(PoseidonHasher::new())
        }
    }
}

// Conditionally export the Poseidon implementations if the feature is enabled.
#[cfg(feature = "poseidon")]
pub use poseidon_impl::{PoseidonHashFunction, PoseidonHasher};

// --- Default Hash Implementations ---

lazy_static::lazy_static! {
    /// Default hash factory instance (uses Blake3 by default).
    pub static ref DEFAULT_HASH_FACTORY: HashFactory = HashFactory::default();
    /// Default hash function instance (Blake3).
    pub static ref DEFAULT_HASH_FUNCTION: Box<dyn HashFunction> = DEFAULT_HASH_FACTORY.create_default_hash_function().expect("Failed to create default hash function");
    /// Default content hasher instance (Blake3).
    pub static ref DEFAULT_CONTENT_HASHER: Box<dyn ContentHasher> = DEFAULT_HASH_FACTORY.create_default_content_hasher().expect("Failed to create default content hasher");
}

/// Convenience function to hash data using the default hash function (Blake3).
pub fn default_hash(data: &[u8]) -> HashOutput {
    DEFAULT_HASH_FUNCTION.hash(data)
}

/// Convenience function to create a default content hasher (Blake3).
pub fn create_default_hasher() -> Box<dyn ContentHasher> {
    DEFAULT_CONTENT_HASHER.reset_copy()
}

/// Generate a random hash output for testing or placeholder purposes.
/// Uses the default algorithm (Blake3).
pub fn random_hash() -> HashOutput {
    let random_bytes: [u8; 32] = rand::random();
    // Use the ::new method from the imported HashOutput
    HashOutput::new(random_bytes, HashAlgorithm::default())
}

/// Create a deterministic hash output from a string using the default algorithm.
pub fn hash_from_string(s: &str) -> HashOutput {
    default_hash(s.as_bytes())
}


// --- Checksum Implementation (Non-Cryptographic) ---
// This section seems separate from the main cryptographic hashing.

/// Output type for non-cryptographic checksum functions (e.g., MD5).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ChecksumOutput {
    /// The raw bytes of the checksum (length depends on algorithm).
    data: Vec<u8>,
}

impl ChecksumOutput {
    /// Create a new checksum output from raw bytes.
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
    
    /// Get the raw bytes of the checksum.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    
    /// Convert the checksum output to a hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(&self.data)
    }
    
    /// Create a checksum output from a hex string.
    /// Returns `HashError::InvalidFormat` if decoding fails.
    pub fn from_hex(hex_str: &str) -> Result<Self, HashError> {
        let bytes = hex::decode(hex_str)
            // Use imported HashError variant
            .map_err(|_| HashError::InvalidFormat)?;
        
        Ok(Self::new(bytes))
    }
}

impl fmt::Display for ChecksumOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Checksum algorithm options (non-cryptographic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumAlgorithm {
    /// MD5 checksum algorithm.
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

/// Interface for non-cryptographic checksum functions.
pub trait ChecksumFunction: Send + Sync {
    /// Compute the checksum of the provided data.
    fn checksum(&self, data: &[u8]) -> ChecksumOutput;
    
    /// Get the algorithm identifier used by this checksum function.
    fn algorithm(&self) -> ChecksumAlgorithm;
}

/// A concrete checksum implementation wrapper.
pub struct Checksum {
    function: Arc<dyn ChecksumFunction>,
}

impl Checksum {
    /// Create a new checksum wrapper with the given function implementation.
    pub fn new(function: Arc<dyn ChecksumFunction>) -> Self {
        Self { function }
    }
    
    /// Compute the checksum of the provided data using the wrapped function.
    pub fn checksum(&self, data: &[u8]) -> ChecksumOutput {
        self.function.checksum(data)
    }
    
    /// Get the algorithm used by the wrapped checksum function.
    pub fn algorithm(&self) -> ChecksumAlgorithm {
        self.function.algorithm()
    }
}

/// Factory for creating checksum function instances.
#[derive(Clone)]
pub struct ChecksumFactory {
    default_algorithm: ChecksumAlgorithm,
}

impl ChecksumFactory {
    /// Create a new checksum factory with the specified default algorithm.
    pub fn new(default_algorithm: ChecksumAlgorithm) -> Self {
        Self { default_algorithm }
    }
    
    /// Create a new checksum factory using the default algorithm (MD5).
    pub fn default() -> Self {
        Self::new(ChecksumAlgorithm::default())
    }
    
    /// Create a checksum instance using the factory's default algorithm.
    pub fn create_checksum(&self) -> Result<Checksum, HashError> {
        self.create_checksum_with_algorithm(self.default_algorithm)
    }
    
    /// Create a checksum instance with the specified algorithm.
    pub fn create_checksum_with_algorithm(&self, algorithm: ChecksumAlgorithm) -> Result<Checksum, HashError> {
        match algorithm {
            ChecksumAlgorithm::Md5 => {
                #[cfg(feature = "md5")]
                {
                    // Ensure the MD5 feature provides Md5ChecksumFunction
                    let function = Arc::new(Md5ChecksumFunction::new());
                    Ok(Checksum::new(function))
                }
                #[cfg(not(feature = "md5"))]
                {
                    // Use imported HashError variant
                    Err(HashError::UnsupportedAlgorithm("MD5 checksum feature not enabled".to_string()))
                }
            },
        }
    }
}


/// MD5 checksum function implementation (conditional on 'md5' feature).
#[cfg(feature = "md5")]
#[derive(Debug, Clone, Copy, Default)] // Added derive Default
pub struct Md5ChecksumFunction;

#[cfg(feature = "md5")]
impl Md5ChecksumFunction {
    /// Create a new MD5 checksum function instance.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Compute the MD5 hash directly without creating an instance.
    /// Requires the `md5` crate to be available.
    pub fn compute(data: &[u8]) -> ChecksumOutput {
        // Use the md5 crate (expected to be available via features)
        let digest = md5::compute(data);
        // MD5 result is 16 bytes ([u8; 16])
        ChecksumOutput::new(digest.0.to_vec()) // Convert [u8; 16] to Vec<u8>
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


// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    // Use the imported types directly from causality_types
    use causality_types::crypto_primitives::{HashAlgorithm, HashOutput, ContentHash, HashError};

    // Test cryptographic hashing (using imported types)
    #[test]
    fn test_hash_output_methods() {
        let data = [0xab; 32];
        let hash_output = HashOutput::new(data, HashAlgorithm::Blake3);
        
        // Test public methods from canonical HashOutput
        assert_eq!(hash_output.algorithm(), HashAlgorithm::Blake3);
        assert_eq!(hash_output.as_bytes(), &data);
        
        // Assuming the canonical HashOutput has to_hex_string()
        let hex_string = hash_output.to_hex_string(); 
        assert_eq!(hex_string, "blake3:abababababababababababababababababababababababababababababababab");
        
        // Assuming the canonical HashOutput has from_hex_string()
        let parsed_output = HashOutput::from_hex_string(&hex_string).unwrap();
        assert_eq!(parsed_output, hash_output);
        
        // Test error cases for from_hex_string
        assert!(matches!(HashOutput::from_hex_string("invalid"), Err(HashError::InvalidFormat)));
        assert!(matches!(HashOutput::from_hex_string("blake3:invalidhex"), Err(HashError::InvalidFormat)));
        assert!(matches!(HashOutput::from_hex_string("blake3:010203"), Err(HashError::InvalidLength))); // Test length error
        assert!(matches!(HashOutput::from_hex_string("unknown:abababababababababababababababababababababababababababababababab"), Err(HashError::UnsupportedAlgorithm(_))));
    }

    #[test]
    fn test_blake3_hash_function_impl() {
        let hash_fn = Blake3HashFunction::new();
        let data1 = b"hello";
        let data2 = b"world";
        
        let hash1 = hash_fn.hash(data1);
        let hash2 = hash_fn.hash(data1);
        let hash3 = hash_fn.hash(data2);
        
        assert_eq!(hash1.algorithm(), HashAlgorithm::Blake3);
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_blake3_content_hasher_impl() {
        let data = b"incremental hashing test";
        // Use the default_hash helper which relies on the HashFunction impl
        let full_hash = default_hash(data);
        
        // Use the create_default_hasher helper which relies on ContentHasher impl
        let mut hasher = create_default_hasher();
        assert_eq!(hasher.algorithm(), HashAlgorithm::Blake3);
        
        hasher.update(&data[0..10]);
        hasher.update(&data[10..]);
        let incremental_hash = hasher.finalize();
        
        assert_eq!(incremental_hash, full_hash);
        
        // Test reset
        hasher.reset();
        hasher.update(b"something else");
        let hash_after_reset = hasher.finalize();
        assert_ne!(hash_after_reset, full_hash);
        assert_ne!(hash_after_reset, incremental_hash);
        let expected_reset_hash = default_hash(b"something else");
        assert_eq!(hash_after_reset, expected_reset_hash);
    }

    #[test]
    fn test_default_hash_helpers_consistency() {
        let data = b"test data";
        let hash1 = default_hash(data);
        let hash2 = DEFAULT_HASH_FUNCTION.hash(data);
        assert_eq!(hash1, hash2);
        
        let mut hasher = create_default_hasher();
        hasher.update(data);
        let hash3 = hasher.finalize();
        assert_eq!(hash1, hash3);
    }

    #[test]
    fn test_random_hash_differs() {
        let mut seen_hashes = HashSet::new();
        for _ in 0..100 {
            let hash = random_hash();
            // Check it uses the default algorithm
            assert_eq!(hash.algorithm(), HashAlgorithm::default());
            // Check that hashes are unique (highly probable for random 32 bytes)
            assert!(seen_hashes.insert(hash));
        }
    }

    #[test]
    fn test_hash_from_string_consistency() {
        let str1 = "unique string 1";
        let str2 = "unique string 2";
        
        let hash1 = hash_from_string(str1);
        let hash2 = hash_from_string(str1);
        let hash3 = hash_from_string(str2);
        
        // Should be consistent and use default algorithm
        assert_eq!(hash1.algorithm(), HashAlgorithm::default());
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
    
    #[test]
    fn test_hash_algorithm_from_str_imported() {
        // Assumes canonical HashAlgorithm impls FromStr (which it does in crypto_primitives)
        assert_eq!(HashAlgorithm::from_str("blake3").unwrap(), HashAlgorithm::Blake3);
        assert_eq!(HashAlgorithm::from_str("BLAKE3").unwrap(), HashAlgorithm::Blake3);
        
        #[cfg(feature = "poseidon")]
        {
            assert_eq!(HashAlgorithm::from_str("poseidon").unwrap(), HashAlgorithm::Poseidon);
            assert_eq!(HashAlgorithm::from_str("Poseidon").unwrap(), HashAlgorithm::Poseidon);
        }
        
        #[cfg(not(feature = "poseidon"))]
        {
            assert!(matches!(HashAlgorithm::from_str("poseidon"), Err(HashError::UnsupportedAlgorithm(_))));
        }
        
        // Check error type for unknown algorithm
        assert!(matches!(HashAlgorithm::from_str("sha256"), Err(HashError::UnsupportedAlgorithm(_))));
    }

    // Test Checksum functions
    #[test]
    #[cfg(feature = "md5")]
    fn test_md5_checksum_impl() {
        let checksum_fn = Md5ChecksumFunction::new();
        let data = b"test data for checksum";
        let checksum = checksum_fn.checksum(data);
        
        // MD5 should be 16 bytes
        assert_eq!(checksum.as_bytes().len(), 16);
        // Check algorithm id
        assert_eq!(checksum_fn.algorithm(), ChecksumAlgorithm::Md5);
        
        let checksum2 = checksum_fn.checksum(data);
        assert_eq!(checksum, checksum2);
        
        let different_data = b"different data";
        let different_checksum = checksum_fn.checksum(different_data);
        assert_ne!(checksum, different_checksum);
        
        // Test the static compute method
        let computed_checksum = Md5ChecksumFunction::compute(data);
        assert_eq!(checksum, computed_checksum);
    }

    #[test]
    fn test_checksum_factory_logic() {
        let factory = ChecksumFactory::default();
        
        // Default algorithm should be MD5
        assert_eq!(factory.default_algorithm, ChecksumAlgorithm::Md5);
        
        #[cfg(feature = "md5")]
        {
            // Create an MD5 checksum instance via factory
            let md5_checksum_res = factory.create_checksum_with_algorithm(ChecksumAlgorithm::Md5);
            assert!(md5_checksum_res.is_ok());
            let md5_checksum = md5_checksum_res.unwrap();
            assert_eq!(md5_checksum.algorithm(), ChecksumAlgorithm::Md5);
            
            // Create a default checksum instance (should be MD5)
            let default_checksum_res = factory.create_checksum();
            assert!(default_checksum_res.is_ok());
            let default_checksum = default_checksum_res.unwrap();
            assert_eq!(default_checksum.algorithm(), ChecksumAlgorithm::Md5);
        }
        
        #[cfg(not(feature = "md5"))]
        {
            // Creating MD5 checksum should fail if feature is not enabled
             let md5_checksum_res = factory.create_checksum_with_algorithm(ChecksumAlgorithm::Md5);
             assert!(matches!(md5_checksum_res, Err(HashError::UnsupportedAlgorithm(_))));
             let default_checksum_res = factory.create_checksum();
             assert!(matches!(default_checksum_res, Err(HashError::UnsupportedAlgorithm(_))));
        }
    }

    #[test]
    fn test_checksum_output_hex_conversion() {
        // Example MD5 hash (hex)
        let md5_hex = "d8e8fca2dc0f896fd7cb4cb0031ba249"; 
        let checksum_from_hex = ChecksumOutput::from_hex(md5_hex).unwrap();
        
        // Check bytes length
        assert_eq!(checksum_from_hex.as_bytes().len(), 16);
        
        // Convert back to hex
        let hex_from_checksum = checksum_from_hex.to_hex();
        
        // Should be the same as the original hex (lowercase)
        assert_eq!(hex_from_checksum, md5_hex);
        
        // Test invalid hex
        assert!(matches!(ChecksumOutput::from_hex("invalid hex string"), Err(HashError::InvalidFormat)));
    }

    // ContentHash related tests are commented out as ContentHash is defined and implemented in causality-types
    /* 
    #[test]
    fn test_content_hash_display_and_to_string() {
        // ... This test would need to use the canonical ContentHash from types ...
    }
    
    #[test]
    fn test_content_hash_from_hash_output() {
        // ... This test would need to use the canonical ContentHash from types ...
    }
    */
} 