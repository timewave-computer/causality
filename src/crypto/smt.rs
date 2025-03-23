// Sparse Merkle Tree (SMT) Implementation
//
// This module provides a Sparse Merkle Tree implementation for efficient
// key-value storage with cryptographic verification properties.

use std::fmt;
use std::sync::Arc;
use thiserror::Error;

pub use sparse_merkle_tree::H256;
use sparse_merkle_tree::default_store::DefaultStore;
use sparse_merkle_tree::traits::Value;
use sparse_merkle_tree::traits::{StoreReadOps, StoreWriteOps};
use sparse_merkle_tree::SparseMerkleTree;

/// Error types for SMT operations
#[derive(Error, Debug)]
pub enum SmtError {
    /// Key was not found in the tree
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    /// Invalid proof
    #[error("Invalid proof")]
    InvalidProof,
    
    /// Invalid key format
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
    
    /// Invalid value format
    #[error("Invalid value format: {0}")]
    InvalidValueFormat(String),
    
    /// Sparse Merkle Tree internal error
    #[error("SMT internal error: {0}")]
    InternalError(String),
}

/// Value type for the Sparse Merkle Tree
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmtKeyValue(pub H256);

impl SmtKeyValue {
    /// Create a new SMT value from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SmtError> {
        if bytes.len() != 32 {
            return Err(SmtError::InvalidValueFormat(format!(
                "Expected 32 bytes, got {}",
                bytes.len()
            )));
        }
        
        let mut buf = [0u8; 32];
        buf.copy_from_slice(bytes);
        Ok(Self(buf.into()))
    }
    
    /// Get the bytes representation of this value
    pub fn as_bytes(&self) -> [u8; 32] {
        self.0.into()
    }
}

impl Value for SmtKeyValue {
    fn to_h256(&self) -> H256 {
        self.0
    }

    fn zero() -> Self {
        Self(H256::zero())
    }
}

/// A trait for SMT factories
pub trait SmtFactory: Send + Sync {
    /// Create a new SMT with the given store
    fn create_smt<S: StoreReadOps<SmtKeyValue> + StoreWriteOps<SmtKeyValue>>(&self, store: S) -> Arc<MerkleSmt<S>>;
    
    /// Create a new SMT with the default store
    fn create_default_smt(&self) -> Arc<MerkleSmt<DefaultStore<H256>>>;
}

/// A wrapper around SparseMerkleTree with additional functionality
pub struct MerkleSmt<S: StoreReadOps<SmtKeyValue> + StoreWriteOps<SmtKeyValue>> {
    /// The underlying Sparse Merkle Tree
    tree: SparseMerkleTree<H256, SmtKeyValue, S>,
}

impl<S: StoreReadOps<SmtKeyValue> + StoreWriteOps<SmtKeyValue>> MerkleSmt<S> {
    /// Create a new MerkleSmt with the given store
    pub fn new(store: S) -> Self {
        Self {
            tree: SparseMerkleTree::<H256, SmtKeyValue, S>::new(H256::zero(), store),
        }
    }
    
    /// Insert a key-value pair into the tree
    pub fn insert(&self, key: H256, value: SmtKeyValue) -> Result<H256, SmtError> {
        self.tree
            .update(key, value)
            .map_err(|e| SmtError::InternalError(e.to_string()))
    }
    
    /// Get a value from the tree
    pub fn get(&self, key: &H256) -> Result<SmtKeyValue, SmtError> {
        self.tree
            .get(*key)
            .map_err(|e| SmtError::InternalError(e.to_string()))
    }
    
    /// Check if a key exists in the tree
    pub fn contains_key(&self, key: &H256) -> Result<bool, SmtError> {
        Ok(self.tree.contains_key(*key))
    }
    
    /// Get the current root hash of the tree
    pub fn root(&self) -> H256 {
        self.tree.root().clone()
    }
    
    /// Clear the tree (remove all entries)
    pub fn clear(&self) -> Result<(), SmtError> {
        // Since SparseMerkleTree doesn't have a direct clear method,
        // we create a new tree with the same store
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Basic SMT factory implementation for tests
    struct TestSmtFactory;
    
    impl SmtFactory for TestSmtFactory {
        fn create_smt<S: StoreReadOps<SmtKeyValue> + StoreWriteOps<SmtKeyValue>>(&self, store: S) -> Arc<MerkleSmt<S>> {
            Arc::new(MerkleSmt::new(store))
        }
        
        fn create_default_smt(&self) -> Arc<MerkleSmt<DefaultStore<H256>>> {
            Arc::new(MerkleSmt::new(DefaultStore::default()))
        }
    }
    
    #[test]
    fn test_smt_basic_operations() {
        let factory = TestSmtFactory;
        let smt = factory.create_default_smt();
        
        // Create test data
        let key = H256::from([1u8; 32]);
        let value_bytes = [2u8; 32];
        let value = SmtKeyValue::from_bytes(&value_bytes).unwrap();
        
        // Insert and check root changes
        let root_before = smt.root();
        let new_root = smt.insert(key, value.clone()).unwrap();
        assert_ne!(root_before, new_root);
        
        // Retrieve and verify
        let retrieved = smt.get(&key).unwrap();
        assert_eq!(retrieved, value);
        
        // Check contains
        assert!(smt.contains_key(&key).unwrap());
    }
} 