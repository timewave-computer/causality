# Committee Indexing and Log Reconstruction

This document provides a comprehensive guide to the Committee Indexing and Log Reconstruction system, a crucial component of the Causality framework that enables external observation, extraction, and reconstruction of facts from blockchain networks.

## Overview

Committees are third-party entities running light clients that interact with blockchain nodes. They act as trusted observers, extracting facts from external chains and reconstructing logs that can be shared with other system components. The Committee Indexing system provides a robust framework for this observation, extraction, and reconstruction process.

## Core Components

### 1. External Indexer Proxy

The External Indexer Proxy component serves as an intermediary between the committee and external blockchain networks. It handles:

- Connection to external chain nodes via RPC
- Block and transaction observation
- Event monitoring and extraction
- Callback registration for new data

```rust
// Creating and initializing a committee with proxy capabilities
let config = CommitteeConfig {
    id: "ethereum-indexer".to_string(),
    proxy: ProxyConfig {
        chains: vec![IndexerConfig {
            chain_id: "ethereum-mainnet".to_string(),
            chain_type: "ethereum".to_string(),
            rpc_url: "https://mainnet.infura.io/v3/your-key".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    },
    ..Default::default()
};

let committee = create_committee(config)?;
committee.initialize().await?;
committee.start().await?;
```

### 2. Fact Extraction Rules

The Fact Extraction Rules system enables the definition of rules for extracting facts from blockchain data. It includes:

- Rule definition in TOML format
- Conditional extraction based on patterns
- Field mapping from source to destination
- Transformations and validations

```rust
// Example of a fact extraction rule for Ethereum ERC-20 transfers
let rule = ExtractionRule {
    rule_id: "erc20-transfer".to_string(),
    chain_id: "ethereum-mainnet".to_string(),
    domain: "tokens".to_string(),
    fact_type: "transfer".to_string(),
    conditions: vec![
        ExtractionCondition::Equals("event_signature".to_string(), 
                                  "Transfer(address,address,uint256)".to_string())
    ],
    field_mappings: vec![
        FieldMapping {
            source_field: "topics[1]".to_string(),
            destination_field: "from_address".to_string(),
            transformation: Some("address_decode".to_string()),
        },
        FieldMapping {
            source_field: "topics[2]".to_string(),
            destination_field: "to_address".to_string(),
            transformation: Some("address_decode".to_string()),
        },
        FieldMapping {
            source_field: "data".to_string(),
            destination_field: "amount".to_string(),
            transformation: Some("uint256_decode".to_string()),
        },
    ],
};

committee.add_extraction_rule(rule)?;
```

Example TOML rule file:

```toml
# ERC-20 Transfer Rule
[[rules]]
rule_id = "erc20-transfer"
chain_id = "ethereum-mainnet"
domain = "tokens"
fact_type = "transfer"

[[rules.conditions]]
type = "equals"
field = "event_signature"
value = "Transfer(address,address,uint256)"

[[rules.field_mappings]]
source_field = "topics[1]"
destination_field = "from_address"
transformation = "address_decode"

[[rules.field_mappings]]
source_field = "topics[2]"
destination_field = "to_address"
transformation = "address_decode"

[[rules.field_mappings]]
source_field = "data"
destination_field = "amount"
transformation = "uint256_decode"
```

Loading rules from TOML:

```rust
let toml_str = std::fs::read_to_string("rules.toml")?;
committee.load_extraction_rules_from_toml(&toml_str)?;
```

### 3. FactLog Reconstruction

The FactLog Reconstruction component enables rebuilding of logs from extracted facts. It includes:

- Conversion of extracted facts to log entries
- Deduplication to prevent duplicate entries
- Batch processing for efficiency
- Storage to persistent log stores

```rust
// The log reconstruction system is automatically connected to the proxy
// through the committee initialization process. Facts extracted by the
// proxy are forwarded to the reconstruction engine, which rebuilds logs.

// Getting a reconstructor for a specific domain
let reconstructor = committee
    .reconstructor_registry()
    .get("tokens")?;

// Get statistics about the reconstruction process
let stats = reconstructor.get_stats()?;
println!("Facts processed: {}", stats.facts_processed);
println!("Entries reconstructed: {}", stats.entries_reconstructed);

// Get the reconstructed log storage
let storage = reconstructor.storage();
let entries = storage.get_entries(0, 100).await?;
```

### 4. Data Provider Interface

The Data Provider Interface offers a standardized way to access external data sources beyond blockchain nodes. It includes:

- Unified interface for different data sources
- Query capabilities with parameters
- Authentication and rate limiting
- Response parsing utilities

```rust
// Creating and using a data provider
let provider_config = ProviderConfig {
    id: "coingecko-api".to_string(),
    provider_type: "http".to_string(),
    connection: {
        let mut map = HashMap::new();
        map.insert("base_url".to_string(), "https://api.coingecko.com/api/v3".to_string());
        map
    },
    ..Default::default()
};

// Register and connect to the provider
let provider = committee
    .provider_registry()
    .create_and_register(provider_config)?;

provider.connect().await?;

// Query data from the provider
let query = DataQuery::new("coins/markets")
    .with_parameter("vs_currency", "usd")
    .with_parameter("ids", "bitcoin,ethereum");

let response = provider.query(&query).await?;
let data = response.body_as_json()?;
```

## Integration Flow

The Committee Indexing and Log Reconstruction system operates in the following flow:

1. **Initialization**: Committee is initialized with configuration for proxies, reconstructors, and providers.
2. **Connection**: Proxies connect to external chain nodes and providers to data sources.
3. **Extraction**: As new blocks are observed, the proxy extracts facts using the defined rules.
4. **Reconstruction**: Extracted facts are sent to reconstructors which rebuild logs.
5. **Query**: Applications can query the reconstructed logs or external data through providers.

```rust
// Complete integration flow example
async fn run_committee() -> Result<()> {
    // 1. Create configuration
    let config = CommitteeConfig {
        id: "main-committee".to_string(),
        proxy: ProxyConfig {
            chains: vec![
                IndexerConfig {
                    chain_id: "ethereum-mainnet".to_string(),
                    chain_type: "ethereum".to_string(),
                    rpc_url: "https://mainnet.infura.io/v3/your-key".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        },
        reconstruction: ReconstructionConfig {
            domain: "tokens".to_string(),
            ..Default::default()
        },
        providers: vec![
            ProviderConfig {
                id: "coingecko-api".to_string(),
                provider_type: "http".to_string(),
                connection: {
                    let mut map = HashMap::new();
                    map.insert("base_url".to_string(), 
                              "https://api.coingecko.com/api/v3".to_string());
                    map
                },
                ..Default::default()
            },
        ],
    };

    // 2. Create and initialize committee
    let committee = create_committee(config)?;
    committee.initialize().await?;

    // 3. Load extraction rules
    let toml_str = std::fs::read_to_string("rules.toml")?;
    committee.load_extraction_rules_from_toml(&toml_str)?;

    // 4. Start the committee
    committee.start().await?;

    // 5. Application can now access reconstructed logs
    // ... wait for extraction and reconstruction ...
    
    let reconstructor = committee.reconstructor_registry().get("tokens")?;
    let storage = reconstructor.storage();
    let entries = storage.get_entries(0, 100).await?;
    
    // 6. Access external data through providers
    let provider = committee.provider_registry().get("coingecko-api")?;
    let query = DataQuery::new("coins/markets")
        .with_parameter("vs_currency", "usd")
        .with_parameter("ids", "bitcoin,ethereum");
    
    let response = provider.query(&query).await?;
    
    // 7. Process queried data and combine with reconstructed logs
    // ...

    // 8. Properly shut down when done
    committee.stop().await?;
    
    Ok(())
}
```

## Best Practices

1. **Rule Design**
   - Make extraction rules as specific as possible to avoid irrelevant data
   - Use conditions to filter out unwanted events
   - Group related extractions into domains

2. **Performance Optimization**
   - Use batch processing for efficiency
   - Implement appropriate caching strategies
   - Consider rate limiting for external providers

3. **Error Handling**
   - Implement robust error handling for network failures
   - Log errors for debugging
   - Add retry mechanisms for transient failures

4. **Security**
   - Secure API keys and authentication tokens
   - Validate extracted facts before processing
   - Implement proper access controls for reconstructed logs

## Monitoring and Maintenance

1. **Status Monitoring**
   - Regularly check the status of proxies and providers
   - Monitor reconstruction statistics
   - Set up alerts for connectivity issues

2. **Log Management**
   - Implement log rotation for reconstructed logs
   - Archive old logs to save space
   - Periodically verify log integrity

3. **Rule Updates**
   - Update extraction rules when chain contracts change
   - Test new rules before deploying
   - Version control your rule definitions

## Conclusion

The Committee Indexing and Log Reconstruction system provides a powerful framework for observing external chains, extracting facts, and reconstructing logs that can be shared with other components of the Causality system. By following the guidelines and best practices outlined in this documentation, you can effectively use this system to build robust, data-driven applications that interact with blockchain networks.

With the ability to define custom extraction rules, reconstruct logs, and access external data through a unified interface, the system offers flexibility and extensibility to meet various use cases and requirements. 