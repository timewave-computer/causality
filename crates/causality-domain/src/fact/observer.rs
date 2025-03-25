// Fact observer implementation for domains
// Original file: src/domain/fact/observer.rs

// Fact Observer Module for Causality
//
// This module provides the functionality for observing facts from domains.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::domain::DomainId;
use causality_domain::FactQuery;
use causality_domain::DomainRegistry;
use causality_types::{Error, Result};
use causality_engine_types::FactType;

/// Observer for domain facts
pub struct FactObserver {
    /// Domain registry
    registry: Arc<DomainRegistry>,
    /// Cache TTL in seconds
    cache_ttl: u64,
    /// Fact cache
    fact_cache: Arc<Mutex<HashMap<String, (FactType, SystemTime)>>>,
}

impl FactObserver {
    /// Create a new fact observer
    pub fn new(registry: Arc<DomainRegistry>, cache_ttl_seconds: u64) -> Self {
        Self {
            registry,
            cache_ttl: cache_ttl_seconds,
            fact_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Observe a fact
    pub async fn observe_fact(
        &self,
        domain_id: &DomainId,
        query: FactQuery,
        bypass_cache: bool,
    ) -> Result<FactType> {
        // Generate a cache key
        let cache_key = format!("{}:{}:{}",
            domain_id,
            query.fact_type,
            serde_json::to_string(&query.parameters).unwrap_or_default()
        );
        
        // Check cache if not bypassing
        if !bypass_cache {
            if let Some(fact) = self.get_from_cache(&cache_key) {
                return Ok(fact);
            }
        }
        
        // Get the adapter
        let adapter = self.registry.get_adapter(domain_id)
            .ok_or_else(|| Error::DomainError(format!("No adapter found for domain: {}", domain_id)))?;
        
        // Observe the fact
        let fact = adapter.observe_fact(query).await?;
        
        // Cache the fact
        self.cache_fact(&cache_key, fact.clone());
        
        Ok(fact)
    }
    
    /// Get a fact from the cache
    fn get_from_cache(&self, cache_key: &str) -> Option<FactType> {
        let cache = self.fact_cache.lock().unwrap();
        if let Some((fact, timestamp)) = cache.get(cache_key) {
            let elapsed = timestamp.elapsed().unwrap_or(Duration::from_secs(0));
            if elapsed.as_secs() < self.cache_ttl {
                return Some(fact.clone());
            }
        }
        None
    }
    
    /// Cache a fact
    fn cache_fact(&self, cache_key: &str, fact: FactType) {
        let mut cache = self.fact_cache.lock().unwrap();
        cache.insert(cache_key.to_string(), (fact, SystemTime::now()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_domain::DomainAdapter;
    use async_trait::async_trait;
    
    #[derive(Debug)]
    struct MockAdapter {
        domain_id: DomainId,
    }
    
    #[async_trait]
    impl DomainAdapter for MockAdapter {
        async fn observe_fact(&self, _query: FactQuery) -> Result<FactType> {
            Ok(FactType::BlockFact)
        }
        
        // Other required methods would be implemented here
    }
    
    #[tokio::test]
    async fn test_fact_observer() -> Result<()> {
        let registry = Arc::new(DomainRegistry::new());
        let observer = FactObserver::new(registry.clone(), 60);
        
        // Test without an adapter should fail
        let domain_id = DomainId::new("test_domain");
        let query = FactQuery::new("test_fact").with_parameter("key", "value");
        let result = observer.observe_fact(&domain_id, query, false).await;
        assert!(result.is_err());
        
        Ok(())
    }
}
