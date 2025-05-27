# Causality Almanac Client Adapter

This crate provides an alternative client implementation for interacting with the Almanac indexer service in Causality. It implements the `IndexerAdapter` and `IndexerAdapterFactory` traits from the `causality-indexer-adapter` crate.

## Features

- HTTP client for querying indexed blockchain events by ID, resource, or chain
- WebSocket client for subscribing to new events in real-time
- Robust retry logic and error handling
- Conversion between Almanac's event format and Causality's `IndexedFact` format
- Optional integration with the Almanac bridge

## Usage

### Basic Client Usage

```rust
use causality_indexer_almanac_client::{AlmanacClientConfig, create_client_factory};
use causality_indexer_adapter::{ChainId, FactFilter, IndexerAdapter, IndexerAdapterFactory, QueryOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client factory with custom configuration
    let factory = create_client_factory(
        "http://almanac-api.example.com", 
        "ws://almanac-api.example.com/ws",
        Some("your-api-key".to_string())
    );
    
    // Create a client instance
    let client = factory.create().await?;
    
    // Query events by resource ID
    let events = client.get_facts_by_resource(
        "0x1234567890abcdef1234567890abcdef12345678",
        QueryOptions::default()
    ).await?;
    
    for event in events {
        println!("Event ID: {}, Block: {}", event.id.0, event.block_height);
    }
    
    // Query chain status
    let status = client.get_chain_status(&ChainId::new("ethereum:1")).await?;
    println!("Latest indexed block: {}", status.latest_indexed_height);
    
    // Subscribe to events
    let filter = FactFilter {
        resources: Some(vec!["0x1234567890abcdef1234567890abcdef12345678".to_string()]),
        chains: Some(vec![ChainId::new("ethereum:1")]),
        event_types: Some(vec!["Transfer".to_string()]),
        from_height: Some(15_000_000),
        to_height: None,
    };
    
    let mut subscription = client.subscribe(filter).await?;
    
    // Process events as they arrive
    while let Some(event) = subscription.next_fact().await? {
        println!("Received event: {} at block {}", event.id.0, event.block_height);
        
        // Process the event data
        // ...
    }
    
    Ok(())
}
```

### Bridge Integration

The client can be used with the Almanac bridge to provide a unified interface. This requires enabling the `bridge-integration` feature:

```toml
causality-indexer-almanac-client = { version = "0.1", features = ["bridge-integration"] }
```

```rust
use causality_indexer_almanac_client::{AlmanacClientConfig, create_client_factory};
use causality_indexer_almanac_client::bridge::create_bridge_adapter_factory;
use causality_indexer_adapter::{IndexerAdapterFactory};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client factory
    let client_factory = create_client_factory(
        "http://almanac-api.example.com", 
        "ws://almanac-api.example.com/ws",
        Some("your-api-key".to_string())
    );
    
    // Create a bridge adapter factory
    let bridge_factory = create_bridge_adapter_factory(client_factory);
    
    // Create a bridge adapter
    let bridge = bridge_factory.create().await?;
    
    // Use the bridge adapter as an IndexerAdapter
    // ...
    
    Ok(())
}
```

## Error Handling

The client provides detailed error types through the `AlmanacClientError` enum, which covers various failure modes:

- Connection errors
- HTTP errors
- WebSocket errors
- JSON parsing errors
- Subscription errors
- Not found errors

## Configuration

The `AlmanacClientConfig` struct allows customizing the client behavior:

```rust
let config = AlmanacClientConfig {
    http_url: "http://almanac-api.example.com".to_string(),
    ws_url: "ws://almanac-api.example.com/ws".to_string(),
    api_key: Some("your-api-key".to_string()),
    http_timeout: 30, // seconds
    max_retries: 3,
    retry_delay_ms: 500,
};

// Create a factory with this configuration
let factory = AlmanacClientFactory::new(config);
```

## License

MIT 