// Resource operations for TEL
// Original file: src/tel/resource/operations.rs

// Resource operations module for TEL
//
// This module defines the core resource operation interfaces
// for the Temporal Effect Language (TEL).

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use causality_tel::{
    ResourceId, Address, Domain, Metadata, 
    OperationId, Proof, Parameters, Timestamp,
};
use causality_telel::RegisterContents;
use crate::crypto::ContentId;

/// Type of resource operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceOperationType {
    /// Create a new resource
    Create,
    /// Update an existing resource
    Update,
    /// Delete a resource
    Delete,
    /// Transfer a resource to a new owner
    Transfer,
    /// Lock a resource
    Lock,
    /// Unlock a resource
    Unlock,
    /// Merge multiple resources
    Merge,
    /// Split a resource into multiple resources
    Split,
    /// Verify a resource's integrity
    Verify,
    /// Commit a resource state
    Commit,
    /// Rollback a resource state
    Rollback,
    /// Custom operation
    Custom(u16),
}

/// Operation on a resource
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceOperation {
    /// Type of operation
    pub operation_type: ResourceOperationType,
    /// ID of the register being operated on
    pub target: ContentId,
    /// Input register contents
    pub inputs: Vec<RegisterContents>,
    /// Output register contents (for operations that produce new registers)
    pub outputs: Vec<RegisterContents>,
    /// Proof of the operation's validity
    pub proof: Option<Proof>,
    /// Verification key for the proof
    pub verification_key: Option<Vec<u8>>,
    /// Domain where the operation is being performed
    pub domain: Domain,
    /// Address of the initiator
    pub initiator: Address,
    /// Operation parameters
    pub parameters: Parameters,
    /// Metadata
    pub metadata: Metadata,
    /// Timestamp when the operation was created
    pub timestamp: Timestamp,
}

impl ResourceOperation {
    /// Create a new resource operation
    pub fn new(
        operation_type: ResourceOperationType,
        target: ContentId,
        domain: Domain,
        initiator: Address,
    ) -> Self {
        Self {
            operation_type,
            target,
            inputs: Vec::new(),
            outputs: Vec::new(),
            proof: None,
            verification_key: None,
            domain,
            initiator,
            parameters: HashMap::new(),
            metadata: HashMap::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
    
    /// Add an input to the operation
    pub fn with_input(mut self, content: RegisterContents) -> Self {
        self.inputs.push(content);
        self
    }
    
    /// Add multiple inputs to the operation
    pub fn with_inputs(mut self, contents: Vec<RegisterContents>) -> Self {
        self.inputs.extend(contents);
        self
    }
    
    /// Add an output to the operation
    pub fn with_output(mut self, content: RegisterContents) -> Self {
        self.outputs.push(content);
        self
    }
    
    /// Add a proof to the operation
    pub fn with_proof(mut self, proof: Proof, verification_key: Option<Vec<u8>>) -> Self {
        self.proof = Some(proof);
        self.verification_key = verification_key;
        self
    }
    
    /// Add a parameter to the operation
    pub fn with_parameter(mut self, key: &str, value: serde_json::Value) -> Self {
        self.parameters.insert(key.to_string(), value);
        self
    }
    
    /// Add metadata to the operation
    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
    
    /// Verify the operation's proof
    pub fn verify(&self) -> bool {
        if let (Some(proof), Some(key)) = (&self.proof, &self.verification_key) {
            // Instead of just returning true, we'll do some basic checks
            // In a real implementation, this would use cryptographic verification
            
            // 1. Check that the proof has data
            if proof.data.is_empty() {
                tracing::warn!("Empty proof data for operation on {:?}", self.target);
                return false;
            }
            
            // 2. Check that we have a verification key with data
            if key.is_empty() {
                tracing::warn!("Empty verification key for operation on {:?}", self.target);
                return false;
            }
            
            // 3. Check that the proof type is recognized
            match proof.proof_type.as_str() {
                "signature" | "zk" | "merkle" | "sha256" | "multisig" => {
                    // Recognized proof types
                    // In a real implementation, we would do type-specific verification
                    tracing::info!("Recognized proof type: {}", proof.proof_type);
                    
                    // For now just check that the key and data have reasonable lengths
                    key.len() >= 16 && proof.data.len() >= 32
                }
                _ => {
                    tracing::warn!("Unrecognized proof type: {}", proof.proof_type);
                    false
                }
            }
        } else {
            // No proof or key provided
            false
        }
    }
}

/// Resource operation builder
pub struct ResourceOperationBuilder {
    /// Type of operation
    operation_type: ResourceOperationType,
    /// ID of the register being operated on
    target: ContentId,
    /// Input register contents
    inputs: Vec<RegisterContents>,
    /// Output register contents
    outputs: Vec<RegisterContents>,
    /// Proof of the operation's validity
    proof: Option<Proof>,
    /// Verification key for the proof
    verification_key: Option<Vec<u8>>,
    /// Domain where the operation is being performed
    domain: Domain,
    /// Address of the initiator
    initiator: Address,
    /// Operation parameters
    parameters: Parameters,
    /// Metadata
    metadata: Metadata,
}

impl ResourceOperationBuilder {
    /// Create a new operation builder
    pub fn new(
        operation_type: ResourceOperationType,
        target: ContentId,
        domain: Domain,
        initiator: Address,
    ) -> Self {
        Self {
            operation_type,
            target,
            inputs: Vec::new(),
            outputs: Vec::new(),
            proof: None,
            verification_key: None,
            domain,
            initiator,
            parameters: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add an input to the operation
    pub fn add_input(mut self, content: RegisterContents) -> Self {
        self.inputs.push(content);
        self
    }
    
    /// Add an output to the operation
    pub fn add_output(mut self, content: RegisterContents) -> Self {
        self.outputs.push(content);
        self
    }
    
    /// Add a proof to the operation
    pub fn add_proof(mut self, proof: Proof, verification_key: Option<Vec<u8>>) -> Self {
        self.proof = Some(proof);
        self.verification_key = verification_key;
        self
    }
    
    /// Add a parameter to the operation
    pub fn add_parameter(mut self, key: &str, value: serde_json::Value) -> Self {
        self.parameters.insert(key.to_string(), value);
        self
    }
    
    /// Add metadata to the operation
    pub fn add_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
    
    /// Build the operation
    pub fn build(self) -> ResourceOperation {
        ResourceOperation {
            operation_type: self.operation_type,
            target: self.target,
            inputs: self.inputs,
            outputs: self.outputs,
            proof: self.proof,
            verification_key: self.verification_key,
            domain: self.domain,
            initiator: self.initiator,
            parameters: self.parameters,
            metadata: self.metadata,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
} 
