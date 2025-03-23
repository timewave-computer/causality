// Cross-Domain Relationship Query System
//
// This module provides a query system for traversing cross-domain relationships,
// including efficient indexing, path-finding, and query caching.

use std::collections::{HashMap, HashSet, VecDeque, BTreeMap};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::fmt;
use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};
use crate::types::{ResourceId, DomainId, Timestamp, Metadata};
use crate::resource::relationship_tracker::{RelationshipTracker, ResourceRelationship, RelationshipType, RelationshipDirection};

/// Maximum depth for relationship traversal
const MAX_TRAVERSAL_DEPTH: usize = 10;

/// Default cache expiration time in seconds
const DEFAULT_CACHE_EXPIRATION_SEC: u64 = 300; // 5 minutes

/// Relationship path between resources
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipPath {
    /// Source resource ID
    pub source_id: ResourceId,
    
    /// Target resource ID
    pub target_id: ResourceId,
    
    /// Ordered list of relationships in the path
    pub relationships: Vec<ResourceRelationship>,
    
    /// Total path length (number of hops)
    pub length: usize,
    
    /// Domains traversed in the path
    pub domains: HashSet<DomainId>,
    
    /// When this path was calculated
    pub calculated_at: Timestamp,
}

impl RelationshipPath {
    /// Create a new path
    pub fn new(source_id: ResourceId, target_id: ResourceId) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Self {
            source_id,
            target_id,
            relationships: Vec::new(),
            length: 0,
            domains: HashSet::new(),
            calculated_at: now,
        }
    }
    
    /// Add a relationship to the path
    pub fn add_relationship(&mut self, relationship: ResourceRelationship, domain_id: Option<DomainId>) {
        self.relationships.push(relationship);
        self.length += 1;
        
        if let Some(domain) = domain_id {
            self.domains.insert(domain);
        }
    }
    
    /// Merge two paths
    pub fn merge(&mut self, other: RelationshipPath) -> Result<()> {
        // Check that paths can be merged (target of self = source of other)
        if self.target_id != other.source_id {
            return Err(Error::InvalidOperation(format!(
                "Cannot merge paths: {} != {}",
                self.target_id, other.source_id
            )));
        }
        
        // Merge relationships
        self.relationships.extend(other.relationships);
        self.length += other.length;
        
        // Update target
        self.target_id = other.target_id;
        
        // Merge domains
        self.domains.extend(other.domains);
        
        Ok(())
    }
    
    /// Get all domains traversed in order
    pub fn get_domain_sequence(&self) -> Vec<DomainId> {
        let mut domains = Vec::new();
        let mut seen = HashSet::new();
        
        for rel in &self.relationships {
            if let Some(domain) = rel.metadata.get("domain_id") {
                if let Ok(domain_id) = DomainId::try_from(domain.clone()) {
                    if !seen.contains(&domain_id) {
                        domains.push(domain_id.clone());
                        seen.insert(domain_id);
                    }
                }
            }
        }
        
        domains
    }
}

/// Relationship query parameters
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RelationshipQuery {
    /// Source resource ID
    pub source_id: Option<ResourceId>,
    
    /// Target resource ID
    pub target_id: Option<ResourceId>,
    
    /// Relationship types to consider
    pub relationship_types: Option<Vec<RelationshipType>>,
    
    /// Maximum depth to traverse
    pub max_depth: usize,
    
    /// Domains to include
    pub include_domains: Option<HashSet<DomainId>>,
    
    /// Domains to exclude
    pub exclude_domains: Option<HashSet<DomainId>>,
    
    /// Whether to find all paths or just the shortest
    pub find_all_paths: bool,
}

impl RelationshipQuery {
    /// Create a new query from source to target
    pub fn new(source_id: ResourceId, target_id: ResourceId) -> Self {
        Self {
            source_id: Some(source_id),
            target_id: Some(target_id),
            relationship_types: None,
            max_depth: MAX_TRAVERSAL_DEPTH,
            include_domains: None,
            exclude_domains: None,
            find_all_paths: false,
        }
    }
    
    /// Create a query to find all related resources from a source
    pub fn from_source(source_id: ResourceId) -> Self {
        Self {
            source_id: Some(source_id),
            target_id: None,
            relationship_types: None,
            max_depth: MAX_TRAVERSAL_DEPTH,
            include_domains: None,
            exclude_domains: None,
            find_all_paths: true,
        }
    }
    
    /// Add relationship type filter
    pub fn with_relationship_type(mut self, rel_type: RelationshipType) -> Self {
        let types = self.relationship_types.get_or_insert(Vec::new());
        types.push(rel_type);
        self
    }
    
    /// Set maximum traversal depth
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }
    
    /// Add domain to include
    pub fn include_domain(mut self, domain_id: DomainId) -> Self {
        let domains = self.include_domains.get_or_insert(HashSet::new());
        domains.insert(domain_id);
        self
    }
    
    /// Add domain to exclude
    pub fn exclude_domain(mut self, domain_id: DomainId) -> Self {
        let domains = self.exclude_domains.get_or_insert(HashSet::new());
        domains.insert(domain_id);
        self
    }
    
    /// Set whether to find all paths
    pub fn find_all_paths(mut self, find_all: bool) -> Self {
        self.find_all_paths = find_all;
        self
    }
}

/// Cache entry for relationship queries
struct CacheEntry {
    /// Query result
    result: Vec<RelationshipPath>,
    /// When this entry expires
    expires_at: Instant,
}

/// Relationship query executor
pub struct RelationshipQueryExecutor {
    /// Relationship tracker
    tracker: Arc<RelationshipTracker>,
    
    /// Query cache
    cache: Mutex<HashMap<RelationshipQuery, CacheEntry>>,
    
    /// Cache expiration time
    cache_expiration: Duration,
    
    /// Domain index for quick domain lookup
    domain_index: RwLock<HashMap<DomainId, HashSet<ResourceId>>>,
}

impl RelationshipQueryExecutor {
    /// Create a new query executor
    pub fn new(tracker: Arc<RelationshipTracker>) -> Self {
        Self {
            tracker,
            cache: Mutex::new(HashMap::new()),
            cache_expiration: Duration::from_secs(DEFAULT_CACHE_EXPIRATION_SEC),
            domain_index: RwLock::new(HashMap::new()),
        }
    }
    
    /// Set custom cache expiration time
    pub fn with_cache_expiration(mut self, seconds: u64) -> Self {
        self.cache_expiration = Duration::from_secs(seconds);
        self
    }
    
    /// Execute a relationship query
    pub fn execute(&self, query: &RelationshipQuery) -> Result<Vec<RelationshipPath>> {
        // Check cache first
        if let Some(cached) = self.check_cache(query) {
            return Ok(cached);
        }
        
        // Execute the query
        let result = match (query.source_id.as_ref(), query.target_id.as_ref()) {
            (Some(source), Some(target)) => {
                // Source-to-target path finding
                self.find_paths(source, target, query)
            },
            (Some(source), None) => {
                // Find all reachable resources from source
                self.find_reachable_resources(source, query)
            },
            (None, Some(target)) => {
                // Find all resources that can reach target
                self.find_resources_reaching(target, query)
            },
            (None, None) => {
                // No source or target specified
                Err(Error::InvalidOperation("Query must specify at least source or target".to_string()))
            }
        }?;
        
        // Cache the result
        self.cache_result(query.clone(), result.clone());
        
        Ok(result)
    }
    
    /// Find paths between source and target resources
    fn find_paths(&self, source: &ResourceId, target: &ResourceId, query: &RelationshipQuery) -> Result<Vec<RelationshipPath>> {
        if source == target {
            // Source and target are the same, return empty path
            let mut path = RelationshipPath::new(source.clone(), target.clone());
            return Ok(vec![path]);
        }
        
        // Use breadth-first search for shortest path
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut paths = Vec::new();
        
        // Start from source
        let initial_path = RelationshipPath::new(source.clone(), source.clone());
        queue.push_back(initial_path);
        visited.insert(source.clone());
        
        while let Some(path) = queue.pop_front() {
            // If we've reached max depth, skip
            if path.length >= query.max_depth {
                continue;
            }
            
            // Get the current resource ID (end of the current path)
            let current_id = path.target_id.clone();
            
            // Get all relationships for this resource
            let relationships = self.tracker.get_resource_relationships(&current_id)?;
            
            for rel in relationships {
                // Determine next resource ID based on relationship direction
                let next_id = if rel.source_id == current_id {
                    &rel.target_id
                } else if matches!(rel.direction, RelationshipDirection::Bidirectional | RelationshipDirection::ChildToParent) {
                    &rel.source_id
                } else {
                    continue; // Can't traverse this relationship in this direction
                };
                
                // Skip if we've already visited this resource
                if visited.contains(next_id) {
                    continue;
                }
                
                // Check relationship type filter
                if let Some(types) = &query.relationship_types {
                    if !types.contains(&rel.relationship_type) {
                        continue;
                    }
                }
                
                // Get domain from relationship metadata
                let domain_id = rel.metadata.get("domain_id")
                    .and_then(|d| DomainId::try_from(d.clone()).ok());
                
                // Check domain inclusion/exclusion
                if let Some(domain) = &domain_id {
                    if let Some(include) = &query.include_domains {
                        if !include.contains(domain) {
                            continue;
                        }
                    }
                    
                    if let Some(exclude) = &query.exclude_domains {
                        if exclude.contains(domain) {
                            continue;
                        }
                    }
                }
                
                // Create a new path by extending the current one
                let mut new_path = path.clone();
                new_path.add_relationship(rel.clone(), domain_id.clone());
                new_path.target_id = next_id.clone();
                
                // If we've reached the target, add this path to results
                if next_id == target {
                    paths.push(new_path.clone());
                    
                    // If we only need the shortest path, we can return now
                    if !query.find_all_paths {
                        return Ok(vec![new_path]);
                    }
                }
                
                // Mark as visited and add to queue for further exploration
                visited.insert(next_id.clone());
                queue.push_back(new_path);
            }
        }
        
        Ok(paths)
    }
    
    /// Find all resources reachable from the source
    fn find_reachable_resources(&self, source: &ResourceId, query: &RelationshipQuery) -> Result<Vec<RelationshipPath>> {
        // Use breadth-first search
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut paths = HashMap::new();
        
        // Start from source
        let initial_path = RelationshipPath::new(source.clone(), source.clone());
        queue.push_back(initial_path);
        visited.insert(source.clone());
        
        while let Some(path) = queue.pop_front() {
            // If we've reached max depth, skip
            if path.length >= query.max_depth {
                continue;
            }
            
            // Get the current resource ID (end of the current path)
            let current_id = path.target_id.clone();
            
            // If this is not the source, add to results
            if current_id != *source {
                paths.insert(current_id.clone(), path.clone());
            }
            
            // Get all relationships for this resource
            let relationships = self.tracker.get_resource_relationships(&current_id)?;
            
            for rel in relationships {
                // Determine next resource ID based on relationship direction
                let next_id = if rel.source_id == current_id {
                    &rel.target_id
                } else if matches!(rel.direction, RelationshipDirection::Bidirectional | RelationshipDirection::ChildToParent) {
                    &rel.source_id
                } else {
                    continue; // Can't traverse this relationship in this direction
                };
                
                // Skip if we've already visited this resource
                if visited.contains(next_id) {
                    continue;
                }
                
                // Check relationship type filter
                if let Some(types) = &query.relationship_types {
                    if !types.contains(&rel.relationship_type) {
                        continue;
                    }
                }
                
                // Get domain from relationship metadata
                let domain_id = rel.metadata.get("domain_id")
                    .and_then(|d| DomainId::try_from(d.clone()).ok());
                
                // Check domain inclusion/exclusion
                if let Some(domain) = &domain_id {
                    if let Some(include) = &query.include_domains {
                        if !include.contains(domain) {
                            continue;
                        }
                    }
                    
                    if let Some(exclude) = &query.exclude_domains {
                        if exclude.contains(domain) {
                            continue;
                        }
                    }
                }
                
                // Create a new path by extending the current one
                let mut new_path = path.clone();
                new_path.add_relationship(rel.clone(), domain_id.clone());
                new_path.target_id = next_id.clone();
                
                // Mark as visited and add to queue for further exploration
                visited.insert(next_id.clone());
                queue.push_back(new_path);
            }
        }
        
        // Convert results to vec
        Ok(paths.into_values().collect())
    }
    
    /// Find all resources that can reach the target
    fn find_resources_reaching(&self, target: &ResourceId, query: &RelationshipQuery) -> Result<Vec<RelationshipPath>> {
        // Similar to find_reachable_resources but in reverse direction
        // Use breadth-first search
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut paths = HashMap::new();
        
        // Start from target
        let initial_path = RelationshipPath::new(target.clone(), target.clone());
        queue.push_back(initial_path);
        visited.insert(target.clone());
        
        while let Some(path) = queue.pop_front() {
            // If we've reached max depth, skip
            if path.length >= query.max_depth {
                continue;
            }
            
            // Get the current resource ID (end of the current path)
            let current_id = path.target_id.clone();
            
            // If this is not the target, add to results
            if current_id != *target {
                paths.insert(current_id.clone(), path.clone());
            }
            
            // Get all relationships for this resource
            let relationships = self.tracker.get_resource_relationships(&current_id)?;
            
            for rel in relationships {
                // Determine next resource ID based on relationship direction (reversed)
                let next_id = if rel.target_id == current_id {
                    &rel.source_id
                } else if matches!(rel.direction, RelationshipDirection::Bidirectional | RelationshipDirection::ParentToChild) {
                    &rel.target_id
                } else {
                    continue; // Can't traverse this relationship in this direction
                };
                
                // Skip if we've already visited this resource
                if visited.contains(next_id) {
                    continue;
                }
                
                // Check relationship type filter
                if let Some(types) = &query.relationship_types {
                    if !types.contains(&rel.relationship_type) {
                        continue;
                    }
                }
                
                // Get domain from relationship metadata
                let domain_id = rel.metadata.get("domain_id")
                    .and_then(|d| DomainId::try_from(d.clone()).ok());
                
                // Check domain inclusion/exclusion
                if let Some(domain) = &domain_id {
                    if let Some(include) = &query.include_domains {
                        if !include.contains(domain) {
                            continue;
                        }
                    }
                    
                    if let Some(exclude) = &query.exclude_domains {
                        if exclude.contains(domain) {
                            continue;
                        }
                    }
                }
                
                // Create a new path by extending the current one
                let mut new_path = path.clone();
                new_path.add_relationship(rel.clone(), domain_id.clone());
                new_path.target_id = next_id.clone();
                
                // Mark as visited and add to queue for further exploration
                visited.insert(next_id.clone());
                queue.push_back(new_path);
            }
        }
        
        // Convert results to vec
        Ok(paths.into_values().collect())
    }
    
    /// Check if a query result is cached
    fn check_cache(&self, query: &RelationshipQuery) -> Option<Vec<RelationshipPath>> {
        let cache = self.cache.lock().unwrap();
        
        if let Some(entry) = cache.get(query) {
            if entry.expires_at > Instant::now() {
                return Some(entry.result.clone());
            }
        }
        
        None
    }
    
    /// Cache a query result
    fn cache_result(&self, query: RelationshipQuery, result: Vec<RelationshipPath>) {
        let mut cache = self.cache.lock().unwrap();
        
        let entry = CacheEntry {
            result,
            expires_at: Instant::now() + self.cache_expiration,
        };
        
        cache.insert(query, entry);
    }
    
    /// Clear the query cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
    
    /// Index a resource by domain
    pub fn index_resource(&self, resource_id: ResourceId, domain_id: DomainId) -> Result<()> {
        let mut index = self.domain_index.write().unwrap();
        
        let entry = index.entry(domain_id).or_insert_with(HashSet::new);
        entry.insert(resource_id);
        
        Ok(())
    }
    
    /// Get all resources in a domain
    pub fn get_resources_in_domain(&self, domain_id: &DomainId) -> Result<HashSet<ResourceId>> {
        let index = self.domain_index.read().unwrap();
        
        match index.get(domain_id) {
            Some(resources) => Ok(resources.clone()),
            None => Ok(HashSet::new()),
        }
    }
    
    /// Find cross-domain paths between resources
    pub fn find_cross_domain_path(
        &self,
        source_id: &ResourceId,
        target_id: &ResourceId,
        source_domain: &DomainId,
        target_domain: &DomainId,
    ) -> Result<Vec<RelationshipPath>> {
        if source_domain == target_domain {
            // If resources are in the same domain, use regular path finding
            let query = RelationshipQuery::new(source_id.clone(), target_id.clone())
                .include_domain(source_domain.clone());
                
            return self.execute(&query);
        }
        
        // For cross-domain paths, we need to find boundary resources
        let source_domain_resources = self.get_resources_in_domain(source_domain)?;
        let target_domain_resources = self.get_resources_in_domain(target_domain)?;
        
        // Find paths from source to all resources in source domain
        let query1 = RelationshipQuery::from_source(source_id.clone())
            .include_domain(source_domain.clone());
            
        let source_paths = self.execute(&query1)?;
        
        // Find paths from all resources in target domain to target
        let query2 = RelationshipQuery::new(target_id.clone(), target_id.clone())
            .find_all_paths(true)
            .include_domain(target_domain.clone());
            
        let target_paths = self.find_resources_reaching(target_id, &query2)?;
        
        // Check for connections between domains
        let mut complete_paths = Vec::new();
        
        for spath in &source_paths {
            for tpath in &target_paths {
                // Check if there's a direct relationship between these resources
                if let Ok(bridge_relationships) = self.tracker.get_direct_relationships(&spath.target_id, &tpath.source_id) {
                    for bridge in bridge_relationships {
                        let mut new_path = spath.clone();
                        
                        // Add the bridging relationship
                        let bridge_domain = bridge.metadata.get("domain_id")
                            .and_then(|d| DomainId::try_from(d.clone()).ok());
                            
                        new_path.add_relationship(bridge.clone(), bridge_domain);
                        
                        // Add the target domain part
                        if let Ok(()) = new_path.merge(tpath.clone()) {
                            complete_paths.push(new_path);
                        }
                    }
                }
            }
        }
        
        Ok(complete_paths)
    }
}

/// Query language for relationship searches
pub mod query_language {
    use super::*;
    
    /// Query operation type
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum QueryOperation {
        /// Find path from source to target
        FindPath(ResourceId, ResourceId),
        
        /// Find all resources reachable from source
        FindReachable(ResourceId),
        
        /// Find all resources that can reach target
        FindSources(ResourceId),
        
        /// Find cross-domain path
        FindCrossDomainPath(ResourceId, ResourceId, DomainId, DomainId),
        
        /// Check if path exists
        PathExists(ResourceId, ResourceId),
        
        /// Get path length
        PathLength(ResourceId, ResourceId),
    }
    
    /// Query filter
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum QueryFilter {
        /// Filter by relationship type
        RelationshipType(RelationshipType),
        
        /// Filter by max depth
        MaxDepth(usize),
        
        /// Filter by included domain
        IncludeDomain(DomainId),
        
        /// Filter by excluded domain
        ExcludeDomain(DomainId),
        
        /// Find all paths
        FindAllPaths,
    }
    
    /// Parsed relationship query
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ParsedQuery {
        /// Main operation
        pub operation: QueryOperation,
        
        /// Filters to apply
        pub filters: Vec<QueryFilter>,
    }
    
    impl ParsedQuery {
        /// Convert to RelationshipQuery
        pub fn to_relationship_query(&self) -> Result<RelationshipQuery> {
            match &self.operation {
                QueryOperation::FindPath(source, target) => {
                    let mut query = RelationshipQuery::new(source.clone(), target.clone());
                    self.apply_filters(&mut query)?;
                    Ok(query)
                },
                QueryOperation::FindReachable(source) => {
                    let mut query = RelationshipQuery::from_source(source.clone());
                    self.apply_filters(&mut query)?;
                    Ok(query)
                },
                QueryOperation::FindSources(target) => {
                    let mut query = RelationshipQuery {
                        source_id: None,
                        target_id: Some(target.clone()),
                        relationship_types: None,
                        max_depth: MAX_TRAVERSAL_DEPTH,
                        include_domains: None,
                        exclude_domains: None,
                        find_all_paths: true,
                    };
                    self.apply_filters(&mut query)?;
                    Ok(query)
                },
                QueryOperation::FindCrossDomainPath(source, target, _, _) => {
                    // For cross-domain paths, we'll use the executor method directly
                    // Just create a base query for validation
                    let mut query = RelationshipQuery::new(source.clone(), target.clone());
                    self.apply_filters(&mut query)?;
                    Ok(query)
                },
                QueryOperation::PathExists(source, target) => {
                    let mut query = RelationshipQuery::new(source.clone(), target.clone());
                    // For path existence check, we only need one path
                    query.find_all_paths = false;
                    self.apply_filters(&mut query)?;
                    Ok(query)
                },
                QueryOperation::PathLength(source, target) => {
                    let mut query = RelationshipQuery::new(source.clone(), target.clone());
                    // For path length, we only need the shortest path
                    query.find_all_paths = false;
                    self.apply_filters(&mut query)?;
                    Ok(query)
                },
            }
        }
        
        /// Apply filters to a query
        fn apply_filters(&self, query: &mut RelationshipQuery) -> Result<()> {
            for filter in &self.filters {
                match filter {
                    QueryFilter::RelationshipType(rt) => {
                        let types = query.relationship_types.get_or_insert(Vec::new());
                        types.push(rt.clone());
                    },
                    QueryFilter::MaxDepth(depth) => {
                        query.max_depth = *depth;
                    },
                    QueryFilter::IncludeDomain(domain) => {
                        let domains = query.include_domains.get_or_insert(HashSet::new());
                        domains.insert(domain.clone());
                    },
                    QueryFilter::ExcludeDomain(domain) => {
                        let domains = query.exclude_domains.get_or_insert(HashSet::new());
                        domains.insert(domain.clone());
                    },
                    QueryFilter::FindAllPaths => {
                        query.find_all_paths = true;
                    },
                }
            }
            
            Ok(())
        }
    }
    
    /// Parse a query string into a ParsedQuery
    pub fn parse_query(query_str: &str) -> Result<ParsedQuery> {
        // Placeholder implementation
        // In a real implementation, this would parse a DSL for relationship queries
        Err(Error::NotImplemented("Query language parser not implemented".to_string()))
    }
    
    /// Execute a parsed query
    pub fn execute_parsed_query(
        executor: &RelationshipQueryExecutor,
        parsed_query: &ParsedQuery,
    ) -> Result<Vec<RelationshipPath>> {
        match &parsed_query.operation {
            QueryOperation::FindPath(source, target) => {
                let query = parsed_query.to_relationship_query()?;
                executor.execute(&query)
            },
            QueryOperation::FindReachable(source) => {
                let query = parsed_query.to_relationship_query()?;
                executor.execute(&query)
            },
            QueryOperation::FindSources(target) => {
                let query = parsed_query.to_relationship_query()?;
                executor.execute(&query)
            },
            QueryOperation::FindCrossDomainPath(source, target, source_domain, target_domain) => {
                executor.find_cross_domain_path(source, target, source_domain, target_domain)
            },
            QueryOperation::PathExists(source, target) => {
                let query = parsed_query.to_relationship_query()?;
                let paths = executor.execute(&query)?;
                Ok(paths)
            },
            QueryOperation::PathLength(source, target) => {
                let query = parsed_query.to_relationship_query()?;
                let paths = executor.execute(&query)?;
                Ok(paths)
            },
        }
    }
} 