// causality-domain/src/indexer.rs
//
// This module provides integration between domain adapters and the indexer system

use causality_indexer_adapter::{ChainId, IndexedFact, IndexerAdapter, QueryOptions};
use crate::adapter::DomainAdapter;
use crate::error::Result;
use crate::selection::DomainId;
use causality_types::ResourceId;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// Type alias for the common boxed error type used with IndexerAdapter
pub type IndexerError = Box<dyn std::error::Error + Send + Sync>;

/// Extension trait for domain adapters that can access historical data through an indexer
#[async_trait]
pub trait IndexerExtension {
    /// Get historical facts for a resource
    async fn get_historical_facts(
        &self,
        resource_id: &ResourceId,
        from_time: Option<DateTime<Utc>>,
        to_time: Option<DateTime<Utc>>,
        limit: Option<u32>,
    ) -> Result<Vec<IndexedFact>>;
    
    /// Get the indexer adapter, if available
    fn get_indexer(&self) -> Option<&Arc<dyn IndexerAdapter<Error = IndexerError>>>;
    
    /// Set the indexer adapter
    fn with_indexer(self, indexer: Arc<dyn IndexerAdapter<Error = IndexerError>>) -> Self;
}

/// A domain adapter that is aware of an indexer
pub trait IndexerAwareDomainAdapter: DomainAdapter + IndexerExtension {}

// Blanket implementation for any type that implements both required traits
impl<T> IndexerAwareDomainAdapter for T where T: DomainAdapter + IndexerExtension {}

/// Helper function to convert a domain ID to a chain ID for indexer queries
pub fn domain_id_to_chain_id(domain_id: &DomainId) -> ChainId {
    // Example implementation - actual implementation would depend on domain ID format
    ChainId::new(domain_id.to_string())
}

/// Helper function to convert a resource ID to a query-compatible string
pub fn resource_id_to_query_string(resource_id: &ResourceId) -> String {
    // Example implementation - actual implementation would depend on resource ID format
    resource_id.to_string()
}

/// Default implementation of the IndexerExtension trait that can be used with any DomainAdapter
/// that supports storing an IndexerAdapter
pub struct DefaultIndexerExtension<D: DomainAdapter> {
    /// The domain adapter
    domain_adapter: D,
    
    /// The indexer adapter
    indexer: Option<Arc<dyn IndexerAdapter<Error = IndexerError>>>,
}

impl<D: DomainAdapter> DefaultIndexerExtension<D> {
    /// Create a new DefaultIndexerExtension
    pub fn new(domain_adapter: D) -> Self {
        Self {
            domain_adapter,
            indexer: None,
        }
    }
    
    /// Create a new DefaultIndexerExtension with an indexer
    pub fn with_indexer(domain_adapter: D, indexer: Arc<dyn IndexerAdapter<Error = IndexerError>>) -> Self {
        Self {
            domain_adapter,
            indexer: Some(indexer),
        }
    }
}

#[async_trait]
impl<D: DomainAdapter + Send + Sync> IndexerExtension for DefaultIndexerExtension<D> {
    async fn get_historical_facts(
        &self,
        resource_id: &ResourceId,
        from_time: Option<DateTime<Utc>>,
        to_time: Option<DateTime<Utc>>,
        limit: Option<u32>,
    ) -> Result<Vec<IndexedFact>> {
        if let Some(indexer) = &self.indexer {
            let options = QueryOptions {
                limit,
                offset: None,
                ascending: true,
            };
            
            let resource_id_str = resource_id_to_query_string(resource_id);
            let facts = indexer.get_facts_by_resource(&resource_id_str, options)
                .await
                .map_err(|e| crate::error::DomainError::Other(Box::new(e)))?;
            
            // Filter by time if needed
            let facts = facts.into_iter()
                .filter(|fact| {
                    if let Some(from) = from_time {
                        if fact.timestamp < from {
                            return false;
                        }
                    }
                    if let Some(to) = to_time {
                        if fact.timestamp > to {
                            return false;
                        }
                    }
                    true
                })
                .collect();
            
            Ok(facts)
        } else {
            Ok(Vec::new())
        }
    }
    
    fn get_indexer(&self) -> Option<&Arc<dyn IndexerAdapter<Error = IndexerError>>> {
        self.indexer.as_ref()
    }
    
    fn with_indexer(mut self, indexer: Arc<dyn IndexerAdapter<Error = IndexerError>>) -> Self {
        self.indexer = Some(indexer);
        self
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added as implementation progresses
} 