//! SSZ-Compatible Circuit Inputs
//!
//! This module provides utilities for creating and working with SSZ-serialized
//! inputs for ZK circuits. It ensures consistent serialization and hashing
//! between the runtime and ZK environments.

use causality_types::anyhow::{anyhow, Result};
use causality_types::{
    core::id::AsId,
    resource::Resource,
    expr::value::ValueExpr,
    serialization::{Encode, Decode},
};
use sha2::{Digest, Sha256};

/// Represents a circuit input that has been serialized with SSZ
#[derive(Debug, Clone)]
pub struct SszCircuitInput {
    /// The SSZ-serialized bytes of the input
    pub serialized_bytes: Vec<u8>,
    
    /// The hash of the serialized bytes (used for Merkle proofs)
    pub hash: [u8; 32],
    
    /// Metadata about the input (type, size, etc.)
    pub metadata: SszInputMetadata,
}

/// Metadata about an SSZ circuit input
#[derive(Debug, Clone)]
pub struct SszInputMetadata {
    /// The type of the input
    pub input_type: SszInputType,
    
    /// The original size of the input in bytes
    pub original_size: usize,
    
    /// Optional identifier for the input
    pub id: Option<[u8; 32]>,
}

/// Types of SSZ circuit inputs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SszInputType {
    /// A resource
    Resource,
    
    /// A value expression
    ValueExpr,
    
    /// A raw byte array
    RawBytes,
    
    /// A Merkle proof
    MerkleProof,
}

impl SszCircuitInput {
    /// Create a new SSZ circuit input from a Resource
    pub fn from_resource(resource: &Resource) -> Self {
        let serialized_bytes = resource.as_ssz_bytes();
        let original_size = serialized_bytes.len();
        let mut hasher = Sha256::new();
        hasher.update(&serialized_bytes);
        let hash = hasher.finalize().into();
        
        Self {
            serialized_bytes,
            hash,
            metadata: SszInputMetadata {
                input_type: SszInputType::Resource,
                original_size,
                id: Some(resource.id.inner()),
            },
        }
    }
    
    /// Create a new SSZ circuit input from a ValueExpr
    pub fn from_value_expr(value_expr: &ValueExpr) -> Result<Self> {
        let serialized_bytes = value_expr.as_ssz_bytes();
        let original_size = serialized_bytes.len();
        let mut hasher = Sha256::new();
        hasher.update(&serialized_bytes);
        let hash = hasher.finalize().into();
        
        Ok(Self {
            serialized_bytes,
            hash,
            metadata: SszInputMetadata {
                input_type: SszInputType::ValueExpr,
                original_size,
                id: None,
            },
        })
    }
    
    /// Create a new SSZ circuit input from raw bytes
    pub fn from_raw_bytes(bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let hash = hasher.finalize().into();
        
        Self {
            serialized_bytes: bytes.to_vec(),
            hash,
            metadata: SszInputMetadata {
                input_type: SszInputType::RawBytes,
                original_size: bytes.len(),
                id: None,
            },
        }
    }
    
    /// Get the hash of this input as a circuit-friendly field element
    pub fn hash_as_field_element(&self) -> Vec<u8> {
        // This will be implemented based on the specific field implementation
        // For now, we just return the hash as is
        self.hash.to_vec()
    }
    
    /// Convert this input into a circuit-friendly representation
    pub fn to_circuit_representation(&self) -> Vec<u8> {
        // This will be implemented based on the specific circuit requirements
        // For now, we just return the serialized bytes
        self.serialized_bytes.clone()
    }
    
    /// Try to recover the original Resource from this input
    pub fn try_as_resource(&self) -> Result<Resource> {
        if self.metadata.input_type != SszInputType::Resource {
            return Err(anyhow!("Input is not a Resource"));
        }
        
        Resource::from_ssz_bytes(&self.serialized_bytes)
            .map_err(|e| anyhow!("Failed to deserialize Resource: {}", e))
    }
    
    /// Try to recover the original ValueExpr from this input
    pub fn try_as_value_expr(&self) -> Result<ValueExpr> {
        if self.metadata.input_type != SszInputType::ValueExpr {
            return Err(anyhow!("Input is not a ValueExpr"));
        }
        
        ValueExpr::from_ssz_bytes(&self.serialized_bytes)
            .map_err(|e| anyhow!("Failed to deserialize ValueExpr: {}", e))
    }
} 