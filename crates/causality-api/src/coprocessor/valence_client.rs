//! Valence Coprocessor Client Implementation
//!
//! This module implements a wrapper around the valence-coprocessor-client library
//! to provide a seamless integration with the Causality API's coprocessor module.

use std::net::SocketAddr;
use std::path::Path;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use uuid::Uuid;

use valence_coprocessor_client::{
    CoprocessorClient as ValenceCoprocessorClient, CoprocessorConfig,
    CoprocessorError,
};

use crate::coprocessor::generator::ProofGenerator;
use crate::coprocessor::types::{
    CoprocessorId, Proof, ProofRequest, ProofRequestId, ProofStatus,
};
use crate::gateway::ApiError;

/// A wrapper around the ValenceCoprocessorClient that adapts it to the Causality API.
#[derive(Debug, Clone)]
pub struct ValenceCoprocessorClientWrapper {
    /// The underlying Valence coprocessor client
    inner: ValenceCoprocessorClient,
    /// The socket address of the coprocessor, stored for ID generation.
    coprocessor_socket_addr: SocketAddr,
}

#[allow(dead_code)]
const DEFAULT_COPROCESSOR_SOCKET_ADDR: &str = "127.0.0.1:37281";

impl Default for ValenceCoprocessorClientWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl ValenceCoprocessorClientWrapper {
    /// Create a new ValenceCoprocessorClientWrapper with default configuration.
    pub fn new() -> Self {
        let config = CoprocessorConfig::default();
        Self {
            inner: ValenceCoprocessorClient::new(),
            coprocessor_socket_addr: config.socket,
        }
    }

    /// Create a new ValenceCoprocessorClientWrapper with custom configuration.
    pub fn with_config(config: CoprocessorConfig) -> Self {
        Self {
            inner: ValenceCoprocessorClient::with_config(config.clone()),
            coprocessor_socket_addr: config.socket,
        }
    }

    /// Create a new ValenceCoprocessorClientWrapper with custom socket address.
    pub fn with_socket(socket: SocketAddr) -> Self {
        Self {
            inner: ValenceCoprocessorClient::with_socket(socket),
            coprocessor_socket_addr: socket,
        }
    }

    /// Deploy a domain to the coprocessor
    /// The actual WASM bytes are read from wasm_path by the underlying client.
    pub fn deploy_domain(
        &self,
        name: &str,
        wasm_path: &Path,
    ) -> Result<String, CoprocessorError> {
        self.inner.deploy_domain(name, wasm_path)
    }

    /// Deploy a program to the coprocessor
    /// The actual WASM/ELF bytes are read from paths by the underlying client.
    pub fn deploy_program(
        &self,
        wasm_path: &Path,
        elf_path: &Path,
        nonce: u64,
    ) -> Result<String, CoprocessorError> {
        self.inner.deploy_program(wasm_path, elf_path, nonce)
    }

    /// Submits a proof request structured according to Causality API types.
    pub fn submit_causality_proof_request(
        &self,
        request: ProofRequest,
    ) -> Result<ProofRequestId, anyhow::Error> {
        // Prepare arguments for the ZK program's get_witnesses function
        // let args = request.params.custom_args.clone().unwrap_or(Value::Null); // Temporarily commented out
        let args = serde_json::Value::Null; // Placeholder for args

        // The path on the coprocessor's VFS where the proof should be stored.
        let vfs_output_path = Path::new(&request.output_vfs_path);

        let request_id_str = self.inner.submit_proof_request(
            &request.program_id,
            Some(args),
            vfs_output_path, // This is the path on the coprocessor's VFS
        )?;

        Ok(ProofRequestId(request_id_str))
    }

    /// Get the verification key for a program from the coprocessor.
    pub fn get_verification_key(
        &self,
        program_id: &str,
    ) -> Result<String, CoprocessorError> {
        self.inner.get_verification_key(program_id)
    }

    /// Get the status of a proof request.
    pub fn get_proof_status(
        &self,
        _request_id: &ProofRequestId,
        program_id: &str,
        output_vfs_path: &str,
    ) -> Result<ProofStatus, anyhow::Error> {
        match self
            .inner
            .read_storage(program_id, Path::new(output_vfs_path))
        {
            Ok(_) => Ok(ProofStatus::Completed), // File exists, assume completed
            Err(CoprocessorError::IoError(io_err)) => {
                if io_err.kind() == std::io::ErrorKind::NotFound {
                    Ok(ProofStatus::Pending)
                } else {
                    Ok(ProofStatus::Failed)
                }
            }
            Err(CoprocessorError::NoDataReceived)
            | Err(CoprocessorError::InvalidDataReceived) => {
                // These errors could indicate a file not found or empty file
                Ok(ProofStatus::Pending)
            }
            Err(CoprocessorError::RequestFailed(req_err)) => {
                if let Some(status) = req_err.status() {
                    if status.as_u16() == 404 {
                        Ok(ProofStatus::Pending)
                    } else {
                        Ok(ProofStatus::Failed)
                    }
                } else {
                    Ok(ProofStatus::Failed)
                }
            }
            Err(e) => {
                // Other CoprocessorError variants
                Err(anyhow!("Failed to determine proof status for program {} at path {} due to coprocessor error: {:?}", program_id, output_vfs_path, e))
            }
        }
    }

    /// Get a completed proof from the coprocessor's VFS.
    pub fn get_proof(
        &self,
        _request_id: &ProofRequestId,
        program_id: &str,
        output_vfs_path: &str,
    ) -> Result<Proof, anyhow::Error> {
        match self.inner.read_storage(program_id, Path::new(output_vfs_path)) {
            Ok(json_string) => {
                match serde_json::from_str::<Proof>(&json_string) {
                    Ok(proof) => Ok(proof),
                    Err(e) => Err(anyhow!("Failed to deserialize proof from VFS file {} for program {}: {}", output_vfs_path, program_id, e)),
                }
            }
            Err(CoprocessorError::IoError(io_err)) => {
                if io_err.kind() == std::io::ErrorKind::NotFound {
                    Err(anyhow!("Proof file not found at {} for program {}", output_vfs_path, program_id))
                } else {
                    Err(anyhow!("IO error reading proof file {} for program {}: {}", output_vfs_path, program_id, io_err))
                }
            }
            Err(CoprocessorError::NoDataReceived) => {
                 Err(anyhow!("No data received when trying to read proof file {} for program {}", output_vfs_path, program_id))
            }
            Err(CoprocessorError::InvalidDataReceived) => {
                 Err(anyhow!("Invalid data received when trying to read proof file {} for program {}", output_vfs_path, program_id))
            }
            Err(CoprocessorError::RequestFailed(req_err)) => {
                if let Some(status) = req_err.status() {
                    if status.as_u16() == 404 {
                        Err(anyhow!("Proof file not found (HTTP 404) at {} for program {}", output_vfs_path, program_id))
                    } else {
                        Err(anyhow!("HTTP error ({}) reading proof file {} for program {}: {}", status.as_u16(), output_vfs_path, program_id, req_err))
                    }
                } else {
                    Err(anyhow!("HTTP error reading proof file {} for program {}: {}", output_vfs_path, program_id, req_err))
                }
            }
            Err(e) => { // Other CoprocessorError variants
                Err(anyhow!("Failed to get proof from VFS {} for program {} due to coprocessor error: {:?}", output_vfs_path, program_id, e))
            }
        }
    }
}

/// Factory for creating ValenceCoprocessorClientWrapper instances
pub fn create_coprocessor_client() -> ValenceCoprocessorClientWrapper {
    ValenceCoprocessorClientWrapper::new()
}

/// Factory for creating ValenceCoprocessorClientWrapper instances with custom socket
pub fn create_coprocessor_client_with_socket(
    socket: SocketAddr,
) -> ValenceCoprocessorClientWrapper {
    ValenceCoprocessorClientWrapper::with_socket(socket)
}

#[async_trait]
impl ProofGenerator for ValenceCoprocessorClientWrapper {
    async fn submit_causality_proof_request(
        &self,
        request: ProofRequest,
    ) -> Result<ProofRequestId, ApiError> {
        // Avoid spawn_blocking to prevent Tokio runtime issues
        match self.submit_causality_proof_request(request) {
            Ok(id) => Ok(id),
            Err(e) => Err(ApiError::InvalidRequest(format!(
                "Failed to submit proof request: {}",
                e
            ))),
        }
    }

    async fn get_proof_status(
        &self,
        request_id: &ProofRequestId,
        program_id: &str,
        output_vfs_path: &str,
    ) -> Result<ProofStatus, ApiError> {
        // Avoid spawn_blocking to prevent Tokio runtime issues
        match self.get_proof_status(request_id, program_id, output_vfs_path) {
            Ok(status) => Ok(status),
            Err(e) => Err(ApiError::InternalError(format!(
                "Failed to get proof status: {}",
                e
            ))),
        }
    }

    async fn get_proof(
        &self,
        request_id: &ProofRequestId,
        program_id: &str,
        output_vfs_path: &str,
    ) -> Result<Proof, ApiError> {
        // Avoid spawn_blocking to prevent Tokio runtime issues
        match self.get_proof(request_id, program_id, output_vfs_path) {
            Ok(proof) => Ok(proof),
            Err(e) => Err(ApiError::InternalError(format!(
                "Failed to get proof: {}",
                e
            ))),
        }
    }

    async fn cancel_proof_request(
        &self,
        _request_id: &ProofRequestId,
    ) -> Result<(), ApiError> {
        // The valence-coprocessor-client does not yet support cancellation.
        println!("Warning: cancel_proof_request is not implemented in ValenceCoprocessorClientWrapper.");
        Err(ApiError::InternalError(
            "Cancel operation not supported".to_string(),
        ))
    }

    async fn health_check(&self) -> Result<(), ApiError> {
        // Use a different approach than spawn_blocking to avoid runtime issues
        // Just test if we can get a connection to the coprocessor
        match self
            .inner
            .get_verification_key("health_check_dummy_program_id")
        {
            Ok(_) => Ok(()), // Successfully communicated, even if program doesn't exist
            Err(CoprocessorError::RequestFailed(e)) => {
                if let Some(status) = e.status() {
                    if status.as_u16() == 404 {
                        // 404 Not Found is expected for a non-existent program ID
                        Ok(())
                    } else {
                        // Other HTTP errors suggest the service is not healthy
                        Err(ApiError::InternalError(format!(
                            "Health check failed: HTTP status {}",
                            status.as_u16()
                        )))
                    }
                } else {
                    Err(ApiError::InternalError(format!(
                        "Health check failed: {}",
                        e
                    )))
                }
            }
            Err(e) => Err(ApiError::InternalError(format!(
                "Health check failed: {:?}",
                e
            ))),
        }
    }

    fn coprocessor_id(&self) -> CoprocessorId {
        let socket_str = self.coprocessor_socket_addr.to_string();
        // Using a standard namespace like DNS for generating UUIDv5.
        let id_namespace = Uuid::NAMESPACE_DNS;
        let uuid = Uuid::new_v5(&id_namespace, socket_str.as_bytes());
        CoprocessorId(*uuid.as_bytes())
    }
}
