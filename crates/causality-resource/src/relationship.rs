// Resource relationship system
// Original file: src/resource/relationship/mod.rs

// Resource Relationship Module
//
// This module defines relationships between resources, including cross-domain relationships
// that span different domains in the system.

use std::collections::HashMap;
use borsh::{BorshSerialize, BorshDeserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::crypto::content_addressed::{ContentAddressed, ContentId};

pub mod cross_domain;
pub mod validation;
pub mod sync;
pub mod scheduler;
pub mod query;

// Re-export main types
pub use cross_domain::{
    CrossDomainRelationship,
    CrossDomainRelationshipType,
    CrossDomainMetadata,
    CrossDomainRelationshipManager,
};

pub use validation::{
    ValidationLevel,
    ValidationResult,
    ValidationRule,
    ValidationRuleType,
    CrossDomainRelationshipValidator,
};

pub use sync::{
    SyncStrategy,
    SyncStatus,
    SyncResult,
    CrossDomainSyncManager,
};

pub use scheduler::{
    SchedulerConfig,
    RetryBackoff,
    SchedulerStatus,
    CrossDomainSyncScheduler,
};

pub use query::{
    RelationshipPath,
    RelationshipQuery,
    RelationshipQueryExecutor,
};

use causality_types::{Error, Result};
use causality_types::{*};
use causality_crypto::ContentId;;

/// Content data for relationship content addressing
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RelationshipContentData {
    /// Source resource identifier
    pub source: String,
    
    /// Target resource identifier
    pub target: String,
    
    /// Relationship type identifier
    pub relationship_type_id: String,
    
    /// Creation timestamp
    pub timestamp: u64,
    
    /// Random nonce for uniqueness
    pub nonce: [u8; 8],
}

impl ContentAddressed for RelationshipContentData {
    fn content_hash(&self) -> Result<ContentId> {
        let bytes = self.to_bytes()?;
        Ok(ContentId::from_bytes(&bytes)?)
    }
    
    fn verify(&self, content_id: &ContentId) -> Result<bool> {
        let calculated_id = self.content_hash()?;
        Ok(calculated_id == *content_id)
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let bytes = borsh::to_vec(self)
            .map_err(|e| Error::Serialization(format!("Failed to serialize RelationshipContentData: {}", e)))?;
        Ok(bytes)
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        borsh::from_slice(bytes)
            .map_err(|e| Error::Deserialization(format!("Failed to deserialize RelationshipContentData: {}", e)))
    }
}

/// Represents a relationship between two resources
#[derive(Debug, Clone)]
pub struct Relationship {
    /// Unique identifier for this relationship
    pub id: String,
    
    /// Source resource identifier
    pub source: String,
    
    /// Target resource identifier
    pub target: String,
    
    /// Type of the relationship
    pub relationship_type: RelationshipType,
    
    /// Additional metadata for this relationship
    pub metadata: HashMap<String, String>,
}

/// Represents different types of relationships between resources
#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipType {
    /// Direct reference to another resource
    Reference,
    
    /// Ownership relationship (source owns target)
    Ownership,
    
    /// Derivation relationship (target is derived from source)
    Derived,
    
    /// Cross-domain relationship (managed through cross-domain mechanisms)
    CrossDomain(CrossDomainRelationshipType),
    
    /// Custom relationship type with a name
    Custom(String),
}

impl Relationship {
    /// Create a new relationship between resources
    pub fn new(
        source: String,
        target: String,
        relationship_type: RelationshipType,
    ) -> Self {
        // Generate a content-based ID
        let type_id = match &relationship_type {
            RelationshipType::Reference => "reference".to_string(),
            RelationshipType::Ownership => "ownership".to_string(),
            RelationshipType::Derived => "derived".to_string(),
            RelationshipType::CrossDomain(cross_type) => format!("cross_domain:{}", cross_type.type_id()),
            RelationshipType::Custom(name) => format!("custom:{}", name),
        };
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let mut nonce = [0u8; 8];
        getrandom::getrandom(&mut nonce).expect("Failed to generate random nonce");
        
        let content_data = RelationshipContentData {
            source: source.clone(),
            target: target.clone(),
            relationship_type_id: type_id,
            timestamp: now,
            nonce,
        };
        
        let id = content_data.content_hash()
            .map(|id| id.to_string())
            .unwrap_or_else(|_| format!("error-generating-id-{}", now));
        
        Self {
            id,
            source,
            target,
            relationship_type,
            metadata: HashMap::new(),
        }
    }
    
    /// Set metadata value
    pub fn set_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// Convert to cross-domain relationship if compatible
    pub fn to_cross_domain(
        &self,
        source_domain: String,
        target_domain: String,
        cross_domain_type: CrossDomainRelationshipType,
        metadata: CrossDomainMetadata,
        bidirectional: bool,
    ) -> Result<CrossDomainRelationship> {
        // Create the cross domain relationship
        Ok(CrossDomainRelationship::new(
            self.source.clone(),
            source_domain,
            self.target.clone(),
            target_domain,
            cross_domain_type,
            metadata,
            bidirectional,
        ))
    }
}

/// Manager for handling relationships between resources
#[derive(Debug, Default)]
pub struct RelationshipManager {
    relationships: HashMap<String, Relationship>,
}

impl RelationshipManager {
    /// Create a new relationship manager
    pub fn new() -> Self {
        Self {
            relationships: HashMap::new(),
        }
    }
    
    /// Create a relationship between resources
    pub fn create_relationship(
        &mut self,
        source: String,
        target: String,
        relationship_type: RelationshipType,
    ) -> Relationship {
        let relationship = Relationship::new(source, target, relationship_type);
        self.relationships.insert(relationship.id.clone(), relationship.clone());
        relationship
    }
    
    /// Create a cross-domain relationship
    pub fn create_cross_domain_relationship(
        &mut self,
        source: String,
        source_domain: String,
        target: String,
        target_domain: String,
        cross_domain_type: CrossDomainRelationshipType,
        metadata: CrossDomainMetadata,
        bidirectional: bool,
    ) -> Result<CrossDomainRelationship> {
        // Create a standard relationship first
        let relationship = self.create_relationship(
            source.clone(),
            target.clone(),
            RelationshipType::CrossDomain(cross_domain_type.clone()),
        );
        
        // Convert to cross-domain relationship
        relationship.to_cross_domain(
            source_domain,
            target_domain,
            cross_domain_type,
            metadata,
            bidirectional,
        )
    }
    
    // Other methods for managing relationships omitted for brevity
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_relationship() {
        let mut manager = RelationshipManager::new();
        let relationship = manager.create_relationship(
            "resource1".to_string(),
            "resource2".to_string(),
            RelationshipType::Reference,
        );
        
        assert_eq!(relationship.source, "resource1");
        assert_eq!(relationship.target, "resource2");
        assert_eq!(relationship.relationship_type, RelationshipType::Reference);
    }
    
    #[test]
    fn test_convert_to_cross_domain() -> Result<()> {
        let mut manager = RelationshipManager::new();
        let relationship = manager.create_relationship(
            "resource1".to_string(),
            "resource2".to_string(),
            RelationshipType::Reference,
        );
        
        let metadata = CrossDomainMetadata {
            origin_domain: "domain1".to_string(),
            target_domain: "domain2".to_string(),
            requires_sync: true,
            sync_strategy: SyncStrategy::EventDriven,
        };
        
        let cross_domain = relationship.to_cross_domain(
            "domain1".to_string(),
            "domain2".to_string(),
            CrossDomainRelationshipType::Reference,
            metadata,
            true,
        )?;
        
        assert_eq!(cross_domain.source_resource, "resource1");
        assert_eq!(cross_domain.target_resource, "resource2");
        assert_eq!(cross_domain.source_domain, "domain1");
        assert_eq!(cross_domain.target_domain, "domain2");
        assert_eq!(cross_domain.relationship_type, CrossDomainRelationshipType::Reference);
        assert!(cross_domain.bidirectional);
        
        Ok(())
    }
} 
