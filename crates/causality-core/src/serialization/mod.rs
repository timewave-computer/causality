// Serialization utilities
//
// This module provides utilities for serializing and deserializing data
// in a consistent and canonical way, especially for content addressing.

use thiserror::Error;

/// Error type for serialization operations
#[derive(Debug, Error, Clone)]
pub enum SerializationError {
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
    
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    #[error("Missing field: {0}")]
    MissingField(String),
    
    #[error("Unsupported type: {0}")]
    UnsupportedType(String),
    
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(String),
    
    #[error("{0}")]
    Other(String),
}

/// A utility for serializing and deserializing data in a canonical format
pub struct Serializer;

impl Serializer {
    /// Serialize data to bytes in a canonical format
    pub fn to_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, SerializationError> {
        bincode::serialize(value)
            .map_err(|e| SerializationError::SerializationFailed(e.to_string()))
    }
    
    /// Deserialize data from bytes
    pub fn from_bytes<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, SerializationError> {
        bincode::deserialize(bytes)
            .map_err(|e| SerializationError::DeserializationFailed(e.to_string()))
    }
    
    /// Serialize data to a string in a canonical format
    pub fn to_string<T: serde::Serialize>(value: &T) -> Result<String, SerializationError> {
        serde_json::to_string(value)
            .map_err(|e| SerializationError::SerializationFailed(e.to_string()))
    }
    
    /// Deserialize data from a string
    pub fn from_string<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, SerializationError> {
        serde_json::from_str(s)
            .map_err(|e| SerializationError::DeserializationFailed(e.to_string()))
    }
    
    /// Serialize data to a pretty-printed string
    pub fn to_pretty_string<T: serde::Serialize>(value: &T) -> Result<String, SerializationError> {
        serde_json::to_string_pretty(value)
            .map_err(|e| SerializationError::SerializationFailed(e.to_string()))
    }
}

/// A type that can be serialized and deserialized
pub trait Serializable: serde::Serialize + serde::de::DeserializeOwned + Sized {
    /// Convert to bytes
    fn to_bytes(&self) -> Result<Vec<u8>, SerializationError> {
        Serializer::to_bytes(self)
    }
    
    /// Create from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, SerializationError> {
        Serializer::from_bytes(bytes)
    }
    
    /// Convert to a string
    fn to_string(&self) -> Result<String, SerializationError> {
        Serializer::to_string(self)
    }
    
    /// Create from a string
    fn from_string(s: &str) -> Result<Self, SerializationError> {
        Serializer::from_string(s)
    }
    
    /// Convert to a pretty-printed string
    fn to_pretty_string(&self) -> Result<String, SerializationError> {
        Serializer::to_pretty_string(self)
    }
}

// Implement Serializable for common types
impl<T: serde::Serialize + serde::de::DeserializeOwned> Serializable for T {}

// Convenience functions for module-level access
/// Serialize data to bytes in a canonical format
pub fn to_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, SerializationError> {
    Serializer::to_bytes(value)
}

/// Deserialize data from bytes
pub fn from_bytes<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, SerializationError> {
    Serializer::from_bytes(bytes)
}

/// Serialize data to a string in a canonical format
pub fn to_string<T: serde::Serialize>(value: &T) -> Result<String, SerializationError> {
    Serializer::to_string(value)
}

/// Deserialize data from a string
pub fn from_string<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, SerializationError> {
    Serializer::from_string(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct TestStruct {
        a: i32,
        b: String,
        c: Vec<u8>,
    }
    
    #[test]
    fn test_serialization_roundtrip() {
        let test = TestStruct {
            a: 42,
            b: "hello".to_string(),
            c: vec![1, 2, 3],
        };
        
        // Serialize to bytes
        let bytes = Serializer::to_bytes(&test).unwrap();
        
        // Deserialize from bytes
        let deserialized: TestStruct = Serializer::from_bytes(&bytes).unwrap();
        
        // Should be the same
        assert_eq!(test, deserialized);
    }
    
    #[test]
    fn test_serializable_trait() {
        let test = TestStruct {
            a: 42,
            b: "hello".to_string(),
            c: vec![1, 2, 3],
        };
        
        // Serialize to bytes
        let bytes = test.to_bytes().unwrap();
        
        // Deserialize from bytes
        let deserialized = TestStruct::from_bytes(&bytes).unwrap();
        
        // Should be the same
        assert_eq!(test, deserialized);
    }
    
    #[test]
    fn test_json_serialization() {
        let test = TestStruct {
            a: 42,
            b: "hello".to_string(),
            c: vec![1, 2, 3],
        };
        
        // Serialize to string
        let json = test.to_string().unwrap();
        
        // Deserialize from string
        let deserialized = TestStruct::from_string(&json).unwrap();
        
        // Should be the same
        assert_eq!(test, deserialized);
    }
    
    #[test]
    fn test_module_functions() {
        let test = TestStruct {
            a: 42,
            b: "hello".to_string(),
            c: vec![1, 2, 3],
        };
        
        // Test module-level functions
        let bytes = to_bytes(&test).unwrap();
        let deserialized: TestStruct = from_bytes(&bytes).unwrap();
        assert_eq!(test, deserialized);
        
        let json = to_string(&test).unwrap();
        let deserialized: TestStruct = from_string(&json).unwrap();
        assert_eq!(test, deserialized);
    }
} 