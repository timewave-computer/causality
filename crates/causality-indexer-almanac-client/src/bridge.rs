// causality-indexer-almanac-client/src/bridge.rs
//
// Bridge integration for Almanac indexer service

use super::*;
use causality_indexer_adapter::{IndexerAdapter, IndexerAdapterFactory};
use causality_indexer_bridge::{AlmanacBridge, AlmanacBridgeFactory};
use indexer_core::event::Event as AlmanacEvent;
use indexer_storage::Storage as AlmanacStorage;
use std::sync::Arc;

// For convenience, re-export relevant bridge types
pub use causality_indexer_bridge::{AlmanacBridge, AlmanacBridgeFactory, AlmanacStorageExt};

/// Storage adapter that wraps an AlmanacClient and provides the Storage trait
pub struct ClientStorageAdapter {
    /// The client to use for storage operations
    client: Arc<AlmanacClient>,
}

impl ClientStorageAdapter {
    /// Create a new adapter
    pub fn new(client: Arc<AlmanacClient>) -> Self {
        Self { client }
    }
}

// This is an implementation of the AlmanacStorage trait that uses the client
// to provide data to the bridge.
#[async_trait::async_trait]
impl AlmanacStorage for ClientStorageAdapter {
    async fn store_event(&self, _chain: &str, _event: Box<dyn AlmanacEvent>) -> indexer_core::Result<()> {
        // Not implemented - we're using the client as a read-only source
        Err(indexer_core::Error::from("Operation not supported in client adapter"))
    }
    
    async fn get_events(&self, chain: &str, from_block: u64, to_block: u64) -> indexer_core::Result<Vec<Box<dyn AlmanacEvent>>> {
        // Convert to ChainId
        let chain_id = ChainId::new(chain);
        
        // Use the client to get events
        let facts = self.client.get_facts_by_chain(
            &chain_id,
            Some(from_block),
            Some(to_block),
            QueryOptions {
                limit: None,
                offset: None,
                ascending: true,
            },
        )
        .await
        .map_err(|e| indexer_core::Error::from(e.to_string()))?;
        
        // In a full implementation, we would convert from IndexedFact to Almanac Event
        // For now, this is just a placeholder
        Ok(Vec::new())
    }
    
    async fn get_latest_block(&self, chain: &str) -> indexer_core::Result<u64> {
        // Convert to ChainId
        let chain_id = ChainId::new(chain);
        
        // Get chain status
        let status = self.client.get_chain_status(&chain_id)
            .await
            .map_err(|e| indexer_core::Error::from(e.to_string()))?;
            
        Ok(status.latest_indexed_height)
    }
    
    async fn get_latest_block_with_status(&self, chain: &str, _status: indexer_core::BlockStatus) -> indexer_core::Result<u64> {
        // Simplification - just get the latest block
        self.get_latest_block(chain).await
    }
    
    // The following methods are stubs as they're not needed for read-only access
    
    async fn mark_block_processed(&self, _chain: &str, _block_number: u64, _tx_hash: &str, _status: indexer_core::BlockStatus) -> indexer_core::Result<()> {
        Err(indexer_core::Error::from("Operation not supported in client adapter"))
    }

    async fn update_block_status(&self, _chain: &str, _block_number: u64, _status: indexer_core::BlockStatus) -> indexer_core::Result<()> {
        Err(indexer_core::Error::from("Operation not supported in client adapter"))
    }
    
    async fn get_events_with_status(&self, chain: &str, from_block: u64, to_block: u64, _status: indexer_core::BlockStatus) -> indexer_core::Result<Vec<Box<dyn AlmanacEvent>>> {
        // Simplification - ignore status
        self.get_events(chain, from_block, to_block).await
    }
    
    async fn reorg_chain(&self, _chain: &str, _from_block: u64) -> indexer_core::Result<()> {
        Err(indexer_core::Error::from("Operation not supported in client adapter"))
    }

    // Additional required methods - simplified for this example
    
    async fn set_processor_state(&self, _chain: &str, _block_number: u64, _state: &str) -> indexer_core::Result<()> {
        Err(indexer_core::Error::from("Operation not supported in client adapter"))
    }

    async fn get_processor_state(&self, _chain: &str, _block_number: u64) -> indexer_core::Result<Option<String>> {
        Err(indexer_core::Error::from("Operation not supported in client adapter"))
    }

    async fn set_historical_processor_state(&self, _chain: &str, _block_number: u64, _state: &str) -> indexer_core::Result<()> {
        Err(indexer_core::Error::from("Operation not supported in client adapter"))
    }

    async fn get_historical_processor_state(&self, _chain: &str, _block_number: u64) -> indexer_core::Result<Option<String>> {
        Err(indexer_core::Error::from("Operation not supported in client adapter"))
    }
}

/// Extension to ClientStorageAdapter for methods needed by the AlmanacBridge
#[async_trait::async_trait]
impl causality_indexer_bridge::AlmanacStorageExt for ClientStorageAdapter {
    async fn get_chains(&self) -> indexer_core::Result<Vec<String>> {
        // This would be implemented by querying available chains from Almanac
        // For now, return a static list
        Ok(vec!["ethereum:1".to_string(), "polygon:137".to_string()])
    }
}

/// Create a bridge adapter factory that uses the client as a data source
pub fn create_bridge_adapter_factory(
    client_factory: AlmanacClientFactory,
) -> impl IndexerAdapterFactory<Adapter = AlmanacBridge, Error = BoxError> {
    // Create a factory that produces AlmanacBridge instances
    AlmanacBridgeFactory::new(move || {
        // We need to handle async client creation in a synchronous context
        // Use a current_thread runtime to block on the async operation
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| Box::new(e) as BoxError)?;
            
        // Block on the async client creation
        let client = runtime.block_on(client_factory.create())
            .map_err(|e| Box::new(e) as BoxError)?;
        
        // Create a storage adapter that wraps the client
        let storage_adapter = Arc::new(ClientStorageAdapter::new(Arc::new(client)));
        
        // Return the storage adapter to be used in the bridge
        Ok(storage_adapter as Arc<dyn AlmanacStorage + Send + Sync>)
    })
} 