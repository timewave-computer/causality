// Repository module for Causality Engine
//
// This module provides interfaces and implementations for code repositories
// that store content-addressed code.

use std::fmt::Debug;
use async_trait::async_trait;
use causality_error::Result;
use causality_types::crypto_primitives::ContentHash;

/// A repository for storing and retrieving content-addressed code
#[async_trait]
pub trait CodeRepository: Debug + Send + Sync {
    /// Get code by its content hash
    async fn get_code(&self, hash: &ContentHash) -> Result<Option<Vec<u8>>>;
    
    /// Store code and get its content hash
    async fn store_code(&self, code: &[u8]) -> Result<ContentHash>;
    
    /// Check if code exists in the repository
    async fn has_code(&self, hash: &ContentHash) -> Result<bool>;
    
    /// Remove code from the repository
    async fn remove_code(&self, hash: &ContentHash) -> Result<bool>;
}

/// A simple in-memory implementation of a code repository for testing
#[cfg(test)]
pub mod memory {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    
    /// An in-memory code repository
    #[derive(Debug)]
    pub struct InMemoryCodeRepository {
        storage: Arc<RwLock<HashMap<ContentHash, Vec<u8>>>>,
    }
    
    impl InMemoryCodeRepository {
        /// Create a new in-memory code repository
        pub fn new() -> Self {
            InMemoryCodeRepository {
                storage: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }
    
    #[async_trait]
    impl CodeRepository for InMemoryCodeRepository {
        async fn get_code(&self, hash: &ContentHash) -> Result<Option<Vec<u8>>> {
            let storage = self.storage.read().map_err(|_| causality_error::Error::LockError)?;
            Ok(storage.get(hash).cloned())
        }
        
        async fn store_code(&self, code: &[u8]) -> Result<ContentHash> {
            use causality_crypto::{HashFactory, ContentAddressed};
            
            // Hash the code
            let hash_factory = HashFactory::default();
            let hasher = hash_factory.create_hasher()?;
            let hash = hasher.hash(code);
            let content_hash = ContentHash::from_bytes(hash.as_bytes())
                .map_err(|e| causality_error::Error::CryptoError(e.to_string()))?;
            
            // Store the code
            let mut storage = self.storage.write().map_err(|_| causality_error::Error::LockError)?;
            storage.insert(content_hash.clone(), code.to_vec());
            
            Ok(content_hash)
        }
        
        async fn has_code(&self, hash: &ContentHash) -> Result<bool> {
            let storage = self.storage.read().map_err(|_| causality_error::Error::LockError)?;
            Ok(storage.contains_key(hash))
        }
        
        async fn remove_code(&self, hash: &ContentHash) -> Result<bool> {
            let mut storage = self.storage.write().map_err(|_| causality_error::Error::LockError)?;
            Ok(storage.remove(hash).is_some())
        }
    }
} 