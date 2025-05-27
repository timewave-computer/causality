// Operation definitions for the Temporal Effect Graph
// This file defines operations that can be executed as part of the TEG.

use std::collections::HashMap;
use causality_types::{ContentId, ContentAddressed, HashOutput, HashError};
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};

use crate::{ResourceId};
use super::edge::EdgeId;

/// Type alias for operation ID
pub type OperationId = ContentId;

/// Unique identifier for an agent in the system
pub type AgentId = String;

/// Types of operations that can be performed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum OperationType {
    /// Create a new resource
    Create,
    /// Read a resource
    Read,
    /// Update an existing resource
    Update,
    /// Delete a resource
    Delete,
    /// Custom operation type
    Custom(String),
}

/// Represents an operation in the Temporal Effect Graph
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Operation {
    /// Unique identifier for this operation
    pub id: OperationId,
    
    /// The agent performing the operation (optional for system operations)
    pub agent_id: Option<String>,
    
    /// The resource being operated on
    pub target_resource_id: ResourceId,
    
    /// The type of operation being performed
    pub operation_type: OperationType,
    
    /// Parameters for this operation (optional)
    pub parameters: Option<HashMap<String, String>>,
    
    /// Additional metadata about this operation (optional)
    pub metadata: Option<HashMap<String, String>>,
}

impl Operation {
    /// Create a new operation
    pub fn new(
        target_resource_id: ResourceId,
        operation_type: OperationType,
        agent_id: Option<String>,
    ) -> Self {
        let bytes = format!("{}:{:?}:{:?}", 
            target_resource_id, 
            operation_type,
            agent_id
        ).into_bytes();
        
        let hash_output = causality_types::content_addressing::content_hash_from_bytes(&bytes);
        
        Self {
            id: ContentId::from(hash_output),
            agent_id,
            target_resource_id,
            operation_type,
            parameters: None,
            metadata: None,
        }
    }
    
    /// Add a parameter to this operation
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let params = self.parameters.get_or_insert_with(|| HashMap::new());
        params.insert(key.into(), value.into());
        self
    }
    
    /// Add metadata to this operation
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let meta = self.metadata.get_or_insert_with(|| HashMap::new());
        meta.insert(key.into(), value.into());
        self
    }
    
    /// Check if this operation has an agent
    pub fn has_agent(&self) -> bool {
        self.agent_id.is_some()
    }
    
    /// Get the agent ID if present
    pub fn agent_id(&self) -> Option<&String> {
        self.agent_id.as_ref()
    }
    
    /// Get parameters if present
    pub fn parameters(&self) -> Option<&HashMap<String, String>> {
        self.parameters.as_ref()
    }
    
    /// Get metadata if present
    pub fn metadata(&self) -> Option<&HashMap<String, String>> {
        self.metadata.as_ref()
    }
}

impl ContentAddressed for Operation {
    fn content_hash(&self) -> Result<HashOutput, HashError> {
        // Simple implementation - in a real system, we'd use a proper hash function
        let mut hasher = blake3::Hasher::new();
        if let Some(agent) = &self.agent_id {
            hasher.update(agent.as_bytes());
        }
        hasher.update(self.target_resource_id.as_bytes());
        hasher.update(&[self.operation_type.to_byte()]);
        
        if let Some(params) = &self.parameters {
            for (key, value) in params {
                hasher.update(key.as_bytes());
                hasher.update(value.as_bytes());
            }
        }
        
        if let Some(meta) = &self.metadata {
            for (key, value) in meta {
                hasher.update(key.as_bytes());
                hasher.update(value.as_bytes());
            }
        }
        
        let hash = hasher.finalize();
        let mut output = [0u8; 32];
        output.copy_from_slice(hash.as_bytes());
        
        Ok(HashOutput::new(output, causality_types::HashAlgorithm::Blake3))
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>, HashError> {
        borsh::to_vec(self).map_err(|e| HashError::SerializationError(e.to_string()))
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        borsh::from_slice(bytes).map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

impl OperationType {
    /// Convert the operation type to a byte for hashing
    fn to_byte(&self) -> u8 {
        match self {
            OperationType::Create => 1,
            OperationType::Read => 2,
            OperationType::Update => 3,
            OperationType::Delete => 4,
            OperationType::Custom(_) => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_operation_creation() {
        let op = Operation::new(
            "resource:123".into(),
            OperationType::Create,
            Some("agent:456".into()),
        );
        
        assert_eq!(op.target_resource_id, "resource:123");
        assert_eq!(op.operation_type, OperationType::Create);
        assert_eq!(op.agent_id.as_ref().unwrap(), "agent:456");
        assert!(op.parameters.is_none());
        assert!(op.metadata.is_none());
    }
    
    #[test]
    fn test_operation_with_parameters() {
        let op = Operation::new(
            "resource:123".into(),
            OperationType::Update,
            Some("agent:456".into()),
        )
        .with_parameter("key1", "value1")
        .with_parameter("key2", "value2");
        
        let params = op.parameters.as_ref().unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(params.get("key1").unwrap(), "value1");
        assert_eq!(params.get("key2").unwrap(), "value2");
    }
    
    #[test]
    fn test_operation_with_metadata() {
        let op = Operation::new(
            "resource:123".into(),
            OperationType::Delete,
            None,
        )
        .with_metadata("origin", "system")
        .with_metadata("timestamp", "123456789");
        
        let meta = op.metadata.as_ref().unwrap();
        assert_eq!(meta.len(), 2);
        assert_eq!(meta.get("origin").unwrap(), "system");
        assert_eq!(meta.get("timestamp").unwrap(), "123456789");
        assert_eq!(op.has_agent(), false);
    }
    
    #[test]
    fn test_content_addressed() {
        let op1 = Operation::new(
            "resource:123".into(),
            OperationType::Create,
            Some("agent:456".into()),
        )
        .with_parameter("param", "value");
        
        let op2 = Operation::new(
            "resource:123".into(),
            OperationType::Create,
            Some("agent:456".into()),
        )
        .with_parameter("param", "value");
        
        // Same content should produce same hash
        let hash1 = op1.content_hash().unwrap();
        let hash2 = op2.content_hash().unwrap();
        assert_eq!(hash1, hash2);
        
        // Different content should produce different hash
        let op3 = op1.clone().with_parameter("param2", "value2");
        let hash3 = op3.content_hash().unwrap();
        assert_ne!(hash1, hash3);
    }
} 