# causality-engine Library Reference

*This document provides reference information for the `causality-engine` crate.*

*Last updated: 2023-08-20*

## Overview

The `causality-engine` crate implements the execution engine for Causality, providing the runtime environment for executing operations and managing system resources. It integrates all of the core components into a coherent execution environment.

## Key Modules

### causality_engine::engine

Core engine implementation for executing operations and managing resources.

```rust
use causality_engine::engine::{
    CausalityEngine,
    EngineConfig,
    ExecutionContext,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `CausalityEngine` | Main entry point for the Causality system |
| `EngineConfig` | Configuration for the engine |
| `ExecutionContext` | Context for operation execution |

### causality_engine::vm

Virtual machine for executing operations.

```rust
use causality_engine::vm::{
    VirtualMachine,
    VMState,
    Instruction,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `VirtualMachine` | VM for secure operation execution |
| `VMState` | State of the virtual machine |
| `Instruction` | VM instruction |

### causality_engine::facts

Fact management and verification.

```rust
use causality_engine::facts::{
    FactStore,
    FactVerifier,
    FactChain,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `FactStore` | Storage for verified facts |
| `FactVerifier` | Verifies facts |
| `FactChain` | Chain of related facts |

### causality_engine::transaction

Transaction management and lifecycle.

```rust
use causality_engine::transaction::{
    Transaction,
    TransactionManager,
    TransactionLog,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `Transaction` | Atomic unit of operations |
| `TransactionManager` | Manages transactions |
| `TransactionLog` | Log of executed transactions |

### causality_engine::domains

Domain integration and management.

```rust
use causality_engine::domains::{
    DomainRegistry,
    DomainAdapter,
    DomainEvent,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `DomainRegistry` | Registry of available domains |
| `DomainAdapter` | Adapter for domain integration |
| `DomainEvent` | Event from a domain |

## Engine Architecture

The engine serves as the integration point for all Causality components. It brings together resources, agents, effects, and domains into a unified execution environment.

### System Integration

The engine integrates the following components:

- **Resource Manager**: Manages all resources in the system
- **Agent Manager**: Manages agent resources and authentication
- **Effect System**: Handles effectful computations
- **Capability Verifier**: Verifies capabilities for operations
- **Domain Registry**: Integrates external domains

### Engine Initialization

```rust
// Create an engine with default configuration
let mut engine = CausalityEngine::new()?;

// Create an engine with custom configuration
let config = EngineConfig::builder()
    .data_directory("/path/to/data")
    .max_concurrent_operations(16)
    .enable_transaction_logs(true)
    .build()?;
    
let mut engine = CausalityEngine::with_config(config)?;
```

### Resource Registration

Resources can be registered with the engine:

```rust
// Register a resource type
engine.register_resource_type::<Database>("database")?;

// Create a resource instance
engine.create_resource("main_db", Database::new())?;
```

### Agent Registration

Agents can be registered with the engine:

```rust
// Register a user agent
let user_agent = AgentBuilder::new()
    .user("alice")
    .with_capabilities(user_capabilities)
    .build()?;
    
engine.register_agent(user_agent)?;
```

### Operation Execution

Operations can be executed through the engine:

```rust
// Create an operation
let operation = OperationBuilder::new()
    .target_resource("main_db")
    .action("query")
    .parameters(params)
    .build()?;

// Execute the operation
let result = engine.execute_operation(
    "alice",  // Agent ID
    operation,
    execution_context
)?;
```

## Virtual Machine

The engine uses a virtual machine to execute operations in a controlled environment:

```rust
// Get the VM from the engine
let vm = engine.get_vm()?;

// Prepare an operation for execution
let prepared_operation = vm.prepare(operation)?;

// Execute the operation
let result = vm.execute(prepared_operation, execution_context)?;
```

## Transaction Management

Operations can be grouped into transactions:

```rust
// Begin a transaction
let transaction = engine.begin_transaction()?;

// Add operations to the transaction
transaction.add_operation(operation1)?;
transaction.add_operation(operation2)?;

// Commit the transaction
engine.commit_transaction(transaction)?;
```

## Fact Generation and Verification

The engine generates and verifies facts:

```rust
// Generate a fact from an operation
let fact = engine.generate_fact(operation_result)?;

// Verify a fact
let is_valid = engine.verify_fact(fact)?;

// Add a fact to the fact store
engine.add_fact(fact)?;
```

## Domain Integration

The engine integrates with external domains:

```rust
// Register a domain adapter
engine.register_domain_adapter(
    ethereum_adapter
)?;

// Get events from a domain
let events = engine.get_domain_events("ethereum")?;

// Process domain events
engine.process_domain_events(events)?;
```

## Usage Example

```rust
use causality_engine::{
    engine::{CausalityEngine, EngineConfig},
    vm::{VirtualMachine},
};
use causality_agent::{
    agent::{AgentBuilder},
    operation::{OperationBuilder},
};

// Create an engine
let mut engine = CausalityEngine::new()?;

// Register resource types
engine.register_resource_type::<Database>("database")?;
engine.register_resource_type::<FileSystem>("filesystem")?;

// Register domain adapters
engine.register_domain_adapter(
    EthereumAdapter::new(ethereum_config)?
)?;

// Create resources
engine.create_resource("main_db", Database::new("customer_data")?)?;
engine.create_resource("file_system", FileSystem::new()?)?;

// Register a user agent
let user_agent = AgentBuilder::new()
    .user("alice")
    .with_capability(read_capability)
    .with_capability(write_capability)
    .build()?;
    
engine.register_agent(user_agent)?;

// Authenticate an agent
let session = engine.authenticate(
    "alice", 
    AuthMethod::Password("password123".to_string())
)?;

// Create an operation
let operation = OperationBuilder::new()
    .target_resource("main_db")
    .action("query")
    .parameters(json!({ "query": "SELECT * FROM customers" }))
    .build()?;

// Execute the operation
let result = engine.execute_operation(
    session,
    operation,
    execution_context
)?;

// Extract facts from the result
let facts = engine.extract_facts(result)?;

// Commit facts to the fact store
engine.commit_facts(facts)?;
```

## References

- [ADR-032: Role-Based Resource System](../../../spec/adr_032-role-based-resource-system.md)
- [System Contract](../../../spec/system_contract.md)
- [Engine Architecture](../../architecture/engine/) 