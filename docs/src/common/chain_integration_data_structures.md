# Chain Integration Data Structures

This document provides a comprehensive reference for the data structures used in chain integration within the Causality system.

## Core Identity and Reference Structures

### DomainId

The `DomainId` structure uniquely identifies a blockchain domain in the system.

**Fields:**
- `id`: String - A unique identifier for the domain, typically in the format `chain:network` (e.g., `ethereum:mainnet`, `cosmos:juno-1`)

**Behavior:**
- Serves as a primary key for domain lookups in the registry
- Used in effects to target specific blockchain domains
- Used in cross-domain operations to identify source and target domains

### FactQuery

The `FactQuery` structure represents a request for domain-specific state information.

**Fields:**
- `domain_id`: DomainId - The domain to query
- `fact_type`: String - The type of fact to observe (e.g., "account_balance", "block_info")
- `parameters`: HashMap<String, String> - Query parameters specific to the fact type

**Behavior:**
- Sent to domain adapters to request state information
- Used with the `observe_fact` method of the domain adapter

### Fact

The `Fact` structure represents an observed piece of domain state with associated metadata.

**Fields:**
- `id`: String - Unique identifier for the fact
- `domain_id`: DomainId - Domain that produced the fact
- `fact_type`: String - Type of the fact (e.g., "account_balance", "block_info")
- `data`: HashMap<String, String> - The fact's actual data
- `block_height`: Option<u64> - Block height when the fact was observed
- `timestamp`: Option<u64> - Timestamp when the fact was observed
- `proof`: Option<Vec<u8>> - Optional cryptographic proof of the fact
- `metadata`: HashMap<String, String> - Additional fact metadata

**Behavior:**
- Returned from domain adapters in response to fact queries
- Can be verified with on-chain data
- May include cryptographic proofs for critical facts

## Data Commitment Structures

### Commitment

The `Commitment` structure represents a cryptographic commitment to a piece of data.

**Fields:**
- `commitment_type`: String - The type of commitment (e.g., "merkle", "pedersen")
- `value`: Vec<u8> - The commitment value (typically a hash)
- `metadata`: HashMap<String, String> - Additional metadata about the commitment

**Behavior:**
- Used to commit to data without revealing it
- Can be stored on-chain for later verification
- Used in privacy-preserving operations

### Nullifier

The `Nullifier` structure represents a cryptographic nullifier for a used commitment.

**Fields:**
- `value`: Vec<u8> - The nullifier value
- `commitment_id`: Option<String> - Reference to the nullified commitment
- `metadata`: HashMap<String, String> - Additional metadata

**Behavior:**
- Used to prevent double-spending of commitments
- Typically stored on-chain in a nullifier set
- Critical for maintaining system integrity

## Domain Adapter Structures

### DomainAdapter

The `DomainAdapter` trait defines the interface for domain-specific blockchain adapters.

**Methods:**
- Identity: `domain_id()`, `domain_info()`
- Block and time: `current_height()`, `current_hash()`, `current_time()`, `time_map_entry()`
- State observation: `observe_fact()`
- Transactions: `submit_transaction()`, `transaction_receipt()`, `transaction_confirmed()`, `wait_for_confirmation()`
- Capabilities: `capabilities()`, `has_capability()`, `estimate_fee()`

**Behavior:**
- Implements domain-specific logic for blockchain interaction
- Handles RPC connections and protocol details
- Manages state caching and optimization

### DomainInfo

The `DomainInfo` structure provides metadata about a blockchain domain.

**Fields:**
- `id`: DomainId - The domain's unique identifier
- `name`: String - Human-readable name
- `domain_type`: String - Type of domain (e.g., "evm", "cosmwasm", "zk")
- `status`: DomainStatus - Current status (Active, Inactive, etc.)
- `parameters`: HashMap<String, String> - Domain-specific parameters
- `capabilities`: Vec<String> - List of supported capabilities

**Behavior:**
- Used for domain discovery and selection
- Contains metadata needed for domain operations
- Queried through the `domain_info()` method of domain adapters

## Time Synchronization Structures

### TimeMapEntry

The `TimeMapEntry` structure maps block heights to timestamps across domains.

**Fields:**
- `domain_id`: DomainId - Domain this entry belongs to
- `height`: u64 - Block height
- `hash`: String - Block hash
- `timestamp`: u64 - Timestamp (in milliseconds since epoch)
- `confidence`: f64 - Confidence score (0.0-1.0)
- `verified`: bool - Whether this entry has been cryptographically verified
- `source`: String - Source of the entry (e.g., "adapter", "oracle")
- `metadata`: HashMap<String, String> - Additional metadata

**Behavior:**
- Used for cross-domain time synchronization
- Critical for time-sensitive operations across domains
- Updated by domain adapters as new blocks are produced

## Transaction Structures

### Transaction

The `Transaction` structure represents a transaction to be submitted to a blockchain.

**Fields:**
- `domain_id`: DomainId - Target domain for the transaction
- `tx_type`: String - Type of transaction (e.g., "transfer", "contract_call")
- `sender`: Option<String> - Transaction sender
- `parameters`: HashMap<String, String> - Transaction parameters
- `gas_limit`: Option<u64> - Maximum gas to use
- `gas_price`: Option<u64> - Gas price in native units
- `nonce`: Option<u64> - Transaction nonce
- `signature`: Option<Vec<u8>> - Transaction signature
- `metadata`: HashMap<String, String> - Additional metadata

**Behavior:**
- Created by applications to request blockchain state changes
- Submitted to domain adapters via `submit_transaction()`
- Converted to domain-specific transaction formats internally

### TransactionReceipt

The `TransactionReceipt` structure contains information about a processed transaction.

**Fields:**
- `tx_id`: TransactionId - Unique identifier for the transaction
- `domain_id`: DomainId - Domain where the transaction was executed
- `block_height`: Option<u64> - Block containing the transaction
- `block_hash`: Option<String> - Hash of the block
- `status`: TransactionStatus - Status (Success, Failed, Pending)
- `gas_used`: Option<u64> - Amount of gas used
- `logs`: Vec<Log> - Transaction logs
- `events`: Vec<Event> - Events emitted during execution
- `metadata`: HashMap<String, String> - Additional metadata

**Behavior:**
- Returned by domain adapters after transaction submission
- Used to check transaction status and extract results
- Contains domain-specific transaction details

## Domain Effect Structures

### DomainAdapterEffect

The `DomainAdapterEffect` trait is the base for all domain-specific effects.

**Methods:**
- `domain_id()`: Returns the target domain for this effect
- `as_any()`: Allows downcasting to specific effect types

**Behavior:**
- Common interface for all domain-specific effects
- Enables effect system integration with domain adapters
- Used in the domain registry for effect handling

### DomainQueryEffect

The `DomainQueryEffect` structure represents a domain-specific query effect.

**Fields:**
- `id`: EffectId - Unique identifier for the effect
- `domain_id`: DomainId - Target domain for the query
- `query_type`: String - Type of query (e.g., "account_balance", "block_info")
- `parameters`: HashMap<String, String> - Query parameters
- `timepoint`: Option<Timepoint> - Optional specific time for the query

**Behavior:**
- Used to query domain state via the effect system
- Executed by the domain effect handler
- Results are mapped to effect outcomes

### DomainTransactionEffect

The `DomainTransactionEffect` structure represents a domain-specific transaction effect.

**Fields:**
- `id`: EffectId - Unique identifier for the effect
- `domain_id`: DomainId - Target domain for the transaction
- `tx_type`: String - Type of transaction (e.g., "transfer", "contract_call")
- `parameters`: HashMap<String, String> - Transaction parameters
- `gas_limit`: Option<u64> - Maximum gas to use
- `gas_price`: Option<u64> - Gas price in native units

**Behavior:**
- Used to submit transactions via the effect system
- Executed by the domain effect handler
- Results (transaction receipt) are mapped to effect outcomes

### DomainSelectionEffect

The `DomainSelectionEffect` structure represents a domain selection operation.

**Fields:**
- `id`: EffectId - Unique identifier for the effect
- `selection_criteria`: HashMap<String, String> - Criteria for domain selection
- `operation_type`: String - Type of operation to perform on the selected domain

**Behavior:**
- Used to dynamically select domains based on criteria
- May consider capabilities, performance metrics, and other factors
- Results in a selected domain ID for subsequent effects

## EVM-Specific Effect Structures

### EvmContractCallEffect

The `EvmContractCallEffect` structure represents a call to an EVM smart contract.

**Fields:**
- `id`: EffectId - Unique identifier for the effect
- `domain_id`: DomainId - Target EVM domain
- `contract_address`: String - Address of the contract to call
- `function_signature`: String - Function signature (e.g., "transfer(address,uint256)")
- `function_arguments`: Vec<String> - Arguments to pass to the function
- `value`: Option<String> - Native token value to send with the call
- `gas_limit`: Option<u64> - Maximum gas to use
- `transaction_type`: EvmTransactionType - Type of transaction (View, Transaction)
- `parameters`: HashMap<String, String> - Additional parameters

**Behavior:**
- Used for both read-only and state-changing contract interactions
- For read-only (View) calls, no transaction is created
- For state-changing (Transaction) calls, a transaction is submitted

### EvmStateQueryEffect

The `EvmStateQueryEffect` structure represents a query for EVM state.

**Fields:**
- `id`: EffectId - Unique identifier for the effect
- `domain_id`: DomainId - Target EVM domain
- `query_type`: EvmStateQueryType - Type of query (Balance, Storage, Code, Transaction, Block)
- `target`: String - Target address or hash
- `block_number`: Option<u64> - Optional specific block for the query
- `parameters`: HashMap<String, String> - Additional parameters

**Behavior:**
- Used to query various aspects of EVM state
- More specialized than the general DomainQueryEffect
- Results are formatted according to the query type

### EvmGasEstimationEffect

The `EvmGasEstimationEffect` structure represents a gas estimation request.

**Fields:**
- `id`: EffectId - Unique identifier for the effect
- `domain_id`: DomainId - Target EVM domain
- `contract_address`: String - Address of the contract
- `function_signature`: String - Function signature
- `function_arguments`: Vec<String> - Function arguments
- `value`: Option<String> - Native token value
- `parameters`: HashMap<String, String> - Additional parameters

**Behavior:**
- Used to estimate gas cost before submitting a transaction
- Performs a dry-run of the transaction without submitting it
- Results in estimated gas amount and possible failures

## CosmWasm-Specific Effect Structures

### CosmWasmExecuteEffect

The `CosmWasmExecuteEffect` structure represents a CosmWasm contract execution.

**Fields:**
- `id`: EffectId - Unique identifier for the effect
- `domain_id`: DomainId - Target Cosmos domain
- `contract_address`: String - Address of the contract
- `msg`: String - JSON-encoded execute message
- `funds`: Option<Vec<(String, u128)>> - Funds to send with the message
- `parameters`: HashMap<String, String> - Additional parameters

**Behavior:**
- Used to execute messages on CosmWasm contracts
- Results in a transaction on the Cosmos chain
- Response contains transaction hash and execution results

### CosmWasmQueryEffect

The `CosmWasmQueryEffect` structure represents a query to a CosmWasm contract.

**Fields:**
- `id`: EffectId - Unique identifier for the effect
- `domain_id`: DomainId - Target Cosmos domain
- `contract_address`: String - Address of the contract
- `query`: String - JSON-encoded query message
- `parameters`: HashMap<String, String> - Additional parameters

**Behavior:**
- Used for read-only queries to CosmWasm contracts
- Does not create a transaction
- Response contains the query result as JSON

### CosmWasmInstantiateEffect

The `CosmWasmInstantiateEffect` structure represents contract instantiation.

**Fields:**
- `id`: EffectId - Unique identifier for the effect
- `domain_id`: DomainId - Target Cosmos domain
- `code_id`: u64 - Code ID to instantiate
- `msg`: String - JSON-encoded instantiate message
- `label`: String - Label for the new contract
- `funds`: Option<Vec<(String, u128)>> - Funds to send with the message
- `parameters`: HashMap<String, String> - Additional parameters

**Behavior:**
- Used to create a new contract instance from existing code
- Results in a transaction on the Cosmos chain
- Response contains the new contract address and transaction hash

### CosmWasmCodeUploadEffect

The `CosmWasmCodeUploadEffect` structure represents a contract code upload.

**Fields:**
- `id`: EffectId - Unique identifier for the effect
- `domain_id`: DomainId - Target Cosmos domain
- `wasm_bytecode`: String - Base64-encoded WASM bytecode
- `parameters`: HashMap<String, String> - Additional parameters

**Behavior:**
- Used to upload new contract code to the chain
- Results in a transaction on the Cosmos chain
- Response contains the new code ID and transaction hash

## ZK/Succinct-Specific Effect Structures

### ZkProveEffect

The `ZkProveEffect` structure represents a zero-knowledge proof generation request.

**Fields:**
- `id`: EffectId - Unique identifier for the effect
- `domain_id`: DomainId - Target ZK domain
- `circuit_id`: String - Identifier for the circuit to use
- `private_inputs`: String - JSON-encoded private inputs
- `public_inputs`: Vec<String> - Public inputs
- `parameters`: HashMap<String, String> - Additional parameters

**Behavior:**
- Used to generate zero-knowledge proofs
- Private inputs are never revealed
- Results in a proof that can be verified on-chain

### ZkVerifyEffect

The `ZkVerifyEffect` structure represents a proof verification request.

**Fields:**
- `id`: EffectId - Unique identifier for the effect
- `domain_id`: DomainId - Target ZK domain
- `verification_key_id`: String - Identifier for the verification key
- `proof`: String - The proof to verify
- `public_inputs`: Vec<String> - Public inputs for verification
- `parameters`: HashMap<String, String> - Additional parameters

**Behavior:**
- Used to verify zero-knowledge proofs
- Can be executed both on-chain and off-chain
- Results in a boolean verification result

### ZkWitnessEffect

The `ZkWitnessEffect` structure represents a witness generation request.

**Fields:**
- `id`: EffectId - Unique identifier for the effect
- `domain_id`: DomainId - Target ZK domain
- `circuit_id`: String - Identifier for the circuit
- `witness_data`: String - JSON-encoded witness data
- `parameters`: HashMap<String, String> - Additional parameters

**Behavior:**
- Used to generate witnesses for ZK circuits
- Typically a preparatory step for proof generation
- Results in a witness that can be used for proving

### ZkProofCompositionEffect

The `ZkProofCompositionEffect` structure represents proof composition.

**Fields:**
- `id`: EffectId - Unique identifier for the effect
- `domain_id`: DomainId - Target ZK domain
- `composition_circuit_id`: String - Circuit for composition
- `source_proof_hashes`: Vec<String> - Proofs to compose
- `parameters`: HashMap<String, String> - Additional parameters

**Behavior:**
- Used to compose multiple proofs into a single proof
- Enables recursive proof compositions
- Results in a composed proof that verifies multiple statements 