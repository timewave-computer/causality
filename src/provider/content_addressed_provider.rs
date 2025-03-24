// Content-addressed provider registry
//
// This module implements a content-addressed version of the provider registry,
// allowing providers to be registered and retrieved using their content hashes.

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use async_trait::async_trait;
use borsh::{BorshSerialize, BorshDeserialize};
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::crypto::{ContentAddressed, ContentId, HashOutput, HashError, HashFactory};
use crate::crypto::storage::ContentAddressedStorage;
use crate::error::{Error, Result};

/// Error type for content-addressed provider operations
#[derive(Error, Debug)]
pub enum ProviderError {
    /// Provider not found
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    
    /// Verification failed
    #[error("Provider verification failed: {0}")]
    VerificationFailed(String),
    
    /// Internal error
    #[error("Internal provider error: {0}")]
    InternalError(String),
    
    /// Hash error
    #[error("Hash error: {0}")]
    HashError(#[from] HashError),
    
    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(#[from] Error),
}

/// Provider interface for content-addressed systems
pub trait Provider<T: ContentAddressed>: Send + Sync {
    /// Get the provider's ID
    fn id(&self) -> &str;
    
    /// Get the provider's name
    fn name(&self) -> &str;
    
    /// Get the provider's type
    fn provider_type(&self) -> &str;
    
    /// Check if the provider can handle the given content ID
    fn can_handle(&self, content_id: &ContentId) -> bool;
    
    /// Get content by its ID
    async fn get_content(&self, content_id: &ContentId) -> std::result::Result<T, ProviderError>;
    
    /// Store content and return its ID
    async fn store_content(&self, content: &T) -> std::result::Result<ContentId, ProviderError>;
    
    /// Verify content against its hash
    async fn verify_content(&self, content: &T) -> std::result::Result<bool, ProviderError>;
}

/// A content-addressed provider registry
pub struct ContentAddressedProviderRegistry<T: ContentAddressed> {
    /// Providers by their ID
    providers: RwLock<HashMap<String, Arc<dyn Provider<T>>>>,
    /// Providers by content type
    providers_by_type: RwLock<HashMap<String, Vec<String>>>,
    /// Storage for provider metadata
    storage: Arc<dyn ContentAddressedStorage>,
}

/// Provider metadata for content-addressed storage
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ProviderMetadata {
    /// Provider ID
    pub id: String,
    /// Provider name
    pub name: String,
    /// Provider type
    pub provider_type: String,
    /// Provider capabilities
    pub capabilities: Vec<String>,
    /// Provider content hash
    pub content_hash: Option<HashOutput>,
}

impl ContentAddressed for ProviderMetadata {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        hasher.hash(&serialized) == hash
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
    
    fn from_bytes(bytes: &[u8]) -> std::result::Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl<T: ContentAddressed + 'static> ContentAddressedProviderRegistry<T> {
    /// Create a new content-addressed provider registry
    pub fn new(storage: Arc<dyn ContentAddressedStorage>) -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
            providers_by_type: RwLock::new(HashMap::new()),
            storage,
        }
    }
    
    /// Register a provider for a content type
    pub fn register_provider(&self, provider: Arc<dyn Provider<T>>) -> Result<ContentId, ProviderError> {
        // Create metadata for the provider
        let metadata = ProviderMetadata {
            id: provider.id().to_string(),
            name: provider.name().to_string(),
            provider_type: provider.provider_type().to_string(),
            capabilities: Vec::new(), // Could be expanded based on provider capabilities
            content_hash: None,
        };
        
        // Store metadata in content-addressed storage
        let content_id = self.storage.store(&metadata)
            .map_err(|e| ProviderError::StorageError(e))?;
        
        // Update the registries
        {
            let mut providers = self.providers.write().map_err(|_| {
                ProviderError::InternalError("Failed to acquire lock on providers".to_string())
            })?;
            
            providers.insert(provider.id().to_string(), provider.clone());
        }
        
        {
            let mut providers_by_type = self.providers_by_type.write().map_err(|_| {
                ProviderError::InternalError("Failed to acquire lock on providers by type".to_string())
            })?;
            
            let provider_ids = providers_by_type
                .entry(provider.provider_type().to_string())
                .or_insert_with(Vec::new);
            
            provider_ids.push(provider.id().to_string());
        }
        
        Ok(content_id)
    }
    
    /// Get a provider by ID
    pub fn get_provider(&self, id: &str) -> Result<Arc<dyn Provider<T>>, ProviderError> {
        let providers = self.providers.read().map_err(|_| {
            ProviderError::InternalError("Failed to acquire lock on providers".to_string())
        })?;
        
        providers.get(id).cloned().ok_or_else(|| {
            ProviderError::ProviderNotFound(id.to_string())
        })
    }
    
    /// Get all providers of a specific type
    pub fn get_providers_by_type(&self, provider_type: &str) -> Result<Vec<Arc<dyn Provider<T>>>, ProviderError> {
        let providers_by_type = self.providers_by_type.read().map_err(|_| {
            ProviderError::InternalError("Failed to acquire lock on providers by type".to_string())
        })?;
        
        let providers = self.providers.read().map_err(|_| {
            ProviderError::InternalError("Failed to acquire lock on providers".to_string())
        })?;
        
        let provider_ids = providers_by_type.get(provider_type).ok_or_else(|| {
            ProviderError::ProviderNotFound(format!("No providers of type '{}'", provider_type))
        })?;
        
        let mut result = Vec::new();
        for id in provider_ids {
            if let Some(provider) = providers.get(id) {
                result.push(provider.clone());
            }
        }
        
        Ok(result)
    }
    
    /// Get all providers
    pub fn get_all_providers(&self) -> Result<Vec<Arc<dyn Provider<T>>>, ProviderError> {
        let providers = self.providers.read().map_err(|_| {
            ProviderError::InternalError("Failed to acquire lock on providers".to_string())
        })?;
        
        Ok(providers.values().cloned().collect())
    }
    
    /// Find providers that can handle a content ID
    pub fn find_providers_for_content(&self, content_id: &ContentId) -> Result<Vec<Arc<dyn Provider<T>>>, ProviderError> {
        let providers = self.providers.read().map_err(|_| {
            ProviderError::InternalError("Failed to acquire lock on providers".to_string())
        })?;
        
        let mut capable_providers = Vec::new();
        for provider in providers.values() {
            if provider.can_handle(content_id) {
                capable_providers.push(provider.clone());
            }
        }
        
        Ok(capable_providers)
    }
    
    /// Get provider metadata by ID
    pub fn get_provider_metadata(&self, id: &str) -> Result<ProviderMetadata, ProviderError> {
        let providers = self.providers.read().map_err(|_| {
            ProviderError::InternalError("Failed to acquire lock on providers".to_string())
        })?;
        
        let provider = providers.get(id).ok_or_else(|| {
            ProviderError::ProviderNotFound(id.to_string())
        })?;
        
        // Create metadata for the provider
        let metadata = ProviderMetadata {
            id: provider.id().to_string(),
            name: provider.name().to_string(),
            provider_type: provider.provider_type().to_string(),
            capabilities: Vec::new(), // Could be expanded based on provider capabilities
            content_hash: None,
        };
        
        Ok(metadata)
    }
    
    /// Remove a provider by ID
    pub fn remove_provider(&self, id: &str) -> Result<(), ProviderError> {
        // Get provider type before removing
        let provider_type = {
            let providers = self.providers.read().map_err(|_| {
                ProviderError::InternalError("Failed to acquire lock on providers".to_string())
            })?;
            
            let provider = providers.get(id).ok_or_else(|| {
                ProviderError::ProviderNotFound(id.to_string())
            })?;
            
            provider.provider_type().to_string()
        };
        
        // Remove from providers
        {
            let mut providers = self.providers.write().map_err(|_| {
                ProviderError::InternalError("Failed to acquire lock on providers".to_string())
            })?;
            
            providers.remove(id);
        }
        
        // Remove from providers by type
        {
            let mut providers_by_type = self.providers_by_type.write().map_err(|_| {
                ProviderError::InternalError("Failed to acquire lock on providers by type".to_string())
            })?;
            
            if let Some(provider_ids) = providers_by_type.get_mut(&provider_type) {
                provider_ids.retain(|provider_id| provider_id != id);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::storage::InMemoryContentAddressedStorage;
    
    // Mock provider for testing
    struct MockProvider {
        id: String,
        name: String,
        provider_type: String,
        handles_content_ids: Vec<ContentId>,
    }
    
    #[async_trait]
    impl<T: ContentAddressed + Clone + Send + Sync> Provider<T> for MockProvider {
        fn id(&self) -> &str {
            &self.id
        }
        
        fn name(&self) -> &str {
            &self.name
        }
        
        fn provider_type(&self) -> &str {
            &self.provider_type
        }
        
        fn can_handle(&self, content_id: &ContentId) -> bool {
            self.handles_content_ids.contains(content_id)
        }
        
        async fn get_content(&self, _content_id: &ContentId) -> std::result::Result<T, ProviderError> {
            Err(ProviderError::InternalError("Not implemented".to_string()))
        }
        
        async fn store_content(&self, content: &T) -> std::result::Result<ContentId, ProviderError> {
            Ok(content.content_id())
        }
        
        async fn verify_content(&self, content: &T) -> std::result::Result<bool, ProviderError> {
            Ok(content.verify())
        }
    }
    
    // Mock content type for testing
    #[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
    struct MockContent {
        id: String,
        data: Vec<u8>,
    }
    
    impl ContentAddressed for MockContent {
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
    async fn test_provider_registry() {
        // Create storage
        let storage = Arc::new(InMemoryContentAddressedStorage::new());
        
        // Create registry
        let registry = ContentAddressedProviderRegistry::<MockContent>::new(storage);
        
        // Create mock content
        let content1 = MockContent {
            id: "content1".to_string(),
            data: vec![1, 2, 3],
        };
        
        let content2 = MockContent {
            id: "content2".to_string(),
            data: vec![4, 5, 6],
        };
        
        let content_id1 = content1.content_id();
        let content_id2 = content2.content_id();
        
        // Create mock providers
        let provider1 = Arc::new(MockProvider {
            id: "provider1".to_string(),
            name: "Provider 1".to_string(),
            provider_type: "mock".to_string(),
            handles_content_ids: vec![content_id1.clone()],
        });
        
        let provider2 = Arc::new(MockProvider {
            id: "provider2".to_string(),
            name: "Provider 2".to_string(),
            provider_type: "mock".to_string(),
            handles_content_ids: vec![content_id2.clone()],
        });
        
        // Register providers
        let provider_id1 = registry.register_provider(provider1.clone()).unwrap();
        let provider_id2 = registry.register_provider(provider2.clone()).unwrap();
        
        // Test getting providers
        let retrieved_provider1 = registry.get_provider("provider1").unwrap();
        assert_eq!(retrieved_provider1.id(), "provider1");
        
        let retrieved_provider2 = registry.get_provider("provider2").unwrap();
        assert_eq!(retrieved_provider2.id(), "provider2");
        
        // Test getting providers by type
        let mock_providers = registry.get_providers_by_type("mock").unwrap();
        assert_eq!(mock_providers.len(), 2);
        
        // Test finding providers for content
        let providers_for_content1 = registry.find_providers_for_content(&content_id1).unwrap();
        assert_eq!(providers_for_content1.len(), 1);
        assert_eq!(providers_for_content1[0].id(), "provider1");
        
        // Test getting all providers
        let all_providers = registry.get_all_providers().unwrap();
        assert_eq!(all_providers.len(), 2);
        
        // Test removing provider
        registry.remove_provider("provider1").unwrap();
        
        let all_providers_after_remove = registry.get_all_providers().unwrap();
        assert_eq!(all_providers_after_remove.len(), 1);
        assert_eq!(all_providers_after_remove[0].id(), "provider2");
    }
} 