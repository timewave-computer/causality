// Domain adapter framework
// Original file: src/domain/adapter.rs

// Next-generation Domain Adapter trait for standardized fact system

use async_trait::async_trait;
use std::fmt::Debug;
use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

use causality_types::{BlockHeight, BlockHash, Timestamp};
use causality_engine_types::FactType;
use causality_types::Result;
use crate::domain::{
    DomainId, DomainInfo, DomainType, DomainStatus,
    Transaction, TransactionId, TransactionStatus, TransactionReceipt
};
use crate::effect::{Effect, EffectContext, EffectResult, EffectError, EffectOutcome};
use crate::resource::ContentId;
use crate::fact::{Fact, FactId, FactResult};

/// Fact query parameters for observing domain facts
#[derive(Debug, Clone)]
pub struct FactQuery {
    /// Domain ID to query
    pub domain_id: DomainId,
    /// Type of fact (e.g., "balance", "block", "transaction", etc.)
    pub fact_type: String,
    /// Query parameters (key-value pairs)
    pub parameters: HashMap<String, String>,
    /// Optional block height
    pub block_height: Option<BlockHeight>,
    /// Optional block hash
    pub block_hash: Option<BlockHash>,
    /// Optional timestamp
    pub timestamp: Option<Timestamp>,
}

/// Time map entry for domain time synchronization
#[derive(Debug, Clone)]
pub struct TimeMapEntry {
    /// Domain ID
    pub domain_id: DomainId,
    /// Block height
    pub height: BlockHeight,
    /// Block hash
    pub hash: BlockHash,
    /// Timestamp
    pub timestamp: Timestamp,
    /// Confidence level
    pub confidence: f64,
    /// Whether the entry is verified
    pub verified: bool,
    /// Source of the time information
    pub source: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl TimeMapEntry {
    /// Create a new time map entry
    pub fn new(domain_id: DomainId, height: BlockHeight, hash: BlockHash, timestamp: Timestamp) -> Self {
        Self {
            domain_id,
            height,
            hash,
            timestamp,
            confidence: 0.0,
            verified: false,
            source: "unknown".to_string(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set confidence level
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
    
    /// Set verification status
    pub fn with_verification(mut self, verified: bool) -> Self {
        self.verified = verified;
        self
    }
    
    /// Set source
    pub fn with_source<S: ToString>(mut self, source: S) -> Self {
        self.source = source.to_string();
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Domain adapter interface for interacting with blockchain domains
///
/// This trait provides a unified interface for interacting with
/// different blockchain domains, abstracting away their specific details.
#[async_trait]
pub trait DomainAdapter: Send + Sync + std::fmt::Debug {
    /// Get the domain ID
    fn domain_id(&self) -> &DomainId;
    
    /// Get domain information
    async fn domain_info(&self) -> Result<DomainInfo>;
    
    /// Get the current block height
    async fn current_height(&self) -> Result<BlockHeight>;
    
    /// Get the current block hash
    async fn current_hash(&self) -> Result<BlockHash>;
    
    /// Get the current time
    async fn current_time(&self) -> Result<Timestamp>;
    
    /// Get time map entry for a specific height
    async fn time_map_entry(&self, height: BlockHeight) -> Result<TimeMapEntry>;
    
    /// Observe a fact from the domain
    async fn observe_fact(&self, query: &FactQuery) -> FactResult;
    
    /// Submit a transaction to the domain
    async fn submit_transaction(&self, tx: Transaction) -> Result<TransactionId>;
    
    /// Get a transaction receipt
    async fn transaction_receipt(&self, tx_id: &TransactionId) -> Result<TransactionReceipt>;
    
    /// Check if a transaction is confirmed
    async fn transaction_confirmed(&self, tx_id: &TransactionId) -> Result<bool>;
    
    /// Wait for a transaction to be confirmed
    async fn wait_for_confirmation(
        &self, 
        tx_id: &TransactionId, 
        max_wait_ms: Option<u64>
    ) -> Result<TransactionReceipt>;
    
    /// Get capabilities of this domain adapter
    ///
    /// This method returns a list of capability strings that this adapter supports.
    /// Capabilities are standardized identifiers that describe what operations
    /// the adapter can perform. Standard capability identifiers include:
    /// - send_transaction: Ability to send transactions to the domain
    /// - sign_transaction: Ability to sign transactions for the domain
    /// - deploy_contract: Ability to deploy smart contracts
    /// - execute_contract: Ability to execute smart contract functions
    /// - query_contract: Ability to query contract state without transactions
    /// - read_state: Ability to read state from the domain
    /// - write_state: Ability to write state to the domain
    /// - zk_prove: Ability to generate zero-knowledge proofs
    /// - zk_verify: Ability to verify zero-knowledge proofs
    ///
    /// Custom capabilities should have a prefix of "custom_"
    fn capabilities(&self) -> Vec<String> {
        // Default implementation returns common capabilities
        vec![
            "send_transaction".to_string(), 
            "read_state".to_string()
        ]
    }
    
    /// Check if the domain adapter has a specific capability
    ///
    /// This method checks if the adapter supports a specific capability.
    /// The capability parameter should be a standardized capability identifier
    /// as returned by the capabilities() method.
    fn has_capability(&self, capability: &str) -> bool {
        self.capabilities().iter().any(|c| c == capability)
    }
    
    /// Calculate an estimated fee for a transaction
    async fn estimate_fee(&self, tx: &Transaction) -> Result<HashMap<String, u64>> {
        // Default implementation returns empty fee estimation
        Ok(HashMap::new())
    }
    
    /// Get the recommended gas price (for EVM-like chains)
    async fn get_gas_price(&self) -> Result<Option<u64>> {
        // Default implementation returns None
        Ok(None)
    }
}

/// Factory trait for creating domain adapters
#[async_trait]
pub trait DomainAdapterFactory: Send + Sync {
    /// Create a new domain adapter instance
    async fn create_adapter(&self, config: HashMap<String, String>) -> Result<Box<dyn DomainAdapter>>;
    
    /// Get the supported domain types
    fn supported_domain_types(&self) -> Vec<String>;
    
    /// Check if a domain type is supported
    fn supports_domain_type(&self, domain_type: &str) -> bool {
        self.supported_domain_types().iter().any(|t| t == domain_type)
    }
}

/// Domain adapter registry for managing domain adapters
#[derive(Debug, Default)]
pub struct DomainAdapterRegistry {
    /// Map of domain ID to domain adapter
    adapters: HashMap<DomainId, Box<dyn DomainAdapter>>,
}

impl DomainAdapterRegistry {
    /// Create a new domain adapter registry
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }
    
    /// Register a domain adapter
    pub fn register_adapter(&mut self, adapter: Box<dyn DomainAdapter>) -> Result<()> {
        let domain_id = adapter.domain_id().clone();
        self.adapters.insert(domain_id, adapter);
        Ok(())
    }
    
    /// Get a domain adapter by domain ID
    pub fn get_adapter(&self, domain_id: &DomainId) -> Option<&dyn DomainAdapter> {
        self.adapters.get(domain_id).map(|a| a.as_ref())
    }
    
    /// Get a list of all registered domain IDs
    pub fn domain_ids(&self) -> Vec<DomainId> {
        self.adapters.keys().cloned().collect()
    }
    
    /// Get the number of registered adapters
    pub fn adapter_count(&self) -> usize {
        self.adapters.len()
    }
    
    /// Remove a domain adapter
    pub fn remove_adapter(&mut self, domain_id: &DomainId) -> Option<Box<dyn DomainAdapter>> {
        self.adapters.remove(domain_id)
    }
}

/// Extension trait for domain adapters that support the effect system
///
/// This trait extends the domain adapter interface with methods for handling 
/// effects. Domain adapters that implement this trait can directly execute effects.
#[async_trait]
pub trait EffectHandlerAdapter: DomainAdapter {
    /// Check if this adapter can handle a specific effect type
    fn can_handle_effect(&self, effect_name: &str) -> bool;
    
    /// Execute an effect in this domain
    async fn execute_effect(&self, effect: &dyn Effect, context: &EffectContext) -> EffectResult<EffectOutcome>;
    
    /// Create an effect in this domain
    fn create_effect(&self, effect_type: &str, resource_id: ContentId, params: HashMap<String, String>) -> Result<Box<dyn Effect>>;
}

/// Composite domain adapter that delegates to multiple domain adapters
///
/// This adapter tries each adapter in sequence until it finds one that can handle
/// the requested operation. This is useful for creating a unified interface over
/// multiple domain adapters.
pub struct CompositeDomainAdapter {
    /// The adapters to delegate to
    adapters: Vec<Arc<dyn DomainAdapter>>,
    /// The primary domain ID
    primary_domain_id: DomainId,
}

impl CompositeDomainAdapter {
    /// Create a new composite domain adapter
    pub fn new(adapters: Vec<Arc<dyn DomainAdapter>>, primary_domain_id: DomainId) -> Self {
        Self {
            adapters,
            primary_domain_id,
        }
    }
    
    /// Find an adapter for a specific domain
    fn find_adapter(&self, domain_id: &DomainId) -> Option<&Arc<dyn DomainAdapter>> {
        self.adapters.iter().find(|a| a.domain_id() == domain_id)
    }
}

#[async_trait]
impl DomainAdapter for CompositeDomainAdapter {
    fn domain_id(&self) -> &DomainId {
        &self.primary_domain_id
    }
    
    async fn domain_info(&self) -> Result<DomainInfo> {
        if let Some(adapter) = self.find_adapter(&self.primary_domain_id) {
            adapter.domain_info().await
        } else {
            Err(Error::DomainNotFound(self.primary_domain_id.to_string()))
        }
    }
    
    async fn current_height(&self) -> Result<BlockHeight> {
        if let Some(adapter) = self.find_adapter(&self.primary_domain_id) {
            adapter.current_height().await
        } else {
            Err(Error::DomainNotFound(self.primary_domain_id.to_string()))
        }
    }
    
    async fn current_hash(&self) -> Result<BlockHash> {
        if let Some(adapter) = self.find_adapter(&self.primary_domain_id) {
            adapter.current_hash().await
        } else {
            Err(Error::DomainNotFound(self.primary_domain_id.to_string()))
        }
    }
    
    async fn current_time(&self) -> Result<Timestamp> {
        if let Some(adapter) = self.find_adapter(&self.primary_domain_id) {
            adapter.current_time().await
        } else {
            Err(Error::DomainNotFound(self.primary_domain_id.to_string()))
        }
    }
    
    async fn time_map_entry(&self, height: BlockHeight) -> Result<TimeMapEntry> {
        if let Some(adapter) = self.find_adapter(&self.primary_domain_id) {
            adapter.time_map_entry(height).await
        } else {
            Err(Error::DomainNotFound(self.primary_domain_id.to_string()))
        }
    }
    
    async fn observe_fact(&self, query: &FactQuery) -> FactResult {
        if let Some(adapter) = self.find_adapter(&query.domain_id) {
            adapter.observe_fact(query).await
        } else {
            Err(Error::DomainNotFound(query.domain_id.to_string()))
        }
    }
    
    async fn submit_transaction(&self, tx: Transaction) -> Result<TransactionId> {
        if let Some(adapter) = self.find_adapter(&tx.domain_id) {
            adapter.submit_transaction(tx).await
        } else {
            Err(Error::DomainNotFound(tx.domain_id.to_string()))
        }
    }
    
    async fn transaction_receipt(&self, tx_id: &TransactionId) -> Result<TransactionReceipt> {
        for adapter in &self.adapters {
            match adapter.transaction_receipt(tx_id).await {
                Ok(receipt) => return Ok(receipt),
                Err(Error::TransactionNotFound(_)) => continue,
                Err(e) => return Err(e),
            }
        }
        
        Err(Error::TransactionNotFound(tx_id.0.clone()))
    }
    
    async fn transaction_confirmed(&self, tx_id: &TransactionId) -> Result<bool> {
        for adapter in &self.adapters {
            match adapter.transaction_confirmed(tx_id).await {
                Ok(confirmed) => return Ok(confirmed),
                Err(Error::TransactionNotFound(_)) => continue,
                Err(e) => return Err(e),
            }
        }
        
        Err(Error::TransactionNotFound(tx_id.0.clone()))
    }
    
    async fn wait_for_confirmation(&self, tx_id: &TransactionId, max_wait_ms: Option<u64>) -> Result<TransactionReceipt> {
        for adapter in &self.adapters {
            match adapter.wait_for_confirmation(tx_id, max_wait_ms).await {
                Ok(receipt) => return Ok(receipt),
                Err(Error::TransactionNotFound(_)) => continue,
                Err(e) => return Err(e),
            }
        }
        
        Err(Error::TransactionNotFound(tx_id.0.clone()))
    }
}

/// Composite effect handler adapter that delegates to multiple effect handler adapters
///
/// This adapter tries each adapter in sequence until it finds one that can handle
/// the requested effect. This is useful for creating a unified interface over
/// multiple effect handler adapters.
pub struct CompositeEffectHandlerAdapter {
    /// Base composite adapter
    base: CompositeDomainAdapter,
    /// Effect handler adapters
    effect_handlers: Vec<Arc<dyn EffectHandlerAdapter>>,
}

impl CompositeEffectHandlerAdapter {
    /// Create a new composite effect handler adapter
    pub fn new(
        adapters: Vec<Arc<dyn DomainAdapter>>, 
        effect_handlers: Vec<Arc<dyn EffectHandlerAdapter>>, 
        primary_domain_id: DomainId
    ) -> Self {
        Self {
            base: CompositeDomainAdapter::new(adapters, primary_domain_id),
            effect_handlers,
        }
    }
    
    /// Find an effect handler for a specific effect type
    fn find_effect_handler(&self, effect_name: &str) -> Option<&Arc<dyn EffectHandlerAdapter>> {
        self.effect_handlers.iter().find(|h| h.can_handle_effect(effect_name))
    }
}

#[async_trait]
impl DomainAdapter for CompositeEffectHandlerAdapter {
    fn domain_id(&self) -> &DomainId {
        self.base.domain_id()
    }
    
    async fn domain_info(&self) -> Result<DomainInfo> {
        self.base.domain_info().await
    }
    
    async fn current_height(&self) -> Result<BlockHeight> {
        self.base.current_height().await
    }
    
    async fn current_hash(&self) -> Result<BlockHash> {
        self.base.current_hash().await
    }
    
    async fn current_time(&self) -> Result<Timestamp> {
        self.base.current_time().await
    }
    
    async fn time_map_entry(&self, height: BlockHeight) -> Result<TimeMapEntry> {
        self.base.time_map_entry(height).await
    }
    
    async fn observe_fact(&self, query: &FactQuery) -> FactResult {
        self.base.observe_fact(query).await
    }
    
    async fn submit_transaction(&self, tx: Transaction) -> Result<TransactionId> {
        self.base.submit_transaction(tx).await
    }
    
    async fn transaction_receipt(&self, tx_id: &TransactionId) -> Result<TransactionReceipt> {
        self.base.transaction_receipt(tx_id).await
    }
    
    async fn transaction_confirmed(&self, tx_id: &TransactionId) -> Result<bool> {
        self.base.transaction_confirmed(tx_id).await
    }
    
    async fn wait_for_confirmation(&self, tx_id: &TransactionId, max_wait_ms: Option<u64>) -> Result<TransactionReceipt> {
        self.base.wait_for_confirmation(tx_id, max_wait_ms).await
    }
}

#[async_trait]
impl EffectHandlerAdapter for CompositeEffectHandlerAdapter {
    fn can_handle_effect(&self, effect_name: &str) -> bool {
        self.effect_handlers.iter().any(|h| h.can_handle_effect(effect_name))
    }
    
    async fn execute_effect(&self, effect: &dyn Effect, context: &EffectContext) -> EffectResult<EffectOutcome> {
        if let Some(handler) = self.find_effect_handler(effect.name()) {
            handler.execute_effect(effect, context).await
        } else {
            Err(EffectError::UnsupportedEffect(effect.name().to_string()))
        }
    }
    
    fn create_effect(&self, effect_type: &str, resource_id: ContentId, params: HashMap<String, String>) -> Result<Box<dyn Effect>> {
        if let Some(handler) = self.find_effect_handler(effect_type) {
            handler.create_effect(effect_type, resource_id, params)
        } else {
            Err(Error::UnsupportedOperation(format!("No handler found for effect type: {}", effect_type)))
        }
    }
} 
