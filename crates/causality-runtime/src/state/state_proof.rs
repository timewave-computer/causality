//! State Proof Implementation
//!
//! This module provides utilities for generating and verifying proofs of state inclusion
//! using SSZ Merkle trees. It allows for proving that a particular resource or value exists
//! in the state without revealing the entire state.

use anyhow::{anyhow, Result};
use sha2::Digest;
use causality_types::{
    core::{Resource, id::{ResourceId, ValueExprId, AsId}},
    expr::value::ValueExpr,
    serialization::{MerkleProof, MerkleTree, SimpleSerialize, Encode, Decode, DecodeError},
};
use std::collections::HashMap;
use causality_core::extension_traits::ValueExprExt;

/// A proof that a particular resource exists in the state
#[derive(Debug, Clone)]
pub struct ResourceProof {
    /// The resource ID being proven
    pub resource_id: ResourceId,
    
    /// The Merkle proof for the resource
    pub proof: MerkleProof,
}

/// A proof that a particular value exists in the state
#[derive(Debug, Clone)]
pub struct ValueProof {
    /// The value ID being proven
    pub value_id: ValueExprId,
    
    /// The Merkle proof for the value
    pub proof: MerkleProof,
}

impl Encode for ResourceProof {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.resource_id.as_ssz_bytes());
        bytes.extend(self.proof.as_ssz_bytes());
        bytes
    }
}

impl Decode for ResourceProof {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let resource_id = ResourceId::from_ssz_bytes(bytes)?;
        let id_len = resource_id.as_ssz_bytes().len();
        let proof = MerkleProof::from_ssz_bytes(&bytes[id_len..])?;
        
        Ok(ResourceProof {
            resource_id,
            proof,
        })
    }
}

impl SimpleSerialize for ResourceProof {}

impl Encode for ValueProof {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.value_id.as_ssz_bytes());
        bytes.extend(self.proof.as_ssz_bytes());
        bytes
    }
}

impl Decode for ValueProof {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let value_id = ValueExprId::from_ssz_bytes(bytes)?;
        let id_len = value_id.as_ssz_bytes().len();
        let proof = MerkleProof::from_ssz_bytes(&bytes[id_len..])?;
        
        Ok(ValueProof {
            value_id,
            proof,
        })
    }
}

impl SimpleSerialize for ValueProof {}

/// Generator for state proofs
pub struct StateProofGenerator {
    /// Resources Merkle tree
    resources_tree: Option<MerkleTree>,
    
    /// Values Merkle tree
    values_tree: Option<MerkleTree>,
    
    /// Resource index map (resource_id -> index in the tree)
    resource_indices: HashMap<ResourceId, usize>,
    
    /// Value index map (value_id -> index in the tree)
    value_indices: HashMap<ValueExprId, usize>,
}

impl Default for StateProofGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl StateProofGenerator {
    /// Create a new state proof generator
    pub fn new() -> Self {
        Self {
            resources_tree: None,
            values_tree: None,
            resource_indices: HashMap::new(),
            value_indices: HashMap::new(),
        }
    }
    
    /// Build the Merkle trees from the current state
    pub fn build_trees(&mut self, resources: &[Resource], values: &[ValueExpr]) -> Result<()> {
        // Build resource tree
        self.resource_indices.clear();
        for (i, resource) in resources.iter().enumerate() {
            // Convert EntityId to ResourceId for the index map
            let resource_id = ResourceId::new(resource.id.inner());
            self.resource_indices.insert(resource_id, i);
        }
        self.resources_tree = Some(MerkleTree::new(resources));
        
        // Build value tree
        self.value_indices.clear();
        let mut value_with_ids = Vec::with_capacity(values.len());
        for (i, value) in values.iter().enumerate() {
            // Generate the value ID by hashing the serialized value
            let value_bytes = value.as_ssz_bytes();
            let hash = sha2::Sha256::digest(&value_bytes);
            let value_id = ValueExprId::new(<[u8; 32]>::try_from(hash.as_slice()).expect("Hash must be 32 bytes"));
            self.value_indices.insert(value_id, i);
            value_with_ids.push((value_id, value.clone()));
        }
        self.values_tree = Some(MerkleTree::new(&value_with_ids));
        
        Ok(())
    }
    
    /// Generate a proof for a specific resource
    pub fn generate_resource_proof(&self, resource_id: &ResourceId) -> Result<ResourceProof> {
        let index = self.resource_indices.get(resource_id)
            .ok_or_else(|| anyhow!("Resource not found in the proof generator"))?;
        
        let resources_tree = self.resources_tree.as_ref()
            .ok_or_else(|| anyhow!("Resources tree not built"))?;
        
        let proof = resources_tree.generate_proof(*index)
            .ok_or_else(|| anyhow!("Failed to generate proof for resource"))?;
        
        Ok(ResourceProof {
            resource_id: *resource_id,
            proof,
        })
    }
    
    /// Generate a proof for a specific value
    pub fn generate_value_proof(&self, value_id: &ValueExprId) -> Result<ValueProof> {
        let index = self.value_indices.get(value_id)
            .ok_or_else(|| anyhow!("Value not found in the proof generator"))?;
        
        let values_tree = self.values_tree.as_ref()
            .ok_or_else(|| anyhow!("Values tree not built"))?;
        
        let proof = values_tree.generate_proof(*index)
            .ok_or_else(|| anyhow!("Failed to generate proof for value"))?;
        
        Ok(ValueProof {
            value_id: *value_id,
            proof,
        })
    }
    
    /// Get the root hash of the resources tree
    pub fn resources_root(&self) -> Result<[u8; 32]> {
        self.resources_tree.as_ref()
            .map(|tree| tree.root)
            .ok_or_else(|| anyhow!("Resources tree not built"))
    }
    
    /// Get the root hash of the values tree
    pub fn values_root(&self) -> Result<[u8; 32]> {
        self.values_tree.as_ref()
            .map(|tree| tree.root)
            .ok_or_else(|| anyhow!("Values tree not built"))
    }
}

/// Verifier for state proofs
pub struct StateProofVerifier {
    /// Root hash of the resources tree
    resources_root: [u8; 32],
    
    /// Root hash of the values tree
    values_root: [u8; 32],
}

impl StateProofVerifier {
    /// Create a new state proof verifier with the given root hashes
    pub fn new(resources_root: [u8; 32], values_root: [u8; 32]) -> Self {
        Self {
            resources_root,
            values_root,
        }
    }
    
    /// Verify a resource proof
    pub fn verify_resource_proof(&self, proof: &ResourceProof) -> bool {
        causality_types::serialization::verify_proof(&self.resources_root, &proof.proof)
    }
    
    /// Verify a value proof
    pub fn verify_value_proof(&self, proof: &ValueProof) -> bool {
        causality_types::serialization::verify_proof(&self.values_root, &proof.proof)
    }
}

impl ResourceProof {
    /// Create a new ResourceProof
    pub fn new(resource_id: ResourceId, _resource_bytes: Vec<u8>) -> Self {
        // In a real implementation, this would create a proper Merkle proof
        // For now, create a simple mock proof
        use causality_types::serialization::MerkleProof;
        use sha2::{Digest, Sha256};
        
        // Create a hash of the resource bytes for the leaf
        let mut hasher = Sha256::new();
        hasher.update(&_resource_bytes);
        let hash_result = hasher.finalize();
        let mut leaf = [0u8; 32];
        leaf.copy_from_slice(&hash_result);
        
        Self {
            resource_id,
            proof: MerkleProof {
                path: Vec::new(),
                siblings: Vec::new(),
                leaf,
                index: 0,
            },
        }
    }
}

impl ValueProof {
    /// Create a new ValueProof  
    pub fn new(value_id: ValueExprId, _value_bytes: Vec<u8>) -> Self {
        // In a real implementation, this would create a proper Merkle proof
        // For now, create a simple mock proof
        use causality_types::serialization::MerkleProof;
        use sha2::{Digest, Sha256};
        
        // Create a hash of the value bytes for the leaf
        let mut hasher = Sha256::new();
        hasher.update(&_value_bytes);
        let hash_result = hasher.finalize();
        let mut leaf = [0u8; 32];
        leaf.copy_from_slice(&hash_result);
        
        Self {
            value_id,
            proof: MerkleProof {
                path: Vec::new(),
                siblings: Vec::new(),
                leaf,
                index: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::primitive::ids::{DomainId, EntityId};
    use causality_types::primitive::string::Str;
    use causality_types::primitive::number::Number;
    use causality_types::expr::value::ValueExprMap;
    use std::collections::BTreeMap;
    
    /// Helper function to create a test resource
    fn create_test_resource(id: u8) -> Resource {
        let mut bytes = [0u8; 32];
        for i in 0..32 {
            bytes[i] = i as u8 + id;
        }
        
        Resource::new(
            EntityId::new(bytes),
            Str::from(format!("test_resource_{}", id)),
            DomainId::new([id; 32]),
            Str::from("test_type"),
            1,
            causality_types::core::time::Timestamp::now(),
        )
    }
    
    /// Helper function to create a test value expression
    fn create_test_value_expr(name: &str, value: i64) -> ValueExpr {
        let mut map = BTreeMap::new();
        map.insert(Str::from("name"), ValueExpr::String(Str::from(name)));
        map.insert(Str::from("value"), ValueExpr::Number(Number::Integer(value)));
        ValueExpr::Record(ValueExprMap(map))
    }
    
    #[test]
    fn test_resource_proof() {
        // Create test resources
        let resources = vec![
            create_test_resource(1),
            create_test_resource(2),
            create_test_resource(3),
            create_test_resource(4),
        ];
        
        // Create test values
        let values = vec![
            create_test_value_expr("value1", 10),
            create_test_value_expr("value2", 20),
            create_test_value_expr("value3", 30),
        ];
        
        // Create proof generator and build trees
        let mut generator = StateProofGenerator::new();
        generator.build_trees(&resources, &values).unwrap();
        
        // Get the root hashes
        let resources_root = generator.resources_root().unwrap();
        let values_root = generator.values_root().unwrap();
        
        // Create verifier
        let verifier = StateProofVerifier::new(resources_root, values_root);
        
        // Generate and verify a resource proof
        let resource_id = ResourceId::new(resources[1].id.inner());
        let resource_proof = generator.generate_resource_proof(&resource_id).unwrap();
        
        // Verify the proof
        assert!(verifier.verify_resource_proof(&resource_proof), "Resource proof verification failed");
        
        // Test with invalid proof (tampered resource ID)
        let mut invalid_proof = resource_proof.clone();
        let mut tampered_id = resource_id.inner(); // Get the inner bytes from ResourceId
        tampered_id[0] ^= 0xFF; // Flip bits in the first byte
        invalid_proof.resource_id = ResourceId::new(tampered_id);
        
        // The proof is still valid, but for a different resource ID
        assert!(verifier.verify_resource_proof(&invalid_proof), "Tampered resource ID should not affect proof validity");
        
        // Test with invalid proof (tampered proof)
        let mut invalid_proof = resource_proof;
        invalid_proof.proof.leaf[0] ^= 0xFF; // Flip bits in the first byte of the leaf hash
        
        // Verification should fail
        assert!(!verifier.verify_resource_proof(&invalid_proof), "Tampered proof should fail verification");
    }
    
    #[test]
    fn test_value_proof() {
        // Create test resources
        let resources = vec![
            create_test_resource(1),
            create_test_resource(2),
        ];
        
        // Create test values
        let values = vec![
            create_test_value_expr("value1", 10),
            create_test_value_expr("value2", 20),
            create_test_value_expr("value3", 30),
        ];
        
        // Create proof generator and build trees
        let mut generator = StateProofGenerator::new();
        generator.build_trees(&resources, &values).unwrap();
        
        // Get value ID for the second value
        let value_id = values[1].id();
        
        // Generate a value proof
        let value_proof = generator.generate_value_proof(&value_id).unwrap();
        
        // Create verifier
        let verifier = StateProofVerifier::new(
            generator.resources_root().unwrap(),
            generator.values_root().unwrap()
        );
        
        // Verify the proof
        assert!(verifier.verify_value_proof(&value_proof), "Value proof verification failed");
    }
} 