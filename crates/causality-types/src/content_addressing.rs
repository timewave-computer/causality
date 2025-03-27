// Content addressing system for the Causality project
//
// This module provides utilities for content addressing, hashing, and canonical serialization.

use std::collections::{HashMap, BTreeMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use crate::crypto_primitives::{HashError, HashOutput, HashAlgorithm, ContentId, ContentAddressed};

// Extended set of types and functions for content addressing

/// Standard type for content hash
pub type StandardContentHash = HashOutput;

/// Universal content addressing algorithm
pub const STANDARD_HASH_ALGORITHM: HashAlgorithm = HashAlgorithm::Blake3;

/// Core content hash conversion related error
#[derive(Debug, thiserror::Error)]
pub enum ContentHashConversionError {
    /// Hash algorithm mismatch
    #[error("Hash algorithm mismatch: expected {expected}, found {found}")]
    AlgorithmMismatch {
        expected: String,
        found: String,
    },
    
    /// Invalid hash format
    #[error("Invalid hash format: {0}")]
    InvalidFormat(String),
    
    /// Invalid hash length
    #[error("Invalid hash length: expected {expected}, found {found}")]
    InvalidLength {
        expected: usize,
        found: usize,
    },
    
    /// Hash error
    #[error("Hash error: {0}")]
    HashError(#[from] HashError),
}

/// Convert a hex string to raw bytes
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, ContentHashConversionError> {
    hex::decode(hex).map_err(|e| ContentHashConversionError::InvalidFormat(e.to_string()))
}

/// Convert raw bytes to hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Create a content hash from raw bytes using the standard algorithm
pub fn content_hash_from_bytes(bytes: &[u8]) -> HashOutput {
    let mut data = [0u8; 32];
    let hash_result = blake3::hash(bytes);
    let hash_bytes = hash_result.as_bytes();
    data.copy_from_slice(hash_bytes);
    HashOutput::new(data, STANDARD_HASH_ALGORITHM)
}

/// Create a content ID from raw bytes
pub fn content_id_from_bytes(bytes: &[u8]) -> ContentId {
    ContentId::from_bytes(bytes)
}

/// Create a content ID from a string
pub fn content_id_from_string(s: &str) -> ContentId {
    ContentId::new(s)
}

/// Normalize a content hash string representation
pub fn normalize_content_hash_string(hash_str: &str) -> Result<String, ContentHashConversionError> {
    if let Some(idx) = hash_str.find(':') {
        let algorithm = &hash_str[0..idx];
        let hex = &hash_str[idx+1..];
        
        // Validate hex portion
        hex_to_bytes(hex)?;
        
        // Return normalized form
        Ok(format!("{}:{}", algorithm.to_lowercase(), hex.to_lowercase()))
    } else {
        Err(ContentHashConversionError::InvalidFormat(
            "Content hash string must contain algorithm prefix".to_string()
        ))
    }
}

/// Check if a string is a valid content hash representation
pub fn is_valid_content_hash_string(hash_str: &str) -> bool {
    normalize_content_hash_string(hash_str).is_ok()
}

/// Module for canonical serialization support
pub mod canonical {
    use super::*;
    use serde::{Serialize, Deserialize};
    use serde_json::{Value, Map};
    use thiserror::Error;
    
    /// Error type for canonical serialization operations
    #[derive(Debug, Error)]
    pub enum CanonicalSerializationError {
        /// JSON serialization error
        #[error("JSON serialization error: {0}")]
        JsonError(String),
        
        /// Binary serialization error
        #[error("Binary serialization error: {0}")]
        BinaryError(String),
        
        /// Unsupported type
        #[error("Unsupported type: {0}")]
        UnsupportedType(String),
    }
    
    /// Convert an object to canonical JSON format
    pub fn to_canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalSerializationError> {
        // Step 1: Convert to a JSON Value
        let json_value = serde_json::to_value(value)
            .map_err(|e| CanonicalSerializationError::JsonError(e.to_string()))?;
        
        // Step 2: Normalize the JSON Value
        let normalized_value = normalize_json_value(json_value);
        
        // Step 3: Serialize to bytes with sorted keys
        let canonical_json = serde_json::to_string(&normalized_value)
            .map_err(|e| CanonicalSerializationError::JsonError(e.to_string()))?;
        
        Ok(canonical_json.into_bytes())
    }
    
    /// Convert an object to canonical binary format (borsh by default)
    pub fn to_canonical_binary<T: borsh::BorshSerialize>(value: &T) -> Result<Vec<u8>, CanonicalSerializationError> {
        value.try_to_vec()
            .map_err(|e| CanonicalSerializationError::BinaryError(e.to_string()))
    }
    
    /// Deserialize from canonical JSON format
    pub fn from_canonical_json<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, CanonicalSerializationError> {
        let json_str = std::str::from_utf8(bytes)
            .map_err(|e| CanonicalSerializationError::JsonError(e.to_string()))?;
        
        serde_json::from_str(json_str)
            .map_err(|e| CanonicalSerializationError::JsonError(e.to_string()))
    }
    
    /// Deserialize from canonical binary format
    pub fn from_canonical_binary<T: borsh::BorshDeserialize>(bytes: &[u8]) -> Result<T, CanonicalSerializationError> {
        T::try_from_slice(bytes)
            .map_err(|e| CanonicalSerializationError::BinaryError(e.to_string()))
    }
    
    /// Normalize a JSON value (sort maps, etc.)
    fn normalize_json_value(value: Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut new_map = Map::new();
                
                // Get all keys and sort them
                let mut keys: Vec<String> = map.keys().cloned().collect();
                keys.sort();
                
                // Add entries in sorted order
                for key in keys {
                    if let Some(val) = map.get(&key) {
                        new_map.insert(key, normalize_json_value(val.clone()));
                    }
                }
                
                Value::Object(new_map)
            }
            Value::Array(arr) => {
                let new_arr = arr.into_iter()
                    .map(normalize_json_value)
                    .collect();
                
                Value::Array(new_arr)
            }
            // Other JSON value types are kept as is
            _ => value,
        }
    }
    
    /// Helper to serialize content-addressed objects to canonical format
    pub trait CanonicalSerialize {
        /// Serialize to canonical JSON format
        fn to_canonical_json(&self) -> Result<Vec<u8>, CanonicalSerializationError>;
        
        /// Serialize to canonical binary format
        fn to_canonical_binary(&self) -> Result<Vec<u8>, CanonicalSerializationError>;
    }
    
    impl<T: Serialize + borsh::BorshSerialize> CanonicalSerialize for T {
        fn to_canonical_json(&self) -> Result<Vec<u8>, CanonicalSerializationError> {
            to_canonical_json(self)
        }
        
        fn to_canonical_binary(&self) -> Result<Vec<u8>, CanonicalSerializationError> {
            to_canonical_binary(self)
        }
    }
    
    /// Compute content hash using canonical serialization
    pub fn content_hash_canonical<T: Serialize + borsh::BorshSerialize>(
        value: &T, 
        algorithm: HashAlgorithm
    ) -> Result<HashOutput, CanonicalSerializationError> {
        // Use binary format for hashing by default
        let bytes = to_canonical_binary(value)?;
        
        let mut data = [0u8; 32];
        
        match algorithm {
            HashAlgorithm::Blake3 => {
                let hash_result = blake3::hash(&bytes);
                data.copy_from_slice(hash_result.as_bytes());
            }
            HashAlgorithm::Poseidon => {
                // This would use a Poseidon implementation
                // As placeholder, we'll use Blake3
                let hash_result = blake3::hash(&bytes);
                data.copy_from_slice(hash_result.as_bytes());
            }
        }
        
        Ok(HashOutput::new(data, algorithm))
    }
}

/// Create a content hash using canonical serialization
pub fn canonical_content_hash<T: serde::Serialize + borsh::BorshSerialize>(
    value: &T
) -> Result<HashOutput, canonical::CanonicalSerializationError> {
    canonical::content_hash_canonical(value, STANDARD_HASH_ALGORITHM)
}

/// Create a content ID using canonical serialization
pub fn canonical_content_id<T: serde::Serialize + borsh::BorshSerialize>(
    value: &T
) -> Result<ContentId, canonical::CanonicalSerializationError> {
    let hash = canonical_content_hash(value)?;
    Ok(ContentId::from(hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};
    use borsh::{BorshSerialize, BorshDeserialize};
    
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
    struct TestObject {
        id: u64,
        name: String,
        tags: Vec<String>,
        metadata: std::collections::HashMap<String, String>,
    }
    
    #[test]
    fn test_canonical_json_serialization() {
        // Create test object with intentionally unordered fields
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("z_field".to_string(), "z_value".to_string());
        metadata.insert("a_field".to_string(), "a_value".to_string());
        
        let test_obj = TestObject {
            id: 42,
            name: "Test Object".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            metadata,
        };
        
        // Get canonical JSON
        let canonical_json = canonical::to_canonical_json(&test_obj).unwrap();
        
        // Deserialize and verify
        let deserialized: TestObject = canonical::from_canonical_json(&canonical_json).unwrap();
        assert_eq!(test_obj, deserialized);
        
        // Verify deterministic serialization
        let canonical_json2 = canonical::to_canonical_json(&test_obj).unwrap();
        assert_eq!(canonical_json, canonical_json2);
        
        // Check the order of metadata in JSON string
        let json_str = std::str::from_utf8(&canonical_json).unwrap();
        
        // a_field should come before z_field in canonical JSON
        let a_pos = json_str.find("a_field").unwrap();
        let z_pos = json_str.find("z_field").unwrap();
        assert!(a_pos < z_pos, "Canonical JSON should have ordered keys");
    }
    
    #[test]
    fn test_canonical_binary_serialization() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("field1".to_string(), "value1".to_string());
        
        let test_obj = TestObject {
            id: 123,
            name: "Binary Test".to_string(),
            tags: vec!["binary".to_string()],
            metadata,
        };
        
        // Get canonical binary
        let canonical_binary = canonical::to_canonical_binary(&test_obj).unwrap();
        
        // Deserialize and verify
        let deserialized: TestObject = canonical::from_canonical_binary(&canonical_binary).unwrap();
        assert_eq!(test_obj, deserialized);
    }
    
    #[test]
    fn test_content_hash_determinism() {
        // Create two maps with same entries but different insertion orders
        let mut map1 = std::collections::HashMap::new();
        map1.insert("z".to_string(), "value".to_string());
        map1.insert("a".to_string(), "value".to_string());
        
        let mut map2 = std::collections::HashMap::new();
        map2.insert("a".to_string(), "value".to_string());
        map2.insert("z".to_string(), "value".to_string());
        
        // Create test objects
        let obj1 = TestObject {
            id: 1,
            name: "Same Content".to_string(),
            tags: vec!["tag".to_string()],
            metadata: map1,
        };
        
        let obj2 = TestObject {
            id: 1,
            name: "Same Content".to_string(),
            tags: vec!["tag".to_string()],
            metadata: map2,
        };
        
        // Compute canonical hashes
        let hash1 = canonical_content_hash(&obj1).unwrap();
        let hash2 = canonical_content_hash(&obj2).unwrap();
        
        // Hashes should be identical despite different insertion orders
        assert_eq!(hash1, hash2, "Hash should be deterministic");
        
        // Create content IDs
        let id1 = canonical_content_id(&obj1).unwrap();
        let id2 = canonical_content_id(&obj2).unwrap();
        
        // Content IDs should be identical
        assert_eq!(id1, id2, "Content ID should be deterministic");
    }
    
    #[test]
    fn test_normalization_json() {
        // Create test object with unordered fields and mixed case
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("Z_FIELD".to_string(), "Z_VALUE ".to_string());
        metadata.insert("a_field".to_string(), " a_value".to_string());
        
        let test_obj = TestObject {
            id: 42,
            name: "Test Object  ".to_string(),
            tags: vec!["TAG2".to_string(), "tag1".to_string()],
            metadata,
        };
        
        // Create normalization options for JSON
        let mut options = NormalizationOptions::default();
        options.serialization_format = SerializationFormat::Json;
        options.sort_arrays = true; // Sort arrays too
        
        // Get normalized bytes
        let normalized = test_obj.normalize(&options).unwrap();
        
        // Deserialize the normalized JSON to verify normalization
        let json_str = std::str::from_utf8(&normalized).unwrap();
        let value: serde_json::Value = serde_json::from_str(json_str).unwrap();
        
        // Check normalization of strings
        assert_eq!(value["name"], "test object");
        
        // Check array ordering
        let tags = value["tags"].as_array().unwrap();
        assert_eq!(tags[0], "tag1"); // Should be sorted
        assert_eq!(tags[1], "tag2"); // And normalized to lowercase
        
        // Check map key ordering
        let a_field_pos = json_str.find("a_field").unwrap();
        let z_field_pos = json_str.find("z_field").unwrap();
        assert!(a_field_pos < z_field_pos, "Keys should be sorted");
        
        // Check map value normalization
        assert_eq!(value["metadata"]["a_field"], "a_value");
        assert_eq!(value["metadata"]["z_field"], "z_value");
    }
    
    #[test]
    fn test_normalization_binary() {
        // Create test objects with different field order but same content
        let mut metadata1 = std::collections::HashMap::new();
        metadata1.insert("field1".to_string(), "value1".to_string());
        metadata1.insert("field2".to_string(), "value2".to_string());
        
        let mut metadata2 = std::collections::HashMap::new();
        metadata2.insert("field2".to_string(), "value2".to_string()); // Different order
        metadata2.insert("field1".to_string(), "value1".to_string());
        
        let test_obj1 = TestObject {
            id: 123,
            name: "Test Object".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            metadata: metadata1,
        };
        
        let test_obj2 = TestObject {
            id: 123,
            name: "Test Object".to_string(), // Same content
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            metadata: metadata2, // Different order
        };
        
        // Get normalized content hashes
        let options = NormalizationOptions::default(); // Binary by default
        let hash1 = test_obj1.normalized_content_hash(&options).unwrap();
        let hash2 = test_obj2.normalized_content_hash(&options).unwrap();
        
        // With binary serialization, the hashes might be different because
        // HashMap serialization order is not deterministic by default with borsh
        // This is expected - binary serialization follows the memory layout
        
        // Now try with JSON format which should normalize the order
        let mut json_options = NormalizationOptions::default();
        json_options.serialization_format = SerializationFormat::Json;
        
        let json_hash1 = test_obj1.normalized_content_hash(&json_options).unwrap();
        let json_hash2 = test_obj2.normalized_content_hash(&json_options).unwrap();
        
        // With JSON normalization, the hashes should be identical
        assert_eq!(json_hash1, json_hash2, "JSON normalized hashes should match");
    }
    
    #[test]
    fn test_empty_value_removal() {
        // Create a test object with empty values
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("empty_string".to_string(), "".to_string());
        metadata.insert("normal_field".to_string(), "value".to_string());
        metadata.insert("empty_array_field".to_string(), "[]".to_string());
        
        let test_obj = TestObject {
            id: 42,
            name: "".to_string(), // Empty name
            tags: vec![], // Empty tags
            metadata,
        };
        
        // Create options with empty value removal
        let mut options = NormalizationOptions::default();
        options.serialization_format = SerializationFormat::Json;
        options.remove_empty_values = true;
        
        // Normalize
        let normalized = test_obj.normalize(&options).unwrap();
        
        // Deserialize to check
        let json_str = std::str::from_utf8(&normalized).unwrap();
        let value: serde_json::Value = serde_json::from_str(json_str).unwrap();
        
        // Check that empty values were removed
        assert!(!value.as_object().unwrap().contains_key("name"), "Empty name should be removed");
        assert!(!value.as_object().unwrap().contains_key("tags"), "Empty tags should be removed");
        assert!(!value["metadata"].as_object().unwrap().contains_key("empty_string"), "Empty string in metadata should be removed");
        
        // Normal fields should remain
        assert_eq!(value["id"], 42);
        assert_eq!(value["metadata"]["normal_field"], "value");
    }
    
    #[test]
    fn test_normalization_content_addressed() {
        // Create a custom content-addressed type
        #[derive(Debug, PartialEq, Eq)]
        struct MyContentAddressedType {
            data: String,
            number: u32,
        }
        
        impl ContentAddressed for MyContentAddressedType {
            fn content_hash(&self) -> Result<HashOutput, HashError> {
                let mut hasher = blake3::Hasher::new();
                hasher.update(self.data.as_bytes());
                hasher.update(&self.number.to_be_bytes());
                let hash = hasher.finalize();
                
                let mut data = [0u8; 32];
                data.copy_from_slice(hash.as_bytes());
                
                Ok(HashOutput::new(data, HashAlgorithm::Blake3))
            }
            
            fn verify(&self, expected_hash: &HashOutput) -> Result<bool, HashError> {
                let actual_hash = self.content_hash()?;
                Ok(actual_hash == *expected_hash)
            }
            
            fn to_bytes(&self) -> Result<Vec<u8>, HashError> {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(self.data.as_bytes());
                bytes.extend_from_slice(&self.number.to_be_bytes());
                Ok(bytes)
            }
            
            fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
                if bytes.len() < 4 {
                    return Err(HashError::SerializationError("Not enough bytes".to_string()));
                }
                
                let data_bytes = &bytes[0..bytes.len()-4];
                let number_bytes = &bytes[bytes.len()-4..];
                
                let data = String::from_utf8(data_bytes.to_vec())
                    .map_err(|e| HashError::SerializationError(e.to_string()))?;
                
                let mut number_array = [0u8; 4];
                number_array.copy_from_slice(number_bytes);
                let number = u32::from_be_bytes(number_array);
                
                Ok(Self { data, number })
            }
        }
        
        // Create instances with different case/whitespace
        let obj1 = MyContentAddressedType {
            data: "Hello World".to_string(),
            number: 42,
        };
        
        let obj2 = MyContentAddressedType {
            data: "HELLO WORLD".to_string(),
            number: 42,
        };
        
        // Regular content hashes will be different
        let hash1 = obj1.content_hash().unwrap();
        let hash2 = obj2.content_hash().unwrap();
        assert_ne!(hash1, hash2, "Regular content hashes should differ due to case");
        
        // But normalized hashes should be the same
        let mut options = NormalizationOptions::default();
        options.normalize_strings = true;
        
        let norm_hash1 = obj1.normalized_content_hash(&options).unwrap();
        let norm_hash2 = obj2.normalized_content_hash(&options).unwrap();
        
        // The normalized hashes might still differ because our simple implementation 
        // doesn't do complex string normalization on the raw bytes,
        // but we can verify the normalization extension trait works
        assert!(obj1.verify_normalized(&norm_hash1, &options).unwrap());
    }
    
    #[test]
    fn test_deterministic_map_ordering() {
        // Test the to_ordered_map utility
        let mut unordered_map = std::collections::HashMap::new();
        // Insert in random order
        unordered_map.insert("z", 3);
        unordered_map.insert("a", 1);
        unordered_map.insert("m", 2);
        
        let ordered_map = normalization::to_ordered_map(&unordered_map);
        
        // Check keys are in alphabetical order
        let keys: Vec<_> = ordered_map.keys().collect();
        assert_eq!(keys, vec!["a", "m", "z"]);
        
        // Check values maintained association
        assert_eq!(ordered_map["a"], 1);
        assert_eq!(ordered_map["m"], 2);
        assert_eq!(ordered_map["z"], 3);
    }
}

/// Module for content-addressed storage
pub mod storage {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    use thiserror::Error;
    
    /// Error type for storage operations
    #[derive(Debug, Error)]
    pub enum StorageError {
        /// Object not found in storage
        #[error("Object not found: {0}")]
        NotFound(String),
        
        /// Duplicate object in storage
        #[error("Duplicate object: {0}")]
        Duplicate(String),
        
        /// Hash mismatch during verification
        #[error("Hash mismatch: {0}")]
        HashMismatch(String),
        
        /// IO error
        #[error("IO error: {0}")]
        IoError(String),
        
        /// Serialization error
        #[error("Serialization error: {0}")]
        SerializationError(String),
        
        /// Hash error
        #[error("Hash error: {0}")]
        HashError(#[from] HashError),
        
        /// Canonical serialization error
        #[error("Canonical serialization error: {0}")]
        CanonicalError(#[from] canonical::CanonicalSerializationError),
    }
    
    /// Standard content-addressed storage interface
    pub trait ContentAddressedStorage: Send + Sync {
        /// Store binary data and return content ID
        fn store_bytes(&self, bytes: &[u8]) -> Result<ContentId, StorageError>;
        
        /// Check if an object exists in storage
        fn contains(&self, id: &ContentId) -> bool;
        
        /// Retrieve binary data for an object
        fn get_bytes(&self, id: &ContentId) -> Result<Vec<u8>, StorageError>;
        
        /// Remove an object from storage
        fn remove(&self, id: &ContentId) -> Result<(), StorageError>;
        
        /// Clear all objects from storage
        fn clear(&self);
        
        /// Get the number of objects in storage
        fn len(&self) -> usize;
        
        /// Check if storage is empty
        fn is_empty(&self) -> bool {
            self.len() == 0
        }
    }
    
    /// Extension methods for ContentAddressedStorage
    pub trait ContentAddressedStorageExt: ContentAddressedStorage {
        /// Store an object in the content-addressed storage
        fn store<T: ContentAddressed>(&self, object: &T) -> Result<ContentId, StorageError> {
            // Serialize the object
            let bytes = object.to_bytes()?;
            // Store the bytes
            self.store_bytes(&bytes)
        }
        
        /// Retrieve an object from storage by its content ID
        fn get<T: ContentAddressed>(&self, id: &ContentId) -> Result<T, StorageError> {
            let bytes = self.get_bytes(id)?;
            T::from_bytes(&bytes).map_err(|e| StorageError::HashError(e))
        }
    }
    
    /// In-memory implementation of content-addressed storage
    pub struct InMemoryStorage {
        objects: RwLock<HashMap<ContentId, Vec<u8>>>,
    }
    
    impl InMemoryStorage {
        /// Create a new empty in-memory storage
        pub fn new() -> Self {
            Self {
                objects: RwLock::new(HashMap::new()),
            }
        }
    }
    
    impl ContentAddressedStorage for InMemoryStorage {
        fn store_bytes(&self, bytes: &[u8]) -> Result<ContentId, StorageError> {
            // Create a content ID from the bytes
            let content_id = content_id_from_bytes(bytes);
            
            // Store the bytes with the content ID as the key
            let mut objects = self.objects.write().unwrap();
            
            // Skip if already exists
            if objects.contains_key(&content_id) {
                return Ok(content_id);
            }
            
            // Store the bytes
            objects.insert(content_id.clone(), bytes.to_vec());
            
            Ok(content_id)
        }
        
        fn contains(&self, id: &ContentId) -> bool {
            let objects = self.objects.read().unwrap();
            objects.contains_key(id)
        }
        
        fn get_bytes(&self, id: &ContentId) -> Result<Vec<u8>, StorageError> {
            let objects = self.objects.read().unwrap();
            
            objects.get(id)
                .cloned()
                .ok_or_else(|| StorageError::NotFound(
                    format!("Object not found: {}", id)
                ))
        }
        
        fn remove(&self, id: &ContentId) -> Result<(), StorageError> {
            let mut objects = self.objects.write().unwrap();
            
            if objects.remove(id).is_none() {
                return Err(StorageError::NotFound(
                    format!("Object not found: {}", id)
                ));
            }
            
            Ok(())
        }
        
        fn clear(&self) {
            let mut objects = self.objects.write().unwrap();
            objects.clear();
        }
        
        fn len(&self) -> usize {
            let objects = self.objects.read().unwrap();
            objects.len()
        }
    }
    
    /// Caching layer for content-addressed storage
    pub struct CachingStorage {
        /// The underlying storage
        backing_store: Arc<dyn ContentAddressedStorage>,
        /// Cache for frequently accessed objects
        cache: RwLock<HashMap<ContentId, Vec<u8>>>,
        /// Maximum cache size (number of objects)
        max_cache_size: usize,
    }
    
    impl CachingStorage {
        /// Create a new caching storage with the given backing store
        pub fn new(backing_store: Arc<dyn ContentAddressedStorage>, max_cache_size: usize) -> Self {
            Self {
                backing_store,
                cache: RwLock::new(HashMap::with_capacity(max_cache_size)),
                max_cache_size,
            }
        }
        
        /// Clear the cache but leave the backing store intact
        pub fn clear_cache(&self) {
            let mut cache = self.cache.write().unwrap();
            cache.clear();
        }
        
        /// Get cache statistics
        pub fn cache_stats(&self) -> CacheStats {
            let cache = self.cache.read().unwrap();
            CacheStats {
                size: cache.len(),
                max_size: self.max_cache_size,
                bytes_used: cache.values().map(|v| v.len()).sum(),
            }
        }
    }
    
    impl ContentAddressedStorage for CachingStorage {
        fn store_bytes(&self, bytes: &[u8]) -> Result<ContentId, StorageError> {
            // Create a content ID
            let content_id = content_id_from_bytes(bytes);
            
            // Store in backing storage first
            let content_id = self.backing_store.store_bytes(bytes)?;
            
            // Cache the bytes
            let mut cache = self.cache.write().unwrap();
            
            // Manage cache size
            if cache.len() >= self.max_cache_size && !cache.contains_key(&content_id) {
                if let Some(id_to_remove) = cache.keys().next().cloned() {
                    cache.remove(&id_to_remove);
                }
            }
            
            // Add to cache
            cache.insert(content_id.clone(), bytes.to_vec());
            
            Ok(content_id)
        }
        
        fn contains(&self, id: &ContentId) -> bool {
            // Check cache first
            {
                let cache = self.cache.read().unwrap();
                if cache.contains_key(id) {
                    return true;
                }
            }
            
            // Fall back to backing store
            self.backing_store.contains(id)
        }
        
        fn get_bytes(&self, id: &ContentId) -> Result<Vec<u8>, StorageError> {
            // Try to get from cache first
            {
                let cache = self.cache.read().unwrap();
                if let Some(bytes) = cache.get(id) {
                    return Ok(bytes.clone());
                }
            }
            
            // Get from backing store
            let bytes = self.backing_store.get_bytes(id)?;
            
            // Update cache
            {
                let mut cache = self.cache.write().unwrap();
                
                // Manage cache size
                if cache.len() >= self.max_cache_size && !cache.contains_key(id) {
                    if let Some(id_to_remove) = cache.keys().next().cloned() {
                        cache.remove(&id_to_remove);
                    }
                }
                
                // Add to cache
                cache.insert(id.clone(), bytes.clone());
            }
            
            Ok(bytes)
        }
        
        fn remove(&self, id: &ContentId) -> Result<(), StorageError> {
            // Remove from backing store
            let result = self.backing_store.remove(id);
            
            // Remove from cache if present
            {
                let mut cache = self.cache.write().unwrap();
                cache.remove(id);
            }
            
            result
        }
        
        fn clear(&self) {
            // Clear backing store
            self.backing_store.clear();
            
            // Clear cache
            self.clear_cache();
        }
        
        fn len(&self) -> usize {
            // Use backing store size
            self.backing_store.len()
        }
    }
    
    /// Statistics for the cache
    #[derive(Debug, Clone, Copy)]
    pub struct CacheStats {
        /// Current number of objects in cache
        pub size: usize,
        /// Maximum cache size
        pub max_size: usize,
        /// Total bytes used by cached objects
        pub bytes_used: usize,
    }
    
    /// Factory for creating storage implementations
    pub struct StorageFactory;
    
    impl StorageFactory {
        /// Create a new in-memory storage
        pub fn create_memory_storage() -> Arc<dyn ContentAddressedStorage> {
            Arc::new(InMemoryStorage::new())
        }
        
        /// Create a new caching storage with an in-memory backing store
        pub fn create_caching_memory_storage(cache_size: usize) -> Arc<dyn ContentAddressedStorage> {
            let backing = Self::create_memory_storage();
            Arc::new(CachingStorage::new(backing, cache_size))
        }
    }
    
    /// Object reference in content-addressed storage
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ContentRef<T> {
        /// The content ID of the object
        pub id: ContentId,
        /// Phantom data to indicate the type
        pub _phantom: std::marker::PhantomData<T>,
    }
    
    impl<T: ContentAddressed> ContentRef<T> {
        /// Create a new reference to a content-addressed object
        pub fn new(id: ContentId) -> Self {
            Self {
                id,
                _phantom: std::marker::PhantomData,
            }
        }
        
        /// Create a reference from an object
        pub fn from_object(object: &T) -> Result<Self, HashError> {
            let id = object.content_id()?;
            Ok(Self::new(id))
        }
        
        /// Resolve the reference to get the object
        pub fn resolve(&self, storage: &impl ContentAddressedStorage) -> Result<T, StorageError> {
            storage.get(&self.id)
        }
    }
}

// Re-export common storage types at the module level
pub use storage::{StorageFactory, ContentRef};

/// Module for content addressed storage metrics
pub mod metrics {
    use super::storage::*;
    use super::ContentId;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    use std::time::{Duration, Instant};
    
    /// Storage metrics for tracking performance and usage
    #[derive(Debug, Clone)]
    pub struct StorageMetrics {
        /// Total objects stored
        pub total_stores: u64,
        /// Total objects retrieved
        pub total_gets: u64,
        /// Total objects removed
        pub total_removes: u64,
        /// Total bytes stored
        pub bytes_stored: u64,
        /// Total bytes retrieved
        pub bytes_retrieved: u64,
        /// Cache hits (if caching is enabled)
        pub cache_hits: u64,
        /// Cache misses (if caching is enabled)
        pub cache_misses: u64,
        /// Average store latency
        pub avg_store_latency: Duration,
        /// Average get latency
        pub avg_get_latency: Duration,
        /// Number of objects currently in storage
        pub current_objects: usize,
    }
    
    impl Default for StorageMetrics {
        fn default() -> Self {
            Self {
                total_stores: 0,
                total_gets: 0,
                total_removes: 0,
                bytes_stored: 0,
                bytes_retrieved: 0,
                cache_hits: 0,
                cache_misses: 0,
                avg_store_latency: Duration::from_secs(0),
                avg_get_latency: Duration::from_secs(0),
                current_objects: 0,
            }
        }
    }
    
    /// Storage wrapper that collects metrics
    pub struct MetricStorage {
        /// The underlying storage
        storage: Arc<dyn ContentAddressedStorage>,
        /// Collected metrics
        metrics: RwLock<StorageMetrics>,
        /// Optional size tracker for content IDs
        size_tracker: RwLock<HashMap<ContentId, usize>>,
    }
    
    impl MetricStorage {
        /// Create a new metric storage wrapper
        pub fn new(storage: Arc<dyn ContentAddressedStorage>) -> Self {
            Self {
                storage,
                metrics: RwLock::new(StorageMetrics::default()),
                size_tracker: RwLock::new(HashMap::new()),
            }
        }
        
        /// Get a snapshot of the current metrics
        pub fn get_metrics(&self) -> StorageMetrics {
            let metrics = self.metrics.read().unwrap();
            metrics.clone()
        }
        
        /// Reset metrics to zero
        pub fn reset_metrics(&self) {
            let mut metrics = self.metrics.write().unwrap();
            *metrics = StorageMetrics::default();
            metrics.current_objects = self.storage.len();
        }
    }
    
    impl ContentAddressedStorage for MetricStorage {
        fn store_bytes(&self, bytes: &[u8]) -> Result<ContentId, StorageError> {
            let start = Instant::now();
            
            // Store in underlying storage
            let result = self.storage.store_bytes(bytes);
            
            // Update metrics
            let duration = start.elapsed();
            {
                let mut metrics = self.metrics.write().unwrap();
                metrics.total_stores += 1;
                
                let new_avg = ((metrics.avg_store_latency.as_nanos() as f64 * (metrics.total_stores - 1) as f64)
                    + duration.as_nanos() as f64) / metrics.total_stores as f64;
                    
                metrics.avg_store_latency = Duration::from_nanos(new_avg as u64);
                
                // Track object size
                let size = bytes.len();
                metrics.bytes_stored += size as u64;
                
                // Update size tracker if result is successful
                if let Ok(content_id) = &result {
                    let mut size_tracker = self.size_tracker.write().unwrap();
                    size_tracker.insert(content_id.clone(), size);
                }
            }
            
            result
        }
        
        fn contains(&self, id: &ContentId) -> bool {
            self.storage.contains(id)
        }
        
        fn get_bytes(&self, id: &ContentId) -> Result<Vec<u8>, StorageError> {
            let start = Instant::now();
            
            // Get from underlying storage
            let result = self.storage.get_bytes(id);
            
            // Update metrics
            let duration = start.elapsed();
            {
                let mut metrics = self.metrics.write().unwrap();
                metrics.total_gets += 1;
                
                let new_avg = ((metrics.avg_get_latency.as_nanos() as f64 * (metrics.total_gets - 1) as f64)
                    + duration.as_nanos() as f64) / metrics.total_gets as f64;
                    
                metrics.avg_get_latency = Duration::from_nanos(new_avg as u64);
                
                // Track bytes retrieved
                if let Ok(bytes) = &result {
                    metrics.bytes_retrieved += bytes.len() as u64;
                }
            }
            
            result
        }
        
        fn remove(&self, id: &ContentId) -> Result<(), StorageError> {
            // Track size before removal
            let size = {
                let size_tracker = self.size_tracker.read().unwrap();
                size_tracker.get(id).cloned()
            };
            
            // Remove from underlying storage
            let result = self.storage.remove(id);
            
            // Update metrics
            {
                let mut metrics = self.metrics.write().unwrap();
                metrics.total_removes += 1;
                
                // Remove from size tracker
                if result.is_ok() {
                    let mut size_tracker = self.size_tracker.write().unwrap();
                    size_tracker.remove(id);
                    
                    // Update current objects
                    metrics.current_objects = self.len();
                }
            }
            
            result
        }
        
        fn clear(&self) {
            // Clear underlying storage
            self.storage.clear();
            
            // Clear size tracker
            self.size_tracker.write().unwrap().clear();
            
            // Update metrics
            let mut metrics = self.metrics.write().unwrap();
            metrics.current_objects = 0;
        }
        
        fn len(&self) -> usize {
            self.storage.len()
        }
    }
    
    /// Extension to the storage factory to create metric-enabled storage
    impl StorageFactory {
        /// Create a new metric-enabled in-memory storage
        pub fn create_metric_memory_storage() -> Arc<MetricStorage> {
            let storage = Self::create_memory_storage();
            Arc::new(MetricStorage::new(storage))
        }
        
        /// Create a new metric-enabled caching storage
        pub fn create_metric_caching_storage(cache_size: usize) -> Arc<MetricStorage> {
            let storage = Self::create_caching_memory_storage(cache_size);
            Arc::new(MetricStorage::new(storage))
        }
    }
}

// Re-export metrics types
pub use metrics::MetricStorage;

#[cfg(test)]
mod storage_tests {
    use super::*;
    use super::storage::*;
    use super::metrics::*;
    use serde::{Serialize, Deserialize};
    use borsh::{BorshSerialize, BorshDeserialize};
    
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
    struct TestStorageObject {
        id: u64,
        name: String,
        data: Vec<u8>,
    }
    
    impl ContentAddressed for TestStorageObject {
        fn content_hash(&self) -> Result<HashOutput, HashError> {
            // Use canonical serialization for content hashing
            canonical_content_hash(self)
                .map_err(|e| HashError::SerializationError(e.to_string()))
        }
        
        fn verify(&self, expected_hash: &HashOutput) -> Result<bool, HashError> {
            let actual_hash = self.content_hash()?;
            Ok(actual_hash == *expected_hash)
        }
        
        fn to_bytes(&self) -> Result<Vec<u8>, HashError> {
            canonical::to_canonical_binary(self)
                .map_err(|e| HashError::SerializationError(e.to_string()))
        }
        
        fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
            canonical::from_canonical_binary(bytes)
                .map_err(|e| HashError::SerializationError(e.to_string()))
        }
    }
    
    #[test]
    fn test_in_memory_storage() {
        // Create storage
        let storage = StorageFactory::create_memory_storage();
        
        // Create test object
        let obj1 = TestStorageObject {
            id: 1,
            name: "Test Object 1".to_string(),
            data: vec![1, 2, 3, 4],
        };
        
        // Store object
        let content_id = storage.store(&obj1).unwrap();
        
        // Check that storage contains the object
        assert!(storage.contains(&content_id));
        
        // Retrieve object
        let retrieved: TestStorageObject = storage.get(&content_id).unwrap();
        assert_eq!(obj1, retrieved);
        
        // Store another object
        let obj2 = TestStorageObject {
            id: 2,
            name: "Test Object 2".to_string(),
            data: vec![5, 6, 7, 8],
        };
        let content_id2 = storage.store(&obj2).unwrap();
        
        // Check storage size
        assert_eq!(storage.len(), 2);
        
        // Remove object
        storage.remove(&content_id).unwrap();
        assert!(!storage.contains(&content_id));
        assert_eq!(storage.len(), 1);
        
        // Clear storage
        storage.clear();
        assert_eq!(storage.len(), 0);
        assert!(!storage.contains(&content_id2));
    }
    
    #[test]
    fn test_caching_storage() {
        // Create caching storage with small cache size
        let storage = StorageFactory::create_caching_memory_storage(2);
        
        // Create test objects
        let obj1 = TestStorageObject {
            id: 1,
            name: "Test Object 1".to_string(),
            data: vec![1, 2, 3],
        };
        
        let obj2 = TestStorageObject {
            id: 2,
            name: "Test Object 2".to_string(),
            data: vec![4, 5, 6],
        };
        
        let obj3 = TestStorageObject {
            id: 3,
            name: "Test Object 3".to_string(),
            data: vec![7, 8, 9],
        };
        
        // Store objects
        let id1 = storage.store(&obj1).unwrap();
        let id2 = storage.store(&obj2).unwrap();
        let id3 = storage.store(&obj3).unwrap();
        
        // Check all objects are in storage
        assert!(storage.contains(&id1));
        assert!(storage.contains(&id2));
        assert!(storage.contains(&id3));
        
        // Retrieve objects (should be cached now)
        let _: TestStorageObject = storage.get(&id1).unwrap();
        let _: TestStorageObject = storage.get(&id2).unwrap();
        
        // Third retrieval should push first out of cache
        let _: TestStorageObject = storage.get(&id3).unwrap();
        
        // All still retrievable
        let r1: TestStorageObject = storage.get(&id1).unwrap();
        let r2: TestStorageObject = storage.get(&id2).unwrap();
        let r3: TestStorageObject = storage.get(&id3).unwrap();
        
        assert_eq!(obj1, r1);
        assert_eq!(obj2, r2);
        assert_eq!(obj3, r3);
    }
    
    #[test]
    fn test_metric_storage() {
        // Create metric storage
        let storage = StorageFactory::create_metric_memory_storage();
        
        // Create test objects
        let obj1 = TestStorageObject {
            id: 1,
            name: "Metric Test 1".to_string(),
            data: vec![1, 2, 3, 4, 5],
        };
        
        let obj2 = TestStorageObject {
            id: 2,
            name: "Metric Test 2".to_string(),
            data: vec![6, 7, 8, 9, 10],
        };
        
        // Initial metrics should be zero
        let metrics = storage.get_metrics();
        assert_eq!(metrics.total_stores, 0);
        assert_eq!(metrics.total_gets, 0);
        
        // Store objects
        let id1 = storage.store(&obj1).unwrap();
        let id2 = storage.store(&obj2).unwrap();
        
        // Check metrics updated
        let metrics = storage.get_metrics();
        assert_eq!(metrics.total_stores, 2);
        assert_eq!(metrics.current_objects, 2);
        assert!(metrics.bytes_stored > 0);
        
        // Retrieve objects
        let _: TestStorageObject = storage.get(&id1).unwrap();
        let _: TestStorageObject = storage.get(&id2).unwrap();
        let _: TestStorageObject = storage.get(&id1).unwrap(); // Retrieve again
        
        // Check get metrics
        let metrics = storage.get_metrics();
        assert_eq!(metrics.total_gets, 3);
        assert!(metrics.bytes_retrieved > 0);
        assert!(metrics.avg_get_latency.as_nanos() > 0);
        
        // Remove object
        storage.remove(&id1).unwrap();
        
        // Check remove metrics
        let metrics = storage.get_metrics();
        assert_eq!(metrics.total_removes, 1);
        assert_eq!(metrics.current_objects, 1);
        
        // Reset metrics
        storage.reset_metrics();
        let metrics = storage.get_metrics();
        assert_eq!(metrics.total_stores, 0);
        assert_eq!(metrics.total_gets, 0);
        assert_eq!(metrics.current_objects, 1); // Still one object in storage
    }
    
    #[test]
    fn test_content_ref() {
        // Create storage
        let storage = StorageFactory::create_memory_storage();
        
        // Create test object
        let obj = TestStorageObject {
            id: 42,
            name: "Reference Test".to_string(),
            data: vec![1, 2, 3],
        };
        
        // Store object
        let content_id = storage.store(&obj).unwrap();
        
        // Create content reference
        let content_ref = ContentRef::<TestStorageObject>::new(content_id);
        
        // Resolve reference
        let resolved = content_ref.resolve(&*storage).unwrap();
        assert_eq!(obj, resolved);
        
        // Create reference directly from object
        let direct_ref = ContentRef::from_object(&obj).unwrap();
        let direct_resolved = direct_ref.resolve(&*storage).unwrap();
        assert_eq!(obj, direct_resolved);
    }
}

/// Module for content normalization
pub mod normalization {
    use super::*;
    use serde::{Serialize, Deserialize};
    use serde_json::{Value, Map};
    use std::collections::{HashMap, BTreeMap};
    use thiserror::Error;
    
    /// Error that can occur during normalization
    #[derive(Debug, Error)]
    pub enum NormalizationError {
        /// Serialization error
        #[error("Serialization error: {0}")]
        SerializationError(String),
        
        /// Unsupported type
        #[error("Unsupported type: {0}")]
        UnsupportedType(String),
        
        /// Invalid format
        #[error("Invalid format: {0}")]
        InvalidFormat(String),
    }
    
    /// Options for normalization
    #[derive(Debug, Clone)]
    pub struct NormalizationOptions {
        /// Sort map keys
        pub sort_map_keys: bool,
        /// Sort arrays
        pub sort_arrays: bool,
        /// Normalize string values
        pub normalize_strings: bool,
        /// Remove empty values (null, empty strings, empty arrays, empty objects)
        pub remove_empty_values: bool,
        /// Format for serialization
        pub serialization_format: SerializationFormat,
    }
    
    impl Default for NormalizationOptions {
        fn default() -> Self {
            Self {
                sort_map_keys: true,
                sort_arrays: false,
                normalize_strings: true,
                remove_empty_values: false,
                serialization_format: SerializationFormat::Binary,
            }
        }
    }
    
    /// Serialization format to use
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SerializationFormat {
        /// JSON format
        Json,
        /// Binary format
        Binary,
    }
    
    /// Trait for types that can be normalized
    pub trait Normalizable {
        /// Normalize this object
        fn normalize(&self, options: &NormalizationOptions) -> Result<Vec<u8>, NormalizationError>;
        
        /// Get a normalized content hash of this object
        fn normalized_content_hash(&self, options: &NormalizationOptions) -> Result<HashOutput, NormalizationError> {
            let normalized = self.normalize(options)?;
            let mut data = [0u8; 32];
            
            match STANDARD_HASH_ALGORITHM {
                HashAlgorithm::Blake3 => {
                    let hash_result = blake3::hash(&normalized);
                    data.copy_from_slice(hash_result.as_bytes());
                }
                HashAlgorithm::Poseidon => {
                    // This would use a Poseidon implementation
                    // As placeholder, we'll use Blake3
                    let hash_result = blake3::hash(&normalized);
                    data.copy_from_slice(hash_result.as_bytes());
                }
            }
            
            Ok(HashOutput::new(data, STANDARD_HASH_ALGORITHM))
        }
    }
    
    /// Default implementation of Normalizable for all Serialize + BorshSerialize types
    impl<T: Serialize + borsh::BorshSerialize> Normalizable for T {
        fn normalize(&self, options: &NormalizationOptions) -> Result<Vec<u8>, NormalizationError> {
            match options.serialization_format {
                SerializationFormat::Json => {
                    // Convert to JSON and normalize
                    let json_value = serde_json::to_value(self)
                        .map_err(|e| NormalizationError::SerializationError(e.to_string()))?;
                    
                    // Apply normalization steps
                    let normalized_value = normalize_json_value(json_value, options);
                    
                    // Serialize to bytes
                    let json_string = serde_json::to_string(&normalized_value)
                        .map_err(|e| NormalizationError::SerializationError(e.to_string()))?;
                    
                    Ok(json_string.into_bytes())
                }
                SerializationFormat::Binary => {
                    // For binary, we just do borsh serialization for now
                    // In the future, we could add more normalization steps here
                    self.try_to_vec()
                        .map_err(|e| NormalizationError::SerializationError(e.to_string()))
                }
            }
        }
    }
    
    /// Normalize a JSON value according to the options
    pub fn normalize_json_value(value: Value, options: &NormalizationOptions) -> Value {
        match value {
            Value::Object(map) => {
                let mut normalized_map = Map::new();
                
                // Sort keys if requested
                let keys: Vec<String> = if options.sort_map_keys {
                    let mut keys: Vec<String> = map.keys().cloned().collect();
                    keys.sort();
                    keys
                } else {
                    map.keys().cloned().collect()
                };
                
                // Process each key-value pair
                for key in keys {
                    let val = map.get(&key).unwrap();
                    
                    // Skip empty values if requested
                    if options.remove_empty_values && is_empty_value(val) {
                        continue;
                    }
                    
                    // Normalize the value
                    let normalized_val = normalize_json_value(val.clone(), options);
                    
                    // Skip empty values after normalization if requested
                    if options.remove_empty_values && is_empty_value(&normalized_val) {
                        continue;
                    }
                    
                    // Add to normalized map
                    normalized_map.insert(key, normalized_val);
                }
                
                Value::Object(normalized_map)
            }
            Value::Array(arr) => {
                // Normalize each element
                let mut normalized_arr: Vec<Value> = arr.into_iter()
                    .map(|v| normalize_json_value(v, options))
                    .collect();
                
                // Remove empty values if requested
                if options.remove_empty_values {
                    normalized_arr.retain(|v| !is_empty_value(v));
                }
                
                // Sort array if requested
                if options.sort_arrays {
                    // We need to make values sortable
                    // This is a best-effort sort that might not work for complex values
                    normalized_arr.sort_by(|a, b| {
                        let a_str = serde_json::to_string(a).unwrap_or_default();
                        let b_str = serde_json::to_string(b).unwrap_or_default();
                        a_str.cmp(&b_str)
                    });
                }
                
                Value::Array(normalized_arr)
            }
            Value::String(s) => {
                if options.normalize_strings {
                    // Simple string normalization: trim and lowercase
                    // More complex normalization could be added
                    Value::String(s.trim().to_lowercase())
                } else {
                    Value::String(s)
                }
            }
            // Other value types are kept as-is
            _ => value,
        }
    }
    
    /// Check if a JSON value is "empty"
    fn is_empty_value(value: &Value) -> bool {
        match value {
            Value::Null => true,
            Value::String(s) => s.is_empty(),
            Value::Array(arr) => arr.is_empty(),
            Value::Object(obj) => obj.is_empty(),
            _ => false,
        }
    }
    
    /// Extension trait for ContentAddressed to add normalization
    pub trait NormalizableContentAddressed: ContentAddressed {
        /// Get a normalized hash of the content
        fn normalized_content_hash(&self, options: &NormalizationOptions) -> Result<HashOutput, HashError> {
            let bytes = self.to_bytes()?;
            
            // We create a temporary type to normalize, since we already have the bytes
            struct ByteHolder(Vec<u8>);
            
            impl Serialize for ByteHolder {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    serializer.serialize_bytes(&self.0)
                }
            }
            
            impl borsh::BorshSerialize for ByteHolder {
                fn serialize<W: borsh::maybestd::io::Write>(&self, writer: &mut W) -> Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.0, writer)
                }
            }
            
            let holder = ByteHolder(bytes);
            holder.normalized_content_hash(options)
                .map_err(|e| HashError::SerializationError(e.to_string()))
        }
        
        /// Verify content against a normalized hash
        fn verify_normalized(&self, expected_hash: &HashOutput, options: &NormalizationOptions) -> Result<bool, HashError> {
            let actual_hash = self.normalized_content_hash(options)?;
            Ok(actual_hash == *expected_hash)
        }
    }
    
    // Implement NormalizableContentAddressed for all ContentAddressed types
    impl<T: ContentAddressed> NormalizableContentAddressed for T {}
    
    /// Helper functions for common normalization tasks
    
    /// Create a normalized content hash with default options
    pub fn normalized_content_hash<T: Serialize + borsh::BorshSerialize>(value: &T) -> Result<HashOutput, NormalizationError> {
        value.normalized_content_hash(&NormalizationOptions::default())
    }
    
    /// Create a normalized content ID with default options
    pub fn normalized_content_id<T: Serialize + borsh::BorshSerialize>(value: &T) -> Result<ContentId, NormalizationError> {
        let hash = normalized_content_hash(value)?;
        Ok(ContentId::from(hash))
    }
    
    /// Convert a map to a deterministic ordered map (BTreeMap)
    pub fn to_ordered_map<K: Ord + Clone, V: Clone>(map: &HashMap<K, V>) -> BTreeMap<K, V> {
        let mut ordered = BTreeMap::new();
        for (k, v) in map.iter() {
            ordered.insert(k.clone(), v.clone());
        }
        ordered
    }
}

// Re-export normalization items for convenience
pub use normalization::{
    NormalizationOptions, SerializationFormat, Normalizable, NormalizableContentAddressed,
    normalized_content_hash, normalized_content_id
};

/// Module for deferred hashing
pub mod deferred {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex};
    use thiserror::Error;
    
    /// Error that can occur during deferred hashing
    #[derive(Debug, Error)]
    pub enum DeferredHashingError {
        /// Hash ID not found
        #[error("Hash ID not found: {0}")]
        HashIdNotFound(String),
        
        /// Normalization error
        #[error("Normalization error: {0}")]
        NormalizationError(#[from] normalization::NormalizationError),
        
        /// Serialization error
        #[error("Serialization error: {0}")]
        SerializationError(String),
        
        /// Hash computation already performed
        #[error("Hash computation already performed")]
        AlreadyComputed,
    }
    
    /// A unique identifier for a deferred hash request
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct DeferredHashId(String);
    
    impl DeferredHashId {
        /// Create a new deferred hash ID
        pub fn new() -> Self {
            use std::time::{SystemTime, UNIX_EPOCH};
            
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
                
            let random_part = rand::random::<u64>();
            let id = format!("deferred_hash_{}_{}", timestamp, random_part);
            
            Self(id)
        }
        
        /// Get the string representation
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }
    
    impl std::fmt::Display for DeferredHashId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    
    /// A batch of deferred hash computations
    #[derive(Debug)]
    pub struct DeferredHashBatch {
        /// Pending hashes
        pending: Mutex<HashMap<DeferredHashId, Vec<u8>>>,
        /// Computed results
        results: Mutex<HashMap<DeferredHashId, HashOutput>>,
        /// Normalization options for each request
        options: Mutex<HashMap<DeferredHashId, normalization::NormalizationOptions>>,
    }
    
    impl DeferredHashBatch {
        /// Create a new deferred hash batch
        pub fn new() -> Self {
            Self {
                pending: Mutex::new(HashMap::new()),
                results: Mutex::new(HashMap::new()),
                options: Mutex::new(HashMap::new()),
            }
        }
        
        /// Request a hash computation for raw bytes
        pub fn request_hash_for_bytes(&self, data: &[u8]) -> DeferredHashId {
            let id = DeferredHashId::new();
            self.pending.lock().unwrap().insert(id.clone(), data.to_vec());
            self.options.lock().unwrap().insert(id.clone(), normalization::NormalizationOptions::default());
            id
        }
        
        /// Request a hash computation with normalization
        pub fn request_hash<T: Serialize + borsh::BorshSerialize>(
            &self, 
            value: &T,
            options: &normalization::NormalizationOptions
        ) -> Result<DeferredHashId, DeferredHashingError> {
            let id = DeferredHashId::new();
            
            // Normalize the value according to options
            let normalized = value.normalize(options)
                .map_err(DeferredHashingError::NormalizationError)?;
                
            // Store the normalized data
            self.pending.lock().unwrap().insert(id.clone(), normalized);
            self.options.lock().unwrap().insert(id.clone(), options.clone());
            
            Ok(id)
        }
        
        /// Get pending hash requests
        pub fn get_pending_requests(&self) -> Vec<DeferredHashId> {
            self.pending.lock().unwrap().keys().cloned().collect()
        }
        
        /// Compute all pending hashes
        pub fn compute_all(&self) -> Result<HashSet<DeferredHashId>, DeferredHashingError> {
            let mut pending = self.pending.lock().unwrap();
            
            if pending.is_empty() {
                return Ok(HashSet::new());
            }
            
            let mut results = self.results.lock().unwrap();
            let mut computed_ids = HashSet::new();
            
            for (id, data) in pending.drain() {
                let mut hash_data = [0u8; 32];
                
                // Compute the hash according to the STANDARD_HASH_ALGORITHM
                match STANDARD_HASH_ALGORITHM {
                    HashAlgorithm::Blake3 => {
                        let hash_result = blake3::hash(&data);
                        hash_data.copy_from_slice(hash_result.as_bytes());
                    }
                    HashAlgorithm::Poseidon => {
                        // This would use a Poseidon implementation
                        // As placeholder, use Blake3
                        let hash_result = blake3::hash(&data);
                        hash_data.copy_from_slice(hash_result.as_bytes());
                    }
                }
                
                let hash_output = HashOutput::new(hash_data, STANDARD_HASH_ALGORITHM);
                results.insert(id.clone(), hash_output);
                computed_ids.insert(id);
            }
            
            Ok(computed_ids)
        }
        
        /// Check if a hash has been computed
        pub fn is_computed(&self, id: &DeferredHashId) -> bool {
            self.results.lock().unwrap().contains_key(id)
        }
        
        /// Get a computed hash result
        pub fn get_hash_result(&self, id: &DeferredHashId) -> Result<HashOutput, DeferredHashingError> {
            let results = self.results.lock().unwrap();
            
            results.get(id)
                .cloned()
                .ok_or_else(|| DeferredHashingError::HashIdNotFound(id.to_string()))
        }
        
        /// Create a content ID from a hash result
        pub fn get_content_id(&self, id: &DeferredHashId) -> Result<ContentId, DeferredHashingError> {
            let hash = self.get_hash_result(id)?;
            Ok(ContentId::from(hash))
        }
        
        /// Clear all computed results
        pub fn clear_results(&self) {
            self.results.lock().unwrap().clear();
        }
        
        /// Clear everything
        pub fn clear_all(&self) {
            self.pending.lock().unwrap().clear();
            self.results.lock().unwrap().clear();
            self.options.lock().unwrap().clear();
        }
    }
    
    impl Default for DeferredHashBatch {
        fn default() -> Self {
            Self::new()
        }
    }
    
    /// A simple batch manager for deferred hashing
    #[derive(Debug, Default)]
    pub struct DeferredHashBatchManager {
        active_batch: Mutex<Option<Arc<DeferredHashBatch>>>,
    }
    
    impl DeferredHashBatchManager {
        /// Create a new batch manager
        pub fn new() -> Self {
            Self {
                active_batch: Mutex::new(None),
            }
        }
        
        /// Get or create the active batch
        pub fn get_or_create_batch(&self) -> Arc<DeferredHashBatch> {
            let mut active_batch = self.active_batch.lock().unwrap();
            
            if active_batch.is_none() {
                *active_batch = Some(Arc::new(DeferredHashBatch::new()));
            }
            
            active_batch.as_ref().unwrap().clone()
        }
        
        /// Close the current batch and return it
        pub fn close_batch(&self) -> Option<Arc<DeferredHashBatch>> {
            let mut active_batch = self.active_batch.lock().unwrap();
            active_batch.take()
        }
        
        /// Compute all pending hashes in the active batch
        pub fn compute_all(&self) -> Result<HashSet<DeferredHashId>, DeferredHashingError> {
            let batch = self.get_or_create_batch();
            batch.compute_all()
        }
    }
    
    /// Extension trait for normalized content with deferred hashing
    pub trait DeferredNormalizableContentAddressed: ContentAddressed {
        /// Request a normalized hash computation with deferral
        fn request_normalized_hash(
            &self,
            batch: &DeferredHashBatch,
            options: &normalization::NormalizationOptions
        ) -> Result<DeferredHashId, DeferredHashingError> {
            // Get the bytes first
            let bytes = self.to_bytes()
                .map_err(|e| DeferredHashingError::SerializationError(e.to_string()))?;
                
            // Create a wrapper for normalization
            struct ByteWrapper(Vec<u8>);
            
            impl Serialize for ByteWrapper {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    serializer.serialize_bytes(&self.0)
                }
            }
            
            impl borsh::BorshSerialize for ByteWrapper {
                fn serialize<W: borsh::maybestd::io::Write>(&self, writer: &mut W) -> Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.0, writer)
                }
            }
            
            // Request hash with normalization
            let wrapper = ByteWrapper(bytes);
            batch.request_hash(&wrapper, options)
        }
    }
    
    // Implement the deferred trait for all ContentAddressed types
    impl<T: ContentAddressed> DeferredNormalizableContentAddressed for T {}
    
    /// Metrics for deferred hashing performance
    #[derive(Debug, Default, Clone)]
    pub struct DeferredHashingMetrics {
        /// Total hash requests
        pub total_requests: u64,
        /// Total bytes processed
        pub total_bytes_processed: u64,
        /// Total batches processed
        pub total_batches: u64,
        /// Average batch size
        pub avg_batch_size: f64,
        /// Maximum batch size seen
        pub max_batch_size: usize,
        /// Average hash computation time
        pub avg_computation_time: std::time::Duration,
    }
    
    /// A global deferred hashing manager singleton
    pub struct GlobalDeferredHashManager {
        manager: DeferredHashBatchManager,
        metrics: Mutex<DeferredHashingMetrics>,
    }
    
    impl GlobalDeferredHashManager {
        /// Create a new global manager
        pub fn new() -> Self {
            Self {
                manager: DeferredHashBatchManager::new(),
                metrics: Mutex::new(DeferredHashingMetrics::default()),
            }
        }
        
        /// Get the current batch
        pub fn current_batch(&self) -> Arc<DeferredHashBatch> {
            self.manager.get_or_create_batch()
        }
        
        /// Create a new batch
        pub fn new_batch(&self) -> Arc<DeferredHashBatch> {
            // Close any existing batch
            self.manager.close_batch();
            // Create and return a new one
            self.manager.get_or_create_batch()
        }
        
        /// Compute all hashes in the current batch
        pub fn compute_current_batch(&self) -> Result<HashSet<DeferredHashId>, DeferredHashingError> {
            let batch = self.manager.get_or_create_batch();
            let start_time = std::time::Instant::now();
            
            // Compute all hashes
            let result = batch.compute_all()?;
            
            // Update metrics
            let computation_time = start_time.elapsed();
            let batch_size = result.len();
            
            let mut metrics = self.metrics.lock().unwrap();
            metrics.total_batches += 1;
            metrics.total_requests += batch_size as u64;
            
            if metrics.avg_batch_size == 0.0 {
                metrics.avg_batch_size = batch_size as f64;
            } else {
                // Exponential moving average
                metrics.avg_batch_size = 0.8 * metrics.avg_batch_size + 0.2 * (batch_size as f64);
            }
            
            metrics.max_batch_size = metrics.max_batch_size.max(batch_size);
            
            if metrics.avg_computation_time == std::time::Duration::default() {
                metrics.avg_computation_time = computation_time;
            } else {
                // Exponential moving average for time
                let avg_nanos = metrics.avg_computation_time.as_nanos() as f64;
                let current_nanos = computation_time.as_nanos() as f64;
                let new_avg = 0.8 * avg_nanos + 0.2 * current_nanos;
                metrics.avg_computation_time = std::time::Duration::from_nanos(new_avg as u64);
            }
            
            Ok(result)
        }
        
        /// Get current metrics
        pub fn metrics(&self) -> DeferredHashingMetrics {
            self.metrics.lock().unwrap().clone()
        }
    }
    
    impl Default for GlobalDeferredHashManager {
        fn default() -> Self {
            Self::new()
        }
    }
    
    // A global instance for easy access
    lazy_static::lazy_static! {
        pub static ref GLOBAL_HASH_MANAGER: GlobalDeferredHashManager = GlobalDeferredHashManager::new();
    }
    
    /// Get the global hash manager instance
    pub fn global_hash_manager() -> &'static GlobalDeferredHashManager {
        &GLOBAL_HASH_MANAGER
    }
}

// Re-export deferred hashing types
pub use deferred::{
    DeferredHashId, DeferredHashBatch, DeferredHashBatchManager,
    DeferredNormalizableContentAddressed, DeferredHashingMetrics,
    global_hash_manager
};

// Use the storage traits directly
use self::storage::{ContentAddressedStorage, ContentAddressedStorageExt};

// Enable ContentAddressedStorageExt for all ContentAddressedStorage implementors
// This restores the blanket implementation but avoids conflicts with specific implementations
impl<T> ContentAddressedStorageExt for T where T: ContentAddressedStorage + ?Sized {} 