//! Traverse Integration for ZK Proof Generation
//!
//! This module provides integration with the Traverse ZK storage path generator,
//! enabling Causality programs to generate ZK proofs of blockchain state queries.
//! It bridges the gap between Causality's high-level state queries and Traverse's
//! low-level proof generation capabilities.

use crate::proof_primitives::{CompiledProof, WitnessData};
use crate::storage_layout::{StorageLayout, TraverseLayoutInfo};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Traverse client for interacting with Traverse services
pub struct TraverseClient {
    /// Base URL for Traverse API
    base_url: String,
    /// HTTP client for making requests
    client: reqwest::Client,
    /// Configuration for proof generation
    config: TraverseClientConfig,
    /// Cache for storage layouts
    layout_cache: BTreeMap<String, TraverseLayoutInfo>,
}

/// Configuration for Traverse client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraverseClientConfig {
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Enable response caching
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
}

/// Request to generate a ZK proof via Traverse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofGenerationRequest {
    /// Storage layout for the contract
    pub layout: TraverseLayoutInfo,
    /// Witness data for the proof
    pub witness_data: WitnessData,
    /// Proof generation parameters
    pub parameters: ProofParameters,
    /// Layout commitment for versioning
    pub layout_commitment: String,
}

/// Parameters for proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofParameters {
    /// Storage query to prove
    pub query: String,
    /// Block number for the proof
    pub block_number: u64,
    /// Contract address
    pub contract_address: String,
    /// Proof type identifier
    pub proof_type: String,
    /// Additional parameters
    pub additional_params: BTreeMap<String, String>,
}

/// Response from Traverse proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofGenerationResponse {
    /// Generated proof data
    pub proof: ProofData,
    /// Proof verification information
    pub verification_info: VerificationInfo,
    /// Generation metadata
    pub metadata: ProofMetadata,
}

/// ZK proof data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofData {
    /// Proof bytes (hex encoded)
    pub proof_bytes: String,
    /// Public inputs
    pub public_inputs: Vec<String>,
    /// Verification key
    pub verification_key: String,
    /// Proof format identifier
    pub format: String,
}

/// Information for verifying the proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationInfo {
    /// Circuit identifier
    pub circuit_id: String,
    /// Layout commitment used
    pub layout_commitment: String,
    /// Verification parameters
    pub verification_params: BTreeMap<String, String>,
}

/// Metadata about proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMetadata {
    /// Generation duration in milliseconds
    pub generation_duration_ms: u64,
    /// Proof size in bytes
    pub proof_size_bytes: usize,
    /// Number of constraints in the circuit
    pub constraint_count: Option<u64>,
    /// Traverse version used
    pub traverse_version: String,
}

/// Errors that can occur during Traverse integration
#[derive(Debug, thiserror::Error)]
pub enum TraverseIntegrationError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Proof generation failed: {0}")]
    ProofGenerationFailed(String),

    #[error("Invalid layout: {0}")]
    InvalidLayout(String),

    #[error("Witness validation failed: {0}")]
    WitnessValidationFailed(String),

    #[error("Traverse service unavailable")]
    ServiceUnavailable,

    #[error("Layout commitment mismatch: expected {expected}, got {actual}")]
    LayoutCommitmentMismatch { expected: String, actual: String },
}

impl TraverseClient {
    /// Create a new Traverse client
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
            config: TraverseClientConfig::default(),
            layout_cache: BTreeMap::new(),
        }
    }

    /// Create a new Traverse client with custom configuration
    pub fn with_config(base_url: String, config: TraverseClientConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            base_url,
            client,
            config,
            layout_cache: BTreeMap::new(),
        }
    }

    /// Generate a ZK proof for a compiled proof
    pub async fn generate_proof(
        &self,
        compiled_proof: &CompiledProof,
    ) -> Result<ProofGenerationResponse, TraverseIntegrationError> {
        let request = self.create_proof_request(compiled_proof)?;

        // Validate the request before sending
        self.validate_proof_request(&request)?;

        // Send the request to Traverse
        let response = self.send_proof_request(&request).await?;

        // Validate the response
        self.validate_proof_response(&response, &request)?;

        Ok(response)
    }

    /// Create a proof generation request from a compiled proof
    fn create_proof_request(
        &self,
        compiled_proof: &CompiledProof,
    ) -> Result<ProofGenerationRequest, TraverseIntegrationError> {
        let parameters = ProofParameters {
            query: format!(
                "{}[{}]",
                compiled_proof.primitive.storage_slot,
                "0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C"
            ),
            block_number: compiled_proof.witness_data.block_number,
            contract_address: compiled_proof.witness_data.contract_address.clone(),
            proof_type: format!("{:?}", compiled_proof.primitive.proof_type),
            additional_params: BTreeMap::new(),
        };

        Ok(ProofGenerationRequest {
            layout: compiled_proof.storage_layout.clone(),
            witness_data: compiled_proof.witness_data.clone(),
            parameters,
            layout_commitment: compiled_proof
                .layout_commitment
                .commitment_hash
                .clone(),
        })
    }

    /// Validate a proof generation request
    fn validate_proof_request(
        &self,
        request: &ProofGenerationRequest,
    ) -> Result<(), TraverseIntegrationError> {
        // Validate layout structure
        if request.layout.storage.is_empty() {
            return Err(TraverseIntegrationError::InvalidLayout(
                "Storage layout is empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Send a proof generation request to Traverse
    async fn send_proof_request(
        &self,
        request: &ProofGenerationRequest,
    ) -> Result<ProofGenerationResponse, TraverseIntegrationError> {
        let url = format!("{}/api/v1/generate-proof", self.base_url);

        let mut retry_count = 0;
        let mut last_error = None;

        while retry_count <= self.config.max_retries {
            match self.client.post(&url).json(request).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<ProofGenerationResponse>().await {
                            Ok(proof_response) => return Ok(proof_response),
                            Err(e) => {
                                last_error =
                                    Some(TraverseIntegrationError::HttpError(e));
                            }
                        }
                    } else if response.status() == 503 {
                        last_error =
                            Some(TraverseIntegrationError::ServiceUnavailable);
                    } else {
                        let error_text = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());
                        last_error =
                            Some(TraverseIntegrationError::ProofGenerationFailed(
                                error_text,
                            ));
                    }
                }
                Err(e) => {
                    last_error = Some(TraverseIntegrationError::HttpError(e));
                }
            }

            retry_count += 1;
            if retry_count <= self.config.max_retries {
                // Exponential backoff
                let delay = std::time::Duration::from_millis(
                    1000 * (2_u64.pow(retry_count - 1)),
                );
                tokio::time::sleep(delay).await;
            }
        }

        Err(last_error.unwrap_or(TraverseIntegrationError::ServiceUnavailable))
    }

    /// Validate a proof generation response
    fn validate_proof_response(
        &self,
        response: &ProofGenerationResponse,
        request: &ProofGenerationRequest,
    ) -> Result<(), TraverseIntegrationError> {
        // Verify layout commitment matches
        if response.verification_info.layout_commitment != request.layout_commitment
        {
            return Err(TraverseIntegrationError::LayoutCommitmentMismatch {
                expected: request.layout_commitment.clone(),
                actual: response.verification_info.layout_commitment.clone(),
            });
        }

        // Validate proof data
        if response.proof.proof_bytes.is_empty() {
            return Err(TraverseIntegrationError::ProofGenerationFailed(
                "Empty proof bytes".to_string(),
            ));
        }

        if response.proof.verification_key.is_empty() {
            return Err(TraverseIntegrationError::ProofGenerationFailed(
                "Empty verification key".to_string(),
            ));
        }

        Ok(())
    }

    /// Compile a storage layout for use with Traverse
    pub fn compile_layout(
        &mut self,
        layout: &StorageLayout,
    ) -> Result<TraverseLayoutInfo, TraverseIntegrationError> {
        let traverse_layout = TraverseLayoutInfo {
            contract_name: layout.contract_name.clone(),
            storage: layout
                .storage
                .iter()
                .map(|entry| crate::storage_layout::TraverseStorageEntry {
                    label: entry.label.clone(),
                    slot: entry.slot.clone(),
                    offset: entry.offset as u32,
                    type_name: entry.type_name.clone(),
                })
                .collect(),
            types: layout
                .types
                .iter()
                .map(|type_info| crate::storage_layout::TraverseTypeInfo {
                    type_name: type_info.label.clone(),
                    encoding: type_info.encoding.clone(),
                    number_of_bytes: type_info.number_of_bytes.clone(),
                    base: type_info.base.clone(),
                    key: type_info.key.clone(),
                    value: type_info.value.clone(),
                })
                .collect(),
        };

        // Cache the compiled layout
        if self.config.enable_caching {
            self.layout_cache
                .insert(layout.contract_name.clone(), traverse_layout.clone());
        }

        Ok(traverse_layout)
    }

    /// Get a cached layout if available
    pub fn get_cached_layout(
        &self,
        contract_name: &str,
    ) -> Option<&TraverseLayoutInfo> {
        if self.config.enable_caching {
            self.layout_cache.get(contract_name)
        } else {
            None
        }
    }

    /// Clear the layout cache
    pub fn clear_cache(&mut self) {
        self.layout_cache.clear();
    }

    /// Check if Traverse service is available
    pub async fn health_check(&self) -> Result<bool, TraverseIntegrationError> {
        let url = format!("{}/health", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

/// Batch proof generation for multiple proofs
pub struct BatchProofGenerator {
    /// Traverse client
    client: TraverseClient,
    /// Batch configuration
    config: BatchConfig,
}

/// Configuration for batch proof generation
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Batch timeout in milliseconds
    pub batch_timeout_ms: u64,
    /// Enable parallel processing
    pub enable_parallel: bool,
}

impl BatchProofGenerator {
    /// Create a new batch proof generator
    pub fn new(client: TraverseClient, config: BatchConfig) -> Self {
        Self { client, config }
    }

    /// Generate proofs for a batch of compiled proofs
    pub async fn generate_batch(
        &self,
        proofs: Vec<CompiledProof>,
    ) -> Result<
        Vec<Result<ProofGenerationResponse, TraverseIntegrationError>>,
        TraverseIntegrationError,
    > {
        if proofs.len() > self.config.max_batch_size {
            return Err(TraverseIntegrationError::ProofGenerationFailed(format!(
                "Batch size {} exceeds maximum {}",
                proofs.len(),
                self.config.max_batch_size
            )));
        }

        if self.config.enable_parallel {
            self.generate_parallel(proofs).await
        } else {
            self.generate_sequential(proofs).await
        }
    }

    /// Generate proofs in parallel
    async fn generate_parallel(
        &self,
        proofs: Vec<CompiledProof>,
    ) -> Result<
        Vec<Result<ProofGenerationResponse, TraverseIntegrationError>>,
        TraverseIntegrationError,
    > {
        let futures: Vec<_> = proofs
            .iter()
            .map(|proof| self.client.generate_proof(proof))
            .collect();

        let results = futures::future::join_all(futures).await;
        Ok(results)
    }

    /// Generate proofs sequentially
    async fn generate_sequential(
        &self,
        proofs: Vec<CompiledProof>,
    ) -> Result<
        Vec<Result<ProofGenerationResponse, TraverseIntegrationError>>,
        TraverseIntegrationError,
    > {
        let mut results = Vec::new();

        for proof in proofs {
            let result = self.client.generate_proof(&proof).await;
            results.push(result);
        }

        Ok(results)
    }
}

impl Default for TraverseClientConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 30000, // 30 seconds
            max_retries: 3,
            enable_caching: true,
            cache_ttl_seconds: 3600, // 1 hour
        }
    }
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 10,
            batch_timeout_ms: 60000, // 1 minute
            enable_parallel: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::almanac_schema::LayoutCommitment;
    use crate::proof_primitives::{
        ProofGenerationConfig, ProofType, ProveStatePrimitive, WitnessData,
        WitnessStrategy,
    };
    use crate::storage_layout::{StorageEntry, StorageLayout, TypeInfo};

    fn create_test_compiled_proof() -> CompiledProof {
        let storage_layout = TraverseLayoutInfo {
            contract_name: "usdc".to_string(),
            storage: vec![crate::storage_layout::TraverseStorageEntry {
                label: "balances".to_string(),
                slot: "1".to_string(),
                offset: 0,
                type_name: "t_mapping_address_uint256".to_string(),
            }],
            types: vec![crate::storage_layout::TraverseTypeInfo {
                type_name: "t_mapping_address_uint256".to_string(),
                encoding: "mapping".to_string(),
                number_of_bytes: "32".to_string(),
                base: Some("address".to_string()),
                key: Some("address".to_string()),
                value: Some("uint256".to_string()),
            }],
        };

        CompiledProof {
            primitive: ProveStatePrimitive {
                contract_id: "usdc".to_string(),
                storage_slot: "balances".to_string(),
                parameters: vec![],
                proof_type: ProofType::BalanceProof,
                witness_strategy: WitnessStrategy::Automatic,
                optimization_hints: vec![],
            },
            witness_data: WitnessData {
                storage_key: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
                storage_value: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                merkle_proof: vec![
                    "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
                ],
                block_number: 18000000,
                contract_address: "usdc".to_string(),
            },
            storage_layout,
            proof_config: ProofGenerationConfig::default(),
            layout_commitment: LayoutCommitment {
                commitment_hash: "test_hash".to_string(),
                version: "1.0.0".to_string(),
                timestamp: 1234567890,
            },
        }
    }

    #[test]
    fn test_traverse_client_creation() {
        let client = TraverseClient::new("http://localhost:8081".to_string());
        assert_eq!(client.base_url, "http://localhost:8081");
    }

    #[test]
    fn test_proof_request_creation() {
        let client = TraverseClient::new("http://localhost:8081".to_string());
        let compiled_proof = create_test_compiled_proof();

        let request = client.create_proof_request(&compiled_proof).unwrap();
        assert_eq!(request.layout_commitment, "test_hash");
        assert_eq!(request.parameters.contract_address, "usdc");
    }

    #[test]
    fn test_proof_request_validation() {
        let client = TraverseClient::new("http://localhost:8081".to_string());
        let compiled_proof = create_test_compiled_proof();
        let request = client.create_proof_request(&compiled_proof).unwrap();

        // Valid request should pass validation
        assert!(client.validate_proof_request(&request).is_ok());
    }
}
