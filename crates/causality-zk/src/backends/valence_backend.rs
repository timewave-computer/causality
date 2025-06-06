//! Valence Coprocessor backend implementation
//!
//! This module provides integration with the Valence coprocessor system
//! for remote ZK proof generation and verification using the causality-api client.

use crate::{
    ZkCircuit, ZkProof, ZkWitness,
    error::{ProofResult, VerificationError, ProofError},
    backends::ZkBackend,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Valence coprocessor backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValenceConfig {
    /// Coprocessor endpoint URL
    pub endpoint: String,
    
    /// API key for authentication
    pub api_key: Option<String>,
    
    /// Controller ID for the deployed circuit
    pub controller_id: Option<String>,
    
    /// Circuit name for deployment
    pub circuit_name: String,
    
    /// Maximum proof generation timeout
    pub timeout: Duration,
    
    /// Whether to auto-deploy circuits
    pub auto_deploy: bool,
}

impl Default for ValenceConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://prover.timewave.computer:37281".to_string(),
            api_key: None,
            controller_id: None,
            circuit_name: "causality-circuit".to_string(),
            timeout: Duration::from_secs(300), // 5 minutes default
            auto_deploy: true,
        }
    }
}

/// Circuit information response from Valence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitInfo {
    pub name: String,
    pub id: String,
    pub description: Option<String>,
    pub created_at: String,
    pub parameters: Option<HashMap<String, String>>,
}

/// Circuit deployment request to Valence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployCircuitRequest {
    pub name: String,
    pub wasm_bytecode: String,
    pub description: Option<String>,
    pub parameters: Option<HashMap<String, String>>,
}

/// ZK proof request to Valence coprocessor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofRequest {
    pub circuit_name: String,
    pub public_inputs: Vec<String>,
    pub private_inputs: Vec<String>,
    pub metadata: Option<HashMap<String, String>>,
}

/// ZK proof response from Valence coprocessor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofResponse {
    pub proof: String,
    pub public_inputs: Vec<String>,
    pub generation_time_ms: u64,
    pub circuit_info: CircuitInfo,
}

/// Proof verification request to Valence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyProofRequest {
    pub proof: String,
    pub public_inputs: Vec<String>,
    pub circuit_name: String,
}

/// Simple Valence client for ZK operations
#[derive(Debug, Clone)]
pub struct ValenceClient {
    endpoint: String,
    client: reqwest::Client,
    api_key: Option<String>,
}

impl ValenceClient {
    /// Create a new Valence client
    pub fn new(endpoint: String, api_key: Option<String>) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::new(),
            api_key,
        }
    }
    
    /// Deploy a circuit to the Valence coprocessor
    pub async fn deploy_circuit(&self, request: DeployCircuitRequest) -> Result<CircuitInfo> {
        let url = format!("{}/api/v1/circuits", self.endpoint);
        
        let mut req = self.client.post(&url).json(&request);
        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req.send().await?;
        let circuit_info: CircuitInfo = response.json().await?;
        
        Ok(circuit_info)
    }
    
    /// Generate a ZK proof using the Valence coprocessor
    pub async fn generate_proof(&self, request: ZkProofRequest) -> Result<ZkProofResponse> {
        let url = format!("{}/api/v1/proofs", self.endpoint);
        
        let mut req = self.client.post(&url).json(&request);
        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req.send().await?;
        let proof_response: ZkProofResponse = response.json().await?;
        
        Ok(proof_response)
    }
    
    /// Verify a ZK proof using the Valence coprocessor
    pub async fn verify_proof(&self, request: VerifyProofRequest) -> Result<bool> {
        let url = format!("{}/api/v1/verify", self.endpoint);
        
        let mut req = self.client.post(&url).json(&request);
        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req.send().await?;
        let is_valid: bool = response.json().await?;
        
        Ok(is_valid)
    }
    
    /// List available circuits
    pub async fn list_circuits(&self) -> Result<Vec<CircuitInfo>> {
        let url = format!("{}/api/v1/circuits", self.endpoint);
        
        let mut req = self.client.get(&url);
        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req.send().await?;
        let circuits: Vec<CircuitInfo> = response.json().await?;
        
        Ok(circuits)
    }
}

/// Valence backend for ZK proof generation and verification
pub struct ValenceBackend {
    config: ValenceConfig,
    client: ValenceClient,
}

impl ValenceBackend {
    /// Create a new Valence backend with default configuration
    pub fn new() -> Self {
        let config = ValenceConfig::default();
        let client = ValenceClient::new(config.endpoint.clone(), config.api_key.clone());
        
        Self { config, client }
    }
    
    /// Create a new Valence backend with custom configuration
    pub fn with_config(config: ValenceConfig) -> Self {
        let client = ValenceClient::new(config.endpoint.clone(), config.api_key.clone());
        Self { config, client }
    }
    
    /// Prepare circuit deployment request from ZK circuit
    fn prepare_circuit_deployment(&self, circuit: &ZkCircuit) -> Result<DeployCircuitRequest> {
        // Convert ZK circuit to WASM bytecode for Valence deployment
        let wasm_bytecode = self.compile_circuit_to_wasm(circuit)?;
        
        Ok(DeployCircuitRequest {
            name: self.config.circuit_name.clone(),
            wasm_bytecode: BASE64.encode(&wasm_bytecode),
            description: Some(format!("Causality ZK Circuit: {}", circuit.id)),
            parameters: Some(HashMap::from([
                ("circuit_id".to_string(), circuit.id.clone()),
                ("constraint_count".to_string(), circuit.constraints.len().to_string()),
                ("public_input_count".to_string(), circuit.public_inputs.len().to_string()),
            ])),
        })
    }
    
    /// Compile ZK circuit to WASM bytecode
    fn compile_circuit_to_wasm(&self, circuit: &ZkCircuit) -> Result<Vec<u8>> {
        // For now, create a simple WASM representation of the circuit
        // In a production system, this would compile the circuit constraints to actual WASM
        let circuit_json = serde_json::to_string(circuit)?;
        Ok(circuit_json.into_bytes())
    }
    
    /// Prepare proof request from circuit and witness
    fn prepare_proof_request(&self, circuit: &ZkCircuit, witness: &ZkWitness) -> Result<ZkProofRequest> {
        // Convert private inputs to strings
        let private_inputs: Vec<String> = witness.private_inputs.iter()
            .map(|&input| input.to_string())
            .collect();
        
        // Extract public inputs from circuit constraints
        let public_inputs: Vec<String> = circuit.constraints.iter()
            .enumerate()
            .map(|(i, constraint)| {
                // Extract public values from constraints
                // This is a simplified implementation
                format!("constraint_{}_{}", i, constraint)
            })
            .collect();
        
        Ok(ZkProofRequest {
            circuit_name: self.config.circuit_name.clone(),
            public_inputs,
            private_inputs,
            metadata: Some(HashMap::from([
                ("circuit_id".to_string(), circuit.id.clone()),
                ("witness_id".to_string(), witness.id.clone()),
                ("backend".to_string(), "valence".to_string()),
            ])),
        })
    }
    
    /// Deploy circuit to Valence coprocessor if needed
    async fn ensure_circuit_deployed(&mut self, circuit: &ZkCircuit) -> Result<String> {
        if let Some(controller_id) = &self.config.controller_id {
            return Ok(controller_id.clone());
        }
        
        // Check if circuit is already deployed
        let existing_circuits = self.client.list_circuits().await?;
        if let Some(existing) = existing_circuits.iter().find(|c| c.name == self.config.circuit_name) {
            self.config.controller_id = Some(existing.id.clone());
            return Ok(existing.id.clone());
        }
        
        // Deploy new circuit
        if self.config.auto_deploy {
            let deploy_request = self.prepare_circuit_deployment(circuit)?;
            let circuit_info = self.client.deploy_circuit(deploy_request).await?;
            self.config.controller_id = Some(circuit_info.id.clone());
            
            log::info!("Deployed circuit '{}' with controller ID: {}", 
                      self.config.circuit_name, circuit_info.id);
            
            Ok(circuit_info.id)
        } else {
            Err(anyhow::anyhow!("Circuit not deployed and auto_deploy is disabled"))
        }
    }
}

impl Default for ValenceBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ZkBackend for ValenceBackend {
    fn generate_proof(&self, circuit: &ZkCircuit, witness: &ZkWitness) -> ProofResult<ZkProof> {
        // Since the ZkBackend trait is sync but Valence operations are async,
        // we need to use a runtime to execute the async operation
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| ProofError::BackendError(format!("Failed to create runtime: {}", e)))?;
        
        rt.block_on(async {
            // Clone self to avoid mutable borrow issues in async context
            let mut backend = Self::with_config(self.config.clone());
            
            // Ensure circuit is deployed
            let _controller_id = backend.ensure_circuit_deployed(circuit).await
                .map_err(|e| ProofError::BackendError(format!("Failed to deploy circuit: {}", e)))?;
            
            // Prepare proof request
            let proof_request = backend.prepare_proof_request(circuit, witness)
                .map_err(|e| ProofError::InvalidWitness(format!("Failed to prepare proof request: {}", e)))?;
            
            // Generate proof via Valence coprocessor
            let proof_response = backend.client.generate_proof(proof_request).await
                .map_err(|e| ProofError::ProofGeneration(format!("Valence proof generation failed: {}", e)))?;
            
            // Convert response to ZkProof
            let proof_data = BASE64.decode(&proof_response.proof)
                .map_err(|e| ProofError::SerializationError(format!("Failed to decode proof: {}", e)))?;
            
            // Parse public inputs
            let public_outputs: Vec<u8> = proof_response.public_inputs.iter()
                .flat_map(|input| input.as_bytes().to_vec())
                .collect();
            
            Ok(ZkProof::new(
                circuit.id.clone(),
                proof_data,
                public_outputs,
            ))
        })
    }
    
    fn verify_proof(&self, proof: &ZkProof, public_inputs: &[i64]) -> Result<bool, VerificationError> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| VerificationError::BackendError(format!("Failed to create runtime: {}", e)))?;
        
        rt.block_on(async {
            // Prepare verification request
            let verify_request = VerifyProofRequest {
                proof: BASE64.encode(&proof.proof_data),
                public_inputs: public_inputs.iter().map(|i| i.to_string()).collect(),
                circuit_name: self.config.circuit_name.clone(),
            };
            
            // Verify proof via Valence coprocessor
            let is_valid = self.client.verify_proof(verify_request).await
                .map_err(|e| VerificationError::VerificationFailed(format!("Valence verification failed: {}", e)))?;
            
            Ok(is_valid)
        })
    }
    
    fn backend_name(&self) -> &'static str {
        "valence"
    }
    
    fn is_available(&self) -> bool {
        // For now, assume Valence backend is always available
        // In practice, we might want to do a health check
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ZkCircuit, ZkWitness};
    
    #[test]
    fn test_valence_backend_creation() {
        let backend = ValenceBackend::new();
        assert_eq!(backend.backend_name(), "valence");
        assert_eq!(backend.config.endpoint, "http://prover.timewave.computer:37281");
    }
    
    #[test]
    fn test_valence_backend_with_config() {
        let config = ValenceConfig {
            endpoint: "http://localhost:8080".to_string(),
            circuit_name: "test-circuit".to_string(),
            timeout: Duration::from_secs(600),
            auto_deploy: false,
            ..Default::default()
        };
        
        let backend = ValenceBackend::with_config(config);
        assert_eq!(backend.config.endpoint, "http://localhost:8080");
        assert_eq!(backend.config.circuit_name, "test-circuit");
        assert_eq!(backend.config.timeout, Duration::from_secs(600));
        assert!(!backend.config.auto_deploy);
    }
    
    #[test]
    fn test_circuit_deployment_preparation() {
        let backend = ValenceBackend::new();
        let mut circuit = ZkCircuit::new(vec![], vec![42]);
        circuit.id = "test_circuit".to_string();
        
        let deploy_request = backend.prepare_circuit_deployment(&circuit);
        assert!(deploy_request.is_ok());
        
        let request = deploy_request.unwrap();
        assert_eq!(request.name, "causality-circuit");
        assert!(request.description.is_some());
        assert!(request.parameters.is_some());
    }
    
    #[test]
    fn test_proof_request_preparation() {
        let backend = ValenceBackend::new();
        let mut circuit = ZkCircuit::new(vec![], vec![42]);
        circuit.id = "test_circuit".to_string();
        
        let witness = ZkWitness::new("test_circuit".to_string(), vec![42], vec![1, 2, 3]);
        
        let proof_request = backend.prepare_proof_request(&circuit, &witness);
        assert!(proof_request.is_ok());
        
        let request = proof_request.unwrap();
        assert_eq!(request.circuit_name, "causality-circuit");
        assert!(!request.private_inputs.is_empty());
        assert!(request.metadata.is_some());
    }
} 