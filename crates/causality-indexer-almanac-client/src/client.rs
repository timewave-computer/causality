// causality-indexer-almanac-client/src/client.rs
//
// Implementation of the HTTP and WebSocket clients for the Almanac indexer service

use super::*;
use causality_indexer_adapter::{FactSubscription, IndexerAdapter, IndexerAdapterFactory};
use futures_util::{SinkExt, StreamExt};
use reqwest::{Client as HttpClient, StatusCode, Url};
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::debug;

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
            if let Ok(value) = reqwest::header::HeaderValue::from_str(api_key) {
                headers.insert("X-API-Key", value);
            }
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

        // Connect to WebSocket
        let (mut ws_stream, _) = connect_async(ws_url).await?;
        debug!("WebSocket connected successfully");

        // Send subscription request
        let subscribe_msg = models::SubscriptionRequest {
            message_type: "subscribe".to_string(),
            filter,
            api_key,
        };

        let subscribe_json = serde_json::to_string(&subscribe_msg)?;
        ws_stream.send(Message::Text(subscribe_json)).await?;

        // Spawn a background task to process messages
        tokio::spawn(async move {
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
        let subscription_filter = models::SubscriptionFilter::from(filter);
        
        let (subscription, tx) = AlmanacSubscription::new(
            self.ws_url.clone(),
            subscription_filter,
            self.api_key.clone(),
        );
        
        // Start the subscription
        subscription.start_subscription(tx).await?;
        
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
    type Error = AlmanacClientError;
    type Adapter = AlmanacClient;

    async fn create(&self) -> Result<Self::Adapter, Self::Error> {
        AlmanacClient::new(self.config.clone())
    }
} 