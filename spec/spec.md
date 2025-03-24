# Causality System Specification (SPEC)

---

## Version

**Current Revision:** 2025-03-22

---

## Overview

This document defines the **technical specification** for Causality, capturing the latest understanding of its architecture, data structures, processes, and operational semantics. This specification serves as a **technical reference** for implementers, contributors, and auditors.

---

# System Purpose

Causality provides a **secure, provable, and composable execution environment** for programs that span **multiple Domains** (chains, event logs, or external systems of record). Programs interact with **resources** and **facts** from these Domains while preserving:

- Full causal traceability.
- Deterministic replayability.
- Sovereign ownership (Users own their programs entirely).
- Zero knowledge proofs of execution.
- Compatibility with **heterogeneous infrastructure** across Domains.

---

# Core Actors

## Users

- Deploy programs.
- Own programs and account programs.
- Initiate cross-domain deposits and withdrawals.
- Propose schema upgrades.
- Submit messages to Domains via Committees.

## Committees

- One per Domain.
- Observe external facts (balances, prices, transactions).
- Validate and timestamp facts.
- Append facts to per-Domain **FactLog**.
- Respond to fact queries from programs and Operators.
- Observe register state and verify register operations with ZK proofs.
- Maintain register state synchronization across the network.

## Causality

- Operate the execution network.
- Execute program logic.
- Propose and apply effects.
- Generate ZK proofs of execution.
- Maintain **unified logs** for all programs and account programs.
- Disseminate facts and effects across the P2P network.
- Coordinate register operations and execution sequences.
- Verify ZK proofs for register state transitions.

---

# Core Programs

## Logic Programs

- Define user-defined effect pipelines (business logic).
- Apply effects based on facts and received invocations.
- Declare:
    - Schema (state format).
    - Safe state policy.
    - Evolution rules.
- Interact with registers through the register interface.

## Account Programs

- Each User has one account program per deployment context.
- Owns all external assets (tokens, balances) across Domains through registers.
- Exposes:
    - Deposit API.
    - Withdrawal API.
    - Cross-program transfer API.
    - Balance query API.
    - Register creation and management API.
- Tracks causal history in its own effect DAG.
- Manages register authorization and operations.

---

# Core Data Structures

## Fact

```rust
/// A unique identifier for a fact
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactId(pub String);

/// A struct representing a point-in-time snapshot of facts
/// that an effect depends on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactSnapshot {
    /// Facts observed before the effect
    pub observed_facts: Vec<FactId>,
    
    /// The observer (committee) that observed the facts
    pub observer: String,
    
    /// The timestamp when the snapshot was created
    pub created_at: Timestamp,
    
    /// Register observations included in this snapshot
    pub register_observations: HashMap<RegisterId, RegisterObservation>,
    
    /// Domains that contributed facts to this snapshot
    pub domains: HashSet<DomainId>,
    
    /// Additional metadata for the snapshot
    pub metadata: HashMap<String, String>,
}

/// Base Effect trait that can be used as an object
pub trait Effect: Send + Sync {
    /// The output type of this effect
    type Output;
    
    /// Get the type of this effect
    fn get_type(&self) -> EffectType;
    
    /// Get a debug representation of this effect
    fn as_debug(&self) -> &dyn std::fmt::Debug;
    
    /// Clone this effect
    fn clone_box(&self) -> Box<dyn Effect<Output = Self::Output>>;
    
    /// Get the resources affected by this effect
    fn resources(&self) -> Vec<ResourceId>;
    
    /// Get the domains involved in this effect
    fn domains(&self) -> Vec<DomainId>;
    
    /// Execute this effect using the given handler
    fn execute(self, handler: &dyn EffectHandler) -> Self::Output;
    
    /// Get the fact dependencies for this effect (default implementation)
    fn fact_dependencies(&self) -> Vec<FactDependency> {
        Vec::new()
    }
    
    /// Get the fact snapshot for this effect (default implementation)
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        None
    }
}

/// Represents an observation of a register's state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterObservation {
    /// The observed register ID
    pub register_id: RegisterId,
    
    /// The fact ID of the register observation
    pub fact_id: FactId,
    
    /// The domain the register was observed in
    pub domain_id: DomainId,
    
    /// The timestamp of the observation
    pub observed_at: Timestamp,
    
    /// The hash of the register data
    pub data_hash: String,
}

/// Domain identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DomainId(pub String);
```

## Effect

```rust
/// Effect types in the Causality system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    /// Deposit assets into a domain
    Deposit {
        /// Target domain for the deposit
        domain: DomainId,
        /// Asset type being deposited
        asset: Asset,
        /// Amount being deposited
        amount: Amount,
    },
    
    /// Withdraw assets from a domain
    Withdraw {
        /// Source domain for the withdrawal
        domain: DomainId,
        /// Asset type being withdrawn
        asset: Asset,
        /// Amount being withdrawn
        amount: Amount,
    },
    
    /// Transfer assets between programs
    Transfer {
        /// Source program ID
        from_program: ProgramId,
        /// Target program ID
        to_program: ProgramId,
        /// Asset type being transferred
        asset: Asset,
        /// Amount being transferred
        amount: Amount,
    },
    
    /// Observe a fact from a domain
    ObserveFact {
        /// ID of the fact being observed
        fact_id: FactId,
    },
    
    /// Invoke another program
    Invoke {
        /// Target program ID to invoke
        target_program: ProgramId,
        /// Invocation data
        invocation: Invocation,
    },
    
    /// Evolve a program's schema
    EvolveSchema {
        /// Old schema being upgraded from
        old_schema: Schema,
        /// New schema being upgraded to
        new_schema: Schema,
        /// Result of the evolution process
        evolution_result: EvolutionResult,
    },
    
    /// Perform an operation on a register
    RegisterOp {
        /// ID of the register to operate on
        register_id: RegisterId,
        /// Operation to perform
        operation: RegisterOperation,
        /// Authorization method
        auth_method: AuthorizationMethod,
    },
    
    /// Create a new register
    RegisterCreate {
        /// Owner of the new register
        owner: Address,
        /// Contents of the new register
        contents: RegisterContents,
    },
    
    /// Transfer a register to another domain
    RegisterTransfer {
        /// Source register ID
        source_reg_id: RegisterId,
        /// Target domain
        target_domain: DomainId,
        /// Controller label for the transfer
        controller_label: ControllerLabel,
    },
    
    /// Custom effect with arbitrary data
    CustomEffect {
        /// Effect type name
        name: String,
        /// Effect data
        data: serde_json::Value,
    },
}
```

## Effect Adapter 

```rust
/// A definition of an effect for an adapter schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectDefinition {
    /// Type of effect (e.g., "eth_transfer", "http_request")
    pub effect_type: String,
    
    /// Transaction format description (how to serialize the transaction)
    pub tx_format: String,
    
    /// Proof format description (how to verify proofs)
    pub proof_format: String,
    
    /// RPC call specification (if applicable)
    pub rpc_call: Option<String>,
    
    /// Required fields for this effect
    pub required_fields: Vec<String>,
    
    /// Optional fields for this effect
    pub optional_fields: Vec<String>,
    
    /// Mapping of fields to domain-specific fields
    pub field_mappings: HashMap<String, String>,
    
    /// Serialization format for the effect
    pub serialization: String,
    
    /// Gas/fee estimation function (if applicable)
    pub gas_estimation: Option<String>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// A definition of a fact for an adapter schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactDefinition {
    /// Type of fact (e.g., "eth_block", "http_response")
    pub fact_type: String,
    
    /// Data format description
    pub data_format: String,
    
    /// How to derive a fact from domain-specific data
    pub derivation: String,
    
    /// Required fields in this fact
    pub required_fields: Vec<String>,
    
    /// Field transformations (if any)
    pub transformations: HashMap<String, String>,
    
    /// Time field specification (for time ordering)
    pub time_field: Option<String>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Adapter schema defining a domain adapter's capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterSchema {
    /// Schema name
    pub name: String,
    
    /// Target domain this adapter connects to
    pub domain: String,
    
    /// Schema version
    pub version: String,
    
    /// Supported effects
    pub effects: Vec<EffectDefinition>,
    
    /// Supported facts
    pub facts: Vec<FactDefinition>,
    
    /// Supported proofs
    pub proofs: Vec<ProofDefinition>,
    
    /// RPC interface specification
    pub rpc_interface: String,
    
    /// Time synchronization settings
    pub time_sync: Option<TimeSyncDefinition>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}
```

## Register

```rust
/// Register structure for resource management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Register {
    /// Unique identifier for this register
    pub register_id: RegisterId,
    
    /// Owner address of this register
    pub owner: Address,
    
    /// Contents stored in this register
    pub contents: RegisterContents,
    
    /// Last update block height
    pub last_updated: BlockHeight,
    
    /// Metadata for this register
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Optional controller label
    pub controller_label: Option<ControllerLabel>,
}

/// Contents that can be stored in a register
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegisterContents {
    /// Resource data
    Resource(Resource),
    
    /// Token balance
    TokenBalance {
        /// Token type
        token_type: TokenType,
        /// Owner address
        address: Address,
        /// Balance amount
        amount: Amount,
    },
    
    /// Non-fungible token content
    NFTContent {
        /// Collection address
        collection_address: CollectionAddress,
        /// Token ID
        token_id: TokenId,
    },
    
    /// State commitment
    StateCommitment {
        /// Type of commitment
        commitment_type: CommitmentType,
        /// Commitment data
        data: Vec<u8>,
    },
    
    /// Time map commitment
    TimeMapCommitment {
        /// Block height
        block_height: BlockHeight,
        /// Commitment data
        data: Vec<u8>,
    },
    
    /// Generic data object
    DataObject {
        /// Format of the data
        data_format: DataFormat,
        /// Object data
        data: Vec<u8>,
    },
    
    /// Effect DAG
    EffectDAG {
        /// Effect ID
        effect_id: EffectId,
        /// DAG data
        data: Vec<u8>,
    },
    
    /// Resource nullifier
    ResourceNullifier {
        /// Nullifier key
        nullifier_key: NullifierKey,
        /// Nullifier data
        data: Vec<u8>,
    },
    
    /// Resource commitment
    ResourceCommitment {
        /// Commitment key
        commitment_key: CommitmentKey,
        /// Commitment data
        data: Vec<u8>,
    },
    
    /// Composite contents
    CompositeContents(Vec<RegisterContents>),
}
```

## Content-Addressed Code

```rust
/// A code definition with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDefinition {
    /// The content hash of this definition
    pub hash: ContentHash,
    /// The human-readable name (if any)
    pub name: Option<String>,
    /// The actual code representation (AST or bytecode)
    pub content: CodeContent,
    /// Dependencies of this code definition
    pub dependencies: Vec<ContentHash>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Represents the content of a code definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodeContent {
    /// Raw bytes
    Bytes(Vec<u8>),
    /// JSON-encoded data
    Json(String),
    /// RISC-V binary
    RiscV(Vec<u8>),
    /// Abstract Syntax Tree (AST) representation
    Ast(serde_json::Value),
    /// WebAssembly binary
    Wasm(Vec<u8>),
}

/// Content-addressable effect
pub struct ContentAddressedEffect {
    /// The content hash of the effect
    pub hash: CodeHash,
    /// The effect type/name
    pub effect_type: String,
    /// Parameter schema for the effect
    pub parameter_schema: HashMap<String, String>,
    /// Result type schema
    pub result_schema: String,
    /// Is this effect pure (no side effects)
    pub is_pure: bool,
    /// Resource requirements for this effect
    pub resource_requirements: EffectResourceRequirements,
    /// Fact dependencies for this effect
    pub fact_dependencies: Vec<FactDependency>,
    /// Fact snapshot this effect depends on
    pub fact_snapshot: Option<FactSnapshot>,
}
```

## Authorization Method

```rust
/// Methods for authorizing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorizationMethod {
    /// Zero-knowledge proof authorization
    ZKProofAuthorization {
        /// Verification key
        verification_key: VerificationKey,
        /// Proof data
        proof: Proof,
    },
    
    /// Token ownership authorization
    TokenOwnershipAuthorization {
        /// Token address
        token_address: TokenAddress,
        /// Amount owned
        amount: Amount,
    },
    
    /// NFT ownership authorization
    NFTOwnershipAuthorization {
        /// Collection address
        collection_address: CollectionAddress,
        /// Token ID
        token_id: TokenId,
    },
    
    /// Multi-signature authorization
    MultiSigAuthorization {
        /// Signer addresses
        addresses: Vec<Address>,
        /// Threshold of required signatures
        threshold: u32,
        /// Signatures
        signatures: Vec<Signature>,
    },
    
    /// DAO-based authorization
    DAOAuthorization {
        /// DAO address
        dao_address: DAOAddress,
        /// Proposal ID
        proposal_id: ProposalId,
    },
    
    /// Timelock authorization
    TimelockAuthorization {
        /// Authorized address
        address: Address,
        /// Unlock timestamp
        timestamp: Timestamp,
    },
    
    /// Composite authorization
    CompositeAuthorization {
        /// Authorization methods
        methods: Vec<AuthorizationMethod>,
        /// How to combine the methods
        combinator: AuthCombinator,
    },
}
```

## Register Operation

```rust
/// Operation on a register
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterOperation {
    /// Type of operation
    pub op_type: OperationType,
    
    /// Registers involved in the operation
    pub registers: Vec<RegisterId>,
    
    /// New contents (if applicable)
    pub new_contents: Option<RegisterContents>,
    
    /// Authorization for this operation
    pub authorization: Authorization,
    
    /// Proof for this operation
    pub proof: Proof,
    
    /// Resource delta from this operation
    pub resource_delta: Delta,
}
```

## Program State

```rust
/// State of a program in the Causality system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramState {
    /// Program schema
    pub schema: Schema,
    
    /// Safe state policy
    pub safe_state_policy: SafeStatePolicy,
    
    /// Effect DAG for this program
    pub effect_dag: EffectDAG,
    
    /// Fact snapshots keyed by effect ID
    pub fact_snapshots: HashMap<EffectId, FactSnapshot>,
    
    /// Managed registers with their capabilities
    pub managed_registers: HashMap<RegisterId, RegisterCapabilities>,
}
```

## Account Program State

```rust
/// State of an account program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountProgramState {
    /// Token balances by domain and asset
    pub balances: HashMap<(DomainId, Asset), Amount>,
    
    /// Effect DAG for this account
    pub effect_dag: EffectDAG,
    
    /// Managed registers with their capabilities
    pub managed_registers: HashMap<RegisterId, RegisterCapabilities>,
    
    /// ZK capabilities by circuit type
    pub zk_capabilities: HashMap<CircuitType, VerificationKey>,
    
    /// Time map commitment
    pub time_map_commitment: TimeMapCommitment,
    
    /// Execution sequences by sequence ID
    pub execution_sequences: HashMap<SequenceId, ExecutionStatus>,
}
```

## Execution Sequence

```rust
/// A sequence of execution steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSequence {
    /// Unique identifier for this sequence
    pub sequence_id: SequenceId,
    
    /// Execution nodes by node ID
    pub nodes: HashMap<NodeId, ExecutionNode>,
    
    /// Edges between nodes
    pub edges: Vec<Edge>,
    
    /// Entry point nodes
    pub entry_points: Vec<NodeId>,
    
    /// Exit point nodes
    pub exit_points: Vec<NodeId>,
    
    /// Commitment for this sequence
    pub commitment: Vec<u8>,
}

/// Node in an execution sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionNode {
    /// Unique identifier for this node
    pub node_id: NodeId,
    
    /// Type of node
    pub node_type: NodeType,
    
    /// Operation to execute
    pub operation: Operation,
    
    /// Register dependencies
    pub register_dependencies: Vec<RegisterId>,
    
    /// Completion proof (if available)
    pub completion_proof: Option<Proof>,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}
```

---

# Effect Pipeline

- Effects are **causally ordered** — each effect references parents in the effect DAG.
- Every effect depends on a **FactSnapshot** — facts observed before the effect applied.
- Effects are **content-addressed** — their hash becomes part of the DAG.
- Effects are gossiped across the Operator network before being finalized.
- Register operations are validated with ZK proofs and incorporated into the effect DAG.

---

# Effect Adapter System

The Effect Adapter System provides the boundary between Causality and external domains:

- **Domain Bridging**: Adapters translate between abstract effects and domain-specific operations.
- **Schema-Driven Generation**: Adapter implementations are generated from declarative schemas.
- **Content-Addressed Deployment**: Adapters are content-addressed for immutable, verifiable deployment.
- **Cross-Domain Consistency**: Standardized interfaces ensure consistent behavior across domains.
- **Fact Standardization**: External data is normalized into canonical fact formats.

## Adapter Lifecycle

1. **Schema Definition**: Domain experts define adapter schemas specifying supported effects and facts.
2. **Code Generation**: The system generates adapter code from schemas in the target language.
3. **Deployment**: Generated adapters are content-addressed and deployed to the adapter registry.
4. **Runtime Binding**: Effects are dynamically bound to appropriate adapters at execution time.
5. **Validation**: All adapter operations undergo runtime validation against their schema.

## Adapter Composition

Adapters can be composed to create complex cross-domain operations:

- **Sequential Composition**: Chain multiple adapters to form multi-step processes.
- **Parallel Composition**: Execute independent adapter operations concurrently.
- **Conditional Composition**: Select adapters based on runtime conditions.
- **Recursive Composition**: Adapters can invoke other adapters for nested operations.

---

# Unified Log

Each program and account program maintains an **append-only, content-addressed log**, recording:

- Applied effects.
- Observed facts.
- Lifecycle events (e.g., schema upgrade, safe state transition).
- Register operations and ZK proof verifications.
- Execution sequence progress and completions.

Each log segment is content-addressed, enabling efficient distribution and verification across the network.

---

# Content-Addressed Code System

The content-addressed code system provides immutable, verifiable program representation:

- **Code Repository**: Stores code entries with their content hash as the identifier.
- **Name Registry**: Maps human-readable names to content hashes for easy reference.
- **Compatibility Checker**: Verifies code compatibility with execution environments.
- **Executor**: Runs content-addressed code in secure, isolated environments.

## Code Execution

1. **Resolution**: Code is referenced by hash or name, then resolved to a content hash.
2. **Compatibility Check**: The system verifies that code is compatible with the runtime.
3. **Sandbox Creation**: A secure execution environment is created with specified constraints.
4. **Execution**: Code is executed with provided arguments and context.
5. **Effect Application**: Any generated effects are validated and applied.

---

# Time System

The time system ensures causal consistency across multiple domains:

- **Lamport Clocks**: Track causal relationships between events.
- **Time Windows**: Define temporal ranges for operations.
- **Time Synchronization**: Map timestamps across heterogeneous domains.
- **Time Dual**: Represent both total and partial ordering of events.

---

# System Implementation Components

## Core Implementation

- **Effect System**: Algebraic effects framework for composable operations
- **Time Module**: Unified representation of time across domains
- **ResourceRegister System**: Unified resource and register management with zero-knowledge verification
- **Fact System**: Standardized blockchain state representation
- **Log System**: Append-only, content-addressed event logs

## Domain Integration

- **Effect Adapters**: Bridge between abstract effects and domain-specific implementations
- **Fact Observers**: Extract standardized facts from external domains
- **Cross-Domain Resource Manager**: Manages resource lifecycle and transfers across domains
- **Domain Connectors**: Network interfaces to external blockchain and API endpoints

## Verification System

- **ZK Verifier**: Verification of zero-knowledge proofs for operations
- **Proof Generator**: Generation of proofs for resource operations
- **Register Lifecycle Management**: Secure lifecycle management with proof verification
- **Content-Addressed Operations**: Content-addressed operation model for verifiable execution

---

# System Architecture

The Causality system is structured into the following architectural layers:

## Interface Layer

- **Temporal Effect Language (TEL)**: DSL for time-bound effects and causal dependencies
- **ResourceRegister API**: Unified API for resource and register operations
- **CLI Tools**: Command-line tools for system interaction
- **API Endpoints**: RESTful and GraphQL interfaces

## Execution Layer

- **Effect Executor**: Applies effects across domains
- **ResourceRegister Manager**: Controls access to unified resource registers
- **Operation Pipeline**: Processes operations through abstract, register, and physical contexts
- **Content-Addressable Executor**: Runs code by hash or name

## Storage Layer

- **Unified Log**: Append-only log of all system events
- **Content-Addressed Storage**: Immutable storage for code and data
- **ResourceRegister State**: Current state of all system resource registers
- **Snapshot System**: Provides point-in-time snapshots of register state

## Integration Layer

- **Domain Adapters**: Connect to external blockchains and systems
- **Cross-Domain Operations**: Execute operations spanning multiple domains
- **P2P Network**: Disseminate facts and effects across the network
- **Committee Coordinator**: Coordinate committee activities

## Adapter Error Types

```rust
/// Effect adapter error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdapterError {
    /// Communication error with external Domain
    CommunicationError(String),
    /// Invalid transaction format
    InvalidTransactionFormat(String),
    /// Insufficient funds or resources
    InsufficientFunds(String),
    /// Unauthorized operation
    Unauthorized(String),
    /// External Domain is unavailable
    DomainUnavailable(String),
    /// Transaction rejected by external Domain
    TransactionRejected(String),
    /// Unsupported operation
    UnsupportedOperation(String),
    /// Other errors
    Other(String),
}
```

