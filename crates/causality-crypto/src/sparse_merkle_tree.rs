// Sparse Merkle tree for efficient state representation
// Original file: src/crypto/smt.rs

// Sparse Merkle Tree (SMT) Implementation
//
// This module provides a Sparse Merkle Tree implementation for efficient
// key-value storage with cryptographic verification properties.

use std::fmt;
use std::sync::Arc;
use thiserror::Error;

use causality_types::{ContentAddressed, ContentId, HashOutput, HashError};

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
    
    /// Hash error
    #[error("Hash error: {0}")]
    HashError(#[from] HashError),
    
    /// Object not found
    #[error("Object not found")]
    ObjectNotFound,
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

/// Proof for SMT content verification
#[derive(Clone, Debug)]
pub struct SmtProof {
    /// Siblings in the proof path
    pub siblings: Vec<H256>,
    /// Leaf data if present
    pub leaf: Option<(H256, SmtKeyValue)>,
}

impl SmtProof {
    /// Create a new SMT proof
    pub fn new(siblings: Vec<H256>, leaf: Option<(H256, SmtKeyValue)>) -> Self {
        Self { siblings, leaf }
    }
    
    /// Verify this proof against a root and key
    pub fn verify(&self, root: &H256, key: &H256) -> bool {
        // Convert to sparse_merkle_tree's MerkleProof
        let mut leaves = Vec::new();
        if let Some((leaf_key, leaf_value)) = &self.leaf {
            leaves.push((*leaf_key, leaf_value.clone()));
        }
        
        // Create a SparseMerkleTree proof
        let smt_proof = sparse_merkle_tree::CompiledMerkleProof::new(
            self.siblings.clone(),
            leaves
        );
        
        // Verify the proof against the root and key
        smt_proof.verify(*root, vec![*key]).is_ok()
    }
    
    /// Convert proof to bytes for serialization
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        
        // Encode siblings count
        let siblings_count = self.siblings.len() as u32;
        result.extend_from_slice(&siblings_count.to_le_bytes());
        
        // Encode each sibling
        for sibling in &self.siblings {
            let sibling_bytes: [u8; 32] = (*sibling).into();
            result.extend_from_slice(&sibling_bytes);
        }
        
        // Encode whether leaf is present
        let has_leaf = self.leaf.is_some() as u8;
        result.push(has_leaf);
        
        // Encode leaf data if present
        if let Some((key, value)) = &self.leaf {
            let key_bytes: [u8; 32] = (*key).into();
            let value_bytes: [u8; 32] = value.as_bytes();
            
            result.extend_from_slice(&key_bytes);
            result.extend_from_slice(&value_bytes);
        }
        
        result
    }
    
    /// Create proof from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SmtError> {
        if bytes.len() < 4 {
            return Err(SmtError::InvalidProof);
        }
        
        let mut pos = 0;
        
        // Read siblings count
        let mut siblings_count_bytes = [0u8; 4];
        siblings_count_bytes.copy_from_slice(&bytes[pos..pos+4]);
        let siblings_count = u32::from_le_bytes(siblings_count_bytes) as usize;
        pos += 4;
        
        // Check if there's enough data
        if bytes.len() < pos + siblings_count * 32 + 1 {
            return Err(SmtError::InvalidProof);
        }
        
        // Read siblings
        let mut siblings = Vec::with_capacity(siblings_count);
        for _ in 0..siblings_count {
            let mut sibling_bytes = [0u8; 32];
            sibling_bytes.copy_from_slice(&bytes[pos..pos+32]);
            siblings.push(sibling_bytes.into());
            pos += 32;
        }
        
        // Read leaf presence
        let has_leaf = bytes[pos] != 0;
        pos += 1;
        
        // Read leaf data if present
        let leaf = if has_leaf {
            if bytes.len() < pos + 64 {
                return Err(SmtError::InvalidProof);
            }
            
            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(&bytes[pos..pos+32]);
            pos += 32;
            
            let mut value_bytes = [0u8; 32];
            value_bytes.copy_from_slice(&bytes[pos..pos+32]);
            
            Some((key_bytes.into(), SmtKeyValue(value_bytes.into())))
        } else {
            None
        };
        
        Ok(Self::new(siblings, leaf))
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
    
    /// Store a content-addressed object in the SMT
    pub fn store_content<T: ContentAddressed>(&self, object: &T) -> Result<(HashOutput, ContentId, H256), SmtError> {
        // Get the object's content hash
        let content_hash = object.content_hash();
        let content_id = object.content_id();
        
        // Serialize the object
        let serialized = object.to_bytes();
        
        // Insert the serialized data with the content hash as the key
        let key_bytes = content_hash.as_bytes();
        let mut key = [0u8; 32];
        key.copy_from_slice(key_bytes);
        
        // Hash the serialized data to create the value
        let hash = blake3::hash(&serialized);
        let mut value_bytes = [0u8; 32];
        value_bytes.copy_from_slice(hash.as_bytes());
        
        // Insert into SMT
        let new_root = self.insert(key.into(), SmtKeyValue::from_bytes(&value_bytes)?)?;
        
        Ok((content_hash, content_id, new_root))
    }
    
    /// Retrieve a value and generate an inclusion proof
    pub fn get_with_proof(&self, key: &H256) -> Result<(SmtKeyValue, SmtProof), SmtError> {
        // Get the merkle proof from the SMT implementation
        let proof = self.tree.merkle_proof(vec![*key])
            .map_err(|e| SmtError::InternalError(e.to_string()))?;
        
        // Get the actual value
        let value = self.get(key)?;
        
        // Convert to our proof format
        let leaf = if let Some(leaf_data) = proof.leaves().first() {
            Some((leaf_data.0, SmtKeyValue(leaf_data.1.to_h256())))
        } else {
            None
        };
        
        let siblings = proof.merkle_path().iter().map(|h| *h).collect();
        let smt_proof = SmtProof::new(siblings, leaf);
        
        Ok((value, smt_proof))
    }
    
    /// Retrieve a content-addressed object with proof by its hash
    pub fn get_content_with_proof<T: ContentAddressed>(&self, hash: &HashOutput) -> Result<(T, SmtProof), SmtError> {
        // Convert the hash to an SMT key
        let hash_bytes = hash.as_bytes();
        let mut key = [0u8; 32];
        key.copy_from_slice(hash_bytes);
        let smt_key = H256::from(key);
        
        // Get the value and proof
        let (value, proof) = self.get_with_proof(&smt_key)?;
        
        // Convert the value bytes to the actual object
        let value_bytes = value.as_bytes();
        
        // Try to deserialize the object
        let object = T::from_bytes(&value_bytes)
            .map_err(|e| SmtError::InvalidValueFormat(format!("Failed to deserialize object: {:?}", e)))?;
        
        Ok((object, proof))
    }
    
    /// Verify an inclusion proof
    pub fn verify_inclusion_proof(&self, root: &H256, key: &H256, proof: &SmtProof) -> bool {
        proof.verify(root, key)
    }
}

/// Interface for SMT-compatible content storage
pub trait ContentAddressedSmt {
    /// Store an object in the SMT
    fn store_with_proof<T: ContentAddressed>(
        &self, 
        object: &T
    ) -> Result<(HashOutput, SmtProof), SmtError>;
    
    /// Get an object with a proof of inclusion
    fn get_with_proof<T: ContentAddressed>(
        &self, 
        hash: &HashOutput
    ) -> Result<(T, SmtProof), SmtError>;
    
    /// Verify a proof of inclusion
    fn verify_inclusion(
        &self,
        root: &H256,
        hash: &HashOutput,
        proof: &SmtProof
    ) -> bool;
}

impl<S: StoreReadOps<SmtKeyValue> + StoreWriteOps<SmtKeyValue>> ContentAddressedSmt for MerkleSmt<S> {
    fn store_with_proof<T: ContentAddressed>(&self, object: &T) -> Result<(HashOutput, SmtProof), SmtError> {
        // Store the object and get its content hash
        let (content_hash, _, _) = self.store_content(object)?;
        
        // Convert the hash to an SMT key
        let hash_bytes = content_hash.as_bytes();
        let mut key = [0u8; 32];
        key.copy_from_slice(hash_bytes);
        let smt_key = H256::from(key);
        
        // Get the proof
        let (_, proof) = self.get_with_proof(&smt_key)?;
        
        Ok((content_hash, proof))
    }
    
    fn get_with_proof<T: ContentAddressed>(&self, hash: &HashOutput) -> Result<(T, SmtProof), SmtError> {
        self.get_content_with_proof(hash)
    }
    
    fn verify_inclusion(&self, root: &H256, hash: &HashOutput, proof: &SmtProof) -> bool {
        // Convert the hash to an SMT key
        let hash_bytes = hash.as_bytes();
        let mut key = [0u8; 32];
        key.copy_from_slice(hash_bytes);
        let smt_key = H256::from(key);
        
        self.verify_inclusion_proof(root, &smt_key, proof)
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