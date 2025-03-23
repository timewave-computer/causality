// Resource Relationship Module
//
// This module defines relationships between resources, including cross-domain relationships
// that span different domains in the system.

pub mod cross_domain;
pub mod validation;
pub mod sync;
pub mod scheduler;

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

use crate::error::{Error, Result};
use crate::types::{ResourceId, DomainId};

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
        Self {
            id: Uuid::new_v4().to_string(),
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