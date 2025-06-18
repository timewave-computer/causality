// ------------ ALMANAC SCHEMA GENERATION ------------ 
// Purpose: Automatic Almanac schema generation from state query analysis

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use causality_compiler::state_analysis::{StateAnalysisResult, StateQueryRequirement, QueryType};

/// Almanac schema definition for a contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlmanacSchema {
    /// Contract identifier
    pub contract_id: String,
    /// Blockchain domain
    pub domain: String,
    /// Layout commitment for versioning
    pub layout_commitment: String,
    /// Indexed fields
    pub indexed_fields: Vec<IndexedField>,
    /// Query patterns supported
    pub query_patterns: Vec<QueryPattern>,
    /// Schema metadata
    pub metadata: SchemaMetadata,
}

/// An indexed field in the contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedField {
    /// Field name
    pub name: String,
    /// Storage slot or path
    pub storage_path: String,
    /// Field type
    pub field_type: FieldType,
    /// Whether this field is used in conditional logic
    pub is_conditional: bool,
    /// Indexing strategy
    pub indexing_strategy: IndexingStrategy,
}

/// Types of fields that can be indexed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    /// Unsigned integer
    Uint256,
    /// Address
    Address,
    /// Boolean
    Bool,
    /// Bytes
    Bytes,
    /// String
    String,
    /// Mapping from address to uint256
    AddressToUint256,
    /// Custom type
    Custom(String),
}

/// Indexing strategies for different field types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexingStrategy {
    /// Direct indexing (for simple values)
    Direct,
    /// Mapping indexing (for key-value pairs)
    Mapping,
    /// Event-based indexing
    EventBased,
    /// Custom indexing strategy
    Custom(String),
}

/// Query patterns that the schema supports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPattern {
    /// Pattern name
    pub name: String,
    /// Fields involved in the query
    pub fields: Vec<String>,
    /// Query type
    pub query_type: String,
    /// Optimization hints
    pub optimization_hints: Vec<String>,
}

/// Schema metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMetadata {
    /// Schema version
    pub version: String,
    /// Generation timestamp
    pub generated_at: u64,
    /// Source analysis hash
    pub source_hash: String,
    /// Performance characteristics
    pub performance_hints: Vec<String>,
}

/// Schema generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaGenerationResult {
    /// Generated schemas by contract
    pub schemas: HashMap<String, AlmanacSchema>,
    /// Cross-contract dependencies
    pub dependencies: Vec<SchemaDependency>,
    /// Generation statistics
    pub stats: GenerationStats,
}

/// Dependency between schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDependency {
    /// Source contract
    pub from_contract: String,
    /// Target contract
    pub to_contract: String,
    /// Dependency type
    pub dependency_type: String,
    /// Required fields
    pub required_fields: Vec<String>,
}

/// Statistics about schema generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationStats {
    /// Number of schemas generated
    pub schemas_generated: usize,
    /// Total indexed fields
    pub total_fields: usize,
    /// Generation duration in milliseconds
    pub generation_duration_ms: u64,
}

/// Almanac schema generator
pub struct AlmanacSchemaGenerator {
    /// Known contract patterns
    contract_patterns: HashMap<String, ContractPattern>,
    /// Field type mappings
    type_mappings: HashMap<String, FieldType>,
}

/// Pattern for a known contract type
#[derive(Debug, Clone)]
struct ContractPattern {
    /// Contract type (ERC20, ERC721, etc.)
    contract_type: String,
    /// Standard fields
    standard_fields: Vec<StandardField>,
    /// Common query patterns
    common_patterns: Vec<String>,
}

/// Standard field definition
#[derive(Debug, Clone)]
struct StandardField {
    /// Field name
    name: String,
    /// Storage path
    storage_path: String,
    /// Field type
    field_type: FieldType,
    /// Default indexing strategy
    indexing_strategy: IndexingStrategy,
}

impl AlmanacSchemaGenerator {
    /// Create a new schema generator
    pub fn new() -> Self {
        let mut generator = Self {
            contract_patterns: HashMap::new(),
            type_mappings: HashMap::new(),
        };
        
        generator.initialize_patterns();
        generator.initialize_type_mappings();
        generator
    }
    
    /// Generate Almanac schemas from state analysis results
    pub fn generate_schemas(&self, analysis: &StateAnalysisResult) -> SchemaGenerationResult {
        let start_time = std::time::Instant::now();
        let mut schemas = HashMap::new();
        let mut dependencies = Vec::new();
        
        // Generate schema for each contract
        for (contract_id, queries) in &analysis.queries_by_contract {
            let schema = self.generate_contract_schema(contract_id, queries, analysis);
            schemas.insert(contract_id.clone(), schema);
        }
        
        // Detect cross-contract dependencies
        dependencies = self.detect_dependencies(&schemas);
        
        let generation_duration = start_time.elapsed().as_millis() as u64;
        let total_fields: usize = schemas.values()
            .map(|s| s.indexed_fields.len())
            .sum();
        
        SchemaGenerationResult {
            schemas,
            dependencies,
            stats: GenerationStats {
                schemas_generated: analysis.queries_by_contract.len(),
                total_fields,
                generation_duration_ms: generation_duration,
            },
        }
    }
    
    /// Generate schema for a single contract
    fn generate_contract_schema(
        &self,
        contract_id: &str,
        queries: &[StateQueryRequirement],
        analysis: &StateAnalysisResult,
    ) -> AlmanacSchema {
        let domain = queries.first()
            .map(|q| q.domain.clone())
            .unwrap_or_else(|| "ethereum".to_string());
        
        // Generate layout commitment (simplified for now)
        let layout_commitment = self.generate_layout_commitment(contract_id, queries);
        
        // Generate indexed fields
        let indexed_fields = self.generate_indexed_fields(queries);
        
        // Generate query patterns
        let query_patterns = self.generate_query_patterns(queries);
        
        // Generate performance hints
        let performance_hints = self.generate_performance_hints(queries);
        
        AlmanacSchema {
            contract_id: contract_id.to_string(),
            domain,
            layout_commitment,
            indexed_fields,
            query_patterns,
            metadata: SchemaMetadata {
                version: "1.0.0".to_string(),
                generated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                source_hash: self.compute_source_hash(queries),
                performance_hints,
            },
        }
    }
    
    /// Generate indexed fields from query requirements
    fn generate_indexed_fields(&self, queries: &[StateQueryRequirement]) -> Vec<IndexedField> {
        let mut fields = Vec::new();
        let mut seen_fields = HashSet::new();
        
        for query in queries {
            let field_key = format!("{}:{}", query.storage_slot, query.query_type.type_name());
            if seen_fields.contains(&field_key) {
                continue;
            }
            seen_fields.insert(field_key);
            
            let field_type = self.infer_field_type(&query.query_type);
            let indexing_strategy = self.determine_indexing_strategy(&query.query_type);
            
            fields.push(IndexedField {
                name: query.storage_slot.clone(),
                storage_path: self.generate_storage_path(&query.storage_slot),
                field_type,
                is_conditional: query.is_conditional,
                indexing_strategy,
            });
        }
        
        fields
    }
    
    /// Generate query patterns from requirements
    fn generate_query_patterns(&self, queries: &[StateQueryRequirement]) -> Vec<QueryPattern> {
        let mut patterns = Vec::new();
        let mut pattern_groups: HashMap<String, Vec<&StateQueryRequirement>> = HashMap::new();
        
        // Group queries by type
        for query in queries {
            let pattern_key = query.query_type.type_name();
            pattern_groups.entry(pattern_key).or_default().push(query);
        }
        
        // Generate patterns for each group
        for (pattern_type, group_queries) in pattern_groups {
            let fields: Vec<String> = group_queries.iter()
                .map(|q| q.storage_slot.clone())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();
            
            let optimization_hints = self.generate_optimization_hints(&group_queries);
            
            patterns.push(QueryPattern {
                name: format!("{}_pattern", pattern_type),
                fields,
                query_type: pattern_type,
                optimization_hints,
            });
        }
        
        patterns
    }
    
    /// Initialize known contract patterns
    fn initialize_patterns(&mut self) {
        // ERC20 pattern
        let erc20_fields = vec![
            StandardField {
                name: "balances".to_string(),
                storage_path: "balances".to_string(),
                field_type: FieldType::AddressToUint256,
                indexing_strategy: IndexingStrategy::Mapping,
            },
            StandardField {
                name: "allowances".to_string(),
                storage_path: "allowances".to_string(),
                field_type: FieldType::AddressToUint256,
                indexing_strategy: IndexingStrategy::Mapping,
            },
        ];
        
        self.contract_patterns.insert("erc20".to_string(), ContractPattern {
            contract_type: "ERC20".to_string(),
            standard_fields: erc20_fields,
            common_patterns: vec!["balance_query".to_string(), "allowance_query".to_string()],
        });
    }
    
    /// Initialize type mappings
    fn initialize_type_mappings(&mut self) {
        self.type_mappings.insert("balances".to_string(), FieldType::AddressToUint256);
        self.type_mappings.insert("allowances".to_string(), FieldType::AddressToUint256);
        self.type_mappings.insert("totalSupply".to_string(), FieldType::Uint256);
        self.type_mappings.insert("owner".to_string(), FieldType::Address);
    }
    
    /// Infer field type from query type
    fn infer_field_type(&self, query_type: &QueryType) -> FieldType {
        match query_type {
            QueryType::TokenBalance => FieldType::AddressToUint256,
            QueryType::TokenAllowance => FieldType::AddressToUint256,
            QueryType::StorageSlot(slot) => {
                self.type_mappings.get(slot)
                    .cloned()
                    .unwrap_or(FieldType::Bytes)
            }
            QueryType::Custom(_) => FieldType::Custom("unknown".to_string()),
        }
    }
    
    /// Determine indexing strategy
    fn determine_indexing_strategy(&self, query_type: &QueryType) -> IndexingStrategy {
        match query_type {
            QueryType::TokenBalance | QueryType::TokenAllowance => IndexingStrategy::Mapping,
            QueryType::StorageSlot(_) => IndexingStrategy::Direct,
            QueryType::Custom(_) => IndexingStrategy::Custom("unknown".to_string()),
        }
    }
    
    /// Generate storage path for a field
    fn generate_storage_path(&self, field_name: &str) -> String {
        // Simplified storage path generation
        format!("storage.{}", field_name)
    }
    
    /// Generate layout commitment
    fn generate_layout_commitment(&self, contract_id: &str, queries: &[StateQueryRequirement]) -> String {
        // Simplified layout commitment generation
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        contract_id.hash(&mut hasher);
        for query in queries {
            query.storage_slot.hash(&mut hasher);
            query.query_type.type_name().hash(&mut hasher);
        }
        
        format!("layout_{:x}", hasher.finish())
    }
    
    /// Compute source hash
    fn compute_source_hash(&self, queries: &[StateQueryRequirement]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        for query in queries {
            query.hash(&mut hasher);
        }
        
        format!("{:x}", hasher.finish())
    }
    
    /// Generate performance hints
    fn generate_performance_hints(&self, queries: &[StateQueryRequirement]) -> Vec<String> {
        let mut hints = Vec::new();
        
        let conditional_queries = queries.iter().filter(|q| q.is_conditional).count();
        if conditional_queries > 0 {
            hints.push("conditional_indexing_recommended".to_string());
        }
        
        let unique_contracts = queries.iter()
            .map(|q| &q.contract)
            .collect::<HashSet<_>>()
            .len();
        if unique_contracts > 1 {
            hints.push("cross_contract_optimization".to_string());
        }
        
        hints
    }
    
    /// Generate optimization hints for query patterns
    fn generate_optimization_hints(&self, queries: &[&StateQueryRequirement]) -> Vec<String> {
        let mut hints = Vec::new();
        
        if queries.len() > 1 {
            hints.push("batch_query_optimization".to_string());
        }
        
        if queries.iter().any(|q| q.is_conditional) {
            hints.push("conditional_caching".to_string());
        }
        
        hints
    }
    
    /// Detect cross-contract dependencies
    fn detect_dependencies(&self, schemas: &HashMap<String, AlmanacSchema>) -> Vec<SchemaDependency> {
        // Simplified dependency detection
        Vec::new()
    }
}

// QueryType methods are implemented in causality-compiler

impl Default for AlmanacSchemaGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_compiler::state_analysis::{StateQueryRequirement, QueryType};
    
    #[test]
    fn test_schema_generation() {
        let generator = AlmanacSchemaGenerator::new();
        
        // Create mock analysis result
        let queries = vec![
            StateQueryRequirement {
                contract: "usdc".to_string(),
                storage_slot: "balances".to_string(),
                domain: "ethereum".to_string(),
                query_type: QueryType::TokenBalance,
                is_conditional: false,
            },
        ];
        
        let mut queries_by_contract = HashMap::new();
        queries_by_contract.insert("usdc".to_string(), queries.clone());
        
        let analysis = StateAnalysisResult {
            required_queries: queries,
            queries_by_contract,
            queries_by_domain: HashMap::new(),
            metadata: causality_compiler::state_analysis::AnalysisMetadata {
                expressions_analyzed: 1,
                patterns_detected: 1,
                analysis_duration_ms: 0,
            },
        };
        
        let result = generator.generate_schemas(&analysis);
        
        assert_eq!(result.schemas.len(), 1);
        assert!(result.schemas.contains_key("usdc"));
        
        let schema = &result.schemas["usdc"];
        assert_eq!(schema.contract_id, "usdc");
        assert_eq!(schema.domain, "ethereum");
        assert!(!schema.indexed_fields.is_empty());
    }
    
    #[test]
    fn test_field_type_inference() {
        let generator = AlmanacSchemaGenerator::new();
        
        let balance_type = generator.infer_field_type(&QueryType::TokenBalance);
        assert!(matches!(balance_type, FieldType::AddressToUint256));
        
        let allowance_type = generator.infer_field_type(&QueryType::TokenAllowance);
        assert!(matches!(allowance_type, FieldType::AddressToUint256));
    }
    
    #[test]
    fn test_layout_commitment_generation() {
        let generator = AlmanacSchemaGenerator::new();
        
        let queries = vec![
            StateQueryRequirement {
                contract: "test".to_string(),
                storage_slot: "balances".to_string(),
                domain: "ethereum".to_string(),
                query_type: QueryType::TokenBalance,
                is_conditional: false,
            },
        ];
        
        let commitment1 = generator.generate_layout_commitment("test", &queries);
        let commitment2 = generator.generate_layout_commitment("test", &queries);
        
        // Should be deterministic
        assert_eq!(commitment1, commitment2);
        
        // Should be different for different inputs
        let different_queries = vec![
            StateQueryRequirement {
                contract: "test".to_string(),
                storage_slot: "allowances".to_string(),
                domain: "ethereum".to_string(),
                query_type: QueryType::TokenAllowance,
                is_conditional: false,
            },
        ];
        
        let commitment3 = generator.generate_layout_commitment("test", &different_queries);
        assert_ne!(commitment1, commitment3);
    }
} 