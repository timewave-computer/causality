// causality-indexer-client/src/lib.rs
//
// This crate provides a client implementation for interacting with the Almanac indexer service.

use async_trait::async_trait;
use causality_error::common::GenericError;
use causality_indexer_adapter::{
    ChainId, ChainStatus, FactFilter, FactId, FactSubscription, IndexedFact, IndexerAdapter,
    IndexerAdapterFactory, QueryOptions,
};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use reqwest::{Client as HttpClient, StatusCode, Url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error};

#[cfg(test)]
mod tests;

/// Errors that can occur in the Almanac client
#[derive(Error, Debug)]
pub enum AlmanacClientError {
    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// HTTP error
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// WebSocket error
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

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
mod models {
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

/// Almanac HTTP client
pub struct AlmanacHttpClient {
    /// HTTP client
    client: HttpClient,

    /// Base URL
    base_url: Url,

    /// API key
    api_key: Option<String>,

    /// Maximum number of retries
    max_retries: u32,

    /// Delay between retries (in milliseconds)
    retry_delay_ms: u64,
}

impl AlmanacHttpClient {
    /// Create a new Almanac HTTP client
    pub fn new(config: AlmanacClientConfig) -> Result<Self, AlmanacClientError> {
        let client = HttpClient::builder()
            .timeout(Duration::from_secs(config.http_timeout))
            .build()
            .map_err(AlmanacClientError::HttpError)?;

        let base_url = Url::parse(&config.http_url)?;

        Ok(Self {
            client,
            base_url,
            api_key: config.api_key,
            max_retries: config.max_retries,
            retry_delay_ms: config.retry_delay_ms,
        })
    }

    /// Get headers including API key if available
    fn get_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(api_key) = &self.api_key {
            if let Ok(header_value) = reqwest::header::HeaderValue::from_str(api_key) {
                headers.insert("X-API-Key", header_value);
            }
            // If header value creation fails, we just don't add the header
        }
        headers
    }

    /// Get an event by its ID
    pub async fn get_event_by_id(&self, id: &str) -> Result<Option<models::AlmanacEvent>, AlmanacClientError> {
        let url = self.base_url.join(&format!("/events/{}", id))?;

        for retry in 0..=self.max_retries {
            match self.client.get(url.clone())
                .headers(self.get_headers())
                .send()
                .await {
                Ok(response) => {
                    match response.status() {
                        StatusCode::OK => {
                            return response.json::<models::AlmanacEvent>()
                                .await
                                .map(Some)
                                .map_err(AlmanacClientError::HttpError);
                        }
                        StatusCode::NOT_FOUND => {
                            return Ok(None);
                        }
                        status => {
                            if retry == self.max_retries {
                                return Err(AlmanacClientError::InvalidResponse(format!(
                                    "Unexpected status code: {}", status
                                )));
                            }
                        }
                    }
                }
                Err(e) => {
                    if retry == self.max_retries {
                        return Err(AlmanacClientError::HttpError(e));
                    }
                }
            }

            // Retry after delay
            tokio::time::sleep(Duration::from_millis(self.retry_delay_ms)).await;
        }

        // Should never reach here due to return in the loop
        Err(AlmanacClientError::Other("Unexpected error in get_event_by_id".to_string()))
    }

    /// Get events by resource ID with query options
    pub async fn get_events_by_resource(
        &self,
        resource_id: &str,
        options: &QueryOptions,
    ) -> Result<Vec<models::AlmanacEvent>, AlmanacClientError> {
        let mut url = self.base_url.join(&format!("/resources/{}/events", resource_id))?;

        // Add query parameters
        {
            let mut query_pairs = url.query_pairs_mut();
            if let Some(limit) = options.limit {
                query_pairs.append_pair("limit", &limit.to_string());
            }
            if let Some(offset) = options.offset {
                query_pairs.append_pair("offset", &offset.to_string());
            }
            query_pairs.append_pair("order", if options.ascending { "asc" } else { "desc" });
            // query_pairs is dropped here when the block ends
        }

        for retry in 0..=self.max_retries {
            match self.client.get(url.clone())
                .headers(self.get_headers())
                .send()
                .await {
                Ok(response) => {
                    match response.status() {
                        StatusCode::OK => {
                            return response.json::<Vec<models::AlmanacEvent>>()
                                .await
                                .map_err(AlmanacClientError::HttpError);
                        }
                        status => {
                            if retry == self.max_retries {
                                return Err(AlmanacClientError::InvalidResponse(format!(
                                    "Unexpected status code: {}", status
                                )));
                            }
                        }
                    }
                }
                Err(e) => {
                    if retry == self.max_retries {
                        return Err(AlmanacClientError::HttpError(e));
                    }
                }
            }

            // Retry after delay
            tokio::time::sleep(Duration::from_millis(self.retry_delay_ms)).await;
        }

        // Should never reach here due to return in the loop
        Err(AlmanacClientError::Other("Unexpected error in get_events_by_resource".to_string()))
    }

    /// Get events by chain ID with optional block range and query options
    pub async fn get_events_by_chain(
        &self,
        chain_id: &str,
        from_height: Option<u64>,
        to_height: Option<u64>,
        options: &QueryOptions,
    ) -> Result<Vec<models::AlmanacEvent>, AlmanacClientError> {
        let mut url = self.base_url.join(&format!("/chains/{}/events", chain_id))?;

        // Add query parameters
        {
            let mut query_pairs = url.query_pairs_mut();
            if let Some(from) = from_height {
                query_pairs.append_pair("from_block", &from.to_string());
            }
            if let Some(to) = to_height {
                query_pairs.append_pair("to_block", &to.to_string());
            }
            if let Some(limit) = options.limit {
                query_pairs.append_pair("limit", &limit.to_string());
            }
            if let Some(offset) = options.offset {
                query_pairs.append_pair("offset", &offset.to_string());
            }
            query_pairs.append_pair("order", if options.ascending { "asc" } else { "desc" });
            // query_pairs is dropped here when the block ends
        }

        for retry in 0..=self.max_retries {
            match self.client.get(url.clone())
                .headers(self.get_headers())
                .send()
                .await {
                Ok(response) => {
                    match response.status() {
                        StatusCode::OK => {
                            return response.json::<Vec<models::AlmanacEvent>>()
                                .await
                                .map_err(AlmanacClientError::HttpError);
                        }
                        status => {
                            if retry == self.max_retries {
                                return Err(AlmanacClientError::InvalidResponse(format!(
                                    "Unexpected status code: {}", status
                                )));
                            }
                        }
                    }
                }
                Err(e) => {
                    if retry == self.max_retries {
                        return Err(AlmanacClientError::HttpError(e));
                    }
                }
            }

            // Retry after delay
            tokio::time::sleep(Duration::from_millis(self.retry_delay_ms)).await;
        }

        // Should never reach here due to return in the loop
        Err(AlmanacClientError::Other("Unexpected error in get_events_by_chain".to_string()))
    }

    /// Get chain status
    pub async fn get_chain_status(&self, chain_id: &str) -> Result<models::AlmanacChainStatus, AlmanacClientError> {
        let url = self.base_url.join(&format!("/chains/{}/status", chain_id))?;

        for retry in 0..=self.max_retries {
            match self.client.get(url.clone())
                .headers(self.get_headers())
                .send()
                .await {
                Ok(response) => {
                    match response.status() {
                        StatusCode::OK => {
                            return response.json::<models::AlmanacChainStatus>()
                                .await
                                .map_err(AlmanacClientError::HttpError);
                        }
                        StatusCode::NOT_FOUND => {
                            return Err(AlmanacClientError::NotFound(format!(
                                "Chain not found: {}", chain_id
                            )));
                        }
                        status => {
                            if retry == self.max_retries {
                                return Err(AlmanacClientError::InvalidResponse(format!(
                                    "Unexpected status code: {}", status
                                )));
                            }
                        }
                    }
                }
                Err(e) => {
                    if retry == self.max_retries {
                        return Err(AlmanacClientError::HttpError(e));
                    }
                }
            }

            // Retry after delay
            tokio::time::sleep(Duration::from_millis(self.retry_delay_ms)).await;
        }

        // Should never reach here due to return in the loop
        Err(AlmanacClientError::Other("Unexpected error in get_chain_status".to_string()))
    }
}

/// Subscription implementation for the WebSocket client
pub struct AlmanacSubscription {
    /// The WebSocket URL
    ws_url: String,

    /// The subscription filter
    filter: models::SubscriptionFilter,

    /// Receiver for events
    rx: mpsc::Receiver<Result<IndexedFact, AlmanacClientError>>,

    /// Flag to control the subscription thread
    is_active: Arc<Mutex<bool>>,

    /// API key (if any)
    api_key: Option<String>,
}

impl AlmanacSubscription {
    /// Create a new subscription
    pub fn new(
        ws_url: String,
        filter: models::SubscriptionFilter,
        api_key: Option<String>,
    ) -> (Self, mpsc::Sender<Result<IndexedFact, AlmanacClientError>>) {
        let (tx, rx) = mpsc::channel(100);
        let is_active = Arc::new(Mutex::new(true));

        (
            Self {
                ws_url,
                filter,
                rx,
                is_active,
                api_key,
            },
            tx,
        )
    }

    /// Start the subscription
    pub async fn start_subscription(
        &self,
        tx: mpsc::Sender<Result<IndexedFact, AlmanacClientError>>,
    ) -> Result<(), AlmanacClientError> {
        let ws_url = Url::parse(&self.ws_url)?;
        let filter = self.filter.clone();
        let is_active = self.is_active.clone();
        let api_key = self.api_key.clone();

        // Start a background task to handle the WebSocket connection
        tokio::spawn(async move {
            // Connect to WebSocket
            match connect_async(ws_url).await {
                Ok((mut ws_stream, _)) => {
                    debug!("WebSocket connected successfully");

                    // Send subscription request
                    let subscribe_msg = models::SubscriptionRequest {
                        message_type: "subscribe".to_string(),
                        filter,
                        api_key,
                    };

                    let subscribe_json = match serde_json::to_string(&subscribe_msg) {
                        Ok(json) => json,
                        Err(e) => {
                            let _ = tx.send(Err(AlmanacClientError::JsonError(e))).await;
                            return;
                        }
                    };

                    if let Err(e) = ws_stream.send(Message::Text(subscribe_json)).await {
                        let _ = tx.send(Err(AlmanacClientError::WebSocketError(e))).await;
                        return;
                    }

                    // Process messages
                    loop {
                        // Check if subscription is still active
                        if !*is_active.lock().await {
                            break;
                        }

                        tokio::select! {
                            // Process incoming WebSocket messages
                            Some(msg_result) = ws_stream.next() => {
                                match msg_result {
                                    Ok(msg) => {
                                        if let Message::Text(text) = msg {
                                            match serde_json::from_str::<models::WebSocketMessage>(&text) {
                                                Ok(ws_msg) => {
                                                    // Handle different message types
                                                    match ws_msg.message_type.as_str() {
                                                        "event" => {
                                                            if let Some(event) = ws_msg.event {
                                                                let fact = IndexedFact::from(event);
                                                                if let Err(_) = tx.send(Ok(fact)).await {
                                                                    // Channel closed, exit
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                        "error" => {
                                                            if let Some(error) = ws_msg.error {
                                                                let _ = tx.send(Err(AlmanacClientError::SubscriptionError(error))).await;
                                                            }
                                                        }
                                                        _ => {
                                                            // Ignore other message types
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    let _ = tx.send(Err(AlmanacClientError::JsonError(e))).await;
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Err(AlmanacClientError::WebSocketError(e))).await;
                                        break;
                                    }
                                }
                            }
                            // Check every second if we should exit
                            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                                // Just a timeout to check is_active periodically
                            }
                        }
                    }

                    // Send close frame
                    let _ = ws_stream.send(Message::Close(None)).await;

                }
                Err(e) => {
                    let _ = tx.send(Err(AlmanacClientError::WebSocketError(e))).await;
                }
            }
        });

        Ok(())
    }

    /// Start a subscription task that runs independently
    pub async fn start_subscription_task(
        tx: mpsc::Sender<Result<IndexedFact, AlmanacClientError>>,
        ws_url: String,
        filter: models::SubscriptionFilter,
        api_key: Option<String>,
        is_active: Arc<Mutex<bool>>,
    ) -> Result<(), AlmanacClientError> {
        // Create WebSocket URL
        let ws_url = Url::parse(&ws_url)
            .map_err(AlmanacClientError::UrlParseError)?;
        
        // Connect to WebSocket
        let (ws_stream, _) = connect_async(ws_url).await
            .map_err(AlmanacClientError::WebSocketError)?;
        
        // Split the WebSocket stream
        let (mut write, mut read) = ws_stream.split();
        
        // Send subscription request
        let request = models::SubscriptionRequest {
            message_type: "subscribe".to_string(),
            filter,
            api_key,
        };
        
        let request_json = serde_json::to_string(&request)
            .map_err(AlmanacClientError::JsonError)?;
        
        write.send(Message::Text(request_json)).await
            .map_err(AlmanacClientError::WebSocketError)?;
        
        // Read subscription confirmation
        if let Some(Ok(msg)) = read.next().await {
            if let Message::Text(text) = msg {
                let response: models::WebSocketMessage = serde_json::from_str(&text)
                    .map_err(AlmanacClientError::JsonError)?;
                
                if response.message_type == "error" {
                    if let Some(error) = response.error {
                        return Err(AlmanacClientError::SubscriptionError(error));
                    }
                }
            }
        }
        
        // Process incoming messages
        tokio::spawn(async move {
            while let Some(Ok(msg)) = read.next().await {
                if let Message::Text(text) = msg {
                    match serde_json::from_str::<models::WebSocketMessage>(&text) {
                        Ok(message) => {
                            match message.message_type.as_str() {
                                "event" => {
                                    if let Some(event) = message.event {
                                        let fact = IndexedFact::from(event);
                                        if tx.send(Ok(fact)).await.is_err() {
                                            // Receiver dropped, exit loop
                                            break;
                                        }
                                    }
                                },
                                "error" => {
                                    if let Some(error) = message.error {
                                        let _ = tx.send(Err(AlmanacClientError::SubscriptionError(error))).await;
                                    }
                                },
                                _ => {
                                    debug!("Received unknown message type: {}", message.message_type);
                                }
                            }
                        },
                        Err(e) => {
                            let _ = tx.send(Err(AlmanacClientError::JsonError(e))).await;
                        }
                    }
                }
                
                // Check if subscription is still active
                let active = *is_active.lock().await;
                if !active {
                    break;
                }
            }
            
            // Set subscription as inactive when exiting
            let mut active = is_active.lock().await;
            *active = false;
        });
        
        Ok(())
    }
}

/// FactSubscription implementation for AlmanacSubscription
#[async_trait]
impl FactSubscription for AlmanacSubscription {
    type Error = AlmanacClientError;

    async fn next_fact(&mut self) -> Result<Option<IndexedFact>, Self::Error> {
        match self.rx.recv().await {
            Some(result) => match result {
                Ok(fact) => Ok(Some(fact)),
                Err(e) => Err(e),
            },
            None => Ok(None), // Channel closed
        }
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        // Set active flag to false to stop the background task
        let mut active = self.is_active.lock().await;
        *active = false;
        
        // Clear the channel
        while self.rx.try_recv().is_ok() {}

        Ok(())
    }
}

/// Client for the Almanac indexer service
pub struct AlmanacClient {
    /// HTTP client
    http_client: AlmanacHttpClient,

    /// WebSocket URL
    ws_url: String,

    /// API key
    api_key: Option<String>,
}

impl AlmanacClient {
    /// Create a new Almanac client
    pub fn new(config: AlmanacClientConfig) -> Result<Self, AlmanacClientError> {
        let http_client = AlmanacHttpClient::new(config.clone())?;

        Ok(Self {
            http_client,
            ws_url: config.ws_url,
            api_key: config.api_key,
        })
    }
}

#[async_trait]
impl IndexerAdapter for AlmanacClient {
    type Error = AlmanacClientError;

    async fn get_facts_by_resource(
        &self,
        resource_id: &str,
        options: QueryOptions,
    ) -> Result<Vec<IndexedFact>, Self::Error> {
        let events = self.http_client.get_events_by_resource(resource_id, &options).await?;
        let facts = events.into_iter().map(IndexedFact::from).collect();
        Ok(facts)
    }

    async fn get_facts_by_chain(
        &self,
        chain_id: &ChainId,
        from_height: Option<u64>,
        to_height: Option<u64>,
        options: QueryOptions,
    ) -> Result<Vec<IndexedFact>, Self::Error> {
        let events = self.http_client
            .get_events_by_chain(&chain_id.0, from_height, to_height, &options)
            .await?;
        
        let facts = events.into_iter().map(IndexedFact::from).collect();
        Ok(facts)
    }

    async fn get_fact_by_id(&self, fact_id: &FactId) -> Result<Option<IndexedFact>, Self::Error> {
        let event = self.http_client.get_event_by_id(&fact_id.0).await?;
        Ok(event.map(IndexedFact::from))
    }

    async fn subscribe(&self, filter: FactFilter) -> Result<Box<dyn FactSubscription<Error = Self::Error> + Send>, Self::Error> {
        // Convert from FactFilter to SubscriptionFilter
        let subscription_filter = models::SubscriptionFilter::from(filter);
        
        // Create a channel for the subscription
        let (tx, rx) = mpsc::channel(100);
        
        // Create the WebSocket URL and other parameters needed for the subscription
        let ws_url = self.ws_url.clone();
        let api_key = self.api_key.clone();
        let is_active = Arc::new(Mutex::new(true));
        
        // Clone filter for the async task
        let task_filter = subscription_filter.clone();
        
        // Start subscription in a separate async task
        tokio::spawn(async move {
            if let Err(e) = AlmanacSubscription::start_subscription_task(
                tx,
                ws_url.clone(),
                task_filter,
                api_key,
                is_active.clone(),
            ).await {
                error!("Failed to start subscription: {}", e);
            }
        });
        
        // Create and return the subscription
        let subscription = AlmanacSubscription {
            ws_url: self.ws_url.clone(),
            filter: subscription_filter,
            rx,
            is_active: Arc::new(Mutex::new(true)),
            api_key: self.api_key.clone(),
        };
        
        // Return the subscription
        Ok(Box::new(subscription))
    }

    async fn get_chain_status(&self, chain_id: &ChainId) -> Result<ChainStatus, Self::Error> {
        let status = self.http_client.get_chain_status(&chain_id.0).await?;
        Ok(ChainStatus::from(status))
    }
}

/// Factory for creating Almanac clients
pub struct AlmanacClientFactory {
    /// Configuration for all created clients
    config: AlmanacClientConfig,
}

impl AlmanacClientFactory {
    /// Create a new factory with the given configuration
    pub fn new(config: AlmanacClientConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl IndexerAdapterFactory for AlmanacClientFactory {
    type Error = GenericError;
    type Adapter = AlmanacClient;

    async fn create(&self) -> Result<Self::Adapter, Self::Error> {
        AlmanacClient::new(self.config.clone())
            .map_err(|e| GenericError::new(e.to_string()))
    }
}

/// Create a client factory with the default configuration
pub fn create_default_client_factory() -> AlmanacClientFactory {
    AlmanacClientFactory::new(AlmanacClientConfig::default())
}

/// Create a client factory with a custom configuration
pub fn create_client_factory(
    http_url: &str,
    ws_url: &str,
    api_key: Option<String>,
) -> AlmanacClientFactory {
    let config = AlmanacClientConfig {
        http_url: http_url.to_string(),
        ws_url: ws_url.to_string(),
        api_key,
        ..Default::default()
    };
    
    AlmanacClientFactory::new(config)
}

// Optional bridge integration
#[cfg(feature = "bridge-integration")]
pub mod bridge {
    use super::*;
    use causality_indexer_bridge::{AlmanacBridge, AlmanacBridgeFactory};
    use indexer_core::event::Event as AlmanacEvent;
    use indexer_storage::Storage as AlmanacStorage;
    use std::sync::Arc;

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

    // This is a stub implementation of the AlmanacStorage trait that uses the client
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
            
            // Convert to Almanac events
            // This would need a full implementation to convert between types
            // For now, we return an empty list
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
        
        // Note: The following methods are stubs. In a full implementation,
        // more methods would need to be properly implemented to make the bridge fully functional.
        
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

        // Add more stubs for the remaining methods as needed
        // These are marked as unimplemented since they're not used in our read-only scenario
        
        async fn store_valence_account_instantiation(
            &self,
            _account_info: indexer_storage::ValenceAccountInfo,
            _initial_libraries: Vec<indexer_storage::ValenceAccountLibrary>,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }
        
        async fn store_valence_library_approval(
            &self,
            _account_id: &str,
            _library_info: indexer_storage::ValenceAccountLibrary,
            _update_block: u64,
            _update_tx: &str,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }
        
        async fn store_valence_library_removal(
            &self,
            _account_id: &str,
            _library_address: &str,
            _update_block: u64,
            _update_tx: &str,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }
        
        async fn store_valence_ownership_update(
            &self,
            _account_id: &str,
            _new_owner: Option<String>,
            _new_pending_owner: Option<String>,
            _new_pending_expiry: Option<u64>,
            _update_block: u64,
            _update_tx: &str,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }
        
        async fn store_valence_execution(
            &self,
            _execution_info: indexer_storage::ValenceAccountExecution,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn get_valence_account_state(&self, _account_id: &str) -> indexer_core::Result<Option<indexer_storage::ValenceAccountState>> {
            unimplemented!("Not supported in client adapter")
        }

        async fn set_valence_account_state(&self, _account_id: &str, _state: &indexer_storage::ValenceAccountState) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn delete_valence_account_state(&self, _account_id: &str) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn set_historical_valence_account_state(
            &self,
            _account_id: &str,
            _block_number: u64,
            _state: &indexer_storage::ValenceAccountState,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn get_historical_valence_account_state(
            &self,
            _account_id: &str,
            _block_number: u64,
        ) -> indexer_core::Result<Option<indexer_storage::ValenceAccountState>> {
            unimplemented!("Not supported in client adapter")
        }

        async fn delete_historical_valence_account_state(
            &self,
            _account_id: &str,
            _block_number: u64,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn set_latest_historical_valence_block(
            &self,
            _account_id: &str,
            _block_number: u64,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn get_latest_historical_valence_block(&self, _account_id: &str) -> indexer_core::Result<Option<u64>> {
            unimplemented!("Not supported in client adapter")
        }

        async fn delete_latest_historical_valence_block(&self, _account_id: &str) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn store_valence_processor_instantiation(
            &self,
            _processor_info: indexer_storage::ValenceProcessorInfo,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn store_valence_processor_config_update(
            &self,
            _processor_id: &str,
            _config: indexer_storage::ValenceProcessorConfig,
            _update_block: u64,
            _update_tx: &str,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn store_valence_processor_message(
            &self,
            _message: indexer_storage::ValenceProcessorMessage,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn update_valence_processor_message_status(
            &self,
            _message_id: &str,
            _new_status: indexer_storage::ValenceMessageStatus,
            _processed_block: Option<u64>,
            _processed_tx: Option<&str>,
            _retry_count: Option<u32>,
            _next_retry_block: Option<u64>,
            _gas_used: Option<u64>,
            _error: Option<String>,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn get_valence_processor_state(&self, _processor_id: &str) -> indexer_core::Result<Option<indexer_storage::ValenceProcessorState>> {
            unimplemented!("Not supported in client adapter")
        }

        async fn set_valence_processor_state(&self, _processor_id: &str, _state: &indexer_storage::ValenceProcessorState) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn set_historical_valence_processor_state(
            &self,
            _processor_id: &str,
            _block_number: u64,
            _state: &indexer_storage::ValenceProcessorState,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn get_historical_valence_processor_state(
            &self,
            _processor_id: &str,
            _block_number: u64,
        ) -> indexer_core::Result<Option<indexer_storage::ValenceProcessorState>> {
            unimplemented!("Not supported in client adapter")
        }

        async fn store_valence_authorization_instantiation(
            &self,
            _auth_info: indexer_storage::ValenceAuthorizationInfo,
            _initial_policy: Option<indexer_storage::ValenceAuthorizationPolicy>,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn store_valence_authorization_policy(
            &self,
            _policy: indexer_storage::ValenceAuthorizationPolicy,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn update_active_authorization_policy(
            &self,
            _auth_id: &str,
            _policy_id: &str,
            _update_block: u64,
            _update_tx: &str,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn store_valence_authorization_grant(
            &self,
            _grant: indexer_storage::ValenceAuthorizationGrant,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn revoke_valence_authorization_grant(
            &self,
            _auth_id: &str,
            _grantee: &str,
            _resource: &str,
            _revoked_at_block: u64,
            _revoked_at_tx: &str,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn store_valence_authorization_request(
            &self,
            _request: indexer_storage::ValenceAuthorizationRequest,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn update_valence_authorization_request_decision(
            &self,
            _request_id: &str,
            _decision: indexer_storage::ValenceAuthorizationDecision,
            _processed_block: Option<u64>,
            _processed_tx: Option<&str>,
            _reason: Option<String>,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn store_valence_library_instantiation(
            &self,
            _library_info: indexer_storage::ValenceLibraryInfo,
            _initial_version: Option<indexer_storage::ValenceLibraryVersion>,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn store_valence_library_version(
            &self,
            _version: indexer_storage::ValenceLibraryVersion,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn update_active_library_version(
            &self,
            _library_id: &str,
            _version: u32,
            _update_block: u64,
            _update_tx: &str,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn store_valence_library_usage(
            &self,
            _usage: indexer_storage::ValenceLibraryUsage,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn revoke_valence_library_approval(
            &self,
            _library_id: &str,
            _account_id: &str,
            _revoked_at_block: u64,
            _revoked_at_tx: &str,
        ) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn get_valence_library_state(&self, _library_id: &str) -> indexer_core::Result<Option<indexer_storage::ValenceLibraryState>> {
            unimplemented!("Not supported in client adapter")
        }

        async fn set_valence_library_state(&self, _library_id: &str, _state: &indexer_storage::ValenceLibraryState) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn get_valence_library_versions(&self, _library_id: &str) -> indexer_core::Result<Vec<indexer_storage::ValenceLibraryVersion>> {
            unimplemented!("Not supported in client adapter")
        }

        async fn get_valence_library_approvals(&self, _library_id: &str) -> indexer_core::Result<Vec<indexer_storage::ValenceLibraryApproval>> {
            unimplemented!("Not supported in client adapter")
        }

        async fn get_valence_libraries_for_account(&self, _account_id: &str) -> indexer_core::Result<Vec<indexer_storage::ValenceLibraryApproval>> {
            unimplemented!("Not supported in client adapter")
        }

        async fn get_valence_library_usage_history(
            &self,
            _library_id: &str,
            _limit: Option<usize>,
            _offset: Option<usize>,
        ) -> indexer_core::Result<Vec<indexer_storage::ValenceLibraryUsage>> {
            unimplemented!("Not supported in client adapter")
        }

        async fn set_processor_state(&self, _chain: &str, _block_number: u64, _state: &str) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn get_processor_state(&self, _chain: &str, _block_number: u64) -> indexer_core::Result<Option<String>> {
            unimplemented!("Not supported in client adapter")
        }

        async fn set_historical_processor_state(&self, _chain: &str, _block_number: u64, _state: &str) -> indexer_core::Result<()> {
            unimplemented!("Not supported in client adapter")
        }

        async fn get_historical_processor_state(&self, _chain: &str, _block_number: u64) -> indexer_core::Result<Option<String>> {
            unimplemented!("Not supported in client adapter")
        }
    }

    /// Extension to ClientStorageAdapter for methods needed by the AlmanacBridge
    #[async_trait::async_trait]
    impl causality_indexer_bridge::AlmanacStorageExt for ClientStorageAdapter {
        async fn get_chains(&self) -> indexer_core::Result<Vec<String>> {
            // This is a simplified implementation that would need to be expanded
            // with real chain data in a production environment.
            
            // Return a list of predefined chains
            // In a real implementation, we might query a chains endpoint to get this data
            Ok(vec!["ethereum".to_string(), "polygon".to_string(), "optimism".to_string()])
        }
    }

    /// Create a bridge adapter factory that uses the client as a data source
    pub fn create_bridge_adapter_factory(
        client_factory: AlmanacClientFactory,
    ) -> impl IndexerAdapterFactory<Adapter = AlmanacBridge, Error = Box<dyn std::error::Error + Send + Sync>> {
        // Create a factory that produces AlmanacBridge instances
        AlmanacBridgeFactory::new(move || {
            // Create a client
            let client = client_factory.create().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(e)
            })?;
            
            // Create a storage adapter that wraps the client
            let storage_adapter = Arc::new(ClientStorageAdapter::new(Arc::new(client)));
            
            // Return the storage adapter to be used in the bridge
            Ok(storage_adapter as Arc<dyn AlmanacStorage + Send + Sync>)
        })
    }
} 