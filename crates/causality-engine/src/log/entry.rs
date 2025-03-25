// Log entry system
// Original file: src/log/entry.rs

// Log entry types for Causality Unified Log System
//
// This module defines the various entry types that can be recorded in
// the unified log.

mod effect_entry;
mod fact_entry;
mod event_entry;

pub use effect_entry::EffectEntry;
pub use fact_entry::FactEntry;
pub use event_entry::EventEntry;
pub use event_entry::EventSeverity;

use std::fmt;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use blake3::Hasher;
use causality_crypto::{ContentAddressed, ContentId, HashOutput, HashFactory};
use borsh::{BorshSerialize, BorshDeserialize};

use causality_types::content::ContentHash;
use causality_types::{*};
use causality_crypto::ContentId;
use causality_types::{Error, Result};

/// The type of log entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum EntryType {
    /// An effect operation
    Effect,
    /// An observed fact
    Fact,
    /// A system event
    Event,
}

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntryType::Effect => write!(f, "Effect"),
            EntryType::Fact => write!(f, "Fact"),
            EntryType::Event => write!(f, "Event"),
        }
    }
}

/// A unified log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// The unique ID of this entry
    pub id: String,
    /// The timestamp when this entry was created
    pub timestamp: DateTime<Utc>,
    /// The type of entry
    pub entry_type: EntryType,
    /// The entry data
    pub data: EntryData,
    /// The trace ID for grouping related entries
    pub trace_id: Option<String>,
    /// The parent entry ID, if any
    pub parent_id: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Content hash of this entry for verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_hash: Option<ContentHash>,
}

/// The data specific to each entry type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryData {
    /// An effect entry
    Effect(EffectEntry),
    /// A fact entry
    Fact(FactEntry),
    /// An event entry
    Event(EventEntry),
}

/// Event ID content for deterministic IDs
#[derive(BorshSerialize, BorshDeserialize)]
struct EventIdContent {
    resource_id: ContentId,
    domain_id: DomainId,
    event_name: String,
    component: String,
    timestamp: i64,
}

impl ContentAddressed for EventIdContent {
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
    
    fn from_bytes(bytes: &[u8]) -> std::result::Result<Self, causality_crypto::HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| causality_crypto::HashError::SerializationError(e.to_string()))
    }
}

/// Fact ID content for deterministic IDs
#[derive(BorshSerialize, BorshDeserialize)]
struct FactIdContent {
    resource_id: ContentId,
    domain_id: DomainId,
    fact_type: String,
    description: String,
    timestamp: i64,
}

impl ContentAddressed for FactIdContent {
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
    
    fn from_bytes(bytes: &[u8]) -> std::result::Result<Self, causality_crypto::HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| causality_crypto::HashError::SerializationError(e.to_string()))
    }
}

/// Effect ID content for deterministic IDs
#[derive(BorshSerialize, BorshDeserialize)]
struct EffectIdContent {
    resource_id: ContentId,
    domain_id: DomainId,
    effect_type: String,
    description: String,
    timestamp: i64,
}

impl ContentAddressed for EffectIdContent {
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
    
    fn from_bytes(bytes: &[u8]) -> std::result::Result<Self, causality_crypto::HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| causality_crypto::HashError::SerializationError(e.to_string()))
    }
}

impl LogEntry {
    /// Get the entry ID
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Get the entry timestamp
    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }
    
    /// Get the entry type
    pub fn entry_type(&self) -> &EntryType {
        &self.entry_type
    }
    
    /// Get the trace ID, if any
    pub fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }
    
    /// Get the parent entry ID, if any
    pub fn parent_id(&self) -> Option<&str> {
        self.parent_id.as_deref()
    }
    
    /// Add a metadata field to this entry
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Get a metadata field from this entry
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Get the effect entry, if this is an effect entry
    pub fn effect(&self) -> Option<&EffectEntry> {
        match &self.data {
            EntryData::Effect(effect) => Some(effect),
            _ => None,
        }
    }
    
    /// Get the fact entry, if this is a fact entry
    pub fn fact(&self) -> Option<&FactEntry> {
        match &self.data {
            EntryData::Fact(fact) => Some(fact),
            _ => None,
        }
    }
    
    /// Get the event entry, if this is an event entry
    pub fn event(&self) -> Option<&EventEntry> {
        match &self.data {
            EntryData::Event(event) => Some(event),
            _ => None,
        }
    }
    
    /// Add metadata from a serializable object
    pub fn with_metadata_object<T: Serialize>(&mut self, metadata: &T) -> Result<(), bincode::Error> {
        let data = bincode::serialize(metadata)?;
        self.metadata.insert("binary_data".to_string(), hex::encode(data));
        Ok(())
    }

    /// Generate a content hash for this entry based on its contents
    pub fn generate_hash(&mut self) -> ContentHash {
        // Remove any existing hash first to avoid including it in the hash calculation
        self.entry_hash = None;
        
        // Serialize without the hash field
        let serialized = serde_json::to_vec(self).expect("Failed to serialize log entry");
        
        // Generate the hash
        let hash = ContentHash::from_data(&serialized);
        
        // Store the hash and return it
        self.entry_hash = Some(hash.clone());
        hash
    }
    
    /// Create a new log entry with a content hash
    pub fn new_with_hash(
        id: String,
        timestamp: DateTime<Utc>,
        entry_type: EntryType,
        data: EntryData,
        trace_id: Option<String>,
        parent_id: Option<String>,
    ) -> Self {
        let mut entry = Self {
            id,
            timestamp,
            entry_type,
            data,
            trace_id,
            parent_id,
            metadata: HashMap::new(),
            entry_hash: None,
        };
        
        // Generate and set the hash
        entry.generate_hash();
        entry
    }
    
    /// Verify that the entry hash is correct
    pub fn verify_hash(&self) -> bool {
        if let Some(stored_hash) = &self.entry_hash {
            // Create a temporary copy without the hash
            let mut temp = self.clone();
            temp.entry_hash = None;
            
            // Serialize without the hash field
            let serialized = match serde_json::to_vec(&temp) {
                Ok(s) => s,
                Err(_) => return false,
            };
            
            // Generate a hash from the serialized data
            let computed_hash = ContentHash::from_data(&serialized);
            
            // Compare the stored hash with the computed hash
            computed_hash == *stored_hash
        } else {
            false
        }
    }
    
    /// Create a new event entry
    pub fn new_event(
        resource_id: &ContentId,
        domain_id: &DomainId,
        event_name: &str,
        severity: event_entry::EventSeverity,
        component: &str,
        details: Option<serde_json::Value>,
    ) -> Self {
        // Create content for ID generation
        let id_content = EventIdContent {
            resource_id: resource_id.clone(),
            domain_id: domain_id.clone(),
            event_name: event_name.to_string(),
            component: component.to_string(),
            timestamp: Utc::now().timestamp(),
        };
        
        // Generate content-derived ID
        let content_id = id_content.content_id();
        let id = format!("event-{}", content_id);
        
        Self::new_with_hash(
            id,
            Utc::now(),
            EntryType::Event,
            EntryData::Event(EventEntry {
                event_name: event_name.to_string(),
                severity,
                component: component.to_string(),
                details: details.unwrap_or(serde_json::json!({})),
                resources: Some(vec![resource_id.clone()]),
                domains: Some(vec![domain_id.clone()]),
            }),
            None,
            None,
        )
    }
    
    /// Create a new fact entry
    pub fn new_fact(
        resource_id: &ContentId,
        domain_id: &DomainId,
        fact_type: &str,
        description: &str,
        data: Option<serde_json::Value>,
    ) -> Self {
        // Create content for ID generation
        let id_content = FactIdContent {
            resource_id: resource_id.clone(),
            domain_id: domain_id.clone(),
            fact_type: fact_type.to_string(),
            description: description.to_string(),
            timestamp: Utc::now().timestamp(),
        };
        
        // Generate content-derived ID
        let content_id = id_content.content_id();
        let id = format!("fact-{}", content_id);
        
        Self::new_with_hash(
            id,
            Utc::now(),
            EntryType::Fact,
            EntryData::Fact(FactEntry {
                domain: domain_id.clone(),
                block_height: 0,
                block_hash: None,
                observed_at: chrono::Utc::now().timestamp(),
                fact_type: fact_type.to_string(),
                resources: vec![resource_id.clone()],
                data: data.unwrap_or(serde_json::json!({"description": description})),
                verified: true,
            }),
            None,
            None,
        )
    }
    
    /// Create a new effect entry
    pub fn new_effect(
        resource_id: &ContentId,
        domain_id: &DomainId,
        effect_type_str: &str,
        description: &str,
        params: Option<HashMap<String, serde_json::Value>>,
    ) -> Self {
        let mut parameters = params.unwrap_or_default();
        parameters.insert("description".to_string(), serde_json::json!(description));
        
        // Create content for ID generation
        let id_content = EffectIdContent {
            resource_id: resource_id.clone(),
            domain_id: domain_id.clone(),
            effect_type: effect_type_str.to_string(),
            description: description.to_string(),
            timestamp: Utc::now().timestamp(),
        };
        
        // Generate content-derived ID
        let content_id = id_content.content_id();
        let id = format!("effect-{}", content_id);
        
        Self::new_with_hash(
            id,
            Utc::now(),
            EntryType::Effect,
            EntryData::Effect(EffectEntry {
                effect_type: match effect_type_str {
                    "create" => crate::effect::EffectType::Create,
                    "update" => crate::effect::EffectType::Update,
                    "delete" => crate::effect::EffectType::Delete,
                    _ => crate::effect::EffectType::Custom(effect_type_str.to_string()),
                },
                resources: vec![resource_id.clone()],
                domains: vec![domain_id.clone()],
                code_hash: None,
                parameters,
                result: None,
                success: true,
                error: None,
            }),
            None,
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::{*};
    use serde_json::json;
    
    #[test]
    fn test_entry_hash_generation() {
        // Create a simple test entry
        let mut entry = LogEntry {
            id: "test-id".to_string(),
            timestamp: Utc::now(),
            entry_type: EntryType::Event,
            data: EntryData::Event(EventEntry {
                event_name: "test-event".to_string(),
                severity: event_entry::EventSeverity::Info,
                component: "test".to_string(),
                details: json!({"test": "value"}),
                resources: None,
                domains: None,
            }),
            trace_id: Some("trace-1".to_string()),
            parent_id: None,
            metadata: HashMap::new(),
            entry_hash: None,
        };
        
        // Generate a hash
        let hash = entry.generate_hash();
        
        // Verify the hash is set and valid
        assert!(entry.entry_hash.is_some());
        assert_eq!(entry.entry_hash.as_ref().unwrap(), &hash);
        assert!(entry.verify_hash());
        
        // Modify the entry
        entry.metadata.insert("test".to_string(), "value".to_string());
        
        // Hash should now be invalid
        assert!(!entry.verify_hash());
        
        // Regenerate the hash
        let new_hash = entry.generate_hash();
        
        // Hash should now be valid again, but different from before
        assert!(entry.verify_hash());
        assert_ne!(hash, new_hash);
    }
    
    #[test]
    fn test_entry_factory_methods() {
        let resource_id = ContentId::new("test-resource");
        let domain_id = DomainId::new("test-domain");
        
        let event_entry = LogEntry::new_event(
            &resource_id,
            &domain_id,
            "test-event",
            event_entry::EventSeverity::Info,
            "test",
            Some(json!({"test": "value"})),
        );
        
        // Verify the entry has a valid hash
        assert!(event_entry.entry_hash.is_some());
        assert!(event_entry.verify_hash());
        
        let fact_entry = LogEntry::new_fact(
            &resource_id,
            &domain_id,
            "test-fact",
            "test-description",
            Some(json!({"test": "value"})),
        );
        
        // Verify the entry has a valid hash
        assert!(fact_entry.entry_hash.is_some());
        assert!(fact_entry.verify_hash());
        
        let mut parameters = HashMap::new();
        parameters.insert("test".to_string(), json!("value"));
        
        let effect_entry = LogEntry::new_effect(
            &resource_id,
            &domain_id,
            "create",
            "test-description",
            Some(parameters),
        );
        
        // Verify the entry has a valid hash
        assert!(effect_entry.entry_hash.is_some());
        assert!(effect_entry.verify_hash());
    }
} 
