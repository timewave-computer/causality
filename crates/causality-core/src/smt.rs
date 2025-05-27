// Purpose: Sparse Merkle Tree implementation for content-addressable storage
// with SHA256 hashing for compatibility with OCaml implementation

use causality_types::{
    core::{Effect, Handler, Intent, Resource},
    core::id::{EffectId, HandlerId, IntentId, ResourceId, ExprId, DomainId, AsId},
    serialization::{Encode, Decode},
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

// Hash type - 32 bytes for SHA256
pub type Hash = [u8; 32];

// Helper to create empty hash
pub fn empty_hash() -> Hash {
    [0u8; 32]
}

// Helper to convert hash to hex string
pub fn hash_to_hex(hash: &Hash) -> String {
    hex::encode(hash)
}

// Helper to convert hex string to hash
pub fn hash_from_hex(hex_str: &str) -> Result<Hash, hex::FromHexError> {
    let mut hash = [0u8; 32];
    hex::decode_to_slice(hex_str, &mut hash)?;
    Ok(hash)
}

// SMT children structure
#[derive(Debug, Clone)]
pub struct SmtChildren {
    pub left: Hash,
    pub right: Hash,
}

// Merkle opening proof
#[derive(Debug, Clone)]
pub struct Opening {
    pub path: Vec<Hash>,
}

// Storage namespaces matching valence-coprocessor
pub const PREFIX_NODE: &str = "smt-node";
pub const PREFIX_DATA: &str = "smt-data";
pub const PREFIX_KEY: &str = "smt-key";

// TEG-specific namespaces for different data types
pub const TEG_EFFECT_NAMESPACE: &str = "teg-effect";
pub const TEG_HANDLER_NAMESPACE: &str = "teg-handler";
pub const TEG_RESOURCE_NAMESPACE: &str = "teg-resource";
pub const TEG_INTENT_NAMESPACE: &str = "teg-intent";
pub const TEG_CONSTRAINT_NAMESPACE: &str = "teg-constraint";
pub const TEG_NODE_NAMESPACE: &str = "teg-node";
pub const TEG_EDGE_NAMESPACE: &str = "teg-edge";
pub const TEG_CROSS_DOMAIN_NAMESPACE: &str = "teg-cross-domain";
pub const TEG_TEMPORAL_NAMESPACE: &str = "teg-temporal";

// SHA256 hasher implementation
pub struct Sha256Hasher;

impl Sha256Hasher {
    // Prefixes for different hash types
    const DATA_PREFIX: &'static [u8] = &[0x00];
    const MERGE_PREFIX: &'static [u8] = &[0x01];
    
    // Generate key from context and data
    pub fn key(context: &str, data: &[u8]) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(context.as_bytes());
        hasher.update(data);
        hasher.finalize().into()
    }
    
    // Hash data with data prefix
    pub fn hash(data: &[u8]) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(Self::DATA_PREFIX);
        hasher.update(data);
        hasher.finalize().into()
    }
    
    // Merge two hashes
    pub fn merge(left: &Hash, right: &Hash) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(Self::MERGE_PREFIX);
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }
    
    // Digest multiple byte arrays
    pub fn digest(data_list: &[&[u8]]) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(Self::DATA_PREFIX);
        for data in data_list {
            hasher.update(data);
        }
        hasher.finalize().into()
    }
}

// Storage backend trait
pub trait DataBackend {
    fn get(&self, prefix: &str, key: &Hash) -> Option<Vec<u8>>;
    fn set(&mut self, prefix: &str, key: &Hash, data: Vec<u8>) -> Option<Vec<u8>>;
    fn remove(&mut self, prefix: &str, key: &Hash) -> Option<Vec<u8>>;
    fn has(&self, prefix: &str, key: &Hash) -> bool;
}

// In-memory storage backend
#[derive(Debug, Default)]
pub struct MemoryBackend {
    storage: HashMap<(String, Hash), Vec<u8>>,
}

impl MemoryBackend {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
}

impl DataBackend for MemoryBackend {
    fn get(&self, prefix: &str, key: &Hash) -> Option<Vec<u8>> {
        let storage_key = (prefix.to_string(), *key);
        self.storage.get(&storage_key).cloned()
    }
    
    fn set(&mut self, prefix: &str, key: &Hash, data: Vec<u8>) -> Option<Vec<u8>> {
        let storage_key = (prefix.to_string(), *key);
        self.storage.insert(storage_key, data)
    }
    
    fn remove(&mut self, prefix: &str, key: &Hash) -> Option<Vec<u8>> {
        let storage_key = (prefix.to_string(), *key);
        self.storage.remove(&storage_key)
    }
    
    fn has(&self, prefix: &str, key: &Hash) -> bool {
        let storage_key = (prefix.to_string(), *key);
        self.storage.contains_key(&storage_key)
    }
}

// SMT implementation
#[derive(Debug)]
pub struct SmtSha256<B: DataBackend> {
    backend: B,
    root: Hash,
}

impl<B: DataBackend> SmtSha256<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            root: empty_hash(),
        }
    }
    
    pub fn get(&self, prefix: &str, key: &Hash) -> Option<Vec<u8>> {
        self.backend.get(prefix, key)
    }
    
    pub fn set(&mut self, prefix: &str, key: &Hash, data: Vec<u8>) -> Option<Vec<u8>> {
        let result = self.backend.set(prefix, key, data.clone());
        // TODO: Update root hash computation
        self.root = Sha256Hasher::key(prefix, &data);
        result
    }
    
    pub fn remove(&mut self, prefix: &str, key: &Hash) -> Option<Vec<u8>> {
        let result = self.backend.remove(prefix, key);
        // TODO: Update root hash computation
        self.root = empty_hash();
        result
    }
    
    pub fn has(&self, prefix: &str, key: &Hash) -> bool {
        self.backend.has(prefix, key)
    }
    
    pub fn get_root(&self) -> Hash {
        self.root
    }
    
    pub fn generate_proof(&self, _prefix: &str, _key: &Hash) -> Option<Opening> {
        // TODO: Implement proof generation
        None
    }
    
    pub fn verify_proof(&self, _root: &Hash, _prefix: &str, _key: &Hash, _data: &[u8], _proof: &Opening) -> bool {
        // TODO: Implement proof verification
        false
    }
    
    /// Store data with domain-namespaced string key
    pub fn store_namespaced_data(&mut self, namespaced_key: &str, data: &[u8]) -> Result<(), String> {
        let key_hash = Sha256Hasher::key("data", namespaced_key.as_bytes());
        self.set(PREFIX_DATA, &key_hash, data.to_vec());
        Ok(())
    }
    
    /// Get data with domain-namespaced string key
    pub fn get_namespaced_data(&self, namespaced_key: &str) -> Result<Option<Vec<u8>>, String> {
        let key_hash = Sha256Hasher::key("data", namespaced_key.as_bytes());
        Ok(self.get(PREFIX_DATA, &key_hash))
    }
    
    /// Check if domain-namespaced key exists
    pub fn has_namespaced_data(&self, namespaced_key: &str) -> bool {
        let key_hash = Sha256Hasher::key("data", namespaced_key.as_bytes());
        self.has(PREFIX_DATA, &key_hash)
    }
}

//-----------------------------------------------------------------------------
// TEG-Optimized Multi-Domain SMT Implementation
//-----------------------------------------------------------------------------

/// TEG-optimized multi-domain SMT with direct support for TEG data structures
/// Uses a single SMT tree with domain namespacing for efficiency
#[derive(Debug)]
pub struct TegMultiDomainSmt<B: DataBackend> {
    smt: SmtSha256<B>,
}

impl<B: DataBackend> TegMultiDomainSmt<B> {
    /// Create a new TEG-optimized multi-domain SMT
    pub fn new(backend: B) -> Self {
        Self {
            smt: SmtSha256::new(backend),
        }
    }
    
    /// Store a TEG effect with automatic domain-specific key generation
    pub fn store_teg_effect(&mut self, effect: &Effect) -> Result<String, String> {
        let key = effect.domain_id.generate_teg_effect_key(&effect.id.inner());
        let serialized_data = effect.as_ssz_bytes();
        
        self.smt.store_namespaced_data(&key, &serialized_data)
            .map_err(|e| format!("Failed to store TEG effect: {}", e))?;
        
        Ok(key)
    }
    
    /// Retrieve a TEG effect by domain and effect ID
    pub fn get_teg_effect(&self, domain_id: &DomainId, effect_id: &EffectId) -> Result<Option<Effect>, String> {
        let key = domain_id.generate_teg_effect_key(&effect_id.inner());
        
        if let Some(data) = self.smt.get_namespaced_data(&key)
            .map_err(|e| format!("Failed to get TEG effect: {}", e))? {
            
            let effect = Effect::from_ssz_bytes(&data)
                .map_err(|e| format!("Failed to deserialize TEG effect: {}", e.message))?;
            
            Ok(Some(effect))
        } else {
            Ok(None)
        }
    }
    
    /// Store a TEG handler with automatic domain-specific key generation
    pub fn store_teg_handler(&mut self, handler: &Handler) -> Result<String, String> {
        let key = handler.domain_id.generate_teg_handler_key(&handler.id.inner());
        let serialized_data = handler.as_ssz_bytes();
        
        self.smt.store_namespaced_data(&key, &serialized_data)
            .map_err(|e| format!("Failed to store TEG handler: {}", e))?;
        
        Ok(key)
    }
    
    /// Retrieve a TEG handler by domain and handler ID
    pub fn get_teg_handler(&self, domain_id: &DomainId, handler_id: &HandlerId) -> Result<Option<Handler>, String> {
        let key = domain_id.generate_teg_handler_key(&handler_id.inner());
        
        if let Some(data) = self.smt.get_namespaced_data(&key)
            .map_err(|e| format!("Failed to get TEG handler: {}", e))? {
            
            let handler = Handler::from_ssz_bytes(&data)
                .map_err(|e| format!("Failed to deserialize TEG handler: {}", e.message))?;
            
            Ok(Some(handler))
        } else {
            Ok(None)
        }
    }
    
    /// Store a TEG resource with automatic domain-specific key generation
    pub fn store_teg_resource(&mut self, resource: &Resource) -> Result<String, String> {
        let key = resource.domain_id.generate_teg_resource_key(&resource.id.inner());
        let serialized_data = resource.as_ssz_bytes();
        
        self.smt.store_namespaced_data(&key, &serialized_data)
            .map_err(|e| format!("Failed to store TEG resource: {}", e))?;
        
        Ok(key)
    }
    
    /// Retrieve a TEG resource by domain and resource ID
    pub fn get_teg_resource(&self, domain_id: &DomainId, resource_id: &ResourceId) -> Result<Option<Resource>, String> {
        let key = domain_id.generate_teg_resource_key(&resource_id.0);
        
        if let Some(data) = self.smt.get_namespaced_data(&key)
            .map_err(|e| format!("Failed to get TEG resource: {}", e))? {
            
            let resource = Resource::from_ssz_bytes(&data)
                .map_err(|e| format!("Failed to deserialize TEG resource: {}", e.message))?;
            
            Ok(Some(resource))
        } else {
            Ok(None)
        }
    }
    
    /// Store a TEG intent with automatic domain-specific key generation
    pub fn store_teg_intent(&mut self, intent: &Intent) -> Result<String, String> {
        let key = intent.domain_id.generate_teg_intent_key(&intent.id.0);
        let serialized_data = intent.as_ssz_bytes();
        
        self.smt.store_namespaced_data(&key, &serialized_data)
            .map_err(|e| format!("Failed to store TEG intent: {}", e))?;
        
        Ok(key)
    }
    
    /// Retrieve a TEG intent by domain and intent ID
    pub fn get_teg_intent(&self, domain_id: &DomainId, intent_id: &IntentId) -> Result<Option<Intent>, String> {
        let key = domain_id.generate_teg_intent_key(&intent_id.0);
        
        if let Some(data) = self.smt.get_namespaced_data(&key)
            .map_err(|e| format!("Failed to get TEG intent: {}", e))? {
            
            let intent = Intent::from_ssz_bytes(&data)
                .map_err(|e| format!("Failed to deserialize TEG intent: {}", e.message))?;
            
            Ok(Some(intent))
        } else {
            Ok(None)
        }
    }
    
    /// Store a TEG constraint (expression data) with automatic domain-specific key generation
    pub fn store_teg_constraint(&mut self, domain_id: &DomainId, constraint_id: &ExprId, constraint_data: &[u8]) -> Result<String, String> {
        let key = domain_id.generate_teg_constraint_key(&constraint_id.0);
        
        self.smt.store_namespaced_data(&key, constraint_data)
            .map_err(|e| format!("Failed to store TEG constraint: {}", e))?;
        
        Ok(key)
    }
    
    /// Retrieve a TEG constraint by domain and constraint ID
    pub fn get_teg_constraint(&self, domain_id: &DomainId, constraint_id: &ExprId) -> Result<Option<Vec<u8>>, String> {
        let key = domain_id.generate_teg_constraint_key(&constraint_id.0);
        
        self.smt.get_namespaced_data(&key)
            .map_err(|e| format!("Failed to get TEG constraint: {}", e))
    }
    
    /// Store a generic TEG node with type-specific key generation
    pub fn store_teg_node(&mut self, domain_id: &DomainId, node_type: &str, node_id: &[u8], node_data: &[u8]) -> Result<String, String> {
        let key = domain_id.generate_teg_node_key(node_type, node_id);
        
        self.smt.store_namespaced_data(&key, node_data)
            .map_err(|e| format!("Failed to store TEG node: {}", e))?;
        
        Ok(key)
    }
    
    /// Retrieve a generic TEG node by domain, type, and node ID
    pub fn get_teg_node(&self, domain_id: &DomainId, node_type: &str, node_id: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let key = domain_id.generate_teg_node_key(node_type, node_id);
        
        self.smt.get_namespaced_data(&key)
            .map_err(|e| format!("Failed to get TEG node: {}", e))
    }
    
    /// Create a cross-domain TEG reference
    pub fn create_cross_domain_teg_reference(&mut self, 
        source_domain: &DomainId, 
        target_domain: &DomainId, 
        target_entity_id: &[u8], 
        reference_data: &[u8]
    ) -> Result<String, String> {
        let key = source_domain.generate_cross_domain_ref_key(target_domain, target_entity_id);
        
        self.smt.store_namespaced_data(&key, reference_data)
            .map_err(|e| format!("Failed to create cross-domain TEG reference: {}", e))?;
        
        Ok(key)
    }
    
    /// Retrieve a cross-domain TEG reference
    pub fn get_cross_domain_teg_reference(&self, 
        source_domain: &DomainId, 
        target_domain: &DomainId, 
        target_entity_id: &[u8]
    ) -> Result<Option<Vec<u8>>, String> {
        let key = source_domain.generate_cross_domain_ref_key(target_domain, target_entity_id);
        
        self.smt.get_namespaced_data(&key)
            .map_err(|e| format!("Failed to get cross-domain TEG reference: {}", e))
    }
    
    /// Store a temporal relationship between TEG entities
    pub fn store_temporal_relationship(&mut self, 
        domain_id: &DomainId, 
        from_entity: &[u8], 
        to_entity: &[u8], 
        relationship_type: &str, 
        relationship_data: &[u8]
    ) -> Result<String, String> {
        let key = format!("{}-temporal-{}-{}-{}", 
            domain_id.namespace_prefix(),
            hex::encode(from_entity),
            hex::encode(to_entity),
            relationship_type
        );
        
        self.smt.store_namespaced_data(&key, relationship_data)
            .map_err(|e| format!("Failed to store temporal relationship: {}", e))?;
        
        Ok(key)
    }
    
    /// Retrieve a temporal relationship between TEG entities
    pub fn get_temporal_relationship(&self, 
        domain_id: &DomainId, 
        from_entity: &[u8], 
        to_entity: &[u8], 
        relationship_type: &str
    ) -> Result<Option<Vec<u8>>, String> {
        let key = format!("{}-temporal-{}-{}-{}", 
            domain_id.namespace_prefix(),
            hex::encode(from_entity),
            hex::encode(to_entity),
            relationship_type
        );
        
        self.smt.get_namespaced_data(&key)
            .map_err(|e| format!("Failed to get temporal relationship: {}", e))
    }
    
    /// Batch store multiple TEG entities atomically within a domain
    pub fn batch_store_teg_entities(&mut self, 
        operations: Vec<(String, Vec<u8>)>
    ) -> Result<Vec<String>, String> {
        let mut stored_keys = Vec::new();
        
        for (key, data) in operations {
            self.smt.store_namespaced_data(&key, &data)
                .map_err(|e| format!("Failed to batch store TEG entity at key {}: {}", key, e))?;
            stored_keys.push(key);
        }
        
        Ok(stored_keys)
    }
    
    /// Store arbitrary data with custom namespaced key
    pub fn store_data(&mut self, namespaced_key: &str, data: &[u8]) -> Result<(), String> {
        self.smt.store_namespaced_data(namespaced_key, data)
    }
    
    /// Get arbitrary data with custom namespaced key
    pub fn get_data(&self, namespaced_key: &str) -> Result<Option<Vec<u8>>, String> {
        self.smt.get_namespaced_data(namespaced_key)
    }
    
    /// Check if namespaced key exists
    pub fn has_data(&self, namespaced_key: &str) -> bool {
        self.smt.has_namespaced_data(namespaced_key)
    }
    
    /// Get the underlying SMT for direct access when needed
    pub fn get_underlying_smt(&self) -> &SmtSha256<B> {
        &self.smt
    }
    
    /// Get mutable access to the underlying SMT for direct operations when needed
    pub fn get_underlying_smt_mut(&mut self) -> &mut SmtSha256<B> {
        &mut self.smt
    }
    
    /// Get the current state root for the entire SMT
    pub fn get_state_root(&self) -> Hash {
        self.smt.get_root()
    }
    
    /// Check if a domain has any TEG data stored
    pub fn domain_has_teg_data(&self, domain_id: &DomainId) -> bool {
        let prefix = domain_id.namespace_prefix();
        
        // Check for any keys with this domain prefix
        // This is a simplified implementation for development
        // In production, we'd want a more efficient approach
        let test_keys = [format!("{}-teg-effect-test", prefix),
            format!("{}-teg-handler-test", prefix),
            format!("{}-teg-resource-test", prefix),
            format!("{}-teg-intent-test", prefix)];
        
        test_keys.iter().any(|key| self.smt.has_namespaced_data(key))
    }

    /// Get all domain IDs that have data in the SMT
    pub fn get_all_domains(&self) -> Result<Vec<DomainId>, String> {
        // This is a simplified implementation for development
        // In production, we'd maintain a domain registry
        Ok(vec![])
    }

    /// Get all effects for a domain
    pub fn get_domain_effects(&self, _domain_id: &DomainId) -> Result<Vec<Effect>, String> {
        // TODO: Implement domain-specific effect retrieval
        Ok(Vec::new())
    }

    /// Get all resources for a domain
    pub fn get_domain_resources(&self, _domain_id: &DomainId) -> Result<Vec<Resource>, String> {
        // TODO: Implement domain-specific resource retrieval
        Ok(Vec::new())
    }

    /// Get all intents for a domain
    pub fn get_domain_intents(&self, _domain_id: &DomainId) -> Result<Vec<Intent>, String> {
        // TODO: Implement domain-specific intent retrieval
        Ok(Vec::new())
    }

    /// Get all handlers for a domain
    pub fn get_domain_handlers(&self, _domain_id: &DomainId) -> Result<Vec<Handler>, String> {
        // TODO: Implement domain-specific handler retrieval
        Ok(Vec::new())
    }
}

//-----------------------------------------------------------------------------
// Utility functions for converting SSZ serialized TEG data to domain-specific SMT keys
//-----------------------------------------------------------------------------

/// Convert SSZ serialized TEG effect data to a domain-specific SMT key
pub fn teg_effect_to_smt_key(domain_id: &DomainId, effect_data: &[u8]) -> String {
    let effect_hash = Sha256Hasher::hash(effect_data);
    domain_id.generate_teg_effect_key(&effect_hash)
}

/// Convert SSZ serialized TEG handler data to a domain-specific SMT key
pub fn teg_handler_to_smt_key(domain_id: &DomainId, handler_data: &[u8]) -> String {
    let handler_hash = Sha256Hasher::hash(handler_data);
    domain_id.generate_teg_handler_key(&handler_hash)
}

/// Convert SSZ serialized TEG resource data to a domain-specific SMT key
pub fn teg_resource_to_smt_key(domain_id: &DomainId, resource_data: &[u8]) -> String {
    let resource_hash = Sha256Hasher::hash(resource_data);
    domain_id.generate_teg_resource_key(&resource_hash)
}

/// Convert SSZ serialized TEG intent data to a domain-specific SMT key
pub fn teg_intent_to_smt_key(domain_id: &DomainId, intent_data: &[u8]) -> String {
    let intent_hash = Sha256Hasher::hash(intent_data);
    domain_id.generate_teg_intent_key(&intent_hash)
}

/// Convert SSZ serialized TEG constraint data to a domain-specific SMT key
pub fn teg_constraint_to_smt_key(domain_id: &DomainId, constraint_data: &[u8]) -> String {
    let constraint_hash = Sha256Hasher::hash(constraint_data);
    domain_id.generate_teg_constraint_key(&constraint_hash)
}

/// Generate content-addressable key for any TEG data by hashing its SSZ bytes
pub fn content_addressable_teg_key(domain_id: &DomainId, entity_type: &str, data: &[u8]) -> String {
    let content_hash = Sha256Hasher::hash(data);
    domain_id.generate_teg_node_key(entity_type, &content_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_consistency() {
        let test_data = b"test data";
        let hash1 = Sha256Hasher::hash(test_data);
        let hash2 = Sha256Hasher::hash(test_data);
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, empty_hash());
    }

    #[test]
    fn test_memory_backend() {
        let mut backend = MemoryBackend::new();
        let test_key = Sha256Hasher::hash(b"key");
        let test_data = b"value".to_vec();
        
        // Test set and get
        assert!(backend.set("test", &test_key, test_data.clone()).is_none());
        assert_eq!(backend.get("test", &test_key), Some(test_data.clone()));
        assert!(backend.has("test", &test_key));
        
        // Test remove
        assert_eq!(backend.remove("test", &test_key), Some(test_data));
        assert_eq!(backend.get("test", &test_key), None);
        assert!(!backend.has("test", &test_key));
    }

    #[test]
    fn test_smt_namespaced_storage() {
        let backend = MemoryBackend::new();
        let mut smt = SmtSha256::new(backend);
        
        // Test namespaced data storage
        let key = "domain-123-data-456";
        let data = b"test data".to_vec();
        
        smt.store_namespaced_data(key, &data).unwrap();
        assert_eq!(smt.get_namespaced_data(key).unwrap(), Some(data.clone()));
        assert!(smt.has_namespaced_data(key));
        
        // Test non-existent key
        assert_eq!(smt.get_namespaced_data("non-existent").unwrap(), None);
        assert!(!smt.has_namespaced_data("non-existent"));
    }

    #[test]
    fn test_teg_multi_domain_smt() {
        let backend = MemoryBackend::new();
        let mut teg_smt = TegMultiDomainSmt::new(backend);
        
        // Test basic data storage with custom keys
        let key = "test-key";
        let data = b"test data";
        
        teg_smt.store_data(key, data).unwrap();
        assert_eq!(teg_smt.get_data(key).unwrap(), Some(data.to_vec()));
        assert!(teg_smt.has_data(key));
        
        // Test batch operations
        let operations = vec![
            ("batch-key-1".to_string(), b"data1".to_vec()),
            ("batch-key-2".to_string(), b"data2".to_vec()),
        ];
        
        let stored_keys = teg_smt.batch_store_teg_entities(operations).unwrap();
        assert_eq!(stored_keys.len(), 2);
        
        // Verify batch stored data
        assert_eq!(teg_smt.get_data("batch-key-1").unwrap(), Some(b"data1".to_vec()));
        assert_eq!(teg_smt.get_data("batch-key-2").unwrap(), Some(b"data2".to_vec()));
    }

    #[test]
    fn test_domain_isolation() {
        let backend = MemoryBackend::new();
        let mut teg_smt = TegMultiDomainSmt::new(backend);
        
        // Create two different domains
        let domain1 = DomainId([1; 32]);
        let domain2 = DomainId([2; 32]);
        
        // Store same node type and ID in different domains
        let node_type = "effect";
        let node_id = [42; 4];
        let data1 = b"domain1 data";
        let data2 = b"domain2 data";
        
        let key1 = teg_smt.store_teg_node(&domain1, node_type, &node_id, data1).unwrap();
        let key2 = teg_smt.store_teg_node(&domain2, node_type, &node_id, data2).unwrap();
        
        // Keys should be different due to domain namespacing
        assert_ne!(key1, key2);
        
        // Data should be isolated by domain
        assert_eq!(teg_smt.get_teg_node(&domain1, node_type, &node_id).unwrap(), Some(data1.to_vec()));
        assert_eq!(teg_smt.get_teg_node(&domain2, node_type, &node_id).unwrap(), Some(data2.to_vec()));
    }

    #[test]
    fn test_cross_domain_references() {
        let backend = MemoryBackend::new();
        let mut teg_smt = TegMultiDomainSmt::new(backend);
        
        let source_domain = DomainId([10; 32]);
        let target_domain = DomainId([20; 32]);
        let target_entity_id = [99; 8];
        let reference_data = b"cross-domain reference";
        
        // Create cross-domain reference
        let ref_key = teg_smt.create_cross_domain_teg_reference(
            &source_domain,
            &target_domain,
            &target_entity_id,
            reference_data
        ).unwrap();
        
        // Retrieve cross-domain reference
        let retrieved_data = teg_smt.get_cross_domain_teg_reference(
            &source_domain,
            &target_domain,
            &target_entity_id
        ).unwrap();
        
        assert_eq!(retrieved_data, Some(reference_data.to_vec()));
        assert!(ref_key.contains("cross-ref"));
    }

    #[test]
    fn test_temporal_relationships() {
        let backend = MemoryBackend::new();
        let mut teg_smt = TegMultiDomainSmt::new(backend);
        
        let domain_id = DomainId([5; 32]);
        let from_entity = [1; 4];
        let to_entity = [2; 4];
        let relationship_type = "depends_on";
        let relationship_data = b"temporal dependency data";
        
        // Store temporal relationship
        let rel_key = teg_smt.store_temporal_relationship(
            &domain_id,
            &from_entity,
            &to_entity,
            relationship_type,
            relationship_data
        ).unwrap();
        
        // Retrieve temporal relationship
        let retrieved_data = teg_smt.get_temporal_relationship(
            &domain_id,
            &from_entity,
            &to_entity,
            relationship_type
        ).unwrap();
        
        assert_eq!(retrieved_data, Some(relationship_data.to_vec()));
        assert!(rel_key.contains("temporal"));
        assert!(rel_key.contains(relationship_type));
    }

    #[test]
    fn test_content_addressable_keys() {
        let domain_id = DomainId([100; 32]);
        let test_data = b"test data for content addressing";
        
        // Test different utility functions
        let effect_key = teg_effect_to_smt_key(&domain_id, test_data);
        let handler_key = teg_handler_to_smt_key(&domain_id, test_data);
        let resource_key = teg_resource_to_smt_key(&domain_id, test_data);
        let intent_key = teg_intent_to_smt_key(&domain_id, test_data);
        let constraint_key = teg_constraint_to_smt_key(&domain_id, test_data);
        let generic_key = content_addressable_teg_key(&domain_id, "custom", test_data);
        
        // All keys should be different due to different entity types
        assert_ne!(effect_key, handler_key);
        assert_ne!(handler_key, resource_key);
        assert_ne!(resource_key, intent_key);
        assert_ne!(intent_key, constraint_key);
        assert_ne!(constraint_key, generic_key);
        
        // All keys should contain the domain prefix
        let domain_prefix = domain_id.namespace_prefix();
        assert!(effect_key.contains(&domain_prefix));
        assert!(handler_key.contains(&domain_prefix));
        assert!(resource_key.contains(&domain_prefix));
        assert!(intent_key.contains(&domain_prefix));
        assert!(constraint_key.contains(&domain_prefix));
        assert!(generic_key.contains(&domain_prefix));
    }
} 