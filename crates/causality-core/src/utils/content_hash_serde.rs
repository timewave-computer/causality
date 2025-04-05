//! ContentHash serialization utilities
//!
//! This module provides serialization and deserialization
//! utilities for ContentHash values.

use serde::{Deserialize, Deserializer, Serializer};
use causality_types::ContentHash;

/// Serialize a ContentHash as a hex string
pub fn serialize<S>(hash: &ContentHash, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let hex = hash.to_hex();
    serializer.serialize_str(&hex)
}

/// Deserialize a ContentHash from a hex string
pub fn deserialize<'de, D>(deserializer: D) -> Result<ContentHash, D::Error>
where
    D: Deserializer<'de>,
{
    let hex = String::deserialize(deserializer)?;
    let bytes = hex::decode(&hex).map_err(serde::de::Error::custom)?;
    
    if bytes.len() != 32 {
        return Err(serde::de::Error::custom(format!(
            "ContentHash must be 32 bytes, got {} bytes", bytes.len()
        )));
    }
    
    Ok(ContentHash::new("blake3", bytes))
}

/// A module providing serde with_attr functions for ContentHash
pub mod with {
    use super::*;
    use serde::{Deserializer, Serializer};
    
    /// Serde wrapper for serializing ContentHash as hex string
    pub fn serialize<S>(hash: &ContentHash, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::serialize(hash, serializer)
    }
    
    /// Serde wrapper for deserializing ContentHash from hex string
    pub fn deserialize<'de, D>(deserializer: D) -> Result<ContentHash, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::deserialize(deserializer)
    }
}

/// A module that implements serialization for Option<ContentHash>
pub mod option {
    use super::*;
    
    /// Serialize an Option<ContentHash> as a hex string or null
    pub fn serialize<S>(opt_hash: &Option<ContentHash>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match opt_hash {
            Some(hash) => super::serialize(hash, serializer),
            None => serializer.serialize_none(),
        }
    }
    
    /// Deserialize an Option<ContentHash> from a hex string or null
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ContentHash>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<String>::deserialize(deserializer)?
            .map(|hex| {
                let bytes = hex::decode(&hex).map_err(serde::de::Error::custom)?;
                
                if bytes.len() != 32 {
                    return Err(serde::de::Error::custom(format!(
                        "ContentHash must be 32 bytes, got {} bytes", bytes.len()
                    )));
                }
                
                Ok(ContentHash::new("blake3", bytes))
            })
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    
    #[derive(Serialize, Deserialize)]
    struct TestStruct {
        #[serde(with = "self::with")]
        hash: ContentHash,
        
        #[serde(with = "self::option")]
        opt_hash: Option<ContentHash>,
    }
    
    #[test]
    fn test_content_hash_serialization() {
        // Create a test hash
        let bytes = [1u8; 32];
        let hash = ContentHash::new("blake3", bytes.to_vec());
        
        // Create a test struct
        let test = TestStruct {
            hash: hash.clone(),
            opt_hash: Some(hash.clone()),
        };
        
        // Serialize to JSON
        let json = serde_json::to_string(&test).unwrap();
        
        // Deserialize back
        let deserialized: TestStruct = serde_json::from_str(&json).unwrap();
        
        // Check that hashes match
        assert_eq!(test.hash, deserialized.hash);
        assert_eq!(test.opt_hash, deserialized.opt_hash);
    }
    
    #[test]
    fn test_content_hash_null_option() {
        // Create a test hash
        let bytes = [1u8; 32];
        let hash = ContentHash::new("blake3", bytes.to_vec());
        
        // Create a test struct with None
        let test = TestStruct {
            hash: hash.clone(),
            opt_hash: None,
        };
        
        // Serialize to JSON
        let json = serde_json::to_string(&test).unwrap();
        
        // Deserialize back
        let deserialized: TestStruct = serde_json::from_str(&json).unwrap();
        
        // Check that hashes match
        assert_eq!(test.hash, deserialized.hash);
        assert_eq!(test.opt_hash, deserialized.opt_hash);
        assert_eq!(deserialized.opt_hash, None);
    }
} 