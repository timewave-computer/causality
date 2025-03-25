// Content-addressed time map for domains
// Original file: src/domain/map/content_addressed_time_map.rs

// Content-addressed Time Map Implementation
//
// This module implements a content-addressed version of the time map
// for tracking the observed state of domains over time.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};

use causality_crypto::{ContentAddressed, ContentId, HashOutput, HashError, HashFactory};
use causality_types::{DomainId, BlockHeight, BlockHash, Timestamp};
use super::TimeRange;

/// A content-addressed time map entry for a domain
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ContentAddressedTimeMapEntry {
    /// Domain identifier
    pub domain_id: DomainId,
    /// Block height
    pub height: BlockHeight,
    /// Block hash
    pub hash: BlockHash,
    /// Timestamp
    pub timestamp: Timestamp,
    /// When this entry was observed
    pub observed_at: DateTime<Utc>,
    /// Confidence in this entry (0.0-1.0)
    pub confidence: f64,
    /// Whether this entry is verified
    pub verified: bool,
    /// Source of this entry (e.g., "rpc", "peers", "cache")
    pub source: String,
    /// Additional metadata about this entry
    pub metadata: HashMap<String, String>,
}

impl ContentAddressedTimeMapEntry {
    /// Create a new time map entry
    pub fn new(
        domain_id: DomainId,
        height: BlockHeight,
        hash: BlockHash,
        timestamp: Timestamp,
        source: impl Into<String>,
    ) -> Self {
        Self {
            domain_id,
            height,
            hash,
            timestamp,
            observed_at: Utc::now(),
            confidence: 1.0,
            verified: false,
            source: source.into(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set the confidence level
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.max(0.0).min(1.0);
        self
    }
    
    /// Set the verification status
    pub fn with_verification(mut self, verified: bool) -> Self {
        self.verified = verified;
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl ContentAddressed for ContentAddressedTimeMapEntry {
    fn content_hash(&self) -> HashOutput {
        let hasher = HashFactory::default().create_hasher().unwrap();
        let data = self.try_to_vec().unwrap();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hasher = HashFactory::default().create_hasher().unwrap();
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

/// A content-addressed time map
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ContentAddressedTimeMap {
    /// Map from domain ID to time map entry content ID
    pub entries: HashMap<DomainId, ContentId>,
    /// When this time map was created
    pub created_at: DateTime<Utc>,
    /// Version of this time map (for tracking updates)
    pub version: u64,
    /// Additional metadata about this time map
    pub metadata: HashMap<String, String>,
}

impl ContentAddressedTimeMap {
    /// Create a new empty time map
    pub fn new() -> Self {
        ContentAddressedTimeMap {
            entries: HashMap::new(),
            created_at: Utc::now(),
            version: 1,
            metadata: HashMap::new(),
        }
    }
    
    /// Update or insert a domain entry
    pub fn update_domain(&mut self, entry: &ContentAddressedTimeMapEntry) {
        let content_id = entry.content_id();
        self.entries.insert(entry.domain_id.clone(), content_id);
        self.version += 1;
    }
    
    /// Remove a domain entry
    pub fn remove_domain(&mut self, domain_id: &DomainId) -> bool {
        let removed = self.entries.remove(domain_id).is_some();
        if removed {
            self.version += 1;
        }
        removed
    }
    
    /// Check if the time map contains a domain
    pub fn contains_domain(&self, domain_id: &DomainId) -> bool {
        self.entries.contains_key(domain_id)
    }
    
    /// Get all domains in the time map
    pub fn domains(&self) -> Vec<&DomainId> {
        self.entries.keys().collect()
    }
    
    /// Get the number of domains in the time map
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Check if the time map is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl ContentAddressed for ContentAddressedTimeMap {
    fn content_hash(&self) -> HashOutput {
        let hasher = HashFactory::default().create_hasher().unwrap();
        let data = self.try_to_vec().unwrap();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let hash = self.content_hash();
        let serialized = self.to_bytes();
        
        let hasher = HashFactory::default().create_hasher().unwrap();
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

/// Thread-safe content-addressed time map
pub struct SharedContentAddressedTimeMap {
    /// The underlying time map
    time_map: RwLock<ContentAddressedTimeMap>,
    /// Storage for entries
    entry_storage: Arc<dyn crate::content_addressed_storage::ContentAddressedStorage>,
}

impl SharedContentAddressedTimeMap {
    /// Create a new shared time map
    pub fn new(storage: Arc<dyn crate::content_addressed_storage::ContentAddressedStorage>) -> Self {
        Self {
            time_map: RwLock::new(ContentAddressedTimeMap::new()),
            entry_storage: storage,
        }
    }
    
    /// Update a domain entry
    pub fn update_domain(&self, entry: ContentAddressedTimeMapEntry) -> Result<ContentId, HashError> {
        // Store the entry
        let content_id = self.entry_storage.store(&entry)
            .map_err(|e| HashError::Other(e.to_string()))?;
        
        // Update the time map
        let mut time_map = self.time_map.write()
            .map_err(|_| HashError::Other("Failed to acquire time map write lock".to_string()))?;
        
        time_map.update_domain(&entry);
        
        Ok(content_id)
    }
    
    /// Retrieve a domain entry
    pub fn get_entry(&self, domain_id: &DomainId) -> Result<ContentAddressedTimeMapEntry, HashError> {
        let time_map = self.time_map.read()
            .map_err(|_| HashError::Other("Failed to acquire time map read lock".to_string()))?;
        
        let content_id = time_map.entries.get(domain_id)
            .ok_or_else(|| HashError::Other(format!("Domain not found: {:?}", domain_id)))?;
        
        self.entry_storage.get(content_id)
            .map_err(|e| HashError::Other(e.to_string()))
    }
    
    /// Get all entries
    pub fn get_all_entries(&self) -> Result<Vec<ContentAddressedTimeMapEntry>, HashError> {
        let time_map = self.time_map.read()
            .map_err(|_| HashError::Other("Failed to acquire time map read lock".to_string()))?;
        
        let mut entries = Vec::with_capacity(time_map.entries.len());
        
        for content_id in time_map.entries.values() {
            let entry = self.entry_storage.get(content_id)
                .map_err(|e| HashError::Other(e.to_string()))?;
            
            entries.push(entry);
        }
        
        Ok(entries)
    }
    
    /// Query entries within a specific time range
    pub fn query_by_time(&self, range: &TimeRange) -> Result<Vec<ContentAddressedTimeMapEntry>, HashError> {
        let entries = self.get_all_entries()?;
        
        Ok(entries.into_iter()
            .filter(|entry| range.contains(entry.timestamp))
            .collect())
    }
    
    /// Get the content ID of the current time map
    pub fn content_id(&self) -> Result<ContentId, HashError> {
        let time_map = self.time_map.read()
            .map_err(|_| HashError::Other("Failed to acquire time map read lock".to_string()))?;
        
        Ok(time_map.content_id())
    }
    
    /// Store the current time map
    pub fn store(&self) -> Result<ContentId, HashError> {
        let time_map = self.time_map.read()
            .map_err(|_| HashError::Other("Failed to acquire time map read lock".to_string()))?;
        
        self.entry_storage.store(&*time_map)
            .map_err(|e| HashError::Other(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content_addressed_storage::{StorageFactory, InMemoryStorage};
    
    #[test]
    fn test_content_addressed_time_map_entry() {
        // Create a time map entry
        let entry = ContentAddressedTimeMapEntry::new(
            "domain1".to_string(),
            42,
            "0x1234abcd".to_string(),
            Timestamp(1625097600),
            "test",
        )
        .with_confidence(0.9)
        .with_verification(true)
        .with_metadata("extra_info", "some_value");
        
        // Test content addressing
        let hash = entry.content_hash();
        assert!(entry.verify());
        
        // Test serialization roundtrip
        let bytes = entry.to_bytes();
        let deserialized = ContentAddressedTimeMapEntry::from_bytes(&bytes).unwrap();
        
        assert_eq!(deserialized.domain_id, entry.domain_id);
        assert_eq!(deserialized.height, entry.height);
        assert_eq!(deserialized.hash, entry.hash);
        assert_eq!(deserialized.timestamp, entry.timestamp);
        assert_eq!(deserialized.verified, entry.verified);
        assert_eq!(deserialized.confidence, entry.confidence);
        assert_eq!(deserialized.metadata.get("extra_info"), Some(&"some_value".to_string()));
    }
    
    #[test]
    fn test_content_addressed_time_map() {
        // Create a storage
        let storage = StorageFactory::create_memory_storage();
        let map = SharedContentAddressedTimeMap::new(storage);
        
        // Create some entries
        let entry1 = ContentAddressedTimeMapEntry::new(
            "domain1".to_string(),
            10,
            "0xabc1".to_string(),
            Timestamp(1625097600),
            "test",
        );
        
        let entry2 = ContentAddressedTimeMapEntry::new(
            "domain2".to_string(),
            20,
            "0xabc2".to_string(),
            Timestamp(1625184000),
            "test",
        );
        
        // Store entries
        let id1 = map.update_domain(entry1.clone()).unwrap();
        let id2 = map.update_domain(entry2.clone()).unwrap();
        
        // Retrieve entries
        let retrieved1 = map.get_entry(&"domain1".to_string()).unwrap();
        let retrieved2 = map.get_entry(&"domain2".to_string()).unwrap();
        
        assert_eq!(retrieved1.height, 10);
        assert_eq!(retrieved2.height, 20);
        
        // Query by time range
        let range = TimeRange::new(
            Timestamp(1625097500),
            Timestamp(1625097700),
        );
        
        let results = map.query_by_time(&range).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].domain_id, "domain1");
        
        // Test time map storage
        let map_id = map.store().unwrap();
        assert!(!map_id.to_string().is_empty());
    }
} 