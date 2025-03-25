// Capability proof structures
// Original file: src/resource/capability/proof.rs

//! Capability proof system for cross-system authorization
//!
//! This module provides a capability proof system that enables secure
//! authorization across different systems without requiring direct access
//! to the capability repository.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use serde::{Serialize, Deserialize};

use causality_types::Address;
use crate::resource::{
    ResourceId, Right,
    capability::{
        CapabilityId, CapabilityRepository, Capability,
        validation::{CapabilityValidator, CapabilityValidationResult}
    }
};
use causality_types::{Error, Result};

/// Capability proof for cross-system authorization
///
/// A capability proof is a portable representation of a capability that
/// can be verified without access to the full capability chain.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapabilityProof {
    /// ID of the capability this proof represents
    pub capability_id: CapabilityId,
    
    /// Owner of the capability
    pub owner: Address,
    
    /// Resource ID the capability grants access to
    pub resource_id: ContentId,
    
    /// Right granted by the capability
    pub right: Right,
    
    /// When the proof was created
    pub created_at: SystemTime,
    
    /// Expiration time for the proof
    pub expires_at: Option<SystemTime>,
    
    /// Verification hash derived from the capability chain
    pub verification_hash: String,
    
    /// Additional metadata for the proof
    pub metadata: HashMap<String, String>,
    
    /// Signature by the proof issuer
    pub signature: Option<String>,
}

impl CapabilityProof {
    /// Create a new capability proof
    pub fn new(
        capability_id: CapabilityId,
        owner: Address,
        resource_id: ContentId,
        right: Right,
        verification_hash: String,
        ttl: Option<Duration>,
    ) -> Self {
        let now = SystemTime::now();
        let expires_at = ttl.map(|duration| now + duration);
        
        Self {
            capability_id,
            owner,
            resource_id,
            right,
            created_at: now,
            expires_at,
            verification_hash,
            metadata: HashMap::new(),
            signature: None,
        }
    }
    
    /// Check if the proof has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            SystemTime::now() > expires_at
        } else {
            false
        }
    }
    
    /// Add metadata to the proof
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Set the signature for the proof
    pub fn with_signature(mut self, signature: &str) -> Self {
        self.signature = Some(signature.to_string());
        self
    }
    
    /// Serialize the proof to a string
    pub fn serialize(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| Error::SerializationError(format!("Failed to serialize proof: {}", e)))
    }
    
    /// Deserialize from a string to a proof
    pub fn deserialize(data: &str) -> Result<Self> {
        serde_json::from_str(data)
            .map_err(|e| Error::SerializationError(format!("Failed to deserialize proof: {}", e)))
    }
}

impl fmt::Display for CapabilityProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CapabilityProof({}, {}, {})", 
            self.capability_id, self.owner, self.resource_id)
    }
}

/// Capability proof manager for creating and verifying proofs
pub struct ProofManager {
    /// Capability validator for validating capabilities
    validator: CapabilityValidator,
    
    /// Secret key for signing proofs (in a real implementation, this would be more secure)
    secret_key: String,
    
    /// Default TTL for proofs if not specified
    default_ttl: Duration,
}

impl ProofManager {
    /// Create a new proof manager
    pub fn new(
        repository: Arc<dyn CapabilityRepository>,
        secret_key: String,
        default_ttl: Duration,
    ) -> Self {
        let validator = CapabilityValidator::new(repository);
        Self { validator, secret_key, default_ttl }
    }
    
    /// Issue a capability proof
    pub fn issue_proof(
        &self,
        capability_id: &CapabilityId,
        owner: &Address,
        ttl: Option<Duration>,
    ) -> Result<CapabilityProof> {
        // 1. Validate the capability
        let validation = self.validator.validate_capability(capability_id, owner)?;
        
        if !validation.valid {
            let reason = validation.reason.unwrap_or_else(|| "Unknown validation failure".to_string());
            return Err(Error::CapabilityError(
                format!("Cannot issue proof for invalid capability: {}", reason)
            ));
        }
        
        // 2. Get the capability chain
        let chain = validation.capability_chain.ok_or_else(|| {
            Error::CapabilityError("Missing capability chain in validation result".to_string())
        })?;
        
        // 3. Get the root capability
        let root = chain.first().ok_or_else(|| {
            Error::CapabilityError("Empty capability chain".to_string())
        })?;
        
        // 4. Create the verification hash
        let verification_hash = self.compute_verification_hash(&chain);
        
        // 5. Create the proof
        let mut proof = CapabilityProof::new(
            capability_id.clone(),
            owner.clone(),
            root.resource_id.clone(),
            root.right,
            verification_hash,
            ttl.or(Some(self.default_ttl)),
        );
        
        // 6. Add some metadata
        proof = proof
            .with_metadata("chain_length", &chain.len().to_string())
            .with_metadata("root_capability", &root.id.to_string());
        
        // 7. Sign the proof
        let signature = self.sign_proof(&proof)?;
        proof = proof.with_signature(&signature);
        
        Ok(proof)
    }
    
    /// Verify a capability proof
    pub fn verify_proof(&self, proof: &CapabilityProof) -> Result<bool> {
        // 1. Check if the proof has expired
        if proof.is_expired() {
            return Ok(false);
        }
        
        // 2. Verify the signature
        if let Some(signature) = &proof.signature {
            let is_valid = self.verify_signature(proof, signature)?;
            if !is_valid {
                return Ok(false);
            }
        } else {
            return Ok(false); // No signature, invalid proof
        }
        
        // 3. If we have access to the validator, verify the capability is still valid
        let validation = self.validator.validate_capability(&proof.capability_id, &proof.owner)?;
        
        if !validation.valid {
            return Ok(false);
        }
        
        // 4. If we have the chain, verify the hash
        if let Some(chain) = validation.capability_chain {
            let computed_hash = self.compute_verification_hash(&chain);
            if computed_hash != proof.verification_hash {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Compute a verification hash for a capability chain
    fn compute_verification_hash(&self, chain: &Vec<Capability>) -> String {
        // In a real implementation, this would be a cryptographic hash
        // For demonstration purposes, we'll create a simple hash
        let mut hash_input = String::new();
        
        for capability in chain {
            hash_input.push_str(&format!(
                "{}:{}:{}:",
                capability.id,
                capability.resource_id,
                capability.right,
            ));
        }
        
        hash_input.push_str(&self.secret_key);
        
        // Simple hash function for demo
        format!("PROOF_HASH[{}]", hash_input)
    }
    
    /// Sign a capability proof
    fn sign_proof(&self, proof: &CapabilityProof) -> Result<String> {
        // In a real implementation, this would use a cryptographic signature
        // For demonstration purposes, we'll create a simple signature
        let mut signature_input = String::new();
        
        signature_input.push_str(&format!(
            "{}:{}:{}:{}:{}",
            proof.capability_id,
            proof.owner,
            proof.resource_id,
            proof.right,
            proof.verification_hash,
        ));
        
        if let Some(expires) = proof.expires_at {
            // Format as ISO string - in reality, use a proper timestamp
            signature_input.push_str(&format!(":{:?}", expires));
        }
        
        signature_input.push_str(&self.secret_key);
        
        // Simple signature for demo
        Ok(format!("SIG[{}]", signature_input))
    }
    
    /// Verify a signature
    fn verify_signature(&self, proof: &CapabilityProof, signature: &str) -> Result<bool> {
        // In a real implementation, this would verify a cryptographic signature
        // For demonstration purposes, we'll recreate the signature and compare
        let expected = self.sign_proof(proof)?;
        Ok(expected == signature)
    }
    
    /// Renew a capability proof with a new expiration
    pub fn renew_proof(
        &self,
        proof: &CapabilityProof,
        new_ttl: Duration,
    ) -> Result<CapabilityProof> {
        // 1. Verify the proof is valid
        if !self.verify_proof(proof)? {
            return Err(Error::CapabilityError(
                format!("Cannot renew invalid proof")
            ));
        }
        
        // 2. Create a new proof with a new expiration
        self.issue_proof(&proof.capability_id, &proof.owner, Some(new_ttl))
    }
    
    /// Create a proof token (compact representation for transmission)
    pub fn create_proof_token(&self, proof: &CapabilityProof) -> Result<String> {
        // 1. Serialize the proof
        let serialized = proof.serialize()?;
        
        // 2. In a real implementation, we might compress, encrypt, or encode
        // For demo purposes, just return the serialized version
        Ok(serialized)
    }
    
    /// Verify a proof token and reconstruct the proof
    pub fn verify_proof_token(&self, token: &str) -> Result<CapabilityProof> {
        // 1. Deserialize the token to a proof
        let proof = CapabilityProof::deserialize(token)?;
        
        // 2. Verify the proof
        if !self.verify_proof(&proof)? {
            return Err(Error::CapabilityError(
                format!("Invalid proof token")
            ));
        }
        
        Ok(proof)
    }
}

/// Remote proof verifier for systems without direct access to the capability repository
pub struct RemoteProofVerifier {
    /// Secret key for verifying proof signatures
    secret_key: String,
}

impl RemoteProofVerifier {
    /// Create a new remote proof verifier
    pub fn new(secret_key: String) -> Self {
        Self { secret_key }
    }
    
    /// Verify a proof token without access to the capability repository
    pub fn verify_token(&self, token: &str) -> Result<CapabilityProof> {
        // 1. Deserialize the token to a proof
        let proof = CapabilityProof::deserialize(token)?;
        
        // 2. Check if the proof has expired
        if proof.is_expired() {
            return Err(Error::CapabilityError(
                format!("Proof has expired")
            ));
        }
        
        // 3. Verify the signature (simplified for demo)
        if let Some(signature) = &proof.signature {
            let is_valid = self.verify_signature(&proof, signature)?;
            if !is_valid {
                return Err(Error::CapabilityError(
                    format!("Invalid proof signature")
                ));
            }
        } else {
            return Err(Error::CapabilityError(
                format!("Proof has no signature")
            ));
        }
        
        Ok(proof)
    }
    
    /// Verify a signature (simplified for demo)
    fn verify_signature(&self, proof: &CapabilityProof, signature: &str) -> Result<bool> {
        // In a real implementation, this would verify a cryptographic signature
        // For demonstration purposes, we'll recreate a simple signature
        
        let mut signature_input = String::new();
        
        signature_input.push_str(&format!(
            "{}:{}:{}:{}:{}",
            proof.capability_id,
            proof.owner,
            proof.resource_id,
            proof.right,
            proof.verification_hash,
        ));
        
        if let Some(expires) = proof.expires_at {
            signature_input.push_str(&format!(":{:?}", expires));
        }
        
        signature_input.push_str(&self.secret_key);
        
        // Compare with the provided signature
        let expected = format!("SIG[{}]", signature_input);
        Ok(expected == signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_resource::MockCapabilityRepository;
    
    fn create_test_setup() -> (
        ProofManager,
        Arc<MockCapabilityRepository>,
        Address,
        ResourceId,
        Right,
    ) {
        let repo = Arc::new(MockCapabilityRepository::new());
        let manager = ProofManager::new(
            repo.clone(),
            "test-secret-key".to_string(),
            Duration::from_secs(3600), // 1 hour default TTL
        );
        
        let owner = Address::new("test-owner");
        let resource_id = ResourceId::new("test-resource");
        let right = Right::Read;
        
        (manager, repo, owner, resource_id, right)
    }
    
    #[test]
    fn test_issue_and_verify_proof() -> Result<()> {
        let (manager, repo, owner, resource_id, right) = create_test_setup();
        
        // Create a capability
        let capability = Capability::new_root(
            owner.clone(),
            resource_id.clone(),
            right,
            "Test capability".to_string(),
        );
        
        let capability_id = capability.id.clone();
        repo.store_capability(capability)?;
        
        // Issue a proof
        let proof = manager.issue_proof(
            &capability_id,
            &owner,
            Some(Duration::from_secs(60)),  // 1 minute TTL
        )?;
        
        // Verify the proof
        assert!(manager.verify_proof(&proof)?);
        
        // Check the proof properties
        assert_eq!(proof.capability_id, capability_id);
        assert_eq!(proof.owner, owner);
        assert_eq!(proof.resource_id, resource_id);
        assert_eq!(proof.right, right);
        assert!(proof.signature.is_some());
        
        Ok(())
    }
    
    #[test]
    fn test_proof_token() -> Result<()> {
        let (manager, repo, owner, resource_id, right) = create_test_setup();
        
        // Create a capability
        let capability = Capability::new_root(
            owner.clone(),
            resource_id.clone(),
            right,
            "Test capability".to_string(),
        );
        
        let capability_id = capability.id.clone();
        repo.store_capability(capability)?;
        
        // Issue a proof
        let proof = manager.issue_proof(
            &capability_id,
            &owner,
            Some(Duration::from_secs(60)),
        )?;
        
        // Create a token
        let token = manager.create_proof_token(&proof)?;
        
        // Verify the token
        let verified_proof = manager.verify_proof_token(&token)?;
        
        // Check that the verification worked
        assert_eq!(verified_proof.capability_id, capability_id);
        assert_eq!(verified_proof.owner, owner);
        assert_eq!(verified_proof.resource_id, resource_id);
        
        Ok(())
    }
    
    #[test]
    fn test_remote_verification() -> Result<()> {
        let (manager, repo, owner, resource_id, right) = create_test_setup();
        
        // Create a capability
        let capability = Capability::new_root(
            owner.clone(),
            resource_id.clone(),
            right,
            "Test capability".to_string(),
        );
        
        let capability_id = capability.id.clone();
        repo.store_capability(capability)?;
        
        // Issue a proof
        let proof = manager.issue_proof(
            &capability_id,
            &owner,
            Some(Duration::from_secs(60)),
        )?;
        
        // Create a token
        let token = manager.create_proof_token(&proof)?;
        
        // Create a remote verifier with the same secret key
        let remote_verifier = RemoteProofVerifier::new("test-secret-key".to_string());
        
        // Verify the token remotely
        let verified_proof = remote_verifier.verify_token(&token)?;
        
        // Check that the verification worked
        assert_eq!(verified_proof.capability_id, capability_id);
        assert_eq!(verified_proof.owner, owner);
        assert_eq!(verified_proof.resource_id, resource_id);
        
        Ok(())
    }
    
    #[test]
    fn test_expired_proof() -> Result<()> {
        let (manager, repo, owner, resource_id, right) = create_test_setup();
        
        // Create a capability
        let capability = Capability::new_root(
            owner.clone(),
            resource_id.clone(),
            right,
            "Test capability".to_string(),
        );
        
        let capability_id = capability.id.clone();
        repo.store_capability(capability)?;
        
        // Create a proof that's already expired
        let mut proof = manager.issue_proof(
            &capability_id,
            &owner,
            Some(Duration::from_secs(60)),
        )?;
        
        // Manually set the expiration to the past
        proof.expires_at = Some(SystemTime::now() - Duration::from_secs(10));
        
        // Verify the proof (should fail)
        assert!(!manager.verify_proof(&proof)?);
        
        Ok(())
    }
} 
