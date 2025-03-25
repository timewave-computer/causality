// Domain Adapter Effect Integration
//
// This module implements bidirectional integration between domain adapters and effects,
// allowing domain adapters to be used as effects and effects to leverage domain adapters.

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::effect::{Effect, EffectContext, EffectResult, EffectError, EffectOutcome};
use crate::effect_id::EffectId;
use causality_domain::{
    adapter::DomainAdapter,
    domain::{
        DomainId, DomainInfo, Transaction, TransactionId, TransactionReceipt,
        TransactionStatus
    },
    fact::{FactQuery, FactResult, Fact},
    types::{BlockHeight, BlockHash, Timestamp, Result as DomainResult, Error as DomainError}
};
use causality_crypto::ContentId;

/// Core trait for domain-specific effects
///
/// This trait extends the standard Effect trait with domain-specific
/// functionality, allowing domain operations to be used as effects.
#[async_trait]
pub trait DomainAdapterEffect: Effect {
    /// Get the domain ID this effect operates on
    fn domain_id(&self) -> &DomainId;
    
    /// Create a domain context from an effect context
    fn create_domain_context(&self, base_context: &EffectContext) -> DomainContext;
    
    /// Map domain result to effect outcome
    fn map_outcome<T: Any>(&self, domain_result: DomainResult<T>) -> EffectResult<EffectOutcome>;
}

/// Context for domain effect execution
#[derive(Debug, Clone)]
pub struct DomainContext {
    /// Domain ID
    pub domain_id: DomainId,
    
    /// Caller identity
    pub caller: Option<String>,
    
    /// Authorization tokens
    pub auth_tokens: HashMap<String, String>,
    
    /// Parameters for execution
    pub parameters: HashMap<String, String>,
    
    /// Timestamp for execution
    pub timestamp: u64,
}

impl DomainContext {
    /// Create a new domain context
    pub fn new(domain_id: DomainId) -> Self {
        Self {
            domain_id,
            caller: None,
            auth_tokens: HashMap::new(),
            parameters: HashMap::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
    
    /// Create from effect context
    pub fn from_effect_context(domain_id: DomainId, context: &EffectContext) -> Self {
        let mut domain_context = Self::new(domain_id);
        domain_context.caller = context.caller.clone();
        domain_context.parameters = context.params.clone();
        domain_context.timestamp = context.started_at.timestamp() as u64;
        domain_context
    }
    
    /// Set caller
    pub fn with_caller(mut self, caller: impl Into<String>) -> Self {
        self.caller = Some(caller.into());
        self
    }
    
    /// Add auth token
    pub fn with_auth_token(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.auth_tokens.insert(key.into(), value.into());
        self
    }
    
    /// Add parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
}

/// Domain query effect for querying facts from a domain
#[derive(Debug)]
pub struct DomainQueryEffect {
    /// Effect ID
    id: EffectId,
    
    /// Domain ID
    domain_id: DomainId,
    
    /// Query parameters
    query: FactQuery,
    
    /// Additional parameters
    parameters: HashMap<String, String>,
}

impl DomainQueryEffect {
    /// Create a new domain query effect
    pub fn new(domain_id: DomainId, query: FactQuery) -> Self {
        Self {
            id: EffectId::new(),
            domain_id,
            query,
            parameters: HashMap::new(),
        }
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Get the fact query
    pub fn query(&self) -> &FactQuery {
        &self.query
    }
}

#[async_trait]
impl Effect for DomainQueryEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "domain_query"
    }
    
    fn description(&self) -> &str {
        "Query a fact from a domain"
    }
    
    async fn execute(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // This will be implemented after the Domain Registry supports Effect Handler
        Err(EffectError::Unimplemented)
    }
}

#[async_trait]
impl DomainAdapterEffect for DomainQueryEffect {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    fn create_domain_context(&self, base_context: &EffectContext) -> DomainContext {
        DomainContext::from_effect_context(self.domain_id.clone(), base_context)
    }
    
    fn map_outcome<T: Any>(&self, domain_result: DomainResult<T>) -> EffectResult<EffectOutcome> {
        match domain_result {
            Ok(result) => {
                // Handle Fact result specifically
                if let Some(fact) = (&result as &dyn Any).downcast_ref::<Fact>() {
                    let mut outcome = EffectOutcome::success(self.id.clone());
                    
                    // Add fact data as key-value pairs
                    for (key, value) in &fact.data {
                        outcome = outcome.with_data(key, value.to_string());
                    }
                    
                    // Add fact metadata
                    outcome = outcome.with_data("fact_id", fact.id.to_string());
                    outcome = outcome.with_data("fact_type", fact.fact_type.clone());
                    
                    if let Some(height) = fact.block_height {
                        outcome = outcome.with_data("block_height", height.to_string());
                    }
                    
                    if let Some(hash) = &fact.block_hash {
                        outcome = outcome.with_data("block_hash", hex::encode(hash));
                    }
                    
                    if let Some(timestamp) = fact.timestamp {
                        outcome = outcome.with_data("timestamp", timestamp.to_string());
                    }
                    
                    Ok(outcome)
                } else {
                    // Generic success for other types
                    Ok(EffectOutcome::success(self.id.clone()))
                }
            }
            Err(err) => {
                Err(EffectError::ExecutionError(format!("Domain error: {}", err)))
            }
        }
    }
}

/// Domain transaction effect for submitting transactions to a domain
#[derive(Debug)]
pub struct DomainTransactionEffect {
    /// Effect ID
    id: EffectId,
    
    /// Domain ID
    domain_id: DomainId,
    
    /// Transaction to submit
    transaction: Transaction,
    
    /// Whether to wait for confirmation
    wait_for_confirmation: bool,
    
    /// Maximum wait time in milliseconds
    max_wait_ms: Option<u64>,
    
    /// Additional parameters
    parameters: HashMap<String, String>,
}

impl DomainTransactionEffect {
    /// Create a new domain transaction effect
    pub fn new(domain_id: DomainId, transaction: Transaction) -> Self {
        // Ensure the transaction's domain ID matches
        assert_eq!(domain_id, transaction.domain_id, 
            "Transaction domain ID must match effect domain ID");
            
        Self {
            id: EffectId::new(),
            domain_id,
            transaction,
            wait_for_confirmation: false,
            max_wait_ms: None,
            parameters: HashMap::new(),
        }
    }
    
    /// Wait for confirmation
    pub fn with_confirmation(mut self, wait: bool) -> Self {
        self.wait_for_confirmation = wait;
        self
    }
    
    /// Set maximum wait time
    pub fn with_max_wait(mut self, max_wait_ms: u64) -> Self {
        self.max_wait_ms = Some(max_wait_ms);
        self
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Get the transaction
    pub fn transaction(&self) -> &Transaction {
        &self.transaction
    }
}

#[async_trait]
impl Effect for DomainTransactionEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "domain_transaction"
    }
    
    fn description(&self) -> &str {
        "Submit a transaction to a domain"
    }
    
    async fn execute(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // This will be implemented after the Domain Registry supports Effect Handler
        Err(EffectError::Unimplemented)
    }
}

#[async_trait]
impl DomainAdapterEffect for DomainTransactionEffect {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    fn create_domain_context(&self, base_context: &EffectContext) -> DomainContext {
        DomainContext::from_effect_context(self.domain_id.clone(), base_context)
    }
    
    fn map_outcome<T: Any>(&self, domain_result: DomainResult<T>) -> EffectResult<EffectOutcome> {
        match domain_result {
            Ok(result) => {
                // Handle TransactionId result
                if let Some(tx_id) = (&result as &dyn Any).downcast_ref::<TransactionId>() {
                    let outcome = EffectOutcome::success(self.id.clone())
                        .with_data("transaction_id", tx_id.to_string());
                    
                    Ok(outcome)
                } 
                // Handle TransactionReceipt result
                else if let Some(receipt) = (&result as &dyn Any).downcast_ref::<TransactionReceipt>() {
                    let mut outcome = EffectOutcome::success(self.id.clone())
                        .with_data("transaction_id", receipt.tx_id.to_string())
                        .with_data("status", format!("{:?}", receipt.status));
                    
                    if let Some(height) = receipt.block_height {
                        outcome = outcome.with_data("block_height", height.to_string());
                    }
                    
                    if let Some(hash) = &receipt.block_hash {
                        outcome = outcome.with_data("block_hash", hex::encode(hash));
                    }
                    
                    if let Some(gas_used) = receipt.gas_used {
                        outcome = outcome.with_data("gas_used", gas_used.to_string());
                    }
                    
                    // Add log data
                    for (i, log) in receipt.logs.iter().enumerate() {
                        outcome = outcome.with_data(
                            format!("log_{}_address", i), 
                            log.address.clone()
                        );
                        outcome = outcome.with_data(
                            format!("log_{}_data", i), 
                            hex::encode(&log.data)
                        );
                    }
                    
                    // Add event data
                    for (i, event) in receipt.events.iter().enumerate() {
                        outcome = outcome.with_data(
                            format!("event_{}_type", i), 
                            event.event_type.clone()
                        );
                        outcome = outcome.with_data(
                            format!("event_{}_data", i), 
                            event.data.clone()
                        );
                    }
                    
                    Ok(outcome)
                } else {
                    // Generic success for other types
                    Ok(EffectOutcome::success(self.id.clone()))
                }
            }
            Err(err) => {
                Err(EffectError::ExecutionError(format!("Domain error: {}", err)))
            }
        }
    }
}

/// Domain time map effect for working with time synchronization between domains
#[derive(Debug)]
pub struct DomainTimeMapEffect {
    /// Effect ID
    id: EffectId,
    
    /// Domain ID
    domain_id: DomainId,
    
    /// Block height to query
    height: Option<BlockHeight>,
    
    /// Additional parameters
    parameters: HashMap<String, String>,
}

impl DomainTimeMapEffect {
    /// Create a new domain time map effect
    pub fn new(domain_id: DomainId) -> Self {
        Self {
            id: EffectId::new(),
            domain_id,
            height: None,
            parameters: HashMap::new(),
        }
    }
    
    /// With specific block height
    pub fn with_height(mut self, height: BlockHeight) -> Self {
        self.height = Some(height);
        self
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
}

#[async_trait]
impl Effect for DomainTimeMapEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "domain_time_map"
    }
    
    fn description(&self) -> &str {
        "Query time map entry from a domain"
    }
    
    async fn execute(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // This will be implemented after the Domain Registry supports Effect Handler
        Err(EffectError::Unimplemented)
    }
}

#[async_trait]
impl DomainAdapterEffect for DomainTimeMapEffect {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    fn create_domain_context(&self, base_context: &EffectContext) -> DomainContext {
        DomainContext::from_effect_context(self.domain_id.clone(), base_context)
    }
    
    fn map_outcome<T: Any>(&self, domain_result: DomainResult<T>) -> EffectResult<EffectOutcome> {
        match domain_result {
            Ok(_) => {
                // Generic success
                Ok(EffectOutcome::success(self.id.clone()))
            }
            Err(err) => {
                Err(EffectError::ExecutionError(format!("Domain error: {}", err)))
            }
        }
    }
}

/// Domain capability effect for checking domain capabilities
#[derive(Debug)]
pub struct DomainCapabilityEffect {
    /// Effect ID
    id: EffectId,
    
    /// Domain ID
    domain_id: DomainId,
    
    /// Capability to check
    capability: String,
    
    /// Additional parameters
    parameters: HashMap<String, String>,
}

impl DomainCapabilityEffect {
    /// Create a new domain capability effect
    pub fn new(domain_id: DomainId, capability: impl Into<String>) -> Self {
        Self {
            id: EffectId::new(),
            domain_id,
            capability: capability.into(),
            parameters: HashMap::new(),
        }
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Get the capability
    pub fn capability(&self) -> &str {
        &self.capability
    }
}

#[async_trait]
impl Effect for DomainCapabilityEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "domain_capability"
    }
    
    fn description(&self) -> &str {
        "Check domain capability"
    }
    
    async fn execute(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // This will be implemented after the Domain Registry supports Effect Handler
        Err(EffectError::Unimplemented)
    }
}

#[async_trait]
impl DomainAdapterEffect for DomainCapabilityEffect {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    fn create_domain_context(&self, base_context: &EffectContext) -> DomainContext {
        DomainContext::from_effect_context(self.domain_id.clone(), base_context)
    }
    
    fn map_outcome<T: Any>(&self, domain_result: DomainResult<T>) -> EffectResult<EffectOutcome> {
        match domain_result {
            Ok(result) => {
                // Handle boolean result
                if let Some(has_capability) = (&result as &dyn Any).downcast_ref::<bool>() {
                    let outcome = EffectOutcome::success(self.id.clone())
                        .with_data("has_capability", has_capability.to_string());
                    
                    Ok(outcome)
                } else {
                    // Generic success for other types
                    Ok(EffectOutcome::success(self.id.clone()))
                }
            }
            Err(err) => {
                Err(EffectError::ExecutionError(format!("Domain error: {}", err)))
            }
        }
    }
}

// Utility functions for working with domain effects

/// Create a domain query effect
pub fn query_domain_fact(domain_id: DomainId, fact_type: impl Into<String>) -> DomainQueryEffect {
    let mut query = FactQuery {
        domain_id: domain_id.clone(),
        fact_type: fact_type.into(),
        parameters: HashMap::new(),
        block_height: None,
        block_hash: None,
        timestamp: None,
    };
    
    DomainQueryEffect::new(domain_id, query)
}

/// Create a domain transaction effect
pub fn submit_domain_transaction(domain_id: DomainId, tx_type: impl Into<String>) -> DomainTransactionEffect {
    let transaction = Transaction {
        domain_id: domain_id.clone(),
        tx_type: tx_type.into(),
        sender: None,
        target: None,
        data: Vec::new(),
        gas_limit: None,
        gas_price: None,
        nonce: None,
        signature: None,
        metadata: HashMap::new(),
    };
    
    DomainTransactionEffect::new(domain_id, transaction)
}

/// Create a domain time map effect
pub fn get_domain_time_map(domain_id: DomainId) -> DomainTimeMapEffect {
    DomainTimeMapEffect::new(domain_id)
}

/// Create a domain capability check effect
pub fn check_domain_capability(domain_id: DomainId, capability: impl Into<String>) -> DomainCapabilityEffect {
    DomainCapabilityEffect::new(domain_id, capability)
}

// TODO: Implement domain registry integration with effect system 