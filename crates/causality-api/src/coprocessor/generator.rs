//! Proof Generator Interface
//!
//! This module defines the core interfaces for generating Zero-Knowledge proofs
//! through external coprocessors. It provides traits for submitting proof requests,
//! checking status, and retrieving generated proofs with error handling.

//-----------------------------------------------------------------------------
// Proof Generator Interface
//-----------------------------------------------------------------------------

use async_trait::async_trait;
// Removed unused import: std::sync::Arc

use super::types::{
    CoprocessorId, Proof, ProofRequest, ProofRequestId, ProofStatus,
};
use crate::gateway::ApiError;

/// Core trait for ZK coprocessor clients
#[async_trait]
pub trait ProofGenerator: Send + Sync {
    /// Returns the unique identifier of the coprocessor.
    fn coprocessor_id(&self) -> CoprocessorId;

    /// Submits a proof request to the coprocessor.
    async fn submit_causality_proof_request(
        &self,
        request: ProofRequest,
    ) -> Result<ProofRequestId, ApiError>;

    /// Retrieves the status of a proof generation task.
    async fn get_proof_status(
        &self,
        request_id: &ProofRequestId,
        program_id: &str,
        output_vfs_path: &str,
    ) -> Result<ProofStatus, ApiError>;

    /// Retrieves the generated proof.
    async fn get_proof(
        &self,
        request_id: &ProofRequestId,
        program_id: &str,
        output_vfs_path: &str,
    ) -> Result<Proof, ApiError>;

    /// Cancels a proof request.
    async fn cancel_proof_request(
        &self,
        request_id: &ProofRequestId,
    ) -> Result<(), ApiError>;

    /// Performs a health check on the coprocessor.
    async fn health_check(&self) -> Result<(), ApiError>;
}

/// Factory for creating proof generators
#[async_trait]
pub trait ProofGeneratorFactory: Send + Sync {
    /// The type of proof generator this factory creates
    type Generator: ProofGenerator;

    /// Create a new proof generator
    async fn create_generator(
        &self,
        coprocessor_id: CoprocessorId,
    ) -> Result<Self::Generator, ApiError>;
}
