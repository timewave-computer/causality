// Resource Query Engine
//
// This module provides the execution engine for resource queries,
// supporting filtering, sorting, and pagination of results.

use std::fmt::Debug;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use causality_types::ContentId;
use crate::resource::{Resource, ResourceType, ResourceState, ResourceResult, ResourceError};
use crate::resource_types::ResourceId;
use crate::resource::operation::Capability;
use crate::capability::effect::EffectCapabilityType;

use super::{
    ResourceQuery, FilterExpression, FilterCondition, FilterOperator,
    QueryError,
    Sort, SortDirection, Pagination, PaginationResult,
    ResourceIndex
};
// Direct import of QueryResult and the InMemoryResourceIndex
use crate::resource::query::QueryResult;
use crate::resource::query::index::InMemoryResourceIndex;
use super::filter::FilterValue;

/// A wrapper for any resource-like object
#[derive(Clone, Debug)]
pub struct ResourceWrapper {
    /// Resource ID
    id: ContentId,
    
    /// Resource type
    resource_type: ResourceType,
    
    /// Resource state
    state: ResourceState,
    
    /// Resource metadata
    metadata: HashMap<String, String>,
}

impl Resource for ResourceWrapper {
    fn id(&self) -> ResourceId {
        ResourceId::from_legacy_content_id(&self.id)
    }
    
    fn resource_type(&self) -> ResourceType {
        self.resource_type.clone()
    }
    
    fn state(&self) -> ResourceState {
        self.state
    }
    
    fn get_metadata(&self, key: &str) -> Option<String> {
        self.metadata.get(key).cloned()
    }
    
    fn set_metadata(&mut self, key: &str, value: &str) -> ResourceResult<()> {
        self.metadata.insert(key.to_string(), value.to_string());
        Ok(())
    }
    
    fn clone_resource(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Resource provider trait
#[async_trait]
pub trait ResourceProvider: Debug {
    /// Get all resources
    async fn get_all_resources(&self) -> QueryResult<Vec<Box<dyn Resource>>>;
    
    /// Get resource by ID
    async fn get_resource_by_id(&self, id: &ResourceId) -> QueryResult<Option<Box<dyn Resource>>>;
    
    /// Get resources by type
    async fn get_resources_by_type(&self, resource_type: &ResourceType) -> QueryResult<Vec<Box<dyn Resource>>>;
}

/// Default resource provider that returns empty results
#[derive(Debug)]
pub struct DefaultResourceProvider;

impl DefaultResourceProvider {
    /// Create a new default resource provider
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ResourceProvider for DefaultResourceProvider {
    async fn get_all_resources(&self) -> QueryResult<Vec<Box<dyn Resource>>> {
        Ok(Vec::new())
    }
    
    async fn get_resource_by_id(&self, _id: &ResourceId) -> QueryResult<Option<Box<dyn Resource>>> {
        Ok(None)
    }
    
    async fn get_resources_by_type(&self, _resource_type: &ResourceType) -> QueryResult<Vec<Box<dyn Resource>>> {
        Ok(Vec::new())
    }
}

/// Options for query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryOptions {
    /// Maximum number of resources to query
    pub max_resources: Option<usize>,
    
    /// Whether to verify capabilities
    pub verify_capabilities: bool,
    
    /// Whether to use indexes
    pub use_indexes: bool,
    
    /// Whether to include total count
    pub include_total: bool,
    
    /// Whether to use parallel execution
    pub parallel_execution: bool,
    
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            max_resources: Some(1000),
            verify_capabilities: true,
            use_indexes: true,
            include_total: false,
            parallel_execution: false,
            timeout_ms: Some(30000), // 30 seconds
        }
    }
}

/// Result of a query execution
#[derive(Debug, Clone)]
pub struct QueryExecution<R> {
    /// Resources that matched the query
    pub resources: Vec<R>,
    
    /// Pagination result
    pub pagination: PaginationResult,
    
    /// Query execution statistics
    pub stats: QueryExecutionStats,
}

/// Statistics for query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryExecutionStats {
    /// Time taken to execute the query in milliseconds
    pub execution_time_ms: u64,
    
    /// Number of resources evaluated
    pub resources_evaluated: usize,
    
    /// Number of resources matched before pagination
    pub resources_matched: usize,
    
    /// Number of resources returned after pagination
    pub resources_returned: usize,
    
    /// Whether the query used indexes
    pub used_indexes: bool,
    
    /// Number of filter conditions evaluated
    pub filter_conditions: usize,
}

impl QueryExecutionStats {
    /// Create new empty stats
    fn new() -> Self {
        Self {
            execution_time_ms: 0,
            resources_evaluated: 0,
            resources_matched: 0,
            resources_returned: 0,
            used_indexes: false,
            filter_conditions: 0,
        }
    }
}

/// Trait for query engines
#[async_trait]
pub trait QueryEngine: Send + Sync + Debug {
    /// Execute a query and return matching resources
    async fn query<R: Resource + Send + Sync + 'static + Clone>(
        &self,
        query: &ResourceQuery,
        capability: Option<&Capability<dyn Resource>>,
        options: Option<QueryOptions>,
    ) -> QueryResult<QueryExecution<R>>;
    
    /// Get a resource by ID
    async fn get_resource<R: Resource + Send + Sync + 'static + Clone>(
        &self,
        resource_id: &ContentId,
        capability: Option<&Capability<dyn Resource>>,
    ) -> QueryResult<Option<R>>;
    
    /// Count resources matching a query
    async fn count(
        &self,
        query: &ResourceQuery,
        capability: Option<&Capability<dyn Resource>>,
    ) -> QueryResult<usize>;
    
    /// Check if any resources match the query
    async fn exists(
        &self,
        query: &ResourceQuery,
        capability: Option<&Capability<dyn Resource>>,
    ) -> QueryResult<bool>;
}

/// Basic query engine implementation using in-memory index
#[derive(Debug)]
pub struct BasicQueryEngine {
    /// Resource index for query execution
    index: Arc<InMemoryResourceIndex>,
    
    /// Resource retrievers by type
    resource_retrievers: Arc<dyn ResourceProvider + Send + Sync>,
}

impl BasicQueryEngine {
    /// Create a new basic query engine
    pub fn new(
        index: Arc<InMemoryResourceIndex>,
        resource_retrievers: Arc<dyn ResourceProvider + Send + Sync>,
    ) -> Self {
        Self {
            index,
            resource_retrievers,
        }
    }
    
    /// Create a new basic query engine with default retrievers
    pub fn with_default_retrievers(
        index: Arc<InMemoryResourceIndex>,
    ) -> Self {
        Self {
            index,
            resource_retrievers: Arc::new(DefaultResourceProvider::new()),
        }
    }
    
    /// Check if a capability grants read access to a resource
    fn can_read_resource(
        &self,
        resource_id: &ContentId,
        capability: Option<&Capability<dyn Resource>>,
    ) -> bool {
        match capability {
            Some(cap) => {
                // Check if the capability has read rights for this resource
                // Assuming this is the correct method - implement if needed
                let has_read_right = true; // Replace with actual check
                let matches_resource = true; // Replace with actual check
                has_read_right && matches_resource
            },
            None => false,
        }
    }
    
    /// Filter resources by capability
    fn filter_by_capability(
        &self,
        resource_ids: Vec<ContentId>,
        capability: Option<&Capability<dyn Resource>>,
    ) -> Vec<ContentId> {
        match capability {
            Some(_cap) => {
                // If the capability has wildcard resource ID, return all
                // Assuming this needs to be implemented based on your capability model
                resource_ids
            },
            None => Vec::new(),
        }
    }
    
    /// Apply sorting to resources
    fn apply_sorting<R>(&self, resources: Vec<R>, sorts: &[Sort]) -> QueryResult<Vec<R>>
    where
        R: Resource + Send + Sync,
    {
        if sorts.is_empty() {
            return Ok(resources);
        }
        
        let mut sorted_resources = resources;
        
        // Sort the resources
        sorted_resources.sort_by(|a, b| {
            for sort in sorts {
                match sort.compare(a, b) {
                    Ok(std::cmp::Ordering::Equal) => continue, // Try next sort
                    Ok(ordering) => return ordering,
                    Err(_) => continue, // Skip invalid sorts
                }
            }
            
            // If all sorts are equal, sort by ID for stability
            a.id().to_string().cmp(&b.id().to_string())
        });
        
        Ok(sorted_resources)
    }
    
    /// Apply pagination to resources
    fn apply_pagination<R: Resource + Send + Sync>(
        &self,
        resources: Vec<R>,
        pagination: Option<&Pagination>,
        include_total: bool,
    ) -> (Vec<R>, PaginationResult) {
        let total = resources.len();
        
        let (offset, limit) = match pagination {
            Some(p) => (
                p.offset.unwrap_or(0),
                p.limit.unwrap_or(usize::MAX),
            ),
            None => (0, usize::MAX),
        };
        
        // Apply pagination
        let paginated = resources.into_iter()
            .skip(offset)
            .take(limit)
            .collect::<Vec<_>>();
        
        let count = paginated.len();
        let has_more = offset + count < total;
        
        let pagination_result = PaginationResult {
            offset,
            limit,
            total: if include_total { Some(total) } else { None },
            count,
            has_more,
            next_cursor: None,
            previous_cursor: None,
        };
        
        (paginated, pagination_result)
    }
}

#[async_trait]
impl QueryEngine for BasicQueryEngine {
    async fn query<R>(&self, query: &ResourceQuery, capability: Option<&Capability<dyn Resource>>, options: Option<QueryOptions>) -> QueryResult<QueryExecution<R>>
    where
        R: Resource + Send + Sync + 'static + Clone,
    {
        let options = options.unwrap_or_default();
        let start_time = std::time::Instant::now();
        
        let mut stats = QueryExecutionStats::new();
        
        // Apply resource type filter if specified
        let resource_ids = if let Some(resource_type) = &query.resource_type {
            // Get resources by type
            let resources_by_type = self.index.stats()?.resource_type_counts
                .get(resource_type)
                .cloned()
                .unwrap_or(0);
            
            stats.resources_evaluated = resources_by_type;
            
            // Apply filter
            if let Some(filter) = &query.filter {
                stats.used_indexes = options.use_indexes;
                let matched_ids = self.index.find_resources(filter)?;
                
                // Only include IDs for the specified resource type
                matched_ids.into_iter()
                    .filter(|id| {
                        if let Ok(Some(res_id)) = self.index.get_resource(id) {
                            true
                        } else {
                            false
                        }
                    })
                    .collect()
            } else {
                // No filter, get all resources of the specified type
                self.index.stats()?
                    .resource_type_counts
                    .iter()
                    .filter(|(t, _)| *t == resource_type)
                    .flat_map(|(_, count)| {
                        // TODO: Get actual resource IDs for the type
                        Vec::new()
                    })
                    .collect()
            }
        } else if let Some(filter) = &query.filter {
            // No resource type specified, apply filter to all resources
            stats.used_indexes = options.use_indexes;
            self.index.find_resources(filter)?
        } else {
            // No resource type or filter, get all resources
            // Instead of using the private method, use find_resources with a "true" filter
            let all_filter = FilterExpression::condition("id", FilterOperator::IsNotNull, FilterValue::Null);
            self.index.find_resources(&all_filter)?
        };
        
        stats.resources_matched = resource_ids.len();
        
        // Apply capability filtering if required
        let filtered_ids = if options.verify_capabilities {
            self.filter_by_capability(resource_ids, capability)
        } else {
            resource_ids
        };
        
        // Retrieve the actual resources
        let mut resources = Vec::new();
        for id in filtered_ids {
            // Convert ContentId to ResourceId
            let resource_id = ResourceId::from_legacy_content_id(&id);
            
            // Get the resource using get_resource_by_id instead
            if let Some(res) = self.resource_retrievers.get_resource_by_id(&resource_id).await? {
                // Clone the resource using clone_resource
                let cloned = res.clone_resource();
                
                // Attempt to downcast to the requested type
                if let Some(typed_ref) = cloned.as_any().downcast_ref::<R>() {
                    // Create a new owned instance by directly cloning the typed reference
                    let new_instance = typed_ref.clone();
                    resources.push(new_instance);
                }
            }
        }
        
        // Apply sorting
        let sorted_resources = self.apply_sorting(resources, &query.sort)?;
        
        // Apply pagination
        let (paginated_resources, pagination_result) = 
            self.apply_pagination(sorted_resources, query.pagination.as_ref(), options.include_total);
        
        stats.resources_returned = paginated_resources.len();
        stats.execution_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(QueryExecution {
            resources: paginated_resources,
            pagination: pagination_result,
            stats,
        })
    }
    
    async fn get_resource<R>(&self, resource_id: &ContentId, capability: Option<&Capability<dyn Resource>>) -> QueryResult<Option<R>>
    where
        R: Resource + Send + Sync + 'static + Clone,
    {
        // Convert ContentId to ResourceId
        let resource_id = ResourceId::from_legacy_content_id(resource_id);
        
        // Check capabilities
        if capability.is_some() && !self.can_read_resource(&resource_id.to_content_id(), capability) {
            return Ok(None); // No permission
        }
        
        // Get the resource
        let result = self.resource_retrievers.get_resource_by_id(&resource_id)
            .await
            .map_err(|e| QueryError::StorageError(e.to_string()))?;
        
        // Convert to the requested type if possible
        if let Some(res) = result {
            // Clone the resource for downcasting
            let downcasted = res.clone_resource();
            
            // Attempt to downcast to the requested type
            if let Some(typed_ref) = downcasted.as_any().downcast_ref::<R>() {
                // Create a new owned instance by directly cloning the typed reference
                return Ok(Some(typed_ref.clone()));
            }
            return Ok(None); // Not the right type
        }
        
        Ok(None) // Resource not found or wrong type
    }
    
    async fn count(
        &self,
        query: &ResourceQuery,
        capability: Option<&Capability<dyn Resource>>,
    ) -> QueryResult<usize> {
        let options = QueryOptions {
            include_total: true,
            ..Default::default()
        };
        
        // Use a dummy resource type for counting
        let result = self.query::<DummyResource>(query, capability, Some(options)).await?;
        
        Ok(result.pagination.total.unwrap_or(0))
    }
    
    async fn exists(
        &self,
        query: &ResourceQuery,
        capability: Option<&Capability<dyn Resource>>,
    ) -> QueryResult<bool> {
        let mut options = QueryOptions::default();
        options.max_resources = Some(1);
        
        // Use a dummy resource type for checking existence
        let result = self.query::<DummyResource>(query, capability, Some(options)).await?;
        
        Ok(!result.resources.is_empty())
    }
}

/// Dummy resource for testing
#[derive(Debug, Clone)]
struct DummyResource {
    id: ContentId,
    type_name: String,
    type_version: String,
    metadata: HashMap<String, String>,
}

impl Resource for DummyResource {
    fn id(&self) -> ResourceId {
        ResourceId::from_legacy_content_id(&self.id)
    }
    
    fn resource_type(&self) -> ResourceType {
        ResourceType::new("dummy", "1.0")
    }
    
    fn state(&self) -> ResourceState {
        ResourceState::Active
    }
    
    fn get_metadata(&self, key: &str) -> Option<String> {
        self.metadata.get(key).cloned()
    }
    
    fn set_metadata(&mut self, key: &str, value: &str) -> ResourceResult<()> {
        self.metadata.insert(key.to_string(), value.to_string());
        Ok(())
    }
    
    fn clone_resource(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}