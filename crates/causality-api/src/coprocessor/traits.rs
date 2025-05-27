// Purpose: Defines traits for interacting with ZK Coprocessors.

use anyhow::Result;
use async_trait::async_trait;

// Use the canonical ExecutionTrace from causality_types.
use causality_types::trace::ExecutionTrace;

pub use crate::coprocessor::types::{
    CoprocessorApiError, Proof, ProofRequestId, ProofStatus, PublicInputs,
};

// Placeholder for ExecutionTrace until it's moved to causality-types
// and causality-api depends on causality-types correctly.
// This is a temporary measure.
// #[derive(Debug, Clone, Default)] // Add ssz if needed by the trait directly, but likely passed by value.
// pub struct ExecutionTrace { pub dummy_data: Vec<u8> } // Removed placeholder

/// Trait defining the API for a ZK Coprocessor.
#[async_trait]
pub trait ZkCoprocessorApi: Send + Sync {
    /// Submits an execution trace to the coprocessor for ZK proof generation.
    async fn submit_trace_for_proving(
        &self,
        trace: ExecutionTrace,
    ) -> Result<ProofRequestId, CoprocessorApiError>;

    /// Checks the status of a proof generation job.
    async fn check_proof_status(
        &self,
        job_id: &ProofRequestId,
    ) -> Result<ProofStatus, CoprocessorApiError>;

    /// Retrieves the generated ZK proof if the job status is `Completed`.
    /// Returns an error if the job is not completed or if the proof cannot be retrieved.
    async fn get_proof(
        &self,
        job_id: &ProofRequestId,
    ) -> Result<Proof, CoprocessorApiError>;

    /// Verifies a ZK proof against the given public inputs.
    /// Returns `Ok(true)` if verification succeeds, `Ok(false)` if it fails (but the operation was valid),
    /// or an error for operational failures.
    async fn verify_proof(
        &self,
        proof: &Proof,
        public_inputs: &PublicInputs,
    ) -> Result<bool, CoprocessorApiError>;
}
