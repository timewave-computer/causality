// Content-addressed domain interface
// Original file: src/domain/content_addressed_interface.rs

// Content-addressed cross-domain interface
//
// This module implements a content-addressed interface for cross-domain operations,
// allowing objects to be resolved, verified, and queried across different domains
// using their content hashes.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use borsh::{BorshSerialize, BorshDeserialize};
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::crypto::{
    ContentAddressed, ContentId, HashOutput, HashError, HashFactory,
    ContentAddressedStorage, StorageError
};
use crate::domain::{DomainId, DomainAdapter};
use causality_types::{Error, Result};

/// Error type for cross-domain content operations
#[derive(Error, Debug)]
pub enum CrossDomainError {
    /// Object not found in the domain
    #[error("Object not found in domain {0}: {1}")]
    ObjectNotFound(DomainId, String),
    
    /// Domain not found
    #[error("Domain not found: {0}")]
    DomainNotFound(DomainId),
    
    /// Verification failed
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    
    /// Query failed
    #[error("Query failed: {0}")]
    QueryFailed(String),
    
    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
    
    /// Hash error
    #[error("Hash error: {0}")]
    HashError(#[from] HashError),
    
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Commitment proof from another domain
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct CommitmentProof {
    /// Domain ID where the proof was generated
    pub domain_id: DomainId,
    
    /// Root hash of the domain's commitment tree
    pub root_hash: HashOutput,
    
    /// Proof path from the object to the root
    pub path: Vec<HashOutput>,
    
    /// Leaf value (hash of the object)
    pub leaf_hash: HashOutput,
    
    /// Timestamp when the proof was generated
    pub timestamp: u64,
    
    /// Additional verification data
    pub verification_data: HashMap<String, Vec<u8>>,
}

impl CommitmentProof {
    /// Create a new commitment proof
    pub fn new(
        domain_id: DomainId, 
        root_hash: HashOutput, 
        path: Vec<HashOutput>, 
        leaf_hash: HashOutput
    ) -> Self {
        Self {
            domain_id,
            root_hash,
            path,
            leaf_hash,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            verification_data: HashMap::new(),
        }
    }
    
    /// Add verification data
    pub fn add_verification_data(&mut self, key: impl Into<String>, value: Vec<u8>) {
        self.verification_data.insert(key.into(), value);
    }
    
    /// Verify the proof against a root hash
    pub fn verify(&self, expected_root: &HashOutput) -> bool {
        // In a real implementation, this would verify the Merkle path
        // For now, this is a simplified placeholder implementation
        &self.root_hash == expected_root
    }
}

/// Cross-domain interface for content-addressed objects
#[async_trait]
pub trait ContentAddressedDomainInterface: Send + Sync {
    /// Resolve a content address from another domain
    async fn resolve_content(&self, domain_id: &DomainId, id: &ContentId) -> std::result::Result<Vec<u8>, CrossDomainError>;
    
    /// Verify a content address with proof
    async fn verify_content<T: ContentAddressed + Send + Sync>(
        &self, 
        domain_id: &DomainId,
        object: &T, 
        proof: &CommitmentProof
    ) -> std::result::Result<bool, CrossDomainError>;
    
    /// Query content by parameters
    async fn query_content(
        &self,
        domain_id: &DomainId,
        params: &HashMap<String, String>
    ) -> std::result::Result<Vec<ContentId>, CrossDomainError>;
}

/// Registry for content-addressed cross-domain interfaces
#[derive(Debug)]
pub struct ContentAddressedDomainRegistry {
    /// Domain adapters for cross-domain communication
    adapters: RwLock<HashMap<DomainId, Arc<dyn DomainAdapter>>>,
    
    /// Content storage for local objects
    storage: Arc<dyn ContentAddressedStorage>,
}

impl ContentAddressedDomainRegistry {
    /// Create a new cross-domain registry
    pub fn new(storage: Arc<dyn ContentAddressedStorage>) -> Self {
        Self {
            adapters: RwLock::new(HashMap::new()),
            storage,
        }
    }
    
    /// Register a domain adapter
    pub fn register_adapter(&self, adapter: Arc<dyn DomainAdapter>) -> Result<()> {
        let domain_id = adapter.domain_id().clone();
        let mut adapters = self.adapters.write().map_err(|_| 
            Error::SystemError("Failed to acquire write lock on cross-domain registry".to_string())
        )?;
        
        adapters.insert(domain_id, adapter);
        Ok(())
    }
    
    /// Unregister a domain adapter
    pub fn unregister_adapter(&self, domain_id: &DomainId) -> Result<()> {
        let mut adapters = self.adapters.write().map_err(|_| 
            Error::SystemError("Failed to acquire write lock on cross-domain registry".to_string())
        )?;
        
        adapters.remove(domain_id);
        Ok(())
    }
    
    /// Get a domain adapter
    pub fn get_adapter(&self, domain_id: &DomainId) -> Result<Arc<dyn DomainAdapter>> {
        let adapters = self.adapters.read().map_err(|_| 
            Error::SystemError("Failed to acquire read lock on cross-domain registry".to_string())
        )?;
        
        adapters.get(domain_id)
            .cloned()
            .ok_or_else(|| Error::DomainNotFound(domain_id.clone()))
    }
}

/// Implementation of the content-addressed cross-domain interface
#[async_trait]
impl ContentAddressedDomainInterface for ContentAddressedDomainRegistry {
    async fn resolve_content(&self, domain_id: &DomainId, id: &ContentId) -> std::result::Result<Vec<u8>, CrossDomainError> {
        // First check if we have the content in local storage
        if self.storage.contains(id) {
            return self.storage.get_bytes(id).map_err(|e| e.into());
        }
        
        // Get the domain adapter
        let adapter = self.get_adapter(domain_id)
            .map_err(|_| CrossDomainError::DomainNotFound(domain_id.clone()))?;
        
        // Query the domain for the content
        // This would typically involve a specialized message to the domain
        // For now, we'll use a fact query as a placeholder
        let params = HashMap::from([
            ("content_id".to_string(), id.to_string()),
            ("operation".to_string(), "resolve_content".to_string()),
        ]);
        
        let query = crate::domain::FactQuery {
            fact_type: "content_addressed_object".to_string(),
            parameters: params,
            domain_id: domain_id.clone(),
        };
        
        let result = adapter.observe_fact(&query).await
            .map_err(|e| CrossDomainError::Other(format!("Failed to observe fact: {}", e)))?;
        
        // Extract the binary data from the fact
        match result.0 {
            causality_domain::FactType::Binary(data) => Ok(data),
            _ => Err(CrossDomainError::ObjectNotFound(
                domain_id.clone(),
                id.to_string(),
            )),
        }
    }
    
    async fn verify_content<T: ContentAddressed + Send + Sync>(
        &self, 
        domain_id: &DomainId,
        object: &T, 
        proof: &CommitmentProof
    ) -> std::result::Result<bool, CrossDomainError> {
        // Get the domain adapter
        let adapter = self.get_adapter(domain_id)
            .map_err(|_| CrossDomainError::DomainNotFound(domain_id.clone()))?;
        
        // Calculate the expected content hash
        let content_hash = object.content_hash();
        
        // Ensure the leaf hash matches the object's content hash
        if content_hash != proof.leaf_hash {
            return Err(CrossDomainError::VerificationFailed(
                "Leaf hash does not match object content hash".to_string()
            ));
        }
        
        // For verification, we need to get the current root hash from the domain
        // This would typically involve a specialized query to the domain
        // For now, we'll use a fact query as a placeholder
        let params = HashMap::from([
            ("timestamp".to_string(), proof.timestamp.to_string()),
            ("operation".to_string(), "get_root_hash".to_string()),
        ]);
        
        let query = crate::domain::FactQuery {
            fact_type: "commitment_root".to_string(),
            parameters: params,
            domain_id: domain_id.clone(),
        };
        
        let result = adapter.observe_fact(&query).await
            .map_err(|e| CrossDomainError::Other(format!("Failed to observe root fact: {}", e)))?;
        
        // Extract the root hash from the fact
        let root_hash = match result.0 {
            causality_domain::FactType::Binary(data) => {
                if data.len() != 32 {
                    return Err(CrossDomainError::VerificationFailed(
                        "Invalid root hash length".to_string()
                    ));
                }
                
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&data);
                HashOutput::new(hash)
            },
            _ => return Err(CrossDomainError::VerificationFailed(
                "Invalid root hash format".to_string()
            )),
        };
        
        // Verify the proof against the root hash
        Ok(proof.verify(&root_hash))
    }
    
    async fn query_content(
        &self,
        domain_id: &DomainId,
        params: &HashMap<String, String>
    ) -> std::result::Result<Vec<ContentId>, CrossDomainError> {
        // Get the domain adapter
        let adapter = self.get_adapter(domain_id)
            .map_err(|_| CrossDomainError::DomainNotFound(domain_id.clone()))?;
        
        // Build a query to find content IDs matching the parameters
        let mut query_params = params.clone();
        query_params.insert("operation".to_string(), "query_content".to_string());
        
        let query = crate::domain::FactQuery {
            fact_type: "content_query".to_string(),
            parameters: query_params,
            domain_id: domain_id.clone(),
        };
        
        let result = adapter.observe_fact(&query).await
            .map_err(|e| CrossDomainError::QueryFailed(format!("Query failed: {}", e)))?;
        
        // Extract the content IDs from the query result
        match result.0 {
            causality_domain::FactType::String(data) => {
                // Parse comma-separated list of content IDs
                let ids: Vec<ContentId> = data.split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| ContentId::parse(s))
                    .collect::<Result<Vec<ContentId>, HashError>>()
                    .map_err(|e| CrossDomainError::HashError(e))?;
                
                Ok(ids)
            },
            causality_domain::FactType::Json(json) => {
                // Expect an array of content ID strings
                if let Some(array) = json.as_array() {
                    let mut ids = Vec::with_capacity(array.len());
                    
                    for item in array {
                        if let Some(id_str) = item.as_str() {
                            let id = ContentId::parse(id_str)
                                .map_err(|e| CrossDomainError::HashError(e))?;
                            ids.push(id);
                        } else {
                            return Err(CrossDomainError::QueryFailed(
                                "Invalid content ID format in JSON array".to_string()
                            ));
                        }
                    }
                    
                    Ok(ids)
                } else {
                    Err(CrossDomainError::QueryFailed(
                        "Expected JSON array result".to_string()
                    ))
                }
            },
            _ => Err(CrossDomainError::QueryFailed(
                "Unexpected result format".to_string()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::StorageFactory;
    use causality_domain::{
        FactType, FactObservationMeta, BlockHeight, BlockHash, Timestamp
    };
    use std::sync::Mutex;
    
    // Mock domain adapter for testing
    struct MockDomainAdapter {
        domain_id: DomainId,
        mock_facts: Mutex<HashMap<String, FactType>>,
    }
    
    impl MockDomainAdapter {
        fn new(domain_id: DomainId) -> Self {
            Self {
                domain_id,
                mock_facts: Mutex::new(HashMap::new()),
            }
        }
        
        fn add_mock_fact(&self, key: impl Into<String>, fact: FactType) {
            let mut facts = self.mock_facts.lock().unwrap();
            facts.insert(key.into(), fact);
        }
    }
    
    #[async_trait]
    impl DomainAdapter for MockDomainAdapter {
        fn domain_id(&self) -> &DomainId {
            &self.domain_id
        }
        
        async fn domain_info(&self) -> Result<causality_domain::DomainInfo> {
            todo!()
        }
        
        async fn current_height(&self) -> Result<BlockHeight> {
            Ok(BlockHeight(100))
        }
        
        async fn current_hash(&self) -> Result<BlockHash> {
            Ok(BlockHash([0; 32]))
        }
        
        async fn current_time(&self) -> Result<Timestamp> {
            Ok(Timestamp::now())
        }
        
        async fn time_map_entry(&self, _height: BlockHeight) -> Result<causality_domain::TimeMapEntry> {
            todo!()
        }
        
        async fn observe_fact(&self, query: &crate::domain::FactQuery) -> causality_domain::FactResult {
            let facts = self.mock_facts.lock().unwrap();
            
            // Construct a unique key for this query
            let key = format!(
                "{}:{}:{}",
                query.fact_type,
                query.parameters.get("operation").unwrap_or(&"unknown".to_string()),
                query.parameters.get("content_id").unwrap_or(&"".to_string())
            );
            
            if let Some(fact) = facts.get(&key) {
                let meta = FactObservationMeta {
                    observed_at: Timestamp::now(),
                    block_height: Some(BlockHeight(100)),
                    reliability: 1.0,
                    source: "mock".to_string(),
                    metadata: HashMap::new(),
                };
                
                Ok((fact.clone(), meta))
            } else {
                Err(Error::FactNotFound)
            }
        }
        
        async fn submit_transaction(&self, _tx: causality_domain::Transaction) -> Result<causality_domain::TransactionId> {
            todo!()
        }
        
        async fn transaction_receipt(&self, _tx_id: &causality_domain::TransactionId) -> Result<causality_domain::TransactionReceipt> {
            todo!()
        }
        
        async fn transaction_confirmed(&self, _tx_id: &causality_domain::TransactionId) -> Result<bool> {
            todo!()
        }
        
        async fn wait_for_confirmation(&self, _tx_id: &causality_domain::TransactionId, _max_wait_ms: Option<u64>) -> Result<causality_domain::TransactionReceipt> {
            todo!()
        }
    }
    
    // Simple test object that implements ContentAddressed
    #[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
    struct TestObject {
        id: String,
        data: Vec<u8>,
    }
    
    impl TestObject {
        fn new(id: &str, data: &[u8]) -> Self {
            Self {
                id: id.to_string(),
                data: data.to_vec(),
            }
        }
    }
    
    impl ContentAddressed for TestObject {
        fn content_hash(&self) -> HashOutput {
            let hasher = HashFactory::default().create_hasher().unwrap();
            let data = self.try_to_vec().unwrap();
            hasher.hash(&data)
        }
        
        fn verify(&self) -> bool {
            true
        }
        
        fn to_bytes(&self) -> Vec<u8> {
            self.try_to_vec().unwrap()
        }
        
        fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
            BorshDeserialize::try_from_slice(bytes)
                .map_err(|e| HashError::SerializationError(e.to_string()))
        }
    }
    
    #[tokio::test]
    async fn test_resolve_content() {
        // Create storage and registry
        let storage = StorageFactory::default().create_storage();
        let registry = ContentAddressedDomainRegistry::new(storage.clone());
        
        // Create a test domain adapter
        let domain_id = DomainId::new("test-domain");
        let adapter = Arc::new(MockDomainAdapter::new(domain_id.clone()));
        
        // Register the adapter
        registry.register_adapter(adapter.clone()).unwrap();
        
        // Create a test object
        let obj = TestObject::new("test", b"test data");
        let content_id = obj.content_id();
        
        // Store the object locally
        storage.store(&obj).unwrap();
        
        // Resolve the object
        let bytes = registry.resolve_content(&domain_id, &content_id).await.unwrap();
        
        // Deserialize and compare
        let resolved_obj = TestObject::from_bytes(&bytes).unwrap();
        assert_eq!(resolved_obj.id, obj.id);
        assert_eq!(resolved_obj.data, obj.data);
    }
    
    #[tokio::test]
    async fn test_verify_content() {
        // Create storage and registry
        let storage = StorageFactory::default().create_storage();
        let registry = ContentAddressedDomainRegistry::new(storage);
        
        // Create a test domain adapter
        let domain_id = DomainId::new("test-domain");
        let adapter = Arc::new(MockDomainAdapter::new(domain_id.clone()));
        
        // Create a test object
        let obj = TestObject::new("test", b"test data");
        let content_hash = obj.content_hash();
        
        // Create a mock proof
        let root_hash = HashOutput::new([1; 32]);
        let proof = CommitmentProof::new(
            domain_id.clone(),
            root_hash.clone(),
            vec![HashOutput::new([2; 32]), HashOutput::new([3; 32])],
            content_hash,
        );
        
        // Register the adapter
        registry.register_adapter(adapter.clone()).unwrap();
        
        // Add a mock fact for the root hash
        adapter.add_mock_fact(
            "commitment_root:get_root_hash:",
            FactType::Binary(root_hash.as_bytes().to_vec()),
        );
        
        // Verify the content
        let verified = registry.verify_content(&domain_id, &obj, &proof).await.unwrap();
        assert!(verified);
    }
    
    #[tokio::test]
    async fn test_query_content() {
        // Create storage and registry
        let storage = StorageFactory::default().create_storage();
        let registry = ContentAddressedDomainRegistry::new(storage);
        
        // Create a test domain adapter
        let domain_id = DomainId::new("test-domain");
        let adapter = Arc::new(MockDomainAdapter::new(domain_id.clone()));
        
        // Register the adapter
        registry.register_adapter(adapter.clone()).unwrap();
        
        // Create some test content IDs
        let content_id1 = ContentId::parse("cid:0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let content_id2 = ContentId::parse("cid:0000000000000000000000000000000000000000000000000000000000000002").unwrap();
        
        // Add a mock fact for the query
        adapter.add_mock_fact(
            "content_query:query_content:",
            FactType::String(format!("{},{}", content_id1, content_id2)),
        );
        
        // Query content
        let params = HashMap::new();
        let ids = registry.query_content(&domain_id, &params).await.unwrap();
        
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], content_id1);
        assert_eq!(ids[1], content_id2);
    }
} 