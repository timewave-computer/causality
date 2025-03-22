# Domain Adapters in the Three-Layer Effect Architecture

## Overview

Domain adapters provide a consistent interface between the Causality system and external blockchain networks. The domain adapter layer is a critical component of the three-layer effect architecture, bridging the gap between abstract effects and concrete chain-specific implementations.

## Architecture Layers

The three-layer effect architecture consists of:

1. **Abstract Effect Layer**: Defines effect interfaces and constraints
2. **Domain Adapter Layer**: Translates abstract effects to domain-specific implementations
3. **Chain-Specific Layer**: Handles the actual on-chain execution

## Domain Adapter Components

Each domain adapter includes:

- **Core Adapter**: Main adapter implementation with connection handling
- **Type Definitions**: Domain-specific data types
- **Effect Implementations**: Concrete implementations of abstract effects
- **Storage Strategies**: Domain-specific storage for the unified ResourceRegister model

## Supported Domains

The system currently supports the following domains:

- **EVM** (Ethereum Virtual Machine): For Ethereum and compatible chains
- **CosmWasm**: For Cosmos ecosystem chains
- **Other**: Generic adapter for testing and custom implementations

## Implementation Pattern

Domain adapters follow a consistent pattern:

```rust
// Example module structure for a domain adapter
pub mod domain_name {
    // Core adapter implementation
    pub mod adapter;
    
    // Type definitions
    pub mod types;
    
    // Storage strategy implementation
    pub mod storage_strategy;
    
    // Re-export core types and create factory functions
    pub use adapter::DomainAdapter;
    pub use adapter::DomainConfig;
    pub use types::DomainAddress;
    pub use storage_strategy::DomainStorageEffectFactory;
    
    // Factory function for creating adapters
    pub fn create_domain_adapter(config: DomainConfig) -> Result<DomainAdapter> {
        DomainAdapter::new(config)
    }
}
```

## Storage Strategies

Domain adapters implement storage strategies for the unified ResourceRegister model:

- **FullyOnChain**: All register data is stored on-chain
- **CommitmentBased**: Only commitments to register data are stored on-chain
- **Hybrid**: Critical fields stored on-chain, with commitments for remaining data

Each domain implements custom storage effects:

- `DomainStoreEffect`: For storing register data on-chain
- `DomainCommitmentEffect`: For storing register commitment on-chain
- `DomainNullifierEffect`: For storing register nullifier on-chain

## Integration Example

```rust
// Creating a domain-specific store effect
let domain_info = domain_registry.get_domain_info(&domain_id)?;
let effect = create_domain_specific_store_effect(
    register_id,
    fields,
    domain_id,
    invoker,
    &domain_info,
)?;

// Execute using the effect runtime
let outcome = effect_runtime.execute_effect(effect, context).await?;
```

## EVM Adapter

The EVM adapter supports Ethereum and other EVM-compatible chains with:

- Ethers.rs integration for contract interactions
- Gas estimation and transaction management
- Support for different network types (mainnet, testnet)
- ABIs for common register storage contracts

## CosmWasm Adapter

The CosmWasm adapter supports Cosmos ecosystem chains with:

- CosmWasm contract interaction
- Support for different network types and gas configurations
- Integration with various Cosmos SDK modules

## Adding New Domain Adapters

To add a new domain adapter:

1. Create a new module in `src/domain_adapters/{domain_name}/`
2. Implement the core adapter with connection handling
3. Define domain-specific types
4. Implement storage strategies for the unified ResourceRegister model
5. Register the domain type in the domain registry
6. Update effect factory functions to support the new domain

## Storage Effect Factory Pattern

Domain adapters provide effect factories to create domain-specific effects:

```rust
// Example storage effect factory
pub struct DomainStorageEffectFactory {
    contract_address: String,
    config: DomainConfig,
    domain_id: DomainId,
}

impl DomainStorageEffectFactory {
    // Create a store effect
    pub fn create_store_effect(&self, register_id: ResourceId, fields: HashSet<String>, invoker: Address) -> Result<Arc<dyn Effect>>;
    
    // Create a commitment effect
    pub fn create_commitment_effect(&self, register_id: ResourceId, commitment: Commitment, invoker: Address) -> Result<Arc<dyn Effect>>;
}
``` 