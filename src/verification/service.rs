// Verification Service Module
//
// This module defines the verification service for the unified verification framework.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::Utc;
use thiserror::Error;

use crate::types::{*};
use crate::crypto::hash::ContentId;;
use crate::verification::{
    UnifiedProof,
    VerificationContext,
    VerificationError,
    VerificationResult,
    VerificationStatus,
    VerificationType,
    Verifiable,
    VerificationProvider,
    VerificationProviderRegistry,
};

/// Verification service error
#[derive(Debug, Error)]
pub enum VerificationServiceError {
    /// No providers available
    #[error("No providers available for verification type: {0:?}")]
    NoProvidersAvailable(VerificationType),
    
    /// Verification failed
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    
    /// Provider error
    #[error("Provider error: {0}")]
    ProviderError(#[from] VerificationError),
    
    /// Invalid proof
    #[error("Invalid proof: {0}")]
    InvalidProof(String),
    
    /// Missing context
    #[error("Missing verification context")]
    MissingContext,
    
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Verification result with additional metadata
#[derive(Debug, Clone)]
pub struct DetailedVerificationResult {
    /// Overall verification status
    pub status: VerificationStatus,
    
    /// Individual results from different providers
    pub provider_results: Vec<VerificationResult>,
    
    /// Combined confidence score (average of all successful verifications)
    pub combined_confidence: f64,
    
    /// Timestamp of verification
    pub verification_time: chrono::DateTime<Utc>,
    
    /// Metadata about the verification
    pub metadata: HashMap<String, String>,
}

impl DetailedVerificationResult {
    /// Create a new detailed verification result
    pub fn new(status: VerificationStatus) -> Self {
        Self {
            status,
            provider_results: Vec::new(),
            combined_confidence: 0.0,
            verification_time: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a provider result
    pub fn add_provider_result(&mut self, result: VerificationResult) {
        self.provider_results.push(result);
        self.update_combined_confidence();
    }
    
    /// Update the combined confidence score
    fn update_combined_confidence(&mut self) {
        let successful_results: Vec<&VerificationResult> = self.provider_results
            .iter()
            .filter(|r| r.success)
            .collect();
        
        if successful_results.is_empty() {
            self.combined_confidence = 0.0;
        } else {
            let sum: f64 = successful_results.iter().map(|r| r.confidence).sum();
            self.combined_confidence = sum / successful_results.len() as f64;
        }
    }
    
    /// Check if all verifications were successful
    pub fn all_successful(&self) -> bool {
        !self.provider_results.is_empty() && 
        self.provider_results.iter().all(|r| r.success)
    }
    
    /// Check if any verification was successful
    pub fn any_successful(&self) -> bool {
        self.provider_results.iter().any(|r| r.success)
    }
    
    /// Get error messages from failed verifications
    pub fn error_messages(&self) -> Vec<String> {
        self.provider_results
            .iter()
            .filter_map(|r| r.error_message.clone())
            .collect()
    }
}

/// Cache entry for verification results
#[derive(Debug, Clone)]
struct VerificationCacheEntry {
    /// The proof that was verified
    proof_id: String,
    
    /// Verification result
    result: DetailedVerificationResult,
    
    /// Timestamp when this entry was cached
    cached_at: chrono::DateTime<Utc>,
    
    /// Expiration time (if any)
    expires_at: Option<chrono::DateTime<Utc>>,
}

/// Type of verifications to perform
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationMode {
    /// Verify using all available providers
    All,
    
    /// Verify using specified providers
    Specific(Vec<String>),
    
    /// Verify using a minimum number of providers
    Minimum(usize),
    
    /// Verify using providers of specific types
    Types(Vec<VerificationType>),
}

impl Default for VerificationMode {
    fn default() -> Self {
        Self::All
    }
}

/// Options for verification
#[derive(Debug, Clone)]
pub struct VerificationOptions {
    /// Mode of verification
    pub mode: VerificationMode,
    
    /// Minimum confidence required for success
    pub min_confidence: f64,
    
    /// Whether to use cached results if available
    pub use_cache: bool,
    
    /// Cache duration (if None, uses service default)
    pub cache_duration: Option<std::time::Duration>,
    
    /// Whether to require all verifications to succeed
    pub require_all: bool,
}

impl Default for VerificationOptions {
    fn default() -> Self {
        Self {
            mode: VerificationMode::default(),
            min_confidence: 0.7,
            use_cache: true,
            cache_duration: None,
            require_all: false,
        }
    }
}

/// Configuration for the verification service
#[derive(Debug, Clone)]
pub struct VerificationServiceConfig {
    /// Default cache duration
    pub default_cache_duration: std::time::Duration,
    
    /// Maximum cache size
    pub max_cache_size: usize,
    
    /// Whether to cache results by default
    pub cache_by_default: bool,
    
    /// Default minimum confidence
    pub default_min_confidence: f64,
}

impl Default for VerificationServiceConfig {
    fn default() -> Self {
        Self {
            default_cache_duration: std::time::Duration::from_secs(3600), // 1 hour
            max_cache_size: 1000,
            cache_by_default: true,
            default_min_confidence: 0.7,
        }
    }
}

/// Verification service
pub struct VerificationService {
    /// Registry of verification providers
    provider_registry: Arc<RwLock<VerificationProviderRegistry>>,
    
    /// Cache of verification results
    cache: RwLock<HashMap<String, VerificationCacheEntry>>,
    
    /// Default verification context
    default_context: RwLock<Option<VerificationContext>>,
    
    /// Service configuration
    config: VerificationServiceConfig,
}

impl VerificationService {
    /// Create a new verification service
    pub fn new(config: VerificationServiceConfig) -> Self {
        Self {
            provider_registry: Arc::new(RwLock::new(VerificationProviderRegistry::new())),
            cache: RwLock::new(HashMap::new()),
            default_context: RwLock::new(None),
            config,
        }
    }
    
    /// Create a new verification service with default configuration
    pub fn default() -> Self {
        Self::new(VerificationServiceConfig::default())
    }
    
    /// Register a verification provider
    pub fn register_provider(&self, provider: Arc<dyn VerificationProvider>) {
        let mut registry = self.provider_registry.write().unwrap();
        registry.register_provider(provider);
    }
    
    /// Set the default verification context
    pub fn set_default_context(&self, context: VerificationContext) {
        let mut default_context = self.default_context.write().unwrap();
        *default_context = Some(context);
    }
    
    /// Get a reference to the provider registry
    pub fn provider_registry(&self) -> Arc<RwLock<VerificationProviderRegistry>> {
        self.provider_registry.clone()
    }
    
    /// Clear the verification cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }
    
    /// Clean expired cache entries
    pub fn clean_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        let now = Utc::now();
        
        cache.retain(|_, entry| {
            entry.expires_at.map_or(true, |expires| expires > now)
        });
        
        // If cache is still too large, remove oldest entries
        if cache.len() > self.config.max_cache_size {
            let mut entries: Vec<_> = cache.iter().collect();
            entries.sort_by(|(_, a), (_, b)| a.cached_at.cmp(&b.cached_at));
            
            let to_remove = entries.len() - self.config.max_cache_size;
            for (key, _) in entries.iter().take(to_remove) {
                cache.remove(*key);
            }
        }
    }
    
    /// Get cached verification result if available
    fn get_cached_result(&self, proof_id: &str) -> Option<DetailedVerificationResult> {
        let cache = self.cache.read().unwrap();
        let now = Utc::now();
        
        cache.get(proof_id).and_then(|entry| {
            if entry.expires_at.map_or(true, |expires| expires > now) {
                Some(entry.result.clone())
            } else {
                None
            }
        })
    }
    
    /// Cache a verification result
    fn cache_result(
        &self, 
        proof_id: &str, 
        result: DetailedVerificationResult,
        cache_duration: Option<std::time::Duration>,
    ) {
        if !self.config.cache_by_default && cache_duration.is_none() {
            return;
        }
        
        let mut cache = self.cache.write().unwrap();
        let now = Utc::now();
        let expires_at = cache_duration
            .or(Some(self.config.default_cache_duration))
            .map(|duration| now + chrono::Duration::from_std(duration).unwrap());
        
        // Clean cache if it's getting too large
        if cache.len() >= self.config.max_cache_size {
            self.clean_cache();
        }
        
        cache.insert(
            proof_id.to_string(),
            VerificationCacheEntry {
                proof_id: proof_id.to_string(),
                result,
                cached_at: now,
                expires_at,
            },
        );
    }
    
    /// Select providers for verification based on the verification mode
    fn select_providers(
        &self,
        proof: &UnifiedProof,
        mode: &VerificationMode,
    ) -> Result<Vec<Arc<dyn VerificationProvider>>, VerificationServiceError> {
        let registry = self.provider_registry.read().unwrap();
        
        match mode {
            VerificationMode::All => {
                let providers = registry.find_capable_providers(proof);
                if providers.is_empty() {
                    return Err(VerificationServiceError::NoProvidersAvailable(
                        VerificationType::Custom("any".to_string())
                    ));
                }
                Ok(providers)
            },
            
            VerificationMode::Specific(names) => {
                let mut providers = Vec::new();
                for name in names {
                    if let Some(provider) = registry.get_provider(name) {
                        if provider.can_verify(proof) {
                            providers.push(provider);
                        }
                    }
                }
                
                if providers.is_empty() {
                    return Err(VerificationServiceError::NoProvidersAvailable(
                        VerificationType::Custom("specified providers".to_string())
                    ));
                }
                Ok(providers)
            },
            
            VerificationMode::Minimum(min_count) => {
                let providers = registry.find_capable_providers(proof);
                if providers.len() < *min_count {
                    return Err(VerificationServiceError::NoProvidersAvailable(
                        VerificationType::Custom(format!("minimum {} providers", min_count))
                    ));
                }
                Ok(providers)
            },
            
            VerificationMode::Types(types) => {
                let mut providers = Vec::new();
                for vtype in types {
                    let type_providers = registry.providers_by_type(vtype);
                    for provider in type_providers {
                        if provider.can_verify(proof) {
                            providers.push(provider);
                        }
                    }
                }
                
                if providers.is_empty() {
                    return Err(VerificationServiceError::NoProvidersAvailable(
                        VerificationType::Custom("specified types".to_string())
                    ));
                }
                Ok(providers)
            }
        }
    }
    
    /// Verify a proof using the specified options and context
    pub async fn verify_proof(
        &self,
        proof: &UnifiedProof,
        options: &VerificationOptions,
        context: Option<&VerificationContext>,
    ) -> Result<DetailedVerificationResult, VerificationServiceError> {
        // Check cache first if enabled
        if options.use_cache {
            if let Some(cached_result) = self.get_cached_result(&proof.id) {
                return Ok(cached_result);
            }
        }
        
        // Get verification context
        let context = match context {
            Some(ctx) => ctx,
            None => {
                let default_context = self.default_context.read().unwrap();
                match &*default_context {
                    Some(ctx) => ctx,
                    None => return Err(VerificationServiceError::MissingContext),
                }
            }
        };
        
        // Select providers based on verification mode
        let providers = self.select_providers(proof, &options.mode)?;
        
        // Verify using selected providers
        let mut detailed_result = DetailedVerificationResult::new(VerificationStatus::InProgress);
        
        for provider in providers {
            match provider.verify(proof, context).await {
                Ok(result) => {
                    detailed_result.add_provider_result(result);
                },
                Err(err) => {
                    // Add failed result
                    detailed_result.add_provider_result(VerificationResult {
                        success: false,
                        verification_time: Utc::now(),
                        error_message: Some(err.to_string()),
                        confidence: 0.0,
                        provider_name: provider.name().to_string(),
                        verification_type: provider.supported_verification_types().first()
                            .unwrap_or(&VerificationType::Custom("unknown".to_string()))
                            .clone(),
                        metadata: HashMap::new(),
                    });
                }
            }
        }
        
        // Determine overall verification status
        let status = if detailed_result.provider_results.is_empty() {
            VerificationStatus::NotVerified
        } else if options.require_all && detailed_result.all_successful() {
            VerificationStatus::Verified
        } else if !options.require_all && detailed_result.any_successful() {
            VerificationStatus::Verified
        } else {
            VerificationStatus::Failed
        };
        
        detailed_result.status = status;
        
        // Cache the result if needed
        if options.use_cache {
            self.cache_result(&proof.id, detailed_result.clone(), options.cache_duration);
        }
        
        Ok(detailed_result)
    }
    
    /// Verify an entity that implements the Verifiable trait
    pub async fn verify<T: Verifiable>(
        &self,
        verifiable: &T,
        options: &VerificationOptions,
        context: Option<&VerificationContext>,
    ) -> Result<DetailedVerificationResult, VerificationServiceError> {
        // Generate proof from verifiable
        let context_ref = match context {
            Some(ctx) => ctx,
            None => {
                let default_context = self.default_context.read().unwrap();
                match &*default_context {
                    Some(ctx) => ctx,
                    None => return Err(VerificationServiceError::MissingContext),
                }
            }
        };
        
        let proof = verifiable.generate_proof(context_ref)
            .map_err(|e| VerificationServiceError::VerificationFailed(e.to_string()))?;
        
        self.verify_proof(&proof, options, context).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::verification::proof::ZkProofData;
    
    // Mock implementation of VerificationProvider for testing
    struct MockProvider {
        name: String,
        should_succeed: bool,
    }
    
    #[async_trait]
    impl VerificationProvider for MockProvider {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn supported_verification_types(&self) -> Vec<VerificationType> {
            vec![VerificationType::ZkProof]
        }
        
        fn capabilities(&self) -> Vec<crate::verification::ProviderCapability> {
            vec![]
        }
        
        fn can_verify(&self, proof: &UnifiedProof) -> bool {
            proof.zk_components.is_some()
        }
        
        async fn verify(
            &self,
            _proof: &UnifiedProof,
            _context: &VerificationContext,
        ) -> Result<VerificationResult, VerificationError> {
            if self.should_succeed {
                Ok(VerificationResult {
                    success: true,
                    verification_time: Utc::now(),
                    error_message: None,
                    confidence: 1.0,
                    provider_name: self.name.clone(),
                    verification_type: VerificationType::ZkProof,
                    metadata: HashMap::new(),
                })
            } else {
                Err(VerificationError::InvalidProof("Test failure".to_string()))
            }
        }
    }
    
    #[tokio::test]
    async fn test_verification_service() {
        // Create service
        let service = VerificationService::default();
        
        // Register providers
        service.register_provider(Arc::new(MockProvider {
            name: "success_provider".to_string(),
            should_succeed: true,
        }));
        
        service.register_provider(Arc::new(MockProvider {
            name: "failure_provider".to_string(),
            should_succeed: false,
        }));
        
        // Create a simple context
        let context = VerificationContext::new();
        service.set_default_context(context);
        
        // Create a proof
        let proof = UnifiedProof::new("test-proof".to_string())
            .with_zk_components(ZkProofData {
                system: "test".to_string(),
                proof: vec![1, 2, 3],
                public_inputs: vec![],
                verification_key_id: "test-key".to_string(),
                created_at: Utc::now(),
                metadata: HashMap::new(),
            });
        
        // Test verification with all providers
        let options = VerificationOptions {
            mode: VerificationMode::All,
            require_all: false,
            ..Default::default()
        };
        
        let result = service.verify_proof(&proof, &options, None).await.unwrap();
        assert_eq!(result.status, VerificationStatus::Verified);
        assert_eq!(result.provider_results.len(), 2);
        assert!(result.any_successful());
        assert!(!result.all_successful());
        
        // Test verification requiring all to succeed
        let strict_options = VerificationOptions {
            mode: VerificationMode::All,
            require_all: true,
            ..Default::default()
        };
        
        let strict_result = service.verify_proof(&proof, &strict_options, None).await.unwrap();
        assert_eq!(strict_result.status, VerificationStatus::Failed);
        
        // Test verification with specific provider
        let specific_options = VerificationOptions {
            mode: VerificationMode::Specific(vec!["success_provider".to_string()]),
            ..Default::default()
        };
        
        let specific_result = service.verify_proof(&proof, &specific_options, None).await.unwrap();
        assert_eq!(specific_result.status, VerificationStatus::Verified);
        assert_eq!(specific_result.provider_results.len(), 1);
        
        // Test caching
        let cached_result = service.get_cached_result(&proof.id).unwrap();
        assert_eq!(cached_result.status, VerificationStatus::Verified);
    }
} 
