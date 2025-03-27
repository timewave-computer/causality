// SMT-based content-addressed storage
//
// This module provides an implementation of ContentAddressedStorage
// backed by a Sparse Merkle Tree (SMT) for efficient storage with
// cryptographic verification properties.

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use sparse_merkle_tree::default_store::DefaultStore;
use sparse_merkle_tree::H256;

use crate::{
    ContentAddressed, ContentId, HashOutput, HashError,
    content_store::{ContentAddressedStorage, StorageError},
    sparse_merkle_tree::{
        MerkleSmt, SmtKeyValue, SmtError, SmtProof, SmtFactory, ContentAddressedSmt
    }
};

/// A content-addressed storage implementation backed by a Sparse Merkle Tree
pub struct SmtContentStore<S> {
    /// The underlying Sparse Merkle Tree
    smt: Arc<MerkleSmt<S>>,
    /// Cache for content data (optional)
    data_cache: RwLock<HashMap<ContentId, Vec<u8>>>,
    /// Current root hash
    root: RwLock<H256>,
}

impl<S: sparse_merkle_tree::traits::StoreReadOps<SmtKeyValue> + 
        sparse_merkle_tree::traits::StoreWriteOps<SmtKeyValue> + 
        Send + Sync + 'static> 
    SmtContentStore<S> 
{
    /// Create a new SMT-backed content store with the given SMT
    pub fn new(smt: Arc<MerkleSmt<S>>) -> Self {
        Self {
            root: RwLock::new(smt.root()),
            smt,
            data_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Get the current root hash
    pub fn root(&self) -> H256 {
        *self.root.read().unwrap()
    }

    /// Update the root hash
    pub fn update_root(&self, new_root: H256) {
        *self.root.write().unwrap() = new_root;
    }

    /// Get a proof for a content hash
    pub fn get_proof(&self, id: &ContentId) -> Result<SmtProof, StorageError> {
        let hash = id.hash();
        let hash_bytes = hash.as_bytes();
        let mut key = [0u8; 32];
        key.copy_from_slice(hash_bytes);
        let smt_key = H256::from(key);
        
        let (_, proof) = self.smt.get_with_proof(&smt_key)
            .map_err(|e| StorageError::NotFound(format!("Error getting proof: {}", e)))?;
        
        Ok(proof)
    }

    /// Verify an inclusion proof
    pub fn verify_proof(&self, id: &ContentId, proof: &SmtProof) -> bool {
        let hash = id.hash();
        self.smt.verify_inclusion(&self.root(), &hash, proof)
    }
}

impl SmtContentStore<DefaultStore<H256>> {
    /// Create a new SMT content store with default storage
    pub fn new_default() -> Self {
        let factory = DefaultSmtFactory;
        let smt = factory.create_default_smt();
        Self::new(smt)
    }
}

/// Default SMT factory implementation
pub struct DefaultSmtFactory;

impl SmtFactory for DefaultSmtFactory {
    fn create_smt<S: sparse_merkle_tree::traits::StoreReadOps<SmtKeyValue> + 
                    sparse_merkle_tree::traits::StoreWriteOps<SmtKeyValue>>(
        &self, 
        store: S
    ) -> Arc<MerkleSmt<S>> {
        Arc::new(MerkleSmt::new(store))
    }
    
    fn create_default_smt(&self) -> Arc<MerkleSmt<DefaultStore<H256>>> {
        Arc::new(MerkleSmt::new(DefaultStore::default()))
    }
}

impl<S: sparse_merkle_tree::traits::StoreReadOps<SmtKeyValue> + 
        sparse_merkle_tree::traits::StoreWriteOps<SmtKeyValue> + 
        Send + Sync + 'static> 
    ContentAddressedStorage for SmtContentStore<S> 
{
    fn store<T: ContentAddressed>(&self, object: &T) -> Result<ContentId, StorageError> {
        // Get the content hash and ID
        let content_hash = object.content_hash()?;
        let content_id = ContentId::from(content_hash.clone());
        
        // Check if already stored
        if self.contains(&content_id) {
            return Ok(content_id);
        }
        
        // Serialize the object
        let serialized = object.to_bytes()?;
        
        // Store the serialized data in cache
        {
            let mut cache = self.data_cache.write().unwrap();
            cache.insert(content_id.clone(), serialized.clone());
        }
        
        // Store the object in the SMT
        let (_, _, new_root) = self.smt.store_content(object)
            .map_err(|e| StorageError::IoError(format!("SMT store error: {}", e)))?;
        
        // Update root
        self.update_root(new_root);
        
        Ok(content_id)
    }
    
    fn contains(&self, id: &ContentId) -> bool {
        // First check the cache
        {
            let cache = self.data_cache.read().unwrap();
            if cache.contains_key(id) {
                return true;
            }
        }
        
        // Then check the SMT
        let hash = id.hash();
        let hash_bytes = hash.as_bytes();
        let mut key = [0u8; 32];
        key.copy_from_slice(hash_bytes);
        let smt_key = H256::from(key);
        
        match self.smt.contains_key(&smt_key) {
            Ok(exists) => exists,
            Err(_) => false
        }
    }
    
    fn get_bytes(&self, id: &ContentId) -> Result<Vec<u8>, StorageError> {
        // First check the cache
        {
            let cache = self.data_cache.read().unwrap();
            if let Some(data) = cache.get(id) {
                return Ok(data.clone());
            }
        }
        
        // Then try to get from the SMT
        let hash = id.hash();
        
        // Convert object's content hash to SmtKeyValue
        let hash_bytes = hash.as_bytes();
        let mut key = [0u8; 32];
        key.copy_from_slice(hash_bytes);
        let smt_key = H256::from(key);
        
        // Get the value
        let value = self.smt.get(&smt_key)
            .map_err(|e| StorageError::NotFound(format!("Object not found: {}", e)))?;
        
        // Convert value back to bytes
        let bytes = value.as_bytes().to_vec();
        
        // Cache the result
        {
            let mut cache = self.data_cache.write().unwrap();
            cache.insert(id.clone(), bytes.clone());
        }
        
        Ok(bytes)
    }
    
    fn remove(&self, id: &ContentId) -> Result<(), StorageError> {
        // We can't actually remove from the SMT (it's immutable)
        // but we can remove from the cache
        let mut cache = self.data_cache.write().unwrap();
        cache.remove(id);
        
        // Return success even though we can't modify the SMT
        // This is a limitation of the SMT implementation
        Ok(())
    }
    
    fn clear(&self) {
        // Clear the cache
        let mut cache = self.data_cache.write().unwrap();
        cache.clear();
        
        // We can't clear the SMT, but we could create a new one
        // and update the root to empty. This is a limitation.
        self.update_root(H256::zero());
    }
    
    fn len(&self) -> usize {
        // This is just an estimate based on the cache
        let cache = self.data_cache.read().unwrap();
        cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HashFactory;
    use borsh::{BorshSerialize, BorshDeserialize};
    
    #[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
    struct TestObject {
        id: u64,
        name: String,
        data: Vec<u8>,
    }
    
    impl ContentAddressed for TestObject {
        fn content_hash(&self) -> Result<HashOutput, HashError> {
            let hash_factory = HashFactory::default();
            let hasher = hash_factory.create_hasher()?;
            let data = self.try_to_vec()
                .map_err(|e| HashError::SerializationError(e.to_string()))?;
            Ok(hasher.hash(&data))
        }
        
        fn to_bytes(&self) -> Result<Vec<u8>, HashError> {
            self.try_to_vec()
                .map_err(|e| HashError::SerializationError(e.to_string()))
        }
        
        fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
            BorshDeserialize::try_from_slice(bytes)
                .map_err(|e| HashError::SerializationError(e.to_string()))
        }
    }
    
    #[test]
    fn test_smt_content_store() {
        // Create a content store
        let store = SmtContentStore::new_default();
        
        // Create a test object
        let obj = TestObject {
            id: 1,
            name: "Test".to_string(),
            data: vec![1, 2, 3, 4, 5],
        };
        
        // Store the object
        let content_id = store.store(&obj).unwrap();
        
        // Verify contains
        assert!(store.contains(&content_id));
        
        // Retrieve the object
        let retrieved: TestObject = store.get(&content_id).unwrap();
        
        // Verify retrieved object
        assert_eq!(obj, retrieved);
        
        // Get a proof
        let proof = store.get_proof(&content_id).unwrap();
        
        // Verify the proof
        assert!(store.verify_proof(&content_id, &proof));
    }
} 