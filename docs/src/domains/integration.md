<!-- Domain integration guide -->
<!-- Original file: docs/src/domain_integration.md -->

# Domain Integration Patterns

This document describes the integration patterns for working with Domains in the Causality system. Domains represent external chains or systems that Causality interacts with, and these patterns provide a structured approach to building reliable, efficient Domain interactions.

## Contents

1. [Domain Adapter Implementation](#domain-adapter-implementation)
2. [Time Map Management](#time-map-management)
3. [Fact Observation](#fact-observation)
4. [Transaction Submission](#transaction-submission)
5. [Domain Selection Strategies](#domain-selection-strategies)
6. [Monitoring and Health Checks](#monitoring-and-health-checks)
7. [Cross-domain Operations](#cross-domain-operations)
8. [Testing Domains](#testing-domains)

## Domain Adapter Implementation

### Creating a New Domain Adapter

To implement a new Domain adapter, create a struct that implements the `DomainAdapter` trait:

```rust
pub struct MyDomainAdapter {
    // Domain-specific fields (client, config, etc.)
    client: Client,
}

impl DomainAdapter for MyDomainAdapter {
    fn domain_id(&self) -> &DomainId { ... }
    
    async fn domain_info(&self) -> Result<DomainInfo> { ... }
    
    async fn current_height(&self) -> Result<BlockHeight> { ... }
    
    async fn observe_fact(&self, query: FactQuery) -> Result<ObservedFact> { ... }
    
    async fn submit_transaction(&self, tx: Transaction) -> Result<TransactionId> { ... }
    
    async fn get_transaction_receipt(&self, tx_id: &TransactionId) -> Result<TransactionReceipt> { ... }
    
    async fn get_time_map(&self) -> Result<TimeMapEntry> { ... }
    
    async fn check_connectivity(&self) -> Result<bool> { ... }
}
```

### Domain Adapter Best Practices

1. **Caching**: Cache frequently accessed data like block heights and hashes.
2. **Rate Limiting**: Implement rate limiting to avoid overwhelming external APIs.
3. **Retry Logic**: Use exponential backoff for retrying failed operations.
4. **Connection Pooling**: Reuse connections to external systems.
5. **Error Mapping**: Map Domain-specific errors to Causality error types.

## Time Map Management

The Time Map is a critical component that tracks the observed state of all Domains. Here are patterns for effective time map management:

### Periodic Time Map Synchronization

```rust
async fn sync_time_map_periodically(
    time_map: &SharedTimeMap,
    domains: &DomainRegistry,
    interval: Duration,
) {
    loop {
        // Get all Domains
        let domain_adapters = domains.list_domains();
        
        // Update time map with current state of each Domain
        for adapter in domain_adapters {
            match adapter.get_time_map().await {
                Ok(entry) => {
                    if let Err(e) = time_map.update_domain(entry) {
                        log::warn!("Failed to update time map: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Failed to get time map from Domain: {}", e);
                }
            }
        }
        
        tokio::time::sleep(interval).await;
    }
}
```

### On-Demand Time Map Updates

```rust
async fn ensure_current_time_map(
    time_map: &SharedTimeMap,
    domain: &dyn DomainAdapter,
    max_age: Duration,
) -> Result<()> {
    let current_map = time_map.get()?;
    let domain_id = domain.domain_id();
    
    // Check if Domain entry exists and is recent enough
    if let Some(last_updated) = current_map.get_last_updated(domain_id) {
        let elapsed = last_updated.elapsed()?;
        if elapsed < max_age {
            // Time map is current enough
            return Ok(());
        }
    }
    
    // Need to update
    let entry = domain.get_time_map().await?;
    time_map.update_domain(entry)?;
    
    Ok(())
}
```

## Fact Observation

Facts represent state observations from external Domains. Here are patterns for working with facts:

### Balance Query Pattern

```rust
fn build_balance_query(account: &str, domain_id: &DomainId) -> FactQuery {
    FactQuery {
        domain_id: domain_id.clone(),
        fact_type: "balance".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("account".to_string(), account.to_string());
            params
        },
        block_height: None, // Use latest
        block_hash: None,   // Use latest
        timestamp: None,    // Use latest
    }
}

async fn get_token_balance(
    domain_id: &DomainId,
    account: &str,
    token_address: &str,
    registry: &DomainRegistry,
) -> Result<u64> {
    let domain = registry.get_domain(domain_id)
        .ok_or_else(|| Error::DomainNotFound(format!("{}", domain_id)))?;
    
    let mut query = build_balance_query(account, domain_id);
    query.parameters.insert("token".to_string(), token_address.to_string());
    
    let fact = domain.observe_fact(query).await?;
    
    // Parse the fact data as a balance
    let balance = u64::from_be_bytes(
        fact.data.try_into()
            .map_err(|_| Error::InvalidDataFormat("Expected 8 bytes for balance".to_string()))?
    );
    
    Ok(balance)
}
```

### Transaction Submission

For submitting transactions to Domains:

```rust
async fn submit_and_wait(
    domain: &dyn DomainAdapter,
    tx: Transaction,
    timeout: Duration,
) -> Result<TransactionReceipt> {
    // Submit the transaction
    let tx_id = domain.submit_transaction(tx).await?;
    
    // Wait for receipt with timeout
    let start = Instant::now();
    loop {
        match domain.get_transaction_receipt(&tx_id).await {
            Ok(receipt) => {
                if receipt.status == TransactionStatus::Success ||
                   receipt.status == TransactionStatus::Failed {
                    return Ok(receipt);
                }
            }
            Err(e) => {
                log::warn!("Error getting receipt: {}", e);
            }
        }
        
        if start.elapsed() > timeout {
            return Err(Error::Timeout("Transaction confirmation timeout".to_string()));
        }
        
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
```

### Multi-Domain Transaction Coordination

```rust
async fn execute_multi_domain_transaction(
    domains: Vec<&dyn DomainAdapter>,
    txs: Vec<Transaction>,
) -> Result<Vec<TransactionReceipt>> {
    if domains.len() != txs.len() {
        return Err(Error::InvalidArgument(
            "Number of Domains must match number of transactions".to_string()
        ));
    }
    
    let mut tx_ids = Vec::new();
    
    // Submit all transactions first
    for (domain, tx) in domains.iter().zip(txs.iter()) {
        let tx_id = domain.submit_transaction(tx.clone()).await?;
        tx_ids.push((domain, tx_id));
    }
    
    // Then wait for all receipts
    let mut receipts = Vec::new();
    for (domain, tx_id) in tx_ids {
        let receipt = submit_and_wait(domain, tx.clone(), Duration::from_secs(300)).await?;
        receipts.push(receipt);
    }
    
    Ok(receipts)
}
```

## Domain Selection Strategies

### Selecting Optimal Domains

Causality provides a flexible Domain selection system:

```rust
async fn select_optimal_domain(
    selector: &DomainSelector,
    operation_type: &str,
) -> Result<DomainId> {
    // Define selection criteria
    let criteria = SelectionCriteria {
        min_reliability: Some(0.9),
        max_latency: Some(200),
        ..Default::default()
    };
    
    // Select Domain
    let result = selector.select_domain(operation_type, &criteria).await?;
    
    // Use the selected Domain
    log::info!(
        "Selected Domain {} with score {}", 
        result.domain_id,
        result.selection_score
    );
    
    Ok(result.domain_id)
}
```

### Selecting Low-Cost Domains

```rust
async fn select_low_cost_domain(
    selector: &DomainSelector,
    operation_type: &str,
) -> Result<DomainId> {
    // Define selection criteria prioritizing low cost
    let criteria = SelectionCriteria {
        max_cost: Some(10), // Max gas/transaction cost
        min_reliability: Some(0.8),
        ..Default::default()
    };
    
    // Select Domain
    let result = selector.select_domain(operation_type, &criteria).await?;
    
    Ok(result.domain_id)
}
```

### Selecting Fault-Tolerant Domains

```rust
async fn select_fault_tolerant_domains(
    selector: &DomainSelector,
    operation_type: &str,
    count: usize,
) -> Result<Vec<DomainId>> {
    // Define selection criteria
    let criteria = SelectionCriteria {
        min_reliability: Some(0.8),
        ..Default::default()
    };
    
    // Select multiple diverse Domains
    let results = selector.select_for_fault_tolerance(
        operation_type,
        &criteria,
        count
    ).await?;
    
    Ok(results.into_iter().map(|r| r.domain_id).collect())
}
```

## Monitoring and Health Checks

### Domain Health Monitoring

```rust
async fn check_domain_health(domain: &dyn DomainAdapter) -> bool {
    match domain.check_connectivity().await {
        Ok(true) => true,
        Ok(false) => {
            log::warn!("Domain {} reports it's not healthy", domain.domain_id());
            false
        },
        Err(e) => {
            log::error!("Failed to check Domain health: {}", e);
            false
        }
    }
}
```

### Domain Metrics Collection

```rust
async fn collect_domain_metrics(
    domain: &dyn DomainAdapter,
    metrics: &mut DomainMetrics,
) -> Result<()> {
    // Record start time
    let start = Instant::now();
    
    // Check connectivity
    let connected = domain.check_connectivity().await?;
    
    // Record latency
    let latency = start.elapsed().as_millis() as u64;
    
    // Update metrics
    if connected {
        metrics.avg_latency = update_moving_average(metrics.avg_latency, latency, 0.1);
        metrics.reliability = update_moving_average(metrics.reliability, 1.0, 0.05);
    } else {
        metrics.reliability = update_moving_average(metrics.reliability, 0.0, 0.2);
    }
    
    metrics.last_update = Utc::now();
    
    Ok(())
}
```

## Cross-domain Operations

### Coordinated Cross-domain Transfers

For operations that need to be coordinated across Domains:

```rust
async fn cross_domain_transfer(
    source_domain: &dyn DomainAdapter,
    target_domain: &dyn DomainAdapter,
    from_account: &str,
    to_account: &str,
    amount: u64,
) -> Result<()> {
    // 1. Create lock records on both Domains
    let lock_tx = Transaction {
        tx_type: "lock".to_string(),
        domain_id: source_domain.domain_id().clone(),
        data: amount.to_be_bytes().to_vec(),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("from".to_string(), from_account.to_string());
            metadata.insert("target_domain".to_string(), target_domain.domain_id().to_string());
            metadata
        },
    };
    
    // Submit to source Domain
    let source_tx_id = submit_and_wait(
        source_domain,
        lock_tx,
        Duration::from_secs(300),
    ).await?;
    
    // 2. Submit the mint transaction on the target Domain
    let mint_tx = Transaction {
        tx_type: "mint".to_string(),
        domain_id: target_domain.domain_id().clone(),
        data: amount.to_be_bytes().to_vec(),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("to".to_string(), to_account.to_string());
            metadata.insert("source_domain".to_string(), source_domain.domain_id().to_string());
            metadata.insert("source_tx_id".to_string(), format!("{}", source_tx_id.tx_id));
            metadata
        },
    };
    
    let target_tx_id = submit_and_wait(
        target_domain,
        mint_tx,
        Duration::from_secs(300),
    ).await?;
    
    Ok(())
}
```

## Testing Domains

### Mock Domain Adapter

For testing, create mock Domain adapters:

```rust
struct MockDomainAdapter {
    domain_id: DomainId,
    block_height: Arc<Mutex<u64>>,
    block_hash: Arc<Mutex<Vec<u8>>>,
    balances: Arc<RwLock<HashMap<String, u64>>>,
}

impl DomainAdapter for MockDomainAdapter {
    // Implement methods with test behavior
}
```

### Testing Time Map Synchronization

```rust
#[tokio::test]
async fn test_time_map_sync() {
    // Create mock Domains
    let domain1 = Arc::new(MockDomainAdapter::new(vec![1]));
    let domain2 = Arc::new(MockDomainAdapter::new(vec![2]));
    
    // Create registry and add Domains
    let mut registry = DomainRegistry::new();
    registry.register_domain(domain1.clone());
    registry.register_domain(domain2.clone());
    
    // Advance block heights
    domain1.set_block_height(100);
    domain2.set_block_height(200);
    
    // Create time map
    let time_map = SharedTimeMap::new();
    
    // Sync time map
    let domains = registry.list_domains();
    for domain in domains {
        let entry = domain.get_time_map().await.unwrap();
        time_map.update_domain(entry).unwrap();
    }
    
    // Verify time map entries
    let map = time_map.get().unwrap();
    assert_eq!(map.get_height(&domain1.domain_id), Some(100));
    assert_eq!(map.get_height(&domain2.domain_id), Some(200));
}
```

### Cross-domain Testing

```rust
#[tokio::test]
async fn test_cross_domain_transfer() {
    // Create mock Domains
    let domain1 = Arc::new(MockDomainAdapter::new(vec![1]));
    let domain2 = Arc::new(MockDomainAdapter::new(vec![2]));
    
    // Set initial balances
    domain1.set_balance("alice", 1000);
    domain2.set_balance("bob", 0);
    
    // Perform cross-domain transfer
    cross_domain_transfer(
        domain1.as_ref(),
        domain2.as_ref(),
        "alice",
        "bob",
        500
    ).await.unwrap();
    
    // Verify balances
    assert_eq!(domain1.get_balance("alice"), 500);
    assert_eq!(domain2.get_balance("bob"), 500);
}
```

This document provides a comprehensive guide to Domain integration in Causality. By following these patterns, developers can create robust, efficient, and consistent Domain adapters and operations. 