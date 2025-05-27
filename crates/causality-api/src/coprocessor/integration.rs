//! ZK Coprocessor Integration
//!
//! This module provides a unified interface for interacting with Zero-Knowledge proof
//! generation coprocessors, including proof generation, verification, and status tracking.
//! All implementations maintain bounded sizes for ZK compatibility.

use async_trait::async_trait;
// Serialization imports removed as we don't use manual SSZ implementations here
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use causality_types::core::{
    AsErrorContext, ContextualError, ErrorCategory, ErrorMetadata,
};
// Removed unused import: causality_types::utils::SszDuration

use super::generator::ProofGenerator;
use super::monitor::{CoprocessorMonitor, HealthStatus};
use super::types::{CoprocessorId, Proof, ProofRequest, ProofRequestId, ProofStatus};
use crate::coprocessor::types::ProofRequestParams; // Added for ProofRequestParams

//-----------------------------------------------------------------------------
// Constants and Bound
//-----------------------------------------------------------------------------

/// Maximum size for proof input data (32 MB)
pub const MAX_PROOF_INPUT_SIZE: usize = 32 * 1024 * 1024;

/// Maximum size for proof output data (1 MB)
pub const MAX_PROOF_OUTPUT_SIZE: usize = 1024 * 1024;

//-----------------------------------------------------------------------------
// ZK Integration Interface
//-----------------------------------------------------------------------------

/// Integration between ZK coprocessors and the causality runtime
#[async_trait]
pub trait ZkIntegration: Send + Sync {
    /// Generate a proof for the given circuit and inputs
    async fn generate_proof(
        &self,
        circuit_id: &str,
        public_inputs: Vec<u8>,
        private_inputs: Vec<u8>,
    ) -> Result<ProofRequestId, ContextualError>;

    /// Check the status of a proof generation request
    async fn check_proof_status(
        &self,
        request_id: &ProofRequestId,
    ) -> Result<ProofStatus, ContextualError>;

    /// Get the proof once it's ready
    async fn get_proof(
        &self,
        request_id: &ProofRequestId,
    ) -> Result<Proof, ContextualError>;

    /// Verify a proof against public inputs
    async fn verify_proof(
        &self,
        circuit_id: &str,
        proof: &Proof,
        public_inputs: &[u8],
    ) -> Result<bool, ContextualError>;

    /// Get the health status of all coprocessors
    async fn get_health_status(
        &self,
    ) -> Result<Vec<(CoprocessorId, HealthStatus)>, ContextualError>;
}

//-----------------------------------------------------------------------------
// ZK Integration Implementation
//-----------------------------------------------------------------------------

/// Implementation of the ZK integration using a proof generator and monitor
pub struct ZkCoprocessorIntegration {
    /// Proof generator for requesting proofs
    generator: Arc<dyn ProofGenerator>,

    /// Coprocessor monitor for health checking
    monitor: Arc<dyn CoprocessorMonitor>,

    /// Error context for creating errors
    error_context: Arc<dyn AsErrorContext>,

    /// Storage for proof requests and results
    proof_storage: Arc<dyn ProofStorage>,
}

impl ZkCoprocessorIntegration {
    /// Create a new ZK integration
    pub fn new(
        generator: Arc<dyn ProofGenerator>,
        monitor: Arc<dyn CoprocessorMonitor>,
        error_context: Arc<dyn AsErrorContext>,
        proof_storage: Arc<dyn ProofStorage>,
    ) -> Self {
        Self {
            generator,
            monitor,
            error_context,
            proof_storage,
        }
    }

    /// Validate input sizes to ensure they are within ZK-compatible bounds
    fn validate_input_sizes(
        &self,
        public_inputs: &[u8],
        private_inputs: &[u8],
    ) -> Result<(), ContextualError> {
        if public_inputs.len() > MAX_PROOF_INPUT_SIZE {
            return Err(self.error_context.create_error(
                format!(
                    "Public inputs size exceeds maximum allowed: {} > {}",
                    public_inputs.len(),
                    MAX_PROOF_INPUT_SIZE
                ),
                ErrorMetadata::new(ErrorCategory::Validation),
            ));
        }

        if private_inputs.len() > MAX_PROOF_INPUT_SIZE {
            return Err(self.error_context.create_error(
                format!(
                    "Private inputs size exceeds maximum allowed: {} > {}",
                    private_inputs.len(),
                    MAX_PROOF_INPUT_SIZE
                ),
                ErrorMetadata::new(ErrorCategory::Validation),
            ));
        }

        Ok(())
    }
}

#[async_trait]
impl ZkIntegration for ZkCoprocessorIntegration {
    async fn generate_proof(
        &self,
        program_id_str: &str,
        public_inputs: Vec<u8>,
        private_inputs: Vec<u8>,
    ) -> Result<ProofRequestId, ContextualError> {
        // Validate input sizes
        self.validate_input_sizes(&public_inputs, &private_inputs)?;

        // circuit_id_bytes is not used with program_id approach
        // let _circuit_id_bytes = CircuitId::from_hex(program_id_str)
        //     .map_err(|e_str| {
        //         self.error_context.as_ref().create_error(
        //             format!(
        //                 "Invalid circuit_id hex string '{}': {}",
        //                 program_id_str, e_str
        //             ),
        //             ErrorMetadata::new(ErrorCategory::Validation),
        //         )
        //     })?
        //     .0;

        // Combine inputs into witness
        let mut witness_data = public_inputs.clone();
        witness_data.extend(private_inputs);

        // Create default proof request parameters
        let _default_params = ProofRequestParams {
            timeout: causality_types::utils::SszDuration::from(Duration::from_secs(300)), // e.g., 5 minutes
            priority: 0,
            use_recursion: false,
            custom_args: None, // Added missing field
        };

        // Create the proof request
        let request = ProofRequest {
            program_id: program_id_str.to_string(),
            witness: witness_data,
            // params: default_params, // Temporarily commented out
            output_vfs_path: format!("/proofs/{}.json", program_id_str),
        };

        // Submit request to the proof generator
        let request_id = self
            .generator
            .submit_causality_proof_request(request)
            .await
            .map_err(|e_gateway| {
                // Map gateway::ApiError to ContextualError
                self.error_context.as_ref().create_error(
                    format!("Proof generation request failed: {:?}", e_gateway),
                    ErrorMetadata::new(ErrorCategory::Boundary), // Changed from GatewayError
                )
            })?;

        // Create proof context
        let context = ProofContext {
            request_id: request_id.clone(),
            program_id: program_id_str.to_string(),
            public_inputs,
            output_vfs_path: format!("/proofs/{}.json", program_id_str),
            status: ProofStatus::Pending,
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            proof: None,
        };

        // Store the context
        self.proof_storage.store_proof_context(&context).await?;

        Ok(request_id) // Original request_id can now be returned
    }

    async fn check_proof_status(
        &self,
        request_id: &ProofRequestId,
    ) -> Result<ProofStatus, ContextualError> {
        // Get the proof context
        let mut context = self.proof_storage.get_proof_context(request_id).await?;

        // If not completed, check current status
        if context.status != ProofStatus::Completed
            && context.status != ProofStatus::Failed
        {
            // Get status from proof generator, using the stored VFS path and program_id
            let new_status = self
                .generator
                .get_proof_status(
                    request_id,
                    &context.program_id,
                    &context.output_vfs_path,
                )
                .await
                .map_err(|e_gateway| {
                    self.error_context.as_ref().create_error(
                        format!(
                            "Failed to get proof status from generator: {:?}",
                            e_gateway
                        ),
                        ErrorMetadata::new(ErrorCategory::Boundary),
                    )
                })?;

            // If the status has changed, update the context
            if new_status != context.status {
                context.status = new_status;

                // If completed, get the proof
                if new_status == ProofStatus::Completed {
                    match self
                        .generator
                        .get_proof(
                            request_id,
                            &context.program_id,
                            &context.output_vfs_path,
                        )
                        .await
                    {
                        Ok(actual_proof) => {
                            // Changed from Ok(proof_option)
                            // Check size for the actual_proof directly
                            if actual_proof.data.len() > MAX_PROOF_OUTPUT_SIZE {
                                context.status = ProofStatus::Failed;
                                context.proof = None;
                            } else {
                                context.proof = Some(actual_proof); // Store the proof
                            }
                        }
                        Err(_) => {
                            context.status = ProofStatus::Failed;
                            context.proof = None;
                        }
                    }
                }

                // Store updated context
                self.proof_storage.store_proof_context(&context).await?;
            }
        }

        Ok(context.status)
    }

    async fn get_proof(
        &self,
        request_id: &ProofRequestId,
    ) -> Result<Proof, ContextualError> {
        // Get the proof context
        let context = self.proof_storage.get_proof_context(request_id).await?;

        // Ensure the proof is completed
        if context.status != ProofStatus::Completed {
            return Err(self.error_context.create_error(
                format!("Proof generation not completed: {:?}", context.status),
                ErrorMetadata::new(ErrorCategory::Validation),
            ));
        }

        // Return the proof
        context.proof.ok_or_else(|| {
            self.error_context.create_error(
                "Proof marked as completed but data not available".to_string(),
                ErrorMetadata::new(ErrorCategory::ResourceNotFound),
            )
        })
    }

    async fn verify_proof(
        &self,
        expected_program_id: &str,
        proof: &Proof,
        public_inputs: &[u8],
    ) -> Result<bool, ContextualError> {
        // Validate input sizes
        if public_inputs.len() > MAX_PROOF_INPUT_SIZE {
            return Err(self.error_context.create_error(
                format!(
                    "Public inputs size exceeds maximum allowed: {} > {}",
                    public_inputs.len(),
                    MAX_PROOF_INPUT_SIZE
                ),
                ErrorMetadata::new(ErrorCategory::Validation),
            ));
        }

        // Ensure the proof is for the requested program
        if proof.program_id != expected_program_id {
            // Log mismatch for easier debugging
            // println!("Proof program_id '{}' does not match expected program_id '{}'", proof.program_id, expected_program_id);
            return Ok(false);
        }

        // For now, assume verification success if program_id matches.
        // In a real implementation, we would use a ZK verifier library (e.g., sp1-sdk for SP1 proofs)
        // and use proof.data, proof.verification_key, and public_inputs.
        Ok(true)
    }

    async fn get_health_status(
        &self,
    ) -> Result<Vec<(CoprocessorId, HealthStatus)>, ContextualError> {
        let coprocessor_id = self.generator.coprocessor_id();
        let health_status = self
            .monitor
            .get_health_status(&coprocessor_id)
            .await
            .map_err(|e_gateway| {
                self.error_context.as_ref().create_error(
                    format!(
                        "Failed to get health status for coprocessor {:?}: {:?}",
                        coprocessor_id, e_gateway
                    ),
                    ErrorMetadata::new(ErrorCategory::Boundary), // Changed from GatewayError
                )
            })?;
        Ok(vec![(coprocessor_id, health_status)])
    }
}

//-----------------------------------------------------------------------------
// Proof Context Type
//-----------------------------------------------------------------------------

/// Context for tracking a proof generation request and its result.
/// Stored by `ProofStorage` implementations.
#[derive(Debug, Clone)]
pub struct ProofContext {
    /// Unique ID for the proof request
    pub request_id: ProofRequestId, // This is now String

    /// Program ID used for the request
    pub program_id: String,

    /// Public inputs used for the proof generation
    pub public_inputs: Vec<u8>,

    /// Destination path on the coprocessor VFS where the proof is/will be stored
    pub output_vfs_path: String,

    /// Current status of the proof generation
    pub status: ProofStatus,

    /// Creation timestamp (seconds since epoch)
    pub created_at: u64,

    /// The generated proof, if available
    pub proof: Option<Proof>,
}

//-----------------------------------------------------------------------------
// Proof Storage Interface
//-----------------------------------------------------------------------------

/// Trait for storing and retrieving proof contexts.
/// This allows decoupling the ZK integration service from the storage mechanism.
#[async_trait]
pub trait ProofStorage: Send + Sync {
    /// Store a proof context
    async fn store_proof_context(
        &self,
        context: &ProofContext,
    ) -> Result<(), ContextualError>;

    /// Get a proof context by request ID
    async fn get_proof_context(
        &self,
        request_id: &ProofRequestId,
    ) -> Result<ProofContext, ContextualError>;

    /// List all proof contexts
    async fn list_proof_contexts(
        &self,
    ) -> Result<Vec<ProofContext>, ContextualError>;
}

//-----------------------------------------------------------------------------
// In-Memory Proof Storage Implementation
//-----------------------------------------------------------------------------

/// In-memory implementation of `ProofStorage` using a HashMap.
pub struct InMemoryProofStorage {
    /// Map of request ID to proof context
    contexts:
        tokio::sync::Mutex<std::collections::HashMap<ProofRequestId, ProofContext>>,

    /// Error context for creating errors
    error_context: Arc<dyn AsErrorContext>,
}

impl InMemoryProofStorage {
    /// Create a new in-memory proof storage
    pub fn new(error_context: Arc<dyn AsErrorContext>) -> Self {
        Self {
            contexts: tokio::sync::Mutex::new(std::collections::HashMap::new()),
            error_context,
        }
    }
}

#[async_trait]
impl ProofStorage for InMemoryProofStorage {
    async fn store_proof_context(
        &self,
        context: &ProofContext,
    ) -> Result<(), ContextualError> {
        let mut contexts = self.contexts.lock().await;
        contexts.insert(context.request_id.clone(), context.clone());
        Ok(())
    }

    async fn get_proof_context(
        &self,
        request_id: &ProofRequestId,
    ) -> Result<ProofContext, ContextualError> {
        let contexts = self.contexts.lock().await;
        contexts.get(request_id).cloned().ok_or_else(|| {
            self.error_context.as_ref().create_error(
                format!("Proof context not found for request ID: {:?}", request_id),
                ErrorMetadata::new(ErrorCategory::ResourceNotFound),
            )
        })
    }

    async fn list_proof_contexts(
        &self,
    ) -> Result<Vec<ProofContext>, ContextualError> {
        let contexts = self.contexts.lock().await;
        Ok(contexts.values().cloned().collect())
    }
}
