//! Merkle Tree AST Implementation for TEL Combinators
//!
//! This module implements the AST as a Merkle tree structure, where each node
//! has a unique content ID based on its value and the content IDs of its
//! children. This enables efficient verification of expressions and execution
//! paths.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use causality_types::content_addressing::{self, canonical::CanonicalSerializationError};
use causality_types::crypto_primitives::{ContentId};
use std::hash::Hash;
use std::default::Default;
use hex;

use super::Combinator;

/// Direction for Merkle path, indicating whether the target is in the left or right subtree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// The target is in the left subtree
    Left,
    /// The target is in the right subtree
    Right,
}

/// A node in the Merkle tree AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MerkleNode {
    /// Content ID of this node
    pub content_id: ContentId,
    /// The combinator at this node
    pub combinator: Combinator,
    /// Children of this node, if any
    pub children: Vec<MerkleNode>,
    /// Additional metadata for this node
    pub metadata: HashMap<String, String>,
}

/// A path through the Merkle tree, used for verifying execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MerklePath {
    /// Path steps, from root to leaf
    pub steps: Vec<MerklePathStep>,
    /// Content ID of the root node
    pub root_id: ContentId,
    /// Content ID of the target node
    pub target_id: ContentId,
}

/// A single step in a Merkle path
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MerklePathStep {
    /// Content ID of the node at this step
    pub node_id: ContentId,
    /// Direction to go from this node
    pub direction: Direction,
    /// Content IDs of sibling nodes
    pub sibling_ids: Vec<ContentId>,
}

impl MerkleNode {
    /// Create a new Merkle node from a combinator
    pub fn new(combinator: Combinator) -> Result<Self, CanonicalSerializationError> {
        let children = Vec::new();
        let metadata = HashMap::new();
        let content_id = Self::compute_content_id(&combinator, &children, &metadata)?;
        
        Ok(MerkleNode {
            content_id,
            combinator,
            children,
            metadata,
        })
    }
    
    /// Create a new Merkle node with children
    pub fn with_children(
        combinator: Combinator,
        children: Vec<MerkleNode>,
        metadata: HashMap<String, String>,
    ) -> Result<Self, CanonicalSerializationError> {
        let content_id = Self::compute_content_id(&combinator, &children, &metadata)?;
        
        Ok(MerkleNode {
            content_id,
            combinator,
            children,
            metadata,
        })
    }
    
    /// Convert a combinator to a Merkle tree recursively
    pub fn from_combinator(combinator: &Combinator) -> Result<Self, CanonicalSerializationError> {
        match combinator {
            Combinator::App { function, argument } => {
                let f_node = Self::from_combinator(function)?;
                let x_node = Self::from_combinator(argument)?;
                
                Self::with_children(
                    combinator.clone(),
                    vec![f_node, x_node],
                    HashMap::new(),
                )
            },
            Combinator::Effect { effect_name: _, args, core_effect: _ } => {
                let arg_nodes = args
                    .iter()
                    .map(Self::from_combinator)
                    .collect::<Result<Vec<_>, _>>()?;
                
                Self::with_children(
                    combinator.clone(),
                    arg_nodes,
                    HashMap::new(),
                )
            },
            Combinator::StateTransition { target_state: _, fields, resource_id: _ } => {
                let field_nodes = fields
                    .values()
                    .map(Self::from_combinator)
                    .collect::<Result<Vec<_>, _>>()?;
                
                let mut metadata = HashMap::new();
                for (key, _) in fields {
                    metadata.insert(key.clone(), format!("field:{}", key));
                }
                
                Self::with_children(
                    combinator.clone(),
                    field_nodes,
                    metadata,
                )
            },
            Combinator::ContentId(expr) | 
            Combinator::Store(expr) | 
            Combinator::Load(expr) => {
                let expr_node = Self::from_combinator(expr)?;
                
                Self::with_children(
                    combinator.clone(),
                    vec![expr_node],
                    HashMap::new(),
                )
            },
            // Base combinators and literals have no children
            _ => Self::new(combinator.clone()),
        }
    }
    
    /// Compute the content ID for a node
    fn compute_content_id(
        combinator: &Combinator,
        children: &[MerkleNode],
        metadata: &HashMap<String, String>,
    ) -> Result<ContentId, CanonicalSerializationError> {
        // Create a structure representing this node for hashing
        let node_data = MerkleNodeData {
            combinator_type: format!("{:?}", combinator),
            children_ids: children.iter().map(|c| c.content_id.clone()).collect(),
            metadata: metadata.clone(),
        };
        
        // Serialize the node data to JSON
        let serialized = serde_json::to_string(&node_data)
            .map_err(|e| CanonicalSerializationError::JsonError(e.to_string()))?;
        
        // Create a ContentId from the serialized data using the proper content addressing system
        // This integrates with the causality_types content addressing system
        let content_id = ContentId::new(serialized);
        
        Ok(content_id)
    }
    
    /// Find a node by content ID
    pub fn find_by_id(&self, id: &ContentId) -> Option<&MerkleNode> {
        if self.content_id == *id {
            Some(self)
        } else {
            for child in &self.children {
                if let Some(found) = child.find_by_id(id) {
                    return Some(found);
                }
            }
            None
        }
    }
    
    /// Create a Merkle path from this node to a target node
    pub fn create_path(&self, target_id: &ContentId) -> Option<MerklePath> {
        if *target_id == self.content_id {
            // Target is self
            return Some(MerklePath {
                steps: Vec::new(),
                root_id: self.content_id.clone(),
                target_id: target_id.clone(),
            });
        }
        
        // Try to find a path through each child
        for (i, child) in self.children.iter().enumerate() {
            if let Some(path) = child.create_path(target_id) {
                // Found path in this child, create a step and prepend it
                let sibling_ids = self.children
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, sibling)| sibling.content_id.clone())
                    .collect();
                
                let direction = if i == 0 { Direction::Left } else { Direction::Right };
                
                let step = MerklePathStep {
                    node_id: self.content_id.clone(),
                    direction,
                    sibling_ids,
                };
                
                let mut steps = vec![step];
                steps.extend(path.steps);
                
                return Some(MerklePath {
                    steps,
                    root_id: self.content_id.clone(),
                    target_id: target_id.clone(),
                });
            }
        }
        
        // No path found
        None
    }
}

/// Internal structure used for content ID computation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MerkleNodeData {
    /// Type of the combinator (for hashing purposes)
    pub combinator_type: String,
    /// Content IDs of children
    pub children_ids: Vec<ContentId>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl MerklePath {
    /// Verify that this path is valid
    pub fn verify(&self) -> bool {
        // Empty path is only valid if root and target are the same
        if self.steps.is_empty() {
            return self.root_id == self.target_id;
        }
        
        // Check that the first step starts from the root
        if self.steps[0].node_id != self.root_id {
            return false;
        }
        
        // Verify each step in the path connects properly
        for i in 0..self.steps.len() - 1 {
            let current = &self.steps[i];
            let next = &self.steps[i + 1];
            
            // The next node ID should NOT be in sibling_ids (since siblings are all children EXCEPT the one in the path)
            if current.sibling_ids.contains(&next.node_id) {
                return false;
            }
        }
        
        // For the last step, we need to ensure it properly connects to the target
        if let Some(last_step) = self.steps.last() {
            // The target should either be the last node itself (if path terminates at this node)
            // or should NOT be in the siblings (if it's the child in the path direction)
            if last_step.node_id == self.target_id {
                true
            } else {
                !last_step.sibling_ids.contains(&self.target_id)
            }
        } else {
            // No steps - only valid if root is target (but we checked this above)
            true
        }
    }
    
    /// Extend this path with another path
    pub fn extend(&self, other: &MerklePath) -> Option<MerklePath> {
        // Paths can only be extended if the target of self is the root of other
        if self.target_id != other.root_id {
            return None;
        }
        
        let mut steps = self.steps.clone();
        steps.extend(other.steps.clone());
        
        Some(MerklePath {
            steps,
            root_id: self.root_id.clone(),
            target_id: other.target_id.clone(),
        })
    }
}

/// Options for Merkle tree operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MerkleOptions {
    /// Hash algorithm to use
    pub hash_algorithm: HashAlgorithm,
}

/// Hash algorithms supported for Merkle trees
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    /// SHA-256 hash algorithm
    Sha256,
    /// SHA-512 hash algorithm
    Sha512,
    /// Blake2b hash algorithm
    Blake2b,
}

/// Output of a hash function
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashOutput {
    /// The raw bytes of the hash
    data: Vec<u8>,
}

impl HashOutput {
    /// Create a new hash output from bytes
    pub fn new(data: Vec<u8>) -> Self {
        HashOutput { data }
    }
    
    /// Get the bytes of the hash
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }
}

/// Hash a string using the specified algorithm
pub fn hash_string(input: &str, algorithm: HashAlgorithm) -> Result<HashOutput, String> {
    // For now, we'll use a simplified approach that delegates to ContentId
    let content_id = ContentId::new(input.to_string());
    
    // Extract the bytes from the content ID
    // This is a placeholder implementation that needs to be updated with the actual hash implementation
    let hash_bytes = content_id.as_bytes().to_vec();
    
    Ok(HashOutput::new(hash_bytes))
}

impl Default for MerkleOptions {
    fn default() -> Self {
        Self {
            hash_algorithm: HashAlgorithm::Sha256,
        }
    }
}

pub fn compute_merkle_root(expr: &Combinator, options: &MerkleOptions) -> Result<HashOutput, String> {
    match expr {
        Combinator::Literal(lit) => {
            // Serialize the literal to bytes
            let serialized = serde_json::to_string(lit)
                .map_err(|e| format!("Failed to serialize literal: {}", e))?;
            
            // Hash the serialized bytes
            hash_string(&serialized, options.hash_algorithm)
        },
        Combinator::App { function, argument } => {
            // Compute hashes for function and argument
            let f_hash = compute_merkle_root(function, options)?;
            let x_hash = compute_merkle_root(argument, options)?;
            
            // Combine the hashes
            let combined = format!("{}:{}", hex::encode(f_hash.bytes()), hex::encode(x_hash.bytes()));
            hash_string(&combined, options.hash_algorithm)
        },
        // ... handle other Combinator variants ...
        // For simplicity, we're using the same approach for all other variants
        _ => {
            // Serialize the combinator to string and hash it
            let serialized = serde_json::to_string(expr)
                .map_err(|e| format!("Failed to serialize combinator: {}", e))?;
            
            hash_string(&serialized, options.hash_algorithm)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combinators::{Combinator, Literal};
    
    #[test]
    fn test_merkle_node_creation() {
        // Create a simple combinator
        let combinator = Combinator::I;
        
        // Create a Merkle node
        let node = MerkleNode::new(combinator.clone()).unwrap();
        
        // Check properties
        assert_eq!(node.combinator, combinator);
        assert!(node.children.is_empty());
        assert!(node.metadata.is_empty());
        assert!(node.content_id.to_string().len() > 0);
    }
    
    #[test]
    fn test_merkle_tree_from_combinator() {
        // Create a more complex combinator
        let combinator = Combinator::app(
            Combinator::app(Combinator::S, Combinator::K),
            Combinator::app(Combinator::I, Combinator::Literal(Literal::Int(42)))
        );
        
        // Convert to Merkle tree
        let merkle_tree = MerkleNode::from_combinator(&combinator).unwrap();
        
        // Check structure
        assert_eq!(merkle_tree.combinator, combinator);
        assert_eq!(merkle_tree.children.len(), 2);
        assert_eq!(merkle_tree.children[0].children.len(), 2);
        assert_eq!(merkle_tree.children[1].children.len(), 2);
    }
    
    #[test]
    fn test_merkle_path_creation_and_verification() {
        // Create a complex combinator
        let combinator = Combinator::app(
            Combinator::app(Combinator::S, Combinator::K),
            Combinator::app(Combinator::I, Combinator::Literal(Literal::Int(42)))
        );
        
        // Convert to Merkle tree
        let merkle_tree = MerkleNode::from_combinator(&combinator).unwrap();
        
        // Target is the integer literal node
        let target_id = merkle_tree.children[1].children[1].content_id.clone();
        
        // Create path
        let path = merkle_tree.create_path(&target_id).unwrap();
        
        // Verify path
        assert!(path.verify());
        assert_eq!(path.root_id, merkle_tree.content_id);
        assert_eq!(path.target_id, target_id);
    }
} 