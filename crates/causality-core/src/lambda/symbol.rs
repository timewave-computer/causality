//! Symbol type
//!
//! This module provides a symbol type suitable for zero-knowledge environments,
//! using fixed-size hashes while maintaining human-readable names for development.

use std::fmt;
use ssz::{Encode, Decode, DecodeError};
use serde::{Serialize, Deserialize};

//-----------------------------------------------------------------------------
// Symbol Type
//-----------------------------------------------------------------------------

/// A symbol represents an interned identifier suitable for ZK environments.
/// 
/// Symbols are represented as fixed-size 32-byte hashes for ZK compatibility,
/// but maintain optional human-readable names for development and debugging.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
    /// Content hash of the symbol (ZK-compatible fixed size)
    pub hash: [u8; 32],
    
    /// Optional human-readable name (for development/debugging)
    /// This is not part of equality comparison or hashing
    #[cfg(feature = "std")]
    pub name: Option<String>,
}

impl Symbol {
    /// Create a new symbol from a string
    pub fn new(name: &str) -> Self {
        use crate::{Hasher, Sha256Hasher};
        let hash = Sha256Hasher::hash(name.as_bytes());
        
        Self {
            hash,
            #[cfg(feature = "std")]
            name: Some(name.to_string()),
        }
    }
    
    /// Create a symbol from raw bytes (unsafe - no validation)
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { 
            hash: bytes,
            #[cfg(feature = "std")]
            name: None,
        }
    }
    
    /// Get the hash bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.hash
    }
    
    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.hash)
    }
    
    /// Get the hash (always available, ZK-compatible)
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
    
    /// Get the human-readable name if available
    #[cfg(feature = "std")]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get the human-readable name if available
    #[cfg(not(feature = "std"))]
    pub fn name(&self) -> Option<&str> {
        None
    }

    /// Get the human-readable name if available, consuming the symbol
    #[cfg(feature = "std")]
    pub fn into_name(self) -> Option<String> {
        self.name
    }

    #[cfg(not(feature = "std"))]
    pub fn into_name(self) -> Option<String> {
        None
    }
    
    /// Get a string representation of the symbol
    pub fn as_str(&self) -> &str {
        #[cfg(feature = "std")]
        if let Some(name) = &self.name {
            return name.as_str();
        }
        
        // If no name is available, we can't return a dynamic string in a &str
        // This is a limitation - we'd need to return a String or have a static cache
        "unknown"
    }
    
    /// Convert to field element representation for ZK circuits
    /// This takes the first 31 bytes to ensure it fits in most field sizes
    pub fn to_field_bytes(&self) -> [u8; 31] {
        let mut field_bytes = [0u8; 31];
        field_bytes.copy_from_slice(&self.hash[..31]);
        field_bytes
    }
    
    /// Create symbol from field element bytes
    pub fn from_field_bytes(field_bytes: [u8; 31]) -> Self {
        let mut hash = [0u8; 32];
        hash[..31].copy_from_slice(&field_bytes);
        // Set MSB to 0 to ensure it's a valid field element
        hash[31] = 0;
        
        Self::from_bytes(hash)
    }
    
    /// Convert to a big-endian u256 representation for field arithmetic
    pub fn to_u256_be(&self) -> [u64; 4] {
        let mut words = [0u64; 4];
        for (i, chunk) in self.hash.chunks(8).enumerate() {
            if i < 4 {
                let mut bytes = [0u8; 8];
                bytes[..chunk.len()].copy_from_slice(chunk);
                words[i] = u64::from_be_bytes(bytes);
            }
        }
        words
    }
    
    /// Create symbol from u256 big-endian representation
    pub fn from_u256_be(words: [u64; 4]) -> Self {
        let mut hash = [0u8; 32];
        for (i, word) in words.iter().enumerate() {
            let bytes = word.to_be_bytes();
            hash[i * 8..(i + 1) * 8].copy_from_slice(&bytes);
        }
        Self::from_bytes(hash)
    }
    
    /// Get the content hash for this symbol
    pub fn content_hash(&self) -> [u8; 32] {
        self.hash
    }
    
    /// Create a small test symbol (for testing only)
    pub fn test_symbol(n: u8) -> Self {
        let mut hash = [0u8; 32];
        hash[0] = n;
        hash[31] = 0;
        
        Self::from_bytes(hash)
    }
    
    /// Create symbol from little-endian u64
    pub fn from_u64_le(value: u64) -> Self {
        let mut hash = [0u8; 32];
        let bytes = value.to_le_bytes();
        hash[..8].copy_from_slice(&bytes);
        Self::from_bytes(hash)
    }
    
    /// Create symbol from array of u64s (little-endian)
    pub fn from_u64_array_le(values: &[u64; 4]) -> Self {
        let mut hash = [0u8; 32];
        for (i, &value) in values.iter().enumerate() {
            let bytes = value.to_le_bytes();
            hash[i * 8..(i + 1) * 8].copy_from_slice(&bytes);
        }
        Self::from_bytes(hash)
    }
    
    /// Get symbol as little-endian u64 (truncated to first 8 bytes)
    pub fn as_u64_le(&self) -> u64 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.hash[..8]);
        u64::from_le_bytes(bytes)
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "std")]
        if let Some(name) = &self.name {
            return write!(f, "Symbol({}: {})", name, &self.to_hex()[..8]);
        }
        
        write!(f, "Symbol({})", &self.to_hex()[..8])
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "std")]
        if let Some(name) = &self.name {
            return write!(f, "{}", name);
        }
        
        write!(f, "sym_{}", &self.to_hex()[..8])
    }
}

impl From<&str> for Symbol {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

//-----------------------------------------------------------------------------
// SSZ Implementation
//-----------------------------------------------------------------------------

impl Encode for Symbol {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        32 // Only the hash is serialized
    }

    fn ssz_bytes_len(&self) -> usize {
        32
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.hash);
    }
}

impl Decode for Symbol {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        32
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() != 32 {
            return Err(DecodeError::InvalidByteLength {
                len: bytes.len(),
                expected: 32,
            });
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(bytes);
        Ok(Symbol {
            hash,
            #[cfg(feature = "std")]
            name: None, // Name is not part of SSZ representation
        })
    }
}

impl From<String> for Symbol {
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

//-----------------------------------------------------------------------------
// Common Symbols
//-----------------------------------------------------------------------------

/// Common symbols used throughout the system
pub mod common {
    use super::Symbol;
    
    /// Unit constructor symbol
    pub fn unit() -> Symbol {
        Symbol::new("unit")
    }
    
    /// True constructor symbol
    pub fn true_sym() -> Symbol {
        Symbol::new("true")
    }
    
    /// False constructor symbol
    pub fn false_sym() -> Symbol {
        Symbol::new("false")
    }
    
    /// Left injection symbol
    pub fn left() -> Symbol {
        Symbol::new("left")
    }
    
    /// Right injection symbol
    pub fn right() -> Symbol {
        Symbol::new("right")
    }
    
    /// Main function symbol
    pub fn main() -> Symbol {
        Symbol::new("main")
    }
    
    /// Apply function symbol
    pub fn apply() -> Symbol {
        Symbol::new("apply")
    }
} 