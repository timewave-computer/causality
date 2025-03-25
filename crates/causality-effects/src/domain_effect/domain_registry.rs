// Domain Adapter Registry for Effect System Integration
//
// This file implements a registry for domain adapters that integrates
// with the Effect System.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use tokio::sync::Mutex;

use causality_domain::{
    adapter::{DomainAdapter, DomainAdapterFactory, DomainAdapterRegistry},
    domain::{DomainId, DomainInfo, DomainType, Transaction, TransactionId, TransactionReceipt},
    fact::{FactQuery, Fact, FactResult},
    types::{Result as DomainResult, Error as DomainError},
};

use crate::domain_effect::{
    DomainAdapterEffect, DomainContext, DomainQueryEffect,
    DomainTransactionEffect, DomainTimeMapEffect, DomainCapabilityEffect,
    domain_selection::{DomainSelectionEffect, DomainSelectionHandler, SelectionCriteria},
    evm_effects::{EvmContractCallEffect, EvmStateQueryEffect, EvmGasEstimationEffect},
    cosmwasm_effects::{CosmWasmExecuteEffect, CosmWasmQueryEffect, CosmWasmInstantiateEffect, CosmWasmCodeUploadEffect},
    zk_effects::{ZkProveEffect, ZkVerifyEffect, ZkWitnessEffect, ZkProofCompositionEffect}
};
use crate::effect::{Effect, EffectContext, EffectResult, EffectError, EffectOutcome};

/// Domain registry that integrates with the Effect System
///
/// This registry implements both the DomainAdapterRegistry trait from the
/// Domain module and connects to the Effect System to enable bidirectional
/// integration.
#[derive(Debug)]
pub struct EffectDomainRegistry {
    /// Domain adapter factories
    factories: RwLock<Vec<Arc<dyn DomainAdapterFactory>>>,
    
    /// Active domain adapters
    adapters: RwLock<HashMap<DomainId, Arc<dyn DomainAdapter>>>,
}

impl EffectDomainRegistry {
    /// Create a new domain registry
    pub fn new() -> Self {
        Self {
            factories: RwLock::new(Vec::new()),
            adapters: RwLock::new(HashMap::new()),
        }
    }
    
    /// Execute a domain query effect
    pub async fn execute_query(&self, effect: &DomainQueryEffect, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Get the domain ID from the effect
        let domain_id = effect.domain_id();
        
        // Get the domain adapter
        let adapter = self.get_adapter(domain_id).await
            .map_err(|e| EffectError::NotFound(format!("Domain adapter not found: {}", e)))?;
            
        // Create domain context
        let domain_context = effect.create_domain_context(context);
        
        // Execute the query
        let fact_result = adapter.observe_fact(&effect.query()).await
            .map_err(|e| EffectError::ExecutionError(format!("Domain query failed: {}", e)))?;
            
        // Map the result to an effect outcome
        effect.map_outcome(Ok(fact_result))
    }
    
    /// Execute a domain transaction effect
    pub async fn execute_transaction(&self, effect: &DomainTransactionEffect, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Get the domain ID from the effect
        let domain_id = effect.domain_id();
        
        // Get the domain adapter
        let adapter = self.get_adapter(domain_id).await
            .map_err(|e| EffectError::NotFound(format!("Domain adapter not found: {}", e)))?;
            
        // Create domain context
        let domain_context = effect.create_domain_context(context);
        
        // Submit the transaction
        let tx_id = adapter.submit_transaction(effect.transaction()).await
            .map_err(|e| EffectError::ExecutionError(format!("Transaction submission failed: {}", e)))?;
            
        // Map the result to an effect outcome
        effect.map_outcome(Ok(tx_id))
    }
    
    /// Execute a domain time map effect
    pub async fn execute_time_map(&self, effect: &DomainTimeMapEffect, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Get the domain ID from the effect
        let domain_id = effect.domain_id();
        
        // Get the domain adapter
        let adapter = self.get_adapter(domain_id).await
            .map_err(|e| EffectError::NotFound(format!("Domain adapter not found: {}", e)))?;
            
        // Create domain context
        let domain_context = effect.create_domain_context(context);
        
        // Get current height
        let height = adapter.current_height().await
            .map_err(|e| EffectError::ExecutionError(format!("Failed to get domain height: {}", e)))?;
            
        // Map the result to an effect outcome
        let outcome = EffectOutcome::success(effect.id().clone())
            .with_data("height", height.to_string());
            
        Ok(outcome)
    }
    
    /// Execute a domain capability effect
    pub async fn execute_capability(&self, effect: &DomainCapabilityEffect, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Get the domain ID from the effect
        let domain_id = effect.domain_id();
        
        // Get the domain adapter
        let adapter = self.get_adapter(domain_id).await
            .map_err(|e| EffectError::NotFound(format!("Domain adapter not found: {}", e)))?;
            
        // Create domain context
        let domain_context = effect.create_domain_context(context);
        
        // Check capability
        let has_capability = adapter.has_capability(effect.capability()).await
            .map_err(|e| EffectError::ExecutionError(format!("Capability check failed: {}", e)))?;
            
        // Map the result to an effect outcome
        let outcome = EffectOutcome::success(effect.id().clone())
            .with_data("has_capability", has_capability.to_string());
            
        Ok(outcome)
    }
    
    /// Execute a domain selection effect
    pub async fn execute_selection(&self, effect: &DomainSelectionEffect, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Use the DomainSelectionHandler trait implementation
        self.execute_domain_selection(effect, context).await
    }
    
    /// Get all adapter instances
    pub async fn get_all_adapters(&self) -> Vec<Arc<dyn DomainAdapter>> {
        // Try to initialize any missing adapters for supported domain types
        self.initialize_missing_adapters().await;
        
        let adapters = self.adapters.read().unwrap();
        adapters.values().cloned().collect()
    }
    
    /// Initialize adapters for all supported domain types
    async fn initialize_missing_adapters(&self) {
        // Get supported domain types from all factories
        let factories = self.factories.read().unwrap();
        let mut domain_types = Vec::new();
        
        for factory in factories.iter() {
            domain_types.extend(factory.supported_domain_types());
        }
        
        // Deduplicate
        domain_types.sort();
        domain_types.dedup();
        
        // Try to create an adapter for each domain type
        // This is a simplified approach; in a real implementation 
        // we would need more sophisticated domain ID generation
        for domain_type in domain_types {
            let domain_id = format!("{}:default", domain_type);
            
            // Check if we already have this adapter
            {
                let adapters = self.adapters.read().unwrap();
                if adapters.contains_key(&domain_id) {
                    continue;
                }
            }
            
            // Try to create the adapter
            if let Ok(adapter) = self.get_adapter(&domain_id).await {
                // Adapter was successfully created and registered
                continue;
            }
        }
    }
    
    /// Get adapter for domain ID
    pub async fn get_adapter(&self, domain_id: &DomainId) -> DomainResult<Arc<dyn DomainAdapter>> {
        // Check if we already have an active adapter
        {
            let adapters = self.adapters.read().unwrap();
            if let Some(adapter) = adapters.get(domain_id) {
                return Ok(adapter.clone());
            }
        }
        
        // No active adapter, try to create one
        let factories = self.factories.read().unwrap();
        
        // Find a factory that supports this domain
        for factory in factories.iter() {
            if let Ok(adapter) = factory.create_adapter(domain_id.clone()).await {
                // Register the new adapter
                let mut adapters = self.adapters.write().unwrap();
                adapters.insert(domain_id.clone(), adapter.clone());
                
                return Ok(adapter);
            }
        }
        
        Err(DomainError::UnsupportedDomain(format!("No adapter available for domain: {}", domain_id)))
    }
}

impl Default for EffectDomainRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainAdapterRegistry for EffectDomainRegistry {
    fn register_factory(&self, factory: Arc<dyn DomainAdapterFactory>) {
        let mut factories = self.factories.write().unwrap();
        factories.push(factory);
    }
    
    fn get_supported_domains(&self) -> Vec<DomainType> {
        let factories = self.factories.read().unwrap();
        let mut domain_types = Vec::new();
        
        for factory in factories.iter() {
            domain_types.extend(factory.supported_domain_types());
        }
        
        // Deduplicate
        domain_types.sort();
        domain_types.dedup();
        
        domain_types
    }
}

/// Extension trait for domain effect handling
#[async_trait]
pub trait DomainEffectHandler {
    /// Execute a domain adapter effect
    async fn execute_domain_effect(&self, effect: &dyn DomainAdapterEffect, context: &EffectContext) -> EffectResult<EffectOutcome>;
    
    /// Check if this handler can handle the given effect
    fn can_handle_effect(&self, effect: &dyn Effect) -> bool;
}

#[async_trait]
impl DomainEffectHandler for EffectDomainRegistry {
    async fn execute_domain_effect(&self, effect: &dyn DomainAdapterEffect, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Cast to specific effect types and dispatch appropriately
        if let Some(query_effect) = effect.as_any().downcast_ref::<DomainQueryEffect>() {
            self.execute_query(query_effect, context).await
        } else if let Some(tx_effect) = effect.as_any().downcast_ref::<DomainTransactionEffect>() {
            self.execute_transaction(tx_effect, context).await
        } else if let Some(time_effect) = effect.as_any().downcast_ref::<DomainTimeMapEffect>() {
            self.execute_time_map(time_effect, context).await
        } else if let Some(capability_effect) = effect.as_any().downcast_ref::<DomainCapabilityEffect>() {
            self.execute_capability(capability_effect, context).await
        } else {
            Err(EffectError::UnsupportedOperation(format!(
                "Unsupported domain effect type: {:?}", effect
            )))
        }
    }
    
    fn can_handle_effect(&self, effect: &dyn Effect) -> bool {
        // Check if this is a DomainAdapterEffect, DomainSelectionEffect, or domain-specific effect
        effect.as_any().downcast_ref::<DomainQueryEffect>().is_some() ||
        effect.as_any().downcast_ref::<DomainTransactionEffect>().is_some() ||
        effect.as_any().downcast_ref::<DomainTimeMapEffect>().is_some() ||
        effect.as_any().downcast_ref::<DomainCapabilityEffect>().is_some() ||
        effect.as_any().downcast_ref::<DomainSelectionEffect>().is_some() ||
        // EVM-specific effects
        effect.as_any().downcast_ref::<EvmContractCallEffect>().is_some() ||
        effect.as_any().downcast_ref::<EvmStateQueryEffect>().is_some() ||
        effect.as_any().downcast_ref::<EvmGasEstimationEffect>().is_some() ||
        // CosmWasm-specific effects
        effect.as_any().downcast_ref::<CosmWasmExecuteEffect>().is_some() ||
        effect.as_any().downcast_ref::<CosmWasmQueryEffect>().is_some() ||
        effect.as_any().downcast_ref::<CosmWasmInstantiateEffect>().is_some() ||
        effect.as_any().downcast_ref::<CosmWasmCodeUploadEffect>().is_some() ||
        // ZK/Succinct-specific effects
        effect.as_any().downcast_ref::<ZkProveEffect>().is_some() ||
        effect.as_any().downcast_ref::<ZkVerifyEffect>().is_some() ||
        effect.as_any().downcast_ref::<ZkWitnessEffect>().is_some() ||
        effect.as_any().downcast_ref::<ZkProofCompositionEffect>().is_some()
    }
}

/// Factory for creating effect-integrated domain adapters
pub struct EffectDomainAdapterFactory<F>
where
    F: DomainAdapterFactory + Send + Sync + 'static,
{
    inner_factory: Arc<F>,
}

impl<F> EffectDomainAdapterFactory<F>
where
    F: DomainAdapterFactory + Send + Sync + 'static,
{
    /// Create a new effect-integrated domain adapter factory
    pub fn new(inner_factory: F) -> Self {
        Self {
            inner_factory: Arc::new(inner_factory),
        }
    }
}

#[async_trait]
impl<F> DomainAdapterFactory for EffectDomainAdapterFactory<F>
where
    F: DomainAdapterFactory + Send + Sync + 'static,
{
    async fn create_adapter(&self, domain_id: DomainId) -> DomainResult<Arc<dyn DomainAdapter>> {
        self.inner_factory.create_adapter(domain_id).await
    }
    
    fn supported_domain_types(&self) -> Vec<DomainType> {
        self.inner_factory.supported_domain_types()
    }
}

// Utility functions

/// Create a domain registry
pub fn create_domain_registry() -> EffectDomainRegistry {
    EffectDomainRegistry::new()
} 