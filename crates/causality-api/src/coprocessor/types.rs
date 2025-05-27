//! ZK Coprocessor Types
//!
//! This module defines the core types used for Zero-Knowledge proof generation
//! and management. All types maintain bounded sizes for ZK compatibility.

// Serialization imports removed as we don't use manual SSZ implementations here
use causality_types::utils::SszDuration;
use serde::{Deserialize, Serialize};
use crate::serialization::SszJsonWrapper;

//-----------------------------------------------------------------------------
// ZK Coprocessor Type
//-----------------------------------------------------------------------------

/// Identifier for a ZK coprocessor
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct CoprocessorId(pub [u8; 16]);

/// Identifier for a proof request
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct ProofRequestId(pub String);

/// Status of a proof generation request
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
)]
pub enum ProofStatus {
    /// The request has been received but not yet processed
    Pending,

    /// The proof is being generated
    InProgress,

    /// The proof has been successfully generated
    Completed,

    /// The proof generation failed
    Failed,

    /// The request was rejected
    Rejected,
}

//-----------------------------------------------------------------------------
// Proof Request Type
//-----------------------------------------------------------------------------

/// ZK proof generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofRequest {
    /// The program identifier (often a hash of the WASM or a registered name)
    pub program_id: String,

    /// The witness data for the proof, or data to be processed by get_witnesses
    pub witness: Vec<u8>,

    /// Destination path on the coprocessor's virtual file system for the generated proof
    pub output_vfs_path: String,
}

/// Parameters for a proof request
#[derive(Debug, Clone)]
pub struct ProofRequestParams {
    /// The maximum time to spend generating the proof
    pub timeout: SszDuration,

    /// Priority level (higher is more important)
    pub priority: u8,

    /// Whether to use recursion for the proof
    pub use_recursion: bool,

    /// Optional, arbitrary JSON value for extended parameters to the ZK program's get_witnesses
    pub custom_args: Option<SszJsonWrapper>,
}

//-----------------------------------------------------------------------------
// Proof Data Type
//-----------------------------------------------------------------------------

/// ZK proof data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    /// The raw proof data
    pub data: Vec<u8>,

    /// The program this proof is for
    pub program_id: String,

    /// Verification key for this proof (potentially base64 encoded string or raw bytes)
    pub verification_key: Vec<u8>,

    /// Optional public inputs if they are distinct from the proof data itself
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_inputs: Option<SszJsonWrapper>,
}

//-----------------------------------------------------------------------------
// ZK Coprocessor Config
//-----------------------------------------------------------------------------

/// Configuration for a ZK coprocessor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkCoproConfig {
    /// Unique identifier for this coprocessor
    pub id: CoprocessorId,
    
    /// The base URL for this coprocessor's API
    pub url: String,
    
    /// Authentication token for accessing the coprocessor (if required)
    pub auth_token: Option<String>,
    
    /// Maximum concurrent requests allowed to this coprocessor
    pub max_concurrent_requests: u16,
    
    /// Default timeout for proof generation requests (milliseconds)
    pub default_timeout_ms: u64,
}

//-----------------------------------------------------------------------------
// Public Inputs Type
//-----------------------------------------------------------------------------

/// Public inputs for a ZK proof verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicInputs {
    /// The public input data
    pub data: Vec<u8>,
}

//-----------------------------------------------------------------------------
// Coprocessor API Error Type
//-----------------------------------------------------------------------------

/// Errors that can occur when interacting with the ZK Coprocessor API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoprocessorApiError {
    /// The coprocessor is unavailable or could not be reached.
    Unavailable,
    /// The request was rejected by the coprocessor (e.g., invalid parameters).
    RequestRejected { message: String },
    /// An error occurred during proof generation on the coprocessor side.
    ProofGenerationFailed { message: String },
    /// The requested proof job ID was not found.
    JobNotFound,
    /// The proof data is invalid or corrupted.
    InvalidProofData,
    /// Verification of the proof failed.
    VerificationFailed,
    /// An internal error occurred within the API client or coprocessor.
    InternalError { message: String },
    /// The operation timed out.
    Timeout,
    /// Insufficient resources on the coprocessor to handle the request.
    InsufficientResources,
    /// An unspecified error, with a message.
    Other { message: String },
}

// Helper for String with max length for ssz (example)
// This is a simplified example. A more robust solution might involve a custom type
// that enforces length constraints on construction or during (de)serialization.
// For now, we'll assume consumers will respect indicated max lengths in comments.
