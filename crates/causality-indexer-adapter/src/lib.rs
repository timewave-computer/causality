// causality-indexer-adapter/src/lib.rs
//
// This crate defines traits and data structures for consuming indexed blockchain data in Causality

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

/// A chain identifier in the format "network:chain-id", e.g., "ethereum:1" for Ethereum mainnet
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChainId(pub String);

impl ChainId {
    /// Create a new chain ID
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Parse a chain ID from a string in the format "network:chain-id"
    pub fn parse(value: &str) -> Option<Self> {
        if value.contains(':') {
            Some(Self(value.to_string()))
        } else {
            None
        }
    }

    /// Get the network part of the chain ID
    pub fn network(&self) -> &str {
        self.0.split(':').next().unwrap_or("")
    }

    /// Get the chain-specific part of the chain ID
    pub fn chain_specific_id(&self) -> &str {
        self.0.split(':').nth(1).unwrap_or("")
    }
}

/// A unique identifier for a blockchain fact or event
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactId(pub String);

impl FactId {
    /// Create a new fact ID
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

/// Represents indexed chain data in a Causality-compatible format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFact {
    /// Unique identifier for this fact
    pub id: FactId,
    
    /// The chain this fact was indexed from
    pub chain_id: ChainId,
    
    /// List of resource IDs this fact is associated with (e.g., contract addresses)
    pub resource_ids: Vec<String>,
    
    /// When this fact occurred (timestamp from the block)
    pub timestamp: DateTime<Utc>,
    
    /// Block height this fact was observed at
    pub block_height: u64,
    
    /// Transaction hash (if applicable)
    pub transaction_hash: Option<String>,
    
    /// Fact-specific data (contract event data, transaction data, etc.)
    pub data: serde_json::Value,
    
    /// Additional metadata (indexer-specific information)
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Filter for fact subscriptions and queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactFilter {
    /// Filter by resource IDs (e.g., contract addresses)
    pub resources: Option<Vec<String>>,
    
    /// Filter by chain IDs
    pub chains: Option<Vec<ChainId>>,
    
    /// Filter by event types (e.g., "Transfer", "Approval", etc.)
    pub event_types: Option<Vec<String>>,
    
    /// Filter by minimum block height (inclusive)
    pub from_height: Option<u64>,
    
    /// Filter by maximum block height (inclusive)
    pub to_height: Option<u64>,
}

/// Status information about an indexed chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStatus {
    /// The chain identifier
    pub chain_id: ChainId,
    
    /// Latest block height that has been indexed
    pub latest_indexed_height: u64,
    
    /// Latest known block height on the chain
    pub latest_chain_height: u64,
    
    /// Difference between latest chain height and indexed height
    pub indexing_lag: u64,
    
    /// Whether the indexer considers this chain healthy
    pub is_healthy: bool,
    
    /// When the last block was indexed
    pub last_indexed_at: DateTime<Utc>,
}

/// Options for querying events
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    /// Maximum number of events to return
    pub limit: Option<u32>,
    
    /// Number of events to skip
    pub offset: Option<u32>,
    
    /// Whether to sort in ascending or descending order
    /// - true: oldest events first (ascending by block height or timestamp)
    /// - false: newest events first (descending by block height or timestamp)
    pub ascending: bool,
}

/// A subscription to a stream of facts
#[async_trait]
pub trait FactSubscription: Send + Sync {
    /// The error type for subscription operations
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Receive the next fact from the subscription
    /// 
    /// Returns:
    /// - Ok(Some(fact)) if a fact is available
    /// - Ok(None) if no facts are currently available (may receive more later)
    /// - Err(_) if an error occurred
    async fn next_fact(&mut self) -> Result<Option<IndexedFact>, Self::Error>;
    
    /// Close the subscription and release any resources
    async fn close(&mut self) -> Result<(), Self::Error>;
}

/// Core trait for querying indexed blockchain data
#[async_trait]
pub trait IndexerAdapter: Send + Sync {
    /// The error type for adapter operations
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Get facts by resource ID (e.g., contract address)
    /// 
    /// Parameters:
    /// - resource_id: The resource identifier (usually a contract address)
    /// - options: Query options for pagination and sorting
    async fn get_facts_by_resource(
        &self, 
        resource_id: &str,
        options: QueryOptions,
    ) -> Result<Vec<IndexedFact>, Self::Error>;
    
    /// Get facts by chain ID and optional block range
    /// 
    /// Parameters:
    /// - chain_id: The chain to query
    /// - from_height: Optional minimum block height (inclusive)
    /// - to_height: Optional maximum block height (inclusive)
    /// - options: Query options for pagination and sorting
    async fn get_facts_by_chain(
        &self, 
        chain_id: &ChainId, 
        from_height: Option<u64>,
        to_height: Option<u64>,
        options: QueryOptions,
    ) -> Result<Vec<IndexedFact>, Self::Error>;
    
    /// Get a specific fact by its ID
    /// 
    /// Parameters:
    /// - fact_id: The unique identifier of the fact
    /// 
    /// Returns:
    /// - Ok(Some(fact)) if the fact was found
    /// - Ok(None) if no fact with that ID exists
    /// - Err(_) if an error occurred
    async fn get_fact_by_id(&self, fact_id: &FactId) -> Result<Option<IndexedFact>, Self::Error>;
    
    /// Subscribe to new facts matching a filter
    /// 
    /// This creates a subscription that will receive facts as they are indexed.
    /// The subscription can be polled for new facts using the next_fact() method.
    /// 
    /// Parameters:
    /// - filter: Criteria for which facts to include in the subscription
    async fn subscribe(&self, filter: FactFilter) -> Result<Box<dyn FactSubscription<Error = Self::Error> + Send>, Self::Error>;
    
    /// Get the status of a chain
    /// 
    /// Parameters:
    /// - chain_id: The chain to get status for
    async fn get_chain_status(&self, chain_id: &ChainId) -> Result<ChainStatus, Self::Error>;
}

/// Creates new indexer adapter instances
#[async_trait]
pub trait IndexerAdapterFactory: Send + Sync {
    /// The error type for adapter creation
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// The concrete adapter type produced
    type Adapter: IndexerAdapter;
    
    /// Create a new adapter instance
    async fn create(&self) -> Result<Self::Adapter, Self::Error>;
}

/// Feature flag for enabling Almanac integration
#[cfg(feature = "almanac")]
pub use almanac;

/// Feature flag for enabling Almanac client integration
#[cfg(feature = "almanac-client")]
pub use almanac_client;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id_parse() {
        let chain_id = ChainId::parse("ethereum:1").unwrap();
        assert_eq!(chain_id.network(), "ethereum");
        assert_eq!(chain_id.chain_specific_id(), "1");
    }
} 