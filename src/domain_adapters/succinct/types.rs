// Types for the Succinct ZK-VM adapter
//
// This module defines the common types used across the Succinct adapter
// implementation.

use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

use crate::error::{Error, Result};
use serde::{Serialize, Deserialize};

/// Identifier for a compiled Succinct program
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProgramId(pub String);

impl ProgramId {
    /// Create a new program ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Get the program ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ProgramId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for ProgramId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

/// Public inputs for a Succinct program
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicInputs {
    /// The public input values
    pub values: HashMap<String, Vec<u8>>,
}

impl PublicInputs {
    /// Create a new empty set of public inputs
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
    
    /// Add a public input value
    pub fn add<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<Vec<u8>>,
    {
        self.values.insert(key.into(), value.into());
    }
    
    /// Add a string public input
    pub fn add_string<K>(&mut self, key: K, value: &str)
    where
        K: Into<String>,
    {
        self.values.insert(key.into(), value.as_bytes().to_vec());
    }
    
    /// Add a numeric public input
    pub fn add_u64<K>(&mut self, key: K, value: u64)
    where
        K: Into<String>,
    {
        self.values.insert(key.into(), value.to_le_bytes().to_vec());
    }
    
    /// Get a public input value
    pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.values.get(key)
    }
    
    /// Get a string public input
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|bytes| {
            String::from_utf8(bytes.clone()).ok()
        })
    }
    
    /// Get a numeric public input
    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.get(key).and_then(|bytes| {
            if bytes.len() == 8 {
                let mut array = [0u8; 8];
                array.copy_from_slice(bytes);
                Some(u64::from_le_bytes(array))
            } else {
                None
            }
        })
    }
    
    /// Get all public inputs as key-value pairs
    pub fn entries(&self) -> impl Iterator<Item = (&String, &Vec<u8>)> {
        self.values.iter()
    }
    
    /// Check if the inputs are empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    
    /// Get the number of inputs
    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl Default for PublicInputs {
    fn default() -> Self {
        Self::new()
    }
}

/// Proof data from a Succinct ZK-VM
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofData {
    /// The raw proof data
    pub data: Vec<u8>,
    /// The type of proof
    pub proof_type: String,
    /// The program ID that generated the proof
    pub program_id: ProgramId,
    /// The public inputs used to generate the proof
    pub public_inputs: PublicInputs,
}

impl ProofData {
    /// Create new proof data
    pub fn new(
        data: Vec<u8>,
        proof_type: impl Into<String>,
        program_id: ProgramId,
        public_inputs: PublicInputs,
    ) -> Self {
        Self {
            data,
            proof_type: proof_type.into(),
            program_id,
            public_inputs,
        }
    }
    
    /// Get the proof data as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    
    /// Get the proof type
    pub fn proof_type(&self) -> &str {
        &self.proof_type
    }
    
    /// Get the program ID
    pub fn program_id(&self) -> &ProgramId {
        &self.program_id
    }
    
    /// Get the public inputs
    pub fn public_inputs(&self) -> &PublicInputs {
        &self.public_inputs
    }
    
    /// Get the size of the proof data in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Options for proof generation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofOptions {
    /// Key-value pairs for proof options
    pub options: HashMap<String, String>,
}

impl ProofOptions {
    /// Create new empty proof options
    pub fn new() -> Self {
        Self {
            options: HashMap::new(),
        }
    }
    
    /// Add a proof option
    pub fn add<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.options.insert(key.into(), value.into());
    }
    
    /// Get a proof option
    pub fn get(&self, key: &str) -> Option<&String> {
        self.options.get(key)
    }
    
    /// Get all options as key-value pairs
    pub fn entries(&self) -> impl Iterator<Item = (&String, &String)> {
        self.options.iter()
    }
    
    /// Check if the options are empty
    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }
    
    /// Get the number of options
    pub fn len(&self) -> usize {
        self.options.len()
    }
}

impl Default for ProofOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Verification key for a program
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationKey {
    /// The raw verification key data
    pub data: Vec<u8>,
    /// The backend this key is for
    pub backend: String,
    /// The program ID this key is for
    pub program_id: ProgramId,
}

impl VerificationKey {
    /// Create a new verification key
    pub fn new(data: Vec<u8>, backend: &str, program_id: ProgramId) -> Self {
        Self {
            data,
            backend: backend.to_string(),
            program_id,
        }
    }
}

/// Statistics for Succinct program execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    /// The estimated time to generate a proof
    pub estimated_proving_time: Duration,
    /// The estimated memory usage for proving
    pub estimated_memory_usage: usize,
    /// The estimated time to verify a proof
    pub estimated_verification_time: Duration,
    /// The estimated size of the proof
    pub estimated_proof_size: usize,
    /// Backend-specific statistics
    pub backend_stats: HashMap<String, String>,
}

impl ExecutionStats {
    /// Create new execution stats
    pub fn new(
        proving_time: Duration,
        memory_usage: usize,
        verification_time: Duration,
        proof_size: usize,
    ) -> Self {
        Self {
            estimated_proving_time: proving_time,
            estimated_memory_usage: memory_usage,
            estimated_verification_time: verification_time,
            estimated_proof_size: proof_size,
            backend_stats: HashMap::new(),
        }
    }
    
    /// Set a different proving time
    pub fn with_proving_time(mut self, time: Duration) -> Self {
        self.estimated_proving_time = time;
        self
    }
    
    /// Set a different memory usage
    pub fn with_memory_usage(mut self, usage: usize) -> Self {
        self.estimated_memory_usage = usage;
        self
    }
    
    /// Add a backend-specific stat
    pub fn with_backend_stat(mut self, key: &str, value: &str) -> Self {
        self.backend_stats.insert(key.to_string(), value.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_program_id() {
        let id = ProgramId::new("test_program");
        assert_eq!(id.as_str(), "test_program");
        
        let id_from_string = ProgramId::from("test_program".to_string());
        assert_eq!(id_from_string.as_str(), "test_program");
        
        let id_from_str = ProgramId::from("test_program");
        assert_eq!(id_from_str.as_str(), "test_program");
    }
    
    #[test]
    fn test_public_inputs() {
        let mut inputs = PublicInputs::new();
        inputs.add("input1", vec![1, 2, 3]);
        inputs.add_string("input2", "test");
        inputs.add_u64("input3", 42);
        
        assert_eq!(inputs.get("input1"), Some(&vec![1, 2, 3]));
        assert_eq!(inputs.get_string("input2"), Some("test".to_string()));
        assert_eq!(inputs.get_u64("input3"), Some(42));
        
        assert_eq!(inputs.len(), 3);
        assert!(!inputs.is_empty());
    }
    
    #[test]
    fn test_proof_data() {
        let program_id = ProgramId::new("test_program");
        let mut public_inputs = PublicInputs::new();
        public_inputs.add_u64("input", 42);
        
        let proof = ProofData::new(
            vec![1, 2, 3, 4],
            "succinct",
            program_id.clone(),
            public_inputs.clone(),
        );
        
        assert_eq!(proof.as_bytes(), &[1, 2, 3, 4]);
        assert_eq!(proof.proof_type(), "succinct");
        assert_eq!(proof.program_id(), &program_id);
        assert_eq!(proof.public_inputs(), &public_inputs);
        assert_eq!(proof.size(), 4);
    }
    
    #[test]
    fn test_proof_options() {
        let mut options = ProofOptions::new();
        options.add("optimization", "medium");
        options.add("parallel", "true");
        
        assert_eq!(options.get("optimization"), Some(&"medium".to_string()));
        assert_eq!(options.get("parallel"), Some(&"true".to_string()));
        
        assert_eq!(options.len(), 2);
        assert!(!options.is_empty());
    }
} 