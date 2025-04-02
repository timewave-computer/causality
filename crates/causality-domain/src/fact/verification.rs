// Fact verification interfaces for domains
// Original file: src/domain/fact/verification.rs

// Fact Verification Module for Causality
//
// This module defines the interfaces for fact verification in Causality.

use std::fmt::Debug;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::fact::types::FactType;
use crate::error::Error;

/// Result of a fact verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether the fact has been verified
    pub verified: bool,
    /// Metadata about the verification
    pub metadata: HashMap<String, String>,
    /// An optional error message if the verification failed
    pub error: Option<String>,
}

impl VerificationResult {
    /// Create a new successful verification result
    pub fn success() -> Self {
        Self {
            verified: true,
            metadata: HashMap::new(),
            error: None,
        }
    }
    
    /// Create a new failed verification result
    pub fn failure(error: impl ToString) -> Self {
        Self {
            verified: false,
            metadata: HashMap::new(),
            error: Some(error.to_string()),
        }
    }
    
    /// Check if verification is valid
    pub fn is_valid(&self) -> bool {
        self.verified
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Standardized fact verification result type
pub type FactVerification = VerificationResult;

/// Interface for fact verifiers
pub trait FactVerifier: Send + Sync + Debug {
    /// Get the verifier name
    fn name(&self) -> &str;
    
    /// Verify a fact
    fn verify(&self, fact: &FactType) -> std::result::Result<FactVerification, Error>;
    
    /// Get the verifier metadata
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}
