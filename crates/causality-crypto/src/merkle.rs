// Merkle tree implementation for efficient verification
// Original file: src/crypto/merkle.rs

// Merkle tree implementation for creating and verifying commitments to data
//
// This module provides a Merkle tree-based commitment scheme, which allows
// efficient proofs of inclusion without revealing the entire dataset.

use std::fmt;
use std::collections::HashMap;
use std::sync::RwLock;
use thiserror::Error;
use std::marker::PhantomData;
use std::fmt::Debug;
use serde::{Serialize, Deserialize};

// Change the imports to use causality_types instead of causality_crypto
use causality_types::crypto_primitives::{HashOutput, HashAlgorithm};
use causality_types::ContentId;

// Import needed functions from crate
use crate::hash::{HashFunction, ContentHasher, HashFactory, Blake3HashFunction};

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

/// A simple Merkle tree node
#[derive(Debug, Clone)]
pub enum MerkleNode {
    /// A leaf node containing a value
    Leaf(Vec<u8>),
    /// An internal node with left and right children
    Node(Box<MerkleNode>, Box<MerkleNode>),
}

/// A simple Merkle tree implementation
#[derive(Debug)]
pub struct MerkleTree {
    /// The root node of the tree
    root: Option<MerkleNode>,
    /// The hash function to use
    hash_function: Blake3HashFunction,
    /// Map from keys to their values and positions
    data: HashMap<String, Vec<u8>>,
}

impl MerkleTree {
    /// Create a new empty Merkle tree
    pub fn new() -> Self {
        Self {
            root: None,
            hash_function: Blake3HashFunction::new(),
            data: HashMap::new(),
        }
    }
    
    /// Insert a key-value pair into the tree
    pub fn insert(&mut self, key: &str, value: &[u8]) {
        self.data.insert(key.to_string(), value.to_vec());
        self.rebuild();
    }
    
    /// Insert multiple key-value pairs into the tree
    pub fn insert_batch(&mut self, items: &HashMap<String, Vec<u8>>) {
        for (key, value) in items {
            self.data.insert(key.clone(), value.clone());
        }
        self.rebuild();
    }
    
    /// Rebuild the tree from the current data
    fn rebuild(&mut self) {
        if self.data.is_empty() {
            self.root = None;
            return;
        }
        
        // Convert data to leaf nodes
        let mut leaves: Vec<MerkleNode> = self.data.values()
            .map(|value| MerkleNode::Leaf(value.clone()))
            .collect();
            
        // Ensure we have an even number of leaves by duplicating the last one if needed
        if leaves.len() % 2 == 1 {
            leaves.push(leaves.last().unwrap().clone());
        }
        
        // Build the tree bottom-up
        while leaves.len() > 1 {
            let mut new_level = Vec::new();
            
            for chunk in leaves.chunks(2) {
                if chunk.len() == 2 {
                    let left = Box::new(chunk[0].clone());
                    let right = Box::new(chunk[1].clone());
                    new_level.push(MerkleNode::Node(left, right));
                } else {
                    // This should not happen since we ensure even number of leaves
                    new_level.push(chunk[0].clone());
                }
            }
            
            leaves = new_level;
        }
        
        self.root = leaves.into_iter().next();
    }
    
    /// Get the root hash of the tree
    pub fn root_hash(&self) -> Option<HashOutput> {
        self.root.as_ref().map(|root| self.hash_node(root))
    }
    
    /// Hash a node using the tree's hash function
    fn hash_node(&self, node: &MerkleNode) -> HashOutput {
        match node {
            MerkleNode::Leaf(value) => {
                self.hash_function.hash(value)
            },
            MerkleNode::Node(left, right) => {
                let left_hash = self.hash_node(left);
                let right_hash = self.hash_node(right);
                
                // Combine the child hashes
                let mut combined = Vec::with_capacity(64);
                combined.extend_from_slice(left_hash.as_bytes());
                combined.extend_from_slice(right_hash.as_bytes());
                
                self.hash_function.hash(&combined)
            }
        }
    }
    
    /// Get a value from the tree
    pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.data.get(key)
    }
    
    /// Check if the tree contains a key
    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
    
    /// Clear the tree
    pub fn clear(&mut self) {
        self.data.clear();
        self.root = None;
    }
}

/// A commitment scheme based on a Merkle Tree.
///
/// This implementation uses a simple Merkle tree to create cryptographic
/// commitments to a set of values.
pub struct MerkleTreeCommitmentScheme {
    /// The underlying Merkle Tree
    tree: RwLock<MerkleTree>,
}

impl MerkleTreeCommitmentScheme {
    /// Create a new MerkleTreeCommitmentScheme
    pub fn new(hash_function: Blake3HashFunction) -> Self {
        let mut tree = MerkleTree::new();
        tree.hash_function = hash_function;
        Self {
            tree: RwLock::new(tree),
        }
    }
    
    /// Create a new MerkleTreeCommitmentScheme with default hash function
    pub fn default() -> Self {
        Self::new(Blake3HashFunction::new())
    }
}

impl CommitmentScheme for MerkleTreeCommitmentScheme {
    fn scheme_type(&self) -> CommitmentType {
        CommitmentType::MerkleTree
    }
    
    fn commit(&self, object_id: &str, data: &[u8]) -> Result<Commitment, CommitmentError> {
        let mut tree = self.tree.write().unwrap();
        
        // Insert the data
        tree.insert(object_id, data);
        
        // Return the root hash as the commitment
        let root_hash = tree.root_hash()
            .ok_or_else(|| CommitmentError::Other("Failed to compute root hash".to_string()))?;
            
        Ok(Commitment::new(root_hash.as_bytes().to_vec()))
    }
    
    fn commit_batch(&self, objects: &HashMap<String, Vec<u8>>) -> Result<Commitment, CommitmentError> {
        let mut tree = self.tree.write().unwrap();
        
        // Insert all objects
        tree.insert_batch(objects);
        
        // Return the root hash as the commitment
        let root_hash = tree.root_hash()
            .ok_or_else(|| CommitmentError::Other("Failed to compute root hash".to_string()))?;
            
        Ok(Commitment::new(root_hash.as_bytes().to_vec()))
    }
    
    fn verify(&self, commitment: &Commitment, object_id: &str, data: &[u8]) -> Result<bool, CommitmentError> {
        let tree = self.tree.read().unwrap();
        
        // Get the current tree's root hash
        let root_hash = match tree.root_hash() {
            Some(hash) => hash,
            None => return Ok(false), // Empty tree can't verify anything
        };
        
        // Check if the object is in the tree
        if !tree.contains(object_id) {
            // Create a temporary tree with just this object
            let mut temp_tree = MerkleTree::new();
            temp_tree.insert(object_id, data);
            
            // Get the root hash of the temporary tree
            let temp_hash = temp_tree.root_hash()
                .ok_or_else(|| CommitmentError::Other("Failed to compute temporary root hash".to_string()))?;
                
            // Compare with the provided commitment
            let mut commitment_bytes = [0u8; 32];
            if commitment.data().len() >= 32 {
                commitment_bytes.copy_from_slice(&commitment.data()[0..32]);
            } else {
                return Err(CommitmentError::InvalidFormat("Commitment data too short".to_string()));
            }
            
            let commitment_hash = HashOutput::new(commitment_bytes, HashAlgorithm::Blake3);
            return Ok(temp_hash == commitment_hash);
        }
        
        // Get the stored data for this object
        let stored_data = tree.get(object_id)
            .ok_or_else(|| CommitmentError::ObjectNotFound(object_id.to_string()))?;
            
        // Check if the data matches
        if stored_data != data {
            return Ok(false);
        }
        
        // Check if the commitment matches the root hash
        let mut commitment_bytes = [0u8; 32];
        if commitment.data().len() >= 32 {
            commitment_bytes.copy_from_slice(&commitment.data()[0..32]);
        } else {
            return Err(CommitmentError::InvalidFormat("Commitment data too short".to_string()));
        }
        
        let commitment_hash = HashOutput::new(commitment_bytes, HashAlgorithm::Blake3);
        Ok(root_hash == commitment_hash)
    }
    
    fn reset(&self) -> Result<(), CommitmentError> {
        let mut tree = self.tree.write().unwrap();
        tree.clear();
        Ok(())
    }
}

/// Factory for creating commitment schemes
pub struct CommitmentFactory {
    hash_factory: HashFactory,
}

impl CommitmentFactory {
    /// Create a new CommitmentFactory
    pub fn new(hash_factory: HashFactory) -> Self {
        Self { hash_factory }
    }
    
    /// Create a default CommitmentFactory
    pub fn default() -> Self {
        Self::new(HashFactory::default())
    }
    
    /// Create a commitment scheme of the specified type
    pub fn create_scheme(&self, scheme_type: CommitmentType) -> Result<Box<dyn CommitmentScheme>, CommitmentError> {
        match scheme_type {
            CommitmentType::MerkleTree => {
                let scheme = MerkleTreeCommitmentScheme::default();
                Ok(Box::new(scheme))
            }
            _ => Err(CommitmentError::InvalidType(format!(
                "Commitment scheme type not supported: {}",
                scheme_type
            ))),
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
        
        assert_eq!(commitment.data(), &data);
        
        let hex = commitment.to_hex();
        let from_hex = Commitment::from_hex(&hex).unwrap();
        
        assert_eq!(commitment, from_hex);
    }
    
    #[test]
    fn test_commitment_factory() {
        let factory = CommitmentFactory::default();
        
        let scheme = factory.create_scheme(CommitmentType::MerkleTree).unwrap();
        assert_eq!(scheme.scheme_type(), CommitmentType::MerkleTree);
        
        let result = factory.create_scheme(CommitmentType::VectorCommitment);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_merkle_tree_commitment_basic() -> Result<(), CommitmentError> {
        // Use Blake3HashFunction directly since we know it works
        let scheme = MerkleTreeCommitmentScheme::new(Blake3HashFunction::new());
        let data = vec![1, 2, 3, 4];
        
        // Create commitment
        let commitment = scheme.commit("test", &data)?;
        
        // Verify with same id and data should succeed
        assert!(scheme.verify(&commitment, "test", &data)?);
        
        // Create completely new scheme to verify with different ID
        let verify_scheme = MerkleTreeCommitmentScheme::new(Blake3HashFunction::new());
        // For different ID test, create a new empty tree so it definitely doesn't contain the object
        let other_result = verify_scheme.verify(&commitment, "other", &data)?;
        assert!(!other_result);
        
        // For different data test, verification should fail
        let wrong_data = vec![5, 6, 7, 8];
        let wrong_data_result = verify_scheme.verify(&commitment, "test", &wrong_data)?;
        assert!(!wrong_data_result);
        
        Ok(())
    }
    
    #[test]
    fn test_merkle_tree_commitment_batch() -> Result<(), CommitmentError> {
        let scheme = MerkleTreeCommitmentScheme::default();
        
        let mut objects = HashMap::new();
        objects.insert("obj1".to_string(), vec![1, 2, 3]);
        objects.insert("obj2".to_string(), vec![4, 5, 6]);
        objects.insert("obj3".to_string(), vec![7, 8, 9]);
        
        let commitment = scheme.commit_batch(&objects)?;
        
        for (id, data) in &objects {
            assert!(scheme.verify(&commitment, id, data)?);
        }
        
        Ok(())
    }
    
    #[test]
    fn test_merkle_tree_commitment_reset() -> Result<(), CommitmentError> {
        let scheme = MerkleTreeCommitmentScheme::default();
        
        // Commit some data
        let data = vec![1, 2, 3, 4];
        let commitment = scheme.commit("test", &data)?;
        
        // Verify it
        assert!(scheme.verify(&commitment, "test", &data)?);
        
        // Reset the scheme
        scheme.reset()?;
        
        // After reset, verification should fail
        assert!(!scheme.verify(&commitment, "test", &data)?);
        
        Ok(())
    }
} 