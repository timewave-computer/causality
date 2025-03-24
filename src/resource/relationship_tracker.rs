// Resource Relationship Tracker for Unified ResourceRegister
//
// This module implements tracking of relationships between resources
// in the unified ResourceRegister model as defined in ADR-021.
// It provides mechanisms to track dependencies, derivations, ownership,
// and other relationships between resources.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::fmt;
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};
use crate::types::{*};
use crate::crypto::hash::ContentId;;
use crate::time::TimeMapSnapshot;
use crate::crypto::hash::{ContentAddressed, ContentId, HashOutput, HashFactory, HashError};
use crate::resource::resource_register::ResourceRegister;

/// Direction of a relationship
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipDirection {
    /// From resource A to resource B
    ParentToChild,
    /// From resource B to resource A
    ChildToParent,
    /// Both ways between A and B
    Bidirectional,
}

impl fmt::Display for RelationshipDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelationshipDirection::ParentToChild => write!(f, "ParentToChild"),
            RelationshipDirection::ChildToParent => write!(f, "ChildToParent"),
            RelationshipDirection::Bidirectional => write!(f, "Bidirectional"),
        }
    }
}

/// Type of relationship between resources
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Parent-child relationship
    ParentChild,
    
    /// Dependency relationship
    Dependency,
    
    /// Consumption relationship
    Consumption,
    
    /// Reference relationship
    Reference,
    
    /// Custom relationship type
    Custom(String),
}

impl fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelationshipType::ParentChild => write!(f, "ParentChild"),
            RelationshipType::Dependency => write!(f, "Dependency"),
            RelationshipType::Consumption => write!(f, "Consumption"),
            RelationshipType::Reference => write!(f, "Reference"),
            RelationshipType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// Resource relationship record for the unified ResourceRegister model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ResourceRelationship {
    /// Unique ID of the relationship
    pub id: String,
    
    /// Source resource content ID
    pub source_id: ContentId,
    
    /// Target resource content ID
    pub target_id: ContentId,
    
    /// Type of relationship
    pub relationship_type: RelationshipType,
    
    /// Direction of the relationship
    pub direction: RelationshipDirection,
    
    /// When the relationship was created
    pub created_at: Timestamp,
    
    /// Transaction that created the relationship
    pub transaction_id: Option<String>,
    
    /// Additional metadata about the relationship
    pub metadata: Metadata,
}

impl ContentAddressed for ResourceRelationship {
    fn content_hash(&self) -> HashOutput {
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        true
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.try_to_vec().unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        BorshDeserialize::try_from_slice(bytes)
            .map_err(|e| HashError::SerializationError(e.to_string()))
    }
}

/// Indexes to efficiently query relationships
#[derive(Debug, Clone, Default)]
struct RelationshipIndex {
    /// Index by resource ID
    by_resource: HashMap<ContentId, HashSet<ContentId>>,
    /// Index by relationship type
    by_type: HashMap<RelationshipType, HashSet<(ContentId, ContentId)>>,
    /// Index by resource pair
    by_pair: HashMap<(ContentId, ContentId), HashSet<ContentId>>,
}

/// Tracker for relationships between resources in the unified ResourceRegister model
pub struct RelationshipTracker {
    /// All relationships, keyed by ID
    relationships: RwLock<HashMap<String, ResourceRelationship>>,
    
    /// Index of relationships by source resource ID
    source_index: RwLock<HashMap<ContentId, HashSet<String>>>,
    
    /// Index of relationships by target resource ID
    target_index: RwLock<HashMap<ContentId, HashSet<String>>>,
    
    /// Index of relationships by type
    type_index: RwLock<HashMap<RelationshipType, HashSet<String>>>,
    
    /// Current time snapshot
    current_snapshot: RwLock<TimeMapSnapshot>,
    
    /// Indexes for efficient querying
    index: RelationshipIndex,
}

impl ResourceRelationship {
    /// Create a new relationship
    pub fn new(
        from_id: ContentId,
        to_id: ContentId,
        relationship_type: RelationshipType,
        direction: RelationshipDirection,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Create a temporary relationship for content ID generation
        let temp_relationship = Self {
            id: String::new(), // Temporary
            source_id: from_id,
            target_id: to_id,
            relationship_type,
            direction,
            created_at: now,
            transaction_id: None,
            metadata: Metadata::default(),
        };
        
        // Generate content-derived ID
        let content_id = temp_relationship.content_id();
        
        // Create the final relationship with the content ID
        let mut result = temp_relationship;
        result.id = format!("rel:{}", content_id);
        result
    }

    /// Add metadata to the relationship and return self
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Get the content ID of this relationship
    pub fn content_id(&self) -> ContentId {
        let hasher = HashFactory::default().create_hasher().unwrap();
        let data = self.try_to_vec().unwrap_or_default();
        ContentId::from(hasher.hash(&data))
    }
}

impl RelationshipTracker {
    /// Create a new relationship tracker
    pub fn new(snapshot: TimeMapSnapshot) -> Self {
        Self {
            relationships: RwLock::new(HashMap::new()),
            source_index: RwLock::new(HashMap::new()),
            target_index: RwLock::new(HashMap::new()),
            type_index: RwLock::new(HashMap::new()),
            current_snapshot: RwLock::new(snapshot),
            index: RelationshipIndex::default(),
        }
    }

    /// Update the current time snapshot
    pub fn update_snapshot(&self, snapshot: TimeMapSnapshot) -> Result<()> {
        let mut current = self.current_snapshot.write().map_err(|_| {
            Error::Internal("Failed to acquire write lock on current_snapshot".to_string())
        })?;
        
        *current = snapshot;
        Ok(())
    }

    /// Get the current time snapshot
    pub fn get_snapshot(&self) -> Result<TimeMapSnapshot> {
        let snapshot = self.current_snapshot.read().map_err(|_| {
            Error::Internal("Failed to acquire read lock on current_snapshot".to_string())
        })?;
        
        Ok(snapshot.clone())
    }

    /// Record a new relationship between two ResourceRegisters
    pub fn record_relationship_between_registers(
        &self,
        source: &ResourceRegister,
        target: &ResourceRegister,
        relationship_type: RelationshipType,
        direction: RelationshipDirection,
        transaction_id: Option<String>,
    ) -> Result<ResourceRelationship> {
        self.record_relationship(
            source.id.clone(),
            target.id.clone(),
            relationship_type,
            direction,
            transaction_id,
        )
    }

    /// Record a new relationship
    pub fn record_relationship(
        &self,
        source_id: ContentId,
        target_id: ContentId,
        relationship_type: RelationshipType,
        direction: RelationshipDirection,
        transaction_id: Option<String>,
    ) -> Result<ResourceRelationship> {
        let snapshot = self.get_snapshot()?;
        
        // Create the relationship
        let relationship = ResourceRelationship::new(
            source_id.clone(),
            target_id.clone(),
            relationship_type.clone(),
            direction,
        );
        
        // Store it in the maps
        let relationship_id = relationship.id.clone();
        
        // Update the main relationship map
        {
            let mut relationships = self.relationships.write().map_err(|_| {
                Error::Internal("Failed to acquire write lock on relationships".to_string())
            })?;
            
            relationships.insert(relationship_id.clone(), relationship.clone());
        }
        
        // Update the source index
        {
            let mut source_index = self.source_index.write().map_err(|_| {
                Error::Internal("Failed to acquire write lock on source_index".to_string())
            })?;
            
            source_index.entry(source_id.clone())
                .or_insert_with(HashSet::new)
                .insert(relationship_id.clone());
        }
        
        // Update the target index
        {
            let mut target_index = self.target_index.write().map_err(|_| {
                Error::Internal("Failed to acquire write lock on target_index".to_string())
            })?;
            
            target_index.entry(target_id.clone())
                .or_insert_with(HashSet::new)
                .insert(relationship_id.clone());
        }
        
        // Update the type index
        {
            let mut type_index = self.type_index.write().map_err(|_| {
                Error::Internal("Failed to acquire write lock on type_index".to_string())
            })?;
            
            type_index.entry(relationship_type.clone())
                .or_insert_with(HashSet::new)
                .insert(relationship_id.clone());
        }
        
        Ok(relationship)
    }

    /// Record a parent-child relationship between ResourceRegisters
    pub fn record_parent_child_relationship_between_registers(
        &self,
        parent: &ResourceRegister,
        child: &ResourceRegister,
        transaction_id: Option<String>,
    ) -> Result<ResourceRelationship> {
        self.record_parent_child_relationship(
            parent.id.clone(),
            child.id.clone(),
            transaction_id,
        )
    }

    /// Record a parent-child relationship
    pub fn record_parent_child_relationship(
        &self,
        parent_id: ContentId,
        child_id: ContentId,
        transaction_id: Option<String>,
    ) -> Result<ResourceRelationship> {
        self.record_relationship(
            parent_id,
            child_id,
            RelationshipType::ParentChild,
            RelationshipDirection::ParentToChild,
            transaction_id,
        )
    }

    /// Record a dependency relationship between ResourceRegisters
    pub fn record_dependency_relationship_between_registers(
        &self,
        dependent: &ResourceRegister,
        dependency: &ResourceRegister,
        transaction_id: Option<String>,
    ) -> Result<ResourceRelationship> {
        self.record_dependency_relationship(
            dependent.id.clone(),
            dependency.id.clone(),
            transaction_id,
        )
    }

    /// Record a dependency relationship
    pub fn record_dependency_relationship(
        &self,
        dependent_id: ContentId,
        dependency_id: ContentId,
        transaction_id: Option<String>,
    ) -> Result<ResourceRelationship> {
        self.record_relationship(
            dependent_id,
            dependency_id,
            RelationshipType::Dependency,
            RelationshipDirection::ChildToParent,
            transaction_id,
        )
    }

    /// Record a derivation relationship between ResourceRegisters
    pub fn record_derivation_relationship_between_registers(
        &self,
        original: &ResourceRegister,
        derived: &ResourceRegister,
        transaction_id: Option<String>,
    ) -> Result<ResourceRelationship> {
        self.record_derivation_relationship(
            original.id.clone(),
            derived.id.clone(),
            transaction_id,
        )
    }

    /// Record a derivation relationship
    pub fn record_derivation_relationship(
        &self,
        original_id: ContentId,
        derived_id: ContentId,
        transaction_id: Option<String>,
    ) -> Result<ResourceRelationship> {
        self.record_relationship(
            original_id,
            derived_id,
            RelationshipType::Dependency,
            RelationshipDirection::ChildToParent,
            transaction_id,
        )
    }

    /// Delete a relationship by ID
    pub fn delete_relationship(&self, relationship_id: &str) -> Result<()> {
        // Get the relationship
        let relationship = {
            let relationships = self.relationships.read().map_err(|_| {
                Error::Internal("Failed to acquire read lock on relationships".to_string())
            })?;
            
            match relationships.get(relationship_id) {
                Some(rel) => rel.clone(),
                None => return Err(Error::NotFound("Relationship not found".to_string())),
            }
        };
        
        // Remove from the main map
        {
            let mut relationships = self.relationships.write().map_err(|_| {
                Error::Internal("Failed to acquire write lock on relationships".to_string())
            })?;
            
            relationships.remove(relationship_id);
        }
        
        // Remove from the source index
        {
            let mut source_index = self.source_index.write().map_err(|_| {
                Error::Internal("Failed to acquire write lock on source_index".to_string())
            })?;
            
            if let Some(set) = source_index.get_mut(&relationship.source_id) {
                set.remove(relationship_id);
                
                // Remove the entry if empty
                if set.is_empty() {
                    source_index.remove(&relationship.source_id);
                }
            }
        }
        
        // Remove from the target index
        {
            let mut target_index = self.target_index.write().map_err(|_| {
                Error::Internal("Failed to acquire write lock on target_index".to_string())
            })?;
            
            if let Some(set) = target_index.get_mut(&relationship.target_id) {
                set.remove(relationship_id);
                
                // Remove the entry if empty
                if set.is_empty() {
                    target_index.remove(&relationship.target_id);
                }
            }
        }
        
        // Remove from the type index
        {
            let mut type_index = self.type_index.write().map_err(|_| {
                Error::Internal("Failed to acquire write lock on type_index".to_string())
            })?;
            
            if let Some(set) = type_index.get_mut(&relationship.relationship_type) {
                set.remove(relationship_id);
                
                // Remove the entry if empty
                if set.is_empty() {
                    type_index.remove(&relationship.relationship_type);
                }
            }
        }
        
        Ok(())
    }

    /// Get all relationships for a resource
    pub fn get_resource_relationships(&self, resource_id: &ContentId) -> Result<Vec<ResourceRelationship>> {
        let relationship_ids = self.get_all_relationship_ids_for_resource(resource_id)?;
        
        let relationships = self.relationships.read().map_err(|_| {
            Error::Internal("Failed to acquire read lock on relationships".to_string())
        })?;
        
        Ok(relationship_ids.iter()
            .filter_map(|id| relationships.get(id).cloned())
            .collect())
    }

    /// Get all relationships of a specific type for a resource
    pub fn get_resource_relationships_by_type(
        &self,
        resource_id: &ContentId,
        relationship_type: &RelationshipType,
    ) -> Result<Vec<ResourceRelationship>> {
        let relationships = self.get_resource_relationships(resource_id)?;
        
        Ok(relationships
            .into_iter()
            .filter(|rel| rel.relationship_type == *relationship_type)
            .collect())
    }

    /// Get all relationships by type
    pub fn get_relationships_by_type(&self, relationship_type: &RelationshipType) -> Result<Vec<ResourceRelationship>> {
        let relationship_ids = {
            let type_index = self.type_index.read().map_err(|_| {
                Error::Internal("Failed to acquire read lock on type_index".to_string())
            })?;
            
            match type_index.get(relationship_type) {
                Some(ids) => ids.clone(),
                None => HashSet::new(),
            }
        };
        
        let relationships = self.relationships.read().map_err(|_| {
            Error::Internal("Failed to acquire read lock on relationships".to_string())
        })?;
        
        Ok(relationship_ids.iter()
            .filter_map(|id| relationships.get(id).cloned())
            .collect())
    }

    /// Get all resources related to a resource by a specific type and direction
    pub fn get_related_resources(
        &self,
        resource_id: &ContentId,
        relationship_type: &RelationshipType,
        direction: Option<RelationshipDirection>,
    ) -> Result<Vec<ContentId>> {
        let relationships = self.get_resource_relationships_by_type(resource_id, relationship_type)?;
        
        let result = relationships.iter()
            .filter(|rel| {
                if let Some(dir) = &direction {
                    rel.direction == *dir || rel.direction == RelationshipDirection::Bidirectional
                } else {
                    true
                }
            })
            .map(|rel| {
                if rel.source_id == *resource_id {
                    rel.target_id.clone()
                } else {
                    rel.source_id.clone()
                }
            })
            .collect();
        
        Ok(result)
    }

    /// Check if two resources are directly related
    pub fn are_resources_related(
        &self,
        resource_a: &ContentId,
        resource_b: &ContentId,
        relationship_type: Option<RelationshipType>,
    ) -> Result<bool> {
        let a_relationships = self.get_resource_relationships(resource_a)?;
        
        for rel in a_relationships {
            if (rel.source_id == *resource_a && rel.target_id == *resource_b) || 
               (rel.source_id == *resource_b && rel.target_id == *resource_a) {
                if let Some(rtype) = &relationship_type {
                    if rel.relationship_type == *rtype {
                        return Ok(true);
                    }
                } else {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }

    /// Find all resources with a specific relationship to a resource
    pub fn find_resources_with_relationship(
        &self,
        relationship_type: &RelationshipType,
        direction: Option<RelationshipDirection>,
    ) -> Result<Vec<(ContentId, ContentId)>> {
        let relationships = self.get_relationships_by_type(relationship_type)?;
        
        let result = relationships.iter()
            .filter(|rel| {
                if let Some(dir) = &direction {
                    rel.direction == *dir || rel.direction == RelationshipDirection::Bidirectional
                } else {
                    true
                }
            })
            .map(|rel| (rel.source_id.clone(), rel.target_id.clone()))
            .collect();
        
        Ok(result)
    }

    /// Check if a relationship exists and get its ID
    pub fn get_relationship_id(
        &self,
        source_id: &ContentId,
        target_id: &ContentId,
        relationship_type: &RelationshipType,
    ) -> Result<Option<String>> {
        let source_relationships = {
            let source_index = self.source_index.read().map_err(|_| {
                Error::Internal("Failed to acquire read lock on source_index".to_string())
            })?;
            
            match source_index.get(source_id) {
                Some(ids) => ids.clone(),
                None => HashSet::new(),
            }
        };
        
        let relationships = self.relationships.read().map_err(|_| {
            Error::Internal("Failed to acquire read lock on relationships".to_string())
        })?;
        
        for rel_id in source_relationships {
            if let Some(rel) = relationships.get(&rel_id) {
                if rel.target_id == *target_id && rel.relationship_type == *relationship_type {
                    return Ok(Some(rel_id));
                }
            }
        }
        
        Ok(None)
    }

    /// Helper to get all relationship IDs for a resource
    fn get_all_relationship_ids_for_resource(&self, resource_id: &ContentId) -> Result<HashSet<String>> {
        let mut result = HashSet::new();
        
        // Get relationships where resource is source
        {
            let source_index = self.source_index.read().map_err(|_| {
                Error::Internal("Failed to acquire read lock on source_index".to_string())
            })?;
            
            if let Some(ids) = source_index.get(resource_id) {
                result.extend(ids.iter().cloned());
            }
        }
        
        // Get relationships where resource is target
        {
            let target_index = self.target_index.read().map_err(|_| {
                Error::Internal("Failed to acquire read lock on target_index".to_string())
            })?;
            
            if let Some(ids) = target_index.get(resource_id) {
                result.extend(ids.iter().cloned());
            }
        }
        
        Ok(result)
    }
}

/// Helper for transitioning from the old ContentId-based relationships
/// to the new ContentId-based relationships
impl RelationshipTracker {
    /// Convert a ContentId to a ContentId (legacy support)
    pub fn resource_id_to_content_id(&self, resource_id: &ContentId) -> ContentId {
        let id_str = resource_id.to_string();
        
        // Create a derived content ID from the resource ID string
        let hasher = HashFactory::default().create_hasher().unwrap();
        let hash = hasher.hash(id_str.as_bytes());
        ContentId::from(hash)
    }
    
    /// Helper method to record relationship using ContentId (legacy support)
    pub fn record_relationship_legacy(
        &self,
        source_id: ContentId,
        target_id: ContentId, 
        relationship_type: RelationshipType,
        direction: RelationshipDirection,
        transaction_id: Option<String>,
    ) -> Result<ResourceRelationship> {
        let source_content_id = self.resource_id_to_content_id(&source_id);
        let target_content_id = self.resource_id_to_content_id(&target_id);
        
        self.record_relationship(
            source_content_id,
            target_content_id,
            relationship_type,
            direction,
            transaction_id,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::resource_register::{ResourceRegister, ResourceLogic, FungibilityDomain, Quantity};
    use crate::resource::resource_register::{StorageStrategy, StateVisibility};
    
    fn create_test_register(id: &str, quantity: u128) -> ResourceRegister {
        let content_id = {
            let hasher = HashFactory::default().create_hasher().unwrap();
            let hash = hasher.hash(id.as_bytes());
            ContentId::from(hash)
        };
        
        ResourceRegister::new(
            content_id,
            ResourceLogic::Fungible,
            FungibilityDomain(format!("test-domain-{}", id)),
            Quantity(quantity),
            HashMap::new(),
            StorageStrategy::FullyOnChain { visibility: StateVisibility::Public },
        )
    }
    
    #[test]
    fn test_basic_relationship_tracking() -> Result<()> {
        // Create a relationship tracker
        let tracker = RelationshipTracker::new(TimeMapSnapshot::default());
        
        // Create test resources
        let parent = create_test_register("parent", 100);
        let child1 = create_test_register("child1", 50);
        let child2 = create_test_register("child2", 25);
        
        // Record parent-child relationships
        let rel1 = tracker.record_parent_child_relationship_between_registers(
            &parent, &child1, None
        )?;
        let rel2 = tracker.record_parent_child_relationship_between_registers(
            &parent, &child2, None
        )?;
        
        // Verify the relationships were recorded
        let parent_relationships = tracker.get_resource_relationships(&parent.id)?;
        assert_eq!(parent_relationships.len(), 2);
        
        // Get children of parent
        let children = tracker.get_related_resources(
            &parent.id,
            &RelationshipType::ParentChild,
            Some(RelationshipDirection::ParentToChild),
        )?;
        assert_eq!(children.len(), 2);
        assert!(children.contains(&child1.id));
        assert!(children.contains(&child2.id));
        
        // Verify resources are related
        assert!(tracker.are_resources_related(&parent.id, &child1.id, None)?);
        assert!(tracker.are_resources_related(&parent.id, &child2.id, Some(RelationshipType::ParentChild))?);
        assert!(!tracker.are_resources_related(&child1.id, &child2.id, None)?);
        
        // Delete a relationship
        tracker.delete_relationship(&rel1.id)?;
        
        // Verify the relationship was deleted
        let parent_relationships = tracker.get_resource_relationships(&parent.id)?;
        assert_eq!(parent_relationships.len(), 1);
        
        // Get children of parent after deletion
        let children = tracker.get_related_resources(
            &parent.id,
            &RelationshipType::ParentChild,
            Some(RelationshipDirection::ParentToChild),
        )?;
        assert_eq!(children.len(), 1);
        assert!(children.contains(&child2.id));
        
        Ok(())
    }
    
    #[test]
    fn test_resource_relationship_content_addressing() -> Result<()> {
        // Create test resources
        let resource1 = create_test_register("resource1", 100);
        let resource2 = create_test_register("resource2", 200);
        
        // Create a relationship
        let relationship = ResourceRelationship::new(
            resource1.id.clone(),
            resource2.id.clone(),
            RelationshipType::ParentChild,
            RelationshipDirection::ParentToChild,
        );
        
        // Verify the relationship is correctly content-addressed
        let content_id = relationship.content_id();
        assert_eq!(format!("rel:{}", content_id), relationship.id);
        
        // Create a relationship with metadata
        let relationship_with_metadata = relationship.clone().with_metadata("key", "value");
        
        // Verify the content IDs are different
        assert_ne!(relationship.id, relationship_with_metadata.id);
        
        Ok(())
    }
} 
