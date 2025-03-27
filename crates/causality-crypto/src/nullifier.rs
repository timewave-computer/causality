// Nullifier generation and validation
// Original file: src/crypto/nullifier.rs

// Content-addressed Nullifier Tracking System
//
// This module implements a nullifier tracking system for content-addressed objects,
// allowing for one-time use verification of objects.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use thiserror::Error;

use causality_crypto::{ContentAddressed, ContentId, HashOutput, HashError, HashAlgorithm};
use causality_crypto::{MerkleSmt, SmtKeyValue, SmtError, H256};
use sparse_merkle_tree::default_store::DefaultStore;
use sparse_merkle_tree::traits::{StoreReadOps, StoreWriteOps};

/// Errors related to nullifier operations
#[derive(Debug, Error)]
pub enum NullifierError {
    /// Nullifier already exists
    #[error("Nullifier already exists: {0}")]
    AlreadyExists(String),
    
    /// Nullifier not found
    #[error("Nullifier not found: {0}")]
    NotFound(String),
    
    /// Invalid nullifier format
    #[error("Invalid nullifier format: {0}")]
    InvalidFormat(String),
    
    /// SMT error
    #[error("SMT error: {0}")]
    SmtError(#[from] SmtError),
    
    /// Hash error
    #[error("Hash error: {0}")]
    HashError(#[from] HashError),
    
    /// General error
    #[error("Nullifier error: {0}")]
    GeneralError(String),
}

/// Represents a nullifier for a content-addressed object
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nullifier {
    /// The nullifier value
    pub value: [u8; 32],
    /// The content ID this nullifier is for
    pub content_id: ContentId,
    /// Metadata associated with this nullifier
    pub metadata: HashMap<String, String>,
}

impl Nullifier {
    /// Create a new nullifier for a content-addressed object
    pub fn new<T: ContentAddressed>(object: &T) -> Result<Self, NullifierError> {
        let content_id = object.content_id();
        let content_hash = object.content_hash();
        
        // Create a nullifier by hashing the content hash with a different algorithm
        // or by using a different domain separator
        let nullifier_value = Self::generate_nullifier_value(&content_hash)?;
        
        Ok(Self {
            value: nullifier_value,
            content_id,
            metadata: HashMap::new(),
        })
    }
    
    /// Generate a nullifier value from a content hash
    fn generate_nullifier_value(content_hash: &HashOutput) -> Result<[u8; 32], NullifierError> {
        // Add a domain separator to ensure nullifiers are different from regular hashes
        let mut data = Vec::with_capacity(content_hash.as_bytes().len() + 8);
        data.extend_from_slice(b"nullifr:"); // Domain separator
        data.extend_from_slice(content_hash.as_bytes());
        
        // Hash the data to get the nullifier value
        let hash_output = content_hash.algorithm()
            .create_hasher()
            .map_err(|e| NullifierError::HashError(e))?
            .hash(&data);

        // Convert to 32-byte array
        let mut value = [0u8; 32];
        let bytes = hash_output.as_bytes();
        value.copy_from_slice(&bytes[0..32]);
        
        Ok(value)
    }
    
    /// Convert the nullifier to an SMT key
    pub fn to_smt_key(&self) -> H256 {
        H256::from(self.value)
    }
    
    /// Add metadata to the nullifier
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

impl fmt::Display for Nullifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(&self.value))
    }
}

impl AsRef<[u8]> for Nullifier {
    fn as_ref(&self) -> &[u8] {
        &self.value
    }
}

/// Status of a nullifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NullifierStatus {
    /// The nullifier is not in the registry
    NotFound,
    
    /// The nullifier is in the registry but not spent
    Registered,
    
    /// The nullifier has been spent
    Spent,
}

impl fmt::Display for NullifierStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "NotFound"),
            Self::Registered => write!(f, "Registered"),
            Self::Spent => write!(f, "Spent"),
        }
    }
}

/// Trait for nullifier tracking systems
pub trait NullifierTracking {
    /// The nullifier type
    type Nullifier: AsRef<[u8]>;
    
    /// Generate a nullifier for a content hash
    fn generate_nullifier(&self, hash: &HashOutput) -> Result<Self::Nullifier, NullifierError>;
    
    /// Register a nullifier
    fn register_nullifier(&self, nullifier: &Self::Nullifier) -> Result<(), NullifierError>;
    
    /// Check if a nullifier has been spent
    fn is_spent(&self, nullifier: &Self::Nullifier) -> bool;
    
    /// Mark a nullifier as spent
    fn mark_spent(&self, nullifier: &Self::Nullifier) -> Result<(), NullifierError>;
    
    /// Get the status of a nullifier
    fn get_status(&self, nullifier: &Self::Nullifier) -> NullifierStatus;
}

/// An SMT-based nullifier registry
pub struct SmtNullifierRegistry<S: StoreReadOps<SmtKeyValue> + StoreWriteOps<SmtKeyValue>> {
    /// The merkle tree for storing nullifiers
    tree: RwLock<MerkleSmt<S>>,
    /// Mapping of nullifier status
    status: RwLock<HashMap<[u8; 32], NullifierStatus>>,
}

impl<S: StoreReadOps<SmtKeyValue> + StoreWriteOps<SmtKeyValue>> SmtNullifierRegistry<S> {
    /// Create a new SMT-based nullifier registry
    pub fn new(store: S) -> Self {
        Self {
            tree: RwLock::new(MerkleSmt::new(store)),
            status: RwLock::new(HashMap::new()),
        }
    }
    
    /// Get the current Merkle root
    pub fn root(&self) -> Result<H256, NullifierError> {
        let tree = self.tree.read().map_err(|_| 
            NullifierError::GeneralError("Failed to acquire read lock on tree".to_string()))?;
        
        tree.root().map_err(|e| e.into())
    }
}

impl<S: StoreReadOps<SmtKeyValue> + StoreWriteOps<SmtKeyValue>> NullifierTracking 
    for SmtNullifierRegistry<S> 
{
    type Nullifier = Nullifier;
    
    fn generate_nullifier(&self, hash: &HashOutput) -> Result<Self::Nullifier, NullifierError> {
        let nullifier_value = Nullifier::generate_nullifier_value(hash)?;
        
        Ok(Nullifier {
            value: nullifier_value,
            content_id: ContentId::from(hash.clone()),
            metadata: HashMap::new(),
        })
    }
    
    fn register_nullifier(&self, nullifier: &Self::Nullifier) -> Result<(), NullifierError> {
        let mut status_map = self.status.write().map_err(|_| 
            NullifierError::GeneralError("Failed to acquire write lock on status".to_string()))?;
        
        if status_map.contains_key(&nullifier.value) {
            return Err(NullifierError::AlreadyExists(
                format!("Nullifier already exists: {}", nullifier)
            ));
        }
        
        // Add to the status map
        status_map.insert(nullifier.value, NullifierStatus::Registered);
        
        Ok(())
    }
    
    fn is_spent(&self, nullifier: &Self::Nullifier) -> bool {
        let status_map = self.status.read()
            .expect("Failed to acquire read lock on status");
        
        matches!(status_map.get(&nullifier.value), Some(NullifierStatus::Spent))
    }
    
    fn mark_spent(&self, nullifier: &Self::Nullifier) -> Result<(), NullifierError> {
        let mut status_map = self.status.write().map_err(|_| 
            NullifierError::GeneralError("Failed to acquire write lock on status".to_string()))?;
        
        let status = status_map.get(&nullifier.value).copied()
            .unwrap_or(NullifierStatus::NotFound);
        
        match status {
            NullifierStatus::NotFound => {
                return Err(NullifierError::NotFound(
                    format!("Nullifier not found: {}", nullifier)
                ));
            },
            NullifierStatus::Spent => {
                return Err(NullifierError::AlreadyExists(
                    format!("Nullifier already spent: {}", nullifier)
                ));
            },
            NullifierStatus::Registered => {
                // Update status to spent
                status_map.insert(nullifier.value, NullifierStatus::Spent);
                
                // Update the Merkle tree
                let mut tree = self.tree.write().map_err(|_| 
                    NullifierError::GeneralError("Failed to acquire write lock on tree".to_string()))?;
                
                let key = nullifier.to_smt_key();
                let value = SmtKeyValue::from_bytes(&[1u8; 32])
                    .map_err(|e| NullifierError::SmtError(e))?;
                
                tree.update(key, value).map_err(|e| NullifierError::SmtError(e))?;
                
                Ok(())
            },
        }
    }
    
    fn get_status(&self, nullifier: &Self::Nullifier) -> NullifierStatus {
        let status_map = self.status.read()
            .expect("Failed to acquire read lock on status");
        
        *status_map.get(&nullifier.value).unwrap_or(&NullifierStatus::NotFound)
    }
}

/// A factory for creating nullifier tracking implementations
pub struct NullifierFactory;

impl NullifierFactory {
    /// Create a new SMT-based nullifier registry with default storage
    pub fn create_smt_registry() -> Arc<dyn NullifierTracking<Nullifier = Nullifier> + Send + Sync> {
        let store = DefaultStore::default();
        let registry = SmtNullifierRegistry::new(store);
        Arc::new(registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_crypto::{HashFactory, HashAlgorithm};
    use crate::crypto::ContentAddressed;
    use borsh::{BorshSerialize, BorshDeserialize};
    
    #[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
    struct TestObject {
        id: u64,
        name: String,
    }
    
    impl ContentAddressed for TestObject {
        fn content_hash(&self) -> Result<HashOutput, HashError> {
            let hasher = HashFactory::default().create_hasher().unwrap();
            let data = self.try_to_vec().map_err(|e| HashError::SerializationError(e.to_string()))?;
            Ok(hasher.hash(&data))
        }
        
        fn verify(&self, expected_hash: &HashOutput) -> Result<bool, HashError> {
            let actual_hash = self.content_hash()?;
            Ok(actual_hash == *expected_hash)
        }
        
        fn to_bytes(&self) -> Result<Vec<u8>, HashError> {
            self.try_to_vec().map_err(|e| HashError::SerializationError(e.to_string()))
        }
        
        fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
            BorshDeserialize::try_from_slice(bytes)
                .map_err(|e| HashError::SerializationError(e.to_string()))
        }
    }
    
    #[test]
    fn test_nullifier_creation() {
        let obj = TestObject {
            id: 1,
            name: "Test".to_string(),
        };
        
        let nullifier = Nullifier::new(&obj).unwrap();
        
        assert_eq!(nullifier.content_id, obj.content_id().unwrap());
        assert!(!nullifier.value.iter().all(|&b| b == 0));
    }
    
    #[test]
    fn test_nullifier_registry() {
        let registry = NullifierFactory::create_smt_registry();
        
        let obj = TestObject {
            id: 2,
            name: "Test Object".to_string(),
        };
        
        let hash = obj.content_hash().unwrap();
        let nullifier = registry.generate_nullifier(&hash).unwrap();
        
        // Initially not in registry
        assert_eq!(registry.get_status(&nullifier), NullifierStatus::NotFound);
        assert!(!registry.is_spent(&nullifier));
        
        // Register the nullifier
        registry.register_nullifier(&nullifier).unwrap();
        assert_eq!(registry.get_status(&nullifier), NullifierStatus::Registered);
        assert!(!registry.is_spent(&nullifier));
        
        // Mark as spent
        registry.mark_spent(&nullifier).unwrap();
        assert_eq!(registry.get_status(&nullifier), NullifierStatus::Spent);
        assert!(registry.is_spent(&nullifier));
        
        // Try to register again (should fail)
        assert!(registry.register_nullifier(&nullifier).is_err());
        
        // Try to mark as spent again (should fail)
        assert!(registry.mark_spent(&nullifier).is_err());
    }
} 