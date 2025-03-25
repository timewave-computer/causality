<!-- Management of facts -->
<!-- Original file: docs/src/fact_management.md -->

# Fact Management System Implementation

This document provides an overview of the Fact Management System implementation in the Causality project.

## Overview

The Fact Management System is a core component of Causality that treats facts as first-class causal entities. Facts represent observations from external domains, and they provide a foundation for effects and operations in the system. This implementation follows the design described in ADR 007.

## Key Components

### Core Fact Types

The system defines several types of facts:

- **BalanceFact**: Facts about token balances
- **TransactionFact**: Facts about transactions on external domains
- **OracleFact**: Facts provided by external oracles
- **BlockFact**: Facts about blocks on chains
- **TimeFact**: Facts about time observations
- **RegisterFact**: Facts about register operations (creation, update, transfer, merge, split)
- **ZKProofFact**: Facts about ZK proof verifications

### Fact Snapshots and Dependencies

Facts are organized into snapshots that track dependencies between facts and effects:

- **FactSnapshot**: A point-in-time collection of facts that an effect depends on
- **FactDependency**: A relationship between an effect and a fact it depends on
- **FactDependencyValidator**: A component that validates fact dependencies

### Fact Observation

Specialized observers handle different types of facts:

- **RegisterFactObserver**: Observes register operations
- **ZKProofFactObserver**: Observes ZK proof verifications

### Resource Manager Integration

The ResourceManager has been extended to emit register facts for operations on registers:

- **ResourceManager**: Manages resource allocation and emits register facts
- **ResourceGuard**: Automatically releases resources when dropped and tracks register IDs

The ResourceManager includes methods to:
- Create registers and emit RegisterCreation facts
- Update registers and emit RegisterUpdate facts
- Transfer registers and emit RegisterTransfer facts
- Merge registers and emit RegisterMerge facts
- Split registers and emit RegisterSplit facts

### Fact Replay and Simulation

Tools for testing and analyzing fact-based systems:

- **FactReplayEngine**: Replays facts in chronological order
- **FactSimulator**: Simulates fact observations for testing

### Bridging Between Old and New Systems

To facilitate a gradual migration path, we've implemented bridging capabilities:

- **Fact Bridge Module**: Provides conversion functions between old and new fact implementations
- **Legacy Support**: Allows existing systems to continue using `ObservedFact` while new systems use `FactType`
- **Domain Adapter Enhancement**: Added `observe_fact_type` method to domain adapters for gradual migration

The bridge provides functions for bidirectional conversion:
- `fact_type_to_observed_fact`: Convert the new FactType to the legacy ObservedFact
- `observed_fact_to_fact_type`: Convert the legacy ObservedFact to the new FactType
- `verified_fact_to_fact_type`: Convert VerifiedFact to FactType with metadata
- `fact_type_to_verified_fact`: Create VerifiedFact from FactType

## Usage Examples

### Creating and Logging Facts

```rust
// Create a fact logger
let storage = Arc::new(Mutex::new(MemoryLogStorage::new()));
let domain_id = DomainId::new("ethereum");
let logger = Arc::new(FactLogger::new(storage, domain_id));

// Create and log a register fact
let register_id = RegisterId::new("register-1");
let initial_data = vec![1, 2, 3, 4];

let fact = RegisterFact::RegisterCreation {
    register_id,
    initial_data,
};

let fact_id = logger.log_register_fact(fact, metadata)?;
```

### Using the RegisterFactObserver

```rust
// Create a register fact observer
let observer = RegisterFactObserver::new(fact_logger.clone());

// Observe a register creation
let register_id = RegisterId::new("register-1");
let initial_data = vec![1, 2, 3, 4];
let trace_id = TraceId::new();

observer.observe_register_creation(
    &register_id,
    &initial_data,
    trace_id
)?;
```

### Using ResourceManager with Register Facts

```rust
// Create a resource manager with register fact observation
let allocator = Arc::new(StaticAllocator::new(
    1024 * 1024, // 1MB memory
    1000,        // 1 second CPU time
    100,         // 100 I/O operations
    10,          // 10 effects
));

let fact_logger = Arc::new(FactLogger::new(storage, domain_id.clone()));
let manager = ResourceManager::with_register_observation(
    allocator,
    domain_id,
    fact_logger,
);

// Set the current trace ID
manager.set_trace(TraceId::from_str("tx-1"));

// Create a register (automatically emits a RegisterCreation fact)
let register_id = ResourceId::new("register-1");
let initial_data = vec![1, 2, 3, 4];
let request = ResourceRequest::new(1024, 100, 10, 1);
let guard = manager.create_register(register_id.clone(), &initial_data, request)?;

// Update a register (automatically emits a RegisterUpdate fact)
let new_data = vec![5, 6, 7, 8];
manager.update_register(register_id.clone(), &new_data, "v1")?;

// Transfer a register (automatically emits a RegisterTransfer fact)
manager.transfer_register(
    register_id.clone(),
    "source-domain",
    "target-domain",
)?;
```

### Using the Fact Bridge

```rust
// Convert from new FactType to legacy ObservedFact
let fact_type = FactType::RegisterFact(RegisterFact::RegisterCreation {
    register_id: register_id.clone(),
    initial_data: vec![1, 2, 3, 4],
});

let observed_fact = fact_type_to_observed_fact(
    &fact_type,
    domain_id.clone(),
    BlockHeight::new(100),
    BlockHash::new("block-hash"),
    Timestamp::new(1000),
    None,
)?;

// Convert from legacy ObservedFact to new FactType
let round_trip_fact_type = observed_fact_to_fact_type(&observed_fact)?;

// Working with VerifiedFact
let verified_fact = fact_type_to_verified_fact(
    &fact_type,
    domain_id.clone(),
    BlockHeight::new(100),
    BlockHash::new("block-hash"),
    Timestamp::new(1000),
    true,
    Some("test-method".to_string()),
    0.95,
    None,
)?;

// Convert back to FactType with verification metadata
let (fact_type_from_verified, metadata) = verified_fact_to_fact_type(&verified_fact)?;
```

### Creating Effects with Fact Dependencies

```rust
// Create a deposit effect with fact dependencies
let effect = deposit_with_facts(
    resource_id,
    domain_id,
    amount,
    snapshot,
);

// Add additional fact dependencies
let mut effect = effect.as_mut();
effect.with_fact_dependency(
    fact_id,
    domain_id,
    FactDependencyType::Required,
);

// Validate fact dependencies
effect.validate_fact_dependencies()?;
```

### Using Domain Adapters with New Fact Types

```rust
// Create domain adapter
let adapter = create_domain_adapter(DomainType::EVM, config)?;

// Using legacy API
let query = FactQuery {
    domain_id: domain_id.clone(),
    fact_type: "balance".to_string(),
    parameters: HashMap::from([
        ("address".to_string(), "0x...".to_string()),
        ("token".to_string(), "ETH".to_string()),
    ]),
    block_height: None,
    block_hash: None,
    timestamp: None,
};

let observed_fact = adapter.observe_fact(query.clone()).await?;

// Using new API that returns FactType
let fact_type = adapter.observe_fact_type(query).await?;

// Both approaches work, allowing for gradual migration
```

### Replaying Facts

```rust
// Create a fact replay engine
let config = FactReplayConfig::default();
let engine = FactReplayEngine::new(storage, config);

// Add a callback for fact replay events
engine.add_callback(Box::new(|fact| {
    println!("Replaying fact: {}", fact.fact_type);
    Ok(())
}));

// Start the replay
engine.start()?;

// Get register state
let state = engine.get_register_state(&register_id);

// Create a snapshot of the current state
let snapshot = engine.create_snapshot("replay-observer");
```

### Simulating Facts

```rust
// Create a fact simulator
let config = FactSimulatorConfig::default();
let mut simulator = FactSimulator::new(logger, config);

// Simulate a register creation
let fact_id = simulator.simulate_register_creation(
    TraceId::from_str("tx-1"),
    register_id,
    vec![1, 2, 3, 4],
)?;

// Simulate a transaction fact
let tx_fact_id = simulator.simulate_transaction_fact(
    TraceId::from_str("tx-2"),
    "tx-hash-1",
)?;

// Create a snapshot with simulated facts
let snapshot = simulator.create_snapshot(&[fact_id, tx_fact_id]);
```

## Testing Utilities

The system includes a set of utilities for testing fact-related components:

- **create_test_fact_logger**: Creates a test fact logger
- **create_test_fact_simulator**: Creates a test fact simulator
- **create_test_fact_replay_engine**: Creates a test fact replay engine
- **create_test_register_facts**: Creates test register facts
- **create_test_zkproof_facts**: Creates test ZK proof facts
- **create_test_fact_snapshot**: Creates a test fact snapshot
- **create_test_transaction_chain**: Creates test transaction facts with dependencies
- **validate_fact_chain**: Validates fact dependencies

## Integration with Other Components

The fact management system integrates with various other Causality components:

- **Effect System**: Effects include fact dependencies, ensuring proper causality
- **Resource Manager**: Resource operations automatically generate register facts
- **Domain Adapters**: Domain adapters observe external facts and translate them
- **Log System**: Facts are logged in the unified log system

## Migration Strategy

The fact management system is designed to allow for gradual migration from the old system to the new:

1. **Bridge Module**: Enables bidirectional conversion between old and new fact types
2. **Enhanced Domain Adapters**: Support both old-style observation and new FactType-based observation
3. **Parallel Support**: Both systems can run side-by-side during migration
4. **Common Interfaces**: Key interfaces like DomainAdapter have been standardized for both systems

## Future Work

Planned improvements to the fact management system:

1. Cleaning up old fact-related code and standardizing interfaces
2. Enhancing domain adapters to use the new fact system
3. Adding comprehensive tests for all fact-related components 
4. Extending automatic fact emission to other components beyond ResourceManager
5. Removing deprecated fact implementations once migration is complete 