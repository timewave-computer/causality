// Provider module for Causality
//
// This module provides content-addressed provider implementations for Causality,
// including registries and factories for provider management.

pub mod content_addressed_provider;

// Re-export provider types
pub use content_addressed_provider::{
    Provider,
    ContentAddressedProviderRegistry,
    ProviderMetadata,
    ProviderError,
};

use crate::crypto::ContentAddressed;
use crate::crypto::storage::ContentAddressedStorage;
use std::sync::Arc;

/// Factory for creating provider registries
pub struct ProviderFactory {
    /// Storage for provider data
    storage: Arc<dyn ContentAddressedStorage>,
}

impl ProviderFactory {
    /// Create a new provider factory
    pub fn new(storage: Arc<dyn ContentAddressedStorage>) -> Self {
        Self { storage }
    }
    
    /// Create a content-addressed provider registry
    pub fn create_registry<T: ContentAddressed + 'static>(&self) -> ContentAddressedProviderRegistry<T> {
        ContentAddressedProviderRegistry::new(self.storage.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::storage::InMemoryContentAddressedStorage;
    use crate::provider::content_addressed_provider::Provider;
    use async_trait::async_trait;
    use serde::{Serialize, Deserialize};
    use borsh::{BorshSerialize, BorshDeserialize};
    use crate::crypto::{ContentAddressed, ContentId, HashOutput, HashError, HashFactory};
    
    // Mock content type for testing
    #[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
    struct TestContent {
        id: String,
        data: Vec<u8>,
    }
    
    impl ContentAddressed for TestContent {
        fn content_hash(&self) -> HashOutput {
            let hash_factory = HashFactory::default();
            let hasher = hash_factory.create_hasher().unwrap();
            let data = self.try_to_vec().unwrap();
            hasher.hash(&data)
        }
        
        fn verify(&self) -> bool {
            true
        }
        
        fn to_bytes(&self) -> Vec<u8> {
            self.try_to_vec().unwrap()
        }
        
        fn from_bytes(bytes: &[u8]) -> std::result::Result<Self, HashError> {
            BorshDeserialize::try_from_slice(bytes)
                .map_err(|e| HashError::SerializationError(e.to_string()))
        }
    }
    
    #[tokio::test]
    async fn test_provider_factory() {
        // Create storage
        let storage = Arc::new(InMemoryContentAddressedStorage::new());
        
        // Create factory
        let factory = ProviderFactory::new(storage);
        
        // Create registry
        let registry = factory.create_registry::<TestContent>();
        
        // Verify registry was created successfully
        assert!(registry.get_all_providers().is_ok());
    }
} 