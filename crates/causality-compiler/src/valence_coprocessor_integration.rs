//! Valence Coprocessor Integration for ZK Proof Generation
//!
//! This module provides integration with the Valence coprocessor system for generating
//! and verifying ZK proofs. It handles proof lifecycle management, submission tracking,
//! and result processing for cross-chain state verification.

use std::collections::BTreeMap;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use tokio::time::{timeout, Duration};
use crate::proof_primitives::CompiledProof;
use crate::traverse_integration::{ProofGenerationResponse, ProofData, VerificationInfo};

// Use existing ZK infrastructure instead of reimplementing
use causality_zk::{ZkProofGenerator, ZkProof, StorageProofGenerator};

// Conditional imports for Valence coprocessor
#[cfg(feature = "valence")]
use valence_coprocessor_core::{Context, Data, Domain, Hash, Registry, Smt, Vm, Zkvm};

/// Valence coprocessor client for proof operations
pub struct ValenceCoprocessorClient {
    /// Base URL for coprocessor API
    base_url: String,
    /// HTTP client for requests
    client: reqwest::Client,
    /// Client configuration
    config: CoprocessorClientConfig,
    /// Active proof submissions
    active_proofs: BTreeMap<String, ProofSubmission>,
    /// Proof result cache
    result_cache: BTreeMap<String, CachedProofResult>,
}

/// Configuration for Valence coprocessor client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoprocessorClientConfig {
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Polling interval for proof status in milliseconds
    pub polling_interval_ms: u64,
    /// Enable result caching
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Maximum concurrent proof submissions
    pub max_concurrent_proofs: usize,
}

/// Proof submission tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofSubmission {
    /// Unique submission ID
    pub submission_id: String,
    /// Original compiled proof
    pub compiled_proof: CompiledProof,
    /// Submission timestamp
    pub submitted_at: u64,
    /// Current status
    pub status: ProofStatus,
    /// Status updates history
    pub status_history: Vec<StatusUpdate>,
    /// Estimated completion time
    pub estimated_completion: Option<u64>,
}

/// Status of a proof submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofStatus {
    /// Proof is queued for processing
    Queued,
    /// Proof generation is in progress
    InProgress { progress_percentage: u8 },
    /// Proof generation completed successfully
    Completed { result: ProofGenerationResponse },
    /// Proof generation failed
    Failed { error: String },
    /// Proof was cancelled
    Cancelled,
    /// Proof verification in progress
    Verifying,
    /// Proof verified successfully
    Verified { verification_result: VerificationResult },
}

/// Status update entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusUpdate {
    /// Update timestamp
    pub timestamp: u64,
    /// Previous status
    pub from_status: String,
    /// New status
    pub to_status: String,
    /// Optional message
    pub message: Option<String>,
}

/// Cached proof result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedProofResult {
    /// The proof result
    pub result: ProofGenerationResponse,
    /// Cache timestamp
    pub cached_at: u64,
    /// Layout commitment used
    pub layout_commitment: String,
    /// Number of times accessed
    pub access_count: u64,
}

/// Request to submit a proof for generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofSubmissionRequest {
    /// Compiled proof to generate
    pub compiled_proof: CompiledProof,
    /// Priority level (1-10, higher is more urgent)
    pub priority: u8,
    /// Optional callback URL for status updates
    pub callback_url: Option<String>,
    /// Additional metadata
    pub metadata: BTreeMap<String, String>,
}

/// Response from proof submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofSubmissionResponse {
    /// Unique submission ID for tracking
    pub submission_id: String,
    /// Initial status
    pub status: ProofStatus,
    /// Estimated processing time in seconds
    pub estimated_processing_time: u64,
    /// Position in queue (if queued)
    pub queue_position: Option<u32>,
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether the proof is valid
    pub is_valid: bool,
    /// Verification details
    pub verification_details: VerificationDetails,
    /// Verification duration in milliseconds
    pub verification_duration_ms: u64,
}

/// Details about proof verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationDetails {
    /// Circuit used for verification
    pub circuit_id: String,
    /// Public inputs verified
    pub public_inputs: Vec<String>,
    /// Verification key hash
    pub verification_key_hash: String,
    /// Any verification warnings
    pub warnings: Vec<String>,
}

/// Errors that can occur during coprocessor integration
#[derive(Debug, thiserror::Error)]
pub enum CoprocessorError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Proof submission failed: {0}")]
    SubmissionFailed(String),
    
    #[error("Proof generation failed: {0}")]
    GenerationFailed(String),
    
    #[error("Proof verification failed: {0}")]
    VerificationFailed(String),
    
    #[error("Submission not found: {0}")]
    SubmissionNotFound(String),
    
    #[error("Timeout waiting for proof completion")]
    Timeout,
    
    #[error("Coprocessor service unavailable")]
    ServiceUnavailable,
    
    #[error("Invalid proof data: {0}")]
    InvalidProofData(String),
    
    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),
}

impl ValenceCoprocessorClient {
    /// Create a new Valence coprocessor client
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
            config: CoprocessorClientConfig::default(),
            active_proofs: BTreeMap::new(),
            result_cache: BTreeMap::new(),
        }
    }
    
    /// Create a new client with custom configuration
    pub fn with_config(base_url: String, config: CoprocessorClientConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
            
        Self {
            base_url,
            client,
            config,
            active_proofs: BTreeMap::new(),
            result_cache: BTreeMap::new(),
        }
    }
    
    /// Submit a proof for generation
    pub async fn submit_proof(&mut self, request: ProofSubmissionRequest) -> Result<ProofSubmissionResponse, CoprocessorError> {
        // Check cache first if enabled
        if self.config.enable_caching {
            if let Some(cached) = self.get_cached_result(&request.compiled_proof) {
                let submission_id = uuid::Uuid::new_v4().to_string();
                return Ok(ProofSubmissionResponse {
                    submission_id,
                    status: ProofStatus::Completed { result: cached.result },
                    estimated_processing_time: 0,
                    queue_position: None,
                });
            }
        }
        
        // Check concurrent proof limit
        if self.active_proofs.len() >= self.config.max_concurrent_proofs {
            return Err(CoprocessorError::SubmissionFailed(
                "Maximum concurrent proofs exceeded".to_string()
            ));
        }
        
        let submission_id = uuid::Uuid::new_v4().to_string();
        
        // Mock submission for development (real implementation would call coprocessor API)
        let submission_response = ProofSubmissionResponse {
            submission_id: submission_id.clone(),
            status: ProofStatus::Queued,
            estimated_processing_time: 30, // 30 seconds
            queue_position: Some(self.active_proofs.len() as u32 + 1),
        };
        
        // Track the submission
        let submission = ProofSubmission {
            submission_id: submission_id.clone(),
            compiled_proof: request.compiled_proof,
            submitted_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            status: submission_response.status.clone(),
            status_history: vec![],
            estimated_completion: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() + submission_response.estimated_processing_time
            ),
        };
        
        self.active_proofs.insert(submission_id, submission);
        Ok(submission_response)
    }
    
    /// Get the status of a proof submission
    pub async fn get_proof_status(&mut self, submission_id: &str) -> Result<ProofStatus, CoprocessorError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // First, check if submission exists and get the current status
        let (current_status, submitted_at) = {
            if let Some(submission) = self.active_proofs.get(submission_id) {
                (submission.status.clone(), submission.submitted_at)
            } else {
                return Err(CoprocessorError::SubmissionNotFound(submission_id.to_string()));
            }
        };
        
        let elapsed = now - submitted_at;
        
        // Determine new status based on elapsed time
        let new_status = match &current_status {
            ProofStatus::Queued if elapsed > 5 => {
                ProofStatus::InProgress { progress_percentage: 25 }
            },
            ProofStatus::InProgress { progress_percentage } if elapsed > 15 => {
                if *progress_percentage < 100 {
                    ProofStatus::InProgress { progress_percentage: (*progress_percentage + 25).min(100) }
                } else {
                    // Create mock proof result
                    let compiled_proof = self.active_proofs.get(submission_id).unwrap().compiled_proof.clone();
                    ProofStatus::Completed { 
                        result: self.create_mock_proof_result(&compiled_proof)
                    }
                }
            },
            status => status.clone(),
        };
        
        // Check if status has changed and update tracking
        if !matches!(current_status.clone(), new_status) {
            if let Some(submission) = self.active_proofs.get_mut(submission_id) {
                let old_status = format!("{:?}", current_status);
                let new_status_str = format!("{:?}", new_status);
                
                submission.status_history.push(StatusUpdate {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    from_status: old_status,
                    to_status: new_status_str,
                    message: Some(format!("Status updated from {:?} to {:?}", current_status, new_status)),
                });
                
                submission.status = new_status.clone();
                submission.estimated_completion = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() + 30 // Estimate 30 seconds remaining
                );
            }
        }
        
        Ok(new_status)
    }
    
    /// Wait for a proof to complete
    pub async fn wait_for_completion(&mut self, submission_id: &str, timeout_seconds: u64) -> Result<ProofGenerationResponse, CoprocessorError> {
        let timeout_duration = Duration::from_secs(timeout_seconds);
        let polling_interval = Duration::from_millis(self.config.polling_interval_ms);
        
        timeout(timeout_duration, async {
            loop {
                match self.get_proof_status(submission_id).await? {
                    ProofStatus::Completed { result } => {
                        // Cache the result if enabled
                        if self.config.enable_caching {
                            let compiled_proof = self.active_proofs.get(submission_id)
                                .map(|s| s.compiled_proof.clone());
                            if let Some(proof) = compiled_proof {
                                self.cache_result(&proof, &result);
                            }
                        }
                        
                        // Remove from active proofs
                        self.active_proofs.remove(submission_id);
                        return Ok(result);
                    },
                    ProofStatus::Failed { error } => {
                        self.active_proofs.remove(submission_id);
                        return Err(CoprocessorError::GenerationFailed(error));
                    },
                    ProofStatus::Cancelled => {
                        self.active_proofs.remove(submission_id);
                        return Err(CoprocessorError::GenerationFailed("Proof was cancelled".to_string()));
                    },
                    _ => {
                        // Continue polling
                        tokio::time::sleep(polling_interval).await;
                    }
                }
            }
        }).await.map_err(|_| CoprocessorError::Timeout)?
    }
    
    /// Verify a generated proof
    pub async fn verify_proof(&self, proof_data: &ProofData, verification_info: &VerificationInfo) -> Result<VerificationResult, CoprocessorError> {
        // Mock verification for development
        Ok(VerificationResult {
            is_valid: true,
            verification_details: VerificationDetails {
                circuit_id: verification_info.circuit_id.clone(),
                public_inputs: proof_data.public_inputs.clone(),
                verification_key_hash: "mock_vk_hash".to_string(),
                warnings: vec![],
            },
            verification_duration_ms: 100,
        })
    }
    
    /// Create mock proof result for development
    fn create_mock_proof_result(&self, compiled_proof: &CompiledProof) -> ProofGenerationResponse {
        use crate::traverse_integration::{ProofMetadata};
        
        ProofGenerationResponse {
            proof: ProofData {
                proof_bytes: "0x1234567890abcdef".to_string(),
                public_inputs: vec![
                    compiled_proof.witness_data.storage_value.clone(),
                    compiled_proof.witness_data.block_number.to_string(),
                ],
                verification_key: "mock_verification_key".to_string(),
                format: "groth16".to_string(),
            },
            verification_info: VerificationInfo {
                circuit_id: format!("{}_circuit", compiled_proof.primitive.contract_id),
                layout_commitment: compiled_proof.layout_commitment.commitment_hash.clone(),
                verification_params: BTreeMap::new(),
            },
            metadata: ProofMetadata {
                generation_duration_ms: 5000,
                proof_size_bytes: 256,
                constraint_count: Some(1000000),
                traverse_version: "0.1.0".to_string(),
            },
        }
    }
    
    /// Get cached result if available
    fn get_cached_result(&self, compiled_proof: &CompiledProof) -> Option<CachedProofResult> {
        let cache_key = self.generate_cache_key(compiled_proof);
        
        if let Some(cached) = self.result_cache.get(&cache_key) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if now - cached.cached_at < self.config.cache_ttl_seconds {
                return Some(cached.clone());
            }
        }
        
        None
    }
    
    /// Cache a proof result
    fn cache_result(&mut self, compiled_proof: &CompiledProof, result: &ProofGenerationResponse) {
        let cache_key = self.generate_cache_key(compiled_proof);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let cached_result = CachedProofResult {
            result: result.clone(),
            cached_at: now,
            layout_commitment: compiled_proof.layout_commitment.commitment_hash.clone(),
            access_count: 1,
        };
        
        self.result_cache.insert(cache_key, cached_result);
    }
    
    /// Generate cache key for a compiled proof
    fn generate_cache_key(&self, compiled_proof: &CompiledProof) -> String {
        format!("{}:{}:{}",
            compiled_proof.primitive.contract_id,
            compiled_proof.primitive.storage_slot,
            compiled_proof.layout_commitment.commitment_hash
        )
    }
    
    /// Health check for the coprocessor service
    pub async fn health_check(&self) -> Result<bool, CoprocessorError> {
        // Mock health check - always return true for development
        Ok(true)
    }
}

impl Default for CoprocessorClientConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 60000, // 1 minute
            max_retries: 3,
            polling_interval_ms: 5000, // 5 seconds
            enable_caching: true,
            cache_ttl_seconds: 3600, // 1 hour
            max_concurrent_proofs: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_primitives::{ProveStatePrimitive, ProofType, WitnessStrategy};
    use crate::storage_layout::TraverseLayoutInfo;
    
    fn create_test_compiled_proof() -> CompiledProof {
        CompiledProof {
            primitive: ProveStatePrimitive {
                contract_id: "test_contract".to_string(),
                storage_slot: "balances".to_string(),
                parameters: vec![],
                proof_type: ProofType::BalanceProof,
                witness_strategy: WitnessStrategy::Automatic,
                optimization_hints: vec![],
            },
            witness_data: WitnessData {
                storage_key: "0x123".to_string(),
                storage_value: "0x456".to_string(),
                merkle_proof: vec!["0x789".to_string()],
                block_number: 12345,
                contract_address: "0xabc".to_string(),
            },
            storage_layout: TraverseLayoutInfo {
                storage: vec![],
                types: vec![],
            },
            proof_config: ProofGenerationConfig::default(),
            layout_commitment: LayoutCommitment {
                commitment_hash: "test_hash".to_string(),
                version: "1.0.0".to_string(),
                timestamp: 0,
            },
        }
    }
    
    #[test]
    fn test_client_creation() {
        let client = ValenceCoprocessorClient::new("http://localhost:8080".to_string());
        assert_eq!(client.base_url, "http://localhost:8080");
        assert!(client.active_proofs.is_empty());
    }
    
    #[test]
    fn test_cache_key_generation() {
        let client = ValenceCoprocessorClient::new("http://localhost:8080".to_string());
        let proof = create_test_compiled_proof();
        
        let key = client.generate_cache_key(&proof);
        assert_eq!(key, "test_contract:balances:test_hash");
    }
    
    #[tokio::test]
    async fn test_health_check() {
        let client = ValenceCoprocessorClient::new("http://localhost:8080".to_string());
        let result = client.health_check().await;
        assert!(result.is_ok());
        assert!(result.unwrap()); // Mock always returns true
    }
} 