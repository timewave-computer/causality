// Content-addressed Fact System
//
// This module implements a content-addressed version of the temporal fact system,
// ensuring that facts can be verified and retrieved by their content hash.

use std::collections::{HashMap, BTreeMap};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use borsh::{BorshSerialize, BorshDeserialize};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use getrandom;

use crate::crypto::hash::{ContentAddressed, ContentId, HashOutput, HashError, HashFactory};
use crate::types::{*};
use crate::crypto::hash::ContentId;;
use crate::log::fact_types::{FactType, RegisterFact, ZKProofFact};
use crate::crypto::content_addressed_storage::{ContentAddressedStorage, StorageError};

/// Error type for fact operations
#[derive(Debug, Error)]
pub enum FactError {
    /// Fact not found
    #[error("Fact not found: {0}")]
    NotFound(String),
    
    /// Fact already exists
    #[error("Fact already exists: {0}")]
    AlreadyExists(String),
    
    /// Invalid fact data
    #[error("Invalid fact data: {0}")]
    InvalidData(String),
    
    /// Verification failed
    #[error("Fact verification failed: {0}")]
    VerificationFailed(String),
    
    /// Dependency error
    #[error("Fact dependency error: {0}")]
    DependencyError(String),
    
    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
    
    /// Hash error
    #[error("Hash error: {0}")]
    HashError(#[from] HashError),
}

/// Enum for fact content types
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum FactContent {
    /// JSON-encoded content
    Json(String),
    
    /// Binary content
    Binary(Vec<u8>),
    
    /// Content hash (for large content stored elsewhere)
    ContentHash {
        /// Hash of the content
        hash: String,
        /// Location where the full content can be retrieved
        location: String,
    },
}

/// Enum for fact proof types
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum FactProof {
    /// Signature-based proof
    Signature {
        /// Signature data
        signature: Vec<u8>,
        /// Public key that created the signature
        public_key: Vec<u8>,
    },
    
    /// Merkle proof
    MerkleProof {
        /// Merkle root
        root: String,
        /// Proof path
        path: Vec<(bool, String)>,
    },
    
    /// Zero-knowledge proof
    ZkProof {
        /// ZK proof data
        proof: Vec<u8>,
    },
    
    /// Custom proof type
    Custom {
        /// Proof type identifier
        proof_type: String,
        /// Proof data
        data: Vec<u8>,
    },
}

/// Unique identifier for facts
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct FactId(pub String);

impl FactId {
    /// Create a new fact ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Generate a content-addressed fact ID
    pub fn generate() -> Self {
        // Generate timestamp for uniqueness
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        
        // Add some randomness
        let mut nonce = [0u8; 8];
        getrandom::getrandom(&mut nonce).expect("Failed to generate random nonce");
        
        // Combine data for ID generation
        let id_source = format!("fact:{}:{:?}", now, nonce);
        
        // Use the crypto module's hash function
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().expect("Failed to create hasher");
        
        // Generate a content-based ID
        let hash = hasher.hash(id_source.as_bytes());
        Self(format!("fact:{}", hash.to_hex().split_at(16).0))
    }
    
    /// Get the underlying ID string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A content-addressed temporal fact
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ContentAddressedFact {
    /// Unique identifier for this fact
    pub id: FactId,
    
    /// Type of the fact
    pub fact_type: FactType,
    
    /// Timestamp when the fact was created
    pub timestamp: Timestamp,
    
    /// Domain that created the fact
    pub origin_domain: DomainId,
    
    /// Content of the fact
    pub content: FactContent,
    
    /// Cryptographic proof of the fact
    pub proof: Option<FactProof>,
    
    /// References to other facts this fact depends on
    pub dependencies: Vec<ContentId>,
    
    /// Optional metadata associated with this fact
    pub metadata: HashMap<String, String>,
}

impl ContentAddressedFact {
    /// Create a new content-addressed fact
    pub fn new(
        id: FactId,
        fact_type: FactType,
        timestamp: Timestamp,
        origin_domain: DomainId,
        content: FactContent,
        proof: Option<FactProof>,
        dependencies: Vec<ContentId>,
    ) -> Self {
        Self {
            id,
            fact_type,
            timestamp,
            origin_domain,
            content,
            proof,
            dependencies,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the fact
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Get the resource ID if this fact is about a resource
    pub fn resource_id(&self) -> Option<&ContentId> {
        match &self.fact_type {
            FactType::RegisterFact(RegisterFact::RegisterCreation { register_id, .. }) => Some(register_id),
            FactType::RegisterFact(RegisterFact::RegisterUpdate { register_id, .. }) => Some(register_id),
            FactType::RegisterFact(RegisterFact::RegisterTransfer { register_id, .. }) => Some(register_id),
            FactType::RegisterFact(RegisterFact::RegisterMerge { result_register, .. }) => Some(result_register),
            FactType::RegisterFact(RegisterFact::RegisterSplit { source_register, .. }) => Some(source_register),
            FactType::RegisterFact(RegisterFact::RegisterConsumption { register_id, .. }) => Some(register_id),
            FactType::RegisterFact(RegisterFact::RegisterStateChange { register_id, .. }) => Some(register_id),
            FactType::RegisterFact(RegisterFact::RegisterOwnershipTransfer { register_id, .. }) => Some(register_id),
            FactType::RegisterFact(RegisterFact::RegisterLock { register_id, .. }) => Some(register_id),
            FactType::RegisterFact(RegisterFact::RegisterUnlock { register_id, .. }) => Some(register_id),
            FactType::RegisterFact(RegisterFact::RegisterEpochTransition { register_id, .. }) => Some(register_id),
            FactType::RegisterFact(RegisterFact::RegisterSummarization { summary_register_id, .. }) => Some(summary_register_id),
            FactType::RegisterFact(RegisterFact::RegisterArchival { register_id, .. }) => Some(register_id),
            FactType::RegisterFact(RegisterFact::RegisterAuthorization { register_id, .. }) => Some(register_id),
            FactType::RegisterFact(RegisterFact::RegisterNullifierCreation { register_id, .. }) => Some(register_id),
            _ => None,
        }
    }
    
    /// Validate the fact's dependencies
    pub fn validate_dependencies<S: ContentAddressedStorage>(
        &self,
        storage: &S,
    ) -> Result<bool, FactError> {
        // Check if all dependencies exist in storage
        for dep_id in &self.dependencies {
            if !storage.contains(dep_id) {
                return Err(FactError::DependencyError(
                    format!("Dependency not found: {}", dep_id)
                ));
            }
        }
        
        Ok(true)
    }
}

impl ContentAddressed for ContentAddressedFact {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        
        // Create a canonical serialization of the fact
        let data = self.try_to_vec().unwrap();
        
        // Compute hash with configured hasher
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
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

/// A key for fact type indexing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactTypeKey(String);

impl FactTypeKey {
    /// Create a new fact type key
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
}

impl From<&FactType> for FactTypeKey {
    fn from(fact_type: &FactType) -> Self {
        match fact_type {
            FactType::BalanceFact => Self("balance".into()),
            FactType::TransactionFact => Self("transaction".into()),
            FactType::OracleFact => Self("oracle".into()),
            FactType::BlockFact => Self("block".into()),
            FactType::TimeFact => Self("time".into()),
            FactType::RegisterFact(_) => Self("register".into()),
            FactType::ZKProofFact(_) => Self("zkproof".into()),
            FactType::Custom(name) => Self(format!("custom:{}", name)),
        }
    }
}

/// Filter for querying facts
#[derive(Debug, Clone)]
pub struct FactFilter {
    /// Filter by fact type
    pub fact_type: Option<FactType>,
    
    /// Filter by resource ID
    pub resource_id: Option<ContentId>,
    
    /// Filter by time range
    pub time_range: Option<(Timestamp, Timestamp)>,
    
    /// Filter by origin domain
    pub origin_domain: Option<DomainId>,
}

/// A registry for content-addressed facts
pub struct ContentAddressedFactRegistry {
    /// Storage for facts
    storage: Arc<dyn ContentAddressedStorage>,
    
    /// Index of facts by resource
    resource_index: RwLock<HashMap<ContentId, Vec<ContentId>>>,
    
    /// Index of facts by timestamp
    time_index: RwLock<BTreeMap<Timestamp, Vec<ContentId>>>,
    
    /// Index of facts by origin domain
    domain_index: RwLock<HashMap<DomainId, Vec<ContentId>>>,
    
    /// Index of facts by type
    type_index: RwLock<HashMap<FactTypeKey, Vec<ContentId>>>,
    
    /// Map of fact IDs to content IDs
    fact_id_map: RwLock<HashMap<FactId, ContentId>>,
}

impl ContentAddressedFactRegistry {
    /// Create a new content-addressed fact registry
    pub fn new(storage: Arc<dyn ContentAddressedStorage>) -> Self {
        Self {
            storage,
            resource_index: RwLock::new(HashMap::new()),
            time_index: RwLock::new(BTreeMap::new()),
            domain_index: RwLock::new(HashMap::new()),
            type_index: RwLock::new(HashMap::new()),
            fact_id_map: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a new fact
    pub fn register_fact(&self, fact: ContentAddressedFact) -> Result<ContentId, FactError> {
        // Validate the fact before registering
        if !fact.verify() {
            return Err(FactError::VerificationFailed(
                format!("Fact failed verification: {:?}", fact.id)
            ));
        }
        
        // Validate dependencies
        fact.validate_dependencies(&*self.storage)?;
        
        // Store the fact
        let content_id = self.storage.store(&fact)
            .map_err(|e| FactError::StorageError(e))?;
        
        // Update indices
        self.update_indices(&fact, &content_id)?;
        
        // Map fact ID to content ID
        let mut fact_id_map = self.fact_id_map.write().unwrap();
        fact_id_map.insert(fact.id.clone(), content_id.clone());
        
        Ok(content_id)
    }
    
    /// Update the indices for a fact
    fn update_indices(&self, fact: &ContentAddressedFact, content_id: &ContentId) -> Result<(), FactError> {
        // Update resource index if applicable
        if let Some(resource_id) = fact.resource_id() {
            let mut resource_index = self.resource_index.write().unwrap();
            resource_index
                .entry(resource_id.clone())
                .or_insert_with(Vec::new)
                .push(content_id.clone());
        }
        
        // Update time index
        let mut time_index = self.time_index.write().unwrap();
        time_index
            .entry(fact.timestamp)
            .or_insert_with(Vec::new)
            .push(content_id.clone());
        
        // Update domain index
        let mut domain_index = self.domain_index.write().unwrap();
        domain_index
            .entry(fact.origin_domain.clone())
            .or_insert_with(Vec::new)
            .push(content_id.clone());
        
        // Update type index
        let type_key = FactTypeKey::from(&fact.fact_type);
        let mut type_index = self.type_index.write().unwrap();
        type_index
            .entry(type_key)
            .or_insert_with(Vec::new)
            .push(content_id.clone());
        
        Ok(())
    }
    
    /// Get a fact by its content ID
    pub fn get_fact(&self, content_id: &ContentId) -> Result<ContentAddressedFact, FactError> {
        self.storage.get(content_id)
            .map_err(|e| FactError::from(e))
    }
    
    /// Get a fact by its fact ID
    pub fn get_fact_by_id(&self, fact_id: &FactId) -> Result<ContentAddressedFact, FactError> {
        let fact_id_map = self.fact_id_map.read().unwrap();
        
        match fact_id_map.get(fact_id) {
            Some(content_id) => self.get_fact(content_id),
            None => Err(FactError::NotFound(format!("Fact not found: {:?}", fact_id))),
        }
    }
    
    /// Query facts based on a filter
    pub fn query_facts(
        &self,
        filter: &FactFilter,
        limit: Option<usize>,
    ) -> Result<Vec<ContentAddressedFact>, FactError> {
        // Determine the most efficient index to query
        let content_ids = if let Some(resource_id) = &filter.resource_id {
            // Query by resource ID
            let resource_index = self.resource_index.read().unwrap();
            resource_index.get(resource_id)
                .map(|ids| ids.clone())
                .unwrap_or_default()
        } else if let Some((start, end)) = &filter.time_range {
            // Query by time range
            let time_index = self.time_index.read().unwrap();
            let mut ids = Vec::new();
            
            for (timestamp, time_ids) in time_index.range(start.clone()..=end.clone()) {
                ids.extend(time_ids.clone());
            }
            
            ids
        } else if let Some(domain_id) = &filter.origin_domain {
            // Query by domain
            let domain_index = self.domain_index.read().unwrap();
            domain_index.get(domain_id)
                .map(|ids| ids.clone())
                .unwrap_or_default()
        } else if let Some(fact_type) = &filter.fact_type {
            // Query by fact type
            let type_key = FactTypeKey::from(fact_type);
            let type_index = self.type_index.read().unwrap();
            type_index.get(&type_key)
                .map(|ids| ids.clone())
                .unwrap_or_default()
        } else {
            // No specific filter, use the fact ID map for all facts
            let fact_id_map = self.fact_id_map.read().unwrap();
            fact_id_map.values().cloned().collect()
        };
        
        // Retrieve the facts
        let mut facts = Vec::new();
        for content_id in content_ids {
            if let Ok(fact) = self.get_fact(&content_id) {
                // Apply any remaining filters
                if let Some(resource_id) = &filter.resource_id {
                    if fact.resource_id().map(|id| id != resource_id).unwrap_or(true) {
                        continue;
                    }
                }
                
                if let Some((start, end)) = &filter.time_range {
                    if fact.timestamp < *start || fact.timestamp > *end {
                        continue;
                    }
                }
                
                if let Some(domain_id) = &filter.origin_domain {
                    if &fact.origin_domain != domain_id {
                        continue;
                    }
                }
                
                if let Some(fact_type) = &filter.fact_type {
                    // This is a simplified check, might need more sophisticated matching
                    match (fact_type, &fact.fact_type) {
                        (FactType::BalanceFact, FactType::BalanceFact) => {},
                        (FactType::TransactionFact, FactType::TransactionFact) => {},
                        (FactType::OracleFact, FactType::OracleFact) => {},
                        (FactType::BlockFact, FactType::BlockFact) => {},
                        (FactType::TimeFact, FactType::TimeFact) => {},
                        (FactType::RegisterFact(_), FactType::RegisterFact(_)) => {},
                        (FactType::ZKProofFact(_), FactType::ZKProofFact(_)) => {},
                        (FactType::Custom(a), FactType::Custom(b)) if a == b => {},
                        _ => continue,
                    }
                }
                
                facts.push(fact);
                
                // Check if we've reached the limit
                if let Some(limit) = limit {
                    if facts.len() >= limit {
                        break;
                    }
                }
            }
        }
        
        Ok(facts)
    }
    
    /// Get facts for a specific resource
    pub fn get_facts_for_resource(
        &self,
        resource_id: &ContentId,
        time_range: Option<(Timestamp, Timestamp)>,
    ) -> Result<Vec<ContentAddressedFact>, FactError> {
        let filter = FactFilter {
            fact_type: None,
            resource_id: Some(resource_id.clone()),
            time_range,
            origin_domain: None,
        };
        
        self.query_facts(&filter, None)
    }
    
    /// Get the number of facts in the registry
    pub fn count(&self) -> usize {
        let fact_id_map = self.fact_id_map.read().unwrap();
        fact_id_map.len()
    }
    
    /// Clear all facts and indices
    pub fn clear(&self) {
        self.storage.clear();
        
        let mut resource_index = self.resource_index.write().unwrap();
        resource_index.clear();
        
        let mut time_index = self.time_index.write().unwrap();
        time_index.clear();
        
        let mut domain_index = self.domain_index.write().unwrap();
        domain_index.clear();
        
        let mut type_index = self.type_index.write().unwrap();
        type_index.clear();
        
        let mut fact_id_map = self.fact_id_map.write().unwrap();
        fact_id_map.clear();
    }
}

/// A factory for creating content-addressed fact registries
pub struct FactRegistryFactory;

impl FactRegistryFactory {
    /// Create a new in-memory content-addressed fact registry
    pub fn create_memory_registry() -> Arc<ContentAddressedFactRegistry> {
        let storage = crate::crypto::content_addressed_storage::StorageFactory::create_memory_storage();
        Arc::new(ContentAddressedFactRegistry::new(storage))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::content_addressed_storage::InMemoryStorage;
    
    #[test]
    fn test_content_addressed_fact() {
        // Create a test fact
        let fact = ContentAddressedFact::new(
            FactId::generate(),
            FactType::TimeFact,
            Timestamp(100),
            "test-domain".to_string(),
            FactContent::Json(r#"{"value": 42}"#.to_string()),
            None,
            Vec::new(),
        );
        
        // Verify content addressing
        assert!(fact.verify());
        let content_id = fact.content_id();
        
        // Serialize and deserialize
        let bytes = fact.to_bytes();
        let deserialized = ContentAddressedFact::from_bytes(&bytes).unwrap();
        
        // Verify deserialized fact
        assert_eq!(deserialized.id.0, fact.id.0);
        assert_eq!(deserialized.timestamp, fact.timestamp);
        assert_eq!(deserialized.origin_domain, fact.origin_domain);
        assert!(deserialized.verify());
        assert_eq!(deserialized.content_id(), content_id);
    }
    
    #[test]
    fn test_fact_registry() {
        // Create a registry
        let storage = Arc::new(InMemoryStorage::new());
        let registry = ContentAddressedFactRegistry::new(storage);
        
        // Create test facts
        let fact1 = ContentAddressedFact::new(
            FactId::generate(),
            FactType::BlockFact,
            Timestamp(100),
            "domain1".to_string(),
            FactContent::Json(r#"{"block": 1}"#.to_string()),
            None,
            Vec::new(),
        );
        
        let register_id = "register1".to_string();
        let fact2 = ContentAddressedFact::new(
            FactId::generate(),
            FactType::RegisterFact(RegisterFact::RegisterCreation {
                register_id: register_id.clone(),
                initial_data: vec![1, 2, 3],
                owner: "owner1".to_string(),
                domain: "domain1".to_string(),
            }),
            Timestamp(200),
            "domain1".to_string(),
            FactContent::Binary(vec![4, 5, 6]),
            None,
            Vec::new(),
        );
        
        // Register facts
        let content_id1 = registry.register_fact(fact1.clone()).unwrap();
        let content_id2 = registry.register_fact(fact2.clone()).unwrap();
        
        // Retrieve facts
        let retrieved1 = registry.get_fact(&content_id1).unwrap();
        let retrieved2 = registry.get_fact(&content_id2).unwrap();
        
        assert_eq!(retrieved1.id.0, fact1.id.0);
        assert_eq!(retrieved2.id.0, fact2.id.0);
        
        // Query by fact ID
        let by_fact_id = registry.get_fact_by_id(&fact1.id).unwrap();
        assert_eq!(by_fact_id.id.0, fact1.id.0);
        
        // Query by resource
        let by_resource = registry.get_facts_for_resource(&register_id, None).unwrap();
        assert_eq!(by_resource.len(), 1);
        assert_eq!(by_resource[0].id.0, fact2.id.0);
        
        // Query by filter
        let filter = FactFilter {
            fact_type: Some(FactType::BlockFact),
            resource_id: None,
            time_range: None,
            origin_domain: None,
        };
        
        let filtered = registry.query_facts(&filter, None).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id.0, fact1.id.0);
        
        // Count
        assert_eq!(registry.count(), 2);
        
        // Clear
        registry.clear();
        assert_eq!(registry.count(), 0);
    }
} 
