# Domain System Unification in Causality

This document outlines the unified Domain System in the Causality Project, detailing the components, architecture, and integration mechanisms that enable cross-domain operations.

## Components

### Domain Identifier Unification

Standardized `DomainId` format provides a consistent way of identifying domains across the system.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DomainId(pub String);
```

### Domain Status and Type Unification

Standardized enums for domain status and type ensure consistency:

```rust
pub enum DomainType {
    EVM,
    CosmWasm,
    SOL,
    TEL,
    Unknown,
}

pub enum DomainStatus {
    Active,
    Inactive,
    Maintenance,
    Error,
    Initializing,
    Unknown,
}
```

### Domain Adapter Interface

The `DomainAdapter` trait provides a unified interface for interacting with external domains:

```rust
#[async_trait]
pub trait DomainAdapter: Send + Sync + std::fmt::Debug {
    fn domain_id(&self) -> &DomainId;
    async fn domain_info(&self) -> Result<DomainInfo>;
    async fn current_height(&self) -> Result<BlockHeight>;
    async fn submit_transaction(&self, tx: Transaction) -> Result<TransactionId>;
    // Additional methods...
}
```

### Domain Registry

The Domain Registry manages domain adapters with methods for registration, unregistration, and retrieval:

```rust
pub struct DomainRegistry {
    adapters: RwLock<HashMap<DomainId, Arc<dyn DomainAdapter>>>,
    default_strategy: RwLock<Box<dyn DomainSelectionStrategy>>,
    factories: RwLock<Vec<Arc<dyn DomainAdapterFactory>>>,
}
```

### Domain Selection Strategies

Various strategies are implemented for selecting domains based on preferences and capabilities:

- `PreferredDomainStrategy`: Selects domains from a preferred list
- `LatencyBasedStrategy`: Selects domains based on latency
- `CostBasedStrategy`: Selects domains based on operation cost
- `CompositeStrategy`: Combines multiple strategies with weights

### Domain Time Map

A time synchronization mechanism for tracking and relating time across domains:

```rust
pub struct DomainTimeMap {
    entries: RwLock<Vec<TimeMapEntry>>,
    domains: RwLock<HashSet<DomainId>>,
}
```

### Domain Capability System

A unified capability system that integrates with the resource capability system:

```rust
pub enum DomainCapability {
    // Transaction capabilities
    SendTransaction,
    SignTransaction,
    BatchTransactions,
    
    // Smart contract capabilities
    DeployContract,
    ExecuteContract,
    QueryContract,
    
    // State capabilities
    ReadState,
    WriteState,
    
    // Cryptographic capabilities
    VerifySignature,
    GenerateProof,
    VerifyProof,
    
    // ZK capabilities
    ZkProve,
    ZkVerify,
    
    // Additional capabilities...
    Custom(String)
}

pub struct DomainCapabilityManager {
    capability_system: Arc<dyn CapabilitySystem>,
    default_capabilities: HashMap<DomainType, HashSet<DomainCapability>>,
    domain_capabilities: HashMap<DomainId, HashSet<DomainCapability>>,
}
```

### Domain Adapter Implementations

Comprehensive adapter implementations for various domains:

- `CosmWasmAdapter`: For Cosmos-based chains with WebAssembly contract support
- `EVMAdapter`: For Ethereum-compatible blockchains
- Both adapters include integrated ZK operations where applicable

### Domain Resource Integration

A unified resource integration layer providing operations across domains:

```rust
pub struct CrossDomainResourceManager {
    domain_registry: Arc<DomainRegistry>,
    resource_adapters: HashMap<DomainType, Box<dyn DomainResourceAdapter>>,
}

pub trait DomainResourceAdapter: Send + Sync {
    fn store_resource(&self, resource: ResourceHandle, domain_id: &DomainId) -> Result<ResourceStoreReceipt>;
    fn retrieve_resource(&self, resource_id: &ResourceId, domain_id: &DomainId) -> Result<ResourceHandle>;
    // Additional methods...
}
```

## Architecture Diagram

```
                     +-------------------+
                     |  DomainRegistry   |
                     +-------------------+
                     | - register_adapter |
                     | - get_adapter      |
                     | - select_domain    |
                     +--------+----------+
                              |
                              | manages
                              v
+----------------+   +-------------------+   +----------------+
| CosmWasmAdapter|<--| Domain Adapter    |-->| EVMAdapter     |
+----------------+   | Interface         |   +----------------+
| - domain_info  |   +-------------------+   | - domain_info  |
| - submit_tx    |                           | - submit_tx    |
+-------+--------+                           +--------+-------+
        |                                             |
        | synchronized via                            |
        v                                             v
+----------------+                           +----------------+
| DomainTimeMap  |<------------------------->| Capability     |
+----------------+                           | System         |
                                             +----------------+
```

## Usage Examples

### Registering a Domain Adapter

```rust
let registry = DomainRegistry::new(Box::new(PreferredDomainStrategy::new(vec![])));
let cosm_adapter = CosmWasmAdapter::new(/* parameters */);
registry.register_adapter(Arc::new(cosm_adapter)).unwrap();
```

### Using Domain Selection Strategies

```rust
// Create a set of required capabilities
let mut required_capabilities = HashSet::new();
required_capabilities.insert("execute_contract".to_string());

// Create preference map
let mut preferences = HashMap::new();
preferences.insert("cost".to_string(), "low".to_string());

// Select a domain with the specified capabilities and preferences
let domain_id = registry.select_domain(&required_capabilities, &preferences).await.unwrap();
```

### Domain Capabilities Integration

```rust
// Create capability manager
let capability_manager = DomainCapabilityManager::new(capability_system);

// Register domain adapter capabilities
capability_manager.register_domain_adapter(&adapter)?;

// Check for specific capability
if capability_manager.domain_has_capability(&domain_id, &DomainCapability::ZkProve) {
    // Use ZK prove capability
}

// Create a capability for domain operations
let cap_id = capability_manager.create_domain_capability(
    &domain_id,
    &resource_id,
    &owner_address,
    &issuer_address,
    &[DomainCapability::ExecuteContract, DomainCapability::QueryContract],
    true // delegatable
).await?;
```

### Resource Register Integration

```rust
let resource_manager = CrossDomainResourceManager::new(Arc::clone(&registry));

// Store a resource in a specific domain
let receipt = resource_manager.execute_operation(
    CrossDomainResourceOperation::Store { 
        resource, 
        domain_id: target_domain.clone() 
    }
).await?;

// Transfer a resource between domains
let transfer_receipt = resource_manager.execute_operation(
    CrossDomainResourceOperation::Transfer { 
        resource_id, 
        from_domain: source_domain.clone(),
        to_domain: target_domain.clone() 
    }
).await?;
```

## Future Work

1. **Domain Capability Extensions**: Further expansion of domain-specific capabilities.
2. **Fact Observer Integration**: Integration with the effect system for cross-domain fact observation.
3. **Additional Domain Adapters**: Support for more blockchain types (Polkadot, Near, etc.).
4. **Unified Authentication**: Cross-domain authentication mechanisms.
5. **Domain Discovery**: Automatic discovery of available domains.

This unified domain system provides a robust and extensible framework for interacting with multiple blockchain domains, enabling cross-domain operations while abstracting away domain-specific details. 