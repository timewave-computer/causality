// Nullifier implementation for privacy-preserving operations
// Original file: src/crypto/nullifier.rs

// Nullifier implementation for privacy-preserving operations
//
// Nullifiers are cryptographic constructs used to prevent double-spending
// in privacy-preserving systems. This module provides functionality for
// creating and verifying nullifiers.

use std::fmt;
use std::collections::HashMap;
use std::sync::RwLock;
use thiserror::Error;

use causality_types::crypto_primitives::HashError;
use causality_types::ContentId;

// Import from our crate
use crate::hash::{HashFunction, Blake3HashFunction};

/// Error type for nullifier operations
#[derive(Debug, Error)]
pub enum NullifierError {
    /// Invalid nullifier
    #[error("Invalid nullifier: {0}")]
    InvalidNullifier(String),
    
    /// Invalid data
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(Box<dyn std::error::Error + Send + Sync>),
    
    /// Nullifier already spent
    #[error("Nullifier already spent")]
    AlreadySpent,
    
    /// Hash error
    #[error("Hash error: {0}")]
    HashError(#[from] HashError),
    
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// A cryptographic nullifier used to prevent double-spending
///
/// Nullifiers are deterministic values derived from secret data that
/// can be publicly disclosed to prevent double-spending while maintaining
/// privacy about the source of the nullifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nullifier {
    /// The raw nullifier data
    data: Vec<u8>,
}

impl Nullifier {
    /// Create a new nullifier from raw data
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
    
    /// Create a nullifier from a secret and some public data
    pub fn from_secret(secret: &[u8], public_data: &[u8]) -> Result<Self, NullifierError> {
        // Combine secret and public data with a separator
        let mut combined = Vec::new();
        combined.extend_from_slice(secret);
        combined.push(0); // Separator
        combined.extend_from_slice(public_data);
        
        // Hash the combined data to get the nullifier
        let hasher = Blake3HashFunction::new();
        let hash = hasher.hash(&combined);
        
        Ok(Self::new(hash.as_bytes().to_vec()))
    }
    
    /// Get the raw nullifier data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    
    /// Convert the nullifier to raw bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }
    
    /// Convert the nullifier to a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.data)
    }
    
    /// Create a nullifier from a hex string
    pub fn from_hex(hex: &str) -> Result<Self, NullifierError> {
        match hex::decode(hex) {
            Ok(data) => Ok(Self::new(data)),
            Err(_) => Err(NullifierError::InvalidNullifier("Invalid hex format".to_string())),
        }
    }
    
    /// Create a nullifier from a content hash
    pub fn from_content_hash(content_hash: &ContentId) -> Result<Self, NullifierError> {
        Ok(Self::new(content_hash.as_bytes().to_vec()))
    }
}

impl fmt::Display for Nullifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// A commitment to a nullifier and associated data
///
/// This struct represents a commitment to a nullifier and potentially
/// other associated data. It can be used to prove knowledge of a
/// nullifier without revealing it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NullifierCommitment {
    /// The commitment data
    data: Vec<u8>,
}

impl NullifierCommitment {
    /// Create a new nullifier commitment from raw data
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
    
    /// Create a commitment to a nullifier
    pub fn commit(nullifier: &Nullifier) -> Self {
        let hasher = Blake3HashFunction::new();
        let hash = hasher.hash(nullifier.data());
        Self::new(hash.as_bytes().to_vec())
    }
    
    /// Get the commitment data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    
    /// Verify that a nullifier matches this commitment
    pub fn verify(&self, nullifier: &Nullifier) -> bool {
        let expected = Self::commit(nullifier);
        self.data == expected.data
    }
    
    /// Convert the commitment to raw bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }
    
    /// Convert the commitment to a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.data)
    }
    
    /// Create a commitment from a hex string
    pub fn from_hex(hex: &str) -> Result<Self, NullifierError> {
        match hex::decode(hex) {
            Ok(data) => Ok(Self::new(data)),
            Err(_) => Err(NullifierError::InvalidNullifier("Invalid hex format".to_string())),
        }
    }
}

/// A registry for tracking spent nullifiers
///
/// This struct provides an API for registering and checking nullifiers,
/// which are typically used to prevent double-spending in privacy-preserving
/// protocols.
pub struct NullifierRegistry {
    /// Map of spent nullifiers
    spent: RwLock<HashMap<String, bool>>,
}

impl NullifierRegistry {
    /// Create a new nullifier registry
    pub fn new() -> Self {
        Self {
            spent: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a nullifier as spent
    pub fn register(&self, nullifier: &Nullifier) -> Result<(), NullifierError> {
        let mut spent = self.spent.write().unwrap();
        let key = nullifier.to_hex();
        
        if spent.contains_key(&key) {
            return Err(NullifierError::AlreadySpent);
        }
        
        spent.insert(key, true);
        Ok(())
    }
    
    /// Check if a nullifier has been spent
    pub fn is_spent(&self, nullifier: &Nullifier) -> Result<bool, NullifierError> {
        let spent = self.spent.read().unwrap();
        let key = nullifier.to_hex();
        
        Ok(spent.contains_key(&key))
    }
    
    /// Clear all spent nullifiers
    pub fn clear(&self) -> Result<(), NullifierError> {
        let mut spent = self.spent.write().unwrap();
        spent.clear();
        Ok(())
    }
}

/// Factory for creating nullifier-related components
pub struct NullifierFactory {
    hash_function: Blake3HashFunction,
}

impl NullifierFactory {
    /// Create a new nullifier factory
    pub fn new() -> Self {
        Self {
            hash_function: Blake3HashFunction::new(),
        }
    }
    
    /// Create a nullifier from a secret and public data
    pub fn create_nullifier(&self, secret: &[u8], public_data: &[u8]) -> Result<Nullifier, NullifierError> {
        Nullifier::from_secret(secret, public_data)
    }
    
    /// Create a nullifier registry
    pub fn create_registry(&self) -> NullifierRegistry {
        NullifierRegistry::new()
    }
}

impl Default for NullifierFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nullifier_creation() {
        let secret = b"my_secret";
        let public_data = b"public_data";
        
        let nullifier = Nullifier::from_secret(secret, public_data).unwrap();
        assert!(!nullifier.data().is_empty());
        
        let hex = nullifier.to_hex();
        let from_hex = Nullifier::from_hex(&hex).unwrap();
        assert_eq!(nullifier, from_hex);
    }
    
    #[test]
    fn test_nullifier_commitment() {
        let secret = b"my_secret";
        let public_data = b"public_data";
        
        let nullifier = Nullifier::from_secret(secret, public_data).unwrap();
        let commitment = NullifierCommitment::commit(&nullifier);
        
        assert!(commitment.verify(&nullifier));
        
        // Modify the nullifier - verification should fail
        let modified = Nullifier::new(vec![0, 1, 2, 3]);
        assert!(!commitment.verify(&modified));
    }
    
    #[test]
    fn test_nullifier_registry() {
        let registry = NullifierRegistry::new();
        
        let nullifier1 = Nullifier::from_secret(b"secret1", b"public1").unwrap();
        let nullifier2 = Nullifier::from_secret(b"secret2", b"public2").unwrap();
        
        // Initially, both nullifiers should be unspent
        assert!(!registry.is_spent(&nullifier1).unwrap());
        assert!(!registry.is_spent(&nullifier2).unwrap());
        
        // Register nullifier1
        registry.register(&nullifier1).unwrap();
        
        // Now, nullifier1 should be spent, but nullifier2
        assert!(registry.is_spent(&nullifier1).unwrap());
        assert!(!registry.is_spent(&nullifier2).unwrap());
        
        // Try to register nullifier1 again
        let result = registry.register(&nullifier1);
        assert!(result.is_err());
        
        // Clear the registry
        registry.clear().unwrap();
        
        // Both nullifiers should be unspent again
        assert!(!registry.is_spent(&nullifier1).unwrap());
        assert!(!registry.is_spent(&nullifier2).unwrap());
    }
    
    #[test]
    fn test_nullifier_factory() {
        let factory = NullifierFactory::new();
        
        let nullifier = factory.create_nullifier(b"secret", b"public").unwrap();
        assert!(!nullifier.data().is_empty());
        
        let registry = factory.create_registry();
        assert!(!registry.is_spent(&nullifier).unwrap());
    }
} 