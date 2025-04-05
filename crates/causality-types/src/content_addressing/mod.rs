// Content addressing system for the Causality project
//
// This module provides utilities for content addressing, hashing, and canonical serialization.

use crate::crypto_primitives::{HashError, HashOutput, HashAlgorithm, ContentId};

// Export storage module
pub mod storage;

// Extended set of types and functions for content addressing

/// Standard type for content hash
pub type StandardContentHash = HashOutput;

/// Universal content addressing algorithm
pub const STANDARD_HASH_ALGORITHM: HashAlgorithm = HashAlgorithm::Blake3;

/// Core content hash conversion related error
#[derive(Debug, thiserror::Error)]
pub enum ContentHashConversionError {
    /// Hash algorithm mismatch
    #[error("Hash algorithm mismatch: expected {expected}, found {found}")]
    AlgorithmMismatch {
        expected: String,
        found: String,
    },
    
    /// Invalid hash format
    #[error("Invalid hash format: {0}")]
    InvalidFormat(String),
    
    /// Invalid hash length
    #[error("Invalid hash length: expected {expected}, found {found}")]
    InvalidLength {
        expected: usize,
        found: usize,
    },
    
    /// Hash error
    #[error("Hash error: {0}")]
    HashError(#[from] HashError),
}

/// Convert a hex string to raw bytes
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, ContentHashConversionError> {
    hex::decode(hex).map_err(|e| ContentHashConversionError::InvalidFormat(e.to_string()))
}

/// Convert raw bytes to hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Create a content hash from raw bytes using the standard algorithm
pub fn content_hash_from_bytes(bytes: &[u8]) -> HashOutput {
    let mut data = [0u8; 32];
    let hash_result = blake3::hash(bytes);
    let hash_bytes = hash_result.as_bytes();
    data.copy_from_slice(hash_bytes);
    HashOutput::new(data, STANDARD_HASH_ALGORITHM)
}

/// Create a content ID from raw bytes
pub fn content_id_from_bytes(bytes: &[u8]) -> ContentId {
    ContentId::from_bytes(bytes)
}

/// Create a content ID from a string
pub fn content_id_from_string(s: &str) -> ContentId {
    ContentId::new(s)
}

/// Normalize a content hash string representation
pub fn normalize_content_hash_string(hash_str: &str) -> Result<String, ContentHashConversionError> {
    if let Some(idx) = hash_str.find(':') {
        let algorithm = &hash_str[0..idx];
        let hex = &hash_str[idx+1..];
        
        // Validate hex portion
        hex_to_bytes(hex)?;
        
        // Return normalized form
        Ok(format!("{}:{}", algorithm.to_lowercase(), hex.to_lowercase()))
    } else {
        Err(ContentHashConversionError::InvalidFormat(
            "Content hash string must contain algorithm prefix".to_string()
        ))
    }
}

/// Check if a string is a valid content hash representation
pub fn is_valid_content_hash_string(hash_str: &str) -> bool {
    normalize_content_hash_string(hash_str).is_ok()
}

/// Module for canonical serialization support
pub mod canonical {
    use super::*;
    use serde::{Serialize, Deserialize};
    use serde_json::{Value, Map};
    use thiserror::Error;
    
    /// Error type for canonical serialization operations
    #[derive(Debug, Error)]
    pub enum CanonicalSerializationError {
        /// JSON serialization error
        #[error("JSON serialization error: {0}")]
        JsonError(String),
        
        /// Binary serialization error
        #[error("Binary serialization error: {0}")]
        BinaryError(String),
        
        /// Unsupported type
        #[error("Unsupported type: {0}")]
        UnsupportedType(String),
    }
    
    /// Convert an object to canonical JSON format
    pub fn to_canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalSerializationError> {
        // Step 1: Convert to a JSON Value
        let json_value = serde_json::to_value(value)
            .map_err(|e| CanonicalSerializationError::JsonError(e.to_string()))?;
        
        // Step 2: Normalize the JSON Value
        let normalized_value = normalize_json_value(json_value);
        
        // Step 3: Serialize to bytes with sorted keys
        let canonical_json = serde_json::to_string(&normalized_value)
            .map_err(|e| CanonicalSerializationError::JsonError(e.to_string()))?;
        
        Ok(canonical_json.into_bytes())
    }
    
    /// Convert an object to canonical binary format (borsh by default)
    pub fn to_canonical_binary<T: borsh::BorshSerialize>(value: &T) -> Result<Vec<u8>, CanonicalSerializationError> {
        value.try_to_vec()
            .map_err(|e| CanonicalSerializationError::BinaryError(e.to_string()))
    }
    
    /// Deserialize from canonical JSON format
    pub fn from_canonical_json<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, CanonicalSerializationError> {
        let json_str = std::str::from_utf8(bytes)
            .map_err(|e| CanonicalSerializationError::JsonError(e.to_string()))?;
        
        serde_json::from_str(json_str)
            .map_err(|e| CanonicalSerializationError::JsonError(e.to_string()))
    }
    
    /// Deserialize from canonical binary format
    pub fn from_canonical_binary<T: borsh::BorshDeserialize>(bytes: &[u8]) -> Result<T, CanonicalSerializationError> {
        T::try_from_slice(bytes)
            .map_err(|e| CanonicalSerializationError::BinaryError(e.to_string()))
    }
    
    /// Normalize a JSON value (sort maps, etc.)
    fn normalize_json_value(value: Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut new_map = Map::new();
                
                // Get all keys and sort them
                let mut keys: Vec<String> = map.keys().cloned().collect();
                keys.sort();
                
                // Add entries in sorted order
                for key in keys {
                    if let Some(val) = map.get(&key) {
                        new_map.insert(key, normalize_json_value(val.clone()));
                    }
                }
                
                Value::Object(new_map)
            }
            Value::Array(arr) => {
                let new_arr = arr.into_iter()
                    .map(normalize_json_value)
                    .collect();
                
                Value::Array(new_arr)
            }
            // Other JSON value types are kept as is
            _ => value,
        }
    }
    
    /// Helper to serialize content-addressed objects to canonical format
    pub trait CanonicalSerialize {
        /// Serialize to canonical JSON format
        fn to_canonical_json(&self) -> Result<Vec<u8>, CanonicalSerializationError>;
        
        /// Serialize to canonical binary format
        fn to_canonical_binary(&self) -> Result<Vec<u8>, CanonicalSerializationError>;
    }
    
    impl<T: Serialize + borsh::BorshSerialize> CanonicalSerialize for T {
        fn to_canonical_json(&self) -> Result<Vec<u8>, CanonicalSerializationError> {
            to_canonical_json(self)
        }
        
        fn to_canonical_binary(&self) -> Result<Vec<u8>, CanonicalSerializationError> {
            to_canonical_binary(self)
        }
    }
    
    /// Compute content hash using canonical serialization
    pub fn content_hash_canonical<T: Serialize + borsh::BorshSerialize>(
        value: &T, 
        algorithm: HashAlgorithm
    ) -> Result<HashOutput, CanonicalSerializationError> {
        // Use binary format for hashing by default
        let bytes = to_canonical_binary(value)?;
        
        let mut data = [0u8; 32];
        
        match algorithm {
            HashAlgorithm::Blake3 => {
                let hash_result = blake3::hash(&bytes);
                data.copy_from_slice(hash_result.as_bytes());
            }
            // Remove Poseidon arm
            /*
            HashAlgorithm::Poseidon => {
                // This would use a Poseidon implementation
                // As placeholder, we'll use Blake3
                let hash_result = blake3::hash(&bytes);
                data.copy_from_slice(hash_result.as_bytes());
            }
            */
            // TODO: Handle Sha256, Sha512, Custom variants
            _ => {
                // For now, default to Blake3 for unhandled algorithms as a placeholder
                // Ideally, this should return an error or use the correct hasher
                let hash_result = blake3::hash(&bytes);
                data.copy_from_slice(hash_result.as_bytes());
            }
        }
        
        Ok(HashOutput::new(data, algorithm))
    }
}

/// Create a content hash using canonical serialization
pub fn canonical_content_hash<T: serde::Serialize + borsh::BorshSerialize>(
    value: &T
) -> Result<HashOutput, canonical::CanonicalSerializationError> {
    canonical::content_hash_canonical(value, STANDARD_HASH_ALGORITHM)
}

/// Create a content ID using canonical serialization
pub fn canonical_content_id<T: serde::Serialize + borsh::BorshSerialize>(
    value: &T
) -> Result<ContentId, canonical::CanonicalSerializationError> {
    let hash = canonical_content_hash(value)?;
    Ok(ContentId::from(hash))
}

// Default implementation for ContentId
impl Default for ContentId {
    fn default() -> Self {
        ContentId::from_bytes(&[0; 32])
    }
}

// Test specific to Poseidon hashing (if applicable)
/*
#[test]
fn test_poseidon_hashing() {
    let data = b"some data for poseidon hashing";
    let hasher = PoseidonHasher {}; // Assuming a specific Poseidon hasher exists
    let hash_output = hasher.hash(data).expect("Poseidon hashing failed");
    
    assert_eq!(hash_output.algorithm(), &HashAlgorithm::Poseidon);
    // Add more specific assertions for Poseidon if needed
}
*/ 