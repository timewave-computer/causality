//! Mock ZK backend for testing

use crate::{
    error::{ProofResult, VerificationError},
    ZkBackend, ZkCircuit, ZkProof, ZkWitness,
};
use serde::{Deserialize, Serialize};

/// Configuration for the mock backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockConfig {
    /// Success rate for proof generation (0.0 to 1.0)
    pub success_rate: f64,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            success_rate: 1.0, // Always succeed by default
        }
    }
}

/// Mock backend that generates fake proofs for testing
pub struct MockBackend {
    /// Whether to simulate proof generation success
    success_rate: f64,
}

impl MockBackend {
    /// Create new mock backend
    pub fn new() -> Self {
        Self {
            success_rate: 1.0, // Always succeed by default
        }
    }

    /// Create mock backend with specified success rate
    pub fn with_success_rate(success_rate: f64) -> Self {
        Self { success_rate }
    }

    /// Create mock backend with configuration
    pub fn with_config(config: MockConfig) -> Self {
        Self {
            success_rate: config.success_rate,
        }
    }
}

impl ZkBackend for MockBackend {
    fn generate_proof(
        &self,
        circuit: &ZkCircuit,
        _witness: &ZkWitness,
    ) -> ProofResult<ZkProof> {
        // Simulate proof generation
        use rand::Rng;

        if rand::thread_rng().gen::<f64>() > self.success_rate {
            return Err(crate::error::ProofError::GenerationFailed(
                "Mock backend simulated failure".to_string(),
            ));
        }

        // Generate fake proof data - ensure it's not empty
        let mock_proof_data = vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8]; // Non-empty mock proof data
        let mock_public_inputs = vec![42u8, 24u8, 13u8, 37u8]; // Non-empty mock public inputs

        Ok(ZkProof::new(
            circuit.id.clone(),
            mock_proof_data,
            mock_public_inputs,
        ))
    }

    fn verify_proof(
        &self,
        _proof: &ZkProof,
        _public_inputs: &[i64],
    ) -> Result<bool, VerificationError> {
        // Mock verification always succeeds (unless simulating failure)
        use rand::Rng;

        if rand::thread_rng().gen::<f64>() > self.success_rate {
            return Err(VerificationError::VerificationFailed(
                "Mock backend simulated verification failure".to_string(),
            ));
        }

        Ok(true)
    }

    fn backend_name(&self) -> &'static str {
        "mock"
    }

    fn is_available(&self) -> bool {
        true // Mock backend is always available
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ZkCircuit;
    use causality_core::machine::instruction::{Instruction, RegisterId};

    #[test]
    fn test_mock_backend_proof_generation() {
        let backend = MockBackend::new();

        let test_instructions = vec![Instruction::Transform {
            morph_reg: RegisterId::new(0),
            input_reg: RegisterId::new(1),
            output_reg: RegisterId::new(2),
        }];
        let circuit = ZkCircuit::new(test_instructions, Vec::new());

        let witness =
            crate::ZkWitness::new(circuit.id.clone(), vec![42], vec![1, 2, 3]);

        let result = backend.generate_proof(&circuit, &witness);
        assert!(result.is_ok());

        let proof = result.unwrap();
        assert_eq!(proof.circuit_id, circuit.id);
        assert!(!proof.proof_data.is_empty());
    }

    #[test]
    fn test_mock_backend_verification() {
        let backend = MockBackend::new();

        let circuit_id = "test_circuit".to_string();
        let proof = ZkProof::new(circuit_id.clone(), vec![1, 2, 3], Vec::new());

        let result = backend.verify_proof(&proof, &[]);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_mock_backend_failure_simulation() {
        let backend = MockBackend::with_success_rate(0.0); // Always fail

        let test_instructions = vec![Instruction::Transform {
            morph_reg: RegisterId::new(0),
            input_reg: RegisterId::new(1),
            output_reg: RegisterId::new(2),
        }];
        let circuit = ZkCircuit::new(test_instructions, Vec::new());

        let witness =
            crate::ZkWitness::new(circuit.id.clone(), vec![42], vec![1, 2, 3]);

        let result = backend.generate_proof(&circuit, &witness);
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_backend_with_config() {
        let config = MockConfig { success_rate: 0.8 };
        let backend = MockBackend::with_config(config);
        assert_eq!(backend.success_rate, 0.8);
    }
}
