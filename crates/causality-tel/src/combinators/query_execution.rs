//! Query Execution Module for TEL
//!
//! This module implements query execution functionality for the TEL query system,
//! including distributed query execution, query results caching, and execution plan 
//! optimization. It extends the basic query capabilities defined in the query.rs file.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use async_trait::async_trait;
use tokio::time::{timeout, Instant};
use causality_types::crypto_primitives::ContentId;
use causality_types::domain::DomainId;

use super::query::{Query, QueryResult, QueryExecutor, QueryExecutionResult, QueryError};
use crate::combinators::Literal;

/// A cache entry for query results
struct QueryCacheEntry {
    /// The cached result
    result: QueryResult,
    /// When this entry was created
    created_at: Instant,
    /// When this entry expires
    expires_at: Instant,
    /// The content ID of the query
    query_id: ContentId,
}

/// Cache for query results to avoid redundant queries
pub struct QueryCache {
    /// The cached query results 
    cache: RwLock<HashMap<ContentId, QueryCacheEntry>>,
    /// Default TTL for cache entries in seconds
    default_ttl: u64,
}

impl QueryCache {
    /// Create a new query cache with the given TTL (in seconds)
    pub fn new(default_ttl: u64) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            default_ttl,
        }
    }
    
    /// Get a cached result for a query, if available
    pub fn get(&self, query_id: &ContentId) -> Option<QueryResult> {
        let now = Instant::now();
        
        if let Ok(cache) = self.cache.read() {
            if let Some(entry) = cache.get(query_id) {
                if now < entry.expires_at {
                    return Some(entry.result.clone());
                }
            }
        }
        
        None
    }
    
    /// Cache a query result
    pub fn put(&self, query_id: ContentId, result: QueryResult, ttl: Option<u64>) {
        let now = Instant::now();
        let ttl = ttl.unwrap_or(self.default_ttl);
        let expires_at = now + Duration::from_secs(ttl);
        
        let entry = QueryCacheEntry {
            result,
            created_at: now,
            expires_at,
            query_id: query_id.clone(),
        };
        
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(query_id, entry);
        }
    }
    
    /// Clear expired entries from the cache
    pub fn clear_expired(&self) {
        let now = Instant::now();
        
        if let Ok(mut cache) = self.cache.write() {
            cache.retain(|_, entry| now < entry.expires_at);
        }
    }
    
    /// Clear the entire cache
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }
}

/// Represents a query execution plan
#[derive(Debug, Clone)]
pub struct QueryExecutionPlan {
    /// The query to execute
    query: Query,
    /// The steps to execute
    steps: Vec<QueryExecutionStep>,
    /// The estimated cost
    estimated_cost: f64,
}

/// A step in a query execution plan
#[derive(Debug, Clone)]
pub enum QueryExecutionStep {
    /// Scan a source directly
    Scan {
        source: String,
        domain: Option<String>,
    },
    /// Apply filters to an intermediate result
    Filter {
        filters: Vec<super::query::Filter>,
    },
    /// Apply projections to intermediate results
    Project {
        projections: Vec<super::query::Projection>,
    },
    /// Sort intermediate results
    Sort {
        sort_specs: Vec<super::query::SortSpec>,
    },
    /// Limit and offset intermediate results
    LimitOffset {
        limit: Option<usize>,
        offset: Option<usize>,
    },
    /// Perform aggregation on intermediate results
    Aggregate {
        operations: Vec<super::query::AggregationOperation>,
    },
    /// Merge results from multiple sources
    Merge {
        strategy: MergeStrategy,
    },
}

/// Strategy for merging results from multiple sources
#[derive(Debug, Clone)]
pub enum MergeStrategy {
    /// Union all results
    Union,
    /// Intersect results (only keep matching records)
    Intersect,
    /// Keep left side results with matching fields from right
    LeftJoin {
        left_field: String,
        right_field: String,
    },
    /// Return only matching records from both sides
    InnerJoin {
        left_field: String,
        right_field: String,
    },
}

/// A query planner generates efficient execution plans for queries
pub trait QueryPlanner: Send + Sync {
    /// Create an optimized execution plan for a query
    fn plan(&self, query: &Query) -> Result<QueryExecutionPlan, QueryError>;
    
    /// Estimate the cost of a query execution plan
    fn estimate_cost(&self, plan: &QueryExecutionPlan) -> f64;
}

/// A default implementation of the query planner
pub struct DefaultQueryPlanner;

impl DefaultQueryPlanner {
    /// Create a new default query planner
    pub fn new() -> Self {
        Self
    }
}

impl QueryPlanner for DefaultQueryPlanner {
    fn plan(&self, query: &Query) -> Result<QueryExecutionPlan, QueryError> {
        let mut steps = Vec::new();
        
        // Step 1: Scan the source
        steps.push(QueryExecutionStep::Scan {
            source: query.source.clone(),
            domain: query.domain.clone(),
        });
        
        // Step 2: Apply filters if present
        if !query.filters.is_empty() {
            steps.push(QueryExecutionStep::Filter {
                filters: query.filters.clone(),
            });
        }
        
        // Step 3: Apply projections if present
        if let Some(projections) = &query.projections {
            if !projections.is_empty() {
                steps.push(QueryExecutionStep::Project {
                    projections: projections.clone(),
                });
            }
        }
        
        // Step 4: Apply sorts if present
        if let Some(sorts) = &query.sorts {
            if !sorts.is_empty() {
                steps.push(QueryExecutionStep::Sort {
                    sort_specs: sorts.clone(),
                });
            }
        }
        
        // Step 5: Apply limit and offset if present
        if query.limit.is_some() || query.offset.is_some() {
            steps.push(QueryExecutionStep::LimitOffset {
                limit: query.limit,
                offset: query.offset,
            });
        }
        
        // Step 6: Apply aggregations if present
        if let Some(aggregations) = &query.aggregations {
            if !aggregations.is_empty() {
                steps.push(QueryExecutionStep::Aggregate {
                    operations: aggregations.clone(),
                });
            }
        }
        
        // Create the plan
        let plan = QueryExecutionPlan {
            query: query.clone(),
            steps,
            estimated_cost: 1.0, // Placeholder cost
        };
        
        // Estimate the cost
        let cost = self.estimate_cost(&plan);
        
        // Return the plan with estimated cost
        Ok(QueryExecutionPlan {
            query: query.clone(),
            steps: plan.steps,
            estimated_cost: cost,
        })
    }
    
    fn estimate_cost(&self, plan: &QueryExecutionPlan) -> f64 {
        // Simple cost model based on the number of steps
        // In a real implementation, this would use statistics and heuristics
        
        let mut cost = 1.0; // Base cost
        
        for step in &plan.steps {
            match step {
                QueryExecutionStep::Scan { .. } => {
                    cost *= 10.0; // Scans are relatively expensive
                }
                QueryExecutionStep::Filter { filters } => {
                    // Each filter adds some cost
                    cost += filters.len() as f64;
                }
                QueryExecutionStep::Sort { .. } => {
                    // Sorting is expensive (O(n log n))
                    cost *= 5.0;
                }
                QueryExecutionStep::Aggregate { .. } => {
                    // Aggregation requires processing all records
                    cost *= 3.0;
                }
                QueryExecutionStep::Merge { .. } => {
                    // Merging is very expensive
                    cost *= 20.0;
                }
                // Other steps have minimal cost
                _ => cost += 1.0,
            }
        }
        
        cost
    }
}

/// An executor for distributed queries across multiple domains
pub struct DistributedQueryExecutor {
    /// Executors for different domains
    executors: HashMap<DomainId, Arc<dyn QueryExecutor>>,
    /// Cache for query results
    cache: Arc<QueryCache>,
    /// Query planner
    planner: Box<dyn QueryPlanner>,
    /// Default timeout for queries in seconds
    timeout_secs: u64,
}

impl DistributedQueryExecutor {
    /// Create a new distributed query executor
    pub fn new(
        cache: Arc<QueryCache>,
        planner: Box<dyn QueryPlanner>,
        timeout_secs: u64,
    ) -> Self {
        Self {
            executors: HashMap::new(),
            cache,
            planner,
            timeout_secs,
        }
    }
    
    /// Register an executor for a domain
    pub fn register_executor(&mut self, domain_id: DomainId, executor: Arc<dyn QueryExecutor>) {
        self.executors.insert(domain_id, executor);
    }
    
    /// Execute a query with timeout
    pub async fn execute_query_with_timeout(
        &self,
        query: &Query,
        timeout_secs: Option<u64>,
    ) -> QueryExecutionResult {
        let timeout_secs = timeout_secs.unwrap_or(self.timeout_secs);
        let timeout_duration = Duration::from_secs(timeout_secs);
        
        // Try to execute with timeout
        match timeout(timeout_duration, self.execute_query_internal(query)).await {
            Ok(result) => result,
            Err(_) => Err(QueryError::Other("Query execution timed out".to_string())),
        }
    }
    
    /// Internal method to execute a query
    async fn execute_query_internal(&self, query: &Query) -> QueryExecutionResult {
        // Generate a content ID for this query for caching
        let query_bytes = serde_json::to_vec(query).map_err(|e| {
            QueryError::Other(format!("Failed to serialize query: {}", e))
        })?;
        
        let query_id = ContentId::from_bytes(&query_bytes);
        
        // Check cache first
        if let Some(cached_result) = self.cache.get(&query_id) {
            return Ok(cached_result);
        }
        
        // Generate an execution plan
        let plan = self.planner.plan(query)?;
        
        // Execute the plan
        let result = self.execute_plan(&plan).await?;
        
        // Cache the result
        self.cache.put(query_id, result.clone(), None);
        
        Ok(result)
    }
    
    /// Execute a query plan
    async fn execute_plan(&self, plan: &QueryExecutionPlan) -> QueryExecutionResult {
        // Find the appropriate executor based on the domain
        let domain = plan.query.domain.as_deref();
        
        // If no domain is specified, use a fallback strategy
        if domain.is_none() {
            // Choose an executor based on the source or other heuristics
            return self.execute_plan_steps(plan).await;
        }
        
        // Get the domain-specific executor
        let domain_id = DomainId::new(domain.unwrap());
        
        if let Some(executor) = self.executors.get(&domain_id) {
            // The domain has a dedicated executor, use it
            executor.execute_query(&plan.query)
        } else {
            // No dedicated executor, fall back to executing the plan steps
            self.execute_plan_steps(plan).await
        }
    }
    
    /// Execute the steps in a query plan
    async fn execute_plan_steps(&self, plan: &QueryExecutionPlan) -> QueryExecutionResult {
        // This is a simplified implementation - a real one would follow each step
        
        // Find any executor that can handle this query
        for executor in self.executors.values() {
            if executor.can_handle(&plan.query) {
                return executor.execute_query(&plan.query);
            }
        }
        
        // No executor found
        Err(QueryError::SourceNotFound(format!(
            "No executor found for source: {}",
            plan.query.source
        )))
    }
}

impl QueryExecutor for DistributedQueryExecutor {
    fn execute_query(&self, query: &Query) -> QueryExecutionResult {
        // For synchronous interface, we use a synchronous wrapper around the async execution
        // This isn't ideal for real use but works for the trait implementation
        tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime")
            .block_on(self.execute_query_internal(query))
    }
    
    fn can_handle(&self, query: &Query) -> bool {
        // Check if we have an executor for this domain
        if let Some(domain) = &query.domain {
            let domain_id = DomainId::new(domain);
            if self.executors.contains_key(&domain_id) {
                return true;
            }
        }
        
        // Otherwise, check if any executor can handle this query
        self.executors.values().any(|executor| executor.can_handle(query))
    }
    
    fn get_source_metadata(&self, source: &str) -> Result<HashMap<String, Literal>, QueryError> {
        // Combine metadata from all executors that know about this source
        let mut combined_metadata = HashMap::new();
        
        for executor in self.executors.values() {
            if let Ok(metadata) = executor.get_source_metadata(source) {
                combined_metadata.extend(metadata);
            }
        }
        
        if combined_metadata.is_empty() {
            Err(QueryError::SourceNotFound(format!("No metadata found for source: {}", source)))
        } else {
            Ok(combined_metadata)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combinators::query::{FilterOperator, SortDirection};
    
    struct MockQueryExecutor {
        can_handle_sources: Vec<String>,
        results: HashMap<String, QueryResult>,
    }
    
    impl MockQueryExecutor {
        fn new() -> Self {
            Self {
                can_handle_sources: vec![],
                results: HashMap::new(),
            }
        }
        
        fn with_source(mut self, source: &str) -> Self {
            self.can_handle_sources.push(source.to_string());
            self
        }
        
        fn with_result(mut self, source: &str, result: QueryResult) -> Self {
            self.results.insert(source.to_string(), result);
            self
        }
    }
    
    impl QueryExecutor for MockQueryExecutor {
        fn execute_query(&self, query: &Query) -> QueryExecutionResult {
            if let Some(result) = self.results.get(&query.source) {
                Ok(result.clone())
            } else {
                Err(QueryError::SourceNotFound(format!("No results for source: {}", query.source)))
            }
        }
        
        fn can_handle(&self, query: &Query) -> bool {
            self.can_handle_sources.contains(&query.source)
        }
        
        fn get_source_metadata(&self, source: &str) -> Result<HashMap<String, Literal>, QueryError> {
            if self.can_handle_sources.contains(&source.to_string()) {
                let mut metadata = HashMap::new();
                metadata.insert("type".to_string(), Literal::String("collection".to_string()));
                Ok(metadata)
            } else {
                Err(QueryError::SourceNotFound(format!("No metadata for source: {}", source)))
            }
        }
    }
    
    #[tokio::test]
    async fn test_distributed_query_executor() {
        // Create mock results
        let users_result = QueryResult {
            results: vec![
                {
                    let mut map = HashMap::new();
                    map.insert("id".to_string(), Literal::String("1".to_string()));
                    map.insert("name".to_string(), Literal::String("Alice".to_string()));
                    map
                }
            ],
            total_count: 1,
        };
        
        let products_result = QueryResult {
            results: vec![
                {
                    let mut map = HashMap::new();
                    map.insert("id".to_string(), Literal::String("101".to_string()));
                    map.insert("name".to_string(), Literal::String("Laptop".to_string()));
                    map
                }
            ],
            total_count: 1,
        };
        
        // Create mock executors
        let users_executor = Arc::new(
            MockQueryExecutor::new()
                .with_source("users")
                .with_result("users", users_result)
        );
        
        let products_executor = Arc::new(
            MockQueryExecutor::new()
                .with_source("products")
                .with_result("products", products_result)
        );
        
        // Create cache and planner
        let cache = Arc::new(QueryCache::new(60)); // 60 seconds TTL
        let planner = Box::new(DefaultQueryPlanner::new());
        
        // Create distributed executor
        let mut dist_executor = DistributedQueryExecutor::new(cache, planner, 10);
        
        // Register domain executors
        dist_executor.register_executor(DomainId::new("auth"), users_executor);
        dist_executor.register_executor(DomainId::new("catalog"), products_executor);
        
        // Test query execution for users
        let users_query = Query::new("users")
            .with_domain("auth")
            .add_filter("active", FilterOperator::Equals, Literal::Bool(true));
        
        let result = dist_executor.execute_query_with_timeout(&users_query, Some(5)).await;
        assert!(result.is_ok());
        
        if let Ok(result) = result {
            assert_eq!(result.total_count, 1);
            assert_eq!(result.results[0].get("name"), Some(&Literal::String("Alice".to_string())));
        }
        
        // Test query execution for products
        let products_query = Query::new("products")
            .with_domain("catalog")
            .add_sort("price", SortDirection::Descending);
        
        let result = dist_executor.execute_query_with_timeout(&products_query, Some(5)).await;
        assert!(result.is_ok());
        
        if let Ok(result) = result {
            assert_eq!(result.total_count, 1);
            assert_eq!(result.results[0].get("name"), Some(&Literal::String("Laptop".to_string())));
        }
    }
    
    #[test]
    fn test_query_cache() {
        // Create a cache with 1-second TTL
        let cache = QueryCache::new(1);
        
        // Create a simple result
        let result = QueryResult {
            results: vec![
                {
                    let mut map = HashMap::new();
                    map.insert("id".to_string(), Literal::String("1".to_string()));
                    map
                }
            ],
            total_count: 1,
        };
        
        // Create a query ID
        let query_id = ContentId::from_bytes(&[1, 2, 3, 4]);
        
        // Cache the result
        cache.put(query_id.clone(), result.clone(), None);
        
        // Retrieve from cache
        let cached = cache.get(&query_id);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().total_count, 1);
        
        // Wait for TTL to expire
        std::thread::sleep(Duration::from_secs(2));
        
        // Item should be expired now
        let cached = cache.get(&query_id);
        assert!(cached.is_none());
    }
    
    #[test]
    fn test_query_planner() {
        let planner = DefaultQueryPlanner::new();
        
        let query = Query::new("users")
            .add_filter("age", FilterOperator::GreaterThan, Literal::Int(21))
            .add_sort("name", SortDirection::Ascending)
            .with_limit(10);
        
        let plan = planner.plan(&query).expect("Failed to create plan");
        
        // Plan should have steps
        assert!(!plan.steps.is_empty());
        
        // First step should be a scan
        match &plan.steps[0] {
            QueryExecutionStep::Scan { source, domain } => {
                assert_eq!(source, "users");
                assert_eq!(domain, &None);
            }
            _ => panic!("Expected scan step"),
        }
        
        // Cost should be greater than zero
        assert!(plan.estimated_cost > 0.0);
    }
} 