// causality-indexer-almanac-client/src/lib.rs
//
// This crate provides a client implementation for interacting with the Almanac indexer service.

use async_trait::async_trait;
use causality_indexer_adapter::{
    ChainId, ChainStatus, FactFilter, FactId, IndexedFact, QueryOptions,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error};

#[cfg(feature = "client")]
mod client;

#[cfg(feature = "client")]
pub use client::*;

#[cfg(test)]
mod tests;

/// Errors that can occur in the Almanac client
#[derive(Error, Debug)]
pub enum AlmanacClientError {
    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    #[cfg(feature = "client")]
    /// HTTP error
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[cfg(feature = "client")]
    /// WebSocket error
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[cfg(feature = "client")]
    /// URL parse error
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    /// Subscription error
    #[error("Subscription error: {0}")]
    SubscriptionError(String),

    /// Invalid response
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Other error
    #[error("Error: {0}")]
    Other(String),
}

/// Configuration for the Almanac client
#[derive(Debug, Clone)]
pub struct AlmanacClientConfig {
    /// Base URL for the HTTP API
    pub http_url: String,

    /// Base URL for the WebSocket API
    pub ws_url: String,

    /// Optional API key
    pub api_key: Option<String>,

    /// Timeout for HTTP requests in seconds
    pub http_timeout: u64,

    /// Maximum number of HTTP retries
    pub max_retries: u32,

    /// Delay between retries (in milliseconds)
    pub retry_delay_ms: u64,
}

impl Default for AlmanacClientConfig {
    fn default() -> Self {
        Self {
            http_url: "http://localhost:8080".to_string(),
            ws_url: "ws://localhost:8081".to_string(),
            api_key: None,
            http_timeout: 30,
            max_retries: 3,
            retry_delay_ms: 500,
        }
    }
}

/// API models for Almanac indexer service
pub mod models {
    use super::*;

    /// Event model from the Almanac API
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AlmanacEvent {
        /// Unique event ID
        pub id: String,

        /// Chain identifier
        pub chain_id: String,

        /// Resource/contract address
        pub address: Option<String>,

        /// Additional resource addresses
        pub related_addresses: Option<Vec<String>>,

        /// Block number
        pub block_number: u64,

        /// Block hash
        pub block_hash: String,

        /// Transaction hash
        pub tx_hash: String,

        /// Timestamp in ISO 8601 format
        pub timestamp: DateTime<Utc>,

        /// Event type (e.g., "Transfer", "Approval")
        pub event_type: String,

        /// Event data
        pub data: serde_json::Value,

        /// Additional metadata
        pub metadata: Option<HashMap<String, serde_json::Value>>,
    }

    /// Chain status from the Almanac API
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AlmanacChainStatus {
        /// Chain identifier
        pub chain_id: String,

        /// Latest indexed block height
        pub latest_indexed_height: u64,

        /// Latest known chain height
        pub latest_chain_height: u64,

        /// Indexing lag (difference between chain and indexed heights)
        pub indexing_lag: u64,

        /// Whether the chain indexer is healthy
        pub is_healthy: bool,

        /// When the last block was indexed
        pub last_indexed_at: DateTime<Utc>,
    }

    /// WebSocket message for subscriptions
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WebSocketMessage {
        /// Message type
        pub message_type: String,

        /// Subscription ID (optional)
        pub subscription_id: Option<String>,

        /// Error message (if any)
        pub error: Option<String>,

        /// Event data (if message_type is "event")
        pub event: Option<AlmanacEvent>,
    }

    /// Subscription request for WebSocket
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SubscriptionRequest {
        /// Message type ("subscribe")
        pub message_type: String,

        /// Filter criteria
        pub filter: SubscriptionFilter,

        /// API key (if needed)
        pub api_key: Option<String>,
    }

    /// Filter for subscriptions
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SubscriptionFilter {
        /// Chain IDs to filter on
        pub chains: Option<Vec<String>>,

        /// Resource addresses to filter on
        pub resources: Option<Vec<String>>,

        /// Event types to filter on
        pub event_types: Option<Vec<String>>,

        /// Minimum block height (inclusive)
        pub from_height: Option<u64>,

        /// Maximum block height (inclusive)
        pub to_height: Option<u64>,
    }

    impl From<FactFilter> for SubscriptionFilter {
        fn from(filter: FactFilter) -> Self {
            Self {
                chains: filter.chains.map(|chains| chains.into_iter().map(|c| c.0).collect()),
                resources: filter.resources,
                event_types: filter.event_types,
                from_height: filter.from_height,
                to_height: filter.to_height,
            }
        }
    }

    impl From<AlmanacEvent> for IndexedFact {
        fn from(event: AlmanacEvent) -> Self {
            // Combine address and related addresses
            let mut resource_ids = Vec::new();
            if let Some(addr) = event.address {
                resource_ids.push(addr);
            }
            if let Some(related) = event.related_addresses {
                resource_ids.extend(related);
            }

            // Create metadata map
            let mut metadata = event.metadata.unwrap_or_default();
            metadata.insert("event_type".to_string(), serde_json::to_value(event.event_type).unwrap_or_default());
            metadata.insert("block_hash".to_string(), serde_json::to_value(event.block_hash).unwrap_or_default());

            IndexedFact {
                id: FactId::new(event.id),
                chain_id: ChainId::new(event.chain_id),
                resource_ids,
                timestamp: event.timestamp,
                block_height: event.block_number,
                transaction_hash: Some(event.tx_hash),
                data: event.data,
                metadata: Some(metadata),
            }
        }
    }

    impl From<AlmanacChainStatus> for ChainStatus {
        fn from(status: AlmanacChainStatus) -> Self {
            ChainStatus {
                chain_id: ChainId::new(status.chain_id),
                latest_indexed_height: status.latest_indexed_height,
                latest_chain_height: status.latest_chain_height,
                indexing_lag: status.indexing_lag,
                is_healthy: status.is_healthy,
                last_indexed_at: status.last_indexed_at,
            }
        }
    }
}

/// Create a client factory with the default configuration
#[cfg(feature = "client")]
pub fn create_default_client_factory() -> AlmanacClientFactory {
    AlmanacClientFactory::new(AlmanacClientConfig::default())
}

/// Create a client factory with a custom configuration
#[cfg(feature = "client")]
pub fn create_client_factory(
    http_url: &str,
    ws_url: &str,
    api_key: Option<String>,
) -> AlmanacClientFactory {
    AlmanacClientFactory::new(AlmanacClientConfig {
        http_url: http_url.to_string(),
        ws_url: ws_url.to_string(),
        api_key,
        ..Default::default()
    })
}

/// Bridge integration module
#[cfg(feature = "bridge-integration")]
pub mod bridge; 