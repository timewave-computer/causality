// Resource Query Indexing
//
// This module provides indexing capabilities for resources,
// enabling efficient querying and filtering.

use std::collections::{HashMap, BTreeMap};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};

use causality_types::ContentId;
use crate::resource::Resource;
use crate::resource_types::ResourceType;
use super::{QueryError, FilterExpression, FilterCondition, FilterOperator};
use crate::resource::query::filter::FilterValue;

/// Key for indexing resources
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IndexKey {
    /// Field path to index (e.g., "metadata.created_at")
    pub field: String,
    
    /// Index type
    pub index_type: IndexType,
}

impl IndexKey {
    /// Create a new index key
    pub fn new(field: impl Into<String>, index_type: IndexType) -> Self {
        Self {
            field: field.into(),
            index_type,
        }
    }
    
    /// Create a unique index key
    pub fn unique(field: impl Into<String>) -> Self {
        Self::new(field, IndexType::Unique)
    }
    
    /// Create a non-unique index key
    pub fn non_unique(field: impl Into<String>) -> Self {
        Self::new(field, IndexType::NonUnique)
    }
    
    /// Create a text index key
    pub fn text(field: impl Into<String>) -> Self {
        Self::new(field, IndexType::Text)
    }
    
    /// Create a geo index key
    pub fn geo(field: impl Into<String>) -> Self {
        Self::new(field, IndexType::Geo)
    }
}

/// Types of indexes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IndexType {
    /// Unique index (one value maps to one resource)
    Unique,
    
    /// Non-unique index (one value maps to multiple resources)
    NonUnique,
    
    /// Text index (for text search)
    Text,
    
    /// Geospatial index
    Geo,
}

/// Entry in an index
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntry {
    /// Key of the indexed field
    pub key: String,
    
    /// Resource IDs that match this key
    pub resource_ids: Vec<ContentId>,
}

impl IndexEntry {
    /// Create a new index entry
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            resource_ids: Vec::new(),
        }
    }
    
    /// Add a resource ID to this entry
    pub fn add_resource(&mut self, resource_id: ContentId) {
        // Add only if not already present
        if !self.resource_ids.contains(&resource_id) {
            self.resource_ids.push(resource_id);
        }
    }
    
    /// Remove a resource ID from this entry
    pub fn remove_resource(&mut self, resource_id: &ContentId) {
        self.resource_ids.retain(|id| id != resource_id);
    }
    
    /// Check if this entry is empty
    pub fn is_empty(&self) -> bool {
        self.resource_ids.is_empty()
    }
}

/// Interface for resource indexing
pub trait ResourceIndex: Send + Sync + Debug {
    /// Add a resource to the index
    fn add_resource(&self, resource: &dyn Resource) -> Result<(), QueryError>;
    
    /// Update a resource in the index
    fn update_resource(&self, resource: &dyn Resource) -> Result<(), QueryError>;
    
    /// Remove a resource from the index
    fn remove_resource(&self, resource_id: &ContentId) -> Result<(), QueryError>;
    
    /// Find resources by filter expression
    fn find_resources(&self, filter: &FilterExpression) -> Result<Vec<ContentId>, QueryError>;
    
    /// Get resource by ID
    fn get_resource(&self, resource_id: &ContentId) -> Result<Option<ContentId>, QueryError>;
    
    /// Clear the index
    fn clear(&self) -> Result<(), QueryError>;
    
    /// Get statistics about the index
    fn stats(&self) -> Result<IndexStats, QueryError>;
}

/// Statistics about a resource index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// Total number of resources indexed
    pub total_resources: usize,
    
    /// Number of indexes
    pub index_count: usize,
    
    /// Counts per resource type
    pub resource_type_counts: HashMap<ResourceType, usize>,
    
    /// Counts per index key
    pub index_entry_counts: HashMap<String, usize>,
}

/// In-memory implementation of ResourceIndex
#[derive(Debug)]
pub struct InMemoryResourceIndex {
    /// Resources by ID
    resources: RwLock<HashMap<ContentId, Arc<dyn Resource + Send + Sync>>>,
    
    /// Resources by type
    resources_by_type: RwLock<HashMap<ResourceType, Vec<ContentId>>>,
    
    /// Indexes
    indexes: RwLock<HashMap<IndexKey, BTreeMap<String, IndexEntry>>>,
}

impl InMemoryResourceIndex {
    /// Create a new in-memory resource index
    pub fn new() -> Self {
        Self {
            resources: RwLock::new(HashMap::new()),
            resources_by_type: RwLock::new(HashMap::new()),
            indexes: RwLock::new(HashMap::new()),
        }
    }
    
    /// Create a new index
    pub fn create_index(&self, key: IndexKey) -> Result<(), QueryError> {
        let mut indexes = self.indexes.write().map_err(|_| 
            QueryError::IndexError("Failed to acquire write lock on indexes".to_string())
        )?;
        
        if indexes.contains_key(&key) {
            return Err(QueryError::IndexError(
                format!("Index already exists for key: {:?}", key)
            ));
        }
        
        indexes.insert(key, BTreeMap::new());
        
        Ok(())
    }
    
    /// Drop an index
    pub fn drop_index(&self, key: &IndexKey) -> Result<(), QueryError> {
        let mut indexes = self.indexes.write().map_err(|_| 
            QueryError::IndexError("Failed to acquire write lock on indexes".to_string())
        )?;
        
        if !indexes.contains_key(key) {
            return Err(QueryError::IndexError(
                format!("Index does not exist for key: {:?}", key)
            ));
        }
        
        indexes.remove(key);
        
        Ok(())
    }
    
    /// Extract value from a resource for indexing
    fn extract_index_value(resource: &dyn Resource, field: &str) -> Result<String, QueryError> {
        // Use direct Resource trait methods instead of serializing to JSON
        if field == "id" {
            return Ok(resource.id().to_string());
        } else if field == "type" || field == "resource_type" {
            return Ok(resource.resource_type().to_string());
        } else if field == "state" {
            return Ok(resource.state().to_string());
        } else if field.starts_with("metadata.") {
            let metadata_key = field.strip_prefix("metadata.").unwrap_or(field);
            match resource.get_metadata(metadata_key) {
                Some(value) => return Ok(value),
                None => return Ok("null".to_string()),
            }
        }
        
        // For other fields, we can't extract without serialization
        // Return an appropriate error or default value
        Err(QueryError::FieldNotFound(format!("Cannot extract non-standard field: {}", field)))
    }
    
    /// Add a resource to an index
    fn add_to_index(
        &self,
        key: &IndexKey,
        resource: &dyn Resource,
    ) -> Result<(), QueryError> {
        let mut indexes = self.indexes.write().map_err(|_| 
            QueryError::IndexError("Failed to acquire write lock on indexes".to_string())
        )?;
        
        let index = indexes.entry(key.clone()).or_insert_with(BTreeMap::new);
        
        // Extract the value to index
        let value = Self::extract_index_value(resource, &key.field)?;
        
        // Add the resource to the index entry
        let entry = index.entry(value).or_insert_with(|| 
            IndexEntry::new(key.field.clone())
        );
        
        if key.index_type == IndexType::Unique && !entry.resource_ids.is_empty() {
            return Err(QueryError::IndexError(
                format!("Unique constraint violation for index: {:?}", key)
            ));
        }
        
        entry.add_resource(resource.id().clone().into());
        
        Ok(())
    }
    
    /// Remove a resource from an index
    fn remove_from_index(
        &self,
        key: &IndexKey,
        resource_id: &ContentId,
        value: Option<&str>,
    ) -> Result<(), QueryError> {
        let mut indexes = self.indexes.write().map_err(|_| 
            QueryError::IndexError("Failed to acquire write lock on indexes".to_string())
        )?;
        
        let index = match indexes.get_mut(key) {
            Some(idx) => idx,
            None => return Ok(()), // Index doesn't exist, nothing to do
        };
        
        if let Some(value) = value {
            // Remove from specific index entry if value is known
            if let Some(entry) = index.get_mut(value) {
                entry.remove_resource(resource_id);
                
                // Remove empty entries
                if entry.is_empty() {
                    index.remove(value);
                }
            }
        } else {
            // Scan all entries if value is not known
            let mut empty_keys = Vec::new();
            
            for (key, entry) in index.iter_mut() {
                entry.remove_resource(resource_id);
                
                if entry.is_empty() {
                    empty_keys.push(key.clone());
                }
            }
            
            // Remove empty entries
            for key in empty_keys {
                index.remove(&key);
            }
        }
        
        Ok(())
    }
    
    /// Find resources by index key and value
    fn find_by_index(
        &self,
        key: &IndexKey,
        value: &str,
    ) -> Result<Vec<ContentId>, QueryError> {
        let indexes = self.indexes.read().map_err(|_| 
            QueryError::IndexError("Failed to acquire read lock on indexes".to_string())
        )?;
        
        let index = match indexes.get(key) {
            Some(idx) => idx,
            None => return Ok(Vec::new()), // Index doesn't exist, return empty
        };
        
        // Find the index entry
        match index.get(value) {
            Some(entry) => Ok(entry.resource_ids.clone()),
            None => Ok(Vec::new()),
        }
    }
    
    /// Find resources by range query
    fn find_by_range(
        &self,
        key: &IndexKey,
        start: Option<&str>,
        end: Option<&str>,
        include_start: bool,
        include_end: bool,
    ) -> Result<Vec<ContentId>, QueryError> {
        let indexes = self.indexes.read().map_err(|_| 
            QueryError::IndexError("Failed to acquire read lock on indexes".to_string())
        )?;
        
        let index = match indexes.get(key) {
            Some(idx) => idx,
            None => return Ok(Vec::new()), // Index doesn't exist, return empty
        };
        
        let mut results = Vec::new();
        
        // Determine the range boundaries
        let range = match (start, end) {
            (Some(start), Some(end)) => {
                if include_start && include_end {
                    index.range::<String, _>(&start.to_string()..=&end.to_string())
                } else if include_start {
                    index.range::<String, _>(&start.to_string()..)
                } else if include_end {
                    index.range::<String, _>(..=&end.to_string())
                } else {
                    index.range::<String, _>(&start.to_string()..&end.to_string())
                }
            },
            (Some(start), None) => {
                if include_start {
                    index.range::<String, _>(&start.to_string()..)
                } else {
                    index.range::<String, _>(&start.to_string()..)
                }
            },
            (None, Some(end)) => {
                if include_end {
                    index.range::<String, _>(..=&end.to_string())
                } else {
                    index.range::<String, _>(..&end.to_string())
                }
            },
            (None, None) => index.range::<String, _>(..)
        };
        
        // Collect all resource IDs in the range
        for (_, entry) in range {
            results.extend(entry.resource_ids.clone());
        }
        
        Ok(results)
    }
}

impl ResourceIndex for InMemoryResourceIndex {
    fn add_resource(&self, resource: &dyn Resource) -> Result<(), QueryError> {
        // Add to resources map
        let mut resources = self.resources.write().map_err(|_| 
            QueryError::IndexError("Failed to acquire write lock on resources".to_string())
        )?;
        
        let resource_id = resource.id().clone();
        let resource_type = resource.resource_type().clone();
        
        // Add resource to resources map
        resources.insert(
            resource_id.clone().into(),
            Arc::new(resource.clone_resource())
        );
        
        // Add to resources_by_type map
        let mut resources_by_type = self.resources_by_type.write().map_err(|_| 
            QueryError::IndexError("Failed to acquire write lock on resources_by_type".to_string())
        )?;
        
        resources_by_type
            .entry(resource_type)
            .or_insert_with(Vec::new)
            .push(resource_id.clone().into());
        
        // Add to all indexes
        let indexes = self.indexes.read().map_err(|_| 
            QueryError::IndexError("Failed to acquire read lock on indexes".to_string())
        )?;
        
        for key in indexes.keys() {
            self.add_to_index(key, resource)?;
        }
        
        Ok(())
    }
    
    fn update_resource(&self, resource: &dyn Resource) -> Result<(), QueryError> {
        // Get the old resource for comparison
        let resources = self.resources.read().map_err(|_| 
            QueryError::IndexError("Failed to acquire read lock on resources".to_string())
        )?;
        
        let resource_id = resource.id();
        let content_id = resource_id.clone().into();
        let old_resource = resources.get(&content_id).cloned();
        
        // Remove from indexes and add back with new values
        drop(resources); // Release read lock before acquiring write lock
        
        // Remove old resource from indexes
        if let Some(old_resource) = old_resource {
            let indexes = self.indexes.read().map_err(|_| 
                QueryError::IndexError("Failed to acquire read lock on indexes".to_string())
            )?;
            
            for key in indexes.keys() {
                let old_value = Self::extract_index_value(old_resource.as_ref(), &key.field).ok();
                self.remove_from_index(key, &content_id, old_value.as_deref())?;
            }
        }
        
        // Add the new resource
        self.add_resource(resource)?;
        
        Ok(())
    }
    
    fn remove_resource(&self, resource_id: &ContentId) -> Result<(), QueryError> {
        // Get the resource for index values
        let resources = self.resources.read().map_err(|_| 
            QueryError::IndexError("Failed to acquire read lock on resources".to_string())
        )?;
        
        let resource = resources.get(resource_id).cloned();
        drop(resources); // Release read lock before acquiring write lock
        
        // Remove from resources map
        let mut resources = self.resources.write().map_err(|_| 
            QueryError::IndexError("Failed to acquire write lock on resources".to_string())
        )?;
        
        if let Some(removed) = resources.remove(resource_id) {
            // Remove from resources_by_type map
            let mut resources_by_type = self.resources_by_type.write().map_err(|_| 
                QueryError::IndexError("Failed to acquire write lock on resources_by_type".to_string())
            )?;
            
            let resource_type = removed.resource_type().clone();
            if let Some(ids) = resources_by_type.get_mut(&resource_type) {
                ids.retain(|id| id != resource_id);
                
                if ids.is_empty() {
                    resources_by_type.remove(&resource_type);
                }
            }
            
            // Remove from all indexes
            if let Some(resource) = resource {
                let indexes = self.indexes.read().map_err(|_| 
                    QueryError::IndexError("Failed to acquire read lock on indexes".to_string())
                )?;
                
                for key in indexes.keys() {
                    let value = Self::extract_index_value(resource.as_ref(), &key.field).ok();
                    self.remove_from_index(key, resource_id, value.as_deref())?;
                }
            }
        }
        
        Ok(())
    }
    
    fn find_resources(&self, filter: &FilterExpression) -> Result<Vec<ContentId>, QueryError> {
        match filter {
            FilterExpression::Condition(condition) => {
                self.find_by_condition(condition)
            },
            FilterExpression::And(left, right) => {
                // Find resources that match both expressions
                let left_results = self.find_resources(left)?;
                let right_results = self.find_resources(right)?;
                
                // Intersection of results (resources that match both)
                Ok(left_results.into_iter()
                    .filter(|id| right_results.contains(id))
                    .collect())
            },
            FilterExpression::Or(left, right) => {
                // Find resources that match either expression
                let left_results = self.find_resources(left)?;
                let right_results = self.find_resources(right)?;
                
                // Union of results (deduplicated)
                let mut results = left_results;
                
                // Add unique IDs from right_results
                for id in right_results {
                    if !results.contains(&id) {
                        results.push(id);
                    }
                }
                
                Ok(results)
            },
            FilterExpression::Not(expression) => {
                let matching = self.find_resources(expression)?;
                let all = self.get_all_resources()?;
                
                Ok(all.into_iter()
                    .filter(|id| !matching.contains(id))
                    .collect())
            },
        }
    }
    
    fn get_resource(&self, resource_id: &ContentId) -> Result<Option<ContentId>, QueryError> {
        let resources = self.resources.read().map_err(|_| 
            QueryError::IndexError("Failed to acquire read lock on resources".to_string())
        )?;
        
        Ok(resources.get(resource_id).map(|_| resource_id.clone()))
    }
    
    fn clear(&self) -> Result<(), QueryError> {
        // Clear resources map
        let mut resources = self.resources.write().map_err(|_| 
            QueryError::IndexError("Failed to acquire write lock on resources".to_string())
        )?;
        resources.clear();
        
        // Clear resources_by_type map
        let mut resources_by_type = self.resources_by_type.write().map_err(|_| 
            QueryError::IndexError("Failed to acquire write lock on resources_by_type".to_string())
        )?;
        resources_by_type.clear();
        
        // Clear all indexes
        let mut indexes = self.indexes.write().map_err(|_| 
            QueryError::IndexError("Failed to acquire write lock on indexes".to_string())
        )?;
        
        for (_, index) in indexes.iter_mut() {
            index.clear();
        }
        
        Ok(())
    }
    
    fn stats(&self) -> Result<IndexStats, QueryError> {
        let resources = self.resources.read().map_err(|_| 
            QueryError::IndexError("Failed to acquire read lock on resources".to_string())
        )?;
        
        let resources_by_type = self.resources_by_type.read().map_err(|_| 
            QueryError::IndexError("Failed to acquire read lock on resources_by_type".to_string())
        )?;
        
        let indexes = self.indexes.read().map_err(|_| 
            QueryError::IndexError("Failed to acquire read lock on indexes".to_string())
        )?;
        
        let mut resource_type_counts = HashMap::new();
        for (rtype, ids) in resources_by_type.iter() {
            resource_type_counts.insert(rtype.clone(), ids.len());
        }
        
        let mut index_entry_counts = HashMap::new();
        for (key, index) in indexes.iter() {
            index_entry_counts.insert(key.field.clone(), index.len());
        }
        
        Ok(IndexStats {
            total_resources: resources.len(),
            index_count: indexes.len(),
            resource_type_counts,
            index_entry_counts,
        })
    }
}

impl InMemoryResourceIndex {
    /// Get all resource IDs
    fn get_all_resources(&self) -> Result<Vec<ContentId>, QueryError> {
        let resources = self.resources.read().map_err(|_| 
            QueryError::IndexError("Failed to acquire read lock on resources".to_string())
        )?;
        
        Ok(resources.keys().cloned().collect())
    }
    
    /// Find resources by condition
    fn find_by_condition(&self, condition: &FilterCondition) -> Result<Vec<ContentId>, QueryError> {
        // Check if we have an index for this field
        let index_key = IndexKey::new(&condition.field, IndexType::NonUnique);
        
        // Try to use the index if available
        match condition.operator {
            FilterOperator::Equal => {
                if let FilterValue::String(ref value) = condition.value {
                    return self.find_by_index(&index_key, value);
                }
                // Fall back to scanning for other value types
            },
            FilterOperator::GreaterThan => {
                if let FilterValue::String(ref value) = condition.value {
                    return self.find_by_range(&index_key, Some(value), None, false, false);
                }
            },
            FilterOperator::GreaterThanOrEqual => {
                if let FilterValue::String(ref value) = condition.value {
                    return self.find_by_range(&index_key, Some(value), None, true, false);
                }
            },
            FilterOperator::LessThan => {
                if let FilterValue::String(ref value) = condition.value {
                    return self.find_by_range(&index_key, None, Some(value), false, false);
                }
            },
            FilterOperator::LessThanOrEqual => {
                if let FilterValue::String(ref value) = condition.value {
                    return self.find_by_range(&index_key, None, Some(value), false, true);
                }
            },
            _ => {
                // Other operators require scanning
            }
        }
        
        // Fall back to scanning all resources
        let filter = FilterExpression::Condition(condition.clone());
        self.scan_resources(&filter)
    }
    
    /// Find resources by scanning all resources
    fn scan_resources(&self, filter: &FilterExpression) -> Result<Vec<ContentId>, QueryError> {
        let resources = self.resources.read().map_err(|_| 
            QueryError::IndexError("Failed to acquire read lock on resources".to_string())
        )?;
        
        let mut results = Vec::new();
        
        for (id, resource) in resources.iter() {
            if filter.matches(resource.as_ref())? {
                results.push(id.clone());
            }
        }
        
        Ok(results)
    }
} 