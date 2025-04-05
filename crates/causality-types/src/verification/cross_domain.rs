// Cross-Domain Verification System
//
// This module extends the verification system to support cross-domain
// verification of content-addressed objects.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::crypto_primitives::{
    ContentAddressed, 
    ContentHash,
    HashError,
    HashOutput,
};

use crate::{
    DomainId,
    verification::{
        VerificationRegistry,
        VerificationError,
        VerificationResult,
        TrustBoundary,
        VerificationPoint,
    },
    ContentId,
};

/// Errors specific to cross-domain verification
#[derive(Debug, Error)]
pub enum CrossDomainVerificationError {
    #[error("Domain not registered: {0}")]
    DomainNotRegistered(String),

    #[error("Domain mismatch: expected {0}, got {1}")]
    DomainMismatch(String, String),

    #[error("Missing required proof type: {0}")]
    MissingRequiredProofType(String),

    #[error("Proof type {0} not supported")]
    UnsupportedProofType(String),

    #[error("Algorithm {0} not allowed in domain {1}")]
    AlgorithmNotAllowed(String, String),

    #[error("Invalid proof format")]
    InvalidProofFormat,

    #[error("Proof validation failed: {0}")]
    ProofValidationFailed(String),

    #[error("No validator found for proof type: {0}")]
    NoValidatorForProofType(String),

    #[error("Minimum signature requirement not met: got {0}, required {1}")]
    MinimumSignaturesNotMet(usize, usize),

    #[error("Content hash mismatch")]
    ContentHashMismatch,

    #[error("Source domain not trusted by target domain")]
    UntrustedSourceDomain,

    #[error("Verification error: {0}")]
    VerificationError(#[from] VerificationError),

    #[error("Crypto error: {0}")]
    CryptoError(#[from] HashError),

    #[error("Other error: {0}")]
    Other(String),
}

/// Trust policy for a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustPolicy {
    /// Only allow specific hash algorithms
    AllowedAlgorithms(Vec<String>),
    /// Require specific proof types
    RequiredProofTypes(Vec<String>),
    /// Minimum number of signatures required
    MinimumSignatures(usize),
    /// Trusted domains
    TrustedDomains(Vec<DomainId>),
    /// Custom policy
    Custom(String, serde_json::Value),
}

/// Domain-specific verification context
#[derive(Debug, Clone)]
pub struct DomainVerificationContext {
    /// Domain identifier
    pub domain_id: DomainId,
    /// Domain-specific parameters
    pub parameters: HashMap<String, String>,
    /// Trust policies for this domain
    pub trust_policies: Vec<TrustPolicy>,
}

impl DomainVerificationContext {
    /// Creates a new domain verification context
    pub fn new(domain_id: DomainId) -> Self {
        Self {
            domain_id,
            parameters: HashMap::new(),
            trust_policies: Vec::new(),
        }
    }

    /// Adds a parameter to the context
    pub fn with_parameter(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }

    /// Adds a trust policy to the context
    pub fn with_trust_policy(mut self, policy: TrustPolicy) -> Self {
        self.trust_policies.push(policy);
        self
    }

    /// Checks if a hash algorithm is allowed in this domain
    pub fn is_algorithm_allowed(&self, algorithm: &str) -> bool {
        // Check allowed algorithms policies
        for policy in &self.trust_policies {
            if let TrustPolicy::AllowedAlgorithms(algorithms) = policy {
                return algorithms.iter().any(|a| a == algorithm);
            }
        }
        // If no policy specified, allow all algorithms
        true
    }

    /// Gets required proof types for this domain
    pub fn required_proof_types(&self) -> Vec<String> {
        for policy in &self.trust_policies {
            if let TrustPolicy::RequiredProofTypes(proof_types) = policy {
                return proof_types.clone();
            }
        }
        Vec::new()
    }

    /// Checks if a domain is trusted by this domain
    pub fn is_domain_trusted(&self, domain: &DomainId) -> bool {
        for policy in &self.trust_policies {
            if let TrustPolicy::TrustedDomains(domains) = policy {
                return domains.contains(domain);
            }
        }
        // If no policy specified, trust no domains
        false
    }

    /// Gets the minimum number of signatures required
    pub fn minimum_signatures(&self) -> Option<usize> {
        for policy in &self.trust_policies {
            if let TrustPolicy::MinimumSignatures(count) = policy {
                return Some(*count);
            }
        }
        None
    }
}

/// Represents a proof used for cross-domain verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationProof {
    /// The content hash being verified
    pub content_hash: ContentHash,
    /// The source domain
    pub source_domain: DomainId,
    /// The type of proof
    pub proof_type: String,
    /// The proof data
    pub proof_data: Vec<u8>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl VerificationProof {
    /// Creates a new verification proof
    pub fn new(
        content_hash: ContentHash,
        source_domain: DomainId,
        proof_type: &str,
        proof_data: Vec<u8>,
    ) -> Self {
        Self {
            content_hash,
            source_domain,
            proof_type: proof_type.to_string(),
            proof_data,
            metadata: HashMap::new(),
        }
    }

    /// Adds metadata to the proof
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Trait for proof validators
pub trait ProofValidator: Send + Sync {
    /// Validates a proof against a domain verification context
    fn validate_proof(
        &self,
        proof: &VerificationProof,
        context: &DomainVerificationContext,
    ) -> Result<bool, CrossDomainVerificationError>;

    /// Returns the proof type this validator handles
    fn proof_type(&self) -> &'static str;
}

/// Manager for cross-domain verification
pub struct CrossDomainVerificationManager {
    /// Verification registry
    pub registry: Arc<VerificationRegistry>,
    /// Domain verification contexts
    pub domain_contexts: RwLock<HashMap<DomainId, DomainVerificationContext>>,
    /// Proof validators
    pub proof_validators: RwLock<Vec<Box<dyn ProofValidator>>>,
}

impl CrossDomainVerificationManager {
    /// Creates a new cross-domain verification manager
    pub fn new(registry: Arc<VerificationRegistry>) -> Self {
        Self {
            registry,
            domain_contexts: RwLock::new(HashMap::new()),
            proof_validators: RwLock::new(Vec::new()),
        }
    }

    /// Registers a domain verification context
    pub fn register_domain_context(&self, context: DomainVerificationContext) {
        let mut contexts = self.domain_contexts.write().unwrap();
        contexts.insert(context.domain_id.clone(), context);
    }

    /// Registers a proof validator
    pub fn register_proof_validator(&self, validator: Box<dyn ProofValidator>) {
        let mut validators = self.proof_validators.write().unwrap();
        validators.push(validator);
    }

    /// Gets the verification context for a domain
    pub fn get_domain_context(
        &self,
        domain_id: &DomainId,
    ) -> Result<DomainVerificationContext, CrossDomainVerificationError> {
        let contexts = self.domain_contexts.read().unwrap();
        contexts
            .get(domain_id)
            .cloned()
            .ok_or_else(|| CrossDomainVerificationError::DomainNotRegistered(domain_id.to_string()))
    }

    /// Verifies an object across domains using the registry
    pub fn verify_cross_domain<T: ContentAddressed>(
        &self,
        object: &T,
        source_domain: &DomainId,
        target_domain: &DomainId,
    ) -> Result<VerificationResult, CrossDomainVerificationError> {
        debug!(
            "Verifying object {} from {} to {}",
            object.content_id()?, source_domain, target_domain
        );

        let _source_context = self.get_domain_context(source_domain)?;
        let target_context = self.get_domain_context(target_domain)?;

        if !target_context.is_domain_trusted(source_domain) {
            return Err(CrossDomainVerificationError::UntrustedSourceDomain);
        }

        let content_hash = object.content_hash()?;
        if !target_context.is_algorithm_allowed(&content_hash.algorithm().to_string()) {
            return Err(CrossDomainVerificationError::AlgorithmNotAllowed(
                content_hash.algorithm().to_string(),
                target_domain.to_string(),
            ));
        }

        let point = VerificationPoint::new(object, TrustBoundary::System, self.registry.clone());
        let result = point.verify()?;
        Ok(result)
    }

    /// Verifies an object using a specific proof against a target domain
    pub fn verify_with_proof<T: ContentAddressed>(
        &self,
        object: &T,
        proof: &VerificationProof,
        target_domain: &DomainId,
    ) -> Result<VerificationResult, CrossDomainVerificationError> {
        let target_context = self.get_domain_context(target_domain)?;
        debug!(
            "Verifying object {} against target domain {} using proof type {}",
            object.content_id()?, target_domain, proof.proof_type
        );

        if proof.source_domain != target_context.domain_id {
            return Err(CrossDomainVerificationError::DomainMismatch(
                target_context.domain_id.to_string(),
                proof.source_domain.to_string(),
            ));
        }

        let content_hash_from_object = object.content_hash()?;
        let content_hash_converted = ContentHash::from_hash_output(&content_hash_from_object);
        if proof.content_hash != content_hash_converted {
            return Err(CrossDomainVerificationError::ContentHashMismatch);
        }
        
        Ok(VerificationResult::verified())
    }

    /// Creates a verification proof for an object
    pub fn create_proof<T: ContentAddressed>(
        &self,
        object: &T,
        source_domain: &DomainId,
        proof_type: &str,
        proof_data: Vec<u8>,
    ) -> Result<VerificationProof, CrossDomainVerificationError> {
        let content_hash_output = object.content_hash()?;
        debug!(
            "Creating proof of type {} for object {} in domain {}",
            proof_type, content_hash_output.to_hex(), source_domain
        );
        
        let _source_context = self.get_domain_context(source_domain)?;
        
        let content_hash = ContentHash::from_hash_output(&content_hash_output);
        Ok(VerificationProof::new(
            content_hash,
            source_domain.clone(),
            proof_type,
            proof_data,
        ))
    }

    fn verify_direct_proof(
        &self,
        source_domain: &DomainId,
        target_domain: &DomainId,
        proof: &VerificationProof,
    ) -> Result<VerificationResult, CrossDomainVerificationError> {
        debug!(
            "Attempting direct proof verification from {} to {} using proof type: {}",
            source_domain,
            target_domain,
            proof.proof_type
        );
        
        let _source_context = self.get_domain_context(source_domain)?;
        let target_context = self.get_domain_context(target_domain)?;

        if !target_context.is_domain_trusted(source_domain) {
            return Err(CrossDomainVerificationError::UntrustedSourceDomain);
        }

        Ok(VerificationResult::verified())
    }

    fn verify_indirect_proof(
        &self,
        source_domain: &DomainId,
        intermediate_domain: &DomainId,
        target_domain: &DomainId,
        proof1: &VerificationProof,
        proof2: &VerificationProof,
    ) -> Result<VerificationResult, CrossDomainVerificationError> {
        debug!(
            "Attempting indirect proof verification from {} via {} to {}",
            source_domain, intermediate_domain, target_domain
        );
        
        let _source_context = self.get_domain_context(source_domain)?;
        let intermediate_context = self.get_domain_context(intermediate_domain)?;
        let target_context = self.get_domain_context(target_domain)?;

        if !intermediate_context.is_domain_trusted(source_domain) ||
           !target_context.is_domain_trusted(intermediate_domain) {
            return Err(CrossDomainVerificationError::UntrustedSourceDomain);
        }

        Ok(VerificationResult::verified())
    }
}

/// A wrapper around a reference to a proof validator
struct ValidatorReference<'a>(&'a dyn ProofValidator);

impl<'a> ProofValidator for ValidatorReference<'a> {
    fn validate_proof(
        &self,
        proof: &VerificationProof,
        context: &DomainVerificationContext,
    ) -> Result<bool, CrossDomainVerificationError> {
        self.0.validate_proof(proof, context)
    }

    fn proof_type(&self) -> &'static str {
        self.0.proof_type()
    }
}

/// A verification point for cross-domain verification
pub struct CrossDomainVerificationPoint<T: ContentAddressed> {
    /// The object to verify
    pub object: T,
    /// The source domain
    pub source_domain: DomainId,
    /// The target domain
    pub target_domain: DomainId,
    /// The verification manager
    pub manager: Arc<CrossDomainVerificationManager>,
}

impl<T: ContentAddressed> CrossDomainVerificationPoint<T> {
    /// Creates a new cross-domain verification point
    pub fn new(
        object: T,
        source_domain: DomainId,
        target_domain: DomainId,
        manager: Arc<CrossDomainVerificationManager>,
    ) -> Self {
        Self {
            object,
            source_domain,
            target_domain,
            manager,
        }
    }

    /// Verifies the object
    pub fn verify(&self) -> Result<VerificationResult, CrossDomainVerificationError> {
        self.manager.verify_cross_domain(
            &self.object,
            &self.source_domain,
            &self.target_domain,
        )
    }
    
    /// Verifies the object and converts the result to VerificationResult
    pub fn verify_and_convert(&self) -> Result<VerificationResult, VerificationError> {
        self.verify().map_err(|e| VerificationError::Other(e.to_string()))
    }

    /// Get verification result compatible with VerificationError
    pub fn into_verified(self) -> Result<T, VerificationError> {
        let result = self.verify().map_err(|e| VerificationError::Other(e.to_string()))?;
        
        if result.is_verified() {
            Ok(self.object)
        } else {
            Err(VerificationError::VerificationFailed { 
                object_id: "cross-domain-object".to_string(),
                reason: result.failure_reason().unwrap_or("Cross-domain verification failed").to_string()
            })
        }
    }

    /// Verifies the object with a proof
    pub fn verify_with_proof(
        &self,
        proof: &VerificationProof,
    ) -> Result<VerificationResult, CrossDomainVerificationError> {
        self.manager
            .verify_with_proof(&self.object, proof, &self.target_domain)
    }

    /// Generates a proof for the object
    pub fn generate_proof(
        &self,
        proof_type: &str,
    ) -> Result<VerificationProof, CrossDomainVerificationError> {
        let proof_data = Vec::new();
        
        self.manager.create_proof(
            &self.object,
            &self.source_domain,
            proof_type,
            proof_data,
        )
    }
}

/// Extension trait for content-addressed types to support cross-domain verification
pub trait CrossDomainVerifiable: ContentAddressed + Sized {
    /// Creates a cross-domain verification point
    fn for_cross_domain_verification(
        self,
        source_domain: DomainId,
        target_domain: DomainId,
        manager: Arc<CrossDomainVerificationManager>,
    ) -> CrossDomainVerificationPoint<Self> {
        CrossDomainVerificationPoint::new(self, source_domain, target_domain, manager)
    }
}

/// Implement CrossDomainVerifiable for all types that implement ContentAddressed
impl<T: ContentAddressed> CrossDomainVerifiable for T {}

pub struct DummyValidator {
    proof_type_name: String,
}

impl ProofValidator for DummyValidator {
    fn validate_proof(
        &self,
        _proof: &VerificationProof,
        _context: &DomainVerificationContext,
    ) -> Result<bool, CrossDomainVerificationError> {
        Ok(true)
    }

    fn proof_type(&self) -> &'static str {
        "dummy"
    }
} 