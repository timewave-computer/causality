//! String type definitions for the Causality framework.
//!
//! This module provides a deterministic, fixed-size string representation
//! suitable for ZK circuit constraints and content-addressed identifiers.

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Borrow;

//-----------------------------------------------------------------------------
// String Type
//-----------------------------------------------------------------------------

/// Fixed-size byte array (64 bytes) used for string representation
///
/// This type provides a deterministic, fixed-size encoding for strings
/// that is compatible with ZK circuit constraints. It's used throughout
/// the Causality framework for consistent string handling in all contexts.
/// Size increased to 64 bytes to accommodate hex representations of 32-byte IDs.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
)]
pub struct Str(pub [u8; 64]);

impl Default for Str {
    fn default() -> Self {
        Self([0u8; 64])
    }
}

#[cfg(feature = "serde")]
impl Serialize for Str {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Str {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = <Vec<u8>>::deserialize(deserializer)?;
        if bytes.len() != 64 {
            return Err(serde::de::Error::custom(format!(
                "Expected 64 bytes, got {}",
                bytes.len()
            )));
        }
        let mut array = [0u8; 64];
        array.copy_from_slice(&bytes);
        Ok(Str(array))
    }
}

impl Str {
    /// Create a new Str from a string slice
    ///
    /// If the string is shorter than 64 bytes, it will be padded with zeros.
    /// If the string is longer than 64 bytes, it will be truncated.
    pub fn new(s: impl AsRef<str>) -> Self {
        let s = s.as_ref();
        let mut bytes = [0u8; 64];
        let copy_len = std::cmp::min(s.len(), 64);
        bytes[..copy_len].copy_from_slice(&s.as_bytes()[..copy_len]);
        Self(bytes)
    }

    /// Create a Str from a static string (equivalent to new)
    /// Used for compatibility with existing code
    pub fn from_static_str(s: &str) -> Self {
        Self::new(s)
    }

    /// Create a Str from a string (equivalent to new)
    /// Used for compatibility with existing code
    pub fn from_string(s: impl Into<String>) -> Self {
        Self::new(s.into())
    }

    /// Create a Str from raw bytes
    pub fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    /// Create a Str from a literal string
    /// Alias to from_static_str for compatibility with existing code
    pub fn from_lit(s: &str) -> Self {
        Self::from_static_str(s)
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    /// Try to convert to a UTF-8 string, stopping at the first null byte
    pub fn as_string(&self) -> String {
        let mut end = self.0.len();
        for (i, &b) in self.0.iter().enumerate() {
            if b == 0 {
                end = i;
                break;
            }
        }

        String::from_utf8_lossy(&self.0[..end]).to_string()
    }

    /// Returns a string slice (`&str`) of the valid UTF-8 portion of the Str,
    /// stopping at the first null byte or the end of the buffer.
    /// Returns an empty string if the content is not valid UTF-8.
    pub fn as_str(&self) -> &str {
        let end = self.0.iter().position(|&b| b == 0).unwrap_or(self.0.len());
        match std::str::from_utf8(&self.0[..end]) {
            Ok(s) => s,
            Err(_) => {
                // Log this ideally, as it indicates non-UTF-8 data in a Str.
                // For now, return an empty string as a fallback.
                log::warn!("Str instance contained non-UTF-8 data when as_str() was called. Content: {:?}", &self.0[..end]);
                ""
            }
        }
    }

    /// Check if the Str is empty (contains only null bytes)
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }

    /// Get the length of the non-null portion of the string
    pub fn len(&self) -> usize {
        let mut len = 0;
        for &b in self.0.iter() {
            if b == 0 {
                break;
            }
            len += 1;
        }
        len
    }
}

impl std::fmt::Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl From<&str> for Str {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for Str {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<[u8; 64]> for Str {
    fn from(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8]> for Str {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Borrow<str> for Str {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

//-----------------------------------------------------------------------------
// Utility Function
//-----------------------------------------------------------------------------

/// Convert a string to the fixed-size Str type
///
/// If the string is shorter than 64 bytes, it will be padded with zeros.
/// If the string is longer than 64 bytes, it will be truncated.
pub fn str_from_string(s: impl AsRef<str>) -> Str {
    Str::new(s)
}
