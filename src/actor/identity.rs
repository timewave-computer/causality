// Actor Identity Module
//
// This module provides identity management for actors in the Causality system.
// It includes verification, authentication, and identity proof mechanisms.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};
use crate::types::{ContentHash, ContentId, TraceId};
use crate::actor::{ActorId, ActorMetadata, ActorRole, ActorCapability, ActorState};

/// Identity provider types for actor authentication
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdentityProvider {
    /// Local identity provider
    Local,
    /// OAuth-based identity provider
    OAuth(String),
    /// JWT-based identity provider
    JWT(String),
    /// Public key-based identity provider
    PublicKey,
    /// Custom identity provider
    Custom(String),
}

/// Identity verification status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// Identity is unverified
    Unverified,
    /// Identity is pending verification
    Pending,
    /// Identity is verified
    Verified,
    /// Identity verification failed
    Failed(String),
}

/// Identity credential for an actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityCredential {
    /// The actor ID this credential belongs to
    pub actor_id: ActorId,
    /// The identity provider for this credential
    pub provider: IdentityProvider,
    /// The credential ID
    pub credential_id: String,
    /// The verification status of this credential
    pub verification_status: VerificationStatus,
    /// When this credential was issued
    pub issued_at: u64,
    /// When this credential expires (if applicable)
    pub expires_at: Option<u64>,
    /// Additional attributes associated with this credential
    pub attributes: HashMap<String, String>,
}

impl IdentityCredential {
    /// Create a new identity credential
    pub fn new(
        actor_id: ActorId,
        provider: IdentityProvider,
        credential_id: impl Into<String>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        IdentityCredential {
            actor_id,
            provider,
            credential_id: credential_id.into(),
            verification_status: VerificationStatus::Unverified,
            issued_at: now,
            expires_at: None,
            attributes: HashMap::new(),
        }
    }
    
    /// Set the expiration time for this credential
    pub fn with_expiration(mut self, expires_at: u64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Add an attribute to this credential
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
    
    /// Check if this credential is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            now > expires_at
        } else {
            false
        }
    }
    
    /// Check if this credential is verified
    pub fn is_verified(&self) -> bool {
        self.verification_status == VerificationStatus::Verified
    }
    
    /// Mark this credential as verified
    pub fn mark_verified(&mut self) {
        self.verification_status = VerificationStatus::Verified;
    }
    
    /// Mark this credential as failed verification
    pub fn mark_failed(&mut self, reason: impl Into<String>) {
        self.verification_status = VerificationStatus::Failed(reason.into());
    }
    
    /// Get a content ID for this credential
    pub fn content_id(&self) -> ContentId {
        let hash_str = format!(
            "{}:{}:{}:{}",
            self.actor_id.as_str(),
            format!("{:?}", self.provider),
            self.credential_id,
            self.issued_at
        );
        
        let hash = ContentHash::new(&hash_str);
        ContentId::new(hash, "identity-credential")
    }
}

/// Identity provider trait for verifying credentials
#[async_trait]
pub trait IdentityVerifier: Send + Sync + Debug {
    /// Get the provider type for this verifier
    fn provider_type(&self) -> IdentityProvider;
    
    /// Verify an identity credential
    async fn verify(&self, credential: &mut IdentityCredential) -> Result<bool>;
}

/// Local identity verifier implementation
#[derive(Debug)]
pub struct LocalIdentityVerifier {
    /// Trusted credentials
    trusted_credentials: RwLock<HashMap<String, String>>,
}

impl LocalIdentityVerifier {
    /// Create a new local identity verifier
    pub fn new() -> Self {
        LocalIdentityVerifier {
            trusted_credentials: RwLock::new(HashMap::new()),
        }
    }
    
    /// Add a trusted credential
    pub fn add_trusted_credential(
        &self,
        credential_id: impl Into<String>,
        secret: impl Into<String>,
    ) -> Result<()> {
        let mut trusted = self.trusted_credentials.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on trusted credentials".to_string())
        })?;
        
        trusted.insert(credential_id.into(), secret.into());
        Ok(())
    }
}

#[async_trait]
impl IdentityVerifier for LocalIdentityVerifier {
    fn provider_type(&self) -> IdentityProvider {
        IdentityProvider::Local
    }
    
    async fn verify(&self, credential: &mut IdentityCredential) -> Result<bool> {
        // For local credentials, verification is based on the credential existing
        // in our trusted credentials store
        let trusted = self.trusted_credentials.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on trusted credentials".to_string())
        })?;
        
        let is_trusted = trusted.contains_key(&credential.credential_id);
        
        if is_trusted {
            credential.mark_verified();
            Ok(true)
        } else {
            credential.mark_failed("Credential not found in trusted store");
            Ok(false)
        }
    }
}

/// Identity service for managing actor identities
#[derive(Debug)]
pub struct IdentityService {
    /// Identity verifiers
    verifiers: RwLock<HashMap<IdentityProvider, Arc<dyn IdentityVerifier>>>,
    /// Actor credentials
    credentials: RwLock<HashMap<ActorId, Vec<IdentityCredential>>>,
}

impl IdentityService {
    /// Create a new identity service
    pub fn new() -> Self {
        IdentityService {
            verifiers: RwLock::new(HashMap::new()),
            credentials: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register an identity verifier
    pub fn register_verifier(&self, verifier: Arc<dyn IdentityVerifier>) -> Result<()> {
        let provider_type = verifier.provider_type();
        
        let mut verifiers = self.verifiers.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on verifiers".to_string())
        })?;
        
        verifiers.insert(provider_type, verifier);
        Ok(())
    }
    
    /// Add a credential for an actor
    pub fn add_credential(&self, credential: IdentityCredential) -> Result<()> {
        let actor_id = credential.actor_id.clone();
        
        let mut credentials = self.credentials.write().map_err(|_| {
            Error::LockError("Failed to acquire write lock on credentials".to_string())
        })?;
        
        let actor_credentials = credentials.entry(actor_id).or_insert_with(Vec::new);
        actor_credentials.push(credential);
        
        Ok(())
    }
    
    /// Get all credentials for an actor
    pub fn get_credentials(&self, actor_id: &ActorId) -> Result<Vec<IdentityCredential>> {
        let credentials = self.credentials.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on credentials".to_string())
        })?;
        
        Ok(credentials.get(actor_id).cloned().unwrap_or_default())
    }
    
    /// Verify a credential
    pub async fn verify_credential(&self, credential: &mut IdentityCredential) -> Result<bool> {
        let verifiers = self.verifiers.read().map_err(|_| {
            Error::LockError("Failed to acquire read lock on verifiers".to_string())
        })?;
        
        let verifier = verifiers.get(&credential.provider).ok_or_else(|| {
            Error::NotFound(format!(
                "No verifier found for provider: {:?}",
                credential.provider
            ))
        })?;
        
        verifier.verify(credential).await
    }
    
    /// Verify all credentials for an actor
    pub async fn verify_all_credentials(&self, actor_id: &ActorId) -> Result<bool> {
        let mut all_valid = true;
        
        // Get a mutable copy of the credentials
        let mut credentials_copy = self.get_credentials(actor_id)?;
        
        for credential in &mut credentials_copy {
            let result = self.verify_credential(credential).await?;
            if !result {
                all_valid = false;
            }
        }
        
        // Update the stored credentials with the verification results
        if !credentials_copy.is_empty() {
            let mut credentials = self.credentials.write().map_err(|_| {
                Error::LockError("Failed to acquire write lock on credentials".to_string())
            })?;
            
            if let Some(actor_credentials) = credentials.get_mut(actor_id) {
                *actor_credentials = credentials_copy;
            }
        }
        
        Ok(all_valid)
    }
}

impl Default for IdentityService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_identity_credential() {
        let actor_id = ActorId::new("test-actor");
        let provider = IdentityProvider::Local;
        
        let credential = IdentityCredential::new(
            actor_id.clone(),
            provider.clone(),
            "test-credential",
        )
        .with_attribute("key1", "value1")
        .with_attribute("key2", "value2");
        
        assert_eq!(credential.actor_id, actor_id);
        assert_eq!(credential.provider, provider);
        assert_eq!(credential.credential_id, "test-credential");
        assert_eq!(credential.verification_status, VerificationStatus::Unverified);
        assert_eq!(credential.attributes.get("key1"), Some(&"value1".to_string()));
        assert_eq!(credential.attributes.get("key2"), Some(&"value2".to_string()));
        assert!(!credential.is_expired());
        assert!(!credential.is_verified());
        
        let content_id = credential.content_id();
        assert_eq!(content_id.content_type, "identity-credential");
    }
    
    #[tokio::test]
    async fn test_local_identity_verifier() -> Result<()> {
        let verifier = LocalIdentityVerifier::new();
        let actor_id = ActorId::new("test-actor");
        
        // Add a trusted credential
        verifier.add_trusted_credential("test-credential", "secret")?;
        
        // Create a credential with the trusted ID
        let mut trusted_credential = IdentityCredential::new(
            actor_id.clone(),
            IdentityProvider::Local,
            "test-credential",
        );
        
        // Create a credential with an untrusted ID
        let mut untrusted_credential = IdentityCredential::new(
            actor_id,
            IdentityProvider::Local,
            "untrusted-credential",
        );
        
        // Verify both credentials
        let trusted_result = verifier.verify(&mut trusted_credential).await?;
        let untrusted_result = verifier.verify(&mut untrusted_credential).await?;
        
        assert!(trusted_result);
        assert!(trusted_credential.is_verified());
        
        assert!(!untrusted_result);
        assert!(!untrusted_credential.is_verified());
        if let VerificationStatus::Failed(reason) = untrusted_credential.verification_status {
            assert_eq!(reason, "Credential not found in trusted store");
        } else {
            panic!("Expected failed verification status");
        }
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_identity_service() -> Result<()> {
        let service = IdentityService::new();
        let actor_id = ActorId::new("test-actor");
        
        // Register a local verifier
        let verifier = Arc::new(LocalIdentityVerifier::new());
        verifier.add_trusted_credential("test-credential", "secret")?;
        service.register_verifier(verifier)?;
        
        // Add a trusted credential
        let trusted_credential = IdentityCredential::new(
            actor_id.clone(),
            IdentityProvider::Local,
            "test-credential",
        );
        service.add_credential(trusted_credential)?;
        
        // Add an untrusted credential
        let untrusted_credential = IdentityCredential::new(
            actor_id.clone(),
            IdentityProvider::Local,
            "untrusted-credential",
        );
        service.add_credential(untrusted_credential)?;
        
        // Get all credentials for the actor
        let credentials = service.get_credentials(&actor_id)?;
        assert_eq!(credentials.len(), 2);
        
        // Verify all credentials
        let result = service.verify_all_credentials(&actor_id).await?;
        assert!(!result); // Not all credentials are valid
        
        // Get the updated credentials
        let updated_credentials = service.get_credentials(&actor_id)?;
        
        // Check the verification status of each credential
        let trusted = updated_credentials.iter()
            .find(|c| c.credential_id == "test-credential")
            .expect("Trusted credential not found");
        
        let untrusted = updated_credentials.iter()
            .find(|c| c.credential_id == "untrusted-credential")
            .expect("Untrusted credential not found");
        
        assert!(trusted.is_verified());
        assert!(!untrusted.is_verified());
        
        Ok(())
    }
} 