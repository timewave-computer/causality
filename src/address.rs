//! Content addressing module
//!
//! This module provides types and utilities for content addressing in the Causality system.
//! It enables content-addressable resources, allowing for integrity verification and 
//! reliable references.

use std::fmt;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use blake3::Hasher;

/// Error types for content addressing operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressError {
    /// Invalid format for content hash
    InvalidFormat,
    /// Hash algorithm not supported
    UnsupportedAlgorithm,
    /// Error during hashing operation
    HashingError,
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddressError::InvalidFormat => write!(f, "Invalid content hash format"),
            AddressError::UnsupportedAlgorithm => write!(f, "Unsupported hashing algorithm"),
            AddressError::HashingError => write!(f, "Error during hashing operation"),
        }
    }
}

impl std::error::Error for AddressError {}

/// Represents a content hash using a specific algorithm
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash {
    /// The algorithm used for hashing
    pub algorithm: String,
    /// The raw hash bytes
    pub bytes: Vec<u8>,
}

impl ContentHash {
    /// Create a new content hash
    pub fn new(algorithm: &str, bytes: Vec<u8>) -> Self {
        Self {
            algorithm: algorithm.to_string(),
            bytes,
        }
    }

    /// Create a Blake3 hash from content
    pub fn blake3(content: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(content);
        let hash = hasher.finalize();
        
        Self {
            algorithm: "blake3".to_string(),
            bytes: hash.as_bytes().to_vec(),
        }
    }

    /// Convert the hash to a hex string
    pub fn to_hex(&self) -> String {
        let mut result = String::with_capacity(self.bytes.len() * 2);
        for byte in &self.bytes {
            result.push_str(&format!("{:02x}", byte));
        }
        result
    }

    /// Get canonical string representation
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.algorithm, self.to_hex())
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl FromStr for ContentHash {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(AddressError::InvalidFormat);
        }

        let algorithm = parts[0];
        let hex_string = parts[1];

        // Check if the algorithm is supported
        match algorithm {
            "blake3" => {},
            _ => return Err(AddressError::UnsupportedAlgorithm),
        }

        // Parse the hex string
        let mut bytes = Vec::with_capacity(hex_string.len() / 2);
        for i in (0..hex_string.len()).step_by(2) {
            if i + 2 > hex_string.len() {
                return Err(AddressError::InvalidFormat);
            }
            let byte = u8::from_str_radix(&hex_string[i..i+2], 16)
                .map_err(|_| AddressError::InvalidFormat)?;
            bytes.push(byte);
        }

        Ok(Self {
            algorithm: algorithm.to_string(),
            bytes,
        })
    }
}

/// Content identifier with metadata
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentId {
    /// The content hash
    pub hash: ContentHash,
    /// Optional domain identifier
    pub domain: Option<String>,
    /// Optional content type
    pub content_type: Option<String>,
    /// Optional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl ContentId {
    /// Create a new content identifier
    pub fn new(
        hash: ContentHash,
        domain: Option<String>,
        content_type: Option<String>,
        metadata: Option<std::collections::HashMap<String, String>>,
    ) -> Self {
        Self {
            hash,
            domain,
            content_type,
            metadata: metadata.unwrap_or_default(),
        }
    }

    /// Create a content identifier from content using Blake3
    pub fn from_content(
        content: &[u8],
        domain: Option<String>,
        content_type: Option<String>,
        metadata: Option<std::collections::HashMap<String, String>>,
    ) -> Self {
        let hash = ContentHash::blake3(content);
        Self::new(hash, domain, content_type, metadata)
    }

    /// Get canonical string representation
    pub fn to_string(&self) -> String {
        let mut parts = vec![self.hash.to_string()];
        
        if let Some(domain) = &self.domain {
            parts.push(format!("domain={}", domain));
        }
        
        if let Some(content_type) = &self.content_type {
            parts.push(format!("type={}", content_type));
        }
        
        for (key, value) in &self.metadata {
            parts.push(format!("{}={}", key, value));
        }
        
        parts.join(";")
    }
}

impl fmt::Display for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl FromStr for ContentId {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(';').collect();
        if parts.is_empty() {
            return Err(AddressError::InvalidFormat);
        }

        let hash = ContentHash::from_str(parts[0])?;
        let mut domain = None;
        let mut content_type = None;
        let mut metadata = std::collections::HashMap::new();

        for i in 1..parts.len() {
            let kv: Vec<&str> = parts[i].split('=').collect();
            if kv.len() != 2 {
                continue;
            }

            let key = kv[0];
            let value = kv[1];

            match key {
                "domain" => domain = Some(value.to_string()),
                "type" => content_type = Some(value.to_string()),
                _ => { metadata.insert(key.to_string(), value.to_string()); }
            }
        }

        Ok(Self {
            hash,
            domain,
            content_type,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_blake3() {
        let content = b"test content";
        let hash = ContentHash::blake3(content);
        
        assert_eq!(hash.algorithm, "blake3");
        assert!(!hash.bytes.is_empty());
    }

    #[test]
    fn test_content_hash_to_hex() {
        let hash = ContentHash::new("blake3", vec![0, 1, 2, 3, 255]);
        assert_eq!(hash.to_hex(), "00010203ff");
    }

    #[test]
    fn test_content_hash_from_str() {
        let hash_str = "blake3:00010203ff";
        let hash = ContentHash::from_str(hash_str).unwrap();
        
        assert_eq!(hash.algorithm, "blake3");
        assert_eq!(hash.bytes, vec![0, 1, 2, 3, 255]);
    }

    #[test]
    fn test_content_hash_display() {
        let hash = ContentHash::new("blake3", vec![0, 1, 2, 3, 255]);
        assert_eq!(hash.to_string(), "blake3:00010203ff");
    }

    #[test]
    fn test_content_id_from_content() {
        let content = b"test content";
        let domain = Some("test-domain".to_string());
        let content_type = Some("text/plain".to_string());
        
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("created".to_string(), "2023-01-01".to_string());
        
        let id = ContentId::from_content(
            content,
            domain.clone(),
            content_type.clone(),
            Some(metadata.clone()),
        );
        
        assert_eq!(id.hash.algorithm, "blake3");
        assert_eq!(id.domain, domain);
        assert_eq!(id.content_type, content_type);
        assert_eq!(id.metadata.get("created"), Some(&"2023-01-01".to_string()));
    }

    #[test]
    fn test_content_id_to_string() {
        let hash = ContentHash::new("blake3", vec![0, 1, 2, 3, 255]);
        let domain = Some("test-domain".to_string());
        let content_type = Some("text/plain".to_string());
        
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("created".to_string(), "2023-01-01".to_string());
        
        let id = ContentId::new(
            hash,
            domain,
            content_type,
            Some(metadata),
        );
        
        assert_eq!(
            id.to_string(),
            "blake3:00010203ff;domain=test-domain;type=text/plain;created=2023-01-01"
        );
    }

    #[test]
    fn test_content_id_from_str() {
        let id_str = "blake3:00010203ff;domain=test-domain;type=text/plain;created=2023-01-01";
        let id = ContentId::from_str(id_str).unwrap();
        
        assert_eq!(id.hash.algorithm, "blake3");
        assert_eq!(id.hash.bytes, vec![0, 1, 2, 3, 255]);
        assert_eq!(id.domain, Some("test-domain".to_string()));
        assert_eq!(id.content_type, Some("text/plain".to_string()));
        assert_eq!(id.metadata.get("created"), Some(&"2023-01-01".to_string()));
    }
} 