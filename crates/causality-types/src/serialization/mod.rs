//! Serialization for Causality types using SSZ (Simple Serialize)
//!
//! This module implements serialization for all core types using the SSZ standard
//! from the Ethereum ecosystem.
//!
//! ## Features
//! 
//! - **Type-safe serialization**: Leverages Rust's type system for safe serialization
//! - **Content addressing**: Supports hash-based content addressing via merkleization
//! - **Cross-language compatibility**: Compatible with Ethereum ecosystem and OCaml implementation
//! - **Performance optimized**: Benchmarked and optimized for common Causality types
//! - **Zero-copy where possible**: Minimizes allocations for better performance

// Define the traits we need
pub trait Encode {
    fn as_ssz_bytes(&self) -> Vec<u8>;
}

pub trait Decode: Sized {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError>;
}

pub trait SimpleSerialize {}

// We don't need a blanket implementation since many types have manual implementations
// Removing this avoids conflicts with manually implemented SimpleSerialize
// impl<T: Encode + Decode> SimpleSerialize for T {}

// Add a macro for SimpleSerialize derive that implements Encode and Decode
#[macro_export]
macro_rules! derive_simple_serialize {
    ($type:ty) => {
        impl $crate::serialization::SimpleSerialize for $type {}
    };
}

// Add a SimpleSerialize derive attribute macro that can be used with #[derive(SimpleSerialize)]
// This only works if the derive feature is enabled
// Removed derive feature - using manual implementations instead

// Define a simple error type
#[derive(Debug, Clone)]
pub struct DecodeError {
    pub message: String,
}

impl DecodeError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SSZ Decode Error: {}", self.message)
    }
}

impl std::error::Error for DecodeError {}

// Implementations for common types
use std::collections::HashMap;
use std::hash::Hash;

impl<K, V> Encode for HashMap<K, V>
where
    K: Encode + Eq + Hash + Clone,
    V: Encode + Clone,
{
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Serialize the number of elements
        bytes.extend((self.len() as u64).as_ssz_bytes());
        // Serialize each key-value pair
        // To ensure deterministic serialization, sort by key first if K: Ord
        // For now, we iterate in arbitrary order, which is fine if the deserialization
        // order matches or if order doesn't matter for the use case.
        // A more robust implementation might require K: Ord and sort here.
        for (key, value) in self {
            bytes.extend(key.as_ssz_bytes());
            bytes.extend(value.as_ssz_bytes());
        }
        bytes
    }
}

impl<K, V> Decode for HashMap<K, V>
where
    K: Decode + Encode + Eq + Hash + Clone,
    V: Decode + Encode + Clone,
{
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut offset = 0;
        // Deserialize the number of elements
        let len = u64::from_ssz_bytes(&bytes[offset..offset + std::mem::size_of::<u64>()])? as usize;
        offset += std::mem::size_of::<u64>();

        let mut map = HashMap::with_capacity(len);
        for _ in 0..len {
            let key = K::from_ssz_bytes(&bytes[offset..])?;
            // This is tricky: we need to know how many bytes the key took up.
            // This requires K::as_ssz_bytes() or a similar mechanism if K is not fixed size.
            // Assuming K has a way to determine its serialized size, or is fixed size.
            // For simplicity, this example might not be robust for all K types.
            // A better approach for variable-size K would be to serialize length-prefixed keys.
            let key_byte_len = key.as_ssz_bytes().len(); // Relies on K implementing Encode to get its size
            offset += key_byte_len;

            let value = V::from_ssz_bytes(&bytes[offset..])?;
            // Similar to key, need to know value's byte length.
            let value_byte_len = value.as_ssz_bytes().len(); // Relies on V implementing Encode
            offset += value_byte_len;
            map.insert(key, value);
        }
        Ok(map)
    }
}

impl<K,V> SimpleSerialize for HashMap<K,V>
where K: Encode + Decode + Eq + Hash + Clone,
      V: Encode + Decode + Clone,
{}

// Re-export from the ssz module
pub mod ssz;
pub use ssz::{serialize, deserialize, serialize_with_depth_limit, deserialize_with_depth_limit, DecodeWithLength};

/// Common utility functions for serialization
pub mod utils {
    use anyhow::Result;
    use sha2::{Digest, Sha256};
    use crate::serialization::Encode;

    /// Computes a content address (ID) for a serializable object
    /// Uses SSZ serialization and SHA-256 hashing
    pub fn compute_content_address<T: Encode>(value: &T) -> Result<[u8; 32]> {
        let serialized = value.as_ssz_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let result = hasher.finalize();

        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        Ok(bytes)
    }

    /// Serializes an object for content addressing
    pub fn serialize_for_content_addressing<T: Encode>(value: &T) -> Vec<u8> {
        value.as_ssz_bytes()
    }

    /// Compute a deterministic hash for a collection of values
    pub fn compute_collection_hash<T>(values: &[T]) -> Result<[u8; 32]>
    where
        T: Encode,
    {
        let mut combined = Vec::new();
        
        for value in values {
            combined.extend_from_slice(&value.as_ssz_bytes());
        }
        
        let mut hasher = Sha256::new();
        hasher.update(&combined);
        let result = hasher.finalize();
        
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        Ok(bytes)
    }
}

/// Convert an SSZ DecodeError to an anyhow Error
pub fn serialization_error_to_anyhow(error: DecodeError) -> anyhow::Error {
    anyhow::anyhow!("SSZ serialization error: {}", error)
}

/// Helper functions for content addressing with SSZ serialization
pub mod content_addressing {
    use anyhow::Result;
    use sha2::{Digest, Sha256};
    use crate::serialization::Encode;

    /// Compute the content address of a value using SSZ serialization and SHA-256
    pub fn compute_content_address<T: Encode>(value: &T) -> Result<[u8; 32]> {
        let serialized = value.as_ssz_bytes();
        let digest = Sha256::digest(&serialized);
        
        let mut result = [0u8; 32];
        result.copy_from_slice(&digest);
        Ok(result)
    }

    /// Helper function to generate a byte array for SSZ-based content addressing
    pub fn generate_bytes_for_content_addressing<T: Encode>(value: &T) -> Vec<u8> {
        value.as_ssz_bytes()
    }
}

/// Foreign Function Interface (FFI) serialization utilities
pub mod ffi;
pub use ffi::{
    serialize_for_ffi, deserialize_from_ffi,
    serialize_to_hex, deserialize_from_hex,
    FfiSerializationError
};

// Merkle tree functionality for SSZ
pub mod merkle;
pub use merkle::{MerkleTree, MerkleProof, verify_proof};

// Re-export derive macros for convenience
// Note: These are only available when the "derive" feature is enabled

