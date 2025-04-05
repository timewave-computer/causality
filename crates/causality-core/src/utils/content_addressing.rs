//! Content addressing utilities
//!
//! This module provides utilities to work with content addressing,
//! particularly for converting between different content hash/ID implementations.

use anyhow::Result;
use serde_json;
use hex;
use causality_types::crypto_primitives::ContentId;
use causality_crypto;
use blake3;

/// Trait for objects that can be content addressed
pub trait ContentAddressed {
    /// Get the content hash for this object
    fn content_hash(&self) -> Result<causality_crypto::ContentHash, anyhow::Error>;
}

/// Convert legacy crypto::ContentHash to types::ContentHash
pub fn convert_legacy_to_types_hash(hash: &causality_crypto::ContentHash) -> causality_types::crypto_primitives::ContentHash {
    // we'll convert to string and then decode the hex
    let hex_str = hash.to_string();
    let bytes = hex::decode(hex_str).unwrap_or_else(|_| vec![0; 32]);
    
    // Create a new causality_types::crypto_primitives::ContentHash
    causality_types::crypto_primitives::ContentHash::new("blake3", bytes)
}

/// Convert from causality_types::ContentHash to causality_types::ContentId
pub fn content_hash_to_id(hash: &causality_types::crypto_primitives::ContentHash) -> causality_types::crypto_primitives::ContentId {
    // Create a HashOutput from the ContentHash
    let algorithm = match hash.algorithm.as_str() {
        "blake3" => causality_types::crypto_primitives::HashAlgorithm::Blake3,
        _ => causality_types::crypto_primitives::HashAlgorithm::Blake3 // Default to Blake3
    };
    
    // Create a HashOutput and ContentId
    let hash_output = hash.to_hash_output().unwrap();
    causality_types::crypto_primitives::ContentId::from(hash_output)
}

/// Convert from causality_types::ContentId to causality_types::ContentHash
pub fn content_id_to_hash(id: &causality_types::crypto_primitives::ContentId) -> causality_types::crypto_primitives::ContentHash {
    // Use the ContentId's hash method to get the HashOutput
    let hash_output = id.hash();
    
    // Convert the HashOutput to ContentHash
    causality_types::crypto_primitives::ContentHash::from_hash_output(hash_output)
}

/// Convert back from crypto_primitives::ContentId
pub fn convert_primitive_content_id(
    id: &ContentId
) -> ContentId {
    // crypto_primitives::ContentId has method hash() to get HashOutput
    // and doesn't have algorithm/value as fields
    let hash_output = id.hash();
    let algorithm = format!("{}", hash_output.algorithm());
    
    ContentId::new(&algorithm)
}

/// Generate a ContentHash from a string
pub fn hash_string(s: &str) -> causality_types::crypto_primitives::ContentHash {
    // Hash the string using blake3
    let bytes = s.as_bytes();
    let hash_result = blake3::hash(bytes);
    let hash_bytes = hash_result.as_bytes().to_vec();
    
    // Create a properly formatted ContentHash
    causality_types::crypto_primitives::ContentHash::new("blake3", hash_bytes)
}

/// Generate a ContentHash from bytes
pub fn hash_bytes(bytes: &[u8]) -> causality_types::crypto_primitives::ContentHash {
    // Hash the bytes using blake3
    let hash_result = blake3::hash(bytes);
    let hash_bytes = hash_result.as_bytes().to_vec();
    
    // Create a properly formatted ContentHash
    causality_types::crypto_primitives::ContentHash::new("blake3", hash_bytes)
}

/// Generate a default ContentHash (zeros)
pub fn default_content_hash() -> causality_types::crypto_primitives::ContentHash {
    let bytes = vec![0u8; 32];
    causality_types::crypto_primitives::ContentHash::new("blake3", bytes)
}

/// Serialize an object to bytes and compute its ContentHash
pub fn hash_object<T: serde::Serialize>(obj: &T) -> Result<causality_types::crypto_primitives::ContentHash, String> {
    let bytes = serde_json::to_vec(obj)
        .map_err(|e| format!("Serialization error: {}", e))?;
    
    // Use blake3 to hash the serialized object
    let hash_result = blake3::hash(&bytes);
    let hash_bytes = hash_result.as_bytes().to_vec();
    
    Ok(causality_types::crypto_primitives::ContentHash::new("blake3", hash_bytes))
}
