//! Serialization helpers for ContentHash

use serde::{Deserialize, Deserializer, Serializer, Serialize};
use causality_types::ContentHash;
use hex;

pub fn serialize<S>(hash: &ContentHash, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Format the hash as a string in the format "algorithm:hex_bytes"
    let hash_str = format!("{}:{}", hash.algorithm(), hash.value());
    hash_str.serialize(serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<ContentHash, D::Error>
where
    D: Deserializer<'de>,
{
    let hash_str = String::deserialize(deserializer)?;
    
    // First try to parse as "algorithm:hex_bytes" format
    let parts: Vec<&str> = hash_str.split(':').collect();
    if parts.len() == 2 {
        return Ok(ContentHash::new(parts[0], parts[1].to_string()));
    }
    
    // If it's just a hex string, try to parse as a Blake3 hash
    if let Ok(bytes) = hex::decode(&hash_str) {
        if !bytes.is_empty() {
            return Ok(ContentHash::new("blake3", hex::encode(&bytes)));
        }
    }
    
    Err(serde::de::Error::custom("Invalid content hash format"))
}
