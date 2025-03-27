# Domain Model

*This document is derived from [ADR-018](../../../spec/adr_018_domain_adapter.md), [ADR-031](../../../spec/adr_031_domain_adapter_as_effect.md), and the [System Specification](../../../spec/spec.md).*

*Last updated: 2023-09-05*

## Overview

The Domain System in Causality provides a unified architecture for interacting with external blockchain networks and other execution environments. It allows the core system to operate seamlessly across multiple domains while handling the complexity and diversity of different blockchain platforms.

## Core Concepts

### Domain Definition

A Domain in Causality represents an external execution environment with its own state model, consensus rules, and transaction format. Examples include:

- Ethereum and EVM-compatible blockchains
- CosmWasm-based blockchains
- Local execution environments
- Zero-Knowledge Virtual Machines

### Domain Components

The Domain System includes several key components:

1. **Domain Identifiers**: Standardized identifiers for referring to domains
2. **Domain Adapters**: Interface implementations for interacting with specific domains
3. **Domain Registry**: Central registry for managing available domain adapters
4. **Domain Capabilities**: Permissions for domain-specific operations
5. **Domain Resources**: Domain-specific resources and their management
6. **Cross-Domain Operations**: Mechanisms for operations spanning multiple domains

## Domain Interface

Each domain exposes a standardized interface through its domain adapter:

```rust
#[async_trait]
pub trait DomainAdapter: Send + Sync + std::fmt::Debug {
    // Core domain information
    fn domain_id(&self) -> &DomainId;
    async fn domain_info(&self) -> Result<DomainInfo, DomainError>;
    async fn current_height(&self) -> Result<BlockHeight, DomainError>;
    
    // Transaction handling
    async fn submit_transaction(&self, tx: Transaction) -> Result<TransactionId, DomainError>;
    async fn get_transaction_status(&self, tx_id: &TransactionId) -> Result<TransactionStatus, DomainError>;
    
    // State querying
    async fn query_state(&self, query: &FactQuery) -> Result<FactData, DomainError>;
    
    // Resource management
    async fn store_resource(&self, resource: &Resource) -> Result<ResourceStoreReceipt, DomainError>;
    async fn get_resource(&self, id: &ResourceId) -> Result<Resource, DomainError>;
    
    // Domain-specific capabilities
    fn capabilities(&self) -> HashSet<DomainCapability>;
}
```

## Domain Registry

The Domain Registry manages available domain adapters and provides methods for domain selection:

```rust
pub struct DomainRegistry {
    adapters: RwLock<HashMap<DomainId, Arc<dyn DomainAdapter>>>,
    default_strategy: RwLock<Box<dyn DomainSelectionStrategy>>,
    factories: RwLock<Vec<Arc<dyn DomainAdapterFactory>>>,
}

impl DomainRegistry {
    // Register a domain adapter
    pub fn register_adapter(&self, adapter: Arc<dyn DomainAdapter>) -> Result<(), RegistryError>;
    
    // Get a specific domain adapter
    pub fn get_adapter(&self, domain_id: &DomainId) -> Result<Arc<dyn DomainAdapter>, RegistryError>;
    
    // Select an appropriate domain based on requirements and preferences
    pub async fn select_domain(
        &self,
        required_capabilities: &HashSet<DomainCapability>,
        preferences: &HashMap<String, String>
    ) -> Result<DomainId, RegistryError>;
}
```

## Domain Adapter Implementation

Domain adapters are implemented for each supported blockchain or execution environment:

### EVM Domain Adapter

```rust
pub struct EVMAdapter {
    domain_id: DomainId,
    provider: Arc<dyn EthersProvider>,
    config: EVMConfig,
    contracts: HashMap<ContractType, Address>,
}

impl EVMAdapter {
    // Create a new EVM domain adapter
    pub fn new(domain_id: DomainId, provider: Arc<dyn EthersProvider>, config: EVMConfig) -> Self;
    
    // Deploy a contract to the EVM domain
    pub async fn deploy_contract(&self, bytecode: Vec<u8>, constructor_args: Vec<Value>) -> Result<Address, DomainError>;
    
    // Call a contract function on the EVM domain
    pub async fn call_contract(&self, address: Address, function: &str, args: Vec<Value>) -> Result<Value, DomainError>;
}
```

### CosmWasm Domain Adapter

```rust
pub struct CosmWasmAdapter {
    domain_id: DomainId,
    client: Arc<CosmWasmClient>,
    config: CosmWasmConfig,
    contracts: HashMap<ContractType, String>,
}

impl CosmWasmAdapter {
    // Create a new CosmWasm domain adapter
    pub fn new(domain_id: DomainId, client: Arc<CosmWasmClient>, config: CosmWasmConfig) -> Self;
    
    // Upload a contract to the CosmWasm domain
    pub async fn upload_contract(&self, wasm_bytes: Vec<u8>) -> Result<u64, DomainError>;
    
    // Instantiate a contract on the CosmWasm domain
    pub async fn instantiate_contract(&self, code_id: u64, init_msg: Value) -> Result<String, DomainError>;
    
    // Execute a contract function on the CosmWasm domain
    pub async fn execute_contract(&self, address: String, msg: Value) -> Result<Value, DomainError>;
}
```

## Integration with Effect System

The Domain System is fully integrated with the Effect System through bidirectional integration:

1. Domain operations can be executed as effects
2. Effects can leverage domain adapters for implementation
3. Cross-domain operations are implemented as composed effects

### Domain Effect Examples

```rust
// Effect for querying domain state
pub struct DomainQueryEffect<R> {
    domain_id: DomainId,
    query: FactQuery,
    continuation: Box<dyn Continuation<FactData, R>>,
}

// Effect for submitting a transaction to a domain
pub struct DomainTransactionEffect<R> {
    domain_id: DomainId,
    transaction: Transaction,
    wait_for_confirmation: bool,
    continuation: Box<dyn Continuation<TransactionResult, R>>,
}
```

## Domain Capabilities

Domain capabilities define what operations can be performed on a domain:

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
    
    // Custom capabilities
    Custom(String),
}
```

## Cross-Domain Operations

Cross-domain operations allow for seamless interaction between different blockchain networks:

### Cross-Domain Resource Transfer

```rust
pub struct CrossDomainTransferEffect<R> {
    source_domain_id: DomainId,
    target_domain_id: DomainId,
    resource_id: ResourceId,
    target_address: String,
    continuation: Box<dyn Continuation<TransferResult, R>>,
}
```

### Cross-Domain Proof Verification

```rust
pub struct CrossDomainProofEffect<R> {
    source_domain_id: DomainId,
    target_domain_id: DomainId,
    proof: Proof,
    public_inputs: Vec<Value>,
    continuation: Box<dyn Continuation<VerificationResult, R>>,
}
```

## Domain Selection Strategies

Various strategies can be used for selecting domains based on requirements:

- **PreferredDomainStrategy**: Selects domains from a preferred list
- **LatencyBasedStrategy**: Selects domains based on latency
- **CostBasedStrategy**: Selects domains based on operation cost
- **CompositeStrategy**: Combines multiple strategies with weights

## Architecture Diagram

```
                     ┌───────────────────┐
                     │   Effect System   │
                     └──────────┬────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────┐
│                   Domain System                     │
│                                                     │
│  ┌─────────────┐        ┌────────────────┐          │
│  │             │        │                │          │
│  │   Domain    │◄──────►│Domain Registry │          │
│  │  Adapters   │        │                │          │
│  │             │        └────────────────┘          │
│  └─────────────┘                                    │
│                                                     │
│  ┌─────────────┐        ┌────────────────┐          │
│  │   Domain    │        │  Cross-Domain  │          │
│  │ Capabilities│◄──────►│  Coordination  │          │
│  │             │        │                │          │
│  └─────────────┘        └────────────────┘          │
│                                                     │
└─────────────────────────────────────────────────────┘
           │                      │
           ▼                      ▼
┌─────────────────┐      ┌────────────────┐
│  EVM Chains     │      │ CosmWasm Chains│
└─────────────────┘      └────────────────┘
```

## Usage Examples

### Registering a Domain Adapter

```rust
// Create a domain registry
let registry = DomainRegistry::new(Box::new(PreferredDomainStrategy::new(vec![])));

// Create and register an EVM adapter
let evm_config = EVMConfig::new(/* parameters */);
let evm_provider = Arc::new(JsonRpcProvider::new(endpoint_url));
let evm_adapter = EVMAdapter::new(DomainId::new("ethereum"), evm_provider, evm_config);
registry.register_adapter(Arc::new(evm_adapter)).unwrap();

// Create and register a CosmWasm adapter
let cosmwasm_config = CosmWasmConfig::new(/* parameters */);
let cosmwasm_client = Arc::new(CosmWasmClient::new(rpc_url));
let cosmwasm_adapter = CosmWasmAdapter::new(DomainId::new("osmosis"), cosmwasm_client, cosmwasm_config);
registry.register_adapter(Arc::new(cosmwasm_adapter)).unwrap();
```

### Using Domain Selection Strategies

```rust
// Create required capabilities
let mut required_capabilities = HashSet::new();
required_capabilities.insert(DomainCapability::ExecuteContract);

// Create preferences
let mut preferences = HashMap::new();
preferences.insert("cost".to_string(), "low".to_string());

// Select a suitable domain
let domain_id = registry.select_domain(&required_capabilities, &preferences).await.unwrap();
```

### Executing Domain-Specific Operations

```rust
// Get a domain adapter
let adapter = registry.get_adapter(&domain_id).unwrap();

// Submit a transaction
let tx = Transaction::new(/* parameters */);
let tx_id = adapter.submit_transaction(tx).await.unwrap();

// Query domain state
let query = FactQuery::new(/* parameters */);
let fact_data = adapter.query_state(&query).await.unwrap();
```

### Cross-Domain Resource Transfer

```rust
// Create a cross-domain transfer effect
let effect = cross_domain_transfer(
    source_domain_id,
    target_domain_id,
    resource_id,
    target_address,
    |result| {
        println!("Transfer complete with receipt: {:?}", result);
        result
    }
);

// Execute the effect
let result = effect_engine.execute(effect, context).await.unwrap();
```

## Supported Domains

The system currently supports the following domains:

1. **Ethereum**: Main Ethereum network
2. **Ethereum Testnets**: Sepolia, Goerli
3. **EVM-Compatible Chains**: Polygon, Avalanche, Binance Smart Chain, Optimism, Arbitrum
4. **CosmWasm Chains**: Osmosis, Juno, Terra
5. **Local Domains**: In-memory domains for testing and development
6. **ZK Virtual Machines**: Integration with various ZK VM implementations

## Further Reading

- [Domain Adapters in the Three-Layer Effect Architecture](../core/three-layer-effect-architecture.md)
- [Cross-Domain Integration Guide](../../guides/implementation/domain-system.md)
- [Effect System](../core/effect-system.md)
