# Domain Adapter Architecture

## Overview

The Domain Adapter architecture in Causality provides a unified interface for interacting with different blockchain networks (domains). Each domain has its own specific implementation that conforms to the `DomainAdapter` trait, allowing the system to work with multiple chains in a consistent manner.

## Directory Structure

```
src/
├── domain/           # Core domain interfaces and shared functionality
│   ├── adapter.rs    # DomainAdapter trait and related types
│   ├── fact.rs       # Fact observation and verification
│   ├── mod.rs        # Domain module definition and re-exports
│   ├── registry.rs   # Domain registry for managing adapters
│   ├── selection.rs  # Domain selection logic
│   ├── time_map.rs   # Time synchronization across domains
│   └── time_sync.rs  # Time synchronization manager
└── domain_adapters/          # Domain-specific adapter implementations
    ├── evm/          # Ethereum Virtual Machine adapter
    │   ├── adapter.rs # EVM-specific adapter implementation
    │   ├── mod.rs     # EVM module definition
    │   └── types.rs   # EVM-specific types
    └── ... other domain adapters
```

## Architecture

### Core Components

1. **DomainAdapter Trait**
   - Defined in `src/domain/adapter.rs`
   - Provides a consistent interface for all domain interactions
   - Methods for getting information, observing facts, submitting transactions
   - Currently being updated to use the new standardized fact system

2. **Domain Registry**
   - Manages all registered domain adapters
   - Allows lookup by domain ID
   - Provides domain metadata

3. **Time Map**
   - Synchronizes time across different domains
   - Maps domain block heights and hashes to consistent timestamps

4. **Domain Selection**
   - Selects appropriate domains based on criteria like reliability, cost, latency

5. **Fact System Integration**
   - Domain adapters are being migrated to use the standardized `FactType` enum
   - Provides type-safe fact observation and verification
   - Enables strong typing for domain-specific facts

### Domain-Specific Adapters

Domain-specific adapters implement the `DomainAdapter` trait and handle the specifics of interacting with a particular blockchain or system. Each adapter is isolated in its own module under `src/domain_adapters/`.

#### Example: EVM Adapter

The EVM adapter (in `src/domain_adapters/evm/`) provides:

- Connection to Ethereum-compatible chains
- Methods to query balances, storage, logs
- Transaction submission and receipt retrieval
- Block information and time synchronization

## Adding a New Domain Adapter

To add support for a new blockchain:

1. Create a new directory under `src/domain_adapters/` for your adapter
2. Implement the required modules:
   - `mod.rs` - Module definition and exports
   - `adapter.rs` - Main adapter implementation
   - `types.rs` - Domain-specific types

3. Implement the `DomainAdapter` trait for your adapter
4. Register your adapter with the `DomainRegistry`

## Usage Example

```rust
// Create an EVM adapter for Ethereum
let eth_config = EthereumConfig {
    domain_id: DomainId::from_str("0x01").unwrap(),
    name: "Ethereum Mainnet".to_string(),
    description: Some("Ethereum main network".to_string()),
    rpc_url: "https://mainnet.infura.io/v3/YOUR_API_KEY".to_string(),
    chain_id: 1,
    explorer_url: Some("https://etherscan.io".to_string()),
    native_currency: "ETH".to_string(),
};

let eth_adapter = EthereumAdapter::new(eth_config)?;

// Register with the domain registry
let mut registry = DomainRegistry::new();
registry.register_domain(Arc::new(eth_adapter));

// Observe facts from the domain
// Note: The domain adapter interface is being migrated to use the new FactType system
let query = FactQuery {
    domain_id: eth_adapter.domain_id().clone(),
    fact_type: "balance".to_string(),
    parameters: {
        let mut params = HashMap::new();
        params.insert("address".to_string(), "0x...".to_string());
        params
    },
    block_height: None,
    block_hash: None,
    timestamp: None,
};

// Observe the fact (in future this will directly return the standardized FactType)
let observed_fact = eth_adapter.observe_fact(query).await?;
```

## Best Practices

1. **Isolate Dependencies**: Each adapter should keep its chain-specific dependencies contained within its module
2. **Clean API Surface**: Only export what's necessary from the domain-specific modules
3. **Error Handling**: Convert chain-specific errors to Causality's error types
4. **Caching**: Implement appropriate caching for expensive blockchain calls
5. **Testing**: Create mock implementations for testing purposes
6. **Fact Standardization**: Use the standardized `FactType` system for all new code 