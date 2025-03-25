<!-- Domain integration guide -->
<!-- Original file: docs/src/domain_integration.md -->

# Domain Integration Patterns

This document provides a comprehensive guide to integrating with blockchain domains in the Causality system. It covers the architecture, implementation patterns, and best practices for domain adapters, with a clear distinction between on-chain and off-chain components.

## Contents

1. [Domain Architecture Overview](#domain-architecture-overview)
2. [Domain Adapter Implementation](#domain-adapter-implementation)
3. [On-Chain vs Off-Chain Components](#on-chain-vs-off-chain-components)
4. [Time Map Integration](#time-map-integration)
5. [Fact Observation Pattern](#fact-observation-pattern)
6. [Transaction Submission](#transaction-submission)
7. [Domain Selection Strategies](#domain-selection-strategies)
8. [Effect System Integration](#effect-system-integration)
9. [Cross-Domain Operations](#cross-domain-operations)
10. [Domain Capability System](#domain-capability-system)
11. [Testing Domain Integrations](#testing-domain-integrations)
12. [Domain-Specific Effects](#domain-specific-effects)

## Domain Architecture Overview

The Causality domain system creates a unified interface for interacting with different blockchain systems. The architecture consists of:

1. **Domain Adapters**: Implementation of the `DomainAdapter` trait for specific blockchain protocols
2. **Domain Registry**: Central repository for registering and retrieving domain adapters
3. **Domain Selection**: Strategies for selecting the optimal domain for operations
4. **Time Map**: Synchronization system for time-related operations across domains
5. **Effect System**: Framework for executing domain-specific actions with proper authorization
6. **Domain Effects**: Domain-specific effect implementations for EVM, CosmWasm, and ZK/Succinct blockchains

```
┌─────────────────────────────────────────────────────┐
│                  Application Layer                  │
└───────────────────────────┬─────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────┐
│                 Domain Selection Layer              │
└───────────────────────────┬─────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────┐
│                  Effect System Layer                │
├─────────────┬─────────────┬─────────────┬───────────┤
│ EVM Effects │  CosmWasm   │    ZK       │  Core     │
│             │   Effects   │  Effects    │  Effects  │
└─────────────┴─────────────┴─────────────┴───────────┘
                            │
┌───────────────────────────┴─────────────────────────┐
│                 Domain Adapter Layer                │
├─────────────┬─────────────┬─────────────┬───────────┤
│ EVM Adapter │ CosmWasm    │ Succinct    │ Other     │
│             │ Adapter     │ Adapter     │ Adapters  │
└─────────────┴─────────────┴─────────────┴───────────┘
              │             │             │
              ▼             ▼             ▼
       ┌─────────────┬─────────────┬─────────────┐
       │   Ethereum  │   Cosmos    │   ZK VM     │
       │  Blockchain │  Blockchain │  Blockchain │
       └─────────────┴─────────────┴─────────────┘
```

## Domain Adapter Implementation

A domain adapter is the interface between Causality and a specific blockchain system. It must implement the `DomainAdapter` trait:

```rust
pub trait DomainAdapter: Send + Sync + std::fmt::Debug {
    // Identity methods
    fn domain_id(&self) -> &DomainId;
    async fn domain_info(&self) -> Result<DomainInfo>;
    
    // Block and time methods
    async fn current_height(&self) -> Result<BlockHeight>;
    async fn current_hash(&self) -> Result<BlockHash>;
    async fn current_time(&self) -> Result<Timestamp>;
    async fn time_map_entry(&self, height: BlockHeight) -> Result<TimeMapEntry>;
    
    // State observation and transaction methods
    async fn observe_fact(&self, query: &FactQuery) -> FactResult;
    async fn submit_transaction(&self, tx: Transaction) -> Result<TransactionId>;
    async fn transaction_receipt(&self, tx_id: &TransactionId) -> Result<TransactionReceipt>;
    async fn transaction_confirmed(&self, tx_id: &TransactionId) -> Result<bool>;
    async fn wait_for_confirmation(&self, tx_id: &TransactionId, max_wait_ms: Option<u64>) -> Result<TransactionReceipt>;
    
    // Capability methods
    fn capabilities(&self) -> Vec<String>;
    fn has_capability(&self, capability: &str) -> bool;
    async fn estimate_fee(&self, tx: &Transaction) -> Result<HashMap<String, u64>>;
}
```

### Creating a Domain Adapter

To implement a domain adapter:

1. Create a new struct for your domain adapter
2. Implement the `DomainAdapter` trait
3. Register the adapter with the domain registry

```rust
// 1. Define your adapter
pub struct MyDomainAdapter {
    domain_id: DomainId,
    client: Client,
    config: DomainConfig,
    fact_cache: Arc<Mutex<HashMap<String, Fact>>>,
    latest_block: Arc<Mutex<Option<BlockInfo>>>,
}

// 2. Implement core methods
impl DomainAdapter for MyDomainAdapter {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    async fn domain_info(&self) -> Result<DomainInfo> {
        // Return domain metadata
        Ok(DomainInfo {
            id: self.domain_id.clone(),
            name: self.config.name.clone(),
            domain_type: self.config.domain_type.clone(),
            status: DomainStatus::Active,
            // ... other fields
        })
    }
    
    // ... implement other required methods
}

// 3. Register with the domain registry
pub async fn register_my_domain(registry: &DomainRegistry, config: DomainConfig) -> Result<()> {
    let adapter = Arc::new(MyDomainAdapter::new(config)?);
    registry.register_adapter(adapter)
}
```

### Domain Adapter Factory Pattern

For dynamic creation of domain adapters, implement the `DomainAdapterFactory` trait:

```rust
pub struct MyDomainAdapterFactory;

#[async_trait]
impl DomainAdapterFactory for MyDomainAdapterFactory {
    async fn create_adapter(&self, config: HashMap<String, String>) -> Result<Box<dyn DomainAdapter>> {
        // Parse configuration
        let domain_id = config.get("domain_id")
            .ok_or_else(|| Error::InvalidArgument("Missing domain_id".into()))?;
        let rpc_url = config.get("rpc_url")
            .ok_or_else(|| Error::InvalidArgument("Missing rpc_url".into()))?;
        
        // Create and return the adapter
        let adapter = MyDomainAdapter::new(
            DomainId::new(domain_id),
            Client::new(rpc_url),
            // other config options
        )?;
        
        Ok(Box::new(adapter))
    }
    
    fn supported_domain_types(&self) -> Vec<String> {
        vec!["my_domain_type".to_string()]
    }
    
    fn supports_domain_type(&self, domain_type: &str) -> bool {
        domain_type == "my_domain_type"
    }
}
```

## On-Chain vs Off-Chain Components

Domain adapters operate at the interface between on-chain and off-chain systems. Here's a clear breakdown of components:

### On-Chain Components (Must be implemented in blockchain):

1. **Register Storage Contract**: Smart contracts that store register state or commitments
   ```solidity
   // EVM example (simplified)
   contract RegisterStorage {
       mapping(bytes32 => bytes32) public commitments;
       mapping(bytes32 => bool) public nullifiers;
       
       function storeCommitment(bytes32 registerId, bytes32 commitment) external {
           commitments[registerId] = commitment;
       }
       
       function nullifyRegister(bytes32 registerId, bytes32 nullifier) external {
           require(!nullifiers[nullifier], "Already nullified");
           nullifiers[nullifier] = true;
       }
   }
   ```

2. **Nullifier Set**: On-chain storage to track which commitments have been consumed
   - Must be maintained by the blockchain to prevent double-spending

3. **Verification Contracts**: For verifying proofs and signatures
   - Critical for ensuring integrity of cross-domain operations

4. **Storage Strategy Implementation**: Contracts implementing different storage models:
   - **FullyOnChain**: All register data stored directly on-chain
   - **CommitmentBased**: Only commitments stored on-chain, data kept off-chain
   - **Hybrid**: Critical fields on-chain, with commitments for larger data

### Off-Chain Components (Implemented in Causality):

1. **Domain Adapter Logic**: Connection handling, state queries, transaction creation
   - Implemented in the Causality framework, not on the blockchain

2. **Fact Cache**: Temporary storage for observed blockchain facts
   - Used for performance optimization, not for state consistency

3. **Time Map Management**: Synchronization of time information across domains
   - Managed by Causality, updates sent to on-chain components when needed

4. **Domain Selection Logic**: Strategies for selecting optimal domains
   - Purely off-chain decision making process

5. **Witness Management**: For privacy-preserving operations with ZK-proofs
   - Witnesses are never stored on-chain, only used to generate proofs

6. **Effect System Integration**: Domain-specific effect implementations
   - Bridges the Effect System with domain adapters for seamless operation

## Time Map Integration

The Time Map is a critical component for cross-domain temporal synchronization. It maintains mappings between different domain timelines.

### Implementing Time Map Integration

```rust
impl MyDomainAdapter {
    // Implement time map entry generation
    async fn time_map_entry(&self, height: BlockHeight) -> Result<TimeMapEntry> {
        // Get block information for the specified height
        let block_info = self.get_block_at_height(height).await?;
        
        // Create and return a time map entry
        Ok(TimeMapEntry {
            domain_id: self.domain_id.clone(),
            height,
            hash: block_info.hash.clone(),
            timestamp: block_info.timestamp,
            confidence: 1.0, // Adjust based on finality status
            verified: true,  // Set to true if cryptographically verified
            source: "adapter".to_string(),
            metadata: HashMap::new(),
        })
    }
}
```

### On-Chain Time Anchoring (Optional)

For increased security, time map entries can be anchored on-chain.

## Fact Observation Pattern

The fact observation pattern enables querying domain-specific state through a unified interface.

### Implementing Fact Observation

```rust
impl MyDomainAdapter {
    // Implement fact observation
    async fn observe_fact(&self, query: &FactQuery) -> FactResult {
        // Check if fact is cached
        if let Some(cached_fact) = self.check_fact_cache(query) {
            return Ok(cached_fact);
        }
        
        // Process based on fact type
        match query.fact_type.as_str() {
            "account_balance" => {
                // Extract parameters
                let account = query.parameters.get("account")
                    .ok_or(Error::InvalidArgument("Missing account parameter".into()))?;
                let token = query.parameters.get("token")
                    .unwrap_or(&"native".to_string());
                
                // Query blockchain
                let balance = self.client.get_balance(account, token).await?;
                
                // Create fact
                let mut fact = Fact::new(&self.domain_id, "account_balance");
                fact.data.insert("account".to_string(), account.clone());
                fact.data.insert("token".to_string(), token.clone());
                fact.data.insert("balance".to_string(), balance.to_string());
                
                // Cache fact
                self.cache_fact(query, &fact);
                
                Ok(fact)
            },
            "block_info" => {
                // ... implement block info fact
                // ...
            },
            // ... other fact types
            _ => Err(Error::UnsupportedFactType(query.fact_type.clone())),
        }
    }
}
```

## Transaction Submission

Domain adapters must implement transaction submission to enable state changes on the blockchain.

### Implementing Transaction Submission

```rust
impl MyDomainAdapter {
    // Implement transaction submission
    async fn submit_transaction(&self, tx: &Transaction) -> Result<TransactionId> {
        // Validate transaction
        self.validate_transaction(tx)?;
        
        // Process based on transaction type
        match tx.tx_type.as_str() {
            "transfer" => {
                // Extract parameters
                let recipient = tx.parameters.get("recipient")
                    .ok_or(Error::InvalidArgument("Missing recipient parameter".into()))?;
                let amount = tx.parameters.get("amount")
                    .ok_or(Error::InvalidArgument("Missing amount parameter".into()))?
                    .parse::<u64>()?;
                let token = tx.parameters.get("token")
                    .unwrap_or(&"native".to_string());
                
                // Create domain-specific transaction
                let domain_tx = self.client.create_transfer_tx(
                    tx.sender.as_ref().ok_or(Error::InvalidArgument("Missing sender".into()))?,
                    recipient,
                    amount,
                    token,
                    tx.gas_limit,
                    tx.gas_price,
                )?;
                
                // Sign if signature not provided
                let signed_tx = if let Some(signature) = &tx.signature {
                    self.client.attach_signature(domain_tx, signature)?
                } else if let Some(private_key) = self.config.private_key.as_ref() {
                    self.client.sign_transaction(domain_tx, private_key)?
                } else {
                    return Err(Error::AuthenticationFailed("No signature or private key available".into()));
                };
                
                // Submit transaction
                let tx_id = self.client.submit_transaction(signed_tx).await?;
                
                Ok(TransactionId(tx_id))
            },
            "smart_contract_call" => {
                // ... implement smart contract call
                // ...
            },
            // ... other transaction types
            _ => Err(Error::UnsupportedTransactionType(tx.tx_type.clone())),
        }
    }
}
```

## Domain Selection Strategies

Domain selection strategies determine which domain to use for specific operations.

### Implementing Domain Selection

```rust
// Domain selection by type
pub async fn select_domain_by_type(registry: &DomainRegistry, domain_type: &str) -> Result<DomainId> {
    let domains = registry.list_domains().await?;
    
    // Filter domains by type
    let matching_domains: Vec<_> = domains.into_iter()
        .filter(|domain| domain.domain_type == domain_type)
        .collect();
    
    if matching_domains.is_empty() {
        return Err(Error::DomainNotFound(format!("No domains of type {} found", domain_type)));
    }
    
    // Select the first matching domain (could implement more complex selection logic)
    Ok(matching_domains[0].id.clone())
}

// Domain selection by capability
pub async fn select_domain_by_capability(registry: &DomainRegistry, capability: &str) -> Result<DomainId> {
    let domains = registry.list_domains().await?;
    
    // Filter domains by capability
    let matching_domains: Vec<_> = domains.into_iter()
        .filter(|domain| {
            let adapter = registry.get_adapter(&domain.id).unwrap();
            adapter.has_capability(capability)
        })
        .collect();
    
    if matching_domains.is_empty() {
        return Err(Error::DomainNotFound(format!("No domains with capability {} found", capability)));
    }
    
    // Select the first matching domain (could implement more complex selection logic)
    Ok(matching_domains[0].id.clone())
}
```

## Effect System Integration

The Effect System in Causality allows for the execution of side effects with proper authorization and context tracking. Domain adapters integrate with the effect system to provide a unified interface for blockchain interactions.

### Domain Effect Handler

The `DomainEffectHandler` trait connects domain adapters with the effect system:

```rust
#[async_trait]
pub trait DomainEffectHandler {
    async fn execute_domain_effect(&self, effect: &dyn DomainAdapterEffect, context: &EffectContext) -> EffectResult<EffectOutcome>;
    fn can_handle_effect(&self, effect: &dyn Effect) -> bool;
}
```

### Effect Domain Registry

The `EffectDomainRegistry` implements both the `DomainAdapterRegistry` and `DomainEffectHandler` traits:

```rust
pub struct EffectDomainRegistry {
    factories: RwLock<Vec<Arc<dyn DomainAdapterFactory>>>,
    adapters: RwLock<HashMap<DomainId, Arc<dyn DomainAdapter>>>,
}

impl EffectDomainRegistry {
    // Create a new effect domain registry
    pub fn new() -> Self {
        Self {
            factories: RwLock::new(Vec::new()),
            adapters: RwLock::new(HashMap::new()),
        }
    }
    
    // Execute domain effects
    pub async fn execute_query(&self, effect: &DomainQueryEffect, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Implementation details...
    }
    
    pub async fn execute_transaction(&self, effect: &DomainTransactionEffect, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Implementation details...
    }
    
    // ... other effect execution methods
}
```

### Domain Effect Handler Adapter

The `DomainEffectHandlerAdapter` provides a bridge between the effect system and domain adapters:

```rust
pub struct DomainEffectHandlerAdapter {
    registry: Arc<EffectDomainRegistry>,
}

#[async_trait]
impl EffectHandler for DomainEffectHandlerAdapter {
    async fn handle(&self, effect: Arc<dyn Effect>, context: &EffectContext) -> HandlerResult {
        // Implementation details...
    }
    
    fn can_handle(&self, effect: &dyn Effect) -> bool {
        // Check if this is a domain effect
        self.registry.can_handle_effect(effect)
    }
}
```

## Cross-Domain Operations

Cross-domain operations involve coordinating actions across multiple blockchain domains.

### Implementing Cross-Domain Operations

```rust
// Cross-domain token transfer
pub async fn cross_domain_transfer(
    registry: &DomainRegistry,
    source_domain: &DomainId,
    target_domain: &DomainId,
    sender: &str,
    recipient: &str,
    amount: u64,
    token: &str,
) -> Result<(TransactionId, TransactionId)> {
    // Get domain adapters
    let source_adapter = registry.get_adapter(source_domain)?;
    let target_adapter = registry.get_adapter(target_domain)?;
    
    // Verify sender has sufficient balance
    let balance_query = FactQuery::new(
        source_domain.clone(),
        "account_balance".to_string(),
    ).with_parameter("account", sender.to_string())
     .with_parameter("token", token.to_string());
    
    let balance_fact = source_adapter.observe_fact(&balance_query).await?;
    let balance = balance_fact.data.get("balance")
        .ok_or(Error::InvalidState("Balance not found in fact".into()))?
        .parse::<u64>()?;
    
    if balance < amount {
        return Err(Error::InsufficientFunds(format!("Sender has insufficient balance: {} < {}", balance, amount)));
    }
    
    // Submit lock transaction on source domain
    let lock_tx = Transaction::new(
        source_domain.clone(),
        "lock_tokens".to_string(),
    ).with_parameter("sender", sender.to_string())
     .with_parameter("amount", amount.to_string())
     .with_parameter("token", token.to_string())
     .with_parameter("target_domain", target_domain.to_string())
     .with_parameter("target_recipient", recipient.to_string());
    
    let lock_tx_id = source_adapter.submit_transaction(&lock_tx).await?;
    
    // Wait for confirmation
    let lock_receipt = source_adapter.wait_for_confirmation(&lock_tx_id, None).await?;
    if lock_receipt.status != TransactionStatus::Success {
        return Err(Error::TransactionFailed("Source domain lock transaction failed".into()));
    }
    
    // Submit mint/release transaction on target domain
    let mint_tx = Transaction::new(
        target_domain.clone(),
        "mint_tokens".to_string(),
    ).with_parameter("recipient", recipient.to_string())
     .with_parameter("amount", amount.to_string())
     .with_parameter("token", token.to_string())
     .with_parameter("source_domain", source_domain.to_string())
     .with_parameter("source_tx_id", lock_tx_id.to_string());
    
    let mint_tx_id = target_adapter.submit_transaction(&mint_tx).await?;
    
    Ok((lock_tx_id, mint_tx_id))
}
```

## Domain Capability System

The domain capability system tracks which operations each domain adapter supports.

### Implementing Capability Checks

```rust
impl MyDomainAdapter {
    // Implement capability checks
    fn capabilities(&self) -> Vec<String> {
        // Return list of supported capabilities
        vec![
            "account_balance".to_string(),
            "token_transfer".to_string(),
            "smart_contract_call".to_string(),
            // ... other capabilities
        ]
    }
    
    fn has_capability(&self, capability: &str) -> bool {
        // Check if capability is supported
        self.capabilities().contains(&capability.to_string())
    }
}
```

## Testing Domain Integrations

Proper testing of domain adapters is essential for reliable operation.

### Mock Domain Adapter for Testing

```rust
pub struct MockDomainAdapter {
    domain_id: DomainId,
    mock_facts: HashMap<String, Fact>,
    mock_transactions: HashMap<TransactionId, TransactionReceipt>,
}

impl MockDomainAdapter {
    // Create a new mock adapter
    pub fn new(domain_id: DomainId) -> Self {
        Self {
            domain_id,
            mock_facts: HashMap::new(),
            mock_transactions: HashMap::new(),
        }
    }
    
    // Add a mock fact
    pub fn add_mock_fact(&mut self, query_key: String, fact: Fact) {
        self.mock_facts.insert(query_key, fact);
    }
    
    // Add a mock transaction
    pub fn add_mock_transaction(&mut self, tx_id: TransactionId, receipt: TransactionReceipt) {
        self.mock_transactions.insert(tx_id, receipt);
    }
}

impl DomainAdapter for MockDomainAdapter {
    // Implement trait methods using mock data
    // ...
}
```

## Domain-Specific Effects

The integration between domain adapters and the effect system is enhanced with domain-specific effects for different blockchain types. These effects provide type-safe and protocol-specific interfaces for interacting with various blockchains.

### Core Domain Effect Traits

All domain effects share common traits:

```rust
pub trait DomainAdapterEffect: Effect {
    fn domain_id(&self) -> &DomainId;
    fn as_any(&self) -> &dyn std::any::Any;
}
```

### EVM-Specific Effects

For Ethereum Virtual Machine (EVM) compatible blockchains, the following effects are available:

#### 1. EvmContractCallEffect

For calling functions on EVM smart contracts:

```rust
pub struct EvmContractCallEffect {
    id: EffectId,
    domain_id: DomainId,
    contract_address: String,
    function_signature: String,
    function_arguments: Vec<String>,
    value: Option<String>,
    gas_limit: Option<u64>,
    transaction_type: EvmTransactionType,
    parameters: HashMap<String, String>,
}
```

**Usage Example**:
```rust
// Create a view call effect (read-only)
let balance_call = evm_view_call(
    "ethereum:mainnet",
    "0x1234567890abcdef1234567890abcdef12345678",
    "balanceOf(address)",
    vec!["0xabcdef1234567890abcdef1234567890abcdef12"]
);

// Create a transaction call effect (state-changing)
let transfer_call = evm_transaction_call(
    "ethereum:mainnet",
    "0x1234567890abcdef1234567890abcdef12345678",
    "transfer(address,uint256)",
    vec![
        "0xabcdef1234567890abcdef1234567890abcdef12", 
        "1000000000000000000"
    ]
).with_gas_limit(100000);
```

#### 2. EvmStateQueryEffect

For querying various aspects of EVM state:

```rust
pub struct EvmStateQueryEffect {
    id: EffectId,
    domain_id: DomainId,
    query_type: EvmStateQueryType,
    target: String,
    block_number: Option<u64>,
    parameters: HashMap<String, String>,
}

pub enum EvmStateQueryType {
    Balance,
    Storage,
    Code,
    Transaction,
    Block,
}
```

**Usage Example**:
```rust
// Query account balance
let balance_query = evm_balance(
    "ethereum:mainnet",
    "0xabcdef1234567890abcdef1234567890abcdef12"
);

// Query contract storage
let storage_query = evm_storage(
    "ethereum:mainnet",
    "0x1234567890abcdef1234567890abcdef12345678",
    "0x0000000000000000000000000000000000000000000000000000000000000001"
);
```

#### 3. EvmGasEstimationEffect

For estimating gas costs of transactions:

```rust
pub struct EvmGasEstimationEffect {
    id: EffectId,
    domain_id: DomainId,
    contract_address: String,
    function_signature: String,
    function_arguments: Vec<String>,
    value: Option<String>,
    parameters: HashMap<String, String>,
}
```

**Usage Example**:
```rust
// Estimate gas for a transfer
let gas_estimate = evm_estimate_gas(
    "ethereum:mainnet",
    "0x1234567890abcdef1234567890abcdef12345678",
    "transfer(address,uint256)",
    vec![
        "0xabcdef1234567890abcdef1234567890abcdef12", 
        "1000000000000000000"
    ]
);
```

### CosmWasm-Specific Effects

For CosmWasm-compatible blockchains, the following effects are available:

#### 1. CosmWasmExecuteEffect

For executing messages on CosmWasm contracts:

```rust
pub struct CosmWasmExecuteEffect {
    id: EffectId,
    domain_id: DomainId,
    contract_address: String,
    msg: String,
    funds: Option<Vec<(String, u128)>>,
    parameters: HashMap<String, String>,
}
```

**Usage Example**:
```rust
// Execute a contract message
let execute_effect = cosmwasm_execute(
    "cosmos:juno-1",
    "juno1xyz...",
    r#"{"transfer":{"recipient":"juno1abc...","amount":"1000"}}"#
).with_funds(vec![("ujuno".to_string(), 0u128)]);
```

#### 2. CosmWasmQueryEffect

For querying CosmWasm contracts:

```rust
pub struct CosmWasmQueryEffect {
    id: EffectId,
    domain_id: DomainId,
    contract_address: String,
    query: String,
    parameters: HashMap<String, String>,
}
```

**Usage Example**:
```rust
// Query a contract
let query_effect = cosmwasm_query(
    "cosmos:juno-1",
    "juno1xyz...",
    r#"{"balance":{"address":"juno1abc..."}}"#
);
```

#### 3. CosmWasmInstantiateEffect

For instantiating CosmWasm contracts:

```rust
pub struct CosmWasmInstantiateEffect {
    id: EffectId,
    domain_id: DomainId,
    code_id: u64,
    msg: String,
    label: String,
    funds: Option<Vec<(String, u128)>>,
    parameters: HashMap<String, String>,
}
```

**Usage Example**:
```rust
// Instantiate a contract
let instantiate_effect = cosmwasm_instantiate(
    "cosmos:juno-1",
    123, // code_id
    r#"{"count": 0}"#,
    "My Counter Contract"
);
```

#### 4. CosmWasmCodeUploadEffect

For uploading contract code:

```rust
pub struct CosmWasmCodeUploadEffect {
    id: EffectId,
    domain_id: DomainId,
    wasm_bytecode: String,
    parameters: HashMap<String, String>,
}
```

**Usage Example**:
```rust
// Upload contract code
let upload_effect = cosmwasm_upload(
    "cosmos:juno-1",
    "AGFzbQEB..." // base64 encoded WASM bytecode
);
```

### ZK/Succinct-Specific Effects

For Zero-Knowledge and Succinct-compatible blockchains, the following effects are available:

#### 1. ZkProveEffect

For generating zero-knowledge proofs:

```rust
pub struct ZkProveEffect {
    id: EffectId,
    domain_id: DomainId,
    circuit_id: String,
    private_inputs: String,
    public_inputs: Vec<String>,
    parameters: HashMap<String, String>,
}
```

**Usage Example**:
```rust
// Generate a proof
let prove_effect = zk_prove(
    "zk:succinct:1",
    "factorial_circuit",
    r#"{"n": 5}"#
).with_public_input("120"); // 5! = 120
```

#### 2. ZkVerifyEffect

For verifying zero-knowledge proofs:

```rust
pub struct ZkVerifyEffect {
    id: EffectId,
    domain_id: DomainId,
    verification_key_id: String,
    proof: String,
    public_inputs: Vec<String>,
    parameters: HashMap<String, String>,
}
```

**Usage Example**:
```rust
// Verify a proof
let verify_effect = zk_verify(
    "zk:succinct:1",
    "factorial_vk",
    "proof123hash", // Proof hash or data
    vec!["120"] // Public inputs
);
```

#### 3. ZkWitnessEffect

For creating witnesses for ZK circuits:

```rust
pub struct ZkWitnessEffect {
    id: EffectId,
    domain_id: DomainId,
    circuit_id: String,
    witness_data: String,
    parameters: HashMap<String, String>,
}
```

**Usage Example**:
```rust
// Create a witness
let witness_effect = zk_witness(
    "zk:succinct:1",
    "merkle_circuit",
    r#"{"leaves": ["a", "b", "c", "d"], "proof_path": [1, 0]}"#
);
```

#### 4. ZkProofCompositionEffect

For composing multiple proofs:

```rust
pub struct ZkProofCompositionEffect {
    id: EffectId,
    domain_id: DomainId,
    composition_circuit_id: String,
    source_proof_hashes: Vec<String>,
    parameters: HashMap<String, String>,
}
```

**Usage Example**:
```rust
// Compose proofs
let compose_effect = zk_compose(
    "zk:succinct:1",
    "recursive_circuit"
).with_source_proof_hash("proof123")
 .with_source_proof_hash("proof456");
```

### Bidirectional Integration Benefits

The integration of domain adapters with the effect system provides several key benefits:

1. **Type Safety**: Domain-specific effects provide strong typing for operations, reducing errors.
2. **Composition**: Effects can be composed and chained, simplifying complex operations.
3. **Authorization**: The effect system's authorization model is applied to domain operations.
4. **Unified API**: Applications can use a consistent interface for all domain operations.
5. **Extensibility**: New domain types can be added without changing the core system.
6. **Cross-Domain Operations**: Effects can coordinate operations across multiple domains.
7. **Testing**: Mock implementations of both effects and adapters simplify testing.

By implementing domain-specific effects, the Causality system provides a powerful, type-safe, and composable interface for interacting with diverse blockchain ecosystems. 