// Common traits and interfaces for cryptographic operations
//
// This module defines common traits used throughout the crypto package.

use std::fmt::Debug;
use thiserror::Error;
use causality_types::ContentId;
use causality_types::crypto_primitives::{HashOutput, HashError};
use serde_json;

/// Trait for objects that can be content-addressed
pub trait ContentAddressed {
    /// Get the content hash of this object
    fn content_hash(&self) -> Result<HashOutput, HashError>;
    
    /// Verify that the object matches its hash
    fn verify(&self, expected_hash: &HashOutput) -> Result<bool, HashError> {
        let actual_hash = self.content_hash()?;
        Ok(actual_hash == *expected_hash)
    }
    
    /// Convert to a serialized form for storage
    fn to_bytes(&self) -> Result<Vec<u8>, HashError>;
    
    /// Create from serialized form
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> where Self: Sized;
}

/// Error type for storage operations
#[derive(Debug, Error)]
pub enum StorageError {
    /// Object not found
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    /// Hashing error
    #[error("Hashing error: {0}")]
    HashingError(String),
    
    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),
    
    /// Invalid content ID
    #[error("Invalid content ID: {0}")]
    InvalidContentId(String),
    
    /// Hash error
    #[error("Hash error: {0}")]
    HashError(#[from] HashError),
}

/// Standard content-addressed storage interface
pub trait ContentAddressedStorage: Send + Sync + Debug {
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
        let bytes = object.to_bytes()
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        self.store_bytes(&bytes)
    }
    
    /// Retrieve an object from storage by its content ID
    fn get<T: ContentAddressed>(&self, id: &ContentId) -> Result<T, StorageError> {
        let bytes = self.get_bytes(id)?;
        T::from_bytes(&bytes)
            .map_err(|e| StorageError::DeserializationError(e.to_string()))
    }
    
    /// Store a serializable value that is not ContentAddressed
    fn store_value<T: serde::Serialize>(&self, value: &T) -> Result<ContentId, StorageError> {
        let bytes = serde_json::to_vec(value)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        self.store_bytes(&bytes)
    }
    
    /// Get a deserializable value that is not ContentAddressed
    fn get_value<T: serde::de::DeserializeOwned>(&self, id: &ContentId) -> Result<T, StorageError> {
        let bytes = self.get_bytes(id)?;
        serde_json::from_slice(&bytes)
            .map_err(|e| StorageError::DeserializationError(e.to_string()))
    }
}

// Remove the blanket implementation of ContentAddressedStorageExt for all ContentAddressedStorage types
// This conflicts with specific implementations in other modules
// Each type should implement ContentAddressedStorageExt explicitly
// to avoid conflicts with trait implementations
// impl<T: ContentAddressedStorage> ContentAddressedStorageExt for T {} 