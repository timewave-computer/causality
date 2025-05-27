//! Content addressing utilities
//!
//! This module provides utilities to work with content addressing,
//! particularly for converting between different content hash/ID implementations.

use anyhow::Result;
use serde_json;
use hex;
use causality_types::crypto_primitives::{ContentId, ContentHash, HashAlgorithm, HashOutput};
use blake3;

/// Trait for objects that can be content addressed
pub trait ContentAddressed {
    /// Get the content hash for this object
    fn content_hash(&self) -> Result<ContentHash, anyhow::Error>;
}

/// Generate a ContentHash from a string
pub fn hash_string(s: &str) -> ContentHash {
    // Hash the string using blake3
    let bytes = s.as_bytes();
    let hash_result = blake3::hash(bytes);
    let hash_bytes = hash_result.as_bytes().to_vec();
    
    // Create a properly formatted ContentHash
    ContentHash::new("blake3", hash_bytes)
}

/// Generate a ContentHash from bytes
pub fn hash_bytes(bytes: &[u8]) -> ContentHash {
    // Hash the bytes using blake3
    let hash_result = blake3::hash(bytes);
    let hash_bytes = hash_result.as_bytes().to_vec();
    
    // Create a properly formatted ContentHash
    ContentHash::new("blake3", hash_bytes)
}

/// Generate a default ContentHash (zeros)
pub fn default_content_hash() -> ContentHash {
    let bytes = vec![0u8; 32];
    ContentHash::new("blake3", bytes)
}

/// Serialize an object to bytes and compute its ContentHash
pub fn hash_object<T: serde::Serialize>(obj: &T) -> Result<ContentHash, anyhow::Error> {
    let bytes = serde_json::to_vec(obj)?;
    Ok(hash_bytes(&bytes))
}

/// Create a ContentId from a ContentHash
pub fn content_hash_to_id(hash: &ContentHash) -> Result<ContentId, anyhow::Error> {
    let hash_output = hash.to_hash_output()?;
    Ok(ContentId::from(hash_output))
}

/// Create a ContentHash from a ContentId
pub fn content_id_to_hash(id: &ContentId) -> Result<ContentHash, anyhow::Error> {
    let hash_output = id.hash();
    Ok(ContentHash::from_hash_output(hash_output))
}
