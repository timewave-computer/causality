# Implementing the Domain System

*This guide provides practical implementations for working with the [Domain System](../../architecture/core/domain-system.md).*

*Last updated: 2023-03-26*

## Overview

This guide covers the practical aspects of implementing and working with the Domain System in Causality. It provides code examples, best practices, and implementation patterns for creating and using domains, domain adapters, and cross-domain operations.

## Prerequisites

Before implementing domain-related functionality in your code, make sure you're familiar with:

- The [Domain System Architecture](../../architecture/core/domain-system.md)
- The [Effect System](../../architecture/core/effect-system.md)
- The [Resource System](../../architecture/core/resource-system.md)
- General networking and blockchain concepts

## Implementation Guide

### Required Crates and Imports

```rust
// Core domain types
use causality_types::{
    domain::{
        Domain, DomainId, DomainType, DomainError,
        DomainOperation, OperationType, DomainStatus
    },
    hash::ContentAddressed,
    identity::Identity,
};

// Domain system components
use causality_domain::{
    adapter::{
        DomainAdapter, DomainManager, DomainRegistry,
        StateQuery, StateObservation, EventFilter
    },
    boundary::{
        BoundaryCrossing, CrossingContext, CrossingReceipt,
        CrossingId, CrossingStatus, BoundaryError
    },
    events::{
        EventSubscription, DomainEvent, EventId,
        MultiDomainSubscription
    },
    blockchain::{
        ethereum::EthereumAdapter,
        solana::SolanaAdapter,
    },
    database::{
        sql::SqlAdapter,
        nosql::NoSqlAdapter,
    },
};

// Integration with other systems
use causality_core::{
    effects::{Effect, EffectContext},
    resources::{ResourceId, ResourceManager},
};

// Standard imports
use std::{
    collections::HashMap,
    sync::Arc,
};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use async_trait::async_trait;
```

### Creating and Registering Domains

```rust
/// Create and register a blockchain domain
async fn create_blockchain_domain(
    registry: Arc<dyn DomainRegistry>,
) -> Result<DomainId, DomainError> {
    // Create an Ethereum domain
    let ethereum_domain = Domain::new(
        "ethereum-mainnet",
        DomainType::Blockchain {
            consensus: ConsensusType::ProofOfStake,
            vm_type: VmType::Evm,
        },
        // Domain configuration
        serde_json::to_value(EthereumConfig {
            chain_id: 1,
            rpc_endpoint: "https://mainnet.infura.io/v3/your-api-key".to_string(),
            block_confirmation_count: 12,
        }).unwrap(),
    )?;
    
    // Register the domain
    let domain_id = registry.register_domain(ethereum_domain).await?;
    
    println!("Registered Ethereum domain with ID: {}", domain_id);
    
    Ok(domain_id)
}

/// Create and register a database domain
async fn create_database_domain(
    registry: Arc<dyn DomainRegistry>,
) -> Result<DomainId, DomainError> {
    // Create a PostgreSQL domain
    let postgres_domain = Domain::new(
        "postgres-main",
        DomainType::Database {
            db_type: DatabaseType::Sql,
        },
        // Domain configuration
        serde_json::to_value(SqlConfig {
            connection_string: "postgres://user:password@localhost:5432/mydb".to_string(),
            max_connections: 10,
            ssl_enabled: true,
        }).unwrap(),
    )?;
    
    // Register the domain
    let domain_id = registry.register_domain(postgres_domain).await?;
    
    println!("Registered PostgreSQL domain with ID: {}", domain_id);
    
    Ok(domain_id)
}
```

### Creating Domain Adapters

```rust
/// Create an Ethereum domain adapter
async fn create_ethereum_adapter(
    domain_id: DomainId,
    config: EthereumConfig,
) -> Result<Arc<dyn DomainAdapter>, DomainError> {
    // Create an Ethereum client
    let client = Arc::new(EthereumHttpClient::new(&config.rpc_endpoint)?);
    
    // Create the adapter
    let adapter = Arc::new(EthereumAdapter::new(domain_id, client, config)?);
    
    // Initialize the adapter
    adapter.initialize(&config).await?;
    
    Ok(adapter)
}

/// Create a PostgreSQL domain adapter
async fn create_postgres_adapter(
    domain_id: DomainId,
    config: SqlConfig,
) -> Result<Arc<dyn DomainAdapter>, DomainError> {
    // Create a SQL connection
    let connection = Arc::new(PostgresConnection::new(&config.connection_string)?);
    
    // Create the adapter
    let adapter = Arc::new(SqlAdapter::new(domain_id, connection, config)?);
    
    // Initialize the adapter
    adapter.initialize(&config).await?;
    
    Ok(adapter)
}
```

### Using the Domain Manager

```rust
/// Get domain adapters from the domain manager
async fn use_domain_manager(
    manager: Arc<dyn DomainManager>,
    ethereum_domain_id: &DomainId,
    postgres_domain_id: &DomainId,
) -> Result<(), DomainError> {
    // Get the Ethereum adapter
    let ethereum_adapter = manager.get_adapter(ethereum_domain_id).await?;
    
    // Check the domain status
    let ethereum_status = ethereum_adapter.check_status().await?;
    println!("Ethereum domain status: {:?}", ethereum_status);
    
    // Get the PostgreSQL adapter
    let postgres_adapter = manager.get_adapter(postgres_domain_id).await?;
    
    // Check the domain status
    let postgres_status = postgres_adapter.check_status().await?;
    println!("PostgreSQL domain status: {:?}", postgres_status);
    
    Ok(())
}
```

### Executing Domain Operations

```rust
/// Execute operations on a blockchain domain
async fn execute_blockchain_operations(
    adapter: Arc<dyn DomainAdapter>,
) -> Result<(), DomainError> {
    // Create a read operation (get balance)
    let balance_operation = DomainOperation::new(
        adapter.domain_id().clone(),
        OperationType::Read,
        serde_json::to_value(BalanceQuery {
            address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        }).unwrap(),
    )?;
    
    // Execute the operation
    let balance_result = adapter.execute_operation(balance_operation).await?;
    
    println!("Balance result: {}", balance_result.data["balance"]);
    
    // Create a write operation (send transaction)
    let tx_operation = DomainOperation::new(
        adapter.domain_id().clone(),
        OperationType::Write,
        serde_json::to_value(EthereumTransaction {
            from: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            to: "0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
            value: "1000000000000000000", // 1 ETH
            gas_limit: 21000,
            gas_price: "20000000000", // 20 Gwei
            data: vec![],
        }).unwrap(),
    )?;
    
    // Execute the operation
    let tx_result = adapter.execute_operation(tx_operation).await?;
    
    println!("Transaction hash: {}", tx_result.data["tx_hash"]);
    
    Ok(())
}

/// Execute operations on a database domain
async fn execute_database_operations(
    adapter: Arc<dyn DomainAdapter>,
) -> Result<(), DomainError> {
    // Create a read operation (query)
    let query_operation = DomainOperation::new(
        adapter.domain_id().clone(),
        OperationType::Read,
        serde_json::to_value(SqlQuery {
            query: "SELECT * FROM users WHERE id = $1".to_string(),
            parameters: vec!["user123".into()],
        }).unwrap(),
    )?;
    
    // Execute the operation
    let query_result = adapter.execute_operation(query_operation).await?;
    
    println!("Query result: {:?}", query_result.data["rows"]);
    
    // Create a write operation (insert)
    let insert_operation = DomainOperation::new(
        adapter.domain_id().clone(),
        OperationType::Write,
        serde_json::to_value(SqlQuery {
            query: "INSERT INTO users (id, name, email) VALUES ($1, $2, $3)".to_string(),
            parameters: vec![
                "user456".into(),
                "Alice".into(),
                "alice@example.com".into(),
            ],
        }).unwrap(),
    )?;
    
    // Execute the operation
    let insert_result = adapter.execute_operation(insert_operation).await?;
    
    println!("Insert result: {:?}", insert_result.data["rows_affected"]);
    
    Ok(())
}
```

### Observing Domain State

```rust
/// Observe state from a blockchain domain
async fn observe_blockchain_state(
    adapter: Arc<dyn DomainAdapter>,
) -> Result<(), DomainError> {
    // Create a state query for block height
    let block_query = StateQuery::new(
        "block/latest".to_string(),
        serde_json::to_value(BlockHeightParams {
            include_transactions: false,
        }).unwrap(),
        true, // Require proof
    );
    
    // Execute the query
    let block_observation = adapter.observe_state(block_query).await?;
    
    println!(
        "Latest block: {} at {}",
        block_observation.data["number"],
        block_observation.timestamp
    );
    
    // If proof was requested, verify it
    if let Some(proof) = block_observation.proof {
        let is_valid = verify_blockchain_proof(
            adapter.domain_id(),
            &block_observation.data,
            &proof,
        )?;
        
        println!("Block observation proof is valid: {}", is_valid);
    }
    
    Ok(())
}

/// Observe state from a database domain
async fn observe_database_state(
    adapter: Arc<dyn DomainAdapter>,
) -> Result<(), DomainError> {
    // Create a state query for a database table
    let table_query = StateQuery::new(
        "table/users/count".to_string(),
        serde_json::to_value(TableParams {
            where_clause: "active = true".to_string(),
        }).unwrap(),
        false, // No proof required
    );
    
    // Execute the query
    let table_observation = adapter.observe_state(table_query).await?;
    
    println!(
        "Active users count: {} at {}",
        table_observation.data["count"],
        table_observation.timestamp
    );
    
    Ok(())
}
```

### Subscribing to Domain Events

```rust
/// Subscribe to events from a blockchain domain
async fn subscribe_to_blockchain_events(
    adapter: Arc<dyn DomainAdapter>,
) -> Result<(), DomainError> {
    // Create an event filter for new blocks
    let block_filter = EventFilter::new(
        vec!["block.new".to_string()],
        serde_json::to_value(BlockFilterParams {
            include_transactions: false,
        }).unwrap(),
    );
    
    // Subscribe to events
    let mut subscription = adapter.subscribe(block_filter).await?;
    
    // Process events
    for _ in 0..10 {
        match subscription.next_event().await? {
            Some(event) => {
                println!(
                    "New block event: number={}, hash={}, timestamp={}",
                    event.data["number"],
                    event.data["hash"],
                    event.timestamp
                );
                
                // Acknowledge the event
                subscription.acknowledge(&event.id).await?;
            }
            None => {
                println!("No more events");
                break;
            }
        }
    }
    
    // Close the subscription
    subscription.close().await?;
    
    Ok(())
}

/// Subscribe to events from a database domain
async fn subscribe_to_database_events(
    adapter: Arc<dyn DomainAdapter>,
) -> Result<(), DomainError> {
    // Create an event filter for table changes
    let table_filter = EventFilter::new(
        vec!["table.insert".to_string(), "table.update".to_string()],
        serde_json::to_value(TableFilterParams {
            table_name: "users".to_string(),
        }).unwrap(),
    );
    
    // Subscribe to events
    let mut subscription = adapter.subscribe(table_filter).await?;
    
    // Process events
    for _ in 0..10 {
        match subscription.next_event().await? {
            Some(event) => {
                println!(
                    "Table event: type={}, table={}, row_id={}, timestamp={}",
                    event.event_type,
                    event.data["table_name"],
                    event.data["row_id"],
                    event.timestamp
                );
                
                // Acknowledge the event
                subscription.acknowledge(&event.id).await?;
            }
            None => {
                println!("No more events");
                break;
            }
        }
    }
    
    // Close the subscription
    subscription.close().await?;
    
    Ok(())
}
```

### Cross-Domain Operations

```rust
/// Execute operations across multiple domains
async fn execute_cross_domain_operations(
    manager: Arc<dyn DomainManager>,
    ethereum_domain_id: &DomainId,
    postgres_domain_id: &DomainId,
) -> Result<(), DomainError> {
    // Create an Ethereum read operation
    let eth_operation = DomainOperation::new(
        ethereum_domain_id.clone(),
        OperationType::Read,
        serde_json::to_value(BalanceQuery {
            address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        }).unwrap(),
    )?;
    
    // Create a PostgreSQL write operation that depends on the Ethereum operation
    let postgres_operation = DomainOperation::new_with_dependencies(
        postgres_domain_id.clone(),
        OperationType::Write,
        serde_json::to_value(SqlQuery {
            query: "INSERT INTO balances (address, balance, timestamp) VALUES ($1, $2, $3)".to_string(),
            parameters: vec![
                "0x1234567890abcdef1234567890abcdef12345678".into(),
                "0".into(), // Will be replaced with actual balance
                Utc::now().to_rfc3339().into(),
            ],
        }).unwrap(),
        vec![eth_operation.id()],
    )?;
    
    // Create the operation context
    let context = OperationContext::new(
        Identity::from_public_key(&[/* public key */]),
    );
    
    // Execute the cross-domain operations
    let results = manager.execute_cross_domain(
        vec![eth_operation, postgres_operation],
        &context,
    ).await?;
    
    // Process the results
    println!("Cross-domain operation results:");
    for result in &results {
        println!(
            "Domain: {}, Operation: {}, Status: {}",
            result.domain_id,
            result.operation_id,
            result.status
        );
    }
    
    Ok(())
}
```

### Boundary Crossing

```rust
/// Cross a boundary between domains
async fn cross_domain_boundary(
    boundary: Arc<dyn BoundaryCrossing>,
    from_domain_id: &DomainId,
    to_domain_id: &DomainId,
) -> Result<(), BoundaryError> {
    // Create a payload to transfer
    let payload = Payload::new(
        serde_json::to_value(TransferData {
            asset_id: "eth:0x1234567890abcdef1234567890abcdef12345678".to_string(),
            amount: "1000000000000000000", // 1 ETH
            recipient: "0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
        }).unwrap(),
    )?;
    
    // Create a crossing context
    let context = CrossingContext::new(
        CrossingId::new(),
        Identity::from_public_key(&[/* sender public key */]),
        Identity::from_public_key(&[/* receiver public key */]),
        Utc::now(),
        vec![/* capabilities */],
    );
    
    // Cross the boundary
    let receipt = boundary.cross_boundary(
        from_domain_id,
        to_domain_id,
        &payload,
        &context,
    ).await?;
    
    println!("Boundary crossing receipt: {}", receipt.id());
    
    // Verify the crossing
    let is_valid = boundary.verify_crossing(&receipt).await?;
    println!("Crossing is valid: {}", is_valid);
    
    // Check the crossing status
    let status = boundary.crossing_status(receipt.id()).await?;
    println!("Crossing status: {:?}", status);
    
    Ok(())
}
```

### Domain-Specific Effects

```rust
/// Create a domain-specific effect
fn create_domain_effect(
    domain_id: &DomainId,
) -> Result<DomainEffect, EffectError> {
    // Create an underlying effect
    let transfer_effect = TransferEffect::new(
        ResourceId::from_string("eth:0x1234567890abcdef1234567890abcdef12345678")?,
        ResourceId::from_string("eth:0xabcdef1234567890abcdef1234567890abcdef12")?,
        "1000000000000000000".to_string(), // 1 ETH
    )?;
    
    // Create domain-specific parameters
    let params = HashMap::from([
        ("gas_limit".to_string(), Value::from(21000)),
        ("gas_price".to_string(), Value::from("20000000000")), // 20 Gwei
    ]);
    
    // Create the domain effect
    let domain_effect = DomainEffect::new(
        domain_id.clone(),
        Box::new(transfer_effect),
        params,
    )?;
    
    Ok(domain_effect)
}

/// Execute a domain-specific effect
async fn execute_domain_effect(
    effect_engine: Arc<dyn EffectEngine>,
    domain_effect: DomainEffect,
) -> Result<EffectOutcome, EffectError> {
    // Create an effect context
    let context = EffectContext::new(
        Identity::from_public_key(&[/* public key */]),
        None, // No parent context
    );
    
    // Execute the effect
    let outcome = effect_engine.execute_effect(&domain_effect, &context).await?;
    
    println!("Effect outcome: {:?}", outcome);
    
    Ok(outcome)
}
```

## Best Practices

### Domain Design

1. **Domain Isolation**
   ```rust
   // GOOD: Clear domain boundaries
   struct UserService {
       user_domain: Arc<dyn DomainAdapter>,
       // Other fields...
   }
   
   struct PaymentService {
       payment_domain: Arc<dyn DomainAdapter>,
       // Other fields...
   }
   
   // BAD: Mixed domain responsibilities
   struct UserPaymentService {
       domain: Arc<dyn DomainAdapter>,
       // Mixed user and payment logic...
   }
   ```

2. **Domain Configuration**
   ```rust
   // GOOD: Domain-specific configuration
   let ethereum_config = EthereumConfig {
       chain_id: 1,
       rpc_endpoint: "https://mainnet.infura.io/v3/your-api-key".to_string(),
       block_confirmation_count: 12,
   };
   
   // BAD: Generic configuration
   let generic_config = HashMap::from([
       ("chain_id".to_string(), "1".to_string()),
       ("rpc_endpoint".to_string(), "https://mainnet.infura.io/v3/your-api-key".to_string()),
       ("confirmations".to_string(), "12".to_string()),
   ]);
   ```

### Domain Operations

1. **Operation Dependencies**
   ```rust
   // GOOD: Explicit dependencies
   let read_operation = DomainOperation::new(
       domain_id.clone(),
       OperationType::Read,
       // Parameters...
   )?;
   
   let write_operation = DomainOperation::new_with_dependencies(
       domain_id.clone(),
       OperationType::Write,
       // Parameters...
       vec![read_operation.id()],
   )?;
   
   // BAD: Implicit dependencies
   let read_operation = DomainOperation::new(
       domain_id.clone(),
       OperationType::Read,
       // Parameters...
   )?;
   
   let write_operation = DomainOperation::new(
       domain_id.clone(),
       OperationType::Write,
       // Parameters...
   )?;
   
   // Missing dependency declaration
   ```

2. **Error Handling**
   ```rust
   // GOOD: Domain-specific error handling
   match adapter.execute_operation(operation).await {
       Ok(result) => {
           // Handle success
       },
       Err(DomainError::NotFound(id)) => {
           println!("Resource not found: {}", id);
       },
       Err(DomainError::PermissionDenied) => {
           println!("Permission denied");
       },
       Err(e) => {
           println!("Unexpected error: {}", e);
       }
   }
   
   // BAD: Generic error handling
   match adapter.execute_operation(operation).await {
       Ok(result) => {
           // Handle success
       },
       Err(e) => {
           println!("Error: {}", e);
       }
   }
   ```

### Cross-Domain Operations

1. **Operation Ordering**
   ```rust
   // GOOD: Explicit ordering with dependencies
   let operations = vec![
       operation1.clone(),
       operation2.with_dependencies(vec![operation1.id()]),
       operation3.with_dependencies(vec![operation2.id()]),
   ];
   
   // BAD: Implicit ordering
   let operations = vec![
       operation1,
       operation2, // Depends on operation1 but not declared
       operation3, // Depends on operation2 but not declared
   ];
   ```

2. **Transaction Boundaries**
   ```rust
   // GOOD: Explicit transaction boundary
   let tx_context = TransactionContext::new(
       operations,
       TransactionBoundary::AllOrNothing,
   );
   
   let results = manager.execute_transaction(tx_context).await?;
   
   // BAD: No transaction boundary
   let results = manager.execute_cross_domain(operations, &context).await?;
   ```

### Event Handling

1. **Acknowledgment**
   ```rust
   // GOOD: Acknowledge events after processing
   match subscription.next_event().await? {
       Some(event) => {
           process_event(&event)?;
           subscription.acknowledge(&event.id).await?;
       },
       None => {}
   }
   
   // BAD: Missing acknowledgment
   match subscription.next_event().await? {
       Some(event) => {
           process_event(&event)?;
           // Missing acknowledgment
       },
       None => {}
   }
   ```

2. **Error Handling**
   ```rust
   // GOOD: Retry logic for event processing
   let mut retries = 3;
   while retries > 0 {
       match subscription.next_event().await {
           Ok(Some(event)) => {
               if let Err(e) = process_event(&event) {
                   println!("Error processing event: {}", e);
                   retries -= 1;
                   continue;
               }
               subscription.acknowledge(&event.id).await?;
               break;
           },
           Ok(None) => break,
           Err(e) => {
               println!("Error getting event: {}", e);
               retries -= 1;
           }
       }
   }
   
   // BAD: No retry logic
   match subscription.next_event().await {
       Ok(Some(event)) => {
           process_event(&event)?;
           subscription.acknowledge(&event.id).await?;
       },
       Ok(None) => {},
       Err(e) => println!("Error: {}", e),
   }
   ```

## Testing Domain Implementations

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;
    
    // Mock domain adapter
    mock! {
        DomainAdapter {}
        
        #[async_trait]
        impl DomainAdapter for DomainAdapter {
            fn domain_id(&self) -> &DomainId;
            fn domain_type(&self) -> DomainType;
            async fn initialize(&self, config: &DomainConfig) -> Result<(), DomainError>;
            async fn execute_operation(&self, operation: DomainOperation) -> Result<DomainOperationResult, DomainError>;
            async fn observe_state(&self, query: StateQuery) -> Result<StateObservation, DomainError>;
            async fn subscribe(&self, filter: EventFilter) -> Result<Box<dyn EventSubscription>, DomainError>;
            async fn check_status(&self) -> Result<DomainStatus, DomainError>;
        }
    }
    
    #[tokio::test]
    async fn test_execute_operation() {
        // Create a mock adapter
        let mut mock_adapter = MockDomainAdapter::new();
        
        // Set up the domain ID
        let domain_id = DomainId::new(
            "ethereum".to_string(),
            "mainnet".to_string(),
        ).unwrap();
        
        mock_adapter
            .expect_domain_id()
            .return_const(domain_id.clone());
        
        // Set up the execute_operation expectation
        mock_adapter
            .expect_execute_operation()
            .with(function(|op: &DomainOperation| {
                op.domain_id() == &domain_id && op.operation_type() == &OperationType::Read
            }))
            .times(1)
            .returning(|_| {
                Ok(DomainOperationResult::new(
                    DomainOperationId::new(),
                    serde_json::json!({
                        "balance": "1000000000000000000"
                    }),
                    OperationStatus::Success,
                ))
            });
        
        // Create an operation
        let operation = DomainOperation::new(
            domain_id,
            OperationType::Read,
            serde_json::json!({
                "address": "0x1234567890abcdef1234567890abcdef12345678"
            }),
        ).unwrap();
        
        // Execute the operation
        let result = mock_adapter.execute_operation(operation).await.unwrap();
        
        // Verify the result
        assert_eq!(result.status(), &OperationStatus::Success);
        assert_eq!(result.data()["balance"], "1000000000000000000");
    }
    
    #[tokio::test]
    async fn test_cross_domain_operation() {
        // Create a mock domain manager
        let mut mock_manager = MockDomainManager::new();
        
        // Set up domain IDs
        let ethereum_id = DomainId::new("ethereum", "mainnet").unwrap();
        let postgres_id = DomainId::new("postgres", "main").unwrap();
        
        // Set up the execute_cross_domain expectation
        mock_manager
            .expect_execute_cross_domain()
            .withf(|ops: &Vec<DomainOperation>, _: &OperationContext| {
                ops.len() == 2 &&
                ops[0].domain_id() == &ethereum_id &&
                ops[1].domain_id() == &postgres_id
            })
            .times(1)
            .returning(|_, _| {
                Ok(vec![
                    DomainOperationResult::new(
                        DomainOperationId::new(),
                        serde_json::json!({
                            "balance": "1000000000000000000"
                        }),
                        OperationStatus::Success,
                    ),
                    DomainOperationResult::new(
                        DomainOperationId::new(),
                        serde_json::json!({
                            "rows_affected": 1
                        }),
                        OperationStatus::Success,
                    ),
                ])
            });
        
        // Create operations
        let eth_operation = DomainOperation::new(
            ethereum_id,
            OperationType::Read,
            serde_json::json!({
                "address": "0x1234567890abcdef1234567890abcdef12345678"
            }),
        ).unwrap();
        
        let postgres_operation = DomainOperation::new(
            postgres_id,
            OperationType::Write,
            serde_json::json!({
                "query": "INSERT INTO balances VALUES ($1, $2)",
                "parameters": ["0x1234567890abcdef1234567890abcdef12345678", "1000000000000000000"]
            }),
        ).unwrap();
        
        // Create context
        let context = OperationContext::new(
            Identity::test(),
        );
        
        // Execute the cross-domain operation
        let results = mock_manager
            .execute_cross_domain(
                vec![eth_operation, postgres_operation],
                &context,
            )
            .await
            .unwrap();
        
        // Verify the results
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].status(), &OperationStatus::Success);
        assert_eq!(results[1].status(), &OperationStatus::Success);
        assert_eq!(results[0].data()["balance"], "1000000000000000000");
        assert_eq!(results[1].data()["rows_affected"], 1);
    }
}
```

## Troubleshooting

### Common Issues and Solutions

| Problem | Possible Cause | Solution |
|---------|---------------|----------|
| Domain operation fails | Invalid parameters | Check parameter types and values |
| | Missing dependencies | Ensure all dependencies are declared and valid |
| | Network error | Retry with backoff, check connection |
| | Authentication error | Verify credentials and permissions |
| Cross-domain operation fails | Incompatible domains | Ensure domains can interact with each other |
| | Ordering issues | Check dependency graph for cycles |
| | Boundary crossing error | Verify boundary crossing protocol implementation |
| Event subscription issues | Connection lost | Implement reconnection logic |
| | Missing acknowledgment | Ensure events are acknowledged after processing |
| | Filter too broad | Refine event filter conditions |
| Domain adapter initialization fails | Invalid configuration | Verify configuration parameters |
| | Missing dependencies | Ensure required services are available |
| | Rate limiting | Implement backoff and retry logic |

### Diagnosing Domain Issues

```rust
/// Diagnose domain connection issues
async fn diagnose_domain_connection(
    domain_id: &DomainId,
    manager: Arc<dyn DomainManager>,
) -> Result<(), DomainError> {
    println!("Diagnosing connection to domain: {}", domain_id);
    
    // Get the adapter
    let adapter = match manager.get_adapter(domain_id).await {
        Ok(adapter) => adapter,
        Err(e) => {
            println!("Error getting adapter: {}", e);
            return Err(e);
        }
    };
    
    // Check domain status
    match adapter.check_status().await {
        Ok(status) => {
            println!("Domain status: {:?}", status);
            
            if status == DomainStatus::Connected {
                println!("Domain is connected");
            } else {
                println!("Domain is not connected: {:?}", status);
            }
        },
        Err(e) => {
            println!("Error checking domain status: {}", e);
            return Err(e);
        }
    }
    
    // Try a simple operation
    let ping_operation = DomainOperation::new(
        domain_id.clone(),
        OperationType::Custom("ping".to_string()),
        serde_json::json!({}),
    )?;
    
    match adapter.execute_operation(ping_operation).await {
        Ok(result) => {
            println!("Ping successful: {:?}", result);
        },
        Err(e) => {
            println!("Ping failed: {}", e);
            return Err(e);
        }
    }
    
    println!("Diagnosis complete");
    
    Ok(())
}
```

## References

- [Domain System Architecture](../../architecture/core/domain-system.md)
- [ADR-018: Domain Adapter Pattern](../../../spec/adr_018_domain_adapter_pattern.md)
- [ADR-023: Three-Layer Effect Architecture with TEL Integration](../../../spec/adr_023_domain_adapter_effect_handler_unification.md)
- [ADR-031: Domain-Specific Operations](../../../spec/adr_031_domain_specific_operations.md)
- [System Specification: Domain System](../../../spec/spec.md#domain-system) 