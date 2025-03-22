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
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};
use crate::types::{ResourceId, DomainId, Timestamp, Metadata};
use crate::time::TimeMapSnapshot;

/// Direction of a relationship
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelationshipType {
    /// Parent-child relationship
    ParentChild,
    /// Dependency relationship
    Dependency,
    /// Custom relationship type
    Custom(String),
}

impl fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelationshipType::ParentChild => write!(f, "ParentChild"),
            RelationshipType::Dependency => write!(f, "Dependency"),
            RelationshipType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// Resource relationship record
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRelationship {
    /// Unique ID of the relationship
    pub id: String,
    
    /// Source resource ID
    pub source_id: ResourceId,
    
    /// Target resource ID
    pub target_id: ResourceId,
    
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

/// Indexes to efficiently query relationships
#[derive(Debug, Clone, Default)]
struct RelationshipIndex {
    /// Index by resource ID
    by_resource: HashMap<ResourceId, HashSet<ResourceId>>,
    /// Index by relationship type
    by_type: HashMap<RelationshipType, HashSet<(ResourceId, ResourceId)>>,
    /// Index by resource pair
    by_pair: HashMap<(ResourceId, ResourceId), HashSet<ResourceId>>,
}

/// Tracker for relationships between resources
pub struct RelationshipTracker {
    /// All relationships, keyed by ID
    relationships: RwLock<HashMap<String, ResourceRelationship>>,
    
    /// Index of relationships by source resource ID
    source_index: RwLock<HashMap<ResourceId, HashSet<String>>>,
    
    /// Index of relationships by target resource ID
    target_index: RwLock<HashMap<ResourceId, HashSet<String>>>,
    
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
        from_id: ResourceId,
        to_id: ResourceId,
        relationship_type: RelationshipType,
        direction: RelationshipDirection,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: Uuid::new_v4().to_string(),
            source_id: from_id,
            target_id: to_id,
            relationship_type,
            direction,
            created_at: now,
            transaction_id: None,
            metadata: Metadata::default(),
        }
    }

    /// Add metadata to the relationship and return self
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
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

    /// Record a new relationship
    pub fn record_relationship(
        &self,
        source_id: ResourceId,
        target_id: ResourceId,
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

    /// Record a parent-child relationship
    pub fn record_parent_child_relationship(
        &self,
        parent_id: ResourceId,
        child_id: ResourceId,
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

    /// Record a dependency relationship
    pub fn record_dependency_relationship(
        &self,
        dependent_id: ResourceId,
        dependency_id: ResourceId,
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

    /// Record a derivation relationship
    pub fn record_derivation_relationship(
        &self,
        original_id: ResourceId,
        derived_id: ResourceId,
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
    pub fn get_resource_relationships(&self, resource_id: &ResourceId) -> Result<Vec<ResourceRelationship>> {
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
        resource_id: &ResourceId,
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
        resource_id: &ResourceId,
        relationship_type: &RelationshipType,
        direction: Option<RelationshipDirection>,
    ) -> Result<HashSet<ResourceId>> {
        let relationships = self.get_resource_relationships(resource_id)?;
        let mut result = HashSet::new();
        
        for rel in relationships {
            if rel.relationship_type == *relationship_type {
                if let Some(dir) = &direction {
                    match (dir, &rel.direction) {
                        // For forward relationships where this resource is the source
                        (RelationshipDirection::ParentToChild, RelationshipDirection::ParentToChild | RelationshipDirection::Bidirectional) 
                            if rel.source_id == *resource_id => {
                            result.insert(rel.target_id);
                        },
                        
                        // For reverse relationships where this resource is the target
                        (RelationshipDirection::ChildToParent, RelationshipDirection::ChildToParent | RelationshipDirection::Bidirectional)
                            if rel.target_id == *resource_id => {
                            result.insert(rel.source_id);
                        },
                        
                        // For bidirectional relationships
                        (RelationshipDirection::Bidirectional, _) => {
                            if rel.source_id == *resource_id {
                                result.insert(rel.target_id);
                            } else if rel.target_id == *resource_id {
                                result.insert(rel.source_id);
                            }
                        },
                        
                        // Skip other combinations
                        _ => {},
                    }
                } else {
                    // If no direction specified, include all related resources
                    if rel.source_id == *resource_id {
                        result.insert(rel.target_id);
                    } else if rel.target_id == *resource_id {
                        result.insert(rel.source_id);
                    }
                }
            }
        }
        
        Ok(result)
    }

    /// Get child resources for a parent
    pub fn get_child_resources(&self, parent_id: &ResourceId) -> Result<HashSet<ResourceId>> {
        self.get_related_resources(
            parent_id,
            &RelationshipType::ParentChild,
            Some(RelationshipDirection::ParentToChild),
        )
    }

    /// Get parent resources for a child
    pub fn get_parent_resources(&self, child_id: &ResourceId) -> Result<HashSet<ResourceId>> {
        self.get_related_resources(
            child_id,
            &RelationshipType::ParentChild,
            Some(RelationshipDirection::ChildToParent),
        )
    }

    /// Get dependencies for a resource
    pub fn get_dependency_resources(&self, dependent_id: &ResourceId) -> Result<HashSet<ResourceId>> {
        self.get_related_resources(
            dependent_id,
            &RelationshipType::Dependency,
            Some(RelationshipDirection::ChildToParent),
        )
    }

    /// Get dependent resources for a dependency
    pub fn get_dependent_resources(&self, dependency_id: &ResourceId) -> Result<HashSet<ResourceId>> {
        self.get_related_resources(
            dependency_id,
            &RelationshipType::Dependency,
            Some(RelationshipDirection::ParentToChild),
        )
    }

    /// Get derived resources for an original
    pub fn get_derived_resources(&self, original_id: &ResourceId) -> Result<HashSet<ResourceId>> {
        self.get_related_resources(
            original_id,
            &RelationshipType::Dependency,
            Some(RelationshipDirection::ChildToParent),
        )
    }

    /// Get original resource for a derived resource
    pub fn get_original_resource(&self, derived_id: &ResourceId) -> Result<HashSet<ResourceId>> {
        self.get_related_resources(
            derived_id,
            &RelationshipType::Derivation,
            Some(RelationshipDirection::ParentToChild),
        )
    }

    /// Helper to get all relationship IDs for a resource (as source or target)
    fn get_all_relationship_ids_for_resource(&self, resource_id: &ResourceId) -> Result<HashSet<String>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_relationship_creation_and_retrieval() -> Result<()> {
        let snapshot = TimeMapSnapshot::default();
        let tracker = RelationshipTracker::new(snapshot);
        
        let resource_a = "resource-a".to_string();
        let resource_b = "resource-b".to_string();
        
        // Create a relationship
        let rel = tracker.record_parent_child_relationship(
            resource_a.clone(),
            resource_b.clone(),
            Some("test-tx".to_string()),
        )?;
        
        // Get relationships for resource A
        let relationships = tracker.get_resource_relationships(&resource_a)?;
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].id, rel.id);
        assert_eq!(relationships[0].source_id, resource_a);
        assert_eq!(relationships[0].target_id, resource_b);
        assert_eq!(relationships[0].relationship_type, RelationshipType::ParentChild);
        
        // Get child resources
        let children = tracker.get_child_resources(&resource_a)?;
        assert_eq!(children.len(), 1);
        assert!(children.contains(&resource_b));
        
        // Get parent resources
        let parents = tracker.get_parent_resources(&resource_b)?;
        assert_eq!(parents.len(), 1);
        assert!(parents.contains(&resource_a));
        
        Ok(())
    }
    
    #[test]
    fn test_relationship_deletion() -> Result<()> {
        let snapshot = TimeMapSnapshot::default();
        let tracker = RelationshipTracker::new(snapshot);
        
        let resource_a = "resource-a".to_string();
        let resource_b = "resource-b".to_string();
        
        // Create a relationship
        let rel = tracker.record_parent_child_relationship(
            resource_a.clone(),
            resource_b.clone(),
            Some("test-tx".to_string()),
        )?;
        
        // Verify relationship exists
        let relationships = tracker.get_resource_relationships(&resource_a)?;
        assert_eq!(relationships.len(), 1);
        
        // Delete the relationship
        tracker.delete_relationship(&rel.id)?;
        
        // Verify relationship is gone
        let relationships = tracker.get_resource_relationships(&resource_a)?;
        assert_eq!(relationships.len(), 0);
        
        // Child resources should be empty
        let children = tracker.get_child_resources(&resource_a)?;
        assert_eq!(children.len(), 0);
        
        Ok(())
    }
    
    #[test]
    fn test_multiple_relationship_types() -> Result<()> {
        let snapshot = TimeMapSnapshot::default();
        let tracker = RelationshipTracker::new(snapshot);
        
        let resource_a = "resource-a".to_string();
        let resource_b = "resource-b".to_string();
        let resource_c = "resource-c".to_string();
        
        // Create different relationship types
        tracker.record_parent_child_relationship(
            resource_a.clone(),
            resource_b.clone(),
            Some("tx-1".to_string()),
        )?;
        
        tracker.record_dependency_relationship(
            resource_a.clone(),
            resource_c.clone(),
            Some("tx-2".to_string()),
        )?;
        
        tracker.record_derivation_relationship(
            resource_b.clone(),
            resource_c.clone(),
            Some("tx-3".to_string()),
        )?;
        
        // Check filtering by type
        let parent_child_rels = tracker.get_relationships_by_type(&RelationshipType::ParentChild)?;
        assert_eq!(parent_child_rels.len(), 1);
        assert_eq!(parent_child_rels[0].source_id, resource_a);
        assert_eq!(parent_child_rels[0].target_id, resource_b);
        
        let dependency_rels = tracker.get_relationships_by_type(&RelationshipType::Dependency)?;
        assert_eq!(dependency_rels.len(), 1);
        assert_eq!(dependency_rels[0].source_id, resource_a);
        assert_eq!(dependency_rels[0].target_id, resource_c);
        
        // Check getting all relationships for resource A
        let a_rels = tracker.get_resource_relationships(&resource_a)?;
        assert_eq!(a_rels.len(), 2);
        
        Ok(())
    }
    
    #[test]
    fn test_relationship_with_metadata() -> Result<()> {
        let snapshot = TimeMapSnapshot::default();
        let tracker = RelationshipTracker::new(snapshot);
        
        let resource_a = "resource-a".to_string();
        let resource_b = "resource-b".to_string();
        
        // Create a relationship with metadata
        let rel = tracker.record_relationship(
            resource_a.clone(),
            resource_b.clone(),
            RelationshipType::Composition,
            RelationshipDirection::ParentToChild,
            Some("test-tx".to_string()),
        )?
        .with_metadata("weight", "0.5")
        .with_metadata("description", "Test relationship");
        
        // Verify the relationship exists with metadata
        let relationships = tracker.get_resource_relationships(&resource_a)?;
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].metadata.get("weight"), Some(&"0.5".to_string()));
        assert_eq!(relationships[0].metadata.get("description"), Some(&"Test relationship".to_string()));
        
        Ok(())
    }
} 