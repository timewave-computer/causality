# Causality Indexer Bridge

This crate provides a bridge between Causality's indexer adapter interface and Almanac's storage interface. It allows Causality applications to use Almanac as their indexer backend.

## Overview

The indexer bridge implements the `IndexerAdapter` trait from `causality-indexer-adapter` and uses Almanac's storage interface to fulfill those operations. It efficiently translates between Almanac's data model and Causality's data model.

```
┌─────────────┐     ┌───────────────────┐     ┌───────────────┐
│   Causality │     │  Indexer Bridge   │     │    Almanac    │
│ Application ├────►│                   ├────►│    Storage    │
│             │     │ AlmanacBridge     │     │               │
└─────────────┘     └───────────────────┘     └───────────────┘
```

## Features

- Translate between Almanac events and Causality facts
- Provide access to Almanac's indexed blockchain data
- Support for querying by resource ID, chain ID, and fact ID
- Real-time subscriptions to blockchain events
- Chain status monitoring

## Usage

### Basic Integration

```rust
use causality_indexer_adapter::IndexerAdapter;
use causality_indexer_bridge::{AlmanacBridge, AlmanacBridgeFactory};
use indexer_storage::Storage;
use std::sync::Arc;

// Create a factory function that provides Almanac storage
let storage_factory = || -> Result<Arc<dyn Storage + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
    // Obtain storage from Almanac
    let storage = /* initialize storage */;
    Ok(storage)
};

// Create the bridge factory
let factory = AlmanacBridgeFactory::new(storage_factory);

// Create an adapter instance
let adapter = factory.create().await?;

// Use the adapter
let facts = adapter.get_facts_by_resource("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", options).await?;
```

### Subscriptions

```rust
use causality_indexer_adapter::{ChainId, FactFilter};

// Create a filter
let filter = FactFilter {
    resources: Some(vec!["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string()]),
    chains: Some(vec![ChainId::new("ethereum:1")]),
    event_types: None,
    from_height: None,
    to_height: None,
};

// Subscribe to events
let mut subscription = adapter.subscribe(filter).await?;

// Process events
while let Some(fact) = subscription.next_fact().await? {
    println!("New event: {:?}", fact);
}
```

## Implementation Details

### Data Model Mapping

| Causality | Almanac |
|-----------|---------|
| `IndexedFact` | `Event` |
| `FactId` | `EventId` |
| `ChainId` | Chain name string |
| `resource_ids` | Resource addresses |
| `metadata` | Event metadata |

### Extensions

The bridge provides additional extensions to the Almanac storage interface to support efficient querying:

- `AlmanacStorageExt::get_chains()` - Get all chains known to the storage

## Examples

See the `examples/` directory for complete examples:

- `integration.rs` - Shows how to integrate the bridge with a Causality domain adapter

## Testing

The crate includes a comprehensive test suite with mock implementations of the storage interface for unit testing.

## License

This project is licensed under the MIT License - see the LICENSE file for details. 