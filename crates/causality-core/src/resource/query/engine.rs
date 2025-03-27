// Resource Query Engine
//
// This module provides the execution engine for resource queries,
// supporting filtering, sorting, and pagination of results.

use std::fmt::Debug;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use serde_json::Value;

use causality_types::ContentId;
use crate::resource::Resource;
use crate::resource_types::ResourceType;
use crate::capability::{Capability, Right};
use super::{
    ResourceQuery, FilterExpression, FilterCondition, FilterOperator,
    QueryError, QueryResult,
    Sort, SortDirection, Pagination, PaginationResult,
    ResourceIndex, InMemoryResourceIndex
};

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
    async fn query<R: Resource + Send + Sync + 'static>(
        &self,
        query: &ResourceQuery,
        capability: Option<&Capability>,
        options: Option<QueryOptions>,
    ) -> QueryResult<QueryExecution<R>>;
    
    /// Get a resource by ID
    async fn get_resource<R: Resource + Send + Sync + 'static>(
        &self,
        resource_id: &ContentId,
        capability: Option<&Capability>,
    ) -> QueryResult<Option<R>>;
    
    /// Count resources matching a query
    async fn count(
        &self,
        query: &ResourceQuery,
        capability: Option<&Capability>,
    ) -> QueryResult<usize>;
    
    /// Check if any resources match the query
    async fn exists(
        &self,
        query: &ResourceQuery,
        capability: Option<&Capability>,
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
        capability: Option<&Capability>,
    ) -> bool {
        match capability {
            Some(cap) => {
                // Check if the capability has read rights for this resource
                cap.has_right(&Right::Read) &&
                (cap.resource_id() == "*" || cap.resource_id() == resource_id.to_string())
            },
            None => false,
        }
    }
    
    /// Filter resources by capability
    fn filter_by_capability(
        &self,
        resource_ids: Vec<ContentId>,
        capability: Option<&Capability>,
    ) -> Vec<ContentId> {
        match capability {
            Some(cap) => {
                // If the capability has wildcard resource ID, return all
                if cap.resource_id() == "*" && cap.has_right(&Right::Read) {
                    return resource_ids;
                }
                
                // Filter by capability
                resource_ids.into_iter()
                    .filter(|id| self.can_read_resource(id, Some(cap)))
                    .collect()
            },
            None => {
                // No capability, return empty
                Vec::new()
            }
        }
    }
    
    /// Apply sorting to resources
    fn apply_sorting<R: Resource + Send + Sync>(
        &self,
        resources: Vec<R>,
        sorts: &[Sort],
    ) -> QueryResult<Vec<R>> {
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
            a.resource_id().to_string().cmp(&b.resource_id().to_string())
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
    async fn query<R: Resource + Send + Sync + 'static>(
        &self,
        query: &ResourceQuery,
        capability: Option<&Capability>,
        options: Option<QueryOptions>,
    ) -> QueryResult<QueryExecution<R>> {
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
            self.index.get_all_resources()?
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
            if let Some(res) = self.resource_retrievers.get_resource::<R>(&id).await? {
                resources.push(res);
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
    
    async fn get_resource<R: Resource + Send + Sync + 'static>(
        &self,
        resource_id: &ContentId,
        capability: Option<&Capability>,
    ) -> QueryResult<Option<R>> {
        // Check if the capability grants access
        if let Some(cap) = capability {
            if !self.can_read_resource(resource_id, Some(cap)) {
                return Err(QueryError::PermissionDenied(
                    format!("No permission to read resource: {}", resource_id)
                ));
            }
        }
        
        // Check if the resource exists
        match self.index.get_resource(resource_id)? {
            Some(_) => {
                // Retrieve the resource
                self.resource_retrievers.get_resource::<R>(resource_id).await
            },
            None => Ok(None),
        }
    }
    
    async fn count(
        &self,
        query: &ResourceQuery,
        capability: Option<&Capability>,
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
        capability: Option<&Capability>,
    ) -> QueryResult<bool> {
        let mut options = QueryOptions::default();
        options.max_resources = Some(1);
        
        // Use a dummy resource type for checking existence
        let result = self.query::<DummyResource>(query, capability, Some(options)).await?;
        
        Ok(!result.resources.is_empty())
    }
}

/// Provider for retrieving resources
#[async_trait]
pub trait ResourceProvider: Debug {
    /// Get a resource by ID
    async fn get_resource<R: Resource + Send + Sync + 'static>(
        &self,
        resource_id: &ContentId,
    ) -> QueryResult<Option<R>>;
    
    /// Get resources by IDs
    async fn get_resources<R: Resource + Send + Sync + 'static>(
        &self,
        resource_ids: &[ContentId],
    ) -> QueryResult<Vec<R>>;
}

/// Default resource provider implementation
#[derive(Debug, Default)]
pub struct DefaultResourceProvider {
    // This implementation is a placeholder
    // In a real implementation, this would have storage backends
}

impl DefaultResourceProvider {
    /// Create a new default resource provider
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ResourceProvider for DefaultResourceProvider {
    async fn get_resource<R: Resource + Send + Sync + 'static>(
        &self,
        _resource_id: &ContentId,
    ) -> QueryResult<Option<R>> {
        // This is a placeholder implementation
        // In a real implementation, this would retrieve from storage
        Ok(None)
    }
    
    async fn get_resources<R: Resource + Send + Sync + 'static>(
        &self,
        resource_ids: &[ContentId],
    ) -> QueryResult<Vec<R>> {
        let mut resources = Vec::new();
        
        for id in resource_ids {
            if let Some(resource) = self.get_resource::<R>(id).await? {
                resources.push(resource);
            }
        }
        
        Ok(resources)
    }
}

/// Dummy resource for use in count/exists queries
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DummyResource {
    id: ContentId,
    resource_type: ResourceType,
}

impl Resource for DummyResource {
    fn resource_id(&self) -> &ContentId {
        &self.id
    }
    
    fn resource_type(&self) -> &ResourceType {
        &self.resource_type
    }
    
    fn clone_resource(&self) -> Box<dyn Resource + Send + Sync> {
        Box::new(self.clone())
    }
} 