// Common traits and interfaces for cryptographic operations
//
// This module defines common traits used throughout the crypto package.

use std::fmt;
use std::sync::Arc;
use thiserror::Error;
use crate::hash::{HashOutput, HashError};

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
}

/// Standard content-addressed storage interface
pub trait ContentAddressedStorage: Send + Sync {
    /// Store binary data and return content ID
    fn store_bytes(&self, bytes: &[u8]) -> Result<crate::hash::ContentId, StorageError>;
    
    /// Check if an object exists in storage
    fn contains(&self, id: &crate::hash::ContentId) -> bool;
    
    /// Retrieve binary data for an object
    fn get_bytes(&self, id: &crate::hash::ContentId) -> Result<Vec<u8>, StorageError>;
    
    /// Remove an object from storage
    fn remove(&self, id: &crate::hash::ContentId) -> Result<(), StorageError>;
    
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
    fn store<T: ContentAddressed>(&self, object: &T) -> Result<crate::hash::ContentId, StorageError> {
        // Serialize the object
        let bytes = object.to_bytes()?;
        // Store the bytes
        self.store_bytes(&bytes)
    }
    
    /// Retrieve an object from storage by its content ID
    fn get<T: ContentAddressed>(&self, id: &crate::hash::ContentId) -> Result<T, StorageError> {
        let bytes = self.get_bytes(id)?;
        T::from_bytes(&bytes).map_err(|e| StorageError::HashError(e))
    }
} 