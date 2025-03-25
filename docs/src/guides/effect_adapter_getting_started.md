<!-- Getting started with effect adapters -->
<!-- Original file: docs/src/effect_adapter_getting_started.md -->

# Getting Started with Effect Adapters

This guide will walk you through the process of creating and using effect adapters with the Causality framework.

## What are Effect Adapters?

Effect adapters are components that connect Causality programs to external domains like blockchains, APIs, and other systems. They handle:

- Converting program effects into domain-specific transactions
- Observing facts from external domains
- Validating proofs of actions or state
- Maintaining time synchronization between Causality and external domains

## Using the Adapter Generator

The fastest way to get started is to use the adapter generator, which can automatically create adapter code based on a schema definition.

### 1. Define Your Adapter Schema

Create a TOML file that describes your adapter's capabilities:

```toml
id = "ethereum"
domain_type = "blockchain" 
version = "0.1.0"

[common_metadata]
display_name = "Ethereum Adapter"
description = "An adapter for interacting with Ethereum networks"

[time_sync]
time_model = "block-based"
time_point_call = "eth_blockNumber"
finality_window = 12

# Define the effects your adapter supports
[[effect_definitions]]
effect_type = "transfer"
required_fields = ["from", "to", "value"]
optional_fields = ["gas", "gasPrice"]

# Define the facts your adapter can observe
[[fact_definitions]]
fact_type = "balance"
required_fields = ["address"]
optional_fields = ["block_number"]

# Define the proofs your adapter can validate
[[proof_definitions]]
proof_type = "transaction"
required_fields = ["tx_hash", "block_hash"]
```

Save this file as `ethereum_schema.toml`.

### 2. Generate the Adapter Code

Use the adapter generator CLI to create the adapter code:

```bash
# For Rust
cargo run --bin adapter-gen -- --input ethereum_schema.toml --output ./my_ethereum_adapter --language rust

# For TypeScript
cargo run --bin adapter-gen -- --input ethereum_schema.toml --output ./my_ethereum_adapter_ts --language typescript
```

### 3. Use the Generated Adapter

#### In Rust

```rust
use causality::effect_adapters::{EffectAdapter, EffectParams};
use my_ethereum_adapter::EthereumAdapter;

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an adapter instance
    let config = my_ethereum_adapter::EthereumConfig {
        rpc_url: "https://mainnet.infura.io/v3/YOUR_API_KEY".to_string(),
        chain_id: 1,
        // Set other configuration options
    };
    
    let adapter = EthereumAdapter::new(config)?;
    
    // Apply an effect
    let params = EffectParams {
        effect_type: "transfer".to_string(),
        params: [
            ("from".to_string(), "0x123...".as_bytes().to_vec()),
            ("to".to_string(), "0x456...".as_bytes().to_vec()),
            ("value".to_string(), "1000000000000000000".as_bytes().to_vec()),
        ].into_iter().collect(),
        // Set other parameters
        ..Default::default()
    };
    
    let receipt = adapter.apply_effect(params).await?;
    println!("Transaction ID: {}", receipt.transaction_id);
    
    // Observe a fact
    let fact_params = [
        ("address".to_string(), "0x123...".to_string()),
    ].into_iter().collect();
    
    let fact = adapter.observe_fact("balance", &fact_params).await?;
    println!("Balance: {}", String::from_utf8(fact.data)?);
    
    Ok(())
}
```

#### In TypeScript

```typescript
import { EthereumAdapter } from './my_ethereum_adapter_ts';

async function main() {
    // Create an adapter instance
    const adapter = new EthereumAdapter({
        rpcUrl: 'https://mainnet.infura.io/v3/YOUR_API_KEY',
        chainId: 1,
        // Set other configuration options
    });
    
    // Apply an effect
    const receipt = await adapter.applyEffect({
        effectType: 'transfer',
        params: {
            from: '0x123...',
            to: '0x456...',
            value: '1000000000000000000'
        }
    });
    console.log('Transaction ID:', receipt.transactionId);
    
    // Observe a fact
    const fact = await adapter.observeFact('balance', {
        address: '0x123...'
    });
    console.log('Balance:', fact.data);
}

main().catch(console.error);
```

## Creating an Adapter Manually

If you need more control, you can create adapters manually by implementing the `EffectAdapter` trait.

### Implementing the Trait

```rust
use async_trait::async_trait;
use causality::effect_adapters::{
    EffectAdapter, EffectParams, TransactionReceipt, 
    ObservedFact, AdapterError, ProofError, ObservationError
};
use causality::types::DomainId;
use std::collections::HashMap;

pub struct MyAdapter {
    domain_id: DomainId,
    config: DomainConfig,
    // Add any other fields you need
}

#[async_trait]
impl EffectAdapter for MyAdapter {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    async fn apply_effect(&self, params: EffectParams) -> Result<TransactionReceipt, AdapterError> {
        // Implement the effect application logic
        // ...
    }
    
    async fn validate_proof(&self, effect_type: &str, proof: &[u8]) -> Result<bool, ProofError> {
        // Implement proof validation logic
        // ...
    }
    
    async fn observe_fact(&self, fact_type: &str, query_params: &HashMap<String, String>) 
        -> Result<ObservedFact, ObservationError> 
    {
        // Implement fact observation logic
        // ...
    }
    
    // Implement other methods...
}
```

## Tips for Adapter Development

1. **Test coverage**: Ensure your adapter has comprehensive tests, especially for interactions with external systems.

2. **Error handling**: Provide detailed error messages that help diagnose issues with external domains.

3. **Time synchronization**: Pay special attention to time handling, especially for blockchains where finality can be probabilistic.

4. **Idempotency**: Make sure repeated effect applications are handled correctly to avoid duplicate transactions.

5. **Resource cleanup**: Properly manage connections and resources when interacting with external systems.

## Next Steps

- Check out the [API Reference](api-reference.md) for detailed documentation of the adapter interfaces
- See [Example Adapters](example-adapters.md) for more complete implementations
- Learn about [Advanced Topics](advanced-topics.md) like caching, batching, and gas optimization 