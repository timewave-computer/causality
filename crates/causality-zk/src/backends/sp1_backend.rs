//! SP1 backend implementation for Causality ZK system
//!
//! This module provides integration with the SP1 proving system through
//! the Valence coprocessor infrastructure.

use crate::{
    ZkCircuit, ZkProof, ZkWitness,
    error::{ProofResult, VerificationError, ProofError},
    backends::ZkBackend,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "sp1")]
use sp1_sdk::{ProverClient, SP1Stdin, SP1ProofWithPublicValues, SP1VerifyingKey};

/// SP1 backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sp1Config {
    /// SP1 prover endpoint (if using remote proving)
    pub prover_endpoint: Option<String>,
    
    /// Whether to use local or remote proving
    pub use_remote_prover: bool,
    
    /// Maximum proving timeout in seconds
    pub timeout_secs: u64,
    
    /// SP1 circuit ELF binary (compiled circuit)
    pub circuit_elf: Option<Vec<u8>>,
}

impl Default for Sp1Config {
    fn default() -> Self {
        Self {
            prover_endpoint: None,
            use_remote_prover: false,
            timeout_secs: 300, // 5 minutes default timeout
            circuit_elf: None,
        }
    }
}

/// SP1 backend for generating and verifying ZK proofs
pub struct Sp1Backend {
    config: Sp1Config,
    #[cfg(feature = "sp1")]
    prover_client: Option<ProverClient>,
    #[cfg(feature = "sp1")]
    verification_keys: HashMap<String, SP1VerifyingKey>,
}

impl Sp1Backend {
    /// Create a new SP1 backend with default configuration
    pub fn new() -> Self {
        Self::with_config(Sp1Config::default())
    }
    
    /// Create a new SP1 backend with custom configuration
    pub fn with_config(config: Sp1Config) -> Self {
        #[cfg(feature = "sp1")]
        let prover_client = if config.use_remote_prover {
            // For remote proving, we would configure the client with the endpoint
            config.prover_endpoint.as_ref()
                .and_then(|_| Some(ProverClient::new()))
        } else {
            // For local proving
            Some(ProverClient::new())
        };
        
        Self {
            config,
            #[cfg(feature = "sp1")]
            prover_client,
            #[cfg(feature = "sp1")]
            verification_keys: HashMap::new(),
        }
    }
    
    /// Set the circuit ELF for proving
    pub fn set_circuit_elf(&mut self, elf: Vec<u8>) {
        self.config.circuit_elf = Some(elf);
    }
    
    #[cfg(feature = "sp1")]
    /// Generate witness inputs for SP1 from causality witness
    fn prepare_sp1_inputs(&self, witness: &ZkWitness) -> Result<SP1Stdin> {
        let mut stdin = SP1Stdin::new();
        
        // For now, just write the private inputs as bytes
        // In a real implementation, this would parse the execution trace
        // and extract structured data for SP1
        stdin.write(&witness.private_inputs);
        
        Ok(stdin)
    }
    
    #[cfg(feature = "sp1")]
    /// Get or generate verification key for circuit
    fn get_verification_key(&mut self, circuit: &ZkCircuit) -> Result<&SP1VerifyingKey> {
        let circuit_id = circuit.id.clone();
        
        if !self.verification_keys.contains_key(&circuit_id) {
            // Get the circuit ELF
            let elf = self.config.circuit_elf.as_ref()
                .ok_or_else(|| anyhow::anyhow!("No circuit ELF configured for SP1 backend"))?;
            
            // Generate verification key
            let client = self.prover_client.as_ref()
                .ok_or_else(|| anyhow::anyhow!("SP1 prover client not initialized"))?;
            
            let (_, vk) = client.setup(elf);
            self.verification_keys.insert(circuit_id.clone(), vk);
        }
        
        Ok(&self.verification_keys[&circuit_id])
    }
}

impl Default for Sp1Backend {
    fn default() -> Self {
        Self::new()
    }
}

impl ZkBackend for Sp1Backend {
    fn generate_proof(&self, circuit: &ZkCircuit, witness: &ZkWitness) -> ProofResult<ZkProof> {
        #[cfg(feature = "sp1")]
        {
            // Get the circuit ELF
            let elf = self.config.circuit_elf.as_ref()
                .ok_or_else(|| ProofError::InvalidWitness("No circuit ELF configured".to_string()))?;
            
            // Get prover client
            let client = self.prover_client.as_ref()
                .ok_or_else(|| ProofError::BackendError("SP1 prover client not initialized".to_string()))?;
            
            // Prepare inputs
            let stdin = self.prepare_sp1_inputs(witness)
                .map_err(|e| ProofError::InvalidWitness(format!("Failed to prepare SP1 inputs: {}", e)))?;
            
            // Generate proof
            let proof = client.prove(elf, stdin)
                .map_err(|e| ProofError::ProofGeneration(format!("SP1 proof generation failed: {}", e)))?;
            
            // Convert SP1 proof to our ZkProof format
            let proof_bytes = bincode::serialize(&proof)
                .map_err(|e| ProofError::SerializationError(format!("Failed to serialize SP1 proof: {}", e)))?;
            
            // Extract public outputs from SP1 proof
            let public_outputs = proof.public_values.to_vec();
            
            Ok(ZkProof::new(
                circuit.id.clone(),
                proof_bytes,
                public_outputs,
            ))
        }
        
        #[cfg(not(feature = "sp1"))]
        {
            // Fallback for when SP1 feature is not enabled
            Err(ProofError::BackendError("SP1 backend not available (feature not enabled)".to_string()))
        }
    }
    
    fn verify_proof(&self, proof: &ZkProof, public_inputs: &[i64]) -> Result<bool, VerificationError> {
        #[cfg(feature = "sp1")]
        {
            // Get the circuit ELF
            let elf = self.config.circuit_elf.as_ref()
                .ok_or_else(|| VerificationError::InvalidProof("No circuit ELF configured".to_string()))?;
            
            // Get prover client
            let client = self.prover_client.as_ref()
                .ok_or_else(|| VerificationError::BackendError("SP1 prover client not initialized".to_string()))?;
            
            // Generate verification key
            let (_, vk) = client.setup(elf);
            
            // Deserialize SP1 proof
            let sp1_proof: SP1ProofWithPublicValues = bincode::deserialize(&proof.proof_data)
                .map_err(|e| VerificationError::InvalidProof(format!("Failed to deserialize SP1 proof: {}", e)))?;
            
            // Verify the proof
            let verification_result = client.verify(&sp1_proof, &vk)
                .map_err(|e| VerificationError::VerificationFailed(format!("SP1 verification failed: {}", e)))?;
            
            Ok(verification_result.is_ok())
        }
        
        #[cfg(not(feature = "sp1"))]
        {
            // Fallback for when SP1 feature is not enabled
            Err(VerificationError::BackendError("SP1 backend not available (feature not enabled)".to_string()))
        }
    }
    
    fn backend_name(&self) -> &'static str {
        "sp1"
    }
    
    fn is_available(&self) -> bool {
        #[cfg(feature = "sp1")]
        {
            self.prover_client.is_some() && self.config.circuit_elf.is_some()
        }
        
        #[cfg(not(feature = "sp1"))]
        {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ZkWitness;
    
    #[test]
    fn test_sp1_backend_creation() {
        let backend = Sp1Backend::new();
        assert_eq!(backend.backend_name(), "sp1");
        
        // Without circuit ELF, backend should not be available
        assert!(!backend.is_available());
    }
    
    #[test]
    fn test_sp1_backend_with_config() {
        let config = Sp1Config {
            use_remote_prover: true,
            timeout_secs: 600,
            ..Default::default()
        };
        
        let backend = Sp1Backend::with_config(config);
        assert_eq!(backend.backend_name(), "sp1");
        assert_eq!(backend.config.timeout_secs, 600);
        assert!(backend.config.use_remote_prover);
    }
    
    #[cfg(feature = "sp1")]
    #[tokio::test]
    async fn test_sp1_witness_preparation() {
        let backend = Sp1Backend::new();
        
        let witness = ZkWitness::new("test_circuit".to_string(), vec![42, 84], vec![1, 2, 3]);
        
        // This should not panic and should prepare inputs correctly
        let result = backend.prepare_sp1_inputs(&witness);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_sp1_backend_without_feature() {
        // When SP1 feature is disabled, backend should indicate it's not available
        #[cfg(not(feature = "sp1"))]
        {
            let backend = Sp1Backend::new();
            assert!(!backend.is_available());
        }
    }
} 