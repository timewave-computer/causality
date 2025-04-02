// Fact observer implementation for domains
// Original file: src/domain/fact/observer.rs

// Fact Observer Module for Causality
//
// This module implements a fact observer that can observe facts
// across different domains.

use std::collections::HashMap;
use std::sync::Arc;

use crate::adapter::{
    DomainAdapter,
    DomainAdapterRegistry,
    FactQuery as AdapterFactQuery,
    TimeMapEntry,
};
use crate::error::Error;
use crate::fact::types::{FactType, FactQuery};
use crate::fact::verification::FactVerifier;
use crate::fact::zkproof::ZKFactVerifier;
use crate::selection::DomainId;

/// Metadata from fact observation
#[derive(Debug, Clone)]
pub struct FactObservationMeta {
    /// Source domain
    pub domain_id: DomainId,
    /// Time map entry if available
    pub time_map_entry: Option<TimeMapEntry>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Observer for facts in a domain
#[derive(Debug)]
pub struct FactObserver {
    /// Domain adapter registry
    adapter_registry: Arc<DomainAdapterRegistry>,
    /// Fact verifier for verification
    verifier: Arc<dyn FactVerifier>,
}

impl FactObserver {
    /// Create a new fact observer
    pub fn new(
        adapter_registry: Arc<DomainAdapterRegistry>,
        verifier: Option<Arc<dyn FactVerifier>>,
    ) -> Self {
        let verifier = verifier.unwrap_or_else(|| Arc::new(ZKFactVerifier::new()));
        Self {
            adapter_registry,
            verifier,
        }
    }
    
    /// Register a domain adapter
    pub fn register_adapter(&self, adapter: Arc<dyn DomainAdapter>) {
        self.adapter_registry.register_adapter(adapter);
    }
    
    /// Remove a domain adapter
    pub fn remove_adapter(&self, domain_id: &DomainId) -> bool {
        self.adapter_registry.remove_adapter(domain_id)
    }
    
    /// Get a domain adapter
    pub fn get_adapter(&self, domain_id: &DomainId) -> Option<Arc<dyn DomainAdapter>> {
        self.adapter_registry.get_adapter(domain_id)
    }
    
    /// Observe a fact in a domain
    pub async fn observe_fact(
        &self,
        domain_id: &DomainId,
        query: &FactQuery,
    ) -> Result<(FactType, FactObservationMeta), Error> {
        // Get domain adapter
        let adapter = self.adapter_registry
            .get_adapter(domain_id)
            .ok_or_else(|| Error::DomainNotFound(domain_id.to_string()))?;
        
        // Convert the FactQuery to AdapterFactQuery
        let adapter_query = AdapterFactQuery::new(query.fact_type.clone())
            .with_verification(query.requires_verification);
        
        // Add all parameters
        let adapter_query = query.parameters.iter().fold(
            adapter_query,
            |query, (k, v)| query.with_parameter(k.clone(), v.clone())
        );
        
        // Add all metadata
        let adapter_query = query.metadata.iter().fold(
            adapter_query,
            |query, (k, v)| query.with_metadata(k.clone(), v.clone())
        );
            
        // Observe fact
        let (fact, obs_meta) = adapter.observe_fact(&adapter_query)
            .map_err(|e| Error::DomainAdapter(e.to_string()))?;
        
        // Create observation metadata
        let observation_meta = FactObservationMeta {
            domain_id: domain_id.clone(),
            time_map_entry: None, // TODO: Get time map entry
            metadata: obs_meta.metadata,
        };
        
        // Verify fact if required
        if query.requires_verification {
            let verification = self.verifier.verify(&fact)
                .map_err(|e| Error::FactVerification(e.to_string()))?;
            
            if !verification.is_valid() {
                return Err(Error::InvalidFact("Fact verification failed".to_string()));
            }
        }
        
        Ok((fact, observation_meta))
    }
    
    /// Check if a domain is available
    pub fn has_domain(&self, domain_id: &DomainId) -> bool {
        self.adapter_registry.get_adapter(domain_id).is_some()
    }
    
    /// List all domains
    pub fn list_domains(&self) -> Vec<DomainId> {
        self.adapter_registry.list_domains()
    }
    
    /// Get all adapters
    pub fn get_all_adapters(&self) -> Vec<Arc<dyn DomainAdapter>> {
        self.adapter_registry.get_all_adapters()
    }
}

impl Default for FactObserver {
    fn default() -> Self {
        // Create a new adapter registry
        let adapter_registry = Arc::new(DomainAdapterRegistry::new());
        // Create with default verifier
        Self::new(adapter_registry, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact::types::*;
    
    #[test]
    fn test_fact_observer_basics() {
        // This is a placeholder test
        let query = FactQuery {
            fact_type: "test".to_string(),
            parameters: HashMap::new(),
            requires_verification: false,
            metadata: HashMap::new(),
        };
        
        assert_eq!(query.fact_type, "test");
    }
}
