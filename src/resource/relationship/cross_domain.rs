//! Cross-domain relationships between resources
//!
//! This module defines types and functionality for managing relationships
//! between resources that span multiple domains.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use borsh::{BorshSerialize, BorshDeserialize};
use crate::crypto::content_addressed::{ContentAddressed, ContentId};
use crate::error::{Error, Result};
use std::time::SystemTime;

/// Type of cross-domain relationship
#[derive(Debug, Clone, PartialEq)]
pub enum CrossDomainRelationshipType {
    /// Mirror relationship - resources are identical across domains
    Mirror,
    
    /// Reference relationship - resources reference each other
    Reference,
    
    /// Ownership relationship - source resource owns target resource
    Ownership,
    
    /// Derived relationship - target resource is derived from source
    Derived,
    
    /// Bridge relationship - connects resources across domain boundaries
    Bridge,
    
    /// Custom relationship with specified type
    Custom(String),
}

/// Metadata for cross-domain relationships
#[derive(Debug, Clone)]
pub struct CrossDomainMetadata {
    /// The domain containing the source resource
    pub origin_domain: String,
    
    /// The domain containing the target resource
    pub target_domain: String,
    
    /// Whether this relationship requires synchronization
    pub requires_sync: bool,
    
    /// Strategy for synchronizing the relationship
    pub sync_strategy: SyncStrategy,
}

/// Strategy for synchronizing cross-domain relationships
#[derive(Debug, Clone)]
pub enum SyncStrategy {
    /// Periodic synchronization with specified interval
    Periodic(Duration),
    
    /// Event-driven synchronization (when resources change)
    EventDriven,
    
    /// Hybrid approach with both event-driven and periodic sync
    Hybrid(Duration),
    
    /// Manual synchronization only
    Manual,
}

/// Content data for cross-domain relationship content addressing
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RelationshipContentData {
    /// Source resource identifier
    pub source_resource: String,
    
    /// Domain containing the source resource
    pub source_domain: String,
    
    /// Target resource identifier
    pub target_resource: String,
    
    /// Target domain
    pub target_domain: String,
    
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

/// Represents a relationship between resources across different domains
#[derive(Debug, Clone)]
pub struct CrossDomainRelationship {
    /// Unique identifier for this relationship
    pub id: String,
    
    /// Source resource identifier
    pub source_resource: String,
    
    /// Domain containing the source resource
    pub source_domain: String,
    
    /// Target resource identifier
    pub target_resource: String,
    
    /// Domain containing the target resource
    pub target_domain: String,
    
    /// Type of the relationship
    pub relationship_type: CrossDomainRelationshipType,
    
    /// Metadata for this relationship
    pub metadata: CrossDomainMetadata,
    
    /// Whether the relationship is bidirectional
    pub bidirectional: bool,
}

impl CrossDomainRelationship {
    /// Create a new cross-domain relationship
    pub fn new(
        source_resource: String,
        source_domain: String,
        target_resource: String,
        target_domain: String,
        relationship_type: CrossDomainRelationshipType,
        metadata: CrossDomainMetadata,
        bidirectional: bool,
    ) -> Self {
        // Generate a content-based ID
        let type_id = match &relationship_type {
            CrossDomainRelationshipType::Mirror => "mirror".to_string(),
            CrossDomainRelationshipType::Reference => "reference".to_string(),
            CrossDomainRelationshipType::Ownership => "ownership".to_string(),
            CrossDomainRelationshipType::Derived => "derived".to_string(),
            CrossDomainRelationshipType::Bridge => "bridge".to_string(),
            CrossDomainRelationshipType::Custom(name) => format!("custom:{}", name),
        };
        
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let mut nonce = [0u8; 8];
        getrandom::getrandom(&mut nonce).expect("Failed to generate random nonce");
        
        let content_data = RelationshipContentData {
            source_resource: source_resource.clone(),
            source_domain: source_domain.clone(),
            target_resource: target_resource.clone(),
            target_domain: target_domain.clone(),
            relationship_type_id: type_id,
            timestamp: now,
            nonce,
        };
        
        let id = content_data.content_hash()
            .map(|id| id.to_string())
            .unwrap_or_else(|_| format!("error-generating-id-{}", now));
        
        Self {
            id,
            source_resource,
            source_domain,
            target_resource,
            target_domain,
            relationship_type,
            metadata,
            bidirectional,
        }
    }
    
    /// Get a unique identifier for this relationship type
    pub fn type_id(&self) -> String {
        match &self.relationship_type {
            CrossDomainRelationshipType::Mirror => "mirror".to_string(),
            CrossDomainRelationshipType::Reference => "reference".to_string(),
            CrossDomainRelationshipType::Ownership => "ownership".to_string(),
            CrossDomainRelationshipType::Derived => "derived".to_string(),
            CrossDomainRelationshipType::Bridge => "bridge".to_string(),
            CrossDomainRelationshipType::Custom(name) => format!("custom:{}", name),
        }
    }
    
    /// Check if the relationship requires synchronization
    pub fn requires_sync(&self) -> bool {
        self.metadata.requires_sync
    }
    
    /// Get the sync strategy for this relationship
    pub fn sync_strategy(&self) -> &SyncStrategy {
        &self.metadata.sync_strategy
    }
}

/// Manager for cross-domain relationships
pub struct CrossDomainRelationshipManager {
    // Store relationships by ID for quick lookup
    relationships: RwLock<HashMap<String, CrossDomainRelationship>>,
    
    // Index by source resource for querying
    source_index: RwLock<HashMap<String, Vec<String>>>,
    
    // Index by target resource for querying
    target_index: RwLock<HashMap<String, Vec<String>>>,
    
    // Index by source domain for querying
    source_domain_index: RwLock<HashMap<String, Vec<String>>>,
    
    // Index by target domain for querying
    target_domain_index: RwLock<HashMap<String, Vec<String>>>,
}

impl CrossDomainRelationshipManager {
    /// Create a new cross-domain relationship manager
    pub fn new() -> Self {
        Self {
            relationships: RwLock::new(HashMap::new()),
            source_index: RwLock::new(HashMap::new()),
            target_index: RwLock::new(HashMap::new()),
            source_domain_index: RwLock::new(HashMap::new()),
            target_domain_index: RwLock::new(HashMap::new()),
        }
    }
    
    /// Add a relationship to the manager
    pub fn add_relationship(&self, relationship: CrossDomainRelationship) -> Result<()> {
        // Add to main storage
        {
            let mut relationships = self.relationships.write().unwrap();
            relationships.insert(relationship.id.clone(), relationship.clone());
        }
        
        // Update source resource index
        {
            let mut source_index = self.source_index.write().unwrap();
            source_index
                .entry(relationship.source_resource.clone())
                .or_insert_with(Vec::new)
                .push(relationship.id.clone());
        }
        
        // Update target resource index
        {
            let mut target_index = self.target_index.write().unwrap();
            target_index
                .entry(relationship.target_resource.clone())
                .or_insert_with(Vec::new)
                .push(relationship.id.clone());
        }
        
        // Update source domain index
        {
            let mut source_domain_index = self.source_domain_index.write().unwrap();
            source_domain_index
                .entry(relationship.source_domain.clone())
                .or_insert_with(Vec::new)
                .push(relationship.id.clone());
        }
        
        // Update target domain index
        {
            let mut target_domain_index = self.target_domain_index.write().unwrap();
            target_domain_index
                .entry(relationship.target_domain.clone())
                .or_insert_with(Vec::new)
                .push(relationship.id.clone());
        }
        
        Ok(())
    }
    
    /// Remove a relationship from the manager
    pub fn remove_relationship(&self, relationship_id: &str) -> Result<()> {
        // First get the relationship to update indexes
        let relationship = {
            let relationships = self.relationships.read().unwrap();
            match relationships.get(relationship_id) {
                Some(rel) => rel.clone(),
                None => return Err(Error::NotFound(format!("Relationship not found: {}", relationship_id))),
            }
        };
        
        // Remove from main storage
        {
            let mut relationships = self.relationships.write().unwrap();
            relationships.remove(relationship_id);
        }
        
        // Update source resource index
        {
            let mut source_index = self.source_index.write().unwrap();
            if let Some(ids) = source_index.get_mut(&relationship.source_resource) {
                ids.retain(|id| id != relationship_id);
                if ids.is_empty() {
                    source_index.remove(&relationship.source_resource);
                }
            }
        }
        
        // Update target resource index
        {
            let mut target_index = self.target_index.write().unwrap();
            if let Some(ids) = target_index.get_mut(&relationship.target_resource) {
                ids.retain(|id| id != relationship_id);
                if ids.is_empty() {
                    target_index.remove(&relationship.target_resource);
                }
            }
        }
        
        // Update source domain index
        {
            let mut source_domain_index = self.source_domain_index.write().unwrap();
            if let Some(ids) = source_domain_index.get_mut(&relationship.source_domain) {
                ids.retain(|id| id != relationship_id);
                if ids.is_empty() {
                    source_domain_index.remove(&relationship.source_domain);
                }
            }
        }
        
        // Update target domain index
        {
            let mut target_domain_index = self.target_domain_index.write().unwrap();
            if let Some(ids) = target_domain_index.get_mut(&relationship.target_domain) {
                ids.retain(|id| id != relationship_id);
                if ids.is_empty() {
                    target_domain_index.remove(&relationship.target_domain);
                }
            }
        }
        
        Ok(())
    }
    
    /// Get a specific relationship by ID
    pub fn get_relationship(&self, relationship_id: &str) -> Result<CrossDomainRelationship> {
        let relationships = self.relationships.read().unwrap();
        match relationships.get(relationship_id) {
            Some(rel) => Ok(rel.clone()),
            None => Err(Error::NotFound(format!("Relationship not found: {}", relationship_id))),
        }
    }
    
    /// Get all relationships
    pub fn get_all_relationships(&self) -> Result<Vec<CrossDomainRelationship>> {
        let relationships = self.relationships.read().unwrap();
        Ok(relationships.values().cloned().collect())
    }
    
    /// Get relationships by source resource
    pub fn get_relationships_by_source_resource(&self, resource_id: String) -> Result<Vec<CrossDomainRelationship>> {
        let source_index = self.source_index.read().unwrap();
        let relationships = self.relationships.read().unwrap();
        
        let rel_ids = match source_index.get(&resource_id) {
            Some(ids) => ids,
            None => return Ok(Vec::new()),
        };
        
        Ok(rel_ids
            .iter()
            .filter_map(|id| relationships.get(id).cloned())
            .collect())
    }
    
    /// Get relationships by target resource
    pub fn get_relationships_by_target_resource(&self, resource_id: String) -> Result<Vec<CrossDomainRelationship>> {
        let target_index = self.target_index.read().unwrap();
        let relationships = self.relationships.read().unwrap();
        
        let rel_ids = match target_index.get(&resource_id) {
            Some(ids) => ids,
            None => return Ok(Vec::new()),
        };
        
        Ok(rel_ids
            .iter()
            .filter_map(|id| relationships.get(id).cloned())
            .collect())
    }
    
    /// Get relationships for a resource (either as source or target)
    pub fn get_relationships_for_resource(&self, resource_id: String) -> Result<Vec<CrossDomainRelationship>> {
        let mut result = self.get_relationships_by_source_resource(resource_id.clone())?;
        let target_rels = self.get_relationships_by_target_resource(resource_id)?;
        
        // Add target relationships, avoiding duplicates for bidirectional relationships
        for rel in target_rels {
            if !result.iter().any(|r| r.id == rel.id) {
                result.push(rel);
            }
        }
        
        Ok(result)
    }
    
    /// Get relationships by source domain
    pub fn get_relationships_by_source_domain(&self, domain: String) -> Result<Vec<CrossDomainRelationship>> {
        let source_domain_index = self.source_domain_index.read().unwrap();
        let relationships = self.relationships.read().unwrap();
        
        let rel_ids = match source_domain_index.get(&domain) {
            Some(ids) => ids,
            None => return Ok(Vec::new()),
        };
        
        Ok(rel_ids
            .iter()
            .filter_map(|id| relationships.get(id).cloned())
            .collect())
    }
    
    /// Get relationships by target domain
    pub fn get_relationships_by_target_domain(&self, domain: String) -> Result<Vec<CrossDomainRelationship>> {
        let target_domain_index = self.target_domain_index.read().unwrap();
        let relationships = self.relationships.read().unwrap();
        
        let rel_ids = match target_domain_index.get(&domain) {
            Some(ids) => ids,
            None => return Ok(Vec::new()),
        };
        
        Ok(rel_ids
            .iter()
            .filter_map(|id| relationships.get(id).cloned())
            .collect())
    }
    
    /// Get relationships between two domains
    pub fn get_relationships_between_domains(&self, source_domain: String, target_domain: String) -> Result<Vec<CrossDomainRelationship>> {
        let source_rels = self.get_relationships_by_source_domain(source_domain)?;
        
        Ok(source_rels
            .into_iter()
            .filter(|rel| rel.target_domain == target_domain)
            .collect())
    }
    
    /// Get mirror relationships for a resource
    pub fn get_mirror_relationships(&self, resource_id: String) -> Result<Vec<CrossDomainRelationship>> {
        let all_rels = self.get_relationships_for_resource(resource_id)?;
        
        Ok(all_rels
            .into_iter()
            .filter(|rel| matches!(rel.relationship_type, CrossDomainRelationshipType::Mirror))
            .collect())
    }
} 