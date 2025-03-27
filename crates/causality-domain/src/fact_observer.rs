// Fact observer system for domains
// Original file: src/domain/fact_observer.rs

// Domain Fact Observer System
//
// This module implements a system for observing facts from different domains
// and integrating them with the effect system.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::any::Any;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use borsh::{BorshSerialize, BorshDeserialize};

use crate::domain::{DomainId, DomainAdapter, FactQuery, FactType, FactObservationMeta};
use causality_types::{Error, Result};
use crate::effect::{Effect, EffectContext, EffectResult, EffectOutcome};
use :EffectRuntime:causality_core::effect::runtime::EffectRuntime::ExecutionBoundary;
use crate::crypto::{HashFactory, ContentId};
use causality_crypto::ContentAddressed;

/// A dependency on a domain fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactDependency {
    /// Type of the fact
    pub fact_type: String,
    
    /// Parameters for the fact query
    pub parameters: HashMap<String, String>,
    
    /// Domain ID where the fact should be observed
    pub domain_id: Option<String>,
}

/// A snapshot of a fact at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactSnapshot {
    /// Type of the fact
    pub fact_type: String,
    
    /// Parameters used to query the fact
    pub parameters: HashMap<String, String>,
    
    /// Value of the fact (as a string)
    pub value: String,
    
    /// Timestamp when the fact was observed
    pub timestamp: u64,
    
    /// Source of the fact
    pub source: String,
    
    /// Domain ID where the fact was observed
    pub domain_id: Option<String>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// A fact observed from a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainFact {
    /// ID of the domain this fact was observed from
    pub domain_id: DomainId,
    
    /// Type of the fact
    pub fact_type: FactType,
    
    /// Metadata about the fact observation
    pub meta: FactObservationMeta,
    
    /// Original query that was used to observe this fact
    pub query: String,
    
    /// Parameters used in the query
    pub parameters: HashMap<String, String>,
    
    /// Hash of the fact (for verification)
    pub hash: Option<String>,
}

impl DomainFact {
    /// Create a new domain fact
    pub fn new(
        domain_id: DomainId,
        fact_type: FactType,
        meta: FactObservationMeta,
        query: impl Into<String>,
        parameters: HashMap<String, String>,
    ) -> Self {
        Self {
            domain_id,
            fact_type,
            meta,
            query: query.into(),
            parameters,
            hash: None,
        }
    }
    
    /// Set the hash for this fact
    pub fn with_hash(mut self, hash: impl Into<String>) -> Self {
        self.hash = Some(hash.into());
        self
    }
    
    /// Get a value from this fact as a string
    pub fn value_as_string(&self) -> Option<String> {
        match &self.fact_type {
            FactType::Boolean(b) => Some(b.to_string()),
            FactType::Numeric(n) => Some(n.to_string()),
            FactType::String(s) => Some(s.clone()),
            FactType::Binary(b) => Some(hex::encode(b)),
            FactType::Json(j) => Some(j.to_string()),
        }
    }
    
    /// Convert this fact to a fact dependency for the effect system
    pub fn to_fact_dependency(&self) -> FactDependency {
        FactDependency {
            fact_type: self.query.clone(),
            parameters: self.parameters.clone(),
            domain_id: Some(self.domain_id.to_string()),
        }
    }
    
    /// Convert this fact to a fact snapshot for the effect system
    pub fn to_fact_snapshot(&self) -> FactSnapshot {
        FactSnapshot {
            fact_type: self.query.clone(),
            parameters: self.parameters.clone(),
            value: self.value_as_string().unwrap_or_default(),
            timestamp: self.meta.observed_at.as_u64(),
            source: self.meta.source.clone(),
            domain_id: Some(self.domain_id.to_string()),
            metadata: self.meta.metadata.clone(),
        }
    }
}

/// Trait for observing facts from domains
#[async_trait]
pub trait DomainFactObserver: Send + Sync {
    /// Observe a fact from a domain
    async fn observe_fact(&self, query: FactQuery) -> Result<DomainFact>;
    
    /// Check if this observer supports observing a specific fact type
    fn supports_fact_type(&self, fact_type: &str) -> bool;
    
    /// Get the domain ID this observer is associated with
    fn domain_id(&self) -> &DomainId;
    
    /// Get the supported fact types
    fn supported_fact_types(&self) -> HashSet<String> {
        HashSet::new()
    }
}

/// A fact observer for a specific domain adapter
pub struct AdapterFactObserver {
    /// The domain adapter to use for fact observation
    adapter: Arc<dyn DomainAdapter>,
    
    /// Supported fact types
    fact_types: HashSet<String>,
}

impl AdapterFactObserver {
    /// Create a new adapter fact observer
    pub fn new(adapter: Arc<dyn DomainAdapter>) -> Self {
        Self {
            adapter,
            fact_types: HashSet::new(),
        }
    }
    
    /// Create a new adapter fact observer with specific supported fact types
    pub fn with_fact_types(adapter: Arc<dyn DomainAdapter>, fact_types: HashSet<String>) -> Self {
        Self {
            adapter,
            fact_types,
        }
    }
    
    /// Add a supported fact type
    pub fn add_fact_type(&mut self, fact_type: impl Into<String>) {
        self.fact_types.insert(fact_type.into());
    }
}

#[async_trait]
impl DomainFactObserver for AdapterFactObserver {
    async fn observe_fact(&self, query: FactQuery) -> Result<DomainFact> {
        // Ensure the query is for the correct domain
        if query.domain_id != *self.adapter.domain_id() {
            return Err(Error::InvalidArgument(format!(
                "Query domain ID '{}' does not match observer domain ID '{}'", 
                query.domain_id, self.adapter.domain_id()
            )));
        }
        
        // Check if this fact type is supported
        if !self.supports_fact_type(&query.fact_type) {
            return Err(Error::UnsupportedOperation(format!(
                "Fact type '{}' is not supported by this observer", 
                query.fact_type
            )));
        }
        
        // Observe the fact from the adapter
        let (fact_type, meta) = self.adapter.observe_fact(&query).await?;
        
        // Create a domain fact
        Ok(DomainFact::new(
            query.domain_id.clone(),
            fact_type,
            meta,
            query.fact_type.clone(),
            query.parameters,
        ))
    }
    
    fn supports_fact_type(&self, fact_type: &str) -> bool {
        if self.fact_types.is_empty() {
            // If no specific fact types are defined, assume all are supported
            true
        } else {
            self.fact_types.contains(fact_type)
        }
    }
    
    fn domain_id(&self) -> &DomainId {
        self.adapter.domain_id()
    }
    
    fn supported_fact_types(&self) -> HashSet<String> {
        self.fact_types.clone()
    }
}

/// Registry for domain fact observers
pub struct DomainFactObserverRegistry {
    /// Map of domain ID to fact observer
    observers: RwLock<HashMap<DomainId, Arc<dyn DomainFactObserver>>>,
    
    /// Map of fact type to observers that support it
    fact_type_map: RwLock<HashMap<String, Vec<DomainId>>>,
}

impl DomainFactObserverRegistry {
    /// Create a new domain fact observer registry
    pub fn new() -> Self {
        Self {
            observers: RwLock::new(HashMap::new()),
            fact_type_map: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a domain fact observer
    pub fn register_observer(&self, observer: Arc<dyn DomainFactObserver>) -> Result<()> {
        let domain_id = observer.domain_id().clone();
        
        // Register the observer
        {
            let mut observers = self.observers.write().map_err(|_| {
                Error::ConcurrencyError("Failed to acquire write lock on observers".to_string())
            })?;
            
            observers.insert(domain_id.clone(), observer.clone());
        }
        
        // Update the fact type map
        let supported_types = observer.supported_fact_types();
        let mut fact_type_map = self.fact_type_map.write().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire write lock on fact type map".to_string())
        })?;
        
        for fact_type in supported_types {
            let domains = fact_type_map.entry(fact_type).or_insert_with(Vec::new);
            if !domains.contains(&domain_id) {
                domains.push(domain_id.clone());
            }
        }
        
        Ok(())
    }
    
    /// Unregister a domain fact observer
    pub fn unregister_observer(&self, domain_id: &DomainId) -> Result<()> {
        // Remove the observer
        let removed_observer = {
            let mut observers = self.observers.write().map_err(|_| {
                Error::ConcurrencyError("Failed to acquire write lock on observers".to_string())
            })?;
            
            observers.remove(domain_id)
        };
        
        // Update the fact type map if an observer was removed
        if let Some(observer) = removed_observer {
            let supported_types = observer.supported_fact_types();
            let mut fact_type_map = self.fact_type_map.write().map_err(|_| {
                Error::ConcurrencyError("Failed to acquire write lock on fact type map".to_string())
            })?;
            
            for fact_type in supported_types {
                if let Some(domains) = fact_type_map.get_mut(&fact_type) {
                    domains.retain(|id| id != domain_id);
                    
                    // Remove the entry if no domains support this fact type
                    if domains.is_empty() {
                        fact_type_map.remove(&fact_type);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get a domain fact observer
    pub fn get_observer(&self, domain_id: &DomainId) -> Result<Arc<dyn DomainFactObserver>> {
        let observers = self.observers.read().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire read lock on observers".to_string())
        })?;
        
        observers.get(domain_id)
            .cloned()
            .ok_or_else(|| Error::DomainNotFound(domain_id.clone()))
    }
    
    /// Get all observers that support a specific fact type
    pub fn get_observers_for_fact_type(&self, fact_type: &str) -> Result<Vec<Arc<dyn DomainFactObserver>>> {
        let fact_type_map = self.fact_type_map.read().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire read lock on fact type map".to_string())
        })?;
        
        let domain_ids = fact_type_map.get(fact_type)
            .cloned()
            .unwrap_or_default();
            
        let observers = self.observers.read().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire read lock on observers".to_string())
        })?;
        
        let mut result = Vec::new();
        for domain_id in domain_ids {
            if let Some(observer) = observers.get(&domain_id) {
                result.push(observer.clone());
            }
        }
        
        Ok(result)
    }
    
    /// Observe a fact from a specific domain
    pub async fn observe_fact(&self, query: FactQuery) -> Result<DomainFact> {
        let observer = self.get_observer(&query.domain_id)?;
        observer.observe_fact(query).await
    }
    
    /// Observe facts from all domains that support a specific fact type
    pub async fn observe_facts(&self, fact_type: &str, parameters: HashMap<String, String>) -> Result<Vec<DomainFact>> {
        let observers = self.get_observers_for_fact_type(fact_type)?;
        
        let mut results = Vec::new();
        for observer in observers {
            let query = FactQuery {
                domain_id: observer.domain_id().clone(),
                fact_type: fact_type.to_string(),
                parameters: parameters.clone(),
            };
            
            match observer.observe_fact(query).await {
                Ok(fact) => results.push(fact),
                Err(err) => {
                    // Log the error but continue with other observers
                    eprintln!("Error observing fact from domain {}: {}", observer.domain_id(), err);
                }
            }
        }
        
        Ok(results)
    }
}

/// Trait extension for domain adapters to create fact observers
pub trait DomainFactObserverExtension {
    /// Create a fact observer for this domain adapter
    fn create_fact_observer(&self) -> AdapterFactObserver;
}

impl<T: DomainAdapter> DomainFactObserverExtension for T {
    fn create_fact_observer(&self) -> AdapterFactObserver {
        AdapterFactObserver::new(Arc::new(ClonableDomainAdapter(self)))
    }
}

/// A wrapper to make domain adapters clonable for the fact observer
/// This isn't a full clone but rather just creates a clonable wrapper for the Arc
#[derive(Debug)]
struct ClonableDomainAdapter<'a>(&'a dyn DomainAdapter);

impl<'a> DomainAdapter for ClonableDomainAdapter<'a> {
    fn domain_id(&self) -> &DomainId {
        self.0.domain_id()
    }
    
    async fn domain_info(&self) -> Result<crate::domain::DomainInfo> {
        self.0.domain_info().await
    }
    
    async fn current_height(&self) -> Result<crate::domain::BlockHeight> {
        self.0.current_height().await
    }
    
    async fn current_hash(&self) -> Result<crate::domain::BlockHash> {
        self.0.current_hash().await
    }
    
    async fn current_time(&self) -> Result<crate::domain::Timestamp> {
        self.0.current_time().await
    }
    
    async fn time_map_entry(&self, height: crate::domain::BlockHeight) -> Result<crate::domain::TimeMapEntry> {
        self.0.time_map_entry(height).await
    }
    
    async fn observe_fact(&self, query: &FactQuery) -> crate::domain::FactResult {
        self.0.observe_fact(query).await
    }
    
    async fn submit_transaction(&self, tx: crate::domain::Transaction) -> Result<crate::domain::TransactionId> {
        self.0.submit_transaction(tx).await
    }
    
    async fn transaction_receipt(&self, tx_id: &crate::domain::TransactionId) -> Result<crate::domain::TransactionReceipt> {
        self.0.transaction_receipt(tx_id).await
    }
    
    async fn transaction_confirmed(&self, tx_id: &crate::domain::TransactionId) -> Result<bool> {
        self.0.transaction_confirmed(tx_id).await
    }
    
    async fn wait_for_confirmation(
        &self, 
        tx_id: &crate::domain::TransactionId, 
        max_wait_ms: Option<u64>
    ) -> Result<crate::domain::TransactionReceipt> {
        self.0.wait_for_confirmation(tx_id, max_wait_ms).await
    }
    
    fn capabilities(&self) -> Vec<String> {
        self.0.capabilities()
    }
    
    fn has_capability(&self, capability: &str) -> bool {
        self.0.has_capability(capability)
    }
    
    async fn estimate_fee(&self, tx: &crate::domain::Transaction) -> Result<HashMap<String, u64>> {
        self.0.estimate_fee(tx).await
    }
    
    async fn get_gas_price(&self) -> Result<Option<u64>> {
        self.0.get_gas_price().await
    }
}

/// An effect that depends on domain facts
///
/// This trait is implemented by effects that depend on domain facts.
/// It provides methods for defining dependencies on domain facts and
/// validating those dependencies.
pub trait DomainFactEffect: Effect {
    /// Get the domain fact dependencies for this effect
    fn domain_fact_dependencies(&self) -> Vec<FactDependency> {
        Vec::new()
    }
    
    /// Validate domain fact dependencies for this effect
    async fn validate_domain_fact_dependencies(
        &self, 
        registry: &DomainFactObserverRegistry
    ) -> Result<Vec<DomainFact>> {
        let mut facts = Vec::new();
        
        for dependency in self.domain_fact_dependencies() {
            let domain_id = if let Some(domain_id) = &dependency.domain_id {
                DomainId::new(domain_id)
            } else {
                // Skip dependencies without a domain ID
                continue;
            };
            
            let query = FactQuery {
                domain_id: domain_id.clone(),
                fact_type: dependency.fact_type.clone(),
                parameters: dependency.parameters.clone(),
            };
            
            // Observe the fact
            let fact = registry.observe_fact(query).await?;
            facts.push(fact);
        }
        
        Ok(facts)
    }
}

/// Effect for observing a domain fact
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ObserveDomainFactEffect {
    /// Effect ID
    id: ContentId,
    
    /// Fact query to observe
    query: FactQuery,
    
    /// Whether to cache the result
    cache_result: bool,
}

impl ContentAddressed for ObserveDomainFactEffect {
    fn content_hash(&self) -> crate::crypto::HashOutput {
        // Get the default hasher
        let hash_factory = HashFactory::default();
        let hasher = hash_factory.create_hasher().expect("Failed to create hasher");
        
        // Create a canonical serialization of the effect
        let data = self.to_bytes();
        
        // Compute hash with configured hasher
        hasher.hash(&data)
    }
    
    fn verify(&self) -> bool {
        let id = self.content_id();
        id == self.id
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        borsh::to_vec(self).unwrap_or_default()
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, crate::crypto::HashError> {
        borsh::from_slice(bytes)
            .map_err(|e| crate::crypto::HashError::SerializationError(e.to_string()))
    }
}

impl ObserveDomainFactEffect {
    /// Create a new effect to observe a domain fact
    pub fn new(query: FactQuery) -> Self {
        let effect = Self {
            id: ContentId::nil(), // Temporary ID
            query,
            cache_result: true,
        };
        
        // Generate a content-based ID
        let id = effect.content_id();
        
        Self {
            id,
            query: effect.query,
            cache_result: effect.cache_result,
        }
    }
    
    /// Configure whether to cache the result
    pub fn with_cache(mut self, cache_result: bool) -> Self {
        self.cache_result = cache_result;
        
        // Regenerate ID after changing the cache setting
        let id = self.content_id();
        self.id = id;
        
        self
    }
}

impl Effect for ObserveDomainFactEffect {
    fn id(&self) -> &ContentId {
        &self.id
    }
    
    fn boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::Internal
    }
    
    async fn execute(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Get the fact observer registry from the context
        let registry = context.params.get("observer_registry")
            .ok_or_else(|| crate::effect::EffectError::InvalidParameter(
                "Missing observer_registry parameter".to_string()
            ))?;
        
        // Parse the registry from the param
        let registry: &DomainFactObserverRegistry = registry.downcast_ref()
            .ok_or_else(|| crate::effect::EffectError::InvalidParameter(
                "Invalid observer_registry parameter".to_string()
            ))?;
        
        // Observe the fact
        match registry.observe_fact(self.query.clone()).await {
            Ok(fact) => {
                // Create outcome with the observed fact data
                let mut outcome = EffectOutcome::success(self.id.clone());
                
                // Add general data
                outcome = outcome.with_data("domain_id", fact.domain_id.to_string())
                    .with_data("fact_type", fact.fact_type.to_string());
                
                // Add specific value based on fact type
                if let Some(value) = fact.value_as_string() {
                    outcome = outcome.with_data("value", value);
                }
                
                // Add observation metadata
                outcome = outcome.with_data("observation_time", fact.meta.observed_at.to_string())
                    .with_data("source", fact.meta.source.clone());
                
                Ok(outcome)
            },
            Err(err) => {
                // Create failure outcome
                let outcome = EffectOutcome::failure(
                    self.id.clone(),
                    format!("Failed to observe fact: {}", err)
                );
                
                Ok(outcome)
            }
        }
    }
    
    fn description(&self) -> String {
        format!("Observe fact {} from domain {}", self.query.fact_type, self.query.domain_id)
    }
    
    async fn validate(&self, _context: &EffectContext) -> EffectResult<()> {
        // Basic validation: ensure domain ID and fact type are not empty
        if self.query.domain_id.is_empty() {
            return Err(crate::effect::EffectError::InvalidParameter(
                "Domain ID cannot be empty".to_string()
            ));
        }
        
        if self.query.fact_type.is_empty() {
            return Err(crate::effect::EffectError::InvalidParameter(
                "Fact type cannot be empty".to_string()
            ));
        }
        
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DomainInfo, DomainType, DomainStatus, BlockHeight, BlockHash, Timestamp};
    
    // Mock implementation of DomainAdapter for testing
    #[derive(Debug)]
    struct MockDomainAdapter {
        domain_id: DomainId,
        facts: HashMap<String, (FactType, FactObservationMeta)>,
    }
    
    impl MockDomainAdapter {
        fn new(domain_id: &str) -> Self {
            Self {
                domain_id: DomainId::new(domain_id),
                facts: HashMap::new(),
            }
        }
        
        fn with_fact(mut self, fact_type: &str, value: impl Into<FactType>) -> Self {
            let meta = FactObservationMeta {
                observed_at: Timestamp::now(),
                block_height: Some(BlockHeight(100)),
                reliability: 1.0,
                source: "mock".to_string(),
                metadata: HashMap::new(),
            };
            
            self.facts.insert(fact_type.to_string(), (value.into(), meta));
            self
        }
    }
    
    #[async_trait]
    impl DomainAdapter for MockDomainAdapter {
        fn domain_id(&self) -> &DomainId {
            &self.domain_id
        }
        
        async fn domain_info(&self) -> Result<DomainInfo> {
            Ok(DomainInfo {
                domain_id: self.domain_id.clone(),
                name: format!("Mock Domain {}", self.domain_id),
                domain_type: DomainType::Unknown,
                status: DomainStatus::Active,
                metadata: HashMap::new(),
            })
        }
        
        async fn current_height(&self) -> Result<BlockHeight> {
            Ok(BlockHeight(100))
        }
        
        async fn current_hash(&self) -> Result<BlockHash> {
            Ok(BlockHash([0; 32]))
        }
        
        async fn current_time(&self) -> Result<Timestamp> {
            Ok(Timestamp::now())
        }
        
        async fn time_map_entry(&self, _height: BlockHeight) -> Result<crate::domain::TimeMapEntry> {
            unimplemented!()
        }
        
        async fn observe_fact(&self, query: &FactQuery) -> crate::domain::FactResult {
            // Check if we have this fact type
            if let Some((fact_type, meta)) = self.facts.get(&query.fact_type) {
                Ok((fact_type.clone(), meta.clone()))
            } else {
                Err(Error::FactNotFound(query.fact_type.clone()))
            }
        }
        
        async fn submit_transaction(&self, _tx: crate::domain::Transaction) -> Result<crate::domain::TransactionId> {
            unimplemented!()
        }
        
        async fn transaction_receipt(&self, _tx_id: &crate::domain::TransactionId) -> Result<crate::domain::TransactionReceipt> {
            unimplemented!()
        }
        
        async fn transaction_confirmed(&self, _tx_id: &crate::domain::TransactionId) -> Result<bool> {
            unimplemented!()
        }
        
        async fn wait_for_confirmation(
            &self, 
            _tx_id: &crate::domain::TransactionId, 
            _max_wait_ms: Option<u64>
        ) -> Result<crate::domain::TransactionReceipt> {
            unimplemented!()
        }
    }
    
    #[tokio::test]
    async fn test_adapter_fact_observer() {
        // Create a mock adapter with some facts
        let adapter = MockDomainAdapter::new("test-domain-1")
            .with_fact("balance", FactType::Numeric(100))
            .with_fact("name", FactType::String("Test Account".to_string()));
            
        // Create a fact observer
        let observer = AdapterFactObserver::new(Arc::new(adapter));
        
        // Create a query
        let query = FactQuery {
            domain_id: DomainId::new("test-domain-1"),
            fact_type: "balance".to_string(),
            parameters: HashMap::new(),
        };
        
        // Observe the fact
        let fact = observer.observe_fact(query).await.unwrap();
        
        // Verify the result
        assert_eq!(fact.domain_id, DomainId::new("test-domain-1"));
        match &fact.fact_type {
            FactType::Numeric(value) => assert_eq!(*value, 100),
            _ => panic!("Expected Numeric fact"),
        }
        
        // Check value as string
        assert_eq!(fact.value_as_string(), Some("100".to_string()));
    }
    
    #[tokio::test]
    async fn test_fact_observer_registry() {
        // Create a registry
        let registry = DomainFactObserverRegistry::new();
        
        // Create some mock adapters
        let adapter1 = MockDomainAdapter::new("test-domain-1")
            .with_fact("balance", FactType::Numeric(100))
            .with_fact("name", FactType::String("Test Account 1".to_string()));
            
        let adapter2 = MockDomainAdapter::new("test-domain-2")
            .with_fact("balance", FactType::Numeric(200))
            .with_fact("name", FactType::String("Test Account 2".to_string()));
            
        // Create fact observers
        let observer1 = AdapterFactObserver::with_fact_types(
            Arc::new(adapter1),
            HashSet::from(["balance".to_string(), "name".to_string()]),
        );
        
        let observer2 = AdapterFactObserver::with_fact_types(
            Arc::new(adapter2),
            HashSet::from(["balance".to_string(), "name".to_string()]),
        );
        
        // Register the observers
        registry.register_observer(Arc::new(observer1)).unwrap();
        registry.register_observer(Arc::new(observer2)).unwrap();
        
        // Get observers for a fact type
        let balance_observers = registry.get_observers_for_fact_type("balance").unwrap();
        assert_eq!(balance_observers.len(), 2);
        
        // Observe a fact from a specific domain
        let query = FactQuery {
            domain_id: DomainId::new("test-domain-1"),
            fact_type: "balance".to_string(),
            parameters: HashMap::new(),
        };
        
        let fact = registry.observe_fact(query).await.unwrap();
        match &fact.fact_type {
            FactType::Numeric(value) => assert_eq!(*value, 100),
            _ => panic!("Expected Numeric fact"),
        }
        
        // Observe facts from all domains
        let facts = registry.observe_facts("balance", HashMap::new()).await.unwrap();
        assert_eq!(facts.len(), 2);
        
        // Verify the results
        let values: Vec<i64> = facts.iter()
            .filter_map(|fact| {
                if let FactType::Numeric(value) = &fact.fact_type {
                    Some(*value)
                } else {
                    None
                }
            })
            .collect();
            
        assert!(values.contains(&100));
        assert!(values.contains(&200));
    }
    
    #[tokio::test]
    async fn test_observe_domain_fact_effect() {
        // Create a mock adapter with some facts
        let adapter = MockDomainAdapter::new("test-domain-1")
            .with_fact("balance", FactType::Numeric(100));
            
        // Create a fact observer
        let observer = AdapterFactObserver::new(Arc::new(adapter));
        
        // Create a registry
        let registry = Arc::new(DomainFactObserverRegistry::new());
        registry.register_observer(Arc::new(observer)).unwrap();
        
        // Create an effect context
        let mut context = EffectContext::new();
        context.add_service(registry);
        
        // Create an observe domain fact effect
        let query = FactQuery {
            domain_id: DomainId::new("test-domain-1"),
            fact_type: "balance".to_string(),
            parameters: HashMap::new(),
        };
        
        let effect = ObserveDomainFactEffect::new(query);
        
        // Execute the effect
        let outcome = effect.execute(&context).await.unwrap();
        
        // Verify the result
        assert!(outcome.is_success());
        
        // Check the data
        let value = outcome.get_data("value").unwrap();
        assert_eq!(value, "100");
    }
} 