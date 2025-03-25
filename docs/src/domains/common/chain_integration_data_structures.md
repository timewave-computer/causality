<!-- Data structures for chain integration -->
<!-- Original file: docs/src/chain_integration_data_structures.md -->

# Core Data Structures for Blockchain Integration

When integrating this capability-based resource system into a new blockchain, you'll need to implement several foundational data structures. Here's the complete set with descriptions of their behavior and system interactions:

## 1. Commitment

```rust
struct Commitment {
    hash: [u8; 32],         // Cryptographic commitment hash
    metadata: Option<Bytes> // Optional public metadata
}
```

**Description**: The fundamental unit of program state. A commitment represents the existence of a resource without revealing its contents.

**Behavior**: Immutable once created. Can be verified against a witness without revealing the committed value.

**System Interaction**: Forms the basis of the on-chain state. Commitments are stored in the commitment tree and can be referenced by programs when constructing proofs. The hash is generated using a Pedersen or similar commitment scheme that supports zero-knowledge proofs.

## 2. CommitmentTree

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

## 3. Nullifier

```rust
struct Nullifier {
    hash: [u8; 32],      // Cryptographic nullifier hash
    block_created: u64   // Block when nullifier was added
}
```

**Description**: Represents a commitment that has been "spent" or used as an input to an operation.

**Behavior**: Once a nullifier is recorded, any future operation attempting to use the same nullifier will fail. This prevents double-spending.

**System Interaction**: When an operation consumes a commitment, it generates and reveals a nullifier. The system checks this nullifier against the existing set before accepting the operation. Nullifiers can be derived deterministically from commitments but cannot be linked back to their source commitment without the private witness data.

## 4. NullifierSet

```rust
struct NullifierSet {
    nullifiers: HashSet<[u8; 32]>,  // Set of active nullifiers
    index: HashMap<[u8; 32], u64>   // Maps nullifier to block created
}
```

**Description**: Tracks all nullifiers that have been published to the blockchain.

**Behavior**: Append-only set that allows efficient checking for existence. Once added, nullifiers remain forever.

**System Interaction**: Before accepting any operation, the system verifies that none of the nullifiers in the operation already exist in this set. This is the primary mechanism that prevents double-spending of commitments.

## 5. ZKProof

```rust
struct ZKProof {
    proof_data: Bytes,            // The actual proof bytes
    public_inputs: Vec<Bytes>,    // Publicly revealed inputs
    verification_key_id: String   // Identifier for verification key 
}
```

**Description**: A zero-knowledge proof attesting that an operation is valid without revealing the private inputs.

**Behavior**: Cryptographically verifiable evidence that a computation was performed correctly. The proof reveals nothing about the private inputs beyond what is explicitly shared in public_inputs.

**System Interaction**: When a user wishes to perform an operation, they generate a proof off-chain and submit it. The blockchain verifies the proof against the specified verification key before accepting the operation. Proofs are never stored long-term, only verified at transaction time.

## 6. CircuitVerificationKey

```rust
struct CircuitVerificationKey {
    id: String,                   // Unique identifier
    key_data: Bytes,              // Actual key bytes
    circuit_hash: [u8; 32],       // Hash of the circuit
    parameters: HashMap<String, Value>  // Circuit parameters
}
```

**Description**: The public parameters needed to verify a zero-knowledge proof for a specific circuit.

**Behavior**: Immutable once created. Associated with a specific circuit implementation.

**System Interaction**: The blockchain maintains a registry of verification keys for approved circuits. When a proof is submitted, the system uses the specified verification key to validate it. New keys can be added through governance mechanisms when new circuit types are introduced.

## 7. Register

```rust
struct Register {
    id: [u8; 32],            // Unique register identifier
    commitment: [u8; 32],    // Commitment to register contents
    type_id: u16,            // Type of register (token, NFT, etc)
    status: RegisterStatus,  // Active, nullified, expired
    created_at: u64,         // Block created
    nullified_at: Option<u64> // Block nullified, if applicable
}
```

**Description**: The fundamental unit of state in the system. Represents an atomic piece of state that follows a one-time-use model.

**Behavior**: Once created, a register is active until nullified. It can only be nullified once. Some registers are linked to commitments for privacy, while others might be public.

**System Interaction**: Programs create and nullify registers through operations. Each register transition generates an event that can be observed by the system. Register creation and nullification are the building blocks of all state transitions.

## 8. ResourceDefinition

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

## 9. Operation

```rust
struct Operation {
    type_id: u16,            // Operation type
    inputs: Vec<Input>,      // Inputs (nullifiers, references)
    outputs: Vec<Output>,    // Outputs (new commitments)
    proof: Option<ZKProof>,  // Optional ZK proof
    metadata: Option<Bytes>  // Optional public metadata
}
```

**Description**: Represents a state transition in the system. The fundamental unit of behavior.

**Behavior**: Atomic - either fully succeeds or fails. Operations consume inputs (nullifying them) and produce outputs (creating new commitments or registers).

**System Interaction**: Users and programs construct operations and submit them to the blockchain. The system verifies the operation's validity (checking nullifiers, verifying proofs) before applying it to the state. Each successful operation produces a transaction receipt.

## 10. CapabilityGrant

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

## 11. Program

```rust
struct Program {
    id: [u8; 32],                  // Content-addressed program ID
    code: Bytes,                   // Program bytecode
    state_commitment: [u8; 32],    // Commitment to program state
    version: u32,                  // Program version
    owner: [u8; 32],               // Program owner
    capabilities: Vec<CapabilityGrant> // Capabilities granted to this program
}
```

**Description**: The executable logic that defines how resources can be manipulated.

**Behavior**: Programs are immutable once deployed (content-addressed). They can be upgraded by deploying a new version and updating references.

**System Interaction**: Programs process operations, generating nullifiers and new commitments. They can request capabilities from resources and act as capability grantors themselves. The system executes program code in response to operations, either on-chain or through verified off-chain execution.

## 12. ProgramAccount

```rust
struct ProgramAccount {
    id: [u8; 32],                  // Account identifier
    owner: [u8; 32],               // Account owner
    program_id: [u8; 32],          // Program controlling this account
    public_state: Option<Bytes>,   // Optional public state
    state_commitment: [u8; 32],    // Commitment to full state
    capabilities: Vec<CapabilityGrant> // Capabilities granted to this account
}
```

**Description**: The boundary object that represents a user's interaction with programs and resources.

**Behavior**: Presents a view of resources to users and mediates access to those resources. On-chain representation is minimal, while client-side can be rich.

**System Interaction**: Users interact with the system through program accounts. Accounts control access to resources through capabilities and present a consistent interface to users. Program accounts can invoke operations and hold capabilities on behalf of their owners.

## 13. Witness

```rust
struct Witness {
    commitment: [u8; 32],     // Commitment this witness relates to
    preimage: Bytes,          // The actual committed value (private)
    path: MerklePath,         // Inclusion proof in commitment tree
    nullifier_key: [u8; 32]   // Key for deriving nullifier
}
```

**Description**: Private data that allows proving knowledge of a commitment's contents without revealing them.

**Behavior**: Never stored on-chain. Used to generate nullifiers and construct zero-knowledge proofs.

**System Interaction**: Users maintain witnesses client-side and use them when constructing operations. Witnesses are essential for privacy-preserving operations as they allow users to prove ownership without revealing the commitment contents.

## 14. Event

```rust
struct Event {
    type_id: u16,             // Event type
    emitter: [u8; 32],        // Entity that emitted the event
    data: Bytes,              // Event data
    topics: Vec<[u8; 32]>,    // Indexed topics for filtering
    block_number: u64,        // Block this event was emitted in
    tx_index: u32             // Transaction index in the block
}
```

**Description**: Notifications emitted by programs, operations, or the system itself.

**Behavior**: Append-only, immutable records of activity. Can be filtered and subscribed to.

**System Interaction**: Programs emit events when their state changes. Events are stored in an indexed log that can be queried by clients. They serve as a notification mechanism and are used to update client-side state.

## 15. TransactionReceipt

```rust
struct TransactionReceipt {
    tx_hash: [u8; 32],           // Transaction hash
    block_number: u64,           // Block number
    status: TransactionStatus,   // Success, failure, etc.
    operations: Vec<Operation>,  // Operations in this transaction
    events: Vec<Event>,          // Events emitted
    new_commitments: Vec<[u8; 32]>, // Commitments created
    nullifiers: Vec<[u8; 32]>    // Nullifiers published
}
```

**Description**: Proof that a transaction was processed, including its results.

**Behavior**: Immutable record of transaction execution. Contains enough information to understand what happened without revealing private data.

**System Interaction**: Generated when a transaction is executed. Clients can query receipts to confirm transaction status and update their local state accordingly. Receipts can be used to construct witnesses for subsequent operations.

## 16. TimeMap

```rust
struct TimeMap {
    height: u64,                          // Current block height
    timestamp: u64,                       // Block timestamp
    external_timelines: HashMap<String, ExternalTimeline> // State of external chains
}
```

**Description**: Cross-chain state observation mechanism that ensures consistent views of external timelines.

**Behavior**: Updated with each block to reflect the latest observed state of external chains.

**System Interaction**: Used by cross-chain operations to ensure a consistent view of external state. Operations reference the time map to prove they're acting on the latest information. The time map is part of the block header, making it part of consensus.

## 17. ExternalTimeline

```rust
struct ExternalTimeline {
    chain_id: String,           // External chain identifier
    height: u64,                // Last observed block height
    state_root: [u8; 32],       // Last observed state root
    timestamp: u64,             // Last observed timestamp
    status: TimelineStatus,     // Active, stale, etc.
    last_updated: u64           // When this data was last updated
}
```

**Description**: Represents the observed state of an external blockchain.

**Behavior**: Updated by time keepers when new external blocks are observed. Contains cryptographic commitments to external chain state.

**System Interaction**: Cross-chain operations reference this data to prove they're acting on the latest external state. The system uses this to validate cross-chain proofs and ensure consistency across chains.

## 18. EffectLog

```rust
struct EffectLog {
    effects: Vec<Effect>,            // Ordered list of effects
    last_timestamp: u64,             // Timestamp of last effect
    last_block: u64,                 // Block of last effect
    resource_id: [u8; 32]            // Resource this log belongs to
}
```

**Description**: History of all effects applied to a specific resource.

**Behavior**: Append-only log of causally related effects. Each effect references its predecessors.

**System Interaction**: When programs apply effects to resources, these are recorded in the effect log. The log ensures causal consistency and allows for deterministic replay. Programs can query the log to understand resource history.

## 19. UnifiedLog

```rust
struct UnifiedLog {
    entries: Vec<LogEntry>,        // Log entries
    content_roots: HashMap<u64, [u8; 32]>, // Root hashes by block
    indices: HashMap<String, Vec<u64>> // Indices for efficient lookup
}
```

**Description**: Comprehensive record of all system activity, including effects, facts, and events.

**Behavior**: Append-only, content-addressed log that serves as the system's memory.

**System Interaction**: All actors write to the unified log. It serves as the foundation for replay, audit, and verification. The log is distributed across nodes and can be reconstructed from consensus state.

## 20. FactSnapshot

```rust
struct FactSnapshot {
    observed_facts: Vec<[u8; 32]>,  // Fact IDs observed
    time_map: [u8; 32],             // Hash of time map at observation
    observer: [u8; 32],             // Entity that observed these facts
    timestamp: u64                  // When these facts were observed
}
```

**Description**: Record of external facts that an effect or operation depended on.

**Behavior**: Immutable reference to external state at a specific point in time.

**System Interaction**: Every effect that depends on external state includes a fact snapshot. This ensures effects can be replayed deterministically and verified against the state they observed.

---

These core data structures form the complete system for a blockchain integration. All state transitions and cross-chain interactions can be built using these primitives. 

The most critical design aspects are:
1. The commitment/nullifier pattern for eventual privacy-preserving state
2. The capability model for secure resource access
3. The register model for atomic state transitions
4. The unified logging system for causal consistency and replay

When implementing these on a new blockchain, you'll need to decide which structures live directly in the state trie, which are implemented as specialized data structures, and which are represented implicitly through the execution model.


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


