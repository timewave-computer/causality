// Resource content addressing
//
// This file defines types and functions for content addressing of resources.

use std::collections::HashMap;
use std::fmt::{self, Display};
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};
use sha2::{Sha256, Digest};

use crate::capability::ContentHash;
use super::types::{ResourceId, ResourceType};
use super::state::ResourceStateData;

/// Content addressable resource
///
/// A trait for resources that can be content-addressed.
pub trait ContentAddressable {
    /// Get the content hash of this resource
    fn content_hash(&self) -> ContentHash;
    
    /// Get a structured representation of the content for hashing
    fn content_data(&self) -> HashMap<String, serde_json::Value>;
    
    /// Calculate the content hash of this resource
    fn calculate_hash(&self) -> ContentHash {
        // Get the content data
        let content_data = self.content_data();
        
        // Serialize to JSON and hash
        let json = serde_json::to_string(&content_data)
            .expect("Failed to serialize content data");
            
        // Hash the JSON
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let hash = hasher.finalize();
        
        // Create a ContentHash from the hash
        ContentHash::from_bytes(&hash)
    }
    
    /// Verify that the content hash is correct
    fn verify_hash(&self, expected_hash: &ContentHash) -> bool {
        let actual_hash = self.calculate_hash();
        &actual_hash == expected_hash
    }
}

/// Basic content-addressable resource implementation
///
/// A simple implementation of a content-addressable resource.
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct BasicResource {
    /// Resource ID
    pub id: ResourceId,
    
    /// Resource type
    pub resource_type: ResourceType,
    
    /// Resource data
    pub data: HashMap<String, serde_json::Value>,
    
    /// Resource metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Content hash cache (recalculated when needed)
    #[serde(skip)]
    cache: Option<ContentHash>,
}

impl BasicResource {
    /// Create a new basic resource
    pub fn new(resource_type: ResourceType) -> Self {
        // Create a resource with empty data
        let mut resource = Self {
            id: ResourceId::new(ContentHash::default()),
            resource_type,
            data: HashMap::new(),
            metadata: HashMap::new(),
            cache: None,
        };
        
        // Calculate the initial hash and update the ID
        let hash = resource.calculate_hash();
        resource.id = ResourceId::new(hash);
        resource.cache = Some(hash);
        
        resource
    }
    
    /// Get a data value
    pub fn get_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }
    
    /// Set a data value
    pub fn set_data(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.data.insert(key.into(), value.into());
        
        // Invalidate the hash cache
        self.cache = None;
    }
    
    /// Get a metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
    
    /// Set a metadata value
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.metadata.insert(key.into(), value.into());
        
        // Metadata doesn't affect the content hash, so no need to invalidate the cache
    }
    
    /// Update the resource ID with the current content hash
    pub fn update_id(&mut self) {
        let hash = self.calculate_hash();
        self.id = ResourceId::new(hash);
        self.cache = Some(hash);
    }
    
    /// Convert this resource to a state data object
    pub fn to_state_data(&self, created_at: u64) -> ResourceStateData {
        ResourceStateData::new(
            self.id.clone(),
            self.resource_type.clone(),
            created_at,
        )
    }
}

impl ContentAddressable for BasicResource {
    fn content_hash(&self) -> ContentHash {
        // Return cached hash if available
        if let Some(hash) = &self.cache {
            return hash.clone();
        }
        
        // Otherwise calculate the hash
        self.calculate_hash()
    }
    
    fn content_data(&self) -> HashMap<String, serde_json::Value> {
        // Include resource type and data in the content hash
        let mut content_data = HashMap::new();
        
        // Add resource type
        content_data.insert(
            "type".to_string(),
            serde_json::to_value(&self.resource_type).unwrap(),
        );
        
        // Add resource data
        content_data.insert(
            "data".to_string(),
            serde_json::to_value(&self.data).unwrap(),
        );
        
        content_data
    }
}

impl Display for BasicResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Resource[{}: {}]", self.resource_type, self.id)
    }
}

/// Resource content verification result
#[derive(Debug, Clone)]
pub enum ContentVerificationResult {
    /// Content is valid
    Valid,
    
    /// Content hash doesn't match
    HashMismatch {
        expected: ContentHash,
        actual: ContentHash,
    },
    
    /// Resource is invalid
    Invalid(String),
}

impl ContentVerificationResult {
    /// Check if the result is valid
    pub fn is_valid(&self) -> bool {
        matches!(self, ContentVerificationResult::Valid)
    }
    
    /// Get the error message, if any
    pub fn error_message(&self) -> Option<String> {
        match self {
            ContentVerificationResult::Valid => None,
            ContentVerificationResult::HashMismatch { expected, actual } => {
                Some(format!(
                    "Hash mismatch: expected {}, got {}",
                    expected, actual
                ))
            }
            ContentVerificationResult::Invalid(message) => Some(message.clone()),
        }
    }
}

/// Verify that a resource's content hash matches its ID
pub fn verify_resource_content<T: ContentAddressable>(
    resource: &T,
    expected_id: &ResourceId,
) -> ContentVerificationResult {
    let actual_hash = resource.content_hash();
    let expected_hash = expected_id.hash();
    
    if &actual_hash != expected_hash {
        ContentVerificationResult::HashMismatch {
            expected: expected_hash.clone(),
            actual: actual_hash,
        }
    } else {
        ContentVerificationResult::Valid
    }
} 