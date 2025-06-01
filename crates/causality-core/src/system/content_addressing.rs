//! Content Addressing System
//!
//! This module provides the core content addressing functionality for the Causality framework.
//! All entities in the system are identified by deterministic hashes of their SSZ-serialized
//! content, ensuring global uniqueness, deduplication, and verifiable references.

use ssz::Encode;
use std::fmt;

//-----------------------------------------------------------------------------
// Core Data Structures
//-----------------------------------------------------------------------------

/// Universal content-addressed identifier.
/// 
/// All significant entities in the Causality system (resources, expressions, types,
/// handlers, transactions) are identified by the Blake3 hash of their canonical
/// SSZ serialization. This ensures:
/// 
/// - Deterministic identification (same content always produces same ID)
/// - Global uniqueness and deduplication
/// - Verifiable references and integrity checking
/// - ZK-friendly fixed-size identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId {
    /// The 32-byte hash that uniquely identifies this entity
    pub bytes: [u8; 32],
}

impl EntityId {
    /// Create an EntityId from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }
    
    /// Create an EntityId from the content hash of SSZ-serializable data
    pub fn from_content<T: Encode>(content: &T) -> Self {
        use crate::{Blake3Hasher, Hasher};
        let serialized = content.as_ssz_bytes();
        let hash = Blake3Hasher::hash(&serialized);
        Self { bytes: hash.into() }
    }
    
    /// Get the raw bytes of this EntityId
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
    
    /// Convert to a hex string for debugging/display
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }
    
    /// Create from hex string (for testing/debugging)
    pub fn from_hex(hex_str: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex_str)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Self { bytes: array })
    }
    
    /// Zero EntityId (for testing)
    pub const ZERO: EntityId = EntityId { bytes: [0u8; 32] };
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_hex()[..8]) // Show first 8 chars for readability
    }
}

impl From<[u8; 32]> for EntityId {
    fn from(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }
}

impl AsRef<[u8]> for EntityId {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl ssz::Encode for EntityId {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        32
    }

    fn ssz_bytes_len(&self) -> usize {
        32
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        crate::system::encode_fixed_bytes(&self.bytes, buf);
    }
}

impl ssz::Decode for EntityId {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        32
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        Ok(Self {
            bytes: crate::system::decode_fixed_bytes(bytes)?,
        })
    }
}

//-----------------------------------------------------------------------------
// Type Aliases
//-----------------------------------------------------------------------------

/// Content-addressed identifier for a Resource
pub type ResourceId = EntityId;

/// Content-addressed identifier for a ValueExpr
pub type ValueExprId = EntityId;

/// Content-addressed identifier for an Expr (executable expression)
pub type ExprId = EntityId;

/// Content-addressed identifier for a RowType schema
pub type RowTypeId = EntityId;

/// Content-addressed identifier for a Handler
pub type HandlerId = EntityId;

/// Content-addressed identifier for a Transaction
pub type TransactionId = EntityId;

/// Content-addressed identifier for an Intent
pub type IntentId = EntityId;

/// Content-addressed identifier for a Domain
pub type DomainId = EntityId;

/// Content-addressed identifier for a Nullifier (for preventing double-spending)
pub type NullifierId = EntityId;

//-----------------------------------------------------------------------------
// Trait Definitions
//-----------------------------------------------------------------------------

/// Trait for types that can be content-addressed
pub trait ContentAddressable {
    /// Compute the content ID for this entity
    fn content_id(&self) -> EntityId;
}

impl<T: Encode> ContentAddressable for T {
    fn content_id(&self) -> EntityId {
        EntityId::from_content(self)
    }
}

//-----------------------------------------------------------------------------
// Timestamp Implementation
//-----------------------------------------------------------------------------

/// Unix timestamp in milliseconds (u64 for ZK compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp {
    /// Milliseconds since Unix epoch
    pub millis: u64,
}

impl ssz::Encode for Timestamp {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        8
    }

    fn ssz_bytes_len(&self) -> usize {
        8
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.millis.to_le_bytes());
    }
}

impl ssz::Decode for Timestamp {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        8
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let bytes: [u8; 8] = crate::system::decode_fixed_bytes(bytes)?;
        Ok(Self { millis: u64::from_le_bytes(bytes) })
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.millis)
    }
}

impl Timestamp {
    /// Create a timestamp from Unix milliseconds
    pub fn from_millis(millis: u64) -> Self {
        Self { millis }
    }
    
    /// Get Unix milliseconds
    pub fn as_millis(&self) -> u64 {
        self.millis
    }
    
    /// Current timestamp (requires std)
    #[cfg(feature = "std")]
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self { millis: duration.as_millis() as u64 }
    }
    
    /// Zero timestamp (for testing)
    pub const ZERO: Timestamp = Timestamp { millis: 0 };
}

//-----------------------------------------------------------------------------
// SSZ-Compatible String Type
//-----------------------------------------------------------------------------

/// A string type optimized for SSZ serialization and ZK circuits.
/// For now, we use a simple String wrapper. In the future, this should be
/// replaced with a fixed-size Symbol type for better ZK compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Str {
    /// The inner string value
    pub value: String,
}

impl ssz::Encode for Str {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        4 + self.value.len() // 4 bytes for length + actual string bytes
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        crate::system::encode_with_length(self.value.as_bytes(), buf);
    }
}

impl ssz::Decode for Str {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let (string_bytes, _) = crate::system::decode_with_length(bytes)?;
        let value = String::from_utf8(string_bytes.to_vec())
            .map_err(|_| ssz::DecodeError::BytesInvalid("Invalid UTF-8".into()))?;
        Ok(Self { value })
    }
}

impl Str {
    /// Create a new Str from a string slice
    pub fn new(s: &str) -> Self {
        Self { value: s.to_string() }
    }
    
    /// Get as string slice
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl From<String> for Str {
    fn from(s: String) -> Self {
        Self { value: s }
    }
}

impl From<&str> for Str {
    fn from(s: &str) -> Self {
        Self { value: s.to_string() }
    }
}

impl std::fmt::Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl AsRef<str> for Str {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ssz::Decode;
    
    #[test]
    fn test_entity_id_deterministic() {
        let data1 = vec![1u8, 2, 3, 4];
        let data2 = vec![1u8, 2, 3, 4];
        let data3 = vec![1u8, 2, 3, 5];
        
        let id1 = EntityId::from_content(&data1);
        let id2 = EntityId::from_content(&data2);
        let id3 = EntityId::from_content(&data3);
        
        assert_eq!(id1, id2); // Same content should produce same ID
        assert_ne!(id1, id3); // Different content should produce different ID
    }
    
    #[test]
    fn test_entity_id_hex_roundtrip() {
        let original = EntityId::from_content(&vec![1u8, 2, 3, 4]);
        let hex = original.to_hex();
        let recovered = EntityId::from_hex(&hex).unwrap();
        
        assert_eq!(original, recovered);
    }
    
    #[test]
    fn test_timestamp() {
        let ts = Timestamp::from_millis(1234567890);
        assert_eq!(ts.as_millis(), 1234567890);
    }
    
    #[test]
    fn test_ssz_serialization() {
        let id = EntityId::from_content(&vec![1u8, 2, 3, 4]);
        
        // Test SSZ encoding/decoding
        let encoded = id.as_ssz_bytes();
        let decoded = EntityId::from_ssz_bytes(&encoded).unwrap();
        
        assert_eq!(id, decoded);
    }
} 