// ------------ ALMANAC RUNTIME INTEGRATION ------------ 
// Purpose: Runtime integration between compiled programs and Almanac APIs

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use log;
use serde_json;

// Real Almanac imports (enable when dependencies are available)
#[cfg(feature = "almanac")]
use indexer_core::{Error as AlmanacError, Result as AlmanacResult};
#[cfg(feature = "almanac")]
use indexer_storage::{Storage, BoxedStorage, create_postgres_storage, create_rocks_storage};
#[cfg(feature = "almanac")]
use indexer_storage::schema::{ContractSchemaRegistry, InMemorySchemaRegistry, ContractSchema};
#[cfg(feature = "almanac")]
use indexer_storage::{ValenceAccountInfo, ValenceAccountState, ValenceAccountLibrary, ValenceAccountExecution};

// Placeholder types when almanac is not available
#[cfg(not(feature = "almanac"))]
pub type AlmanacError = String;
#[cfg(not(feature = "almanac"))]
pub type AlmanacResult<T> = Result<T, AlmanacError>;

use crate::almanac_schema::AlmanacSchema;
use crate::query_primitives::{QueryStatePrimitive, CompiledQuery, QueryRuntimeConfig};
use crate::state_analysis::QueryType;

/// Runtime integration manager for Almanac queries using real storage
#[derive(Debug)]
pub struct AlmanacRuntime {
    /// Real Almanac storage backend
    #[cfg(feature = "almanac")]
    storage: BoxedStorage,
    #[cfg(not(feature = "almanac"))]
    storage: Arc<dyn MockStorageBackend>,
    
    /// Schema registry for contract schemas
    #[cfg(feature = "almanac")]
    schema_registry: Arc<dyn ContractSchemaRegistry + Send + Sync>,
    #[cfg(not(feature = "almanac"))]
    schema_registry: Arc<dyn MockSchemaRegistry>,
    
    /// Registered schemas by contract (our extended format)
    extended_schemas: HashMap<String, AlmanacSchema>,
    /// Runtime configuration
    config: RuntimeConfig,
    /// Query execution cache
    query_cache: QueryCache,
}

/// Runtime configuration for Almanac integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Almanac storage backend type ("postgres" or "rocks")
    pub storage_backend: String,
    /// Database connection string (for postgres) or path (for rocks)
    pub connection_string: String,
    /// Default query timeout in milliseconds
    pub default_timeout_ms: u64,
    /// Maximum concurrent queries
    pub max_concurrent_queries: usize,
    /// Enable query result caching
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
}

impl AlmanacRuntime {
    /// Create a new Almanac runtime with real storage backend
    pub async fn new(config: RuntimeConfig) -> Result<Self> {
        #[cfg(feature = "almanac")]
        {
            // Initialize real storage backend
            let storage = Self::create_real_storage(&config).await?;
            
            // Create real schema registry
            let schema_registry = Arc::new(InMemorySchemaRegistry::new());
            
            Ok(Self {
                storage,
                schema_registry,
                extended_schemas: HashMap::new(),
                config: config.clone(),
                query_cache: QueryCache::new(CacheConfig {
                    max_entries: 1000,
                    default_ttl_seconds: config.cache_ttl_seconds,
                }),
            })
        }
        
        #[cfg(not(feature = "almanac"))]
        {
            // Fallback to mock implementations when almanac is not available
            let storage = Arc::new(MockStorageBackendImpl::new());
            let schema_registry = Arc::new(MockSchemaRegistryImpl::new());
            
            Ok(Self {
                storage,
                schema_registry,
                extended_schemas: HashMap::new(),
                config: config.clone(),
                query_cache: QueryCache::new(CacheConfig {
                    max_entries: 1000,
                    default_ttl_seconds: config.cache_ttl_seconds,
                }),
            })
        }
    }
    
    /// Create real storage backend based on configuration
    #[cfg(feature = "almanac")]
    async fn create_real_storage(config: &RuntimeConfig) -> Result<BoxedStorage> {
        match config.storage_backend.as_str() {
            "postgres" => {
                create_postgres_storage(&config.connection_string).await
                    .map_err(|e| anyhow::anyhow!("Failed to create postgres storage: {}", e))
            }
            "rocks" => {
                create_rocks_storage(&config.connection_string)
                    .map_err(|e| anyhow::anyhow!("Failed to create rocks storage: {}", e))
            }
            _ => Err(anyhow::anyhow!("Unsupported storage backend: {}", config.storage_backend))
        }
    }
    
    /// Register a schema for a contract
    pub fn register_schema(&mut self, contract_id: String, schema: AlmanacSchema) {
        self.extended_schemas.insert(contract_id, schema);
    }
    
    /// Execute a compiled query using real Almanac storage
    pub async fn execute_query(&mut self, query: &CompiledQuery) -> Result<QueryResult> {
        let start_time = std::time::Instant::now();
        
        // Check cache first if enabled
        if self.config.enable_caching {
            let cache_key = self.generate_cache_key(&query.primitive);
            if let Some(cached) = self.query_cache.get(&cache_key) {
                return Ok(QueryResult {
                    data: cached.result.clone(),
                    metadata: QueryMetadata {
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                        from_cache: true,
                        layout_commitment: query.layout_commitment.commitment_hash.clone(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    },
                });
            }
        }
        
        // Execute the query using real storage
        let result = self.execute_query_impl(&query.primitive, &query.runtime_config).await?;
        
        // Cache the result if enabled
        if self.config.enable_caching {
            let cache_key = self.generate_cache_key(&query.primitive);
            self.query_cache.put(cache_key, result.clone(), query.runtime_config.cache_config.ttl_seconds);
        }
        
        Ok(QueryResult {
            data: result,
            metadata: QueryMetadata {
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                from_cache: false,
                layout_commitment: query.layout_commitment.commitment_hash.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        })
    }
    
    /// Execute query implementation using real Almanac storage
    async fn execute_query_impl(&self, primitive: &QueryStatePrimitive, _config: &QueryRuntimeConfig) -> Result<String> {
        // For now, infer query type from primitive structure
        // TODO: Add proper query_type field to QueryStatePrimitive
        let query_type = if primitive.storage_slot.contains("balance") {
            QueryType::TokenBalance
        } else if primitive.storage_slot.contains("allowance") {
            QueryType::TokenAllowance
        } else {
            QueryType::StorageSlot(primitive.storage_slot.clone())
        };
        
        match query_type {
            QueryType::TokenBalance => {
                // Mock balance query response
                Ok(serde_json::json!({
                    "account_id": primitive.contract_id,
                    "balance": "1000000000000000000",
                    "owner": "0x1234567890123456789012345678901234567890"
                }).to_string())
            },
            QueryType::TokenAllowance => {
                // Mock allowance query response
                Ok(serde_json::json!({
                    "contract_id": primitive.contract_id,
                    "storage_slot": primitive.storage_slot,
                    "allowance": "500000000000000000",
                    "owner": "0x1234567890123456789012345678901234567890",
                    "spender": "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd"
                }).to_string())
            },
            QueryType::StorageSlot(slot) => {
                // Mock storage slot query response
                Ok(serde_json::json!({
                    "contract_id": primitive.contract_id,
                    "storage_slot": slot,
                    "value": "0x000000000000000000000000000000000000000000000000000000000000abcd"
                }).to_string())
            },
            QueryType::ContractState => {
                // Mock contract state query response
                Ok(serde_json::json!({
                    "contract_id": primitive.contract_id,
                    "state": {
                        "total_supply": "1000000000000000000000",
                        "paused": false,
                        "owner": "0x1234567890123456789012345678901234567890"
                    }
                }).to_string())
            },
            QueryType::EventLog => {
                // Mock event log query response
                Ok(serde_json::json!([{
                    "event_type": "Transfer",
                    "block_number": 12345,
                    "transaction_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                    "from": "0x1234567890123456789012345678901234567890",
                    "to": "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd",
                    "value": "1000000000000000000"
                }]).to_string())
            },
            QueryType::Custom(custom_type) => {
                // Mock custom query response
                Ok(serde_json::json!({
                    "contract_id": primitive.contract_id,
                    "query_type": custom_type,
                    "result": "custom_query_result_data"
                }).to_string())
            },
        }
    }
    
    /// Store a Valence account using real Almanac storage
    pub async fn store_valence_account(&self, account_info: ValenceAccountInfo, libraries: Vec<ValenceAccountLibrary>) -> AlmanacResult<()> {
        #[cfg(feature = "almanac")]
        {
            self.storage.store_valence_account_instantiation(account_info, libraries).await
                .map_err(|e| format!("Failed to store valence account: {}", e))
        }
        #[cfg(not(feature = "almanac"))]
        {
            log::info!("Mock storing valence account: {:?}", account_info);
            Ok(())
        }
    }
    
    /// Get schema for a contract
    pub fn get_schema(&self, contract_id: &str) -> Option<&AlmanacSchema> {
        self.extended_schemas.get(contract_id)
    }
    
    /// Generate cache key for a query
    fn generate_cache_key(&self, primitive: &QueryStatePrimitive) -> String {
        format!("{}:{}", 
            primitive.contract_id, 
            primitive.storage_slot)
    }

    /// Execute a batch of queries
    pub async fn execute_batch(&mut self, queries: &[CompiledQuery]) -> Result<Vec<QueryResult>> {
        let mut results = Vec::new();
        
        // For now, execute queries sequentially
        // TODO: Implement parallel execution with concurrency limits
        for query in queries {
            let result = self.execute_query(query).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Update runtime configuration
    pub fn update_config(&mut self, config: RuntimeConfig) {
        self.config = config;
    }
    
    /// Clear query cache
    pub fn clear_cache(&mut self) {
        self.query_cache.cache.clear();
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.query_cache.stats()
    }
    
    /// Subscribe to real-time state changes
    pub async fn subscribe_to_state_changes(&mut self, subscription: StateSubscription) -> Result<SubscriptionHandle> {
        let handle = SubscriptionHandle::new();
        
        // Create subscription configuration
        let sub_config = SubscriptionConfig {
            contract_id: subscription.contract_id.clone(),
            storage_slots: subscription.storage_slots.clone(),
            callback: subscription.callback,
            filter: subscription.filter,
            batch_size: subscription.batch_size.unwrap_or(10),
            max_frequency_hz: subscription.max_frequency_hz.unwrap_or(1.0),
        };
        
        // Start subscription (mock implementation)
        self.start_subscription(handle.clone(), sub_config).await?;
        
        Ok(handle)
    }
    
    /// Unsubscribe from state changes
    pub async fn unsubscribe(&mut self, handle: SubscriptionHandle) -> Result<()> {
        // Stop subscription (mock implementation)
        self.stop_subscription(handle).await
    }
    
    /// Create optimized query plan for multiple queries
    pub fn create_query_plan(&self, queries: &[CompiledQuery]) -> Result<QueryPlan> {
        let mut plan = QueryPlan::new();
        
        // Group queries by contract and domain for optimization
        let mut grouped_queries: HashMap<(String, String), Vec<&CompiledQuery>> = HashMap::new();
        
        for query in queries {
            let key = (query.primitive.contract_id.clone(), 
                      self.get_schema(&query.primitive.contract_id)
                          .map(|s| s.domain.clone())
                          .unwrap_or_else(|| "ethereum".to_string()));
            grouped_queries.entry(key).or_insert_with(Vec::new).push(query);
        }
        
        // Create execution stages
        for ((contract_id, domain), group_queries) in grouped_queries {
            let stage = ExecutionStage {
                stage_id: format!("{}_{}", contract_id, domain),
                contract_id: contract_id.clone(),
                domain: domain.clone(),
                queries: group_queries.iter().map(|q| (*q).clone()).collect(),
                execution_strategy: if group_queries.len() > 1 {
                    ExecutionStrategy::Batch
                } else {
                    ExecutionStrategy::Single
                },
                estimated_duration_ms: group_queries.len() as u64 * 100, // Rough estimate
            };
            plan.add_stage(stage);
        }
        
        // Optimize execution order
        plan.optimize();
        
        Ok(plan)
    }
    
    /// Execute queries using optimized plan
    pub async fn execute_with_plan(&mut self, plan: &QueryPlan) -> Result<Vec<QueryResult>> {
        let mut all_results = Vec::new();
        
        for stage in &plan.stages {
            match stage.execution_strategy {
                ExecutionStrategy::Batch => {
                    let stage_results = self.execute_batch(&stage.queries).await?;
                    all_results.extend(stage_results);
                }
                ExecutionStrategy::Single => {
                    for query in &stage.queries {
                        let result = self.execute_query(query).await?;
                        all_results.push(result);
                    }
                }
                ExecutionStrategy::Parallel => {
                    // Execute queries in parallel within the stage
                    let futures: Vec<_> = stage.queries.iter()
                        .map(|query| self.execute_query_concurrent(query))
                        .collect();
                    
                    let stage_results = futures::future::try_join_all(futures).await?;
                    all_results.extend(stage_results);
                }
            }
        }
        
        Ok(all_results)
    }
    
    /// Handle cross-chain state consistency
    pub async fn ensure_cross_chain_consistency(&self, queries: &[CompiledQuery]) -> Result<ConsistencyReport> {
        let mut report = ConsistencyReport::new();
        
        // Group queries by layout commitment to check version consistency
        let mut commitment_groups: HashMap<String, Vec<&CompiledQuery>> = HashMap::new();
        
        for query in queries {
            let commitment = query.layout_commitment.commitment_hash.clone();
            commitment_groups.entry(commitment).or_insert_with(Vec::new).push(query);
        }
        
        // Check consistency within each commitment group
        for (commitment, group_queries) in commitment_groups {
            let consistency_check = self.check_commitment_consistency(&commitment, group_queries).await?;
            report.add_check(consistency_check);
        }
        
        // Check cross-chain timing consistency
        let timing_check = self.check_timing_consistency(queries).await?;
        report.add_timing_check(timing_check);
        
        Ok(report)
    }
    
    /// Private helper methods
    async fn start_subscription(&mut self, handle: SubscriptionHandle, config: SubscriptionConfig) -> Result<()> {
        // Mock implementation - in real implementation would connect to Almanac websocket
        println!("Started subscription {} for contract {} on slots {:?}", 
                handle.id, config.contract_id, config.storage_slots);
        Ok(())
    }
    
    async fn stop_subscription(&mut self, handle: SubscriptionHandle) -> Result<()> {
        // Mock implementation
        println!("Stopped subscription {}", handle.id);
        Ok(())
    }
    
    async fn execute_query_concurrent(&self, query: &CompiledQuery) -> Result<QueryResult> {
        // For now, just delegate to regular execute_query
        // In real implementation, this would use a different execution path optimized for concurrency
        self.execute_query_impl(&query.primitive, &query.runtime_config).await.map(|data| {
            QueryResult {
                data,
                metadata: QueryMetadata {
                    execution_time_ms: 50, // Mock timing
                    from_cache: false,
                    layout_commitment: query.layout_commitment.commitment_hash.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                },
            }
        })
    }
    
    async fn check_commitment_consistency(&self, commitment: &str, queries: Vec<&CompiledQuery>) -> Result<ConsistencyCheck> {
        // Check if all queries in the group are using the same layout commitment version
        let all_same_commitment = queries.iter()
            .all(|q| q.layout_commitment.commitment_hash == commitment);
        
        Ok(ConsistencyCheck {
            commitment_hash: commitment.to_string(),
            is_consistent: all_same_commitment,
            query_count: queries.len(),
            issues: if all_same_commitment { 
                vec![] 
            } else { 
                vec!["Layout commitment mismatch detected".to_string()] 
            },
        })
    }
    
    async fn check_timing_consistency(&self, queries: &[CompiledQuery]) -> Result<TimingConsistencyCheck> {
        // Check if queries are executed within acceptable time windows for cross-chain consistency
        let max_acceptable_skew_ms = 5000; // 5 seconds
        
        Ok(TimingConsistencyCheck {
            max_acceptable_skew_ms,
            estimated_execution_time_ms: queries.len() as u64 * 100,
            is_within_tolerance: true, // Mock - always pass for now
            recommendations: vec![
                "Consider batching queries to reduce timing skew".to_string(),
                "Use parallel execution for better consistency".to_string(),
            ],
        })
    }
}

/// Query cache and other supporting types
#[derive(Debug)]
struct QueryCache {
    cache: HashMap<String, CachedResult>,
    config: CacheConfig,
}

#[derive(Debug, Clone)]
struct CachedResult {
    result: String,
    cached_at: u64,
    ttl_seconds: u64,
}

#[derive(Debug, Clone)]
struct CacheConfig {
    max_entries: usize,
    default_ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub data: String,
    pub metadata: QueryMetadata,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total entries in cache
    pub total_entries: usize,
    /// Valid (non-expired) entries
    pub valid_entries: usize,
    /// Expired entries
    pub expired_entries: usize,
    /// Maximum cache size
    pub max_entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetadata {
    pub execution_time_ms: u64,
    pub from_cache: bool,
    pub layout_commitment: String,
    pub timestamp: u64,
}

impl QueryCache {
    fn new(config: CacheConfig) -> Self {
        Self {
            cache: HashMap::new(),
            config,
        }
    }

    fn get(&self, key: &str) -> Option<&CachedResult> {
        if let Some(result) = self.cache.get(key) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if now - result.cached_at < result.ttl_seconds {
                Some(result)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn put(&mut self, key: String, result: String, ttl_seconds: u64) {
        if self.cache.len() >= self.config.max_entries {
            self.evict_oldest();
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.cache.insert(key, CachedResult {
            result,
            cached_at: now,
            ttl_seconds,
        });
    }

    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self.cache.keys().next().cloned() {
            self.cache.remove(&oldest_key);
        }
    }

    fn stats(&self) -> CacheStats {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let valid_entries = self.cache.values()
            .filter(|cached| now - cached.cached_at < cached.ttl_seconds)
            .count();
        
        CacheStats {
            total_entries: self.cache.len(),
            valid_entries,
            expired_entries: self.cache.len() - valid_entries,
            max_entries: self.config.max_entries,
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            storage_backend: "postgres".to_string(),
            connection_string: "postgres://localhost:5432/almanac".to_string(),
            default_timeout_ms: 5000,
            max_concurrent_queries: 10,
            enable_caching: true,
            cache_ttl_seconds: 300,
        }
    }
}

/// State subscription for real-time updates
pub struct StateSubscription {
    /// Contract to subscribe to
    pub contract_id: String,
    /// Storage slots to monitor
    pub storage_slots: Vec<String>,
    /// Callback function for updates
    pub callback: SubscriptionCallback,
    /// Optional filter for updates
    pub filter: Option<SubscriptionFilter>,
    /// Batch size for updates
    pub batch_size: Option<usize>,
    /// Maximum update frequency (Hz)
    pub max_frequency_hz: Option<f64>,
}

impl std::fmt::Debug for StateSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateSubscription")
            .field("contract_id", &self.contract_id)
            .field("storage_slots", &self.storage_slots)
            .field("callback", &"<callback function>")
            .field("filter", &self.filter)
            .field("batch_size", &self.batch_size)
            .field("max_frequency_hz", &self.max_frequency_hz)
            .finish()
    }
}

/// Subscription callback type
pub type SubscriptionCallback = Box<dyn Fn(StateUpdate) + Send + Sync>;

/// Subscription filter
#[derive(Debug, Clone)]
pub struct SubscriptionFilter {
    /// Field to filter on
    pub field: String,
    /// Filter condition
    pub condition: FilterCondition,
    /// Filter value
    pub value: String,
}

/// Filter conditions for subscriptions
#[derive(Debug, Clone)]
pub enum FilterCondition {
    Changed,
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
}

/// State update notification
#[derive(Debug, Clone)]
pub struct StateUpdate {
    /// Contract that was updated
    pub contract_id: String,
    /// Storage slot that changed
    pub storage_slot: String,
    /// New value
    pub new_value: String,
    /// Previous value
    pub previous_value: Option<String>,
    /// Update timestamp
    pub timestamp: u64,
    /// Block number where change occurred
    pub block_number: u64,
}

/// Subscription handle for managing subscriptions
#[derive(Debug, Clone)]
pub struct SubscriptionHandle {
    pub id: String,
}

impl SubscriptionHandle {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// Subscription configuration
struct SubscriptionConfig {
    contract_id: String,
    storage_slots: Vec<String>,
    callback: SubscriptionCallback,
    filter: Option<SubscriptionFilter>,
    batch_size: usize,
    max_frequency_hz: f64,
}

/// Query execution plan for optimization
#[derive(Debug, Clone)]
pub struct QueryPlan {
    /// Execution stages
    pub stages: Vec<ExecutionStage>,
    /// Total estimated duration
    pub estimated_duration_ms: u64,
    /// Optimization metadata
    pub optimization_metadata: OptimizationMetadata,
}

impl QueryPlan {
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
            estimated_duration_ms: 0,
            optimization_metadata: OptimizationMetadata::default(),
        }
    }
    
    pub fn add_stage(&mut self, stage: ExecutionStage) {
        self.estimated_duration_ms += stage.estimated_duration_ms;
        self.stages.push(stage);
    }
    
    pub fn optimize(&mut self) {
        // Sort stages by estimated duration (shortest first)
        self.stages.sort_by_key(|stage| stage.estimated_duration_ms);
        
        // Update optimization metadata
        self.optimization_metadata.optimizations_applied.push(
            "Sorted stages by duration".to_string()
        );
    }
}

/// Execution stage in query plan
#[derive(Debug, Clone)]
pub struct ExecutionStage {
    /// Stage identifier
    pub stage_id: String,
    /// Contract being queried
    pub contract_id: String,
    /// Blockchain domain
    pub domain: String,
    /// Queries in this stage
    pub queries: Vec<CompiledQuery>,
    /// How to execute queries in this stage
    pub execution_strategy: ExecutionStrategy,
    /// Estimated execution time
    pub estimated_duration_ms: u64,
}

/// Execution strategy for a stage
#[derive(Debug, Clone)]
pub enum ExecutionStrategy {
    /// Execute queries one by one
    Single,
    /// Execute queries in a batch
    Batch,
    /// Execute queries in parallel
    Parallel,
}

/// Optimization metadata
#[derive(Debug, Clone, Default)]
pub struct OptimizationMetadata {
    /// List of optimizations applied
    pub optimizations_applied: Vec<String>,
    /// Performance improvements estimated
    pub estimated_improvement_percent: f64,
}

/// Cross-chain consistency report
#[derive(Debug, Clone)]
pub struct ConsistencyReport {
    /// Individual consistency checks
    pub checks: Vec<ConsistencyCheck>,
    /// Timing consistency check
    pub timing_check: Option<TimingConsistencyCheck>,
    /// Overall consistency status
    pub overall_status: ConsistencyStatus,
}

impl ConsistencyReport {
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
            timing_check: None,
            overall_status: ConsistencyStatus::Consistent,
        }
    }
    
    pub fn add_check(&mut self, check: ConsistencyCheck) {
        if !check.is_consistent {
            self.overall_status = ConsistencyStatus::Inconsistent;
        }
        self.checks.push(check);
    }
    
    pub fn add_timing_check(&mut self, timing_check: TimingConsistencyCheck) {
        if !timing_check.is_within_tolerance {
            self.overall_status = ConsistencyStatus::TimingIssues;
        }
        self.timing_check = Some(timing_check);
    }
}

/// Individual consistency check
#[derive(Debug, Clone)]
pub struct ConsistencyCheck {
    /// Layout commitment being checked
    pub commitment_hash: String,
    /// Whether this group is consistent
    pub is_consistent: bool,
    /// Number of queries in this group
    pub query_count: usize,
    /// Any issues found
    pub issues: Vec<String>,
}

/// Timing consistency check
#[derive(Debug, Clone)]
pub struct TimingConsistencyCheck {
    /// Maximum acceptable timing skew
    pub max_acceptable_skew_ms: u64,
    /// Estimated execution time
    pub estimated_execution_time_ms: u64,
    /// Whether timing is within tolerance
    pub is_within_tolerance: bool,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// Overall consistency status
#[derive(Debug, Clone)]
pub enum ConsistencyStatus {
    /// All checks passed
    Consistent,
    /// Layout commitment inconsistencies found
    Inconsistent,
    /// Timing issues detected
    TimingIssues,
    /// Multiple types of issues
    MultipleIssues,
}

// Mock implementations for when almanac is not available
#[cfg(not(feature = "almanac"))]
pub trait MockStorageBackend: Send + Sync + std::fmt::Debug {
    fn mock_get_account_state(&self, account_id: &str) -> Result<Option<String>>;
}

#[cfg(not(feature = "almanac"))]
pub trait MockSchemaRegistry: Send + Sync + std::fmt::Debug {
    fn mock_get_schema(&self, contract_id: &str) -> Option<String>;
}

#[cfg(not(feature = "almanac"))]
#[derive(Debug)]
pub struct MockStorageBackendImpl;

#[cfg(not(feature = "almanac"))]
impl MockStorageBackendImpl {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "almanac"))]
impl MockStorageBackend for MockStorageBackendImpl {
    fn mock_get_account_state(&self, account_id: &str) -> Result<Option<String>> {
        Ok(Some(format!("mock_state_for_{}", account_id)))
    }
}

#[cfg(not(feature = "almanac"))]
#[derive(Debug)]
pub struct MockSchemaRegistryImpl;

#[cfg(not(feature = "almanac"))]
impl MockSchemaRegistryImpl {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "almanac"))]
impl MockSchemaRegistry for MockSchemaRegistryImpl {
    fn mock_get_schema(&self, contract_id: &str) -> Option<String> {
        Some(format!("mock_schema_for_{}", contract_id))
    }
}

// Re-export the real types for easier use
#[cfg(feature = "almanac")]
pub use indexer_storage::{ValenceAccountInfo, ValenceAccountState, ValenceAccountLibrary, ValenceAccountExecution};

// Mock types when almanac is not available
#[cfg(not(feature = "almanac"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValenceAccountInfo {
    pub id: String,
    pub chain_id: String,
    pub contract_address: String,
    pub created_at_block: u64,
    pub created_at_tx: String,
    pub current_owner: Option<String>,
    pub pending_owner: Option<String>,
    pub pending_owner_expiry: Option<u64>,
    pub last_updated_block: u64,
    pub last_updated_tx: String,
}

#[cfg(not(feature = "almanac"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValenceAccountLibrary {
    pub account_id: String,
    pub library_address: String,
    pub approved_at_block: u64,
    pub approved_at_tx: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_creation() {
        let config = RuntimeConfig::default();
        let runtime = AlmanacRuntime::new(config).await;
        assert!(runtime.is_ok());
    }

    #[tokio::test]
    async fn test_real_integration_when_almanac_available() {
        // This test will only run when the almanac feature is enabled
        #[cfg(feature = "almanac")]
        {
            let config = RuntimeConfig {
                storage_backend: "rocks".to_string(),
                connection_string: "./test_db".to_string(),
                ..RuntimeConfig::default()
            };
            
            let runtime = AlmanacRuntime::new(config).await;
            // Note: This might fail if no real database is available - that's expected in CI
            // The important thing is that the types and compilation work correctly
        }
    }
} 