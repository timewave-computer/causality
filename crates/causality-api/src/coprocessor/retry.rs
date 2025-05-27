//! Retry Mechanism for Coprocessor Operations
//!
//! This module provides a retry wrapper for ZK coprocessor operations, implementing
//! exponential backoff, configurable retry limits, and failure handling to improve
//! resilience when working with external proof generation services.

use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use super::generator::ProofGenerator;
use super::types::{Proof, ProofRequest, ProofRequestId, ProofStatus};
use crate::gateway::ApiError;

//-----------------------------------------------------------------------------
// Retry Configuration
//-----------------------------------------------------------------------------

/// Configuration for retrying operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,

    /// Initial delay between retries in milliseconds
    pub initial_delay_ms: u64,

    /// Factor by which to increase delay on each retry
    pub backoff_factor: f64,

    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 1000,
            backoff_factor: 2.0,
            max_delay_ms: 10000,
        }
    }
}

//-----------------------------------------------------------------------------
// Retryable Generator Implementation
//-----------------------------------------------------------------------------

/// Wrapper for a proof generator that adds retry functionality
pub struct RetryableProofGenerator<G: ProofGenerator> {
    /// The wrapped proof generator
    generator: Arc<G>,

    /// Retry configuration
    config: RetryConfig,

    /// Stores VFS paths for ongoing requests
    vfs_paths: Arc<DashMap<ProofRequestId, (String, String)>>,
}

impl<G: ProofGenerator + 'static> RetryableProofGenerator<G> {
    /// Create a new retryable proof generator with the default retry config
    pub fn new(generator: Arc<G>) -> Self {
        Self {
            generator,
            config: RetryConfig::default(),
            vfs_paths: Arc::new(DashMap::new()),
        }
    }

    /// Create a new retryable proof generator with a custom retry config
    pub fn with_config(generator: Arc<G>, config: RetryConfig) -> Self {
        Self {
            generator,
            config,
            vfs_paths: Arc::new(DashMap::new()),
        }
    }

    /// Calculate the delay for a retry attempt
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_ms = (self.config.initial_delay_ms as f64
            * self.config.backoff_factor.powi(attempt as i32))
        .min(self.config.max_delay_ms as f64) as u64;

        Duration::from_millis(delay_ms)
    }

    /// Request a proof with retries
    #[allow(unused_assignments)]
    pub async fn request_proof_with_retry(
        &self,
        request: ProofRequest,
    ) -> Result<ProofRequestId, ApiError> {
        let mut attempt = 0;
        let mut current_delay = self.calculate_delay(attempt);

        // Store details before first attempt
        let output_vfs_path_clone = request.output_vfs_path.clone();
        let program_id_clone = request.program_id.clone();

        loop {
            match self
                .generator
                .submit_causality_proof_request(request.clone())
                .await
            {
                Ok(request_id) => {
                    self.vfs_paths.insert(
                        request_id.clone(),
                        (output_vfs_path_clone, program_id_clone),
                    );
                    return Ok(request_id);
                }
                Err(err) => {
                    attempt += 1;
                    if attempt >= self.config.max_attempts {
                        return Err(err);
                    }

                    current_delay = self.calculate_delay(attempt);
                    sleep(current_delay).await;
                }
            }
        }
    }

    /// Get proof status with retries
    #[allow(unused_assignments)]
    pub async fn get_proof_status_with_retry(
        &self,
        request_id: &ProofRequestId,
    ) -> Result<ProofStatus, ApiError> {
        let details = self.vfs_paths.get(request_id).ok_or_else(|| {
            ApiError::InternalError(format!(
                "Request details not found for ID: {:?}",
                request_id
            ))
        })?;
        let (output_vfs_path, program_id) = details.value();

        let mut attempt = 0;
        let mut current_delay = self.calculate_delay(attempt);
        loop {
            match self
                .generator
                .get_proof_status(request_id, program_id, output_vfs_path)
                .await
            {
                Ok(status) => return Ok(status),
                Err(err) => {
                    attempt += 1;
                    if attempt >= self.config.max_attempts {
                        return Err(err);
                    }
                    current_delay = self.calculate_delay(attempt);
                    sleep(current_delay).await;
                }
            }
        }
    }

    /// Get proof with retries
    #[allow(unused_assignments)]
    pub async fn get_proof_with_retry(
        &self,
        request_id: &ProofRequestId,
    ) -> Result<Proof, ApiError> {
        let details = self.vfs_paths.get(request_id).ok_or_else(|| {
            ApiError::InternalError(format!(
                "Request details not found for ID: {:?}",
                request_id
            ))
        })?;
        let (output_vfs_path, program_id) = details.value();

        let mut attempt = 0;
        let mut current_delay = self.calculate_delay(attempt);
        loop {
            match self
                .generator
                .get_proof(request_id, program_id, output_vfs_path)
                .await
            {
                Ok(proof) => return Ok(proof),
                Err(err) => {
                    attempt += 1;
                    if attempt >= self.config.max_attempts {
                        return Err(err);
                    }
                    current_delay = self.calculate_delay(attempt);
                    sleep(current_delay).await;
                }
            }
        }
    }

    /// Cancels a proof request (no retry logic applied here, direct pass-through).
    pub async fn cancel_proof_request_direct(
        &self,
        request_id: &ProofRequestId,
    ) -> Result<(), ApiError> {
        self.generator.cancel_proof_request(request_id).await
    }

    /// Performs a health check (no retry logic, direct pass-through).
    pub async fn health_check_direct(&self) -> Result<(), ApiError> {
        self.generator.health_check().await
    }
}
