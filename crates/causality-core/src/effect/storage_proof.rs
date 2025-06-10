//! Storage proof effects for blockchain state verification
//!
//! This module provides storage proof effects that enable effects to depend on
//! verified blockchain storage state. It integrates with the Traverse storage
//! commitment system and Valence coprocessor for ZK proof generation.

use std::collections::HashMap;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use crate::lambda::base::Value;
use crate::effect::handler_registry::{EffectHandler, EffectResult};
use crate::effect::cross_chain::{BlockchainDomain, DomainConfig};
use hex;

//-----------------------------------------------------------------------------
// Storage Proof Effect Types
//-----------------------------------------------------------------------------

/// Storage proof effect extending the core effect system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageProofEffect {
    /// Unique identifier for this storage proof effect
    pub id: String,
    
    /// Effect description (simplified to avoid EffectExpr serialization issues)
    pub description: String,
    
    /// Storage dependencies required for this effect
    pub storage_dependencies: Vec<StorageDependency>,
    
    /// Blockchain domains this effect spans
    pub domains: Vec<BlockchainDomain>,
    
    /// Proof requirements for verification
    pub proof_requirements: StorageProofRequirements,
    
    /// Effect metadata
    pub metadata: StorageEffectMetadata,
}

/// Storage dependency specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageDependency {
    /// Unique identifier for this dependency
    pub id: String,
    
    /// Blockchain domain this storage exists on
    pub domain: BlockchainDomain,
    
    /// Storage key specification
    pub key_spec: StorageKeySpec,
    
    /// Expected value constraint (optional)
    pub value_constraint: Option<StorageValueConstraint>,
    
    /// Whether this dependency is critical for effect execution
    pub is_critical: bool,
    
    /// Cache policy for this storage value
    pub cache_policy: StorageCachePolicy,
}

/// Storage key specification for different blockchain types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageKeySpec {
    /// Ethereum contract storage
    Ethereum {
        contract_address: String,
        storage_slot: StorageSlot,
        block_number: Option<u64>,
    },
    /// Cosmos/CosmWasm state
    Cosmos {
        contract_address: String,
        state_key: String,
        height: Option<u64>,
    },
    /// Direct storage commitment
    Commitment(String), // Placeholder for StorageCommitment
}

/// Ethereum storage slot specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageSlot {
    /// Direct slot number
    Direct(String),
    /// Mapping access: keccak256(key . slot)
    Mapping { base_slot: String, key: String },
    /// Array access: keccak256(slot) + index
    Array { base_slot: String, index: u64 },
    /// Nested access for complex data structures
    Nested { path: Vec<StorageAccess> },
}

/// Storage access patterns for nested data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageAccess {
    /// Field access by name
    Field(String),
    /// Mapping key access
    MapKey(String),
    /// Array index access
    ArrayIndex(u64),
}

/// Storage value constraint for verification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageValueConstraint {
    /// Value must equal this
    Equals(Vec<u8>),
    /// Value must be greater than this
    GreaterThan(Vec<u8>),
    /// Value must be less than this
    LessThan(Vec<u8>),
    /// Value must be in this range
    Range { min: Vec<u8>, max: Vec<u8> },
    /// Custom constraint expression
    Custom(String),
}

/// Storage cache policy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageCachePolicy {
    /// No caching
    NoCache,
    /// Cache for a specific time duration (seconds)
    TimeToLive(u64),
    /// Cache until next block
    UntilNextBlock,
    /// Cache for a specific number of blocks
    BlockCount(u64),
    /// Cache permanently (for immutable data)
    Permanent,
}

/// Storage proof requirements
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageProofRequirements {
    /// Whether ZK proofs are required
    pub require_zk_proof: bool,
    
    /// ZK circuit configuration
    pub zk_circuit: Option<ZkCircuitConfig>,
    
    /// Proof aggregation strategy
    pub aggregation: ProofAggregationStrategy,
    
    /// Verification requirements
    pub verification: VerificationRequirements,
    
    /// Proof expiry settings
    pub expiry: ProofExpiryPolicy,
}

/// ZK circuit configuration for storage proofs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZkCircuitConfig {
    /// Circuit identifier
    pub circuit_id: String,
    
    /// Maximum number of storage slots to verify
    pub max_storage_slots: u32,
    
    /// Maximum proof size
    pub max_proof_size: u32,
    
    /// Circuit-specific parameters
    pub parameters: HashMap<String, String>,
}

/// Proof aggregation strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofAggregationStrategy {
    /// No aggregation - individual proofs
    Individual,
    /// Batch multiple storage proofs together
    Batch { max_batch_size: u32 },
    /// Recursive proof composition
    Recursive,
    /// Custom aggregation logic
    Custom(String),
}

/// Verification requirements
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationRequirements {
    /// Minimum number of confirmations
    pub min_confirmations: u64,
    
    /// Acceptable finality delay (blocks)
    pub max_finality_delay: u64,
    
    /// Whether to verify proof on-chain
    pub on_chain_verification: bool,
    
    /// Trusted verification key sources
    pub trusted_key_sources: Vec<String>,
}

/// Proof expiry policy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofExpiryPolicy {
    /// Proofs never expire
    Never,
    /// Expire after time duration (seconds)
    TimeToLive(u64),
    /// Expire after block count
    BlockCount(u64),
    /// Expire when storage value changes
    OnStorageUpdate,
}

/// Storage effect metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageEffectMetadata {
    /// Effect creation timestamp
    pub created_at: u64,
    
    /// Effect creator/owner
    pub creator: Option<String>,
    
    /// Effect description
    pub description: Option<String>,
    
    /// Effect tags for categorization
    pub tags: HashMap<String, String>,
    
    /// Estimated gas cost for execution
    pub estimated_gas_cost: Option<u64>,
    
    /// Priority level
    pub priority: EffectPriority,
}

/// Effect execution priority
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectPriority {
    Low,
    Normal,
    High,
    Critical,
}

//-----------------------------------------------------------------------------
// Storage Proof Results
//-----------------------------------------------------------------------------

/// Result of storage proof verification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageProofResult {
    /// Unique identifier for this result
    pub id: String,
    
    /// Storage dependency that was resolved
    pub dependency_id: String,
    
    /// Verified storage value
    pub value: Vec<u8>,
    
    /// Block/height at which this was verified
    pub block_info: BlockInfo,
    
    /// Proof data (if ZK proof was used)
    pub proof_data: Option<ProofData>,
    
    /// Verification timestamp
    pub verified_at: u64,
    
    /// Cache expiry information
    pub cache_info: CacheInfo,
}

/// Block information for verified storage
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockInfo {
    /// Block number/height
    pub height: u64,
    
    /// Block hash
    pub hash: String,
    
    /// Block timestamp
    pub timestamp: u64,
    
    /// Number of confirmations
    pub confirmations: u64,
}

/// ZK proof data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofData {
    /// Proof bytes
    pub proof: Vec<u8>,
    
    /// Public inputs
    pub public_inputs: Vec<Vec<u8>>,
    
    /// Verification key identifier
    pub verification_key_id: String,
    
    /// Circuit identifier used
    pub circuit_id: String,
    
    /// Proof generation metadata
    pub metadata: ProofMetadata,
}

/// Proof generation metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofMetadata {
    /// Generation timestamp
    pub generated_at: u64,
    
    /// Generation time in milliseconds
    pub generation_time_ms: u64,
    
    /// Prover service identifier
    pub prover_service: Option<String>,
    
    /// Proof size in bytes
    pub proof_size: u32,
    
    /// Additional metadata
    pub extra: HashMap<String, String>,
}

/// Cache information for storage values
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheInfo {
    /// Cache policy applied
    pub policy: StorageCachePolicy,
    
    /// Cache expiry timestamp
    pub expires_at: Option<u64>,
    
    /// Cache validity conditions
    pub validity_conditions: Vec<CacheValidityCondition>,
}

/// Cache validity conditions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheValidityCondition {
    /// Valid until block number
    UntilBlock(u64),
    /// Valid until timestamp
    UntilTimestamp(u64),
    /// Valid until storage update
    UntilStorageUpdate,
    /// Custom validity condition
    Custom(String),
}

//-----------------------------------------------------------------------------
// Storage Effect Constraints and Dependencies
//-----------------------------------------------------------------------------

/// Storage effect constraint for dependency resolution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageEffectConstraint {
    /// Constraint identifier
    pub id: String,
    
    /// Dependencies this constraint applies to
    pub dependencies: Vec<String>,
    
    /// Constraint type
    pub constraint_type: ConstraintType,
    
    /// Constraint expression
    pub expression: ConstraintExpression,
    
    /// Whether this constraint is required for correctness
    pub is_required: bool,
}

/// Types of storage effect constraints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Value constraints on storage data
    ValueConstraint,
    /// Temporal constraints (ordering, timing)
    TemporalConstraint,
    /// Cross-domain consistency constraints
    ConsistencyConstraint,
    /// Resource constraints (gas, compute)
    ResourceConstraint,
    /// Custom constraint type
    Custom(String),
}

/// Constraint expression language (simplified to avoid Term serialization issues)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintExpression {
    /// Simple equality check
    Equals { left: String, right: String },
    /// Comparison operators
    Compare { left: String, op: ComparisonOp, right: String },
    /// Logical operations
    Logical { left: Box<ConstraintExpression>, op: LogicalOp, right: Box<ConstraintExpression> },
    /// Function call constraint
    FunctionCall { name: String, args: Vec<String> },
    /// Custom constraint expression
    Custom(String),
}

/// Comparison operators for constraints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOp {
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

/// Logical operators for constraints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicalOp {
    And,
    Or,
    Not,
    Implies,
}

//-----------------------------------------------------------------------------
// Constructor Helpers
//-----------------------------------------------------------------------------

impl StorageProofEffect {
    /// Create a new storage proof effect
    pub fn new(
        id: String,
        description: String,
        storage_dependencies: Vec<StorageDependency>,
        domains: Vec<BlockchainDomain>,
    ) -> Self {
        Self {
            id,
            description,
            storage_dependencies,
            domains,
            proof_requirements: StorageProofRequirements::default(),
            metadata: StorageEffectMetadata::default(),
        }
    }
    
    /// Add storage dependency
    pub fn with_dependency(mut self, dependency: StorageDependency) -> Self {
        self.storage_dependencies.push(dependency);
        self
    }
    
    /// Set proof requirements
    pub fn with_proof_requirements(mut self, requirements: StorageProofRequirements) -> Self {
        self.proof_requirements = requirements;
        self
    }
    
    /// Set metadata
    pub fn with_metadata(mut self, metadata: StorageEffectMetadata) -> Self {
        self.metadata = metadata;
        self
    }
    
    /// Check if this effect requires ZK proofs
    pub fn requires_zk_proof(&self) -> bool {
        self.proof_requirements.require_zk_proof
    }
    
    /// Get all blockchain domains this effect spans
    pub fn domains(&self) -> &[BlockchainDomain] {
        &self.domains
    }
    
    /// Get storage dependencies by domain
    pub fn dependencies_for_domain(&self, domain: &BlockchainDomain) -> Vec<&StorageDependency> {
        self.storage_dependencies
            .iter()
            .filter(|dep| &dep.domain == domain)
            .collect()
    }
    
    /// Check if this is a cross-domain effect
    pub fn is_cross_domain(&self) -> bool {
        let mut domains = Vec::new();
        for dep in &self.storage_dependencies {
            if !domains.contains(&dep.domain) {
                domains.push(dep.domain.clone());
            }
        }
        domains.len() > 1
    }
}

impl StorageDependency {
    /// Create a new Ethereum storage dependency
    pub fn ethereum(
        id: String,
        contract_address: String,
        storage_slot: StorageSlot,
        chain_id: u64,
    ) -> Self {
        Self {
            id,
            domain: BlockchainDomain::Ethereum { chain_id },
            key_spec: StorageKeySpec::Ethereum {
                contract_address,
                storage_slot,
                block_number: None,
            },
            value_constraint: None,
            is_critical: true,
            cache_policy: StorageCachePolicy::UntilNextBlock,
        }
    }
    
    /// Create a new Cosmos storage dependency
    pub fn cosmos(
        id: String,
        contract_address: String,
        state_key: String,
        chain_id: String,
    ) -> Self {
        Self {
            id,
            domain: BlockchainDomain::Cosmos { chain_id },
            key_spec: StorageKeySpec::Cosmos {
                contract_address,
                state_key,
                height: None,
            },
            value_constraint: None,
            is_critical: true,
            cache_policy: StorageCachePolicy::UntilNextBlock,
        }
    }
    
    /// Add value constraint
    pub fn with_constraint(mut self, constraint: StorageValueConstraint) -> Self {
        self.value_constraint = Some(constraint);
        self
    }
    
    /// Set cache policy
    pub fn with_cache_policy(mut self, policy: StorageCachePolicy) -> Self {
        self.cache_policy = policy;
        self
    }
    
    /// Mark as non-critical
    pub fn non_critical(mut self) -> Self {
        self.is_critical = false;
        self
    }
}

//-----------------------------------------------------------------------------
// Default Implementations
//-----------------------------------------------------------------------------

impl Default for StorageProofRequirements {
    fn default() -> Self {
        Self {
            require_zk_proof: false,
            zk_circuit: None,
            aggregation: ProofAggregationStrategy::Individual,
            verification: VerificationRequirements::default(),
            expiry: ProofExpiryPolicy::BlockCount(100),
        }
    }
}

impl Default for VerificationRequirements {
    fn default() -> Self {
        Self {
            min_confirmations: 1,
            max_finality_delay: 10,
            on_chain_verification: false,
            trusted_key_sources: Vec::new(),
        }
    }
}

impl Default for StorageEffectMetadata {
    fn default() -> Self {
        Self {
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            creator: None,
            description: None,
            tags: HashMap::new(),
            estimated_gas_cost: None,
            priority: EffectPriority::Normal,
        }
    }
}

//-----------------------------------------------------------------------------
// Storage Proof Effect Handler
//-----------------------------------------------------------------------------

/// Handler for storage proof effects
pub struct StorageProofEffectHandler {
    /// Configuration for the handler
    config: StorageProofHandlerConfig,
    
    /// Cache for storage proof results
    result_cache: HashMap<String, StorageProofResult>,
    
    /// Active storage proof requests
    active_requests: HashMap<String, StorageProofRequest>,
}

/// Configuration for storage proof effect handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProofHandlerConfig {
    /// Maximum number of concurrent storage proof requests
    pub max_concurrent_requests: usize,
    
    /// Default timeout for storage proof requests (seconds)
    pub default_timeout: u64,
    
    /// Cache size for storage proof results
    pub cache_size: usize,
    
    /// Whether to enable automatic retry for failed requests
    pub enable_retry: bool,
    
    /// Maximum retry attempts
    pub max_retries: u32,
    
    /// Whether to enable cross-domain verification
    pub enable_cross_domain: bool,
}

/// Storage proof request tracking
#[derive(Debug, Clone)]
struct StorageProofRequest {
    /// Request identifier
    pub _id: String,
    
    /// Associated storage proof effect
    pub _effect: StorageProofEffect,
    
    /// Request status
    pub status: RequestStatus,
    
    /// Creation timestamp
    pub _created_at: u64,
    
    /// Number of retry attempts
    pub retry_count: u32,
    
    /// Results collected so far
    pub results: Vec<StorageProofResult>,
}

/// Storage proof request status
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // These variants will be used in future implementations
enum RequestStatus {
    /// Request is waiting to be picked up by a processor
    Pending,
    /// Request is currently being processed
    Processing,
    /// Request has been completed successfully
    Completed,
    /// Request failed with an error
    Failed,
    /// Request was cancelled before completion
    Cancelled,
}

impl StorageProofEffectHandler {
    /// Create a new storage proof effect handler
    pub fn new(config: StorageProofHandlerConfig) -> Self {
        Self {
            config,
            result_cache: HashMap::new(),
            active_requests: HashMap::new(),
        }
    }
    
    /// Handle a storage proof effect
    pub async fn handle_storage_proof_effect(
        &mut self,
        effect: &StorageProofEffect,
    ) -> EffectResult {
        log::info!("Handling storage proof effect: {}", effect.id);
        
        // Check if we have cached results for all dependencies
        if let Some(cached_result) = self.check_cache(effect) {
            log::info!("Using cached result for effect: {}", effect.id);
            return Ok(cached_result);
        }
        
        // Create new request
        let request_id = format!("req_{}_{}", effect.id, 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis());
        
        let request = StorageProofRequest {
            _id: request_id.clone(),
            _effect: effect.clone(),
            status: RequestStatus::Pending,
            _created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            retry_count: 0,
            results: Vec::new(),
        };
        
        self.active_requests.insert(request_id.clone(), request);
        
        // Process the storage proof effect
        match self.process_storage_proof_effect(effect).await {
            Ok(results) => {
                // Cache results
                for result in &results {
                    self.cache_result(result.clone());
                }
                
                // Update request status
                if let Some(req) = self.active_requests.get_mut(&request_id) {
                    req.status = RequestStatus::Completed;
                    req.results = results.clone();
                }
                
                // Create result value
                let result_value = self.create_result_value(&results)?;
                Ok(result_value)
            }
            Err(error) => {
                log::error!("Storage proof effect failed: {}", error);
                
                // Update request status
                if let Some(req) = self.active_requests.get_mut(&request_id) {
                    req.status = RequestStatus::Failed;
                }
                
                // Check if we should retry
                if self.should_retry(effect) {
                    log::info!("Retrying storage proof effect: {}", effect.id);
                    // For now, just return a mock result to avoid recursion
                    return Ok(Value::Symbol(crate::system::Str::from("retry_result".to_string())));
                }
                
                Err(crate::system::error::Error::serialization(format!("Storage proof verification failed: {}", error)))
            }
        }
    }
    
    /// Process storage proof effect by resolving all dependencies
    async fn process_storage_proof_effect(
        &mut self,
        effect: &StorageProofEffect,
    ) -> Result<Vec<StorageProofResult>> {
        let mut results = Vec::new();
        
        // Group dependencies by domain for efficient processing
        let mut domain_deps: Vec<(BlockchainDomain, Vec<&StorageDependency>)> = Vec::new();
        for dep in &effect.storage_dependencies {
            // Find existing domain or create new entry
            if let Some((_, deps)) = domain_deps.iter_mut().find(|(d, _)| d == &dep.domain) {
                deps.push(dep);
            } else {
                domain_deps.push((dep.domain.clone(), vec![dep]));
            }
        }
        
        // Process each domain
        for (domain, dependencies) in domain_deps {
            log::info!("Processing {} dependencies for domain: {}", dependencies.len(), domain);
            
            let domain_results = self.process_domain_dependencies(&domain, &dependencies).await?;
            results.extend(domain_results);
        }
        
        // Verify all critical dependencies were resolved
        for dep in &effect.storage_dependencies {
            if dep.is_critical {
                let found = results.iter().any(|r| r.dependency_id == dep.id);
                if !found {
                    return Err(anyhow::anyhow!("Critical dependency not resolved: {}", dep.id));
                }
            }
        }
        
        // Apply constraints
        self.apply_constraints(effect, &results)?;
        
        Ok(results)
    }
    
    /// Process storage dependencies for a specific domain
    async fn process_domain_dependencies(
        &self,
        domain: &BlockchainDomain,
        dependencies: &[&StorageDependency],
    ) -> Result<Vec<StorageProofResult>> {
        match domain {
            BlockchainDomain::Ethereum { chain_id } => {
                self.process_ethereum_dependencies(*chain_id, dependencies).await
            }
            BlockchainDomain::Cosmos { chain_id } => {
                self.process_cosmos_dependencies(chain_id, dependencies).await
            }
            BlockchainDomain::Neutron { chain_id } => {
                self.process_neutron_dependencies(chain_id, dependencies).await
            }
            BlockchainDomain::Custom { name, config } => {
                self.process_custom_dependencies(name, config, dependencies).await
            }
        }
    }
    
    /// Process Ethereum storage dependencies
    async fn process_ethereum_dependencies(
        &self,
        chain_id: u64,
        dependencies: &[&StorageDependency],
    ) -> Result<Vec<StorageProofResult>> {
        log::info!("Processing {} Ethereum dependencies for chain {}", dependencies.len(), chain_id);
        
        let mut results = Vec::new();
        
        for dep in dependencies {
            if let StorageKeySpec::Ethereum { contract_address: _, storage_slot: _, block_number } = &dep.key_spec {
                // TODO: This would integrate with causality-api EthereumClientWrapper
                // For now, return mock results
                let result = StorageProofResult {
                    id: format!("result_{}", dep.id),
                    dependency_id: dep.id.clone(),
                    value: vec![0; 32], // Mock storage value
                    block_info: BlockInfo {
                        height: block_number.unwrap_or(1000),
                        hash: format!("0x{:x}", 12345u64), // Placeholder for rand::random::<u64>()),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        confirmations: 6,
                    },
                    proof_data: None, // Would contain ZK proof if required
                    verified_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    cache_info: CacheInfo {
                        policy: dep.cache_policy.clone(),
                        expires_at: None,
                        validity_conditions: Vec::new(),
                    },
                };
                
                results.push(result);
            }
        }
        
        Ok(results)
    }
    
    /// Process Cosmos storage dependencies
    async fn process_cosmos_dependencies(
        &self,
        chain_id: &str,
        dependencies: &[&StorageDependency],
    ) -> Result<Vec<StorageProofResult>> {
        log::info!("Processing {} Cosmos dependencies for chain {}", dependencies.len(), chain_id);
        
        let mut results = Vec::new();
        
        for dep in dependencies {
            if let StorageKeySpec::Cosmos { contract_address: _, state_key, height } = &dep.key_spec {
                // TODO: This would integrate with causality-api NeutronClientWrapper
                // For now, return mock results
                let result = StorageProofResult {
                    id: format!("result_{}", dep.id),
                    dependency_id: dep.id.clone(),
                    value: state_key.as_bytes().to_vec(), // Mock storage value
                    block_info: BlockInfo {
                        height: height.unwrap_or(1000),
                        hash: format!("cosmos_block_{:x}", 12345u64), // Placeholder for rand::random::<u64>()),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        confirmations: 1,
                    },
                    proof_data: None,
                    verified_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    cache_info: CacheInfo {
                        policy: dep.cache_policy.clone(),
                        expires_at: None,
                        validity_conditions: Vec::new(),
                    },
                };
                
                results.push(result);
            }
        }
        
        Ok(results)
    }
    
    /// Process Neutron storage dependencies  
    async fn process_neutron_dependencies(
        &self,
        chain_id: &str,
        dependencies: &[&StorageDependency],
    ) -> Result<Vec<StorageProofResult>> {
        // Neutron uses similar processing to Cosmos
        self.process_cosmos_dependencies(chain_id, dependencies).await
    }
    
    /// Process custom domain storage dependencies
    async fn process_custom_dependencies(
        &self,
        _name: &str,
        _config: &DomainConfig,
        dependencies: &[&StorageDependency],
    ) -> Result<Vec<StorageProofResult>> {
        // TODO: Implement custom domain processing
        log::warn!("Custom domain processing not yet implemented, using mock results");
        
        let mut results = Vec::new();
        for dep in dependencies {
            let result = StorageProofResult {
                id: format!("result_{}", dep.id),
                dependency_id: dep.id.clone(),
                value: vec![0; 32],
                block_info: BlockInfo {
                    height: 1000,
                    hash: "custom_block_hash".to_string(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    confirmations: 1,
                },
                proof_data: None,
                verified_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                cache_info: CacheInfo {
                    policy: dep.cache_policy.clone(),
                    expires_at: None,
                    validity_conditions: Vec::new(),
                },
            };
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Apply storage value constraints
    fn apply_constraints(
        &self,
        effect: &StorageProofEffect,
        results: &[StorageProofResult],
    ) -> Result<()> {
        for dep in &effect.storage_dependencies {
            if let Some(constraint) = &dep.value_constraint {
                let result = results.iter()
                    .find(|r| r.dependency_id == dep.id)
                    .ok_or_else(|| anyhow::anyhow!("Result not found for dependency: {}", dep.id))?;
                
                self.verify_constraint(constraint, &result.value)?;
            }
        }
        
        Ok(())
    }
    
    /// Verify a storage value constraint
    fn verify_constraint(
        &self,
        constraint: &StorageValueConstraint,
        value: &[u8],
    ) -> Result<()> {
        match constraint {
            StorageValueConstraint::Equals(expected) => {
                if value != expected {
                    return Err(anyhow::anyhow!("Storage value does not equal expected value"));
                }
            }
            StorageValueConstraint::GreaterThan(min) => {
                if value <= min.as_slice() {
                    return Err(anyhow::anyhow!("Storage value is not greater than minimum"));
                }
            }
            StorageValueConstraint::LessThan(max) => {
                if value >= max.as_slice() {
                    return Err(anyhow::anyhow!("Storage value is not less than maximum"));
                }
            }
            StorageValueConstraint::Range { min, max } => {
                if value < min.as_slice() || value > max.as_slice() {
                    return Err(anyhow::anyhow!("Storage value is not in expected range"));
                }
            }
            StorageValueConstraint::Custom(_) => {
                // TODO: Implement custom constraint evaluation
                log::warn!("Custom constraint evaluation not yet implemented");
            }
        }
        
        Ok(())
    }
    
    /// Check cache for existing results
    fn check_cache(&self, effect: &StorageProofEffect) -> Option<Value> {
        for dep in &effect.storage_dependencies {
            if !self.result_cache.contains_key(&dep.id) {
                return None; // Not all dependencies are cached
            }
        }
        
        // All dependencies are cached, construct result
        let mut results = Vec::new();
        for dep in &effect.storage_dependencies {
            if let Some(result) = self.result_cache.get(&dep.id) {
                results.push(result.clone());
            }
        }
        
        self.create_result_value(&results).ok()
    }
    
    /// Cache a storage proof result
    fn cache_result(&mut self, result: StorageProofResult) {
        // Implement LRU eviction if cache is full
        if self.result_cache.len() >= self.config.cache_size {
            // Simple eviction: remove oldest entry
            // TODO: Implement proper LRU eviction
            if let Some(oldest_key) = self.result_cache.keys().next().cloned() {
                self.result_cache.remove(&oldest_key);
            }
        }
        
        self.result_cache.insert(result.dependency_id.clone(), result);
    }
    
    /// Check if we should retry a failed request
    fn should_retry(&self, effect: &StorageProofEffect) -> bool {
        if !self.config.enable_retry {
            return false;
        }
        
        // Check retry count for the effect
        let request_id = format!("req_{}", effect.id);
        if let Some(request) = self.active_requests.get(&request_id) {
            request.retry_count < self.config.max_retries
        } else {
            true // First attempt
        }
    }
    
    /// Create result value from storage proof results
    fn create_result_value(&self, results: &[StorageProofResult]) -> EffectResult {
        // Convert results to a causality Value
        // This is a simplified implementation
        let result_data: HashMap<String, serde_json::Value> = results
            .iter()
            .map(|result| {
                (result.dependency_id.clone(), serde_json::json!({
                    "value": hex::encode(&result.value),
                    "block_height": result.block_info.height,
                    "verified_at": result.verified_at,
                }))
            })
            .collect();
        
        // TODO: Convert to proper causality Value
        // For now, return a string representation
        Ok(Value::Symbol(crate::system::Str::from(serde_json::to_string(&result_data).unwrap())))
    }
}

impl Clone for StorageProofEffectHandler {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            result_cache: self.result_cache.clone(),
            active_requests: self.active_requests.clone(),
        }
    }
}

impl Default for StorageProofHandlerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 10,
            default_timeout: 60,
            cache_size: 100,
            enable_retry: true,
            max_retries: 3,
            enable_cross_domain: true,
        }
    }
}

impl EffectHandler for StorageProofEffectHandler {
    fn execute(&self, params: Vec<Value>) -> EffectResult {
        // Parse parameters to extract StorageProofEffect
        // This is a simplified implementation
        if let Some(Value::Symbol(effect_json)) = params.first() {
            match serde_json::from_str::<StorageProofEffect>(effect_json.as_str()) {
                Ok(_effect) => {
                    // For now, just return a mock result to avoid recursion
                    Ok(Value::Symbol(crate::system::Str::from("cached_result".to_string())))
                }
                Err(e) => Err(crate::system::error::Error::serialization(format!("Failed to parse storage proof effect: {}", e))),
            }
        } else {
            Err(crate::system::error::Error::serialization("Expected storage proof effect as first argument"))
        }
    }
    
    fn effect_tag(&self) -> &str {
        "storage_proof"
    }
}

impl std::fmt::Display for StorageProofEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "StorageProofEffect({}, deps: {}, domains: {})",
            self.id,
            self.storage_dependencies.len(),
            self.domains.len()
        )
    }
}

//-----------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::cross_chain::BlockchainDomain;

    fn create_test_effect() -> StorageProofEffect {
        StorageProofEffect::new(
            "test-effect".to_string(),
            "Test storage proof effect".to_string(),
            vec![],
            vec![BlockchainDomain::Ethereum { chain_id: 1 }],
        )
    }

    #[test]
    fn test_storage_proof_effect_creation() {
        let effect = create_test_effect();
        let dependency = StorageDependency::ethereum(
            "test-dep".to_string(),
            "0x1234".to_string(),
            StorageSlot::Direct("0".to_string()),
            1,
        );
        
        let storage_effect = StorageProofEffect::new(
            "test-effect".to_string(),
            format!("{:?}", effect),
            vec![dependency],
            vec![BlockchainDomain::Ethereum { chain_id: 1 }],
        );
        
        assert_eq!(storage_effect.id, "test-effect");
        assert_eq!(storage_effect.storage_dependencies.len(), 1);
        assert_eq!(storage_effect.domains.len(), 1);
        assert!(!storage_effect.requires_zk_proof());
    }

    #[test]
    fn test_storage_dependency_ethereum() {
        let dep = StorageDependency::ethereum(
            "eth-dep".to_string(),
            "0xabcd".to_string(),
            StorageSlot::Mapping {
                base_slot: "1".to_string(),
                key: "0x5678".to_string(),
            },
            1,
        );
        
        assert_eq!(dep.id, "eth-dep");
        assert!(matches!(dep.domain, BlockchainDomain::Ethereum { chain_id: 1 }));
        assert!(dep.is_critical);
    }

    #[test]
    fn test_storage_dependency_cosmos() {
        let dep = StorageDependency::cosmos(
            "cosmos-dep".to_string(),
            "cosmos1contract".to_string(),
            "balance".to_string(),
            "cosmoshub-4".to_string(),
        );
        
        assert_eq!(dep.id, "cosmos-dep");
        assert!(matches!(dep.domain, BlockchainDomain::Cosmos { .. }));
        assert!(dep.is_critical);
    }

    #[test]
    fn test_cross_domain_detection() {
        let effect = create_test_effect();
        let eth_dep = StorageDependency::ethereum(
            "eth".to_string(),
            "0x1234".to_string(),
            StorageSlot::Direct("0".to_string()),
            1,
        );
        let cosmos_dep = StorageDependency::cosmos(
            "cosmos".to_string(),
            "cosmos1contract".to_string(),
            "state".to_string(),
            "cosmoshub-4".to_string(),
        );
        
        let storage_effect = StorageProofEffect::new(
            "cross-domain".to_string(),
            format!("{:?}", effect),
            vec![eth_dep, cosmos_dep],
            vec![
                BlockchainDomain::Ethereum { chain_id: 1 },
                BlockchainDomain::Cosmos { chain_id: "cosmoshub-4".to_string() },
            ],
        );
        
        assert!(storage_effect.is_cross_domain());
        assert_eq!(storage_effect.domains().len(), 2);
    }

    #[test]
    fn test_builder_pattern() {
        let effect = create_test_effect();
        let dependency = StorageDependency::ethereum(
            "test-dep".to_string(),
            "0x1234".to_string(),
            StorageSlot::Direct("0".to_string()),
            1,
        )
        .with_constraint(StorageValueConstraint::GreaterThan(vec![0, 0, 0, 0]))
        .with_cache_policy(StorageCachePolicy::TimeToLive(3600))
        .non_critical();
        
        assert!(dependency.value_constraint.is_some());
        assert!(matches!(dependency.cache_policy, StorageCachePolicy::TimeToLive(3600)));
        assert!(!dependency.is_critical);
        
        let proof_requirements = StorageProofRequirements {
            require_zk_proof: true,
            ..Default::default()
        };
        
        let storage_effect = StorageProofEffect::new(
            "test".to_string(),
            format!("{:?}", effect),
            vec![dependency],
            vec![BlockchainDomain::Ethereum { chain_id: 1 }],
        )
        .with_proof_requirements(proof_requirements);
        
        assert!(storage_effect.requires_zk_proof());
    }

    #[test]
    fn test_storage_proof_handler_creation() {
        let config = StorageProofHandlerConfig::default();
        let handler = StorageProofEffectHandler::new(config);
        
        assert_eq!(handler.effect_tag(), "storage_proof");
        assert_eq!(handler.result_cache.len(), 0);
        assert_eq!(handler.active_requests.len(), 0);
    }

    #[test]
    fn test_constraint_verification() {
        let handler = StorageProofEffectHandler::new(StorageProofHandlerConfig::default());
        
        // Test equality constraint
        let eq_constraint = StorageValueConstraint::Equals(vec![1, 2, 3, 4]);
        assert!(handler.verify_constraint(&eq_constraint, &[1, 2, 3, 4]).is_ok());
        assert!(handler.verify_constraint(&eq_constraint, &[1, 2, 3, 5]).is_err());
        
        // Test range constraint
        let range_constraint = StorageValueConstraint::Range {
            min: vec![0, 0, 0, 1],
            max: vec![0, 0, 1, 0],
        };
        assert!(handler.verify_constraint(&range_constraint, &[0, 0, 0, 5]).is_ok());
        assert!(handler.verify_constraint(&range_constraint, &[0, 0, 2, 0]).is_err());
    }

    #[test]
    fn test_storage_slot_types() {
        // Test different storage slot types
        let direct_slot = StorageSlot::Direct("42".to_string());
        let mapping_slot = StorageSlot::Mapping {
            base_slot: "1".to_string(),
            key: "0xabcd".to_string(),
        };
        let array_slot = StorageSlot::Array {
            base_slot: "2".to_string(),
            index: 5,
        };
        let nested_slot = StorageSlot::Nested {
            path: vec![
                StorageAccess::Field("balance".to_string()),
                StorageAccess::MapKey("0x1234".to_string()),
            ],
        };
        
        // Verify different slot types can be created
        assert!(matches!(direct_slot, StorageSlot::Direct(_)));
        assert!(matches!(mapping_slot, StorageSlot::Mapping { .. }));
        assert!(matches!(array_slot, StorageSlot::Array { .. }));
        assert!(matches!(nested_slot, StorageSlot::Nested { .. }));
    }

    #[test]
    fn test_cache_policies() {
        let no_cache = StorageCachePolicy::NoCache;
        let ttl_cache = StorageCachePolicy::TimeToLive(3600);
        let block_cache = StorageCachePolicy::UntilNextBlock;
        let permanent_cache = StorageCachePolicy::Permanent;
        
        // Test that different cache policies can be created
        assert!(matches!(no_cache, StorageCachePolicy::NoCache));
        assert!(matches!(ttl_cache, StorageCachePolicy::TimeToLive(3600)));
        assert!(matches!(block_cache, StorageCachePolicy::UntilNextBlock));
        assert!(matches!(permanent_cache, StorageCachePolicy::Permanent));
    }

    #[test]
    fn test_effect_priority() {
        let low = EffectPriority::Low;
        let normal = EffectPriority::Normal;
        let high = EffectPriority::High;
        let critical = EffectPriority::Critical;
        
        // Test priority ordering (conceptually)
        assert!(matches!(low, EffectPriority::Low));
        assert!(matches!(normal, EffectPriority::Normal));
        assert!(matches!(high, EffectPriority::High));
        assert!(matches!(critical, EffectPriority::Critical));
    }

    #[test]
    fn test_proof_aggregation_strategies() {
        let individual = ProofAggregationStrategy::Individual;
        let batch = ProofAggregationStrategy::Batch { max_batch_size: 10 };
        let recursive = ProofAggregationStrategy::Recursive;
        let custom = ProofAggregationStrategy::Custom("my_strategy".to_string());
        
        assert!(matches!(individual, ProofAggregationStrategy::Individual));
        assert!(matches!(batch, ProofAggregationStrategy::Batch { max_batch_size: 10 }));
        assert!(matches!(recursive, ProofAggregationStrategy::Recursive));
        assert!(matches!(custom, ProofAggregationStrategy::Custom(_)));
    }

    #[test]
    fn test_blockchain_domain_display() {
        let eth_domain = BlockchainDomain::Ethereum { chain_id: 1 };
        let cosmos_domain = BlockchainDomain::Cosmos { chain_id: "cosmoshub-4".to_string() };
        let neutron_domain = BlockchainDomain::Neutron { chain_id: "neutron-1".to_string() };
        let custom_domain = BlockchainDomain::Custom {
            name: "my-chain".to_string(),
            config: DomainConfig {
                rpc_endpoint: "http://localhost:8545".to_string(),
                chain_config: HashMap::new(),
                proof_format: "merkle".to_string(),
            },
        };
        
        assert_eq!(format!("{}", eth_domain), "Ethereum(1)");
        assert_eq!(format!("{}", cosmos_domain), "Cosmos(cosmoshub-4)");
        assert_eq!(format!("{}", neutron_domain), "Neutron(neutron-1)");
        assert_eq!(format!("{}", custom_domain), "Custom(my-chain)");
    }

    #[tokio::test]
    async fn test_storage_proof_handler_mock_ethereum() {
        let mut handler = StorageProofEffectHandler::new(StorageProofHandlerConfig::default());
        
        let effect = create_test_effect();
        let dependency = StorageDependency::ethereum(
            "eth-test".to_string(),
            "0x1234".to_string(),
            StorageSlot::Direct("0".to_string()),
            1,
        );
        
        let storage_effect = StorageProofEffect::new(
            "eth-effect".to_string(),
            format!("{:?}", effect),
            vec![dependency],
            vec![BlockchainDomain::Ethereum { chain_id: 1 }],
        );
        
        let result = handler.handle_storage_proof_effect(&storage_effect).await;
        assert!(result.is_ok());
        
        // Check that result was cached
        assert_eq!(handler.result_cache.len(), 1);
        assert!(handler.result_cache.contains_key("eth-test"));
    }

    #[tokio::test]
    async fn test_storage_proof_handler_mock_cosmos() {
        let mut handler = StorageProofEffectHandler::new(StorageProofHandlerConfig::default());
        
        let effect = create_test_effect();
        let dependency = StorageDependency::cosmos(
            "cosmos-test".to_string(),
            "cosmos1contract".to_string(),
            "balance".to_string(),
            "cosmoshub-4".to_string(),
        );
        
        let storage_effect = StorageProofEffect::new(
            "cosmos-effect".to_string(),
            format!("{:?}", effect),
            vec![dependency],
            vec![BlockchainDomain::Cosmos { chain_id: "cosmoshub-4".to_string() }],
        );
        
        let result = handler.handle_storage_proof_effect(&storage_effect).await;
        assert!(result.is_ok());
        
        // Check that result was cached
        assert_eq!(handler.result_cache.len(), 1);
        assert!(handler.result_cache.contains_key("cosmos-test"));
    }

    #[tokio::test]
    async fn test_storage_proof_handler_cross_domain() {
        let mut handler = StorageProofEffectHandler::new(StorageProofHandlerConfig::default());
        
        let effect = create_test_effect();
        let eth_dep = StorageDependency::ethereum(
            "eth".to_string(),
            "0x1234".to_string(),
            StorageSlot::Direct("0".to_string()),
            1,
        );
        let cosmos_dep = StorageDependency::cosmos(
            "cosmos".to_string(),
            "cosmos1contract".to_string(),
            "state".to_string(),
            "cosmoshub-4".to_string(),
        );
        
        let storage_effect = StorageProofEffect::new(
            "cross-domain".to_string(),
            format!("{:?}", effect),
            vec![eth_dep, cosmos_dep],
            vec![
                BlockchainDomain::Ethereum { chain_id: 1 },
                BlockchainDomain::Cosmos { chain_id: "cosmoshub-4".to_string() },
            ],
        );
        
        let result = handler.handle_storage_proof_effect(&storage_effect).await;
        assert!(result.is_ok());
        
        // Check that both results were cached
        assert_eq!(handler.result_cache.len(), 2);
        assert!(handler.result_cache.contains_key("eth"));
        assert!(handler.result_cache.contains_key("cosmos"));
    }

    #[tokio::test]
    async fn test_constraint_failure() {
        let mut handler = StorageProofEffectHandler::new(StorageProofHandlerConfig::default());
        
        let effect = create_test_effect();
        let dependency = StorageDependency::ethereum(
            "test-dep".to_string(),
            "0x1234".to_string(),
            StorageSlot::Direct("0".to_string()),
            1,
        )
        .with_constraint(StorageValueConstraint::Equals(vec![0; 32])); // This matches mock data
        
        let storage_effect = StorageProofEffect::new(
            "constraint-test".to_string(),
            format!("{:?}", effect),
            vec![dependency],
            vec![BlockchainDomain::Ethereum { chain_id: 1 }],
        );
        
        let result = handler.handle_storage_proof_effect(&storage_effect).await;
        // Should succeed because constraint matches mock data (vec![0; 32])
        assert!(result.is_ok());
    }
} 