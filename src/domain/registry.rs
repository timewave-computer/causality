// Domain Registry
//
// This module provides a registry for managing domain adapters.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;

use crate::types::DomainId;
use crate::domain::adapter::{DomainAdapter, DomainAdapterFactory};
use crate::domain::types::{DomainInfo, DomainType, DomainStatus};
use crate::domain::selection::DomainSelectionStrategy;
use crate::error::{Error, Result};

/// Domain registry that manages domain adapters
pub struct DomainRegistry {
    /// Registered domain adapters
    adapters: RwLock<HashMap<DomainId, Arc<dyn DomainAdapter>>>,
    /// Default domain selection strategy
    default_strategy: RwLock<Box<dyn DomainSelectionStrategy>>,
    /// Registered domain adapter factories
    factories: RwLock<Vec<Arc<dyn DomainAdapterFactory>>>,
}

impl DomainRegistry {
    /// Create a new domain registry
    pub fn new(default_strategy: Box<dyn DomainSelectionStrategy>) -> Self {
        Self {
            adapters: RwLock::new(HashMap::new()),
            default_strategy: RwLock::new(default_strategy),
            factories: RwLock::new(Vec::new()),
        }
    }
    
    /// Register a domain adapter
    pub fn register_adapter(&self, adapter: Arc<dyn DomainAdapter>) -> Result<()> {
        let domain_id = adapter.domain_id().clone();
        let mut adapters = self.adapters.write().map_err(|_| Error::SystemError("Failed to acquire write lock on domain registry".to_string()))?;
        
        if adapters.contains_key(&domain_id) {
            return Err(Error::DomainAlreadyRegistered(domain_id.to_string()));
        }
        
        adapters.insert(domain_id, adapter);
        Ok(())
    }
    
    /// Unregister a domain adapter
    pub fn unregister_adapter(&self, domain_id: &DomainId) -> Result<()> {
        let mut adapters = self.adapters.write().map_err(|_| Error::SystemError("Failed to acquire write lock on domain registry".to_string()))?;
        
        if !adapters.contains_key(domain_id) {
            return Err(Error::DomainNotFound(domain_id.clone()));
        }
        
        adapters.remove(domain_id);
        Ok(())
    }
    
    /// Get a domain adapter by ID
    pub fn get_adapter(&self, domain_id: &DomainId) -> Result<Arc<dyn DomainAdapter>> {
        let adapters = self.adapters.read().map_err(|_| Error::SystemError("Failed to acquire read lock on domain registry".to_string()))?;
        
        adapters.get(domain_id)
            .cloned()
            .ok_or_else(|| Error::DomainNotFound(domain_id.clone()))
    }
    
    /// Get all registered domain adapters
    pub fn get_all_adapters(&self) -> Result<Vec<Arc<dyn DomainAdapter>>> {
        let adapters = self.adapters.read().map_err(|_| Error::SystemError("Failed to acquire read lock on domain registry".to_string()))?;
        
        Ok(adapters.values().cloned().collect())
    }
    
    /// Get all domain infos
    pub async fn get_all_domain_infos(&self) -> Result<Vec<DomainInfo>> {
        let adapters = self.adapters.read().map_err(|_| Error::SystemError("Failed to acquire read lock on domain registry".to_string()))?;
        
        let mut infos = Vec::with_capacity(adapters.len());
        for adapter in adapters.values() {
            match adapter.domain_info().await {
                Ok(info) => infos.push(info),
                Err(e) => {
                    // If we can't get info, create a basic one with error status
                    infos.push(DomainInfo {
                        id: adapter.domain_id().clone(),
                        domain_type: DomainType::Unknown,
                        name: format!("Unknown ({})", adapter.domain_id()),
                        description: Some(format!("Error fetching domain info: {}", e)),
                        rpc_url: None,
                        explorer_url: None,
                        chain_id: None,
                        native_currency: None,
                        status: DomainStatus::Error,
                        metadata: HashMap::new(),
                    });
                }
            }
        }
        
        Ok(infos)
    }
    
    /// Register a domain adapter factory
    pub fn register_factory(&self, factory: Arc<dyn DomainAdapterFactory>) -> Result<()> {
        let mut factories = self.factories.write().map_err(|_| Error::SystemError("Failed to acquire write lock on domain factories".to_string()))?;
        factories.push(factory);
        Ok(())
    }
    
    /// Get all registered factory types
    pub fn get_supported_domain_types(&self) -> Result<Vec<String>> {
        let factories = self.factories.read().map_err(|_| Error::SystemError("Failed to acquire read lock on domain factories".to_string()))?;
        
        let mut types = Vec::new();
        for factory in factories.iter() {
            types.extend(factory.supported_domain_types());
        }
        
        // Remove duplicates
        types.sort();
        types.dedup();
        
        Ok(types)
    }
    
    /// Create a domain adapter from a factory using the provided configuration
    pub async fn create_adapter_from_factory(&self, config: HashMap<String, String>) -> Result<Arc<dyn DomainAdapter>> {
        let factories = self.factories.read().map_err(|_| Error::SystemError("Failed to acquire read lock on domain factories".to_string()))?;
        
        // Get the domain type from config
        let domain_type = config.get("domain_type")
            .ok_or_else(|| Error::InvalidArgument("Missing domain_type parameter".into()))?;
            
        // Find a factory that supports this domain type
        for factory in factories.iter() {
            if factory.supports_domain_type(domain_type) {
                // Create the adapter
                let adapter = factory.create_adapter(config.clone()).await?;
                
                // Register it
                let adapter_arc = Arc::from(adapter);
                self.register_adapter(adapter_arc.clone())?;
                
                return Ok(adapter_arc);
            }
        }
        
        Err(Error::UnsupportedOperation(format!("No factory found for domain type: {}", domain_type)))
    }
    
    /// Set the default domain selection strategy
    pub fn set_default_strategy(&self, strategy: Box<dyn DomainSelectionStrategy>) -> Result<()> {
        let mut default_strategy = self.default_strategy.write().map_err(|_| Error::SystemError("Failed to acquire write lock on default strategy".to_string()))?;
        *default_strategy = strategy;
        Ok(())
    }
    
    /// Select a domain using the default strategy
    pub async fn select_domain(
        &self,
        required_capabilities: &std::collections::HashSet<String>,
        preferences: &HashMap<String, String>,
    ) -> Result<DomainId> {
        let adapters = self.get_all_adapters()?;
        let strategy = self.default_strategy.read().map_err(|_| Error::SystemError("Failed to acquire read lock on default strategy".to_string()))?;
        
        strategy.select_domain(&adapters, required_capabilities, preferences).await
    }
    
    /// Select multiple domains using the default strategy
    pub async fn select_domains(
        &self,
        required_capabilities: &std::collections::HashSet<String>,
        preferences: &HashMap<String, String>,
        count: usize,
    ) -> Result<Vec<DomainId>> {
        let adapters = self.get_all_adapters()?;
        let strategy = self.default_strategy.read().map_err(|_| Error::SystemError("Failed to acquire read lock on default strategy".to_string()))?;
        
        strategy.select_domains(&adapters, required_capabilities, preferences, count).await
    }
}

impl Default for DomainRegistry {
    fn default() -> Self {
        // Create a default registry with a simple strategy that just returns the first domain
        use crate::domain::selection::PreferredDomainStrategy;
        Self::new(Box::new(PreferredDomainStrategy::new(vec![])))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use crate::types::{BlockHeight, BlockHash, Timestamp};
    use crate::domain::types::{FactQuery, TimeMapEntry};
    use crate::fact::{Fact, FactId, FactResult};
    
    // Mock domain adapter for testing
    #[derive(Debug)]
    struct MockDomainAdapter {
        domain_id: DomainId,
        domain_type: DomainType,
        name: String,
    }
    
    #[async_trait]
    impl DomainAdapter for MockDomainAdapter {
        fn domain_id(&self) -> &DomainId {
            &self.domain_id
        }
        
        async fn domain_info(&self) -> Result<DomainInfo> {
            Ok(DomainInfo {
                id: self.domain_id.clone(),
                domain_type: self.domain_type.clone(),
                name: self.name.clone(),
                description: None,
                rpc_url: None,
                explorer_url: None,
                chain_id: None,
                native_currency: None,
                status: DomainStatus::Active,
                metadata: HashMap::new(),
            })
        }
        
        async fn current_height(&self) -> Result<BlockHeight> {
            Ok(BlockHeight::new(100))
        }
        
        async fn current_hash(&self) -> Result<BlockHash> {
            Ok(BlockHash::new(vec![1, 2, 3, 4]))
        }
        
        async fn current_time(&self) -> Result<Timestamp> {
            Ok(Timestamp::new(1234567890))
        }
        
        async fn time_map_entry(&self, _height: BlockHeight) -> Result<TimeMapEntry> {
            Ok(TimeMapEntry {
                height: BlockHeight::new(100),
                hash: BlockHash::new(vec![1, 2, 3, 4]),
                timestamp: Timestamp::new(1234567890),
            })
        }
        
        async fn observe_fact(&self, _query: &FactQuery) -> FactResult {
            Err(Error::NotImplemented("observe_fact not implemented for mock".to_string()))
        }
        
        async fn submit_transaction(&self, _tx: crate::domain::types::Transaction) -> Result<crate::domain::types::TransactionId> {
            Err(Error::NotImplemented("submit_transaction not implemented for mock".to_string()))
        }
        
        async fn transaction_receipt(&self, _tx_id: &crate::domain::types::TransactionId) -> Result<crate::domain::types::TransactionReceipt> {
            Err(Error::NotImplemented("transaction_receipt not implemented for mock".to_string()))
        }
        
        async fn transaction_confirmed(&self, _tx_id: &crate::domain::types::TransactionId) -> Result<bool> {
            Ok(true)
        }
        
        async fn wait_for_confirmation(&self, _tx_id: &crate::domain::types::TransactionId, _max_wait_ms: Option<u64>) -> Result<crate::domain::types::TransactionReceipt> {
            Err(Error::NotImplemented("wait_for_confirmation not implemented for mock".to_string()))
        }
    }
    
    // Mock domain adapter factory for testing
    #[derive(Debug)]
    struct MockAdapterFactory {
        supported_types: Vec<String>,
    }
    
    #[async_trait]
    impl DomainAdapterFactory for MockAdapterFactory {
        async fn create_adapter(&self, config: HashMap<String, String>) -> Result<Box<dyn DomainAdapter>> {
            let domain_id = config.get("domain_id")
                .ok_or_else(|| Error::InvalidArgument("Missing domain_id".into()))?;
                
            let domain_type = config.get("domain_type")
                .ok_or_else(|| Error::InvalidArgument("Missing domain_type".into()))?;
                
            let parsed_type = match domain_type.as_str() {
                "evm" => DomainType::EVM,
                "cosmwasm" => DomainType::CosmWasm,
                _ => DomainType::Unknown,
            };
            
            let name = config.get("name").cloned().unwrap_or_else(|| "Test Domain".to_string());
            
            Ok(Box::new(MockDomainAdapter {
                domain_id: DomainId::new(domain_id),
                domain_type: parsed_type,
                name,
            }))
        }
        
        fn supported_domain_types(&self) -> Vec<String> {
            self.supported_types.clone()
        }
    }
    
    #[tokio::test]
    async fn test_domain_registry() {
        // Create a registry
        let registry = DomainRegistry::default();
        
        // Create and register adapters
        let adapter1 = Arc::new(MockDomainAdapter {
            domain_id: DomainId::new("domain1"),
            domain_type: DomainType::EVM,
            name: "Domain 1".to_string(),
        });
        
        let adapter2 = Arc::new(MockDomainAdapter {
            domain_id: DomainId::new("domain2"),
            domain_type: DomainType::CosmWasm,
            name: "Domain 2".to_string(),
        });
        
        // Register the adapters
        registry.register_adapter(adapter1.clone()).unwrap();
        registry.register_adapter(adapter2.clone()).unwrap();
        
        // Get an adapter by ID
        let retrieved_adapter = registry.get_adapter(&DomainId::new("domain1")).unwrap();
        assert_eq!(retrieved_adapter.domain_id(), &DomainId::new("domain1"));
        
        // Get all adapters
        let all_adapters = registry.get_all_adapters().unwrap();
        assert_eq!(all_adapters.len(), 2);
        
        // Get all domain infos
        let all_infos = registry.get_all_domain_infos().await.unwrap();
        assert_eq!(all_infos.len(), 2);
        
        // Check info for domain1
        let domain1_info = all_infos.iter().find(|info| info.id == DomainId::new("domain1")).unwrap();
        assert_eq!(domain1_info.name, "Domain 1");
        assert_eq!(domain1_info.domain_type, DomainType::EVM);
        
        // Unregister an adapter
        registry.unregister_adapter(&DomainId::new("domain1")).unwrap();
        
        // Verify it's gone
        let all_adapters_after = registry.get_all_adapters().unwrap();
        assert_eq!(all_adapters_after.len(), 1);
        assert_eq!(all_adapters_after[0].domain_id(), &DomainId::new("domain2"));
    }
    
    #[tokio::test]
    async fn test_domain_factories() {
        // Create a registry
        let registry = DomainRegistry::default();
        
        // Create and register a factory
        let factory = Arc::new(MockAdapterFactory {
            supported_types: vec!["evm".to_string(), "cosmwasm".to_string()],
        });
        registry.register_factory(factory).unwrap();
        
        // Get supported domain types
        let types = registry.get_supported_domain_types().unwrap();
        assert_eq!(types.len(), 2);
        assert!(types.contains(&"evm".to_string()));
        assert!(types.contains(&"cosmwasm".to_string()));
        
        // Create an adapter from factory
        let mut config = HashMap::new();
        config.insert("domain_id".to_string(), "test_domain".to_string());
        config.insert("domain_type".to_string(), "evm".to_string());
        config.insert("name".to_string(), "Test EVM Domain".to_string());
        
        let adapter = registry.create_adapter_from_factory(config).await.unwrap();
        assert_eq!(adapter.domain_id(), &DomainId::new("test_domain"));
        
        // Verify the adapter was registered
        let all_adapters = registry.get_all_adapters().unwrap();
        assert_eq!(all_adapters.len(), 1);
        assert_eq!(all_adapters[0].domain_id(), &DomainId::new("test_domain"));
    }
} 