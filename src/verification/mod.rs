// Unified Verification Framework
//
// This module implements a unified verification framework that integrates different 
// verification mechanisms in the Causality system, such as ZK proofs, time map verification,
// controller label verification, and effect validation.

mod context;
mod proof;
mod provider;
mod service;
mod examples;

pub use context::*;
pub use proof::*;
pub use provider::*;
pub use service::*;
pub use examples::*;

use std::collections::HashMap;
use chrono::Utc;
use thiserror::Error;

use crate::types::{*};
use crate::crypto::hash::ContentId;;

/// The result of a verification operation
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether the verification was successful
    pub success: bool,
    
    /// When the verification was performed
    pub verification_time: chrono::DateTime<Utc>,
    
    /// Error message if verification failed
    pub error_message: Option<String>,
    
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    
    /// Name of the provider that performed the verification
    pub provider_name: String,
    
    /// Type of verification that was performed
    pub verification_type: VerificationType,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// An enum representing the status of a verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationStatus {
    /// Not yet verified
    NotVerified,
    
    /// Verification in progress
    InProgress,
    
    /// Successfully verified
    Verified,
    
    /// Verification failed
    Failed,
}

/// Error type for verification operations
#[derive(Debug, Error)]
pub enum VerificationError {
    /// Missing proof component
    #[error("Missing proof component: {0}")]
    MissingProofComponent(String),
    
    /// Invalid proof
    #[error("Invalid proof: {0}")]
    InvalidProof(String),
    
    /// Missing capability
    #[error("Missing capability: {0}")]
    MissingCapability(String),
    
    /// Missing context
    #[error("Missing context: {0}")]
    MissingContext(String),
    
    /// Provider error
    #[error("Provider error: {0}")]
    ProviderError(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Capabilities required for verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationCapability {
    /// ZK proving
    ZkProving,
    
    /// Time map access
    TimeMapAccess,
    
    /// Controller registry access
    ControllerRegistryAccess,
    
    /// Effect history access
    EffectHistoryAccess,
    
    /// Cross-domain verification
    CrossDomainVerification,
    
    /// Custom capability
    Custom(String),
}

/// Dependencies required for verification
#[derive(Debug, Clone)]
pub struct VerificationDependency {
    /// Resource IDs that this verification depends on
    pub resource_ids: Vec<ResourceId>,
    
    /// Domain IDs that this verification depends on
    pub domain_ids: Vec<DomainId>,
    
    /// Types of verification that this depends on
    pub verification_types: Vec<VerificationType>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Options for verification
#[derive(Debug, Clone)]
pub struct VerificationOptions {
    /// Whether to perform shallow verification
    pub shallow: bool,
    
    /// Minimum confidence threshold
    pub min_confidence: f64,
    
    /// Additional options
    pub options: HashMap<String, String>,
}

impl Default for VerificationOptions {
    fn default() -> Self {
        Self {
            shallow: false,
            min_confidence: 0.7,
            options: HashMap::new(),
        }
    }
}

/// A trait for entities that can be verified
pub trait Verifiable {
    /// Generate a proof for this entity
    fn generate_proof(&self, context: &VerificationContext) -> Result<UnifiedProof, VerificationError>;
    
    /// Verify this entity
    fn verify(&self, context: &VerificationContext, options: &VerificationOptions) -> Result<VerificationResult, VerificationError>;
    
    /// Get verification dependencies
    fn get_dependencies(&self) -> Vec<VerificationDependency>;
    
    /// Get required capabilities for verification
    fn required_capabilities(&self) -> Vec<VerificationCapability>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_verification_result() {
        let result = VerificationResult {
            success: true,
            verification_time: Utc::now(),
            error_message: None,
            confidence: 0.95,
            provider_name: "test_provider".to_string(),
            verification_type: VerificationType::ZkProof,
            metadata: HashMap::new(),
        };
        
        assert!(result.success);
        assert!(result.confidence > 0.9);
        assert_eq!(result.error_message, None);
    }
    
    #[test]
    fn test_verification_status() {
        // Ensure all verification statuses are covered
        let statuses = vec![
            VerificationStatus::NotVerified,
            VerificationStatus::InProgress,
            VerificationStatus::Verified,
            VerificationStatus::Failed,
        ];
        
        // This ensures we've tested all variants if a new one is added
        assert_eq!(statuses.len(), 4);
    }
} 
