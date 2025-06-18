//! Proof Primitives for ZK State Verification
//!
//! This module implements the `prove_state` primitive and related functionality for generating
//! ZK proofs of blockchain state queries. It coordinates between Almanac (for witness data)
//! and Traverse (for proof generation) to create verifiable state proofs.

use std::collections::HashMap;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use causality_lisp::ast::{Expr, ExprKind, LispValue};
use crate::state_analysis::{StateQueryRequirement, QueryType};
use crate::storage_layout::{StorageLayout, TraverseLayoutInfo};
use crate::almanac_schema::LayoutCommitment;
use crate::traverse_almanac_integration::{TraverseAlmanacIntegrator, WitnessGenerationRequest, IntegrationError};

/// Proof state primitive for generating ZK proofs of state queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProveStatePrimitive {
    /// Contract identifier
    pub contract_id: String,
    /// Storage slot or field to prove
    pub storage_slot: String,
    /// Proof parameters
    pub parameters: Vec<ProofParameter>,
    /// Expected proof type
    pub proof_type: ProofType,
    /// Witness generation strategy
    pub witness_strategy: WitnessStrategy,
    /// Proof optimization hints
    pub optimization_hints: Vec<ProofOptimizationHint>,
}

/// Parameter for proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: ProofParameterType,
    /// Whether this parameter is required
    pub required: bool,
    /// Default value if optional
    pub default_value: Option<String>,
}

/// Types of parameters for proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofParameterType {
    /// Ethereum address
    Address,
    /// Block number
    BlockNumber,
    /// Storage key
    StorageKey,
    /// Proof depth
    ProofDepth,
    /// Custom parameter type
    Custom(String),
}

/// Types of proofs that can be generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofType {
    /// Storage inclusion proof
    StorageInclusion,
    /// Balance proof
    BalanceProof,
    /// Allowance proof
    AllowanceProof,
    /// Custom proof type
    Custom(String),
}

/// Strategy for witness generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WitnessStrategy {
    /// Automatic witness generation from Almanac
    Automatic,
    /// Manual witness provision
    Manual,
    /// Cached witness reuse
    Cached { cache_key: String },
    /// Batch witness generation
    Batch { batch_size: usize },
}

/// Optimization hints for proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofOptimizationHint {
    /// Cache the proof for a duration (seconds)
    Cache(u64),
    /// Batch with other proofs
    Batch,
    /// Use specific proving strategy
    ProvingStrategy(String),
    /// Priority level (1-10, higher is more important)
    Priority(u8),
    /// Parallel proof generation
    Parallel,
}

/// Compiled proof with all necessary information for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledProof {
    /// Original primitive
    pub primitive: ProveStatePrimitive,
    /// Generated witness data
    pub witness_data: WitnessData,
    /// Traverse storage layout
    pub storage_layout: TraverseLayoutInfo,
    /// Proof generation configuration
    pub proof_config: ProofGenerationConfig,
    /// Layout commitment for versioning
    pub layout_commitment: LayoutCommitment,
}

/// Witness data for proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessData {
    /// Storage key being proven
    pub storage_key: String,
    /// Storage value at the key
    pub storage_value: String,
    /// Merkle proof path
    pub merkle_proof: Vec<String>,
    /// Block number for the proof
    pub block_number: u64,
    /// Contract address
    pub contract_address: String,
}

/// Configuration for proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofGenerationConfig {
    /// Traverse endpoint configuration
    pub traverse_endpoint: String,
    /// Proof timeout in milliseconds
    pub timeout_ms: u64,
    /// Retry configuration
    pub retry_config: ProofRetryConfig,
    /// Caching configuration
    pub cache_config: ProofCacheConfig,
}

/// Retry configuration for proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofRetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Base delay between retries in milliseconds
    pub base_delay_ms: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
}

/// Caching configuration for proofs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofCacheConfig {
    /// Whether caching is enabled
    pub enabled: bool,
    /// Cache TTL in seconds
    pub ttl_seconds: u64,
    /// Maximum cache size in entries
    pub max_entries: usize,
}

/// Compiler for proof primitives
pub struct ProofPrimitiveCompiler {
    /// Known storage layouts
    storage_layouts: HashMap<String, StorageLayout>,
    /// Default proof generation configuration
    default_proof_config: ProofGenerationConfig,
    /// Traverse-Almanac integrator for automatic witness generation
    integrator: TraverseAlmanacIntegrator,
}

/// Errors that can occur during proof compilation
#[derive(Debug, Clone, thiserror::Error)]
pub enum ProofCompileError {
    #[error("Expression is not a prove_state call")]
    NotProveState,
    
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
    
    #[error("Unknown contract: {0}")]
    UnknownContract(String),
    
    #[error("Storage layout not found: {0}")]
    StorageLayoutNotFound(String),
    
    #[error("Witness generation failed: {0}")]
    WitnessGenerationFailed(String),
    
    #[error("Proof generation error: {0}")]
    ProofGenerationError(String),
}

impl ProofPrimitiveCompiler {
    /// Create a new proof primitive compiler
    pub fn new() -> Self {
        Self {
            storage_layouts: HashMap::new(),
            default_proof_config: ProofGenerationConfig::default(),
            integrator: TraverseAlmanacIntegrator::new(),
        }
    }
    
    /// Register a storage layout for a contract
    pub fn register_storage_layout(&mut self, contract_id: String, layout: StorageLayout) {
        self.storage_layouts.insert(contract_id, layout);
    }
    
    /// Set default proof generation configuration
    pub fn set_default_proof_config(&mut self, config: ProofGenerationConfig) {
        self.default_proof_config = config;
    }
    
    /// Compile a prove_state expression into a compiled proof
    pub fn compile_prove_state(&self, expr: &Expr) -> Result<CompiledProof, ProofCompileError> {
        // Extract the proof primitive from the expression
        let primitive = self.extract_proof_primitive(expr)?;
        
        // Get the storage layout for the contract
        let storage_layout = self.get_storage_layout(&primitive.contract_id)?;
        
        // Generate witness data
        let witness_data = self.generate_witness_data(&primitive, storage_layout)?;
        
        // Create proof generation configuration
        let proof_config = self.generate_proof_config(&primitive)?;
        
        // Convert storage layout to Traverse format
        let traverse_layout = self.to_traverse_layout(storage_layout);
        
        Ok(CompiledProof {
            primitive,
            witness_data,
            storage_layout: traverse_layout,
            proof_config,
            layout_commitment: storage_layout.layout_commitment.clone(),
        })
    }
    
    /// Extract proof primitive from expression
    fn extract_proof_primitive(&self, expr: &Expr) -> Result<ProveStatePrimitive, ProofCompileError> {
        match &expr.kind {
            ExprKind::Apply(func, args) => {
                if let ExprKind::Var(symbol) = &func.kind {
                    if symbol.as_str() == "prove_state" {
                        return self.parse_prove_state_args(args);
                    }
                }
                Err(ProofCompileError::NotProveState)
            }
            _ => Err(ProofCompileError::NotProveState),
        }
    }
    
    /// Parse arguments to prove_state function
    fn parse_prove_state_args(&self, args: &[Expr]) -> Result<ProveStatePrimitive, ProofCompileError> {
        if args.len() < 2 {
            return Err(ProofCompileError::InvalidArguments("prove_state requires at least contract_id and storage_slot".to_string()));
        }
        
        let contract_id = self.extract_string_literal(&args[0])?;
        let storage_slot = self.extract_string_literal(&args[1])?;
        
        // Determine proof type based on storage slot
        let proof_type = self.infer_proof_type(&storage_slot);
        
        Ok(ProveStatePrimitive {
            contract_id,
            storage_slot,
            parameters: vec![],
            proof_type,
            witness_strategy: WitnessStrategy::Automatic,
            optimization_hints: vec![],
        })
    }
    
    /// Infer proof type from storage slot
    fn infer_proof_type(&self, storage_slot: &str) -> ProofType {
        match storage_slot {
            "balances" => ProofType::BalanceProof,
            "allowances" => ProofType::AllowanceProof,
            _ => ProofType::StorageInclusion,
        }
    }
    
    /// Get storage layout for a contract
    fn get_storage_layout(&self, contract_id: &str) -> Result<&StorageLayout, ProofCompileError> {
        self.storage_layouts.get(contract_id)
            .ok_or_else(|| ProofCompileError::StorageLayoutNotFound(contract_id.to_string()))
    }
    
    /// Generate witness data for the proof
    fn generate_witness_data(&self, primitive: &ProveStatePrimitive, layout: &StorageLayout) -> Result<WitnessData, ProofCompileError> {
        // Mock witness generation - in real implementation would call Almanac
        Ok(WitnessData {
            storage_key: format!("0x{:064x}", 0x1234567890abcdefu64), // Mock storage key
            storage_value: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            merkle_proof: vec![
                "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
                "0x2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            ],
            block_number: 18000000,
            contract_address: primitive.contract_id.clone(),
        })
    }
    
    /// Generate witness data asynchronously using Traverse-Almanac integration
    pub async fn generate_witness_data_async(&mut self, primitive: &ProveStatePrimitive, layout: &StorageLayout, block_number: u64, contract_address: &str) -> Result<WitnessData, ProofCompileError> {
        let request = WitnessGenerationRequest {
            contract_id: primitive.contract_id.clone(),
            query: primitive.storage_slot.clone(),
            block_number,
            contract_address: contract_address.to_string(),
            layout_commitment: layout.layout_commitment.clone(),
            parameters: HashMap::new(),
        };
        
        match self.integrator.generate_witness(request).await {
            Ok(result) => Ok(result.witness),
            Err(IntegrationError::FeatureNotEnabled(_)) => {
                // Fall back to mock witness generation
                self.generate_witness_data(primitive, layout)
            },
            Err(e) => Err(ProofCompileError::WitnessGenerationFailed(format!("{:?}", e))),
        }
    }
    
    /// Generate proof configuration
    fn generate_proof_config(&self, primitive: &ProveStatePrimitive) -> Result<ProofGenerationConfig, ProofCompileError> {
        let mut config = self.default_proof_config.clone();
        
        // Apply optimization hints
        for hint in &primitive.optimization_hints {
            match hint {
                ProofOptimizationHint::Cache(duration) => {
                    config.cache_config.enabled = true;
                    config.cache_config.ttl_seconds = *duration;
                }
                ProofOptimizationHint::Priority(level) => {
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
    
    /// Convert storage layout to Traverse format
    fn to_traverse_layout(&self, layout: &StorageLayout) -> TraverseLayoutInfo {
        let traverse_layout = TraverseLayoutInfo {
            storage: layout.storage.iter().map(|entry| crate::storage_layout::TraverseStorageEntry {
                label: entry.label.clone(),
                slot: entry.slot.clone(),
                offset: entry.offset as u32,
                type_name: entry.type_name.clone(),
            }).collect(),
            types: layout.types.iter().map(|type_info| crate::storage_layout::TraverseTypeInfo {
                type_name: type_info.label.clone(),
                encoding: type_info.encoding.clone(),
                number_of_bytes: type_info.number_of_bytes.clone(),
            }).collect(),
        };
        traverse_layout
    }
    
    /// Extract string literal from expression
    fn extract_string_literal(&self, expr: &Expr) -> Result<String, ProofCompileError> {
        match &expr.kind {
            ExprKind::Const(LispValue::String(s)) => Ok(s.to_string()),
            _ => Err(ProofCompileError::InvalidArguments("Expected string literal".to_string())),
        }
    }
}

/// Proof composition for aggregating multiple proofs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofComposition {
    /// Base proofs to compose
    pub base_proofs: Vec<CompiledProof>,
    /// Composition strategy
    pub composition_strategy: CompositionStrategy,
    /// Aggregation rules
    pub aggregation_rules: Vec<AggregationRule>,
}

/// Strategy for composing multiple proofs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompositionStrategy {
    /// Parallel composition (all proofs independent)
    Parallel,
    /// Sequential composition (proofs depend on each other)
    Sequential,
    /// Tree composition (hierarchical proof structure)
    Tree { depth: usize },
}

/// Rules for aggregating proof results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationRule {
    /// Rule identifier
    pub rule_id: String,
    /// Aggregation operation
    pub operation: AggregationOperation,
    /// Input proof indices
    pub input_proofs: Vec<usize>,
}

/// Operations for aggregating proof results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationOperation {
    /// Logical AND of all proofs
    And,
    /// Logical OR of all proofs
    Or,
    /// Sum of numeric values
    Sum,
    /// Minimum value
    Min,
    /// Maximum value
    Max,
    /// Custom aggregation function
    Custom(String),
}

/// Proof verification and validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofVerification {
    /// Proof to verify
    pub proof: CompiledProof,
    /// Verification parameters
    pub verification_params: VerificationParams,
    /// Expected result
    pub expected_result: Option<String>,
}

/// Parameters for proof verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationParams {
    /// Verification timeout in milliseconds
    pub timeout_ms: u64,
    /// Verification strategy
    pub strategy: VerificationStrategy,
    /// Additional verification data
    pub additional_data: HashMap<String, String>,
}

/// Strategy for proof verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStrategy {
    /// Full verification (complete proof check)
    Full,
    /// Fast verification (optimized for speed)
    Fast,
    /// Cached verification (use cached results if available)
    Cached,
}

impl Default for ProofPrimitiveCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ProofGenerationConfig {
    fn default() -> Self {
        Self {
            traverse_endpoint: "http://localhost:8081".to_string(),
            timeout_ms: 30000, // 30 seconds for proof generation
            retry_config: ProofRetryConfig::default(),
            cache_config: ProofCacheConfig::default(),
        }
    }
}

impl Default for ProofRetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            backoff_multiplier: 2.0,
        }
    }
}

impl Default for ProofCacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_seconds: 3600, // 1 hour
            max_entries: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_lisp::ast::{Expr, ExprKind, LispValue};
    use crate::storage_layout::{StorageLayout, StorageEntry, TypeInfo};
    use crate::almanac_schema::LayoutCommitment;
    
    fn create_test_storage_layout() -> StorageLayout {
        StorageLayout {
            contract_name: "usdc".to_string(),
            storage: vec![
                StorageEntry {
                    label: "balances".to_string(),
                    slot: "1".to_string(),
                    offset: 0,
                    type_name: "t_mapping_address_uint256".to_string(),
                }
            ],
            types: vec![
                TypeInfo {
                    label: "t_mapping_address_uint256".to_string(),
                    number_of_bytes: "32".to_string(),
                    encoding: "mapping".to_string(),
                    base: None,
                    key: Some("t_address".to_string()),
                    value: Some("t_uint256".to_string()),
                }
            ],
            layout_commitment: LayoutCommitment {
                commitment_hash: "test_hash".to_string(),
                version: "1.0.0".to_string(),
                timestamp: 1234567890,
            },
            domain: "ethereum".to_string(),
        }
    }
    
    #[test]
    fn test_proof_primitive_compiler_creation() {
        let mut compiler = ProofPrimitiveCompiler::new();
        let layout = create_test_storage_layout();
        compiler.register_storage_layout("usdc".to_string(), layout);
        
        // Basic test that the compiler can be created and layouts registered
        assert!(compiler.storage_layouts.contains_key("usdc"));
    }
    
    #[test]
    fn test_proof_type_inference() {
        let compiler = ProofPrimitiveCompiler::new();
        
        assert!(matches!(compiler.infer_proof_type("balances"), ProofType::BalanceProof));
        assert!(matches!(compiler.infer_proof_type("allowances"), ProofType::AllowanceProof));
        assert!(matches!(compiler.infer_proof_type("other"), ProofType::StorageInclusion));
    }
} 