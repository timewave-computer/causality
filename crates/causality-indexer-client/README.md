# Causality Indexer Client

This crate provides a client implementation for interacting with the Almanac indexer service in Causality. It implements the `IndexerAdapter` and `IndexerAdapterFactory` traits from the `causality-indexer-adapter` crate.

## Features

- HTTP client for querying indexed blockchain data
- WebSocket client for subscribing to real-time events
- Retryable requests with configurable timeouts
- Async-first API with tokio runtime
- Complete implementation of the IndexerAdapter interface
- Optional integration with the Almanac bridge

## Usage

### Basic Example

```rust
use causality_indexer_client::{AlmanacClientConfig, create_client_factory};
use causality_indexer_adapter::{
    ChainId, IndexerAdapter, IndexerAdapterFactory, QueryOptions
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client factory with custom configuration
    let factory = create_client_factory(
        "http://indexer.example.com/api",
        "ws://indexer.example.com/ws",
        Some("api-key-123".to_string()),
    );

    // Create a client
    let client = factory.create().await?;

    // Query events by resource ID
    let facts = client.get_facts_by_resource(
        "0x1234567890abcdef1234567890abcdef12345678",
        QueryOptions {
            limit: Some(10),
            offset: None,
            ascending: false, // most recent first
        },
    ).await?;

    // Process facts
    for fact in facts {
        println!("Fact: {} (Block #{})", fact.id.0, fact.block_height);
        println!("  Chain: {}", fact.chain_id.0);
        println!("  Timestamp: {}", fact.timestamp);
        println!("  Data: {}", serde_json::to_string_pretty(&fact.data)?);
    }

    // Get chain status
    let status = client.get_chain_status(&ChainId::new("ethereum:1")).await?;
    println!("Chain status: {} blocks indexed, {} blocks behind",
        status.latest_indexed_height,
        status.indexing_lag);

    Ok(())
}
```

### Subscribing to Events

```rust
use causality_indexer_client::{create_default_client_factory};
use causality_indexer_adapter::{ChainId, FactFilter, IndexerAdapter, IndexerAdapterFactory};
use futures_util::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with default configuration
    let factory = create_default_client_factory();
    let client = factory.create().await?;

    // Create a filter for Ethereum Transfer events
    let filter = FactFilter {
        chains: Some(vec![ChainId::new("ethereum:1")]),
        resources: Some(vec!["0x1234567890abcdef1234567890abcdef12345678".to_string()]),
        event_types: Some(vec!["Transfer".to_string()]),
        from_height: Some(15_000_000), // Start from block 15M
        to_height: None, // No upper limit
    };

    // Subscribe to events
    let mut subscription = client.subscribe(filter).await?;
    
    // Process events as they arrive
    while let Ok(Some(fact)) = subscription.next_fact().await {
        println!("New event: {} (Block #{})", fact.id.0, fact.block_height);
        println!("  Data: {}", serde_json::to_string_pretty(&fact.data)?);
    }

    Ok(())
}
```

### Bridge Integration

The client can be used with the Almanac bridge to provide a unified interface. This requires enabling the `bridge-integration` feature:

```toml
[dependencies]
causality-indexer-client = { version = "0.1.0", features = ["bridge-integration"] }
```

Then you can use the bridge adapter factory:

```rust
use causality_indexer_client::{AlmanacClientConfig, create_client_factory};
use causality_indexer_client::bridge::create_bridge_adapter_factory;
use causality_indexer_adapter::{IndexerAdapterFactory};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client factory
    let client_factory = create_client_factory(
        "http://indexer.example.com/api",
        "ws://indexer.example.com/ws",
        None,
    );

    // Create a bridge adapter factory that uses the client
    let bridge_factory = create_bridge_adapter_factory(client_factory);
    
    // Create the bridge adapter
    let bridge = bridge_factory.create().await?;
    
    // Use the bridge with the unified interface...
    
    Ok(())
}
```

## Configuration

The client can be configured with the following options:

- `http_url`: Base URL for the HTTP API
- `ws_url`: Base URL for the WebSocket API
- `api_key`: Optional API key for authentication
- `http_timeout`: Timeout for HTTP requests in seconds
- `max_retries`: Maximum number of HTTP retries
- `retry_delay_ms`: Delay between retries in milliseconds

Default values are provided for all configuration options.

## Error Handling

The client provides detailed error types through the `AlmanacClientError` enum, which covers various failure modes:

- Connection errors
- HTTP errors (via reqwest)
- WebSocket errors
- JSON parsing errors
- URL parsing errors
- Subscription errors
- Invalid responses
- Not found errors

## Features Roadmap

- Connection pooling for high-throughput applications
- Improved batch querying capabilities
- Circuit breaker pattern for failure resilience
- Metrics and instrumentation 