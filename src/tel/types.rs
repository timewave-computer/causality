// Core type definitions for TEL
//
// This module provides the core type definitions
// used throughout the Temporal Effect Language (TEL).

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Identifier for a resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub Uuid);

impl ResourceId {
    /// Create a new random resource ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ResourceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Address of an actor in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(pub String);

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Address {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

/// Domain identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Domain(pub String);

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Domain {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

/// Metadata key-value pairs for resources and relationships
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Metadata {
    // Internal map of string keys to string values
    values: HashMap<String, String>,
}

impl Metadata {
    /// Create a new empty metadata container
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
    
    /// Add a key-value pair to the metadata
    pub fn insert(&mut self, key: String, value: String) -> Option<String> {
        self.values.insert(key, value)
    }
    
    /// Get a value from the metadata by key
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
    
    /// Remove a key-value pair from the metadata
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.values.remove(key)
    }
    
    /// Check if the metadata contains a specific key
    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
    
    /// Get the number of key-value pairs in the metadata
    pub fn len(&self) -> usize {
        self.values.len()
    }
    
    /// Check if the metadata is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    
    /// Get an iterator over the key-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.values.iter()
    }
}

/// Identifier for an operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId(pub Uuid);

impl OperationId {
    /// Create a new random operation ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for OperationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Proof for an operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proof {
    /// Type of proof
    pub proof_type: String,
    /// Proof data
    pub data: Vec<u8>,
    /// Verification key
    pub verification_key: Option<Vec<u8>>,
}

/// Parameters for an operation
pub type Parameters = HashMap<String, serde_json::Value>;

/// Time point in milliseconds since UNIX epoch
pub type Timestamp = u64;

/// Type of effect in the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectType {
    /// State transition
    StateTransition,
    /// Resource transfer
    ResourceTransfer,
    /// Computation
    Computation,
    /// Data operation
    DataOperation,
    /// Communication
    Communication,
    /// Access control
    AccessControl,
    /// Custom effect type
    Custom(String),
}

/// Effect identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectId(pub String);

impl EffectId {
    /// Create a new random effect ID
    pub fn new() -> Self {
        Self(format!("effect-{}", Uuid::new_v4()))
    }
}

impl Default for EffectId {
    fn default() -> Self {
        Self::new()
    }
}

/// Effect status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectStatus {
    /// Effect is pending
    Pending,
    /// Effect is being processed
    Processing,
    /// Effect has completed successfully
    Completed,
    /// Effect has failed
    Failed,
    /// Effect has been cancelled
    Cancelled,
}

/// Result of an effect
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EffectResult {
    /// No result
    None,
    /// Boolean result
    Boolean(bool),
    /// Integer result
    Integer(i64),
    /// Float result
    Float(f64),
    /// String result
    String(String),
    /// Binary result
    Binary(Vec<u8>),
    /// JSON result
    Json(serde_json::Value),
    /// Error result
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metadata_basic_operations() {
        let mut metadata = Metadata::new();
        
        // Insert some key-value pairs
        metadata.insert("name".to_string(), "Token X".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());
        
        // Check that keys exist
        assert!(metadata.contains_key("name"));
        assert!(metadata.contains_key("version"));
        assert!(!metadata.contains_key("missing"));
        
        // Check values
        assert_eq!(metadata.get("name"), Some(&"Token X".to_string()));
        assert_eq!(metadata.get("version"), Some(&"1.0".to_string()));
        assert_eq!(metadata.get("missing"), None);
        
        // Check length
        assert_eq!(metadata.len(), 2);
        assert!(!metadata.is_empty());
        
        // Remove a key
        let removed = metadata.remove("name");
        assert_eq!(removed, Some("Token X".to_string()));
        
        // Check updated state
        assert!(!metadata.contains_key("name"));
        assert_eq!(metadata.len(), 1);
    }
    
    #[test]
    fn test_metadata_iteration() {
        let mut metadata = Metadata::new();
        
        metadata.insert("key1".to_string(), "value1".to_string());
        metadata.insert("key2".to_string(), "value2".to_string());
        
        // Collect the keys and values into vectors for testing
        let mut keys: Vec<String> = metadata.iter()
            .map(|(k, _)| k.clone())
            .collect();
        let mut values: Vec<String> = metadata.iter()
            .map(|(_, v)| v.clone())
            .collect();
        
        // Sort for deterministic comparison
        keys.sort();
        values.sort();
        
        assert_eq!(keys, vec!["key1".to_string(), "key2".to_string()]);
        assert_eq!(values, vec!["value1".to_string(), "value2".to_string()]);
    }
} 