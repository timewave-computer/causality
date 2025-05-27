// causality-indexer-bridge/src/lib.rs
//
// This crate provides a bridge between Almanac's indexer and Causality's IndexerAdapter interface

use async_trait::async_trait;
use causality_indexer_adapter::{
    ChainId, ChainReader, ChainStatus, EventStore, FactFilter, FactId, FactSubscription,
    FactSubscriptionAsync, IndexedFact, IndexerAdapter, QueryOptions,
};
use chrono::{DateTime, TimeZone, Utc};
use futures_util::{Stream, StreamExt};
use indexer_core::{
    event::Event as AlmanacEvent,
    types::{ChainId as AlmanacChainId, EventFilter as AlmanacEventFilter},
    Result as AlmanacResult,
};
use indexer_storage::{Storage as AlmanacStorage, EventFilter};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;
use thiserror::Error;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info, trace, warn};

// Include tests module
#[cfg(test)]
mod tests;

/// Errors that can occur in the Almanac bridge
#[derive(Error, Debug)]
pub enum AlmanacBridgeError {
    /// An error from the Almanac event store
    #[error("Event store error: {0}")]
    EventStoreError(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// An error from the Almanac chain reader
    #[error("Chain reader error: {0}")]
    ChainReaderError(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// Resource ID conversion error
    #[error("Failed to convert resource ID: {0}")]
    ResourceIdConversionError(String),

    /// Chain ID conversion error
    #[error("Failed to convert chain ID: {0}")]
    ChainIdConversionError(String),

    /// Subscription error
    #[error("Subscription error: {0}")]
    SubscriptionError(String),

    /// Event conversion error
    #[error("Failed to convert event: {0}")]
    EventConversionError(String),
}

// Helper function to convert SystemTime to DateTime<Utc>
fn system_time_to_datetime(time: SystemTime) -> DateTime<Utc> {
    let duration_since_epoch = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    Utc.timestamp_opt(
        duration_since_epoch.as_secs() as i64,
        duration_since_epoch.subsec_nanos(),
    )
    .unwrap_or_else(|| Utc::now())
}

/// Implements the EventStore trait for Almanac storage
pub struct AlmanacEventStore {
    storage: Arc<dyn AlmanacStorage + Send + Sync>,
}

impl AlmanacEventStore {
    pub fn new(storage: Arc<dyn AlmanacStorage + Send + Sync>) -> Self {
        Self { storage }
    }

    // Helper to convert a causality address to an Almanac address
    fn address_from_resource_id(&self, resource_id: &str) -> Result<String, AlmanacBridgeError> {
        // Parse the resource ID format
        // Common formats include:
        // 1. Simple address: "0x1234..."
        // 2. Namespaced address: "erc20:0x1234..."
        // 3. Complex identifier: "contract:0x1234.../token/123"
        
        // First, try to extract an Ethereum-style address if present
        if let Some(eth_addr) = resource_id.find("0x").map(|idx| {
            let addr_start = idx;
            let addr_end = resource_id[addr_start..].find(|c: char| !c.is_ascii_hexdigit() && c != 'x')
                .map(|end| addr_start + end)
                .unwrap_or(resource_id.len());
            
            &resource_id[addr_start..addr_end]
        }) {
            // Verify it's a valid length Ethereum address
            if eth_addr.len() >= 42 { // 0x + 40 hex chars
                return Ok(eth_addr.to_string());
            }
        }
        
        // If we have a colon, extract the part after it as a potential address
        if let Some(idx) = resource_id.find(':') {
            if idx + 1 < resource_id.len() {
                let after_colon = &resource_id[idx + 1..];
                // If there are further delimiters, extract just the address part
                let end = after_colon.find(|c: char| c == '/' || c == '.')
                    .unwrap_or(after_colon.len());
                return Ok(after_colon[..end].to_string());
            }
        }
        
        // If we can't extract a specific format, use the whole resource ID
        // Almanac's address lookup should handle this appropriately
        Ok(resource_id.to_string())
    }

    // Helper to convert Almanac event to Causality IndexedFact
    fn convert_event_to_fact(&self, event: Box<dyn AlmanacEvent>, causality_chain_id: &ChainId) 
        -> Result<IndexedFact, AlmanacBridgeError> {
        
        let timestamp = system_time_to_datetime(event.timestamp());
        
        // Extracting the address if available
        let mut resource_ids = Vec::new();
        
        // Get the main address if available
        if let Some(address) = event.address() {
            resource_ids.push(address.to_string());
        }
        
        // Try to extract additional related addresses from the raw data
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(event.raw_data()) {
            // Extract additional addresses from common fields
            let address_fields = ["from", "to", "sender", "receiver", "owner", "spender"];
            for field in address_fields {
                if let Some(addr) = json.get(field)
                   .and_then(|v| v.as_str())
                   .filter(|s| s.starts_with("0x")) {
                    // Only add if not already in resource_ids
                    if !resource_ids.contains(&addr.to_string()) {
                        resource_ids.push(addr.to_string());
                    }
                }
            }
        }
        
        // Convert event data to JSON
        let data = match serde_json::from_slice::<serde_json::Value>(event.raw_data()) {
            Ok(json) => json,
            Err(_) => {
                // If parsing as JSON fails, create a simple JSON object with raw data as base64
                let raw_base64 = base64::encode(event.raw_data());
                serde_json::json!({
                    "raw_data_base64": raw_base64,
                    "event_type": event.event_type()
                })
            }
        };
        
        // Prepare metadata including important fields
        let mut metadata = HashMap::new();
        metadata.insert("block_hash".to_string(), serde_json::to_value(event.block_hash()).unwrap_or_default());
        metadata.insert("event_type".to_string(), serde_json::to_value(event.event_type()).unwrap_or_default());
        
        // Add timestamp in different formats for easier filtering
        metadata.insert("timestamp_iso".to_string(), serde_json::to_value(timestamp.to_rfc3339()).unwrap_or_default());
        metadata.insert("timestamp_unix".to_string(), serde_json::to_value(timestamp.timestamp()).unwrap_or_default());
        
        let fact = IndexedFact {
            id: FactId::new(event.id()),
            chain_id: causality_chain_id.clone(),
            resource_ids,
            timestamp,
            block_height: event.block_number(),
            transaction_hash: Some(event.tx_hash().to_string()),
            data,
            metadata: Some(metadata),
        };
        
        Ok(fact)
    }

    async fn get_event_by_id(&self, id: &str) -> Result<Option<Self::Event>, Self::Error> {
        // Almanac doesn't have a direct get_event_by_id method, so we need to
        // implement a search by scanning all chains and events
        
        debug!("Looking for event with ID: {}", id);
        
        // Get all chains that might have this event
        // For simplicity, we'll scan all chains we know
        let chains: Vec<String> = self.storage
            .get_chains()
            .await
            .map_err(|e| AlmanacBridgeError::EventStoreError(Box::new(e)))?;
        
        // If we don't have any chains, we can't find the event
        if chains.is_empty() {
            debug!("No chains found to search for event");
            return Ok(None);
        }
        
        // Try to find the event in each chain
        for chain in chains {
            debug!("Searching for event in chain: {}", chain);
            
            // Get the latest block for this chain
            let latest_block = match self.storage.get_latest_block(&chain).await {
                Ok(block) => block,
                Err(e) => {
                    debug!("Error getting latest block for chain {}: {}", chain, e);
                    // Skip this chain if we can't get its latest block
                    continue;
                }
            };
            
            // We'll search in chunks of blocks to avoid loading too many events at once
            // Start from the latest blocks as events are more likely to be recent
            let chunk_size = 1000; // Adjust based on expected event density
            let mut from_block = latest_block.saturating_sub(chunk_size);
            
            while from_block <= latest_block {
                let to_block = std::cmp::min(from_block + chunk_size, latest_block);
                
                debug!("Searching blocks {} to {} in chain {}", from_block, to_block, chain);
                
                // Get events in this block range
                let events = match self.storage.get_events(&chain, from_block, to_block).await {
                    Ok(events) => events,
                    Err(e) => {
                        debug!("Error getting events for chain {} in blocks {} to {}: {}", 
                              chain, from_block, to_block, e);
                        // Skip this chunk if we can't get events
                        from_block = to_block + 1;
                        continue;
                    }
                };
                
                // Look for the event with the matching ID
                for event in events {
                    if event.id() == id {
                        debug!("Found event with ID {} in chain {}", id, chain);
                        return Ok(Some(event));
                    }
                }
                
                // If we've reached the beginning, we're done with this chain
                if from_block == 0 {
                    break;
                }
                
                // Move to earlier blocks
                from_block = from_block.saturating_sub(chunk_size);
            }
        }
        
        // If we get here, we didn't find the event
        debug!("Event with ID {} not found in any chain", id);
        Ok(None)
    }
}

#[async_trait]
impl EventStore for AlmanacEventStore {
    type Error = AlmanacBridgeError;
    type Address = String;
    type Event = Box<dyn AlmanacEvent>;
    type Filter = EventFilter;
    
    async fn get_events_by_address(
        &self,
        address: &Self::Address,
        options: QueryOptions,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        // We'll need to find all events that involve this address
        // We can use a custom filter for this
        let filter = EventFilter {
            chain: None, // Do not filter by chain in this case
            block_range: None, // No block range filter
            time_range: None, // No time range filter
            event_types: None, // No event type filter
            limit: options.limit.map(|l| l as usize),
            offset: options.offset.map(|o| o as usize),
        };
        
        // This is a simplified implementation - in a real implementation
        // we'd need to query for events involving this address specifically
        // The exact implementation depends on how Almanac stores and queries events
        
        let events = self.storage
            .get_events("*", 0, u64::MAX)
            .await
            .map_err(|e| AlmanacBridgeError::EventStoreError(Box::new(e)))?;
        
        // Filter events by address
        let filtered_events = events.into_iter()
            .filter(|event| {
                if let Some(event_address) = event.address() {
                    event_address == address
                } else {
                    false
                }
            })
            .collect();
        
        Ok(filtered_events)
    }
    
    async fn get_events_by_chain(
        &self,
        chain_id: &ChainId,
        from_height: Option<u64>,
        to_height: Option<u64>,
        options: QueryOptions,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        // Convert Causality ChainId to Almanac chain string
        let almanac_chain = convert_chain_id_to_almanac(chain_id)?;
        
        // Get block range
        let from_block = from_height.unwrap_or(0);
        let to_block = to_height.unwrap_or(u64::MAX);
        
        // Get events from Almanac
        let events = self.storage
            .get_events(&almanac_chain, from_block, to_block)
            .await
            .map_err(|e| AlmanacBridgeError::EventStoreError(Box::new(e)))?;
        
        // Apply pagination if needed
        let events = if let Some(limit) = options.limit {
            let offset = options.offset.unwrap_or(0) as usize;
            let limit = limit as usize;
            
            if offset < events.len() {
                events.into_iter()
                    .skip(offset)
                    .take(limit)
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            events
        };
        
        Ok(events)
    }
    
    async fn subscribe(
        &self,
        filter: Self::Filter,
    ) -> Result<Pin<Box<dyn Future<Output = Result<Option<Self::Event>, Self::Error>> + Send>>, Self::Error> {
        // This is a simplified implementation for subscription
        // In a real implementation, we'd implement a proper event stream
        
        // This is where we'd set up a subscription channel from Almanac
        // For now we'll just create a future that returns None
        
        let future = async move {
            // In a real implementation, we would poll for new events
            // from Almanac and return them one by one
            
            Ok(None as Option<Box<dyn AlmanacEvent>>)
        };
        
        Ok(Box::pin(future))
    }
}

/// Implements ChainReader for Almanac
pub struct AlmanacChainReader {
    storage: Arc<dyn AlmanacStorage + Send + Sync>,
}

impl AlmanacChainReader {
    pub fn new(storage: Arc<dyn AlmanacStorage + Send + Sync>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl ChainReader for AlmanacChainReader {
    type Error = AlmanacBridgeError;
    
    async fn get_chain_status(&self, chain_id: &ChainId) -> Result<ChainStatus, Self::Error> {
        // Convert Causality ChainId to Almanac chain string
        let almanac_chain = convert_chain_id_to_almanac(chain_id)?;
        
        // Get latest indexed block
        let latest_indexed_height = self.storage
            .get_latest_block(&almanac_chain)
            .await
            .map_err(|e| AlmanacBridgeError::ChainReaderError(Box::new(e)))?;
        
        // In a real implementation, we'd get the latest chain height from an RPC call or other source
        // For now, we'll just use the indexed height plus some buffer to simulate a small lag
        let latest_chain_height = latest_indexed_height + 10;
        
        // Calculate indexing lag
        let indexing_lag = latest_chain_height.saturating_sub(latest_indexed_height);
        
        // Consider healthy if lag is less than some threshold (e.g., 20 blocks)
        let is_healthy = indexing_lag < 20;
        
        // Get current time as last indexed time (placeholder)
        let last_indexed_at = Utc::now();
        
        Ok(ChainStatus {
            chain_id: chain_id.clone(),
            latest_indexed_height,
            latest_chain_height,
            indexing_lag,
            is_healthy,
            last_indexed_at,
        })
    }
    
    async fn get_latest_height(&self, chain_id: &ChainId) -> Result<u64, Self::Error> {
        // Convert Causality ChainId to Almanac chain string
        let almanac_chain = convert_chain_id_to_almanac(chain_id)?;
        
        // Get latest indexed block
        let latest_height = self.storage
            .get_latest_block(&almanac_chain)
            .await
            .map_err(|e| AlmanacBridgeError::ChainReaderError(Box::new(e)))?;
        
        Ok(latest_height)
    }
}

/// A subscription to events from Almanac
pub struct AlmanacSubscription {
    /// Receiver for event channel
    event_receiver: mpsc::Receiver<Result<Box<dyn AlmanacEvent>, AlmanacBridgeError>>,
    
    /// Chain ID to include in facts
    chain_id: ChainId,
    
    /// Event store for converting events to facts
    event_store: Arc<AlmanacEventStore>,
    
    /// Whether the subscription is still active
    is_active: bool,
}

impl AlmanacSubscription {
    /// Create a new subscription
    pub fn new(
        event_receiver: mpsc::Receiver<Result<Box<dyn AlmanacEvent>, AlmanacBridgeError>>,
        chain_id: ChainId,
        event_store: Arc<AlmanacEventStore>,
    ) -> Self {
        Self {
            event_receiver,
            chain_id,
            event_store,
            is_active: true,
        }
    }
}

// Base trait implementation
impl FactSubscription for AlmanacSubscription {
    type Error = AlmanacBridgeError;
}

// Async trait implementation
#[async_trait]
impl FactSubscriptionAsync for AlmanacSubscription {
    async fn next_fact(&mut self) -> Result<Option<IndexedFact>, Self::Error> {
        // If subscription is already closed, return None
        if !self.is_active {
            return Ok(None);
        }
        
        // Receive the next event from the channel with timeout
        match tokio::time::timeout(
            std::time::Duration::from_secs(5), 
            self.event_receiver.recv()
        ).await {
            // Got an event within timeout
            Ok(Some(event_result)) => {
                match event_result {
                    Ok(event) => {
                        // Convert the event to a fact
                        let fact = self.event_store.convert_event_to_fact(event, &self.chain_id)?;
                        Ok(Some(fact))
                    }
                    Err(e) => Err(e),
                }
            }
            // Channel was closed or empty
            Ok(None) => {
                // Mark subscription as inactive
                self.is_active = false;
                Ok(None)
            }
            // Timeout occurred, return None but keep subscription active
            Err(_) => Ok(None),
        }
    }
    
    async fn close(&mut self) -> Result<(), Self::Error> {
        // Mark subscription as inactive
        self.is_active = false;
        
        // Clear the channel by dropping the receiver
        self.event_receiver = mpsc::channel(1).1;
        Ok(())
    }
}

/// Helper to convert Causality ChainId to Almanac chain string
fn convert_chain_id_to_almanac(chain_id: &ChainId) -> Result<String, AlmanacBridgeError> {
    // In Almanac, chain IDs are simple strings like "ethereum" or "cosmos"
    // Causality uses a format like "ethereum:1"
    
    // Extract the network part of the chain ID
    let network = chain_id.network();
    if network.is_empty() {
        return Err(AlmanacBridgeError::ChainIdConversionError(format!(
            "Invalid chain ID format: {}", chain_id.0
        )));
    }
    
    // Return the network part as the Almanac chain string
    Ok(network.to_string())
}

/// Helper to convert Almanac chain string to Causality ChainId
fn convert_almanac_to_chain_id(almanac_chain: &str, chain_specific_id: Option<&str>) -> ChainId {
    let chain_specific = chain_specific_id.unwrap_or("0");
    ChainId::new(format!("{}:{}", almanac_chain, chain_specific))
}

/// Helper to convert FactFilter to AlmanacEventFilter
fn convert_filter_to_almanac(filter: &FactFilter) -> Result<EventFilter, AlmanacBridgeError> {
    // Extract chain if available
    let chain = if let Some(chains) = &filter.chains {
        if let Some(first_chain) = chains.first() {
            Some(convert_chain_id_to_almanac(first_chain)?)
        } else {
            None
        }
    } else {
        None
    };
    
    // Extract block range if available
    let block_range = match (filter.from_height, filter.to_height) {
        (Some(from), Some(to)) => Some((from, to)),
        (Some(from), None) => Some((from, u64::MAX)),
        (None, Some(to)) => Some((0, to)),
        (None, None) => None,
    };
    
    // Map event types if available
    let event_types = filter.event_types.clone();
    
    // Create the Almanac filter
    let almanac_filter = EventFilter {
        chain,
        block_range,
        time_range: None, // Time range not supported in Causality filter
        event_types,
        limit: None, // Limit handled separately
        offset: None, // Offset handled separately
    };
    
    Ok(almanac_filter)
}

/// Bridge adapter that implements IndexerAdapter by using Almanac's crates
pub struct AlmanacBridge {
    /// The Almanac event store implementation
    event_store: Arc<AlmanacEventStore>,
    
    /// The Almanac chain reader implementation
    chain_reader: Arc<AlmanacChainReader>,
}

impl AlmanacBridge {
    /// Create a new bridge
    pub fn new(
        storage: Arc<dyn AlmanacStorage + Send + Sync>,
    ) -> Self {
        let event_store = Arc::new(AlmanacEventStore::new(storage.clone()));
        let chain_reader = Arc::new(AlmanacChainReader::new(storage));
        
        Self {
            event_store,
            chain_reader,
        }
    }
}

#[async_trait]
impl IndexerAdapter for AlmanacBridge {
    type Error = AlmanacBridgeError;
    
    async fn get_facts_by_resource(
        &self,
        resource_id: &str,
        options: QueryOptions,
    ) -> Result<Vec<IndexedFact>, Self::Error> {
        // Convert the resource ID to an Almanac address
        let address = self.event_store.address_from_resource_id(resource_id)?;
        
        // Get events from Almanac
        let events = self.event_store
            .get_events_by_address(&address, options)
            .await?;
        
        // Convert events to facts
        let mut facts = Vec::with_capacity(events.len());
        for event in events {
            // We don't know the chain ID from this call, so we need to extract it from the event
            let almanac_chain = event.chain();
            let chain_id = convert_almanac_to_chain_id(almanac_chain, None);
            
            let fact = self.event_store.convert_event_to_fact(event, &chain_id)?;
            facts.push(fact);
        }
        
        Ok(facts)
    }
    
    async fn get_facts_by_chain(
        &self,
        chain_id: &ChainId,
        from_height: Option<u64>,
        to_height: Option<u64>,
        options: QueryOptions,
    ) -> Result<Vec<IndexedFact>, Self::Error> {
        // Get events from Almanac
        let events = self.event_store
            .get_events_by_chain(chain_id, from_height, to_height, options)
            .await?;
        
        // Convert events to facts
        let mut facts = Vec::with_capacity(events.len());
        for event in events {
            let fact = self.event_store.convert_event_to_fact(event, chain_id)?;
            facts.push(fact);
        }
        
        Ok(facts)
    }
    
    async fn get_fact_by_id(&self, fact_id: &FactId) -> Result<Option<IndexedFact>, Self::Error> {
        // Get the event from Almanac
        let event_result = self.event_store.get_event_by_id(&fact_id.0).await?;
        
        if let Some(event) = event_result {
            // Extract chain ID from the event
            let almanac_chain = event.chain();
            let chain_id = convert_almanac_to_chain_id(almanac_chain, None);
            
            // Convert to a fact
            let fact = self.event_store.convert_event_to_fact(event, &chain_id)?;
            Ok(Some(fact))
        } else {
            Ok(None)
        }
    }
    
    async fn subscribe(&self, filter: FactFilter) -> Result<Box<dyn FactSubscription<Error = Self::Error> + Send>, Self::Error> {
        // Convert filter to Almanac format
        let almanac_filter = convert_filter_to_almanac(&filter)?;
        
        // Extract chain ID for the subscription
        let chain_id = if let Some(chains) = &filter.chains {
            if let Some(first_chain) = chains.first() {
                first_chain.clone()
            } else {
                ChainId::new("unknown:0")
            }
        } else {
            ChainId::new("unknown:0")
        };
        
        // Create a channel for events with a reasonable buffer size
        let (tx, rx) = mpsc::channel(100);
        
        // Set up a real subscription to Almanac events
        // This needs to run in a background task that forwards events to our channel
        let storage = self.event_store.storage.clone();
        let sender = tx.clone();
        
        // This task will poll for new events and send them to the channel
        tokio::spawn(async move {
            let from_block = filter.from_height.unwrap_or_else(|| {
                // If no from_height specified, start from current latest block
                match storage.get_latest_block(&convert_chain_id_to_almanac(&chain_id).unwrap_or_default()).await {
                    Ok(height) => height,
                    Err(_) => 0, // If error, start from 0
                }
            });
            
            // Keep track of the highest block we've seen
            let mut last_block = from_block;
            
            // Polling loop runs until channel is closed (when subscription is closed)
            loop {
                // Sleep for 2 seconds between polls
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                
                // Check if channel is closed
                if sender.is_closed() {
                    break;
                }
                
                // Get latest block
                let latest_block = match storage.get_latest_block(&convert_chain_id_to_almanac(&chain_id).unwrap_or_default()).await {
                    Ok(height) => height,
                    Err(_) => {
                        // If error, try again next iteration
                        continue;
                    }
                };
                
                // If no new blocks, continue
                if latest_block <= last_block {
                    continue;
                }
                
                // Get events from the last processed block to the latest
                let almanac_chain = match convert_chain_id_to_almanac(&chain_id) {
                    Ok(chain) => chain,
                    Err(e) => {
                        // Send error to channel
                        let _ = sender.send(Err(e)).await;
                        break;
                    }
                };
                
                // Query for new events
                match storage.get_events(&almanac_chain, last_block + 1, latest_block).await {
                    Ok(events) => {
                        // Send each event to the channel
                        for event in events {
                            // Apply additional filtering if needed based on resource IDs
                            if let Some(resources) = &filter.resources {
                                if let Some(addr) = event.address() {
                                    if !resources.iter().any(|r| r == addr || addr.contains(r) || r.contains(addr)) {
                                        continue; // Skip this event
                                    }
                                }
                            }
                            
                            // Apply filtering based on event types
                            if let Some(event_types) = &filter.event_types {
                                if !event_types.iter().any(|t| t == event.event_type()) {
                                    continue; // Skip this event
                                }
                            }
                            
                            // Send event to channel
                            if let Err(_) = sender.send(Ok(event)).await {
                                // Channel was closed, break out of loop
                                break;
                            }
                        }
                        
                        // Update last block processed
                        last_block = latest_block;
                    }
                    Err(e) => {
                        // Send error to channel
                        let _ = sender.send(Err(AlmanacBridgeError::EventStoreError(Box::new(e)))).await;
                        // Continue trying
                    }
                }
            }
        });
        
        // Create the subscription
        let subscription = AlmanacSubscription::new(
            rx,
            chain_id,
            self.event_store.clone(),
        );
        
        Ok(Box::new(subscription))
    }
    
    async fn get_chain_status(&self, chain_id: &ChainId) -> Result<ChainStatus, Self::Error> {
        // Delegate to chain reader
        self.chain_reader.get_chain_status(chain_id).await
    }
}

/// Factory for creating AlmanacBridge instances
pub struct AlmanacBridgeFactory {
    /// Factory for creating storage
    storage_factory: Arc<dyn Fn() -> Result<Arc<dyn AlmanacStorage + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>,
}

impl AlmanacBridgeFactory {
    /// Create a new factory
    pub fn new(
        storage_factory: impl Fn() -> Result<Arc<dyn AlmanacStorage + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            storage_factory: Arc::new(storage_factory),
        }
    }
}

#[async_trait]
impl causality_indexer_adapter::IndexerAdapterFactory for AlmanacBridgeFactory {
    type Error = AlmanacBridgeError;
    type Adapter = AlmanacBridge;
    
    async fn create(&self) -> Result<Self::Adapter, Self::Error> {
        // Create the storage
        let storage = (self.storage_factory)()
            .map_err(|e| AlmanacBridgeError::EventStoreError(e))?;
        
        // Create the bridge
        Ok(AlmanacBridge::new(storage))
    }
}

// Extension trait for Almanac storage to add additional methods
/// This trait extends the Almanac storage with additional methods needed by the bridge
#[async_trait]
pub trait AlmanacStorageExt: AlmanacStorage {
    /// Get all chains known to the storage
    async fn get_chains(&self) -> indexer_core::Result<Vec<String>>;
}

// Implement the extension trait for any type implementing AlmanacStorage
#[async_trait]
impl<T: AlmanacStorage + ?Sized> AlmanacStorageExt for T {
    async fn get_chains(&self) -> indexer_core::Result<Vec<String>> {
        // Default implementation just returns an empty list
        // Each storage implementation should override this with a proper implementation
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};
    
    #[test]
    fn test_convert_chain_id() {
        let causality_chain_id = ChainId::new("ethereum:1");
        let almanac_chain = convert_chain_id_to_almanac(&causality_chain_id).unwrap();
        assert_eq!(almanac_chain, "ethereum");
        
        let converted_back = convert_almanac_to_chain_id(&almanac_chain, Some("1"));
        assert_eq!(converted_back.0, causality_chain_id.0);
    }
    
    #[test]
    fn test_system_time_to_datetime() {
        let now = SystemTime::now();
        let datetime = system_time_to_datetime(now);
        
        // The converted time should be close to now
        let now_datetime = Utc::now();
        let diff = (now_datetime - datetime).num_seconds().abs();
        assert!(diff < 2, "Converted time differs by more than 2 seconds");
    }
    
    // Additional tests would be added for other components
} 