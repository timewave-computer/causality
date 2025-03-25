<!-- Data structures for chain integration -->
<!-- Original file: docs/src/chain_integration_data_structures.md -->

# Core Data Structures for Blockchain Integration

When integrating the Causality framework with a blockchain system, several foundational data structures are used to represent various components of the integration. This document provides a comprehensive reference of these data structures, their behaviors, and system interactions.

## Core Identity and Reference Structures

### 1. DomainId

```rust
struct DomainId(String);
```

**Description**: A unique identifier for a blockchain domain (e.g., "ethereum-mainnet", "juno-1").

**Behavior**: Immutable, used as a key to reference domains across the system.

**System Interaction**: Used to route operations to the correct domain adapter and to identify the source of facts and resources.

### 2. ContentId

```rust
struct ContentId {
    hash: HashOutput
}
```

**Description**: A content-addressed identifier, typically derived from the hash of the content it represents.

**Behavior**: Unique, deterministic, and derived from content.

**System Interaction**: Used to reference resources, effects, and other content-addressed objects across domains.

## Data Commitment Structures

### 3. Commitment

```rust
struct Commitment {
    hash: [u8; 32],         // Cryptographic commitment hash
    metadata: Option<Bytes> // Optional public metadata
}
```

**Description**: The fundamental unit of program state. A commitment represents the existence of a resource without revealing its contents.

**Behavior**: Immutable once created. Can be verified against a witness without revealing the committed value.

**System Interaction**: Forms the basis of the on-chain state. Commitments are stored in the commitment tree and can be referenced by programs when constructing proofs. The hash is generated using a cryptographic commitment scheme that supports zero-knowledge proofs.

### 4. CommitmentTree

```rust
struct CommitmentTree {
    root: [u8; 32],               // Merkle root
    height: u8,                   // Tree height
    strategy: MerkleStrategy,     // Incremental, sparse, etc.
    leaf_count: u64               // Number of leaves
}
```

**Description**: An authenticated data structure (typically a Merkle tree) containing all valid commitments.

**Behavior**: Append-only structure that generates inclusion proofs. New commitments can be added, but existing ones cannot be removed or modified.

**System Interaction**: When new commitments are created, they're inserted into this tree. The tree provides inclusion proofs that programs use when constructing zero-knowledge proofs. The root hash is used to authenticate the current state in block headers.

### 5. Nullifier

```rust
struct Nullifier {
    hash: [u8; 32],      // Cryptographic nullifier hash
    block_created: u64   // Block when nullifier was added
}
```

**Description**: Represents a commitment that has been "spent" or used as an input to an operation.

**Behavior**: Once a nullifier is recorded, any future operation attempting to use the same nullifier will fail. This prevents double-spending.

**System Interaction**: When an operation consumes a commitment, it generates and reveals a nullifier. The system checks this nullifier against the existing set before accepting the operation. Nullifiers can be derived deterministically from commitments but cannot be linked back to their source commitment without the private witness data.

### 6. NullifierSet

```rust
struct NullifierSet {
    nullifiers: HashSet<[u8; 32]>,  // Set of active nullifiers
    index: HashMap<[u8; 32], u64>   // Maps nullifier to block created
}
```

**Description**: Tracks all nullifiers that have been published to the blockchain.

**Behavior**: Append-only set that allows efficient checking for existence. Once added, nullifiers remain forever.

**System Interaction**: Before accepting any operation, the system verifies that none of the nullifiers in the operation already exist in this set. This is the primary mechanism that prevents double-spending of commitments.

## Domain Adapter Structures

### 7. DomainInfo

```rust
struct DomainInfo {
    id: DomainId,                  // Unique domain ID
    name: String,                  // Human-readable name
    description: Option<String>,   // Optional description
    domain_type: DomainType,       // Type (EVM, CosmWasm, etc.)
    status: DomainStatus,          // Operational status
    capabilities: Vec<String>,     // Supported capabilities
    properties: HashMap<String, String> // Additional properties
}
```

**Description**: Provides key information about a blockchain domain.

**Behavior**: Semi-static, updated when domain status changes.

**System Interaction**: Used to display domain information to users and to determine capabilities for cross-domain operations.

### 8. DomainAdapter

```rust
trait DomainAdapter: Send + Sync + std::fmt::Debug {
    fn domain_id(&self) -> &DomainId;
    async fn domain_info(&self) -> Result<DomainInfo>;
    async fn current_height(&self) -> Result<BlockHeight>;
    async fn current_hash(&self) -> Result<BlockHash>;
    async fn current_time(&self) -> Result<Timestamp>;
    async fn time_map_entry(&self, height: BlockHeight) -> Result<TimeMapEntry>;
    async fn observe_fact(&self, query: &FactQuery) -> FactResult;
    async fn submit_transaction(&self, tx: Transaction) -> Result<TransactionId>;
    async fn transaction_receipt(&self, tx_id: &TransactionId) -> Result<TransactionReceipt>;
    async fn transaction_confirmed(&self, tx_id: &TransactionId) -> Result<bool>;
    async fn wait_for_confirmation(&self, tx_id: &TransactionId, max_wait_ms: Option<u64>) -> Result<TransactionReceipt>;
    fn capabilities(&self) -> Vec<String>;
    fn has_capability(&self, capability: &str) -> bool;
    async fn estimate_fee(&self, tx: &Transaction) -> Result<HashMap<String, u64>>;
}
```

**Description**: Interface for interacting with different blockchain domains.

**Behavior**: Provides a unified API for interacting with various blockchain systems regardless of their underlying protocols.

**System Interaction**: Applications use domain adapters to communicate with blockchains, query state, and submit transactions without needing to understand the specifics of each blockchain protocol.

### 9. DomainAdapterRegistry

```rust
struct DomainAdapterRegistry {
    adapters: HashMap<DomainId, Box<dyn DomainAdapter>>,
}
```

**Description**: Central registry for managing domain adapters.

**Behavior**: Maintains a mapping of domain IDs to adapter instances.

**System Interaction**: Applications query the registry to obtain the appropriate adapter for a given domain ID. The registry can also be used to discover available domains.

### 10. FactQuery

```rust
struct FactQuery {
    domain_id: DomainId,             // Domain to query
    fact_type: String,               // Type of fact to retrieve
    parameters: HashMap<String, String>, // Query parameters
    block_height: Option<BlockHeight>, // Optional block height
    block_hash: Option<BlockHash>,     // Optional block hash
    timestamp: Option<Timestamp>,      // Optional timestamp
}
```

**Description**: Parameters for querying domain-specific facts.

**Behavior**: Immutable once created, used to request specific data from a domain.

**System Interaction**: Applications create fact queries to retrieve information from domains, such as account balances, block details, or contract state.

## Time Synchronization Structures

### 11. TimeMapEntry

```rust
struct TimeMapEntry {
    domain_id: DomainId,        // Domain identifier
    height: BlockHeight,        // Block height
    hash: BlockHash,            // Block hash
    timestamp: Timestamp,       // Block timestamp
    confidence: f64,            // Confidence level
    verified: bool,             // Verification status
    source: String,             // Source of information
    metadata: HashMap<String, String> // Additional metadata
}
```

**Description**: Maps a block in one domain to a point in time, enabling cross-domain time synchronization.

**Behavior**: Created when a block is finalized, may be updated as confidence increases.

**System Interaction**: Used to establish temporal relationships between events in different domains and to determine causal relationships.

## Transaction Structures

### 12. Transaction

```rust
struct Transaction {
    domain_id: DomainId,         // Target domain
    tx_type: String,             // Transaction type
    sender: Option<String>,      // Optional sender
    target: Option<String>,      // Optional target (contract, address)
    data: Vec<u8>,               // Transaction data
    gas_limit: Option<u64>,      // Optional gas limit
    gas_price: Option<u64>,      // Optional gas price
    nonce: Option<u64>,          // Optional nonce
    signature: Option<Vec<u8>>,  // Optional signature
    metadata: HashMap<String, String> // Additional metadata
}
```

**Description**: Represents a transaction to be submitted to a blockchain.

**Behavior**: Immutable once created and signed.

**System Interaction**: Applications create transactions to update state on the blockchain. Domain adapters convert these generic transactions into domain-specific formats before submission.

### 13. TransactionReceipt

```rust
struct TransactionReceipt {
    tx_id: TransactionId,         // Transaction identifier
    domain_id: DomainId,          // Domain identifier
    block_height: Option<u64>,    // Block height if confirmed
    block_hash: Option<Vec<u8>>,  // Block hash if confirmed
    status: TransactionStatus,    // Transaction status
    gas_used: Option<u64>,        // Gas used if applicable
    logs: Vec<TransactionLog>,    // Transaction logs
    events: Vec<Event>,           // Events emitted
    metadata: HashMap<String, String> // Additional metadata
}
```

**Description**: Contains information about a submitted transaction.

**Behavior**: Initially created with pending status, updated as the transaction is processed.

**System Interaction**: Applications use transaction receipts to track the status of submitted transactions and to extract events and logs.

## Domain Effect Structures

### 14. DomainAdapterEffect

```rust
trait DomainAdapterEffect: Effect {
    fn domain_id(&self) -> &DomainId;
    fn as_any(&self) -> &dyn std::any::Any;
}
```

**Description**: Base trait for all domain-specific effects, extending the standard Effect trait.

**Behavior**: Provides a common interface for effects that interact with domain adapters.

**System Interaction**: The effect system uses this trait to route effect executions to the appropriate domain adapter.

### 15. DomainContext

```rust
struct DomainContext<'a> {
    effect_context: &'a EffectContext,
    domain_id: &'a DomainId,
}
```

**Description**: Domain-specific context for effect execution.

**Behavior**: Immutable reference that bridges the effect context with domain-specific information.

**System Interaction**: Used when executing domain effects to provide both general effect context and domain-specific details.

### 16. EffectDomainRegistry

```rust
struct EffectDomainRegistry {
    factories: RwLock<Vec<Arc<dyn DomainAdapterFactory>>>,
    adapters: RwLock<HashMap<DomainId, Arc<dyn DomainAdapter>>>,
}
```

**Description**: Registry that integrates domain adapters with the effect system.

**Behavior**: Thread-safe registry that manages domain adapter factories and instances.

**System Interaction**: The effect system uses this registry to find and create domain adapters for handling domain-specific effects.

## EVM-Specific Effect Structures

### 17. EvmContractCallEffect

```rust
struct EvmContractCallEffect {
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

**Description**: Effect for calling a function on an EVM smart contract.

**Behavior**: Immutable once created, can be configured with various parameters.

**System Interaction**: When executed, translates into appropriate EVM transactions for view or state-changing calls.

### 18. EvmStateQueryEffect

```rust
struct EvmStateQueryEffect {
    id: EffectId,
    domain_id: DomainId,
    query_type: EvmStateQueryType,
    target: String,
    block_number: Option<u64>,
    parameters: HashMap<String, String>,
}
```

**Description**: Effect for querying EVM blockchain state.

**Behavior**: Immutable once created, supports different query types (balance, storage, code, transaction, block).

**System Interaction**: When executed, queries the EVM blockchain for the specified state information.

### 19. EvmGasEstimationEffect

```rust
struct EvmGasEstimationEffect {
    id: EffectId,
    domain_id: DomainId,
    contract_address: String,
    function_signature: String,
    function_arguments: Vec<String>,
    value: Option<String>,
    parameters: HashMap<String, String>,
}
```

**Description**: Effect for estimating gas cost for an EVM contract call.

**Behavior**: Immutable once created, similar structure to EvmContractCallEffect.

**System Interaction**: When executed, calls the EVM node's gas estimation API.

## CosmWasm-Specific Effect Structures

### 20. CosmWasmExecuteEffect

```rust
struct CosmWasmExecuteEffect {
    id: EffectId,
    domain_id: DomainId,
    contract_address: String,
    msg: String,
    funds: Option<Vec<(String, u128)>>,
    parameters: HashMap<String, String>,
}
```

**Description**: Effect for executing a message on a CosmWasm smart contract.

**Behavior**: Immutable once created, configurable with message and funds.

**System Interaction**: When executed, submits an execute message to a CosmWasm contract.

### 21. CosmWasmQueryEffect

```rust
struct CosmWasmQueryEffect {
    id: EffectId,
    domain_id: DomainId,
    contract_address: String,
    query: String,
    parameters: HashMap<String, String>,
}
```

**Description**: Effect for querying a CosmWasm smart contract.

**Behavior**: Immutable once created, configured with a query message.

**System Interaction**: When executed, sends a query message to a CosmWasm contract.

### 22. CosmWasmInstantiateEffect

```rust
struct CosmWasmInstantiateEffect {
    id: EffectId,
    domain_id: DomainId,
    code_id: u64,
    msg: String,
    label: String,
    funds: Option<Vec<(String, u128)>>,
    parameters: HashMap<String, String>,
}
```

**Description**: Effect for instantiating a new CosmWasm smart contract from code.

**Behavior**: Immutable once created, configured with code ID, initialization message, and label.

**System Interaction**: When executed, creates a new CosmWasm contract instance.

### 23. CosmWasmCodeUploadEffect

```rust
struct CosmWasmCodeUploadEffect {
    id: EffectId,
    domain_id: DomainId,
    wasm_bytecode: String,
    parameters: HashMap<String, String>,
}
```

**Description**: Effect for uploading CosmWasm contract code to the blockchain.

**Behavior**: Immutable once created, configured with WASM bytecode.

**System Interaction**: When executed, uploads the compiled WASM code to the Cosmos blockchain.

## ZK/Succinct-Specific Effect Structures

### 24. ZkProveEffect

```rust
struct ZkProveEffect {
    id: EffectId,
    domain_id: DomainId,
    circuit_id: String,
    private_inputs: String,
    public_inputs: Vec<String>,
    parameters: HashMap<String, String>,
}
```

**Description**: Effect for generating a zero-knowledge proof.

**Behavior**: Immutable once created, configured with circuit ID, private inputs, and public inputs.

**System Interaction**: When executed, runs the specified circuit with the provided inputs to generate a ZK proof.

### 25. ZkVerifyEffect

```rust
struct ZkVerifyEffect {
    id: EffectId,
    domain_id: DomainId,
    verification_key_id: String,
    proof: String,
    public_inputs: Vec<String>,
    parameters: HashMap<String, String>,
}
```

**Description**: Effect for verifying a zero-knowledge proof.

**Behavior**: Immutable once created, configured with a verification key, proof, and public inputs.

**System Interaction**: When executed, verifies the provided proof against the specified verification key and public inputs.

### 26. ZkWitnessEffect

```rust
struct ZkWitnessEffect {
    id: EffectId,
    domain_id: DomainId,
    circuit_id: String,
    witness_data: String,
    parameters: HashMap<String, String>,
}
```

**Description**: Effect for creating a witness for a ZK circuit.

**Behavior**: Immutable once created, configured with circuit ID and witness data.

**System Interaction**: When executed, processes the witness data for use in the specified circuit.

### 27. ZkProofCompositionEffect

```rust
struct ZkProofCompositionEffect {
    id: EffectId,
    domain_id: DomainId,
    composition_circuit_id: String,
    source_proof_hashes: Vec<String>,
    parameters: HashMap<String, String>,
}
```

**Description**: Effect for composing multiple ZK proofs into a single proof.

**Behavior**: Immutable once created, configured with a composition circuit and source proof hashes.

**System Interaction**: When executed, combines multiple proofs using the specified composition circuit to create a new, single proof.

## Resource State Structures

### 18. Register

```rust
struct Register {
    id: [u8; 32],                // Unique register identifier
    commitment: [u8; 32],        // Commitment to register contents
    type_id: u16,                // Type of register (token, NFT, etc)
    status: RegisterStatus,      // Active, nullified, expired
    created_at: u64,             // Block created
    nullified_at: Option<u64>    // Block nullified, if applicable
}
```

**Description**: The fundamental unit of state in the system. Represents an atomic piece of state that follows a one-time-use model.

**Behavior**: Once created, a register is active until nullified. It can only be nullified once. Some registers are linked to commitments for privacy, while others might be public.

**System Interaction**: Programs create and nullify registers through operations. Each register transition generates an event that can be observed by the system. Register creation and nullification are the building blocks of all state transitions.

### 19. ResourceDefinition

```rust
struct ResourceDefinition {
    type_id: u16,                  // Resource type identifier
    name: String,                  // Resource name
    schema: Schema,                // Resource schema definition
    capabilities: Vec<Capability>, // What can be done with this resource
    circuit_ids: Vec<String>       // Compatible circuit IDs
}
```

**Description**: Defines a type of resource in the system, its structure, and what operations can be performed on it.

**Behavior**: Immutable once created, though new versions can be registered. Governs the rules for a specific resource type.

**System Interaction**: When processing operations, the system references these definitions to ensure the operation is valid for the resource type. Definitions can be registered through governance mechanisms and are referenced by registers and operations.

## Operation Structures

### 20. Operation

```rust
struct Operation {
    type_id: u16,            // Operation type
    inputs: Vec<Input>,      // Inputs (nullifiers, references)
    outputs: Vec<Output>,    // Outputs (new commitments)
    proof: Option<ZKProof>,  // Optional ZK proof
    metadata: Option<Vec<u8>>  // Optional public metadata
}
```

**Description**: Represents a state transition in the system. The fundamental unit of behavior.

**Behavior**: Atomic - either fully succeeds or fails. Operations consume inputs (nullifying them) and produce outputs (creating new commitments or registers).

**System Interaction**: Users and programs construct operations and submit them to the blockchain. The system verifies the operation's validity (checking nullifiers, verifying proofs) before applying it to the state. Each successful operation produces a transaction receipt.

### 21. Witness

```rust
struct Witness {
    commitment: [u8; 32],     // Commitment this witness relates to
    preimage: Vec<u8>,        // The actual committed value (private)
    path: MerklePath,         // Inclusion proof in commitment tree
    nullifier_key: [u8; 32]   // Key for deriving nullifier
}
```

**Description**: Private data that allows proving knowledge of a commitment's contents without revealing them.

**Behavior**: Never stored on-chain. Used to generate nullifiers and construct zero-knowledge proofs.

**System Interaction**: Users maintain witnesses client-side and use them when constructing operations. Witnesses are essential for privacy-preserving operations as they allow users to prove ownership without revealing the commitment contents.

## Event and Notification Structures

### 22. Event

```rust
struct Event {
    type_id: u16,             // Event type
    emitter: [u8; 32],        // Entity that emitted the event
    data: Vec<u8>,            // Event data
    topics: Vec<[u8; 32]>,    // Indexed topics for filtering
    block_number: u64,        // Block this event was emitted in
    tx_index: u32             // Transaction index in the block
}
```

**Description**: Notifications emitted by programs, operations, or the system itself.

**Behavior**: Append-only, immutable records of activity. Can be filtered and subscribed to.

**System Interaction**: Programs emit events when their state changes. Events are stored in an indexed log that can be queried by clients. They serve as a notification mechanism and are used to update client-side state.

### 23. Fact

```rust
struct Fact {
    id: FactId,                  // Unique fact identifier
    domain_id: DomainId,         // Source domain
    fact_type: String,           // Type of fact
    data: HashMap<String, Value>, // Fact data
    block_height: Option<u64>,    // Block height if applicable
    block_hash: Option<Vec<u8>>,  // Block hash if applicable
    timestamp: Option<u64>,       // Timestamp if applicable
    proof: Option<Vec<u8>>,       // Optional cryptographic proof
    metadata: HashMap<String, String> // Additional metadata
}
```

**Description**: A verifiable piece of information from a specific domain.

**Behavior**: Immutable once created, represents a snapshot of domain state at a specific time.

**System Interaction**: Applications query facts to retrieve information from domains. Facts can be used as inputs to operations and to construct proofs.

## Capability and Authorization Structures

### 24. CapabilityGrant

```rust
struct CapabilityGrant {
    id: [u8; 32],                  // Unique grant identifier
    resource_id: [u8; 32],         // Resource this grant applies to
    grantee: [u8; 32],             // Recipient of the capability
    rights: BitFlags<Right>,       // What actions are permitted
    restrictions: Vec<Restriction>, // Constraints on usage
    expires_at: Option<u64>,       // Optional expiration block
    status: GrantStatus            // Active, revoked, expired
}
```

**Description**: Represents a capability granted to a program or user to perform operations on a resource.

**Behavior**: Capabilities can be delegated, attenuated (restricted), combined, or revoked. They follow a principle of least privilege.

**System Interaction**: Programs request capabilities through the resource API. When a program invokes an operation, the system verifies it has the necessary capabilities before proceeding. Capabilities can be represented either as registers themselves or as separate state objects.

### 25. DomainCapability

```rust
enum DomainCapability {
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
    
    // Custom capability
    Custom(String)
}
```

**Description**: Represents a specific capability that a domain adapter can provide.

**Behavior**: Capabilities are queried at runtime to determine what operations are supported.

**System Interaction**: Applications check domain capabilities before attempting operations. The capability system ensures that operations are only attempted on domains that support them.

### 26. DomainCapabilityManager

```rust
struct DomainCapabilityManager {
    capability_system: Arc<dyn CapabilitySystem>,
    default_capabilities: HashMap<DomainType, HashSet<DomainCapability>>,
    domain_capabilities: HashMap<DomainId, HashSet<DomainCapability>>,
}
```

**Description**: Manages capabilities for domains and provides a unified interface for capability checks.

**Behavior**: Maintains a registry of capabilities for each domain and domain type.

**System Interaction**: Applications query the capability manager to determine if a domain supports specific operations. The capability manager integrates with the resource capability system to provide unified authorization.

## Cross-Domain Integration Structures

### 27. CrossDomainResourceManager

```rust
struct CrossDomainResourceManager {
    domain_registry: Arc<DomainRegistry>,
    resource_adapters: HashMap<DomainType, Box<dyn DomainResourceAdapter>>,
}
```

**Description**: Manages resources across multiple domains and provides a unified interface for resource operations.

**Behavior**: Coordinates resource operations across domains, handling the complexities of cross-domain transfers and synchronization.

**System Interaction**: Applications use the resource manager to perform operations on resources regardless of their domain location. The resource manager handles the routing of operations to the appropriate domain adapters.

### 28. DomainTimeMap

```rust
struct DomainTimeMap {
    entries: Vec<TimeMapEntry>,
    domains: HashSet<DomainId>,
}
```

**Description**: Maps timestamps between different domains, enabling cross-domain temporal synchronization.

**Behavior**: Maintains a registry of time mappings between domains.

**System Interaction**: Applications use the time map to determine temporal relationships between events in different domains and to establish causal ordering.

## Implementation-Specific Adapter Structures

### 29. EthereumAdapter

```rust
struct EthereumAdapter {
    config: EthereumConfig,
    provider: Provider<Http>,
    fact_cache: Arc<Mutex<HashMap<String, FactType>>>,
    latest_block: Arc<Mutex<Option<Block<H256>>>>,
}
```

**Description**: EVM domain adapter implementation.

**Behavior**: Connects to Ethereum-compatible chains, translates generic operations into EVM-specific transactions.

**System Interaction**: Handles all interactions with EVM-based blockchains, including transaction submission, state queries, and event monitoring.

### 30. CosmWasmAdapter

```rust
struct CosmWasmAdapter {
    config: CosmWasmAdapterConfig,
    fact_cache: Arc<Mutex<HashMap<String, Fact>>>,
    latest_block: Arc<Mutex<Option<BlockInfo>>>,
}
```

**Description**: CosmWasm domain adapter implementation.

**Behavior**: Connects to Cosmos-based chains with WebAssembly smart contract support.

**System Interaction**: Handles all interactions with CosmWasm-based blockchains, including transaction submission, state queries, and contract execution.

## Conclusion

These data structures form the foundation of blockchain integration in the Causality framework. By providing a consistent set of abstractions, they enable seamless interaction with different blockchain systems while maintaining a unified programming model. The integration leverages content-addressing, capability-based security, and cross-domain synchronization to create a robust and flexible framework for blockchain application development.


SYSTEM BOUNDARIES
 ┌───────────────────────────────────────────────────────────────────────────────────────┐
 │                                                                                       │
 │ ┌─────────────────────────────────────┐   OFF-CHAIN (CLIENT-SIDE)                     │
 │ │                                     │                                               │
 │ │  ┌─────────────┐                    │                                               │
 │ │  │             │  Maintains private │                                               │
 │ │  │  Witness    │◄─state & generates───┐                                             │
 │ │  │  Storage    │      ZK proofs     │ │                                             │
 │ │  │             │                    │ │                                             │
 │ │  └─────┬───────┘                    │ │                                             │
 │ │        │                            │ │                                             │
 │ │        ▼                            │ │                                             │
 │ │  ┌─────────────┐  Generates proofs  │ │                                             │
 │ │  │             │  and operations    │ │                                             │
 │ │  │ Client-side │                    │ │                                             │
 │ │  │ Program     │                    │ │                                             │
 │ │  │ Account     │──────────────────┐ │ │                                             │
 │ │  │             │                  │ │ │                                             │
 │ │  └──────┬──────┘                  │ │ │                                             │
 │ │         │                         │ │ │                                             │
 │ └─────────┼─────────────────────────┘ │ │                                             │
 │           │  User interface boundary  │ │                                             │
 │           ▼                           │ │                                             │
 │ ┌─────────────────┐   Submit         ┌┴─┴────────┐ Query       ┌──────────────────┐   │
 │ │                 │   operations     │           │ receipts    │                  │   │
 │ │    User (UI)    │─────────────────►│Transaction│◄────────────┤  UnifiedLog      │   │
 │ │                 │                  │Processor  │             │                  │   │
 │ └─────────────────┘                  │           │             └─────┬────────────┘   │
 │                                      └────┬──────┘                   │                │
 └───────────────────────────────────────────┼──────────────────────────┼────────────────┘
                 System boundary             │                          │                
 ┌───────────────────────────────────────────┼──────────────────────────┼────────────────┐
 │                                           │                          │                │
 │ ON-CHAIN                                  │                          │                │
 │                                           ▼                          │ Records        │
 │                        ┌────────────────────────────────┐            │ all activity   │
 │                        │                                │            │                │
 │                        │         Operation              │            │                │
 │                        │                                │            │                │
 │                        └───────────┬──────────┬─────────┘            │                │
 │                                    │          │                      │                │
 │                                    │          │                      │                │
 │                                    ▼          ▼                      ▼                │
 │              ┌────────────┐  ┌───────────┐  ┌───────────────┐  ┌──────────┐           │
 │              │            │  │           │  │               │  │          │           │
 │ Verifies    ┌┤ ZKProof    │  │ Program   │  │  EffectLog    │  │Event     │           │
 │ proofs      ││ Verifier   │  │           │  │               │  │Emitter   │           │
 │           ┌─┼┤            │  │           │  │               │  │          │           │
 │           │ │└─────┬──────┘  └─────┬─────┘  └───────┬───────┘  └──────────┘           │
 │           │ │      │               │                │                                 │
 │           │ │      ▼               ▼                ▼                                 │
 │           │ │┌───────────────┐ ┌───────────┐ ┌──────────┐                             │
 │           │ ││               │ │           │ │          │                             │
 │           │ ││VerificationKey│ │ResourceAPI│ │Resource  │                             │
 │           │ ││               │ │           │ │Definition│                             │
 │           │ │└───────────────┘ └────┬──────┘ └──────────┘                             │
 │           │ │                       │                                                 │
 │           │ │                       ▼                                                 │
 │           │ │                  ┌───────────────┐                                      │
 │           │ └─────────────────►│               │                                      │
 │           │                    │CapabilityGrant│                                      │
 │           └────────────────────►               │                                      │
 │                                └──────┬────────┘                                      │
 │                                       │                                               │
 │                                       ▼                                               │
 │ ┌───────────────┐             ┌───────────────┐             ┌────────────────┐        │
 │ │               │  Consumes   │               │  Creates    │                │        │
 │ │ Nullifier     │◄────────────┤ Register      ├────────────►│ Commitment     │        │
 │ │ Set           │             │               │             │ Tree           │        │
 │ │               │             │               │             │                │        │
 │ └───────────────┘             └───────────────┘             └────────────────┘        │
 │                                                                    ▲                  │
 │ ┌───────────────┐                  ┌───────────────┐               │                  │
 │ │               │  Observes other  │               │               │                  │
 │ │ TimeMap       │◄─────────────────┤ TimeKeeper    │               │                  │
 │ │               │  blockchains     │               │───────────────┘                  │
 │ └───────────────┘                  └───────────────┘     Updates with                 │
 │                                                          observed facts               │
 └───────────────────────────────────────────────────────────────────────────────────────┘

 This diagram maps out the main components and their interactions:

### User interaction flow:

User interacts with Client-side Program Account
Client constructs operations using Witness data
Operations go through Transaction Processor
Processor validates and applies to on-chain state


### Program execution flow:

Programs process operations
They request capabilities through ResourceAPI
Capabilities allow access to Registers
Register operations create Commitments and Nullifiers


### Cross-chain interaction flow:

Committees observe external blockchains
They update the TimeMap with observed facts
Operations can reference the TimeMap
Cross-chain proofs are verified against TimeMap


