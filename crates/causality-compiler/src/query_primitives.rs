// ------------ QUERY PRIMITIVES ------------ 
// Purpose: Implement query_state primitive with type-safe contract interface generation

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use causality_lisp::ast::{Expr, ExprKind, LispValue};
use crate::almanac_schema::{AlmanacSchema, LayoutCommitment};

/// Query state primitive implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStatePrimitive {
    /// Contract identifier
    pub contract_id: String,
    /// Storage slot or field to query
    pub storage_slot: String,
    /// Query parameters
    pub parameters: Vec<QueryParameter>,
    /// Expected return type
    pub return_type: QueryReturnType,
    /// Query optimization hints
    pub optimization_hints: Vec<OptimizationHint>,
}

/// Parameter for a query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Whether this parameter is required
    pub required: bool,
    /// Default value if optional
    pub default_value: Option<String>,
}

/// Types of query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    /// Ethereum address
    Address,
    /// Unsigned integer
    Uint(u32),
    /// Signed integer
    Int(u32),
    /// String value
    String,
    /// Boolean value
    Bool,
    /// Byte array
    Bytes,
}

/// Return type for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryReturnType {
    /// Single value
    Single(ParameterType),
    /// Array of values
    Array(ParameterType),
    /// Mapping result
    Mapping(ParameterType, ParameterType),
    /// Custom structured type
    Struct(Vec<(String, ParameterType)>),
}

/// Optimization hints for query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationHint {
    /// Cache the result for a duration (seconds)
    Cache(u64),
    /// Batch with other queries
    Batch,
    /// Use specific indexing strategy
    IndexingStrategy(String),
    /// Priority level (1-10, higher is more important)
    Priority(u8),
}

/// Query compilation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledQuery {
    /// Original primitive
    pub primitive: QueryStatePrimitive,
    /// Generated OCaml interface code
    pub ocaml_interface: String,
    /// Runtime query configuration
    pub runtime_config: QueryRuntimeConfig,
    /// Layout commitment for versioning
    pub layout_commitment: LayoutCommitment,
}

/// Runtime configuration for query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRuntimeConfig {
    /// Almanac endpoint configuration
    pub almanac_endpoint: String,
    /// Query timeout in milliseconds
    pub timeout_ms: u64,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Caching configuration
    pub cache_config: CacheConfig,
}

/// Retry configuration for failed queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Base delay between retries in milliseconds
    pub base_delay_ms: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
}

/// Caching configuration for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Whether caching is enabled
    pub enabled: bool,
    /// Cache TTL in seconds
    pub ttl_seconds: u64,
    /// Maximum cache size in entries
    pub max_entries: usize,
}

/// Query primitive compiler
pub struct QueryPrimitiveCompiler {
    /// Known contract schemas
    schemas: BTreeMap<String, AlmanacSchema>,
    /// Default runtime configuration
    default_runtime_config: QueryRuntimeConfig,
}

impl QueryPrimitiveCompiler {
    /// Create a new query primitive compiler
    pub fn new() -> Self {
        Self {
            schemas: BTreeMap::new(),
            default_runtime_config: QueryRuntimeConfig::default(),
        }
    }
    
    /// Register a contract schema for query compilation
    pub fn register_schema(&mut self, contract_id: String, schema: AlmanacSchema) {
        self.schemas.insert(contract_id, schema);
    }
    
    /// Set default runtime configuration
    pub fn set_default_runtime_config(&mut self, config: QueryRuntimeConfig) {
        self.default_runtime_config = config;
    }
    
    /// Compile a query_state expression into a typed query primitive
    pub fn compile_query_state(&self, expr: &Expr) -> Result<CompiledQuery, QueryCompileError> {
        let primitive = self.extract_query_primitive(expr)?;
        let schema = self.get_schema(&primitive.contract_id)?;
        
        let ocaml_interface = self.generate_ocaml_interface(&primitive, schema)?;
        let runtime_config = self.generate_runtime_config(&primitive, schema)?;
        
        Ok(CompiledQuery {
            primitive,
            ocaml_interface,
            runtime_config,
            layout_commitment: schema.layout_commitment.clone(),
        })
    }
    
    /// Extract query primitive from Lisp expression
    fn extract_query_primitive(&self, expr: &Expr) -> Result<QueryStatePrimitive, QueryCompileError> {
        match &expr.kind {
            ExprKind::Apply(func, args) => {
                if let ExprKind::Var(symbol) = &func.kind {
                    if symbol.name() == Some("query_state") {
                        return self.parse_query_state_args(args);
                    }
                }
                Err(QueryCompileError::NotQueryState)
            }
            _ => Err(QueryCompileError::NotQueryState),
        }
    }
    
    /// Parse arguments to query_state function
    fn parse_query_state_args(&self, args: &[Expr]) -> Result<QueryStatePrimitive, QueryCompileError> {
        if args.len() < 2 {
            return Err(QueryCompileError::InvalidArguments("query_state requires at least contract_id and storage_slot".to_string()));
        }
        
        let contract_id = self.extract_string_literal(&args[0])?;
        let storage_slot = self.extract_string_literal(&args[1])?;
        
        Ok(QueryStatePrimitive {
            contract_id,
            storage_slot,
            parameters: vec![],
            return_type: QueryReturnType::Single(ParameterType::String),
            optimization_hints: vec![],
        })
    }
    
    /// Get schema for a contract
    fn get_schema(&self, contract_id: &str) -> Result<&AlmanacSchema, QueryCompileError> {
        self.schemas.get(contract_id)
            .ok_or_else(|| QueryCompileError::UnknownContract(contract_id.to_string()))
    }
    
    /// Generate OCaml interface for the query
    fn generate_ocaml_interface(&self, primitive: &QueryStatePrimitive, schema: &AlmanacSchema) -> Result<String, QueryCompileError> {
        let mut code = String::new();
        
        // Generate function signature
        let function_name = format!("query_{}_{}", primitive.contract_id, primitive.storage_slot);
        code.push_str(&format!("let {} ", function_name));
        
        // Add parameters
        for param in &primitive.parameters {
            code.push_str(&format!("~{} ", param.name));
        }
        
        code.push_str("() = \n");
        
        // Generate implementation
        code.push_str("  (* Query implementation *)\n");
        code.push_str(&format!("  let contract_id = \"{}\" in\n", primitive.contract_id));
        code.push_str(&format!("  let storage_slot = \"{}\" in\n", primitive.storage_slot));
        code.push_str(&format!("  let layout_commitment = \"{}\" in\n", schema.layout_commitment.commitment_hash));
        
        // Add query execution logic
        code.push_str("  let query_params = [\n");
        for param in &primitive.parameters {
            code.push_str(&format!("    (\"{}\", {});\n", param.name, param.name));
        }
        code.push_str("  ] in\n");
        
        code.push_str("  (* Execute query via Almanac *)\n");
        code.push_str("  Almanac.execute_query ~contract_id ~storage_slot ~layout_commitment ~params:query_params\n");
        
        Ok(code)
    }
    
    /// Generate runtime configuration for the query
    fn generate_runtime_config(&self, primitive: &QueryStatePrimitive, _schema: &AlmanacSchema) -> Result<QueryRuntimeConfig, QueryCompileError> {
        let mut config = self.default_runtime_config.clone();
        
        // Apply optimization hints
        for hint in &primitive.optimization_hints {
            match hint {
                OptimizationHint::Cache(duration) => {
                    config.cache_config.enabled = true;
                    config.cache_config.ttl_seconds = *duration;
                }
                OptimizationHint::Priority(level) => {
                    // Adjust timeout based on priority
                    if *level > 7 {
                        config.timeout_ms = config.timeout_ms * 2; // High priority gets more time
                    }
                }
                _ => {} // Other hints handled elsewhere
            }
        }
        
        Ok(config)
    }
    
    /// Extract string literal from expression
    fn extract_string_literal(&self, expr: &Expr) -> Result<String, QueryCompileError> {
        match &expr.kind {
            ExprKind::Const(LispValue::String(s)) => Ok(s.to_string()),
            _ => Err(QueryCompileError::InvalidArguments("Expected string literal".to_string())),
        }
    }
    
    /// Compile multi-chain query coordination
    pub fn compile_multi_chain_query(&self, queries: &[Expr]) -> Result<MultiChainQuery, QueryCompileError> {
        let mut chain_queries = BTreeMap::new();
        
        for query_expr in queries {
            let primitive = self.extract_query_primitive(query_expr)?;
            let schema = self.get_schema(&primitive.contract_id)?;
            
            let chain = schema.domain.clone();
            chain_queries.entry(chain)
                .or_insert_with(Vec::new)
                .push(primitive);
        }
        
        Ok(MultiChainQuery {
            chain_queries,
            coordination_strategy: CoordinationStrategy::Parallel,
            timeout_ms: 10000, // 10 seconds for multi-chain
            retry_config: RetryConfig::default(),
        })
    }
    
    /// Create query composition for filtering and aggregation
    pub fn create_query_composition(&self, base_query: &QueryStatePrimitive, filters: Vec<QueryFilter>) -> Result<ComposedQuery, QueryCompileError> {
        let schema = self.get_schema(&base_query.contract_id)?;
        
        Ok(ComposedQuery {
            base_query: base_query.clone(),
            filters,
            aggregation: None,
            composition_type: CompositionType::Filter,
            layout_commitment: schema.layout_commitment.clone(),
        })
    }
    
    /// Add query result caching with invalidation
    pub fn create_cached_query(&self, primitive: &QueryStatePrimitive, cache_strategy: CacheStrategy) -> Result<CachedQuery, QueryCompileError> {
        let schema = self.get_schema(&primitive.contract_id)?;
        
        let ttl_seconds = cache_strategy.ttl_seconds();
        Ok(CachedQuery {
            primitive: primitive.clone(),
            cache_strategy,
            invalidation_rules: vec![
                InvalidationRule::TimeBasedTTL(ttl_seconds),
                InvalidationRule::LayoutCommitmentChange(schema.layout_commitment.commitment_hash.clone()),
            ],
            cache_key_generator: CacheKeyGenerator::Standard,
        })
    }
    
    /// Handle query failures and timeouts with comprehensive error handling
    pub fn create_resilient_query(&self, primitive: &QueryStatePrimitive, error_handling: ErrorHandlingConfig) -> Result<ResilientQuery, QueryCompileError> {
        Ok(ResilientQuery {
            primitive: primitive.clone(),
            error_handling,
            fallback_strategy: FallbackStrategy::RetryWithBackoff,
            circuit_breaker: CircuitBreakerConfig::default(),
        })
    }
}

/// Errors that can occur during query compilation
#[derive(Debug, Clone, thiserror::Error)]
pub enum QueryCompileError {
    #[error("Expression is not a query_state call")]
    NotQueryState,
    
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
    
    #[error("Unknown contract: {0}")]
    UnknownContract(String),
    
    #[error("Unknown type: {0}")]
    UnknownType(String),
    
    #[error("Schema error: {0}")]
    SchemaError(String),
}

impl Default for QueryPrimitiveCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for QueryRuntimeConfig {
    fn default() -> Self {
        Self {
            almanac_endpoint: "http://localhost:8080".to_string(),
            timeout_ms: 5000,
            retry_config: RetryConfig::default(),
            cache_config: CacheConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            backoff_multiplier: 2.0,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_seconds: 300, // 5 minutes
            max_entries: 1000,
        }
    }
}

/// Multi-chain query coordination
#[derive(Debug, Clone)]
pub struct MultiChainQuery {
    /// Queries organized by blockchain
    pub chain_queries: BTreeMap<String, Vec<QueryStatePrimitive>>,
    /// How to coordinate execution across chains
    pub coordination_strategy: CoordinationStrategy,
    /// Total timeout for all chains
    pub timeout_ms: u64,
    /// Retry configuration
    pub retry_config: RetryConfig,
}

/// Coordination strategy for multi-chain queries
#[derive(Debug, Clone)]
pub enum CoordinationStrategy {
    /// Execute all chains in parallel
    Parallel,
    /// Execute chains sequentially
    Sequential,
    /// Execute with dependency ordering
    Dependent(Vec<ChainDependency>),
}

/// Chain dependency for ordered execution
#[derive(Debug, Clone)]
pub struct ChainDependency {
    pub from_chain: String,
    pub to_chain: String,
    pub dependency_type: DependencyType,
}

/// Type of dependency between chains
#[derive(Debug, Clone)]
pub enum DependencyType {
    /// Must complete before starting
    Sequential,
    /// Result is input to next query
    DataFlow,
    /// Conditional execution based on result
    Conditional,
}

/// Query composition for filtering and aggregation
#[derive(Debug, Clone)]
pub struct ComposedQuery {
    /// Base query to compose
    pub base_query: QueryStatePrimitive,
    /// Filters to apply
    pub filters: Vec<QueryFilter>,
    /// Aggregation to perform
    pub aggregation: Option<QueryAggregation>,
    /// Type of composition
    pub composition_type: CompositionType,
    /// Layout commitment for versioning
    pub layout_commitment: LayoutCommitment,
}

/// Query filter for composition
#[derive(Debug, Clone)]
pub struct QueryFilter {
    /// Field to filter on
    pub field: String,
    /// Filter operation
    pub operation: FilterOperation,
    /// Filter value
    pub value: String,
}

/// Filter operations
#[derive(Debug, Clone)]
pub enum FilterOperation {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    Contains,
    StartsWith,
    EndsWith,
}

/// Query aggregation operations
#[derive(Debug, Clone)]
pub enum QueryAggregation {
    Sum(String),
    Count,
    Average(String),
    Min(String),
    Max(String),
    GroupBy(String),
}

/// Composition type
#[derive(Debug, Clone)]
pub enum CompositionType {
    Filter,
    Aggregate,
    Transform,
    Join(String), // Join with another query
}

/// Cached query with invalidation
#[derive(Debug, Clone)]
pub struct CachedQuery {
    /// Base query primitive
    pub primitive: QueryStatePrimitive,
    /// Caching strategy
    pub cache_strategy: CacheStrategy,
    /// Cache invalidation rules
    pub invalidation_rules: Vec<InvalidationRule>,
    /// Cache key generation strategy
    pub cache_key_generator: CacheKeyGenerator,
}

/// Cache strategy options
#[derive(Debug, Clone)]
pub enum CacheStrategy {
    /// Standard time-based caching
    TimeBasedTTL(u64),
    /// Conditional caching based on query frequency
    Conditional { min_frequency: u32, ttl_seconds: u64 },
    /// Write-through caching with immediate updates
    WriteThrough(u64),
    /// No caching
    None,
}

impl CacheStrategy {
    pub fn ttl_seconds(&self) -> u64 {
        match self {
            CacheStrategy::TimeBasedTTL(ttl) => *ttl,
            CacheStrategy::Conditional { ttl_seconds, .. } => *ttl_seconds,
            CacheStrategy::WriteThrough(ttl) => *ttl,
            CacheStrategy::None => 0,
        }
    }
}

/// Cache invalidation rules
#[derive(Debug, Clone)]
pub enum InvalidationRule {
    /// Time-based TTL expiration
    TimeBasedTTL(u64),
    /// Invalidate when layout commitment changes
    LayoutCommitmentChange(String),
    /// Invalidate on specific events
    EventBased(String),
    /// Manual invalidation
    Manual,
}

/// Cache key generation strategy
#[derive(Debug, Clone)]
pub enum CacheKeyGenerator {
    /// Standard key based on query parameters
    Standard,
    /// Custom key generation function
    Custom(String),
    /// Include layout commitment in key
    VersionAware,
}

/// Resilient query with error handling
#[derive(Debug, Clone)]
pub struct ResilientQuery {
    /// Base query primitive
    pub primitive: QueryStatePrimitive,
    /// Error handling configuration
    pub error_handling: ErrorHandlingConfig,
    /// Fallback strategy on failure
    pub fallback_strategy: FallbackStrategy,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
}

/// Error handling configuration
#[derive(Debug, Clone)]
pub struct ErrorHandlingConfig {
    /// Maximum retries
    pub max_retries: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Timeout per attempt in milliseconds
    pub timeout_per_attempt_ms: u64,
    /// Whether to fail fast on certain errors
    pub fail_fast_errors: Vec<String>,
}

/// Fallback strategy on query failure
#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    /// Retry with exponential backoff
    RetryWithBackoff,
    /// Use cached result if available
    UseCachedResult,
    /// Return default value
    DefaultValue(String),
    /// Fail immediately
    FailFast,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,
    /// Time window for failure counting (seconds)
    pub time_window_seconds: u64,
    /// Recovery timeout (seconds)
    pub recovery_timeout_seconds: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            time_window_seconds: 60,
            recovery_timeout_seconds: 30,
        }
    }
}

impl Default for ErrorHandlingConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 1000,
            backoff_multiplier: 2.0,
            timeout_per_attempt_ms: 5000,
            fail_fast_errors: vec![
                "INVALID_CONTRACT".to_string(),
                "SCHEMA_NOT_FOUND".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_lisp::ast::{Expr, ExprKind, LispValue};
    use crate::almanac_schema::{AlmanacSchema, StorageSlotSchema, SlotDataType, IndexingStrategy, SchemaMetadata};
    
    fn create_test_schema() -> AlmanacSchema {
        AlmanacSchema {
            contract_id: "usdc".to_string(),
            domain: "ethereum".to_string(),
            layout_commitment: LayoutCommitment {
                commitment_hash: "test_hash".to_string(),
                version: "1.0.0".to_string(),
                timestamp: 1234567890,
            },
            indexed_slots: vec![
                StorageSlotSchema {
                    slot_id: "balances".to_string(),
                    data_type: SlotDataType::Uint(256),
                    is_hot: true,
                    indexing_strategy: IndexingStrategy::Full,
                }
            ],
            query_patterns: vec![],
            metadata: SchemaMetadata {
                version: "1.0.0".to_string(),
                generated_at: 1234567890,
                queries_analyzed: 1,
                estimated_storage_bytes: 1024,
            },
        }
    }
    
    #[test]
    fn test_query_primitive_compiler_creation() {
        let mut compiler = QueryPrimitiveCompiler::new();
        let schema = create_test_schema();
        compiler.register_schema("usdc".to_string(), schema);
        
        // Basic test that the compiler can be created and schemas registered
        assert!(compiler.schemas.contains_key("usdc"));
    }
} 