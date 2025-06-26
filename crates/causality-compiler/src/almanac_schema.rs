// ------------ ALMANAC SCHEMA GENERATION ------------ 
// Purpose: Generate Almanac indexing schemas from state query analysis

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::state_analysis::{StateAnalysisResult, StateQueryRequirement, QueryType};
// Note: LayoutCommitment will be defined locally to avoid circular dependency

/// Layout commitment for content-addressed contract versioning
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayoutCommitment {
    /// Content-addressed hash of the layout
    pub commitment_hash: String,
    /// Version identifier
    pub version: String,
    /// Timestamp of commitment creation
    pub timestamp: u64,
}

/// Represents an Almanac indexing schema for a specific contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlmanacSchema {
    /// Contract identifier
    pub contract_id: String,
    /// Blockchain domain
    pub domain: String,
    /// Layout commitment for versioning
    pub layout_commitment: LayoutCommitment,
    /// Storage slots to index
    pub indexed_slots: Vec<StorageSlotSchema>,
    /// Query patterns to optimize for
    pub query_patterns: Vec<QueryPattern>,
    /// Schema metadata
    pub metadata: SchemaMetadata,
}

/// Schema for a specific storage slot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSlotSchema {
    /// Slot identifier
    pub slot_id: String,
    /// Data type of the slot
    pub data_type: SlotDataType,
    /// Whether this slot is frequently queried
    pub is_hot: bool,
    /// Indexing strategy for this slot
    pub indexing_strategy: IndexingStrategy,
}

/// Data types for storage slots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlotDataType {
    /// Unsigned integer (with bit width)
    Uint(u32),
    /// Signed integer (with bit width)
    Int(u32),
    /// Ethereum address
    Address,
    /// Boolean value
    Bool,
    /// Byte array (with optional fixed length)
    Bytes(Option<usize>),
    /// String value
    String,
    /// Mapping type (key type, value type)
    Mapping(Box<SlotDataType>, Box<SlotDataType>),
    /// Array type (element type, optional fixed length)
    Array(Box<SlotDataType>, Option<usize>),
}

/// Indexing strategies for different access patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexingStrategy {
    /// Full indexing - store all values
    Full,
    /// Sparse indexing - only index when accessed
    Sparse,
    /// Range indexing - optimize for range queries
    Range,
    /// Hash indexing - optimize for exact matches
    Hash,
}

/// Query pattern for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPattern {
    /// Pattern identifier
    pub pattern_id: String,
    /// Query type
    pub query_type: QueryType,
    /// Expected frequency (queries per hour)
    pub frequency: u32,
    /// Whether this pattern is used in conditional logic
    pub is_conditional: bool,
}

/// Schema metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMetadata {
    /// Schema version
    pub version: String,
    /// Generation timestamp
    pub generated_at: u64,
    /// Number of queries analyzed
    pub queries_analyzed: usize,
    /// Estimated storage requirements (bytes)
    pub estimated_storage_bytes: u64,
}

/// Schema generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaGenerationResult {
    /// Generated schemas by contract
    pub schemas: BTreeMap<String, AlmanacSchema>,
    /// Cross-contract dependencies
    pub dependencies: Vec<SchemaDependency>,
    /// Generation metadata
    pub metadata: GenerationMetadata,
}

/// Dependency between schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDependency {
    /// Source contract
    pub from_contract: String,
    /// Target contract
    pub to_contract: String,
    /// Dependency type
    pub dependency_type: DependencyType,
}

/// Types of schema dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Query dependency (one contract queries another)
    Query,
    /// State dependency (one contract's state affects another)
    State,
    /// Temporal dependency (queries must be ordered)
    Temporal,
}

/// Generation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    /// Total schemas generated
    pub schemas_generated: usize,
    /// Total dependencies identified
    pub dependencies_identified: usize,
    /// Generation duration in milliseconds
    pub generation_duration_ms: u64,
}

/// Almanac schema generator
#[allow(dead_code)]
pub struct AlmanacSchemaGenerator {
    /// Known contract layouts
    contract_layouts: BTreeMap<String, LayoutCommitment>,
    /// Schema optimization settings
    ________optimization_settings: OptimizationSettings,
}

/// Settings for schema optimization
#[derive(Debug, Clone)]
pub struct OptimizationSettings {
    /// Minimum query frequency to consider for optimization
    pub min_frequency_threshold: u32,
    /// Maximum storage overhead percentage
    pub max_storage_overhead: f32,
    /// Enable range query optimization
    pub enable_range_optimization: bool,
    /// Enable conditional query optimization
    pub enable_conditional_optimization: bool,
}

impl AlmanacSchemaGenerator {
    /// Create a new schema generator
    pub fn new() -> Self {
        Self {
            contract_layouts: BTreeMap::new(),
            ________optimization_settings: OptimizationSettings::default(),
        }
    }
    
    /// Create a new schema generator with custom optimization settings
    pub fn with_optimization_settings(settings: OptimizationSettings) -> Self {
        Self {
            contract_layouts: BTreeMap::new(),
            ________optimization_settings: settings,
        }
    }
    
    /// Register a contract layout commitment
    pub fn register_layout(&mut self, contract_id: String, layout: LayoutCommitment) {
        self.contract_layouts.insert(contract_id, layout);
    }
    
    /// Generate Almanac schemas from state analysis results
    pub fn generate_schemas(&self, analysis: &StateAnalysisResult) -> SchemaGenerationResult {
        let start_time = std::time::Instant::now();
        let mut schemas = BTreeMap::new();
        let mut dependencies = Vec::new();
        
        // Generate schema for each contract
        for (contract_id, queries) in &analysis.queries_by_contract {
            let schema = self.generate_contract_schema(contract_id, queries, analysis);
            schemas.insert(contract_id.clone(), schema);
        }
        
        // Identify cross-contract dependencies
        dependencies.extend(self.identify_dependencies(analysis));
        
        let generation_duration = start_time.elapsed().as_millis() as u64;
        let dependencies_count = dependencies.len();
        
        SchemaGenerationResult {
            schemas,
            dependencies,
            metadata: GenerationMetadata {
                schemas_generated: analysis.queries_by_contract.len(),
                dependencies_identified: dependencies_count,
                generation_duration_ms: generation_duration,
            },
        }
    }
    
    /// Generate schema for a specific contract
    fn generate_contract_schema(
        &self,
        contract_id: &str,
        queries: &[StateQueryRequirement],
        _analysis: &StateAnalysisResult,
    ) -> AlmanacSchema {
        let layout_commitment = self.contract_layouts
            .get(contract_id)
            .cloned()
            .unwrap_or_else(|| LayoutCommitment::default_for_contract(contract_id));
        
        let domain = queries.first()
            .map(|q| q.domain.clone())
            .unwrap_or_else(|| "ethereum".to_string());
        
        let indexed_slots = self.generate_storage_slots(queries);
        let query_patterns = self.generate_query_patterns(queries);
        
        let estimated_storage = self.estimate_storage_requirements(&indexed_slots, &query_patterns);
        
        AlmanacSchema {
            contract_id: contract_id.to_string(),
            domain,
            layout_commitment,
            indexed_slots,
            query_patterns,
            metadata: SchemaMetadata {
                version: "1.0.0".to_string(),
                generated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                queries_analyzed: queries.len(),
                estimated_storage_bytes: estimated_storage,
            },
        }
    }
    
    /// Generate storage slot schemas from query requirements
    fn generate_storage_slots(&self, queries: &[StateQueryRequirement]) -> Vec<StorageSlotSchema> {
        let mut slots = BTreeMap::new();
        
        for query in queries {
            let slot_id = query.storage_slot.clone();
            let data_type = self.infer_data_type(&query.query_type);
            let is_hot = query.is_conditional || self.is_frequent_query(query);
            let indexing_strategy = self.determine_indexing_strategy(query, is_hot);
            
            slots.insert(slot_id.clone(), StorageSlotSchema {
                slot_id,
                data_type,
                is_hot,
                indexing_strategy,
            });
        }
        
        slots.into_values().collect()
    }
    
    /// Generate query patterns for optimization
    fn generate_query_patterns(&self, queries: &[StateQueryRequirement]) -> Vec<QueryPattern> {
        let mut patterns = BTreeMap::new();
        
        for query in queries {
            let pattern_id = format!("{}_{}", query.query_type.type_name(), query.storage_slot);
            let frequency = if query.is_conditional { 100 } else { 10 }; // Rough estimate
            
            patterns.insert(pattern_id.clone(), QueryPattern {
                pattern_id,
                query_type: query.query_type.clone(),
                frequency,
                is_conditional: query.is_conditional,
            });
        }
        
        patterns.into_values().collect()
    }
    
    /// Identify cross-contract dependencies
    fn identify_dependencies(&self, analysis: &StateAnalysisResult) -> Vec<SchemaDependency> {
        let mut dependencies = Vec::new();
        let contracts: Vec<_> = analysis.queries_by_contract.keys().collect();
        
        // For now, create simple query dependencies between contracts
        for i in 0..contracts.len() {
            for j in (i + 1)..contracts.len() {
                dependencies.push(SchemaDependency {
                    from_contract: contracts[i].clone(),
                    to_contract: contracts[j].clone(),
                    dependency_type: DependencyType::Query,
                });
            }
        }
        
        dependencies
    }
    
    /// Infer data type from query type
    fn infer_data_type(&self, query_type: &QueryType) -> SlotDataType {
        match query_type {
            QueryType::TokenBalance => SlotDataType::Uint(256),
            QueryType::TokenAllowance => SlotDataType::Uint(256),
            QueryType::StorageSlot(_) => SlotDataType::Bytes(Some(32)), // Default to bytes32
            QueryType::ContractState => SlotDataType::Bytes(None),
            QueryType::EventLog => SlotDataType::Bytes(None),
            QueryType::Custom(_) => SlotDataType::Bytes(None),
        }
    }
    
    /// Determine if a query is frequent based on context
    fn is_frequent_query(&self, query: &StateQueryRequirement) -> bool {
        // Simple heuristic: balance queries are typically frequent
        matches!(query.query_type, QueryType::TokenBalance)
    }
    
    /// Determine optimal indexing strategy
    fn determine_indexing_strategy(&self, query: &StateQueryRequirement, is_hot: bool) -> IndexingStrategy {
        if is_hot {
            IndexingStrategy::Full
        } else if query.is_conditional {
            IndexingStrategy::Hash
        } else {
            IndexingStrategy::Sparse
        }
    }
    
    /// Estimate storage requirements for a schema
    fn estimate_storage_requirements(&self, slots: &[StorageSlotSchema], patterns: &[QueryPattern]) -> u64 {
        let base_storage: u64 = slots.iter().map(|slot| {
            match &slot.data_type {
                SlotDataType::Uint(bits) | SlotDataType::Int(bits) => (*bits as u64) / 8,
                SlotDataType::Address => 20,
                SlotDataType::Bool => 1,
                SlotDataType::Bytes(Some(len)) => *len as u64,
                SlotDataType::Bytes(None) => 1024, // Estimate
                SlotDataType::String => 256, // Estimate
                SlotDataType::Mapping(_, _) => 1024, // Estimate per entry
                SlotDataType::Array(_, Some(len)) => *len as u64 * 32,
                SlotDataType::Array(_, None) => 1024, // Estimate
            }
        }).sum();
        
        // Add overhead for indexing structures
        let indexing_overhead = patterns.len() as u64 * 128;
        
        base_storage + indexing_overhead
    }
}

impl Default for AlmanacSchemaGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for OptimizationSettings {
    fn default() -> Self {
        Self {
            min_frequency_threshold: 10,
            max_storage_overhead: 0.2, // 20%
            enable_range_optimization: true,
            enable_conditional_optimization: true,
        }
    }
}

impl LayoutCommitment {
    /// Create a default layout commitment for a contract
    pub fn default_for_contract(contract_id: &str) -> Self {
        // Generate a deterministic but unique commitment based on contract ID
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        contract_id.hash(&mut hasher);
        let hash = hasher.finish();
        
        LayoutCommitment {
            commitment_hash: format!("layout_{:016x}", hash),
            version: "1.0.0".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_analysis::{StateQueryRequirement, QueryType};
    
    #[test]
    fn test_schema_generation() {
        let mut generator = AlmanacSchemaGenerator::new();
        
        // Create mock analysis result
        let queries = vec![
            StateQueryRequirement {
                contract: "usdc".to_string(),
                storage_slot: "balances".to_string(),
                domain: "ethereum".to_string(),
                query_type: QueryType::TokenBalance,
                is_conditional: true,
            },
            StateQueryRequirement {
                contract: "usdc".to_string(),
                storage_slot: "allowances".to_string(),
                domain: "ethereum".to_string(),
                query_type: QueryType::TokenAllowance,
                is_conditional: false,
            },
        ];
        
        let mut queries_by_contract = BTreeMap::new();
        queries_by_contract.insert("usdc".to_string(), queries.clone());
        
        _analysis = StateAnalysisResult {
            required_queries: queries,
            queries_by_contract,
            queries_by_domain: BTreeMap::new(),
            metadata: crate::state_analysis::AnalysisMetadata {
                expressions_analyzed: 10,
                patterns_detected: 2,
                analysis_duration_ms: 5,
            },
        };
        
        let result = generator.generate_schemas(&analysis);
        
        assert_eq!(result.schemas.len(), 1);
        assert!(result.schemas.contains_key("usdc"));
        
        let usdc_schema = &result.schemas["usdc"];
        assert_eq!(usdc_schema.contract_id, "usdc");
        assert_eq!(usdc_schema.domain, "ethereum");
        assert_eq!(usdc_schema.indexed_slots.len(), 2);
        assert_eq!(usdc_schema.query_patterns.len(), 2);
    }
    
    #[test]
    fn test_storage_slot_generation() {
        let generator = AlmanacSchemaGenerator::new();
        
        let queries = vec![
            StateQueryRequirement {
                contract: "test".to_string(),
                storage_slot: "balance".to_string(),
                domain: "ethereum".to_string(),
                query_type: QueryType::TokenBalance,
                is_conditional: true,
            },
        ];
        
        let slots = generator.generate_storage_slots(&queries);
        
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].slot_id, "balance");
        assert!(slots[0].is_hot);
        assert!(matches!(slots[0].data_type, SlotDataType::Uint(256)));
        assert!(matches!(slots[0].indexing_strategy, IndexingStrategy::Full));
    }
    
    #[test]
    fn test_______optimization_settings() {
        let settings = OptimizationSettings {
            min_frequency_threshold: 50,
            max_storage_overhead: 0.1,
            enable_range_optimization: false,
            enable_conditional_optimization: true,
        };
        
        let generator = AlmanacSchemaGenerator::with_optimization_settings(settings);
        assert_eq!(generator.optimization_settings.min_frequency_threshold, 50);
        assert_eq!(generator.optimization_settings.max_storage_overhead, 0.1);
        assert!(!generator.optimization_settings.enable_range_optimization);
        assert!(generator.optimization_settings.enable_conditional_optimization);
    }
} 