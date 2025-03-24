// Verification Provider Module
//
// This module defines the provider types and traits for the unified verification framework.

use std::sync::Arc;
use async_trait::async_trait;

use crate::types::{*};
use crate::crypto::hash::ContentId;;
use crate::verification::{
    VerificationError, 
    VerificationResult, 
    UnifiedProof, 
    VerificationContext
};

/// The type of verification a provider can perform
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VerificationType {
    /// ZK proof verification
    ZkProof,
    /// Time-based verification
    Temporal,
    /// Ancestral verification (controller paths)
    Ancestral,
    /// Logical verification (effect validation)
    Logical,
    /// Cross-domain verification
    CrossDomain,
    /// Custom verification type
    Custom(String),
}

/// Provider capability
#[derive(Debug, Clone)]
pub struct ProviderCapability {
    /// Verification type
    pub verification_type: VerificationType,
    /// Provider name
    pub provider_name: String,
    /// Version of the provider
    pub version: String,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// A unified verification provider that can verify proofs
#[async_trait]
pub trait VerificationProvider: Send + Sync {
    /// Get the name of this provider
    fn name(&self) -> &str;
    
    /// Get the verification types supported by this provider
    fn supported_verification_types(&self) -> Vec<VerificationType>;
    
    /// Get the capabilities of this provider
    fn capabilities(&self) -> Vec<ProviderCapability>;
    
    /// Check if the provider can verify a specific proof
    fn can_verify(&self, proof: &UnifiedProof) -> bool;
    
    /// Verify a proof
    async fn verify(
        &self,
        proof: &UnifiedProof,
        context: &VerificationContext,
    ) -> Result<VerificationResult, VerificationError>;
}

/// A provider for ZK proof verification
#[async_trait]
pub trait ZkVerificationProvider: VerificationProvider {
    /// Verify a ZK proof
    async fn verify_zk(
        &self,
        proof: &UnifiedProof,
        context: &VerificationContext,
    ) -> Result<VerificationResult, VerificationError>;
}

/// A provider for temporal verification
#[async_trait]
pub trait TemporalVerificationProvider: VerificationProvider {
    /// Verify a temporal proof
    async fn verify_temporal(
        &self,
        proof: &UnifiedProof,
        context: &VerificationContext,
    ) -> Result<VerificationResult, VerificationError>;
}

/// A provider for ancestral verification
#[async_trait]
pub trait AncestralVerificationProvider: VerificationProvider {
    /// Verify an ancestral proof
    async fn verify_ancestral(
        &self,
        proof: &UnifiedProof,
        context: &VerificationContext,
    ) -> Result<VerificationResult, VerificationError>;
}

/// A provider for logical verification
#[async_trait]
pub trait LogicalVerificationProvider: VerificationProvider {
    /// Verify a logical proof
    async fn verify_logical(
        &self,
        proof: &UnifiedProof,
        context: &VerificationContext,
    ) -> Result<VerificationResult, VerificationError>;
}

/// A provider for cross-domain verification
#[async_trait]
pub trait CrossDomainVerificationProvider: VerificationProvider {
    /// Verify a cross-domain proof
    async fn verify_cross_domain(
        &self,
        proof: &UnifiedProof,
        context: &VerificationContext,
    ) -> Result<VerificationResult, VerificationError>;
}

/// A registry of verification providers
#[derive(Default)]
pub struct VerificationProviderRegistry {
    /// The registered providers
    providers: std::collections::HashMap<String, Arc<dyn VerificationProvider>>,
}

impl VerificationProviderRegistry {
    /// Create a new verification provider registry
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
        }
    }
    
    /// Register a provider
    pub fn register_provider(&mut self, provider: Arc<dyn VerificationProvider>) {
        self.providers.insert(provider.name().to_string(), provider);
    }
    
    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn VerificationProvider>> {
        self.providers.get(name).cloned()
    }
    
    /// Get all providers
    pub fn all_providers(&self) -> Vec<Arc<dyn VerificationProvider>> {
        self.providers.values().cloned().collect()
    }
    
    /// Get providers that support a specific verification type
    pub fn providers_by_type(&self, verification_type: &VerificationType) -> Vec<Arc<dyn VerificationProvider>> {
        self.providers
            .values()
            .filter(|p| p.supported_verification_types().contains(verification_type))
            .cloned()
            .collect()
    }
    
    /// Find providers capable of verifying a specific proof
    pub fn find_capable_providers(&self, proof: &UnifiedProof) -> Vec<Arc<dyn VerificationProvider>> {
        self.providers
            .values()
            .filter(|p| p.can_verify(proof))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    struct MockProvider {
        name: String,
        supported_types: Vec<VerificationType>,
    }
    
    #[async_trait]
    impl VerificationProvider for MockProvider {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn supported_verification_types(&self) -> Vec<VerificationType> {
            self.supported_types.clone()
        }
        
        fn capabilities(&self) -> Vec<ProviderCapability> {
            self.supported_types
                .iter()
                .map(|vt| ProviderCapability {
                    verification_type: vt.clone(),
                    provider_name: self.name.clone(),
                    version: "1.0".to_string(),
                    metadata: HashMap::new(),
                })
                .collect()
        }
        
        fn can_verify(&self, proof: &UnifiedProof) -> bool {
            // Mock implementation - can verify if ZK and we support ZK
            (proof.zk_components.is_some() && 
             self.supported_types.contains(&VerificationType::ZkProof)) ||
            (proof.temporal_components.is_some() && 
             self.supported_types.contains(&VerificationType::Temporal))
        }
        
        async fn verify(
            &self,
            _proof: &UnifiedProof,
            _context: &VerificationContext,
        ) -> Result<VerificationResult, VerificationError> {
            // Mock implementation that always succeeds
            Ok(VerificationResult {
                success: true,
                verification_time: chrono::Utc::now(),
                error_message: None,
                confidence: 1.0,
                provider_name: self.name.clone(),
                verification_type: self.supported_types[0].clone(),
                metadata: HashMap::new(),
            })
        }
    }
    
    #[tokio::test]
    async fn test_verification_provider_registry() {
        let mut registry = VerificationProviderRegistry::new();
        
        let zk_provider = Arc::new(MockProvider {
            name: "zk_provider".to_string(),
            supported_types: vec![VerificationType::ZkProof],
        });
        
        let temporal_provider = Arc::new(MockProvider {
            name: "temporal_provider".to_string(),
            supported_types: vec![VerificationType::Temporal],
        });
        
        let multi_provider = Arc::new(MockProvider {
            name: "multi_provider".to_string(),
            supported_types: vec![VerificationType::ZkProof, VerificationType::Temporal],
        });
        
        registry.register_provider(zk_provider);
        registry.register_provider(temporal_provider);
        registry.register_provider(multi_provider);
        
        // Test retrieving by name
        let provider = registry.get_provider("zk_provider").unwrap();
        assert_eq!(provider.name(), "zk_provider");
        
        // Test retrieving by type
        let zk_providers = registry.providers_by_type(&VerificationType::ZkProof);
        assert_eq!(zk_providers.len(), 2);
        
        let temporal_providers = registry.providers_by_type(&VerificationType::Temporal);
        assert_eq!(temporal_providers.len(), 2);
        
        // Test getting all providers
        let all_providers = registry.all_providers();
        assert_eq!(all_providers.len(), 3);
    }
} 
