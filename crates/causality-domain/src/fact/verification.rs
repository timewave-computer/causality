// Fact verification for domains
// Original file: src/domain/fact/verification.rs

// Fact Verification Module for Causality
//
// This module defines the interfaces and types for verifying facts.

use async_trait::async_trait;
use std::fmt::Debug;

use causality_types::Result;
use causality_engine_types::FactType;

/// Result of a fact verification
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether the fact has been verified
    pub verified: bool,
    /// The verification method used
    pub method: String,
    /// A confidence score between 0.0 and 1.0
    pub confidence: f64,
    /// An optional error message if the verification failed
    pub error: Option<String>,
}

/// Interface for fact verifiers
#[async_trait]
pub trait FactVerifier: Send + Sync + Debug {
    /// Check if the verifier can verify a specific fact
    fn can_verify(&self, fact: &FactType) -> bool;
    
    /// Verify a fact
    async fn verify(&self, fact: &FactType) -> Result<VerificationResult>;
}
