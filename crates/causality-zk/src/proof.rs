//! ZK proof generation and management

use crate::{ZkCircuit, ZkProof, ZkWitness, backends::ZkBackend, error::{ProofError, ProofResult}};
use std::collections::HashMap;

/// Proof generator for creating ZK proofs
pub struct ProofGenerator {
    /// ZK backend for proof generation
    backend: Box<dyn ZkBackend>,
    
    /// Circuit cache for reusing compiled circuits
    circuit_cache: HashMap<String, ZkCircuit>,
}

impl ProofGenerator {
    /// Create new proof generator with specified backend
    pub fn new(backend: Box<dyn ZkBackend>) -> Self {
        Self {
            backend,
            circuit_cache: HashMap::new(),
        }
    }
    
    /// Generate proof from circuit and witness
    pub fn generate_proof(&mut self, circuit: &ZkCircuit, witness: &ZkWitness) -> ProofResult<ZkProof> {
        // Create witness schema from circuit instructions and validate
        let witness_schema = crate::witness::WitnessSchema::for_instructions(&circuit.instructions);
        witness_schema.validate_witness(witness)
            .map_err(|e| ProofError::InvalidWitness(e.to_string()))?;
        
        // Generate proof using backend
        self.backend.generate_proof(circuit, witness)
    }
    
    /// Verify an existing proof
    pub fn verify_proof(&self, proof: &ZkProof, circuit: &ZkCircuit) -> ProofResult<bool> {
        // Convert public inputs to i64 values for verification
        let public_inputs: Vec<i64> = circuit.public_inputs.iter().map(|&pi| pi as i64).collect();
        
        // Call backend verification and convert error type
        self.backend.verify_proof(proof, &public_inputs)
            .map_err(|e| ProofError::GenerationFailed(format!("Backend verification failed: {:?}", e)))
    }
    
    /// Cache a circuit for reuse
    pub fn cache_circuit(&mut self, key: String, circuit: ZkCircuit) {
        self.circuit_cache.insert(key, circuit);
    }
    
    /// Get cached circuit
    pub fn get_cached_circuit(&self, key: &str) -> Option<&ZkCircuit> {
        self.circuit_cache.get(key)
    }
} 