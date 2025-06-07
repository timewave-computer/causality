//! Cross-chain effect coordination
//!
//! This module implements coordination of effects across multiple blockchain domains,
//! enabling atomic execution with rollback and recovery mechanisms.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};

use crate::{
    effect::{EffectExpr, EffectExprKind},
    lambda::base::Value,
    system::{
        content_addressing::{EntityId, Timestamp},
        error::{Error, Result},
    },
};

/// Domain configuration for custom blockchain domains
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainConfig {
    /// RPC endpoint
    pub rpc_endpoint: String,
    
    /// Chain-specific configuration
    pub chain_config: HashMap<String, String>,
    
    /// Storage proof format
    pub proof_format: String,
}

impl std::hash::Hash for DomainConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.rpc_endpoint.hash(state);
        self.proof_format.hash(state);
        // Skip chain_config HashMap since it doesn't implement Hash
    }
}

/// Blockchain domain identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockchainDomain {
    /// Ethereum mainnet or testnets
    Ethereum { chain_id: u64 },
    
    /// Cosmos-based chains
    Cosmos { chain_id: String },
    
    /// Neutron (Cosmos-based with CosmWasm)
    Neutron { chain_id: String },
    
    /// Other blockchain domains
    Custom { name: String, config: DomainConfig },
}

impl BlockchainDomain {
    /// Get domain identifier string
    pub fn identifier(&self) -> String {
        match self {
            Self::Ethereum { chain_id } => format!("ethereum-{}", chain_id),
            Self::Cosmos { chain_id } => chain_id.clone(),
            Self::Neutron { chain_id } => chain_id.clone(),
            Self::Custom { name, .. } => name.clone(),
        }
    }
    
    /// Get domain type
    pub fn domain_type(&self) -> &str {
        match self {
            Self::Ethereum { .. } => "ethereum",
            Self::Cosmos { .. } => "cosmos",
            Self::Neutron { .. } => "neutron",
            Self::Custom { .. } => "custom",
        }
    }
    
    /// Check if this domain supports atomic cross-chain operations
    pub fn supports_atomic_operations(&self) -> bool {
        matches!(self, Self::Neutron { .. } | Self::Cosmos { .. })
    }
}

impl std::fmt::Display for BlockchainDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ethereum { chain_id } => write!(f, "Ethereum({})", chain_id),
            Self::Cosmos { chain_id } => write!(f, "Cosmos({})", chain_id),
            Self::Neutron { chain_id } => write!(f, "Neutron({})", chain_id),
            Self::Custom { name, .. } => write!(f, "Custom({})", name),
        }
    }
}

/// Cross-chain transaction state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossChainTxState {
    /// Transaction is being prepared
    Preparing,
    
    /// Transaction is committed on source chain
    SourceCommitted,
    
    /// Transaction is being verified
    Verifying,
    
    /// Transaction is committed on destination chain
    DestinationCommitted,
    
    /// Transaction is fully completed
    Completed,
    
    /// Transaction failed and needs rollback
    Failed(String),
    
    /// Transaction was rolled back
    RolledBack,
}

/// Cross-chain effect that spans multiple domains
#[derive(Debug, Clone)]
pub struct CrossChainEffect {
    /// Unique identifier for this cross-chain operation
    pub id: EntityId,
    
    /// Source blockchain domain
    pub source_domain: BlockchainDomain,
    
    /// Destination blockchain domain
    pub destination_domain: BlockchainDomain,
    
    /// Effect to execute on source chain
    pub source_effect: EffectExpr,
    
    /// Effect to execute on destination chain
    pub destination_effect: EffectExpr,
    
    /// Storage proof requirements for verification
    pub proof_requirements: Vec<StorageProofRequirement>,
    
    /// Current transaction state
    pub state: CrossChainTxState,
    
    /// Timeout for the operation
    pub timeout: Duration,
    
    /// Created timestamp
    pub created_at: Timestamp,
    
    /// Rollback effects in case of failure
    pub rollback_effects: Vec<EffectExpr>,
}

/// Storage proof requirement for cross-chain verification
#[derive(Debug, Clone)]
pub struct StorageProofRequirement {
    /// Source domain for the proof
    pub source_domain: BlockchainDomain,
    
    /// Storage key or query
    pub storage_key: String,
    
    /// Contract address (for smart contract storage)
    pub contract_address: Option<String>,
    
    /// Required proof type
    pub proof_type: ProofType,
    
    /// Verification constraints
    pub constraints: Vec<VerificationConstraint>,
}

/// Type of proof required
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofType {
    /// Merkle-Patricia trie proof
    MerkleProof,
    
    /// ZK proof of storage state
    ZkStorageProof,
    
    /// Aggregated multi-storage proof
    AggregatedProof,
    
    /// Custom proof type
    Custom(String),
}

/// Verification constraint for storage proofs
#[derive(Debug, Clone)]
pub struct VerificationConstraint {
    /// Type of constraint
    pub constraint_type: ConstraintType,
    
    /// Expected value or range
    pub expected_value: Value,
    
    /// Tolerance for numeric comparisons
    pub tolerance: Option<f64>,
}

/// Type of verification constraint
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintType {
    /// Value must equal expected
    Equals,
    
    /// Value must be greater than expected
    GreaterThan,
    
    /// Value must be less than expected
    LessThan,
    
    /// Value must be within range
    Range,
    
    /// Custom constraint
    Custom(String),
}

/// Cross-chain coordinator that manages atomic operations
#[derive(Debug)]
pub struct CrossChainCoordinator {
    /// Active cross-chain operations
    active_operations: HashMap<EntityId, CrossChainEffect>,
    
    /// Operation execution queue
    execution_queue: VecDeque<EntityId>,
    
    /// Proof verification cache
    proof_cache: HashMap<String, (Value, SystemTime)>,
    
    /// Cache TTL in seconds
    cache_ttl: Duration,
    
    /// Maximum concurrent operations
    max_concurrent: usize,
    
    /// Currently executing operations
    executing: HashMap<EntityId, SystemTime>,
}

impl CrossChainCoordinator {
    /// Create a new cross-chain coordinator
    pub fn new() -> Self {
        Self {
            active_operations: HashMap::new(),
            execution_queue: VecDeque::new(),
            proof_cache: HashMap::new(),
            cache_ttl: Duration::from_secs(300), // 5 minutes
            max_concurrent: 10,
            executing: HashMap::new(),
        }
    }
    
    /// Configure the coordinator
    pub fn with_config(mut self, cache_ttl: Duration, max_concurrent: usize) -> Self {
        self.cache_ttl = cache_ttl;
        self.max_concurrent = max_concurrent;
        self
    }
    
    /// Submit a cross-chain effect for execution
    pub fn submit_cross_chain_effect(&mut self, effect: CrossChainEffect) -> Result<EntityId> {
        let effect_id = effect.id;
        
        // Validate the effect
        self.validate_cross_chain_effect(&effect)?;
        
        // Add to active operations
        self.active_operations.insert(effect_id, effect);
        
        // Queue for execution
        self.execution_queue.push_back(effect_id);
        
        Ok(effect_id)
    }
    
    /// Process pending cross-chain operations
    pub fn process_operations(&mut self) -> Result<Vec<CrossChainExecutionResult>> {
        let mut results = Vec::new();
        
        // Clean up expired operations first
        self.cleanup_expired_operations();
        
        // Process operations while under concurrent limit
        while self.executing.len() < self.max_concurrent {
            if let Some(operation_id) = self.execution_queue.pop_front() {
                // Get operation state first to avoid borrowing issues
                let operation_state = self.active_operations.get(&operation_id)
                    .map(|op| op.state.clone());
                    
                if let Some(state) = operation_state {
                    let result = match state {
                        CrossChainTxState::Preparing => {
                            // Execute source effect
                            if let Some(operation) = self.active_operations.get(&operation_id) {
                                match self.execute_source_effect(operation) {
                                    Ok(_) => {
                                        // Update state
                                        if let Some(op) = self.active_operations.get_mut(&operation_id) {
                                            op.state = CrossChainTxState::SourceCommitted;
                                        }
                                        CrossChainExecutionResult {
                                            operation_id,
                                            state: CrossChainTxState::SourceCommitted,
                                            success: true,
                                            error: None,
                                            proof_data: None,
                                        }
                                    }
                                    Err(e) => {
                                        // Mark as failed
                                        if let Some(op) = self.active_operations.get_mut(&operation_id) {
                                            op.state = CrossChainTxState::Failed(e.to_string());
                                        }
                                        CrossChainExecutionResult {
                                            operation_id,
                                            state: CrossChainTxState::Failed(e.to_string()),
                                            success: false,
                                            error: Some(e.to_string()),
                                            proof_data: None,
                                        }
                                    }
                                }
                            } else {
                                continue;
                            }
                        }
                        CrossChainTxState::SourceCommitted => {
                            // Verify storage proofs
                            if let Some(operation) = self.active_operations.get(&operation_id) {
                                match self.verify_storage_proofs(operation) {
                                    Ok(proof_data) => {
                                        // Update state
                                        if let Some(op) = self.active_operations.get_mut(&operation_id) {
                                            op.state = CrossChainTxState::Verifying;
                                        }
                                        CrossChainExecutionResult {
                                            operation_id,
                                            state: CrossChainTxState::Verifying,
                                            success: true,
                                            error: None,
                                            proof_data: Some(proof_data),
                                        }
                                    }
                                    Err(e) => {
                                        // Mark as failed
                                        if let Some(op) = self.active_operations.get_mut(&operation_id) {
                                            op.state = CrossChainTxState::Failed(e.to_string());
                                        }
                                        CrossChainExecutionResult {
                                            operation_id,
                                            state: CrossChainTxState::Failed(e.to_string()),
                                            success: false,
                                            error: Some(e.to_string()),
                                            proof_data: None,
                                        }
                                    }
                                }
                            } else {
                                continue;
                            }
                        }
                        CrossChainTxState::Verifying => {
                            // Execute destination effect
                            if let Some(operation) = self.active_operations.get(&operation_id) {
                                match self.execute_destination_effect(operation) {
                                    Ok(_) => {
                                        // Update state
                                        if let Some(op) = self.active_operations.get_mut(&operation_id) {
                                            op.state = CrossChainTxState::DestinationCommitted;
                                        }
                                        CrossChainExecutionResult {
                                            operation_id,
                                            state: CrossChainTxState::DestinationCommitted,
                                            success: true,
                                            error: None,
                                            proof_data: None,
                                        }
                                    }
                                    Err(e) => {
                                        // Mark as failed
                                        if let Some(op) = self.active_operations.get_mut(&operation_id) {
                                            op.state = CrossChainTxState::Failed(e.to_string());
                                        }
                                        CrossChainExecutionResult {
                                            operation_id,
                                            state: CrossChainTxState::Failed(e.to_string()),
                                            success: false,
                                            error: Some(e.to_string()),
                                            proof_data: None,
                                        }
                                    }
                                }
                            } else {
                                continue;
                            }
                        }
                        CrossChainTxState::DestinationCommitted => {
                            // Finalize the operation
                            if let Some(op) = self.active_operations.get_mut(&operation_id) {
                                op.state = CrossChainTxState::Completed;
                            }
                            CrossChainExecutionResult {
                                operation_id,
                                state: CrossChainTxState::Completed,
                                success: true,
                                error: None,
                                proof_data: None,
                            }
                        }
                        CrossChainTxState::Failed(_) => {
                            // Execute rollback
                            if let Some(operation) = self.active_operations.get(&operation_id) {
                                let _ = self.execute_rollback(operation);
                                if let Some(op) = self.active_operations.get_mut(&operation_id) {
                                    op.state = CrossChainTxState::RolledBack;
                                }
                            }
                            CrossChainExecutionResult {
                                operation_id,
                                state: CrossChainTxState::RolledBack,
                                success: false,
                                error: Some("Operation rolled back".to_string()),
                                proof_data: None,
                            }
                        }
                        _ => {
                            // Already in final state
                            CrossChainExecutionResult {
                                operation_id,
                                state: state.clone(),
                                success: matches!(state, CrossChainTxState::Completed),
                                error: None,
                                proof_data: None,
                            }
                        }
                    };
                    
                    results.push(result);
                    
                    // Track as executing or remove if completed
                    if let Some(operation) = self.active_operations.get(&operation_id) {
                        if matches!(operation.state, CrossChainTxState::Completed | CrossChainTxState::RolledBack) {
                            self.active_operations.remove(&operation_id);
                            self.executing.remove(&operation_id);
                        } else {
                            self.executing.insert(operation_id, SystemTime::now());
                        }
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        Ok(results)
    }
    
    /// Execute a single cross-chain operation
    fn execute_cross_chain_operation(&mut self, operation: &mut CrossChainEffect) -> Result<CrossChainExecutionResult> {
        match &operation.state {
            CrossChainTxState::Preparing => {
                // Execute source effect and generate proofs
                self.execute_source_effect(operation)?;
                operation.state = CrossChainTxState::SourceCommitted;
                
                Ok(CrossChainExecutionResult {
                    operation_id: operation.id,
                    state: operation.state.clone(),
                    success: true,
                    error: None,
                    proof_data: None,
                })
            }
            
            CrossChainTxState::SourceCommitted => {
                // Verify storage proofs
                let proof_data = self.verify_storage_proofs(operation)?;
                operation.state = CrossChainTxState::Verifying;
                
                Ok(CrossChainExecutionResult {
                    operation_id: operation.id,
                    state: operation.state.clone(),
                    success: true,
                    error: None,
                    proof_data: Some(proof_data),
                })
            }
            
            CrossChainTxState::Verifying => {
                // Execute destination effect
                self.execute_destination_effect(operation)?;
                operation.state = CrossChainTxState::DestinationCommitted;
                
                Ok(CrossChainExecutionResult {
                    operation_id: operation.id,
                    state: operation.state.clone(),
                    success: true,
                    error: None,
                    proof_data: None,
                })
            }
            
            CrossChainTxState::DestinationCommitted => {
                // Finalize the operation
                operation.state = CrossChainTxState::Completed;
                
                Ok(CrossChainExecutionResult {
                    operation_id: operation.id,
                    state: operation.state.clone(),
                    success: true,
                    error: None,
                    proof_data: None,
                })
            }
            
            CrossChainTxState::Failed(_) => {
                // Execute rollback
                self.execute_rollback(operation)?;
                operation.state = CrossChainTxState::RolledBack;
                
                Ok(CrossChainExecutionResult {
                    operation_id: operation.id,
                    state: operation.state.clone(),
                    success: false,
                    error: Some("Operation rolled back".to_string()),
                    proof_data: None,
                })
            }
            
            _ => {
                // Already in final state
                Ok(CrossChainExecutionResult {
                    operation_id: operation.id,
                    state: operation.state.clone(),
                    success: matches!(operation.state, CrossChainTxState::Completed),
                    error: None,
                    proof_data: None,
                })
            }
        }
    }
    
    /// Execute the source chain effect
    fn execute_source_effect(&self, operation: &CrossChainEffect) -> Result<()> {
        // In a real implementation, this would interact with the source blockchain
        // For now, we simulate successful execution
        
        println!("Executing source effect on {:?}: {:?}", 
                operation.source_domain, operation.source_effect);
        
        // Simulate some processing time and potential failure
        match &operation.source_effect.kind {
            EffectExprKind::Perform { effect_tag, .. } => {
                if effect_tag == "failing_effect" {
                    return Err(Error::serialization("Source effect execution failed"));
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Execute the destination chain effect
    fn execute_destination_effect(&self, operation: &CrossChainEffect) -> Result<()> {
        // In a real implementation, this would interact with the destination blockchain
        // For now, we simulate successful execution
        
        println!("Executing destination effect on {:?}: {:?}", 
                operation.destination_domain, operation.destination_effect);
        
        // Check if destination domain supports atomic operations
        if !operation.destination_domain.supports_atomic_operations() {
            return Err(Error::serialization("Destination domain does not support atomic operations"));
        }
        
        Ok(())
    }
    
    /// Verify storage proofs for the operation
    fn verify_storage_proofs(&self, operation: &CrossChainEffect) -> Result<HashMap<String, Value>> {
        let mut proof_data = HashMap::new();
        
        for requirement in &operation.proof_requirements {
            let cache_key = format!("{}:{}:{}", 
                                   requirement.source_domain.identifier(),
                                   requirement.storage_key,
                                   requirement.contract_address.as_deref().unwrap_or(""));
            
            // Check cache first
            if let Some((cached_value, cached_time)) = self.proof_cache.get(&cache_key) {
                if cached_time.elapsed().unwrap_or(Duration::MAX) < self.cache_ttl {
                    proof_data.insert(requirement.storage_key.clone(), cached_value.clone());
                    continue;
                }
            }
            
            // Fetch and verify proof
            let verified_value = self.fetch_and_verify_proof(requirement)?;
            
            // Note: We can't cache here since we only have &self, but this is fine for mock implementation
            proof_data.insert(requirement.storage_key.clone(), verified_value);
        }
        
        Ok(proof_data)
    }
    
    /// Fetch and verify a storage proof
    fn fetch_and_verify_proof(&self, requirement: &StorageProofRequirement) -> Result<Value> {
        // In a real implementation, this would:
        // 1. Fetch storage proof from the source blockchain
        // 2. Verify the cryptographic proof
        // 3. Apply verification constraints
        
        println!("Fetching storage proof for {} on {:?}", 
                requirement.storage_key, requirement.source_domain);
        
        // Simulate proof verification
        match requirement.proof_type {
            ProofType::MerkleProof => {
                // Simulate Merkle proof verification
                Ok(Value::Int(42)) // Mock verified value
            }
            ProofType::ZkStorageProof => {
                // Simulate ZK proof verification
                Ok(Value::Bool(true)) // Mock verified value
            }
            ProofType::AggregatedProof => {
                // Simulate aggregated proof verification
                Ok(Value::Product(Box::new(Value::Int(1)), Box::new(Value::Int(2)))) // Mock verified value
            }
            ProofType::Custom(_) => {
                // Simulate custom proof verification
                Ok(Value::Unit)
            }
        }
    }
    
    /// Validate a cross-chain effect before execution
    fn validate_cross_chain_effect(&self, effect: &CrossChainEffect) -> Result<()> {
        // Validate domains are different
        if effect.source_domain == effect.destination_domain {
            return Err(Error::serialization("Source and destination domains must be different for cross-chain operations"));
        }
        
        // Check atomic operation support
        if !effect.source_domain.supports_atomic_operations() && 
           !effect.destination_domain.supports_atomic_operations() {
            return Err(Error::serialization("At least one domain must support atomic operations"));
        }
        
        // Validate timeout
        if effect.timeout < Duration::from_secs(1) || effect.timeout > Duration::from_secs(24 * 60 * 60) {
            return Err(Error::serialization("Timeout must be between 1 second and 24 hours"));
        }
        
        // Validate storage key
        if effect.proof_requirements.iter().any(|r| r.storage_key.is_empty()) {
            return Err(Error::serialization("Storage key cannot be empty"));
        }
        
        Ok(())
    }
    
    /// Initiate rollback for a failed operation
    fn initiate_rollback(&mut self, operation: &CrossChainEffect) -> Result<()> {
        println!("Initiating rollback for operation {}", operation.id);
        
        // Queue rollback effects for execution
        for rollback_effect in &operation.rollback_effects {
            println!("Queuing rollback effect: {:?}", rollback_effect);
        }
        
        Ok(())
    }
    
    /// Execute rollback effects
    fn execute_rollback(&self, operation: &CrossChainEffect) -> Result<()> {
        println!("Executing rollback for operation {}", operation.id);
        
        // Execute rollback effects in reverse order
        for rollback_effect in operation.rollback_effects.iter().rev() {
            println!("Executing rollback effect: {:?}", rollback_effect);
            // In a real implementation, this would execute the rollback effect
        }
        
        Ok(())
    }
    
    /// Clean up expired operations
    fn cleanup_expired_operations(&mut self) {
        let now = SystemTime::now();
        let mut expired_operations = Vec::new();
        
        for (operation_id, operation) in &self.active_operations {
            let elapsed = now.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default()
                          - Duration::from_millis(operation.created_at.millis);
            
            if elapsed > operation.timeout {
                expired_operations.push(*operation_id);
            }
        }
        
        for operation_id in expired_operations {
            if let Some(mut operation) = self.active_operations.remove(&operation_id) {
                operation.state = CrossChainTxState::Failed("Operation timeout".to_string());
                // Attempt rollback for expired operations
                let _ = self.initiate_rollback(&operation);
            }
            self.executing.remove(&operation_id);
        }
        
        // Clean up old cache entries
        self.proof_cache.retain(|_, (_, timestamp)| {
            timestamp.elapsed().unwrap_or(Duration::MAX) < self.cache_ttl
        });
    }
    
    /// Get status of a cross-chain operation
    pub fn get_operation_status(&self, operation_id: EntityId) -> Option<&CrossChainTxState> {
        self.active_operations.get(&operation_id).map(|op| &op.state)
    }
    
    /// Cancel a pending operation
    pub fn cancel_operation(&mut self, operation_id: EntityId) -> Result<()> {
        if let Some(mut operation) = self.active_operations.remove(&operation_id) {
            operation.state = CrossChainTxState::Failed("Operation cancelled".to_string());
            self.initiate_rollback(&operation)?;
            self.executing.remove(&operation_id);
            
            // Remove from queue if present
            self.execution_queue.retain(|&id| id != operation_id);
            
            Ok(())
        } else {
            Err(Error::serialization("Operation not found"))
        }
    }
    
    /// Get statistics about the coordinator
    pub fn get_statistics(&self) -> CrossChainStatistics {
        CrossChainStatistics {
            active_operations: self.active_operations.len(),
            queued_operations: self.execution_queue.len(),
            executing_operations: self.executing.len(),
            cached_proofs: self.proof_cache.len(),
        }
    }
}

/// Result of cross-chain operation execution
#[derive(Debug, Clone)]
pub struct CrossChainExecutionResult {
    /// Operation identifier
    pub operation_id: EntityId,
    
    /// Current state
    pub state: CrossChainTxState,
    
    /// Whether the step was successful
    pub success: bool,
    
    /// Error message if failed
    pub error: Option<String>,
    
    /// Proof data if applicable
    pub proof_data: Option<HashMap<String, Value>>,
}

/// Statistics about the cross-chain coordinator
#[derive(Debug, Clone)]
pub struct CrossChainStatistics {
    /// Number of active operations
    pub active_operations: usize,
    
    /// Number of queued operations
    pub queued_operations: usize,
    
    /// Number of currently executing operations
    pub executing_operations: usize,
    
    /// Number of cached proofs
    pub cached_proofs: usize,
}

impl Default for CrossChainCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl CrossChainEffect {
    /// Create a new cross-chain effect
    pub fn new(
        source_domain: BlockchainDomain,
        destination_domain: BlockchainDomain,
        source_effect: EffectExpr,
        destination_effect: EffectExpr,
    ) -> Self {
        Self {
            id: EntityId::default(),
            source_domain,
            destination_domain,
            source_effect,
            destination_effect,
            proof_requirements: Vec::new(),
            state: CrossChainTxState::Preparing,
            timeout: Duration::from_secs(3600), // 1 hour default
            created_at: Timestamp::now(),
            rollback_effects: Vec::new(),
        }
    }
    
    /// Add a storage proof requirement
    pub fn with_proof_requirement(mut self, requirement: StorageProofRequirement) -> Self {
        self.proof_requirements.push(requirement);
        self
    }
    
    /// Set timeout for the operation
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Add rollback effects
    pub fn with_rollback_effects(mut self, effects: Vec<EffectExpr>) -> Self {
        self.rollback_effects = effects;
        self
    }
}

impl StorageProofRequirement {
    /// Create a new storage proof requirement
    pub fn new(
        source_domain: BlockchainDomain,
        storage_key: String,
        proof_type: ProofType,
    ) -> Self {
        Self {
            source_domain,
            storage_key,
            contract_address: None,
            proof_type,
            constraints: Vec::new(),
        }
    }
    
    /// Set contract address for smart contract storage
    pub fn with_contract_address(mut self, address: String) -> Self {
        self.contract_address = Some(address);
        self
    }
    
    /// Add verification constraint
    pub fn with_constraint(mut self, constraint: VerificationConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }
}

impl VerificationConstraint {
    /// Create an equality constraint
    pub fn equals(value: Value) -> Self {
        Self {
            constraint_type: ConstraintType::Equals,
            expected_value: value,
            tolerance: None,
        }
    }
    
    /// Create a greater-than constraint
    pub fn greater_than(value: Value) -> Self {
        Self {
            constraint_type: ConstraintType::GreaterThan,
            expected_value: value,
            tolerance: None,
        }
    }
    
    /// Create a range constraint
    pub fn range(min: Value, _max: Value) -> Self {
        Self {
            constraint_type: ConstraintType::Range,
            expected_value: min, // Store min in expected_value
            tolerance: None,    // Could store max here or extend the struct
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        effect::EffectExpr,
        lambda::Term,
    };

    fn create_test_effect(tag: &str) -> EffectExpr {
        EffectExpr::new(crate::effect::EffectExprKind::Perform {
            effect_tag: tag.to_string(),
            args: vec![Term::var("test")],
        })
    }

    #[test]
    fn test_blockchain_domain() {
        let ethereum = BlockchainDomain::Ethereum { chain_id: 1 };
        let neutron = BlockchainDomain::Neutron { chain_id: "neutron-1".to_string() };
        
        assert_eq!(ethereum.identifier(), "ethereum-1");
        assert_eq!(ethereum.domain_type(), "ethereum");
        assert!(!ethereum.supports_atomic_operations());
        
        assert_eq!(neutron.identifier(), "neutron-1");
        assert_eq!(neutron.domain_type(), "neutron");
        assert!(neutron.supports_atomic_operations());
    }

    #[test]
    fn test_cross_chain_effect_creation() {
        let source_domain = BlockchainDomain::Ethereum { chain_id: 1 };
        let dest_domain = BlockchainDomain::Neutron { chain_id: "neutron-1".to_string() };
        
        let source_effect = create_test_effect("ethereum_transfer");
        let dest_effect = create_test_effect("neutron_mint");
        
        let cross_chain_effect = CrossChainEffect::new(
            source_domain.clone(),
            dest_domain.clone(),
            source_effect,
            dest_effect,
        );
        
        assert_eq!(cross_chain_effect.source_domain, source_domain);
        assert_eq!(cross_chain_effect.destination_domain, dest_domain);
        assert_eq!(cross_chain_effect.state, CrossChainTxState::Preparing);
        assert!(cross_chain_effect.proof_requirements.is_empty());
    }

    #[test]
    fn test_cross_chain_coordinator_creation() {
        let coordinator = CrossChainCoordinator::new();
        
        assert_eq!(coordinator.active_operations.len(), 0);
        assert_eq!(coordinator.execution_queue.len(), 0);
        assert_eq!(coordinator.max_concurrent, 10);
    }

    #[test]
    fn test_cross_chain_effect_validation() {
        let mut coordinator = CrossChainCoordinator::new();
        
        // Valid cross-chain effect
        let valid_effect = CrossChainEffect::new(
            BlockchainDomain::Ethereum { chain_id: 1 },
            BlockchainDomain::Neutron { chain_id: "neutron-1".to_string() },
            create_test_effect("source"),
            create_test_effect("dest"),
        );
        
        assert!(coordinator.validate_cross_chain_effect(&valid_effect).is_ok());
        
        // Invalid: same source and destination
        let invalid_effect = CrossChainEffect::new(
            BlockchainDomain::Ethereum { chain_id: 1 },
            BlockchainDomain::Ethereum { chain_id: 1 },
            create_test_effect("source"),
            create_test_effect("dest"),
        );
        
        assert!(coordinator.validate_cross_chain_effect(&invalid_effect).is_err());
    }

    #[test]
    fn test_storage_proof_requirement() {
        let requirement = StorageProofRequirement::new(
            BlockchainDomain::Ethereum { chain_id: 1 },
            "balances[0x123...]".to_string(),
            ProofType::MerkleProof,
        )
        .with_contract_address("0xA0b86a33E6441e4E6B9b".to_string())
        .with_constraint(VerificationConstraint::greater_than(Value::Int(100)));
        
        assert_eq!(requirement.storage_key, "balances[0x123...]");
        assert_eq!(requirement.contract_address, Some("0xA0b86a33E6441e4E6B9b".to_string()));
        assert_eq!(requirement.proof_type, ProofType::MerkleProof);
        assert_eq!(requirement.constraints.len(), 1);
    }

    #[test]
    fn test_cross_chain_operation_submission() {
        let mut coordinator = CrossChainCoordinator::new();
        
        let effect = CrossChainEffect::new(
            BlockchainDomain::Ethereum { chain_id: 1 },
            BlockchainDomain::Neutron { chain_id: "neutron-1".to_string() },
            create_test_effect("source"),
            create_test_effect("dest"),
        );
        
        let effect_id = effect.id;
        let result = coordinator.submit_cross_chain_effect(effect);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), effect_id);
        assert_eq!(coordinator.active_operations.len(), 1);
        assert_eq!(coordinator.execution_queue.len(), 1);
    }

    #[test]
    fn test_operation_processing() {
        let mut coordinator = CrossChainCoordinator::new();
        
        let effect = CrossChainEffect::new(
            BlockchainDomain::Ethereum { chain_id: 1 },
            BlockchainDomain::Neutron { chain_id: "neutron-1".to_string() },
            create_test_effect("source"),
            create_test_effect("dest"),
        );
        
        let _effect_id = coordinator.submit_cross_chain_effect(effect).unwrap();
        
        // Process operations
        let results = coordinator.process_operations().unwrap();
        
        assert!(!results.is_empty());
        assert!(results[0].success);
        assert_eq!(results[0].state, CrossChainTxState::SourceCommitted);
    }

    #[test]
    fn test_operation_cancellation() {
        let mut coordinator = CrossChainCoordinator::new();
        
        let effect = CrossChainEffect::new(
            BlockchainDomain::Ethereum { chain_id: 1 },
            BlockchainDomain::Neutron { chain_id: "neutron-1".to_string() },
            create_test_effect("source"),
            create_test_effect("dest"),
        );
        
        let effect_id = coordinator.submit_cross_chain_effect(effect).unwrap();
        
        // Cancel the operation
        let result = coordinator.cancel_operation(effect_id);
        assert!(result.is_ok());
        
        // Operation should be removed
        assert!(coordinator.get_operation_status(effect_id).is_none());
    }

    #[test]
    fn test_verification_constraints() {
        let equals_constraint = VerificationConstraint::equals(Value::Int(42));
        assert_eq!(equals_constraint.constraint_type, ConstraintType::Equals);
        assert_eq!(equals_constraint.expected_value, Value::Int(42));
        
        let gt_constraint = VerificationConstraint::greater_than(Value::Int(100));
        assert_eq!(gt_constraint.constraint_type, ConstraintType::GreaterThan);
        assert_eq!(gt_constraint.expected_value, Value::Int(100));
    }

    #[test]
    fn test_coordinator_statistics() {
        let mut coordinator = CrossChainCoordinator::new();
        
        let stats = coordinator.get_statistics();
        assert_eq!(stats.active_operations, 0);
        assert_eq!(stats.queued_operations, 0);
        assert_eq!(stats.executing_operations, 0);
        
        // Add an operation
        let effect = CrossChainEffect::new(
            BlockchainDomain::Ethereum { chain_id: 1 },
            BlockchainDomain::Neutron { chain_id: "neutron-1".to_string() },
            create_test_effect("source"),
            create_test_effect("dest"),
        );
        
        coordinator.submit_cross_chain_effect(effect).unwrap();
        
        let stats = coordinator.get_statistics();
        assert_eq!(stats.active_operations, 1);
        assert_eq!(stats.queued_operations, 1);
    }

    #[test]
    fn test_proof_type_variants() {
        let merkle = ProofType::MerkleProof;
        let zk = ProofType::ZkStorageProof;
        let aggregated = ProofType::AggregatedProof;
        let custom = ProofType::Custom("my_proof".to_string());
        
        assert_eq!(merkle, ProofType::MerkleProof);
        assert_eq!(zk, ProofType::ZkStorageProof);
        assert_eq!(aggregated, ProofType::AggregatedProof);
        assert_eq!(custom, ProofType::Custom("my_proof".to_string()));
    }

    #[test]
    fn test_cross_chain_state_transitions() {
        let mut state = CrossChainTxState::Preparing;
        assert_eq!(state, CrossChainTxState::Preparing);
        
        state = CrossChainTxState::SourceCommitted;
        assert_eq!(state, CrossChainTxState::SourceCommitted);
        
        state = CrossChainTxState::Failed("test error".to_string());
        assert_eq!(state, CrossChainTxState::Failed("test error".to_string()));
    }
} 