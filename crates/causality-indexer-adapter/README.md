# Causality Indexer Adapter System

This crate defines the traits and data structures for consuming indexed blockchain data in Causality. It establishes a consistent interface for integrating with various blockchain indexing services.

## Architecture

The indexer adapter system follows a modular architecture that allows for different implementations:

1. **Core Adapter Interface**: Defines traits like `IndexerAdapter` and `IndexerAdapterFactory` that all implementations must conform to.
2. **Client Implementations**: Specific implementations for different indexer services (Almanac, etc.)
3. **Bridge Layer**: Optional integration with direct API access for more advanced use cases

## Core Components

### Data Structures

- `ChainId`: Identifies a blockchain network (e.g., "ethereum:1")
- `FactId`: Unique identifier for indexed facts/events
- `IndexedFact`: Core data structure representing indexed blockchain data
- `FactFilter`: Used to filter facts by resources, chains, event types, etc.
- `ChainStatus`: Provides status information about an indexed chain

### Traits

- `FactSubscription`: Async interface for subscribing to facts/events
- `IndexerAdapter`: Main interface for querying indexed data (get by ID/resource/chain, subscribe, etc.)
- `IndexerAdapterFactory`: Creates instances of adapters

## Available Implementations

### Almanac Adapter

The Almanac adapter (`causality-indexer-client`) connects to the Almanac indexer service via HTTP and WebSocket APIs. It provides a full implementation of the adapter traits for interacting with Almanac-indexed blockchain data.

Enable with the `almanac` feature:

```toml
causality-indexer-adapter = { version = "0.1", features = ["almanac"] }
```

### Almanac Client Adapter

The Almanac client adapter (`causality-indexer-almanac-client`) connects to the Almanac indexer service, offering similar functionality to the Almanac adapter but with specific optimizations for the Almanac service.

Enable with the `almanac-client` feature:

```toml
causality-indexer-adapter = { version = "0.1", features = ["almanac-client"] }
```

## Choosing an Implementation

The modularity of the system allows you to choose the appropriate indexer adapter based on your requirements:

1. **Almanac**: Use if you are working with the original Almanac indexer service
2. **Almanac Client**: Use if you are working with the Almanac client service
3. **Custom**: Implement the adapter traits for your own indexer service

## Usage Example

```rust
use causality_indexer_adapter::{ChainId, FactFilter, IndexerAdapter, IndexerAdapterFactory};

// Option 1: Use Almanac adapter
#[cfg(feature = "almanac")]
use causality_indexer_adapter::almanac::create_client_factory as create_almanac_factory;

// Option 2: Use Almanac client adapter  
#[cfg(feature = "almanac-client")]
use causality_indexer_adapter::almanac_client::create_client_factory as create_almanac_client_factory;

async fn get_transfer_events(factory: impl IndexerAdapterFactory, token_address: &str) {
    // Create an adapter instance
    let adapter = factory.create().await.expect("Failed to create adapter");
    
    // Create a filter for Transfer events on the specified token
    let filter = FactFilter {
        resources: Some(vec![token_address.to_string()]),
        chains: Some(vec![ChainId::new("ethereum:1")]),
        event_types: Some(vec!["Transfer".to_string()]),
        from_height: None,
        to_height: None,
    };
    
    // Subscribe to events
    let mut subscription = adapter.subscribe(filter).await.expect("Failed to subscribe");
    
    // Process events as they arrive
    while let Some(event) = subscription.next_fact().await.expect("Subscription error") {
        println!("Transfer event: {} at block {}", event.id.0, event.block_height);
        // Process event data...
    }
}

#[tokio::main]
async fn main() {
    // Choose the appropriate factory based on your needs
    #[cfg(feature = "almanac")]
    let factory = create_almanac_factory(
        "http://almanac-api.example.com",
        "ws://almanac-api.example.com/ws",
        None,
    );
    
    #[cfg(feature = "almanac-client")]
    let factory = create_almanac_client_factory(
        "http://almanac-api.example.com",
        "ws://almanac-api.example.com/ws",
        None,
    );
    
    // Use the factory with your function
    get_transfer_events(factory, "0x1234567890abcdef1234567890abcdef12345678").await;
}
```

## Bridge Integration

Both adapters support integration with the Almanac bridge, which provides additional functionality for direct API access. Enable the bridge integration with the appropriate feature:

```toml
causality-indexer-client = { version = "0.1", features = ["bridge-integration"] }
causality-indexer-almanac-client = { version = "0.1", features = ["bridge-integration"] }
```

## License

MIT 