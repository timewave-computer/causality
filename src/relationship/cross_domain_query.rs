use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use crate::domain::DomainId;
use crate::error::{Error, Result};
use crate::relationship::{
    RelationshipTracker, RelationshipPath, ResourceRelationship,
    RelationshipType, RelationshipFilter, RelationshipPathCacheKey
};
use crate::resource::ContentId;
use crate::time::timestamp::Timestamp;

/// Cache for relationship query results
pub struct RelationshipQueryCache {
    paths: RwLock<HashMap<RelationshipPathCacheKey, CachedPath>>,
    max_size: usize,
    ttl: std::time::Duration,
}

/// A cached path result
struct CachedPath {
    paths: Vec<RelationshipPath>,
    timestamp: Timestamp,
}

impl RelationshipQueryCache {
    /// Creates a new relationship query cache
    pub fn new(max_size: usize, ttl: std::time::Duration) -> Self {
        Self {
            paths: RwLock::new(HashMap::with_capacity(max_size)),
            max_size,
            ttl,
        }
    }

    /// Gets cached paths if they exist and are not expired
    pub async fn get_paths(&self, key: &RelationshipPathCacheKey) -> Option<Vec<RelationshipPath>> {
        let paths = self.paths.read().await;
        if let Some(cached) = paths.get(key) {
            // Check if the cache entry has expired
            let now = Timestamp::now();
            if now.duration_since(&cached.timestamp).unwrap_or_default() < self.ttl {
                return Some(cached.paths.clone());
            }
        }
        None
    }

    /// Stores paths in the cache
    pub async fn store_paths(&self, key: RelationshipPathCacheKey, paths: Vec<RelationshipPath>) {
        let mut cache = self.paths.write().await;
        
        // If we've reached the max size, remove the oldest entry
        if cache.len() >= self.max_size && !cache.contains_key(&key) {
            if let Some((oldest_key, _)) = cache.iter()
                .min_by_key(|(_, v)| v.timestamp.clone()) {
                let oldest_key = oldest_key.clone();
                cache.remove(&oldest_key);
            }
        }
        
        cache.insert(key, CachedPath {
            paths,
            timestamp: Timestamp::now(),
        });
    }

    /// Invalidates cache entries related to a resource
    pub async fn invalidate_for_resource(&self, resource_id: &ContentId) {
        let mut cache = self.paths.write().await;
        cache.retain(|k, _| k.source_id != *resource_id && k.target_id != *resource_id);
    }
}

/// Query for relationship paths
#[derive(Clone, Debug)]
pub struct RelationshipQuery {
    /// Source resource ID
    pub source_id: ContentId,
    
    /// Target resource ID (optional for broader queries)
    pub target_id: Option<ContentId>,
    
    /// Maximum search depth
    pub max_depth: usize,
    
    /// Filter for relationship types
    pub relationship_types: Option<Vec<RelationshipType>>,
    
    /// Domain filter (if specified, only find relationships in these domains)
    pub domain_filter: Option<HashSet<DomainId>>,
    
    /// Whether to include relationships that traverse domain boundaries
    pub allow_cross_domain: bool,
    
    /// Whether to limit results by confidence score
    pub min_confidence: Option<f64>,
    
    /// Maximum number of paths to return
    pub max_results: Option<usize>,
}

impl RelationshipQuery {
    /// Creates a new relationship query
    pub fn new(source_id: ContentId, target_id: ContentId) -> Self {
        Self {
            source_id,
            target_id: Some(target_id),
            max_depth: 5,  // Default max depth
            relationship_types: None,
            domain_filter: None,
            allow_cross_domain: true,
            min_confidence: None,
            max_results: None,
        }
    }

    /// Sets the maximum search depth
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Restricts search to specific relationship types
    pub fn with_relationship_type(mut self, relationship_type: RelationshipType) -> Self {
        let types = self.relationship_types.get_or_insert_with(Vec::new);
        types.push(relationship_type);
        self
    }

    /// Restricts search to specific domains
    pub fn with_domain_filter(mut self, domains: HashSet<DomainId>) -> Self {
        self.domain_filter = Some(domains);
        self
    }

    /// Sets whether cross-domain traversal is allowed
    pub fn with_cross_domain(mut self, allow: bool) -> Self {
        self.allow_cross_domain = allow;
        self
    }

    /// Sets minimum confidence score
    pub fn with_min_confidence(mut self, confidence: f64) -> Self {
        self.min_confidence = Some(confidence);
        self
    }

    /// Sets maximum number of results
    pub fn with_max_results(mut self, max_results: usize) -> Self {
        self.max_results = Some(max_results);
        self
    }

    /// Creates a cache key for this query
    pub fn to_cache_key(&self) -> Option<RelationshipPathCacheKey> {
        match &self.target_id {
            Some(target_id) => Some(RelationshipPathCacheKey {
                source_id: self.source_id.clone(),
                target_id: target_id.clone(),
                max_depth: self.max_depth,
                relationship_types: self.relationship_types.clone().unwrap_or_default(),
                allow_cross_domain: self.allow_cross_domain,
            }),
            None => None, // No caching for queries without a specific target
        }
    }
}

/// Context for executing relationship queries
pub struct QueryContext {
    /// The current domain
    pub current_domain: DomainId,
    
    /// Available capabilities 
    pub capabilities: Vec<String>,
    
    /// Query start time
    pub query_time: Timestamp,
    
    /// Domains known to be available
    pub available_domains: HashSet<DomainId>,
}

/// Trait for domain relationship providers
#[async_trait]
pub trait DomainRelationshipProvider: Send + Sync {
    /// Gets the domain ID
    fn domain_id(&self) -> &DomainId;
    
    /// Finds relationships from a given resource
    async fn find_relationships_from(&self, 
        resource_id: &ContentId, 
        filter: &RelationshipFilter
    ) -> Result<Vec<ResourceRelationship>>;
    
    /// Checks if a resource exists in this domain
    async fn resource_exists(&self, resource_id: &ContentId) -> Result<bool>;
}

/// Executor for relationship queries
pub struct RelationshipQueryExecutor {
    /// Local relationship tracker
    tracker: Arc<RelationshipTracker>,
    
    /// Cross-domain relationship providers
    domain_providers: RwLock<HashMap<DomainId, Arc<dyn DomainRelationshipProvider>>>,
    
    /// Query cache
    cache: RelationshipQueryCache,
}

impl RelationshipQueryExecutor {
    /// Creates a new query executor
    pub fn new(tracker: Arc<RelationshipTracker>) -> Self {
        Self {
            tracker,
            domain_providers: RwLock::new(HashMap::new()),
            cache: RelationshipQueryCache::new(1000, std::time::Duration::from_secs(300)),
        }
    }

    /// Registers a domain relationship provider
    pub async fn register_domain_provider(&self, provider: Arc<dyn DomainRelationshipProvider>) {
        let mut providers = self.domain_providers.write().await;
        providers.insert(provider.domain_id().clone(), provider);
    }

    /// Executes a relationship query
    pub async fn execute(&self, query: &RelationshipQuery) -> Result<Vec<RelationshipPath>> {
        // Check cache first if target is specified
        if let Some(cache_key) = query.to_cache_key() {
            if let Some(cached_paths) = self.cache.get_paths(&cache_key).await {
                return Ok(cached_paths);
            }
        }

        // Perform the query
        let paths = self.find_paths(query).await?;
        
        // Cache the result if appropriate
        if let Some(cache_key) = query.to_cache_key() {
            self.cache.store_paths(cache_key, paths.clone()).await;
        }
        
        Ok(paths)
    }

    /// Finds paths between resources
    async fn find_paths(&self, query: &RelationshipQuery) -> Result<Vec<RelationshipPath>> {
        match &query.target_id {
            Some(target_id) => {
                // Specific target - use BFS to find paths
                self.find_specific_paths(&query.source_id, target_id, query).await
            },
            None => {
                // No specific target - explore all paths up to max depth
                self.explore_paths(&query.source_id, query).await
            }
        }
    }

    /// Finds paths between specific source and target
    async fn find_specific_paths(
        &self,
        source_id: &ContentId,
        target_id: &ContentId,
        query: &RelationshipQuery,
    ) -> Result<Vec<RelationshipPath>> {
        let mut result_paths = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        // Start with the source resource
        queue.push_back((
            source_id.clone(),
            Vec::new(), // Empty path so far
            HashSet::new(), // No domains traversed yet
        ));
        
        visited.insert(source_id.clone());
        
        while let Some((current_id, path_so_far, domains_traversed)) = queue.pop_front() {
            // Check if we've reached max depth
            if path_so_far.len() >= query.max_depth {
                continue;
            }
            
            // Find relationships from the current resource
            let relationships = self.get_outgoing_relationships(&current_id, query).await?;
            
            for relationship in relationships {
                let target = relationship.target_id.clone();
                
                // Skip if already visited
                if visited.contains(&target) {
                    continue;
                }
                
                // Create new path with this relationship
                let mut new_path = path_so_far.clone();
                new_path.push(relationship.clone());
                
                // Track domains traversed
                let mut new_domains = domains_traversed.clone();
                if let Some(domain) = relationship.target_domain.clone() {
                    new_domains.insert(domain);
                }
                if let Some(domain) = relationship.source_domain.clone() {
                    new_domains.insert(domain);
                }
                
                // Check domain filter if specified
                if let Some(domain_filter) = &query.domain_filter {
                    if !new_domains.is_subset(domain_filter) {
                        continue;
                    }
                }
                
                // Check if we've found a path to the target
                if target == *target_id {
                    let relationship_path = RelationshipPath {
                        source_id: source_id.clone(),
                        target_id: target_id.clone(),
                        relationships: new_path,
                        length: new_path.len(),
                        domains: new_domains.clone(),
                        calculated_at: Timestamp::now(),
                    };
                    
                    result_paths.push(relationship_path);
                    
                    // If we've reached max results, return
                    if let Some(max_results) = query.max_results {
                        if result_paths.len() >= max_results {
                            return Ok(result_paths);
                        }
                    }
                    
                    // Don't continue from the target
                    continue;
                }
                
                // Add to queue for further exploration
                visited.insert(target.clone());
                queue.push_back((target, new_path, new_domains));
            }
        }
        
        Ok(result_paths)
    }

    /// Explores all paths from a source up to max depth
    async fn explore_paths(
        &self,
        source_id: &ContentId,
        query: &RelationshipQuery,
    ) -> Result<Vec<RelationshipPath>> {
        let mut result_paths = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        // Start with the source resource
        queue.push_back((
            source_id.clone(),
            Vec::new(), // Empty path so far
            HashSet::new(), // No domains traversed yet
        ));
        
        visited.insert(source_id.clone());
        
        while let Some((current_id, path_so_far, domains_traversed)) = queue.pop_front() {
            // Check if we've reached max depth
            if path_so_far.len() >= query.max_depth {
                continue;
            }
            
            // Find relationships from the current resource
            let relationships = self.get_outgoing_relationships(&current_id, query).await?;
            
            for relationship in relationships {
                let target = relationship.target_id.clone();
                
                // Skip if already visited (avoid cycles)
                if visited.contains(&target) {
                    continue;
                }
                
                // Create new path with this relationship
                let mut new_path = path_so_far.clone();
                new_path.push(relationship.clone());
                
                // Track domains traversed
                let mut new_domains = domains_traversed.clone();
                if let Some(domain) = relationship.target_domain.clone() {
                    new_domains.insert(domain);
                }
                if let Some(domain) = relationship.source_domain.clone() {
                    new_domains.insert(domain);
                }
                
                // Check domain filter if specified
                if let Some(domain_filter) = &query.domain_filter {
                    if !new_domains.is_subset(domain_filter) {
                        continue;
                    }
                }
                
                // Add this as a result path
                let relationship_path = RelationshipPath {
                    source_id: source_id.clone(),
                    target_id: target.clone(),
                    relationships: new_path.clone(),
                    length: new_path.len(),
                    domains: new_domains.clone(),
                    calculated_at: Timestamp::now(),
                };
                
                result_paths.push(relationship_path);
                
                // If we've reached max results, return
                if let Some(max_results) = query.max_results {
                    if result_paths.len() >= max_results {
                        return Ok(result_paths);
                    }
                }
                
                // Add to queue for further exploration
                visited.insert(target.clone());
                queue.push_back((target, new_path, new_domains));
            }
        }
        
        Ok(result_paths)
    }

    /// Gets outgoing relationships from a resource
    async fn get_outgoing_relationships(
        &self,
        resource_id: &ContentId,
        query: &RelationshipQuery,
    ) -> Result<Vec<ResourceRelationship>> {
        // Create a filter based on query parameters
        let filter = RelationshipFilter {
            relationship_types: query.relationship_types.clone(),
            max_results: None,
            include_deleted: false,
        };
        
        // First check local tracker
        let mut relationships = self.tracker.get_outgoing_relationships(resource_id, &filter)?;
        
        // If cross-domain is allowed, check other domains
        if query.allow_cross_domain {
            let providers = self.domain_providers.read().await;
            
            for provider in providers.values() {
                // Skip if domain filter excludes this domain
                if let Some(domain_filter) = &query.domain_filter {
                    if !domain_filter.contains(provider.domain_id()) {
                        continue;
                    }
                }
                
                // Check if resource exists in this domain
                if provider.resource_exists(resource_id).await? {
                    // Get relationships from this domain
                    let domain_relationships = provider.find_relationships_from(resource_id, &filter).await?;
                    relationships.extend(domain_relationships);
                }
            }
        }
        
        // Filter by type if specified
        if let Some(types) = &query.relationship_types {
            relationships.retain(|r| types.contains(&r.relationship_type));
        }
        
        Ok(relationships)
    }

    /// Finds a path between resources in different domains
    pub async fn find_cross_domain_path(
        &self,
        source_id: &ContentId,
        target_id: &ContentId,
        source_domain: &DomainId,
        target_domain: &DomainId,
    ) -> Result<Option<RelationshipPath>> {
        // If domains are the same, use regular path finding
        if source_domain == target_domain {
            let query = RelationshipQuery::new(source_id.clone(), target_id.clone())
                .with_max_depth(10);
                
            let paths = self.execute(&query).await?;
            return Ok(paths.into_iter().next());
        }
        
        // Create domain-specific query
        let mut domains = HashSet::new();
        domains.insert(source_domain.clone());
        domains.insert(target_domain.clone());
        
        let query = RelationshipQuery::new(source_id.clone(), target_id.clone())
            .with_max_depth(10)
            .with_domain_filter(domains)
            .with_cross_domain(true);
            
        let paths = self.execute(&query).await?;
        
        // Filter for paths that actually cross the domains we care about
        for path in paths {
            let path_domains: HashSet<_> = path.domains.iter().cloned().collect();
            if path_domains.contains(source_domain) && path_domains.contains(target_domain) {
                return Ok(Some(path));
            }
        }
        
        Ok(None)
    }
    
    /// Gets domain-aware total relationship counts
    pub async fn get_relationship_counts(&self, 
        resource_id: &ContentId, 
        relationship_types: Option<Vec<RelationshipType>>
    ) -> Result<HashMap<RelationshipType, usize>> {
        let mut result = HashMap::new();
        
        // Get local counts
        let local_counts = self.tracker.get_relationship_counts(resource_id, relationship_types.clone())?;
        
        // Initialize result with local counts
        for (relationship_type, count) in local_counts {
            result.insert(relationship_type, count);
        }
        
        // Add counts from domain providers
        let providers = self.domain_providers.read().await;
        for provider in providers.values() {
            // Skip if resource doesn't exist in this domain
            if !provider.resource_exists(resource_id).await? {
                continue;
            }
            
            // Create filter with relationship types
            let filter = RelationshipFilter {
                relationship_types: relationship_types.clone(),
                max_results: None,
                include_deleted: false,
            };
            
            // Get all relationships for counting
            let relationships = provider.find_relationships_from(resource_id, &filter).await?;
            
            // Count by type
            for relationship in relationships {
                *result.entry(relationship.relationship_type).or_insert(0) += 1;
            }
        }
        
        Ok(result)
    }
    
    /// Invalidates cached paths involving a resource
    pub async fn invalidate_cache_for_resource(&self, resource_id: &ContentId) {
        self.cache.invalidate_for_resource(resource_id).await;
    }
}

/// Creates a resource state transition helper for testing
pub struct ResourceStateTransitionHelper {
    query_executor: Arc<RelationshipQueryExecutor>,
    tracker: Arc<RelationshipTracker>,
}

impl ResourceStateTransitionHelper {
    /// Creates a new helper
    pub fn new(query_executor: Arc<RelationshipQueryExecutor>, tracker: Arc<RelationshipTracker>) -> Self {
        Self {
            query_executor,
            tracker,
        }
    }
    
    /// Validates relationships for a state transition
    pub async fn validate_relationships_for_transition(
        &self,
        resource_id: &ContentId,
        from_state: &str,
        to_state: &str
    ) -> Result<bool> {
        // Example implementation - this would be expanded based on specific rules
        
        // Get all relationships for this resource
        let filter = RelationshipFilter {
            relationship_types: None,
            max_results: None,
            include_deleted: false,
        };
        
        let relationships = self.tracker.get_resource_relationships(resource_id, &filter)?;
        
        // Check if any relationships prevent this transition
        for relationship in &relationships {
            // Example rule: Can't transition to "Deleted" if resource has "ParentChild" relationships
            if to_state == "Deleted" && relationship.relationship_type == RelationshipType::ParentChild {
                return Ok(false);
            }
        }
        
        // For cross-domain relationships, also validate in other domains
        let has_cross_domain = relationships.iter().any(|r| 
            r.source_domain.is_some() && r.target_domain.is_some() && 
            r.source_domain != r.target_domain
        );
        
        if has_cross_domain {
            // In a real implementation, we would query other domains to check their rules
            // For now, just demonstrate the pattern
            
            // Example: Get counts by relationship type across domains
            let counts = self.query_executor.get_relationship_counts(
                resource_id, 
                None
            ).await?;
            
            // Example rule: Can't transition to "Archived" if resource has more than 5 dependency relationships
            if to_state == "Archived" && 
               counts.get(&RelationshipType::Dependency).unwrap_or(&0) > &5 {
                return Ok(false);
            }
        }
        
        // If all checks pass, the transition is valid
        Ok(true)
    }
    
    /// Updates relationships after a state transition
    pub async fn update_relationships_after_transition(
        &self,
        resource_id: &ContentId,
        from_state: &str,
        to_state: &str
    ) -> Result<()> {
        // Example implementation
        
        // Get all relationships for this resource
        let filter = RelationshipFilter {
            relationship_types: None,
            max_results: None,
            include_deleted: false,
        };
        
        let relationships = self.tracker.get_resource_relationships(resource_id, &filter)?;
        
        // Example: If resource is deleted, mark certain relationships as deleted
        if to_state == "Deleted" {
            for relationship in relationships {
                if relationship.source_id == *resource_id {
                    self.tracker.mark_relationship_deleted(&relationship.id)?;
                }
            }
        }
        
        // Invalidate any cached queries involving this resource
        self.query_executor.invalidate_cache_for_resource(resource_id).await;
        
        Ok(())
    }
} 
