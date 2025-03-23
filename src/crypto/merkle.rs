// Merkle tree implementation for creating and verifying commitments to data
//
// This module provides a Merkle tree-based commitment scheme, which allows
// efficient proofs of inclusion without revealing the entire dataset.

use std::fmt;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

pub use sparse_merkle_tree::{H256, MerkleProof};
use sparse_merkle_tree::default_store::DefaultStore;
use sparse_merkle_tree::traits::{StoreReadOps, StoreWriteOps};

use crate::smt::{SmtFactory, MerkleSmt, SmtError, SmtKeyValue};
use crate::crypto::hash::HashFactory;

/// Types of commitment schemes available
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommitmentType {
    /// Merkle tree commitment scheme
    MerkleTree,
    /// Vector commitment scheme
    VectorCommitment,
    /// Polynomial commitment scheme
    PolynomialCommitment,
}

impl fmt::Display for CommitmentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MerkleTree => write!(f, "MerkleTree"),
            Self::VectorCommitment => write!(f, "VectorCommitment"),
            Self::PolynomialCommitment => write!(f, "PolynomialCommitment"),
        }
    }
}

/// A commitment represents a binding commitment to data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commitment {
    /// The commitment data (typically a hash or root hash)
    data: Vec<u8>,
}

impl Commitment {
    /// Create a new commitment from raw data
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Get the commitment data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    
    /// Convert the commitment to a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.data())
    }
    
    /// Create a commitment from a hex string
    pub fn from_hex(hex_str: &str) -> Result<Self, CommitmentError> {
        let data = hex::decode(hex_str)
            .map_err(|_| CommitmentError::InvalidFormat("Invalid hex format".to_string()))?;
        Ok(Self::new(data))
    }
}

/// Error type for commitment operations
#[derive(Debug, Error)]
pub enum CommitmentError {
    /// Invalid commitment scheme type
    #[error("Invalid commitment scheme type: {0}")]
    InvalidType(String),
    
    /// Object not found
    #[error("Object not found: {0}")]
    ObjectNotFound(String),
    
    /// Invalid commitment
    #[error("Invalid commitment")]
    InvalidCommitment,
    
    /// Verification failed
    #[error("Verification failed")]
    VerificationFailed,
    
    /// Invalid format
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    /// SMT error
    #[error("SMT error: {0}")]
    SmtError(#[from] SmtError),
    
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// A trait for commitment schemes
pub trait CommitmentScheme: Send + Sync {
    /// Get the type of this commitment scheme
    fn scheme_type(&self) -> CommitmentType;
    
    /// Commit to a single object
    fn commit(&self, object_id: &str, data: &[u8]) -> Result<Commitment, CommitmentError>;
    
    /// Commit to a batch of objects
    fn commit_batch(&self, objects: &HashMap<String, Vec<u8>>) -> Result<Commitment, CommitmentError>;
    
    /// Verify an object against a commitment
    fn verify(&self, commitment: &Commitment, object_id: &str, data: &[u8]) -> Result<bool, CommitmentError>;
    
    /// Reset the commitment scheme (clear all state)
    fn reset(&self) -> Result<(), CommitmentError>;
}

/// A commitment scheme based on a Merkle Tree.
///
/// This implementation uses a Sparse Merkle Tree to create cryptographic
/// commitments to a set of values. It allows efficient inclusion proofs
/// without revealing the entire dataset.
pub struct MerkleTreeCommitmentScheme {
    /// The underlying Sparse Merkle Tree
    smt: Arc<MerkleSmt<DefaultStore<H256>>>,
    /// Map to track object IDs to their values
    object_map: RwLock<HashMap<String, H256>>,
}

impl MerkleTreeCommitmentScheme {
    /// Create a new MerkleTreeCommitmentScheme
    pub fn new() -> Result<Self, CommitmentError> {
        let smt_factory = SmtFactory::default();
        Ok(Self {
            smt: smt_factory.create_memory_smt().map_err(|e| CommitmentError::Other(e.to_string()))?,
            object_map: RwLock::new(HashMap::new()),
        })
    }
    
    /// Create a key for the SMT from an object ID
    fn create_key(&self, object_id: &str) -> Result<H256, CommitmentError> {
        MerkleSmt::<DefaultStore<H256>>::create_key(object_id.as_bytes())
            .map_err(|e| CommitmentError::Other(format!("Failed to create key: {}", e)))
    }
    
    /// Create a value for the SMT from object data
    fn create_value(&self, data: &[u8]) -> Result<SmtKeyValue, CommitmentError> {
        MerkleSmt::<DefaultStore<H256>>::create_value(data)
            .map_err(|e| CommitmentError::Other(format!("Failed to create value: {}", e)))
    }
}

impl CommitmentScheme for MerkleTreeCommitmentScheme {
    fn scheme_type(&self) -> CommitmentType {
        CommitmentType::MerkleTree
    }
    
    fn commit(&self, object_id: &str, data: &[u8]) -> Result<Commitment, CommitmentError> {
        // Create a key and value for the Merkle tree
        let key = self.create_key(object_id)?;
        let value = self.create_value(data)?;
        
        // Insert into the SMT
        self.smt.insert(key, value)
            .map_err(|e| CommitmentError::Other(format!("SMT insert error: {}", e)))?;
        
        // Add to object map
        let mut object_map = self.object_map.write().unwrap();
        object_map.insert(object_id.to_string(), key);
        
        // Return the root hash as the commitment
        let root = self.smt.root()
            .map_err(|e| CommitmentError::Other(format!("Failed to get root: {}", e)))?;
        
        Ok(Commitment::new(root.as_slice().to_vec()))
    }
    
    fn commit_batch(&self, objects: &HashMap<String, Vec<u8>>) -> Result<Commitment, CommitmentError> {
        // Insert all objects into the SMT
        for (object_id, data) in objects {
            let key = self.create_key(object_id)?;
            let value = self.create_value(data)?;
            
            // Insert into the SMT
            self.smt.insert(key, value)
                .map_err(|e| CommitmentError::Other(format!("SMT insert error: {}", e)))?;
            
            // Add to object map
            let mut object_map = self.object_map.write().unwrap();
            object_map.insert(object_id.to_string(), key);
        }
        
        // Return the root hash as the commitment
        let root = self.smt.root()
            .map_err(|e| CommitmentError::Other(format!("Failed to get root: {}", e)))?;
        
        Ok(Commitment::new(root.as_slice().to_vec()))
    }
    
    fn verify(&self, commitment: &Commitment, object_id: &str, data: &[u8]) -> Result<bool, CommitmentError> {
        // Get the key from the object map
        let object_map = self.object_map.read().unwrap();
        let key = match object_map.get(object_id) {
            Some(k) => *k,
            None => {
                // If we don't have the key in our map, create it from the object ID
                self.create_key(object_id)?
            }
        };
        
        // Create the expected value
        let expected_value = self.create_value(data)?;
        
        // Generate a proof
        let proof = self.smt.merkle_proof(&key)
            .map_err(|e| CommitmentError::Other(format!("Failed to generate proof: {}", e)))?;
        
        // Get the root hash from the commitment
        let root_bytes = commitment.data();
        let mut root = [0u8; 32];
        if root_bytes.len() != 32 {
            return Err(CommitmentError::InvalidCommitment);
        }
        root.copy_from_slice(root_bytes);
        let root = H256::from(root);
        
        // Verify the proof
        self.smt.verify_proof(&root, &key, &expected_value, &proof)
            .map_err(|e| CommitmentError::Other(format!("Failed to verify proof: {}", e)))
    }
    
    fn reset(&self) -> Result<(), CommitmentError> {
        // Reset the SMT
        self.smt.reset()
            .map_err(|e| CommitmentError::Other(format!("Failed to reset SMT: {}", e)))?;
        
        // Clear the object map
        let mut object_map = self.object_map.write().unwrap();
        object_map.clear();
        
        Ok(())
    }
}

/// Factory for creating commitment schemes
#[derive(Clone)]
pub struct CommitmentFactory {
    hash_factory: HashFactory,
}

impl CommitmentFactory {
    /// Create a new commitment factory
    pub fn new(hash_factory: HashFactory) -> Self {
        Self { hash_factory }
    }
    
    /// Create a new commitment factory with default settings
    pub fn default() -> Self {
        Self::new(HashFactory::default())
    }
    
    /// Create a commitment scheme of the specified type
    pub fn create_scheme(&self, scheme_type: CommitmentType) -> Result<Box<dyn CommitmentScheme>, CommitmentError> {
        match scheme_type {
            CommitmentType::MerkleTree => {
                let scheme = MerkleTreeCommitmentScheme::new()?;
                Ok(Box::new(scheme))
            },
            CommitmentType::VectorCommitment => {
                Err(CommitmentError::InvalidType("Vector commitment not implemented".to_string()))
            },
            CommitmentType::PolynomialCommitment => {
                Err(CommitmentError::InvalidType("Polynomial commitment not implemented".to_string()))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_commitment_creation() {
        let data = vec![1, 2, 3, 4];
        let commitment = Commitment::new(data.clone());
        
        assert_eq!(commitment.data(), data.as_slice());
    }
    
    #[test]
    fn test_commitment_factory() {
        let factory = CommitmentFactory::default();
        
        // Should be able to create a Merkle tree commitment scheme
        let scheme = factory.create_scheme(CommitmentType::MerkleTree);
        assert!(scheme.is_ok());
        
        // Vector commitment should return an error
        let scheme = factory.create_scheme(CommitmentType::VectorCommitment);
        assert!(scheme.is_err());
        
        // Polynomial commitment should return an error
        let scheme = factory.create_scheme(CommitmentType::PolynomialCommitment);
        assert!(scheme.is_err());
    }
    
    #[test]
    fn test_merkle_tree_commitment_basic() -> Result<(), CommitmentError> {
        let scheme = MerkleTreeCommitmentScheme::new()?;
        
        // Commit to a single data item
        let data = b"test data";
        let commitment = scheme.commit("test_object", data)?;
        
        // Verify the commitment
        assert!(scheme.verify(&commitment, "test_object", data)?);
        
        // Verify against different data (should fail)
        let other_data = b"other data";
        assert!(!scheme.verify(&commitment, "test_object", other_data)?);
        
        Ok(())
    }
    
    #[test]
    fn test_merkle_tree_commitment_batch() -> Result<(), CommitmentError> {
        let scheme = MerkleTreeCommitmentScheme::new()?;
        
        // Commit to multiple data items
        let mut objects = HashMap::new();
        objects.insert("object1".to_string(), b"data1".to_vec());
        objects.insert("object2".to_string(), b"data2".to_vec());
        objects.insert("object3".to_string(), b"data3".to_vec());
        
        let commitment = scheme.commit_batch(&objects)?;
        
        // Verify each item
        for (object_id, data) in &objects {
            assert!(scheme.verify(&commitment, object_id, data)?);
        }
        
        // Verify against a different item (should fail)
        let other_data = b"other data".to_vec();
        assert!(!scheme.verify(&commitment, "object1", &other_data)?);
        
        Ok(())
    }
    
    #[test]
    fn test_merkle_tree_commitment_reset() -> Result<(), CommitmentError> {
        let scheme = MerkleTreeCommitmentScheme::new()?;
        
        // Commit to data
        let data = b"test data";
        let commitment = scheme.commit("test_object", data)?;
        
        // Verify the commitment
        assert!(scheme.verify(&commitment, "test_object", data)?);
        
        // Reset the scheme
        scheme.reset()?;
        
        // After reset, verification should fail
        assert!(!scheme.verify(&commitment, "test_object", data)?);
        
        Ok(())
    }
} 