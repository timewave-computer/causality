use std::fmt::{Display, Formatter};
use thiserror::Error;

/// Errors that can occur during verification
#[derive(Error, Debug)]
pub enum VerificationError {
    /// Hash computation error
    #[error("Failed to compute hash: {0}")]
    HashError(String),

    /// Hash mismatch error
    #[error("Hash mismatch for {object_id}: expected {expected}, got {actual}")]
    HashMismatch {
        /// The object ID being verified
        object_id: String,
        /// The expected hash
        expected: String,
        /// The actual hash
        actual: String,
    },

    /// Verification failed error
    #[error("Verification failed for {object_id}: {reason}")]
    VerificationFailed {
        /// The object ID being verified
        object_id: String,
        /// The reason for the failure
        reason: String,
    },

    /// Missing proof error
    #[error("Missing required proof for {object_id}: {proof_type}")]
    MissingProof {
        /// The object ID being verified
        object_id: String,
        /// The required proof type
        proof_type: String,
    },

    /// Trust boundary error
    #[error("Trust boundary violation: {0}")]
    TrustBoundaryViolation(String),

    /// Other verification error
    #[error("Verification error: {0}")]
    Other(String),
}

/// Result of a verification operation
#[derive(Debug)]
pub enum VerificationResult {
    /// Verification succeeded
    Verified,

    /// Verification failed
    Failed {
        /// The reason for the failure
        reason: String,
    },
}

impl VerificationResult {
    /// Creates a verified result
    pub fn verified() -> Self {
        Self::Verified
    }

    /// Creates a failed result
    pub fn failed(reason: String) -> Self {
        Self::Failed { reason }
    }

    /// Returns true if the verification succeeded
    pub fn is_verified(&self) -> bool {
        matches!(self, Self::Verified)
    }

    /// Returns the failure reason if the verification failed
    pub fn failure_reason(&self) -> Option<&str> {
        match self {
            Self::Failed { reason } => Some(reason),
            _ => None,
        }
    }
}

impl Display for VerificationResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Verified => write!(f, "Verified"),
            Self::Failed { reason } => write!(f, "Failed: {}", reason),
        }
    }
} 