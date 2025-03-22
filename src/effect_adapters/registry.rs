//! Effect Adapter Registry
//!
//! This module provides a registry for effect adapters, allowing the system to
//! dynamically register, discover, and manage adapters for different domains.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::types::DomainId;
use crate::error::{Error, Result};
use crate::effect_adapters::{EffectAdapter, DomainConfig};

/// Status of an adapter registration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterStatus {
    /// Adapter is registered but not initialized
    Registered,
    /// Adapter is initialized and ready
    Active,
    /// Adapter is temporarily unavailable
    Unavailable,
    /// Adapter has been disabled
    Disabled,
    /// Adapter has encountered an error
    Error,
}

/// Information about a registered adapter
#[derive(Debug)]
pub struct AdapterRegistration {
    /// Domain ID this adapter is for
    pub domain_id: DomainId,
    /// Adapter type/implementation name
    pub adapter_type: String,
    /// Current status of the adapter
    pub status: AdapterStatus,
    /// Version of the adapter
    pub version: String,
    /// Configuration for the adapter's domain
    pub config: DomainConfig,
    /// Last error message, if any
    pub last_error: Option<String>,
}

impl AdapterRegistration {
    /// Create a new adapter registration
    pub fn new(
        domain_id: DomainId,
        adapter_type: impl Into<String>,
        version: impl Into<String>,
        config: DomainConfig,
    ) -> Self {
        AdapterRegistration {
            domain_id,
            adapter_type: adapter_type.into(),
            status: AdapterStatus::Registered,
            version: version.into(),
            config,
            last_error: None,
        }
    }
    
    /// Set the adapter status
    pub fn set_status(&mut self, status: AdapterStatus) {
        self.status = status;
    }
    
    /// Set an error message
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.last_error = Some(error.into());
        self.status = AdapterStatus::Error;
    }
    
    /// Clear the error status
    pub fn clear_error(&mut self) {
        self.last_error = None;
        // Only update status if it was Error
        if self.status == AdapterStatus::Error {
            self.status = AdapterStatus::Registered;
        }
    }
    
    /// Check if the adapter is available
    pub fn is_available(&self) -> bool {
        matches!(self.status, AdapterStatus::Active)
    }
}

/// Registry for effect adapters
#[derive(Debug, Default)]
pub struct AdapterRegistry {
    /// Registered adapters
    registrations: RwLock<HashMap<DomainId, AdapterRegistration>>,
    /// Active adapter instances
    adapters: RwLock<HashMap<DomainId, Arc<dyn EffectAdapter + Send + Sync>>>,
    /// Default adapter to use when no domain is specified
    default_domain: RwLock<Option<DomainId>>,
}

impl AdapterRegistry {
    /// Create a new adapter registry
    pub fn new() -> Self {
        AdapterRegistry {
            registrations: RwLock::new(HashMap::new()),
            adapters: RwLock::new(HashMap::new()),
            default_domain: RwLock::new(None),
        }
    }
    
    /// Register an adapter with the registry
    pub fn register_adapter(
        &self,
        domain_id: DomainId,
        adapter_type: impl Into<String>,
        version: impl Into<String>,
        config: DomainConfig,
    ) -> Result<()> {
        let registration = AdapterRegistration::new(
            domain_id.clone(),
            adapter_type,
            version,
            config,
        );
        
        let mut registrations = self.registrations.write().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire write lock on registrations".to_string())
        })?;
        
        registrations.insert(domain_id, registration);
        Ok(())
    }
    
    /// Set the default domain
    pub fn set_default_domain(&self, domain_id: DomainId) -> Result<()> {
        // Check if the domain is registered
        let registrations = self.registrations.read().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire read lock on registrations".to_string())
        })?;
        
        if !registrations.contains_key(&domain_id) {
            return Err(Error::NotFoundError(format!(
                "Cannot set default domain to {}: domain not registered",
                domain_id
            )));
        }
        
        let mut default_domain = self.default_domain.write().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire write lock on default domain".to_string())
        })?;
        
        *default_domain = Some(domain_id);
        Ok(())
    }
    
    /// Get the default domain
    pub fn get_default_domain(&self) -> Result<Option<DomainId>> {
        let default_domain = self.default_domain.read().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire read lock on default domain".to_string())
        })?;
        
        Ok(default_domain.clone())
    }
    
    /// Get adapter registration by domain ID
    pub fn get_registration(&self, domain_id: &DomainId) -> Result<AdapterRegistration> {
        let registrations = self.registrations.read().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire read lock on registrations".to_string())
        })?;
        
        registrations.get(domain_id)
            .cloned()
            .ok_or_else(|| Error::NotFoundError(format!(
                "No adapter registered for domain {}",
                domain_id
            )))
    }
    
    /// Get all registered domains
    pub fn get_registered_domains(&self) -> Result<Vec<DomainId>> {
        let registrations = self.registrations.read().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire read lock on registrations".to_string())
        })?;
        
        Ok(registrations.keys().cloned().collect())
    }
    
    /// Add an adapter instance to the registry
    pub fn add_adapter_instance(
        &self,
        domain_id: DomainId,
        adapter: impl EffectAdapter + Send + Sync + 'static,
    ) -> Result<()> {
        // Verify that the domain is registered
        {
            let mut registrations = self.registrations.write().map_err(|_| {
                Error::ConcurrencyError("Failed to acquire write lock on registrations".to_string())
            })?;
            
            let registration = registrations.get_mut(&domain_id).ok_or_else(|| {
                Error::NotFoundError(format!(
                    "Cannot add adapter instance for domain {}: domain not registered",
                    domain_id
                ))
            })?;
            
            // Update the registration status
            registration.set_status(AdapterStatus::Active);
        }
        
        // Add the adapter instance
        let mut adapters = self.adapters.write().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire write lock on adapters".to_string())
        })?;
        
        adapters.insert(domain_id, Arc::new(adapter));
        Ok(())
    }
    
    /// Get an adapter instance by domain ID
    pub fn get_adapter(&self, domain_id: &DomainId) -> Result<Arc<dyn EffectAdapter + Send + Sync>> {
        let adapters = self.adapters.read().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire read lock on adapters".to_string())
        })?;
        
        adapters.get(domain_id)
            .cloned()
            .ok_or_else(|| Error::NotFoundError(format!(
                "No adapter instance available for domain {}",
                domain_id
            )))
    }
    
    /// Get the default adapter instance
    pub fn get_default_adapter(&self) -> Result<Arc<dyn EffectAdapter + Send + Sync>> {
        let default_domain = self.get_default_domain()?
            .ok_or_else(|| Error::NotFoundError("No default domain set".to_string()))?;
        
        self.get_adapter(&default_domain)
    }
    
    /// Update the status of an adapter
    pub fn update_adapter_status(&self, domain_id: &DomainId, status: AdapterStatus) -> Result<()> {
        let mut registrations = self.registrations.write().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire write lock on registrations".to_string())
        })?;
        
        let registration = registrations.get_mut(domain_id).ok_or_else(|| {
            Error::NotFoundError(format!(
                "Cannot update status for domain {}: domain not registered",
                domain_id
            ))
        })?;
        
        registration.set_status(status);
        Ok(())
    }
    
    /// Set an error for an adapter
    pub fn set_adapter_error(&self, domain_id: &DomainId, error: impl Into<String>) -> Result<()> {
        let mut registrations = self.registrations.write().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire write lock on registrations".to_string())
        })?;
        
        let registration = registrations.get_mut(domain_id).ok_or_else(|| {
            Error::NotFoundError(format!(
                "Cannot set error for domain {}: domain not registered",
                domain_id
            ))
        })?;
        
        registration.set_error(error);
        Ok(())
    }
    
    /// Remove an adapter instance
    pub fn remove_adapter(&self, domain_id: &DomainId) -> Result<()> {
        // Update the registration status
        {
            let mut registrations = self.registrations.write().map_err(|_| {
                Error::ConcurrencyError("Failed to acquire write lock on registrations".to_string())
            })?;
            
            if let Some(registration) = registrations.get_mut(domain_id) {
                registration.set_status(AdapterStatus::Registered);
            }
        }
        
        // Remove the adapter instance
        let mut adapters = self.adapters.write().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire write lock on adapters".to_string())
        })?;
        
        adapters.remove(domain_id);
        Ok(())
    }
    
    /// Unregister an adapter
    pub fn unregister_adapter(&self, domain_id: &DomainId) -> Result<()> {
        // Check if this is the default domain
        {
            let default_domain = self.default_domain.read().map_err(|_| {
                Error::ConcurrencyError("Failed to acquire read lock on default domain".to_string())
            })?;
            
            if default_domain.as_ref() == Some(domain_id) {
                let mut default_domain_write = self.default_domain.write().map_err(|_| {
                    Error::ConcurrencyError("Failed to acquire write lock on default domain".to_string())
                })?;
                
                *default_domain_write = None;
            }
        }
        
        // Remove the adapter instance
        let mut adapters = self.adapters.write().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire write lock on adapters".to_string())
        })?;
        
        adapters.remove(domain_id);
        
        // Remove the registration
        let mut registrations = self.registrations.write().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire write lock on registrations".to_string())
        })?;
        
        registrations.remove(domain_id);
        Ok(())
    }
}

/// Factory provider trait for creating adapter instances
pub trait AdapterFactory: Send + Sync {
    /// Create a new adapter instance
    fn create_adapter(&self, domain_id: &DomainId, config: &DomainConfig) -> Result<Box<dyn EffectAdapter + Send + Sync>>;
    
    /// Get the adapter type this factory creates
    fn get_adapter_type(&self) -> &str;
    
    /// Get the supported domain types
    fn get_supported_domain_types(&self) -> Vec<&str>;
    
    /// Check if this factory supports a given domain type
    fn supports_domain_type(&self, domain_type: &str) -> bool {
        self.get_supported_domain_types().contains(&domain_type)
    }
}

/// Registry of adapter factories
pub struct AdapterFactoryRegistry {
    /// Registered factories
    factories: RwLock<Vec<Arc<dyn AdapterFactory>>>,
}

impl AdapterFactoryRegistry {
    /// Create a new adapter factory registry
    pub fn new() -> Self {
        AdapterFactoryRegistry {
            factories: RwLock::new(Vec::new()),
        }
    }
    
    /// Register an adapter factory
    pub fn register_factory(&self, factory: impl AdapterFactory + 'static) -> Result<()> {
        let mut factories = self.factories.write().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire write lock on factories".to_string())
        })?;
        
        factories.push(Arc::new(factory));
        Ok(())
    }
    
    /// Find a factory for a given domain type
    pub fn find_factory_for_domain_type(&self, domain_type: &str) -> Result<Arc<dyn AdapterFactory>> {
        let factories = self.factories.read().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire read lock on factories".to_string())
        })?;
        
        for factory in factories.iter() {
            if factory.supports_domain_type(domain_type) {
                return Ok(factory.clone());
            }
        }
        
        Err(Error::NotFoundError(format!(
            "No adapter factory available for domain type {}",
            domain_type
        )))
    }
    
    /// Get all registered factories
    pub fn get_factories(&self) -> Result<Vec<Arc<dyn AdapterFactory>>> {
        let factories = self.factories.read().map_err(|_| {
            Error::ConcurrencyError("Failed to acquire read lock on factories".to_string())
        })?;
        
        Ok(factories.clone())
    }
}

impl Default for AdapterFactoryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::effect_adapters::{
        FactType, FactObservationMeta, TransactionReceipt, EffectParams,
        AdapterError, ProofError, ObservationError,
    };
    
    struct MockAdapter {
        domain_id: DomainId,
        config: DomainConfig,
    }
    
    impl MockAdapter {
        fn new(domain_id: DomainId, config: DomainConfig) -> Self {
            MockAdapter { domain_id, config }
        }
    }
    
    impl EffectAdapter for MockAdapter {
        fn apply_effect(&self, _params: EffectParams) -> std::result::Result<TransactionReceipt, AdapterError> {
            Ok(TransactionReceipt {
                domain_id: self.domain_id.clone(),
                transaction_id: "mock_tx_id".to_string(),
                content_id: None,
                timestamp: None,
                status: true,
                metadata: HashMap::new(),
            })
        }
        
        fn validate_proof(&self, _proof: &[u8], _fact_type: &str) -> std::result::Result<bool, ProofError> {
            Ok(true)
        }
        
        fn observe_fact(&self, _fact_type: &str, _params: &[(&str, &str)]) -> std::result::Result<(FactType, FactObservationMeta), ObservationError> {
            Ok((FactType::Custom("mock".to_string()), FactObservationMeta {
                observation_time: Timestamp::now(),
                domain: self.domain_id.clone(),
                block_height: 0,
                block_hash: None,
                confidence: 1.0,
                source: "mock".to_string(),
                metadata: HashMap::new(),
            }))
        }
        
        fn get_domain_id(&self) -> &DomainId {
            &self.domain_id
        }
        
        fn get_config(&self) -> &DomainConfig {
            &self.config
        }
        
        fn update_config(&mut self, config: DomainConfig) -> Result<()> {
            self.config = config;
            Ok(())
        }
    }
    
    struct MockAdapterFactory;
    
    impl AdapterFactory for MockAdapterFactory {
        fn create_adapter(&self, domain_id: &DomainId, config: &DomainConfig) -> Result<Box<dyn EffectAdapter + Send + Sync>> {
            Ok(Box::new(MockAdapter::new(domain_id.clone(), config.clone())))
        }
        
        fn get_adapter_type(&self) -> &str {
            "mock_adapter"
        }
        
        fn get_supported_domain_types(&self) -> Vec<&str> {
            vec!["test", "mock"]
        }
    }
    
    #[test]
    fn test_adapter_registration() {
        let registry = AdapterRegistry::new();
        let domain_id = DomainId::new("test_domain");
        let config = DomainConfig {
            rpc_endpoints: vec!["http://localhost:8545".to_string()],
            chain_id: Some("1".to_string()),
            network_id: Some("1".to_string()),
            timeout_ms: Some(5000),
            gas_limit: Some(100000),
            auth: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        // Register an adapter
        registry.register_adapter(domain_id.clone(), "test_adapter", "1.0.0", config.clone()).unwrap();
        
        // Get the registration
        let registration = registry.get_registration(&domain_id).unwrap();
        assert_eq!(registration.domain_id, domain_id);
        assert_eq!(registration.adapter_type, "test_adapter");
        assert_eq!(registration.version, "1.0.0");
        assert_eq!(registration.status, AdapterStatus::Registered);
        
        // Set as default domain
        registry.set_default_domain(domain_id.clone()).unwrap();
        let default_domain = registry.get_default_domain().unwrap();
        assert_eq!(default_domain.unwrap(), domain_id);
    }
    
    #[test]
    fn test_adapter_instance_management() {
        let registry = AdapterRegistry::new();
        let domain_id = DomainId::new("test_domain");
        let config = DomainConfig {
            rpc_endpoints: vec!["http://localhost:8545".to_string()],
            chain_id: Some("1".to_string()),
            network_id: Some("1".to_string()),
            timeout_ms: Some(5000),
            gas_limit: Some(100000),
            auth: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        // Register an adapter
        registry.register_adapter(domain_id.clone(), "test_adapter", "1.0.0", config.clone()).unwrap();
        
        // Add adapter instance
        let adapter = MockAdapter::new(domain_id.clone(), config);
        registry.add_adapter_instance(domain_id.clone(), adapter).unwrap();
        
        // Get adapter instance
        let adapter_instance = registry.get_adapter(&domain_id).unwrap();
        assert_eq!(adapter_instance.get_domain_id(), &domain_id);
        
        // Get registration after adding instance
        let registration = registry.get_registration(&domain_id).unwrap();
        assert_eq!(registration.status, AdapterStatus::Active);
        
        // Update adapter status
        registry.update_adapter_status(&domain_id, AdapterStatus::Unavailable).unwrap();
        let registration = registry.get_registration(&domain_id).unwrap();
        assert_eq!(registration.status, AdapterStatus::Unavailable);
        
        // Set an error
        registry.set_adapter_error(&domain_id, "Test error").unwrap();
        let registration = registry.get_registration(&domain_id).unwrap();
        assert_eq!(registration.status, AdapterStatus::Error);
        assert_eq!(registration.last_error.unwrap(), "Test error");
        
        // Remove adapter instance
        registry.remove_adapter(&domain_id).unwrap();
        assert!(registry.get_adapter(&domain_id).is_err());
        
        // Registration should still exist
        let registration = registry.get_registration(&domain_id).unwrap();
        assert_eq!(registration.status, AdapterStatus::Registered);
        
        // Unregister adapter
        registry.unregister_adapter(&domain_id).unwrap();
        assert!(registry.get_registration(&domain_id).is_err());
    }
    
    #[test]
    fn test_adapter_factory_registry() {
        let factory_registry = AdapterFactoryRegistry::new();
        
        // Register a factory
        factory_registry.register_factory(MockAdapterFactory).unwrap();
        
        // Find factory for supported domain type
        let factory = factory_registry.find_factory_for_domain_type("test").unwrap();
        assert_eq!(factory.get_adapter_type(), "mock_adapter");
        
        // Get all factories
        let factories = factory_registry.get_factories().unwrap();
        assert_eq!(factories.len(), 1);
        
        // Try to find factory for unsupported domain type
        let result = factory_registry.find_factory_for_domain_type("unsupported");
        assert!(result.is_err());
    }
} 