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

```haskell
data Effect
    = Deposit { Domain :: DomainID, asset :: Asset, amount :: Amount }
    | Withdraw { Domain :: DomainID, asset :: Asset, amount :: Amount }
    | Transfer { fromProgram :: ProgramID, toProgram :: ProgramID, asset :: Asset, amount :: Amount }
    | ObserveFact { factID :: FactID }
    | Invoke { targetProgram :: ProgramID, invocation :: Invocation }
    | EvolveSchema { oldSchema :: Schema, newSchema :: Schema, evolutionResult :: EvolutionResult }
    | RegisterOp { registerID :: RegisterID, operation :: RegisterOperation, authMethod :: AuthorizationMethod }
    | RegisterCreate { owner :: Address, contents :: RegisterContents }
    | RegisterTransfer { sourceRegID :: RegisterID, targetDomain :: DomainID, controllerLabel :: ControllerLabel }
    | CustomEffect Text Value
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

```haskell
data Register = Register
    { registerId :: RegisterID
    , owner :: Address
    , contents :: RegisterContents
    , lastUpdated :: BlockHeight
    , metadata :: Map Text Value
    , controllerLabel :: Maybe ControllerLabel
    }

data RegisterContents 
    = Resource Resource
    | TokenBalance TokenType Address Amount
    | NFTContent CollectionAddress TokenId
    | StateCommitment CommitmentType ByteString
    | TimeMapCommitment BlockHeight ByteString
    | DataObject DataFormat ByteString
    | EffectDAG EffectID ByteString
    | ResourceNullifier NullifierKey ByteString
    | ResourceCommitment CommitmentKey ByteString
    | CompositeContents [RegisterContents]
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

```haskell
data AuthorizationMethod
    = ZKProofAuthorization VerificationKey Proof
    | TokenOwnershipAuthorization TokenAddress Amount
    | NFTOwnershipAuthorization CollectionAddress TokenId
    | MultiSigAuthorization [Address] Int [Signature]
    | DAOAuthorization DAOAddress ProposalId
    | TimelockAuthorization Address Timestamp
    | CompositeAuthorization [AuthorizationMethod] AuthCombinator
```

## Register Operation

```haskell
data RegisterOperation = RegisterOperation
    { opType :: OperationType
    , registers :: [RegisterID]
    , newContents :: Maybe RegisterContents
    , authorization :: Authorization
    , proof :: Proof
    , resourceDelta :: Delta
    }
```

## FactSnapshot (causal dependency record)

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

## Program State

```haskell
data ProgramState = ProgramState
    { schema :: Schema
    , safeStatePolicy :: SafeStatePolicy
    , effectDAG :: EffectDAG
    , factSnapshots :: Map EffectID FactSnapshot
    , managedRegisters :: Map RegisterID RegisterCapabilities
    }
```

## Account Program State

```haskell
data AccountProgramState = AccountProgramState
    { balances :: Map (DomainID, Asset) Amount
    , effectDAG :: EffectDAG
    , managedRegisters :: Map RegisterID RegisterCapabilities
    , zkCapabilities :: Map CircuitType VerificationKey
    , timeMapCommitment :: TimeMapCommitment
    , executionSequences :: Map SequenceID ExecutionStatus
    }
```

## Execution Sequence

```haskell
data ExecutionSequence = ExecutionSequence
    { sequenceId :: SequenceID
    , nodes :: Map NodeID ExecutionNode
    , edges :: [Edge]
    , entryPoints :: [NodeID]
    , exitPoints :: [NodeID]
    , commitment :: ByteString
    }

data ExecutionNode = ExecutionNode
    { nodeId :: NodeID
    , nodeType :: NodeType
    , operation :: Operation
    , registerDependencies :: [RegisterID]
    , completionProof :: Maybe Proof
    , metadata :: Map Text Value
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
- **Register System**: Secure resource management with ZK verification
- **Fact System**: Standardized blockchain state representation
- **Log System**: Append-only, content-addressed event logs

## Domain Integration

- **Effect Adapters**: Bridge between abstract effects and domain-specific implementations
- **Fact Observers**: Extract standardized facts from external domains
- **Time Maps**: Maintain temporal relationship between different domain timelines
- **Domain Connectors**: Network interfaces to external blockchain and API endpoints

## Verification System

- **RISC-V Compilation**: Translate high-level effects to RISC-V instructions
- **ZK-VM**: Virtual machine optimized for zero-knowledge proof generation
- **Proof Verification**: On-chain and off-chain verification of computational proofs
- **Circuit Optimization**: Automatic optimization for minimal proving time

---

# System Architecture

The Causality system is structured into the following architectural layers:

## Interface Layer

- **Temporal Effect Language (TEL)**: DSL for time-bound effects and causal dependencies
- **Program Account UI**: Serializable views for frontend integration
- **CLI Tools**: Command-line tools for system interaction
- **API Endpoints**: RESTful and GraphQL interfaces

## Execution Layer

- **Effect Executor**: Applies effects across domains
- **Resource Manager**: Controls access to named resources
- **Register Controller**: Manages register state and operations
- **Content-Addressable Executor**: Runs code by hash or name

## Storage Layer

- **Unified Log**: Append-only log of all system events
- **Content-Addressed Storage**: Immutable storage for code and data
- **Register State**: Current state of all system registers
- **Time Map Storage**: Persistent storage of time mappings

## Integration Layer

- **Domain Adapters**: Connect to external blockchains and systems
- **Fact Observers**: Extract standardized facts from domains
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

