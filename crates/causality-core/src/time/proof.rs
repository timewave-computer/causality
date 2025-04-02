// Placeholder for time proof types

use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ProofError {
    #[error("Invalid proof format: {0}")]
    InvalidFormat(String),
    #[error("Proof verification failed: {0}")]
    VerificationFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeProof {
    // Placeholder fields
    pub proof_data: Vec<u8>,
    pub signature: String,
} 