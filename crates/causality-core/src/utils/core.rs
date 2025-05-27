// Purpose: Core utility functions for ID generation, content addressing, and fundamental operations.

use causality_types::primitive::ids::{
    AsId, CapabilityId, DomainId, ExprId, ResourceId, TypeExprId,
};
use causality_types::serialization::Encode;
use sha2::{Digest, Sha256};
use std::fmt::Write as FmtWrite;

//-----------------------------------------------------------------------------
// ID Generation and Manipulation Utilities
//-----------------------------------------------------------------------------

/// Creates a content-addressed ID by hashing content with SHA-256.
/// Generic over any ID type that implements `AsId`.
pub fn create_content_addressed_id<T: AsId>(content: &[u8]) -> T {
    let mut hasher = Sha256::new();
    hasher.update(content);
    T::new(hasher.finalize().into())
}

/// Converts an ID to a hexadecimal string (64 characters).
/// Generic over any ID type that implements `AsId`.
pub fn id_to_hex<T: AsId>(id: &T) -> String {
    let bytes = id.inner();
    let mut s = String::with_capacity(64);
    for &byte in bytes.iter() {
        let _ = write!(s, "{:02x}", byte);
    }
    s
}

/// Creates an ID from a hexadecimal string.
/// The hex string must be 64 characters long.
/// Generic over any ID type that implements `AsId`.
pub fn id_from_hex<T: AsId>(hex_str: &str) -> Result<T, &'static str> {
    if hex_str.len() != 64 {
        return Err("ID hex string must be 64 characters long");
    }
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        let byte_str = &hex_str[i * 2..i * 2 + 2];
        match u8::from_str_radix(byte_str, 16) {
            Ok(b) => bytes[i] = b,
            Err(_) => return Err("Invalid hexadecimal character in ID string"),
        }
    }
    Ok(T::new(bytes))
}

/// Checks if an ID (represented by its inner bytes) starts with the given prefix.
/// Generic over any ID type that implements `AsId`.
pub fn id_starts_with<T: AsId>(id: &T, prefix: &[u8]) -> bool {
    let inner_bytes = id.inner();
    if prefix.len() > inner_bytes.len() {
        return false;
    }
    inner_bytes.starts_with(prefix)
}

/// Default hash computation for any Encode type.
pub fn default_compute_hash<T: Encode + ?Sized>(data: &T) -> [u8; 32] {
    let mut hasher = Sha256::new();
    let bytes = data.as_ssz_bytes();
    hasher.update(&bytes);
    hasher.finalize().into()
}

/// Creates a random ID using the getrandom crate.
/// Generic over any ID type that implements `AsId`.
/// Requires the "getrandom" feature to be enabled for the crate.
#[cfg(feature = "getrandom")]
pub fn create_random_id<T: AsId>() -> T {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).expect("Failed to generate random ID bytes for T");
    T::new(bytes)
}

//-----------------------------------------------------------------------------
// FromStr implementations for ID types
//-----------------------------------------------------------------------------

/// Parse a ResourceId from a string using the extension trait
pub fn resource_id_from_str(s: &str) -> Result<ResourceId, anyhow::Error> {
    use crate::extension_traits::IdFromStr;
    ResourceId::from_str_ext(s)
}

/// Parse an ExprId from a string using the extension trait
pub fn expr_id_from_str(s: &str) -> Result<ExprId, anyhow::Error> {
    use crate::extension_traits::IdFromStr;
    ExprId::from_str_ext(s)
}

/// Parse a TypeExprId from a string using the extension trait
pub fn type_expr_id_from_str(s: &str) -> Result<TypeExprId, anyhow::Error> {
    use crate::extension_traits::IdFromStr;
    TypeExprId::from_str_ext(s)
}

/// Parse a DomainId from a string using the extension trait
pub fn domain_id_from_str(s: &str) -> Result<DomainId, anyhow::Error> {
    use crate::extension_traits::IdFromStr;
    DomainId::from_str_ext(s)
}

/// Parse a CapabilityId from a string using the extension trait
pub fn capability_id_from_str(s: &str) -> Result<CapabilityId, anyhow::Error> {
    use crate::extension_traits::IdFromStr;
    CapabilityId::from_str_ext(s)
}

//-----------------------------------------------------------------------------
// Common Core Utilities
//-----------------------------------------------------------------------------

/// Merges two hash arrays using XOR operation.
/// Useful for combining hashes or creating composite identifiers.
pub fn merge_hashes(hash1: &[u8; 32], hash2: &[u8; 32]) -> [u8; 32] {
    let mut result = [0u8; 32];
    for i in 0..32 {
        result[i] = hash1[i] ^ hash2[i];
    }
    result
}

/// Creates a namespace-aware hash by prefixing content with namespace.
/// Useful for creating domain-scoped or context-aware identifiers.
pub fn namespaced_hash(namespace: &str, content: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(namespace.as_bytes());
    hasher.update(b":");
    hasher.update(content);
    hasher.finalize().into()
}

/// Validates that a byte array represents a valid hash (non-zero).
/// Returns true if the hash is valid (not all zeros).
pub fn is_valid_hash(hash: &[u8; 32]) -> bool {
    hash.iter().any(|&b| b != 0)
} 