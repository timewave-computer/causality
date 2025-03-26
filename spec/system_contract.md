# Causality System Contract

---

## Version

**Current Revision:** 2025-03-26

---

## Purpose

This document defines the **system contract** for Causality, establishing the fundamental invariants, roles, ownership rules, and guarantees that underpin the system. It serves as both a specification and a **social/legal contract** between participants — Users, Operators, and the broader execution network.

This document reflects the **latest design**, incorporating:
- Account programs for resource ownership.
- Unified log for facts, effects, and events.
- Fact observation pipeline.
- Safe state and schema evolution rules.
- Updated concurrency and invocation models.
- Universal content addressing system.
- Cryptographic verification protocols.
- Cross-domain verification guarantees.
- Deferred hashing optimization.
- Role separation between agents.
- Three-layer algebraic effect architecture.
- Resource-scoped concurrency model.

---

# Core Invariants

The following invariants **must always hold**, across all modes of execution (in-memory, multi-process, geo-distributed), across all simulation and production deployments:

1. **Programs do not own resources directly.**
    - All external resources (tokens, balances, positions) are owned by **account programs**, not by individual programs.

2. **Every effect has a complete causal Domain.**
    - Every effect must reference the exact prior facts and effects it depended on.
    - The full causal graph is append-only and content-addressed.

3. **Fact observations are first-class.**
    - Programs cannot act on external state unless that state has been observed and proven by a Committee.
    - Facts are **immutable, content-addressed, and independently provable**.

4. **Programs cannot be forcibly upgraded.**
    - Users (program owners) must explicitly approve schema and logic upgrades.
    - Programs can only evolve schemas while in **safe states**.

5. **Replay is complete and deterministic.**
    - All program state, resource flows, facts, and effects must be fully reconstructible from logs alone — no external state should be required.

6. **Programs remain Domain-agnostic.**
    - Programs do not need Domain-specific logic. All cross-domain interaction is mediated via account programs and fact observations.

7. **All state objects are content-addressed.**
    - Every stateful object is uniquely identified by a cryptographic hash of its content.
    - Content hashes replace UUIDs and are guaranteed to be collision-resistant.
    - Content addressing enables verification, deduplication, and tamper resistance.

8. **Cryptographic verification is universal.**
    - All critical operations undergo cryptographic verification.
    - Verification failures result in explicit errors, not silent failures.
    - Cross-domain operations are verified using content addressing.

9. **Content normalization guarantees deterministic hashing.**
    - All serialized content is normalized before hashing to ensure deterministic results.
    - Map/dictionary field ordering is deterministic across all implementations.
    - Binary representations use canonical formats for consistent hashing.

10. **Effects are uniformly handled across the system.**
    - All operations, whether internal or external, are modeled as explicit effects.
    - Effects are always subject to the same validation, authorization, and composition rules.
    - All effects can be content-addressed, logged, and verified.

11. **Resource concurrency is explicit and deterministic.**
    - All access to concurrent resources is controlled through explicit locks.
    - Resource wait queues are deterministic and follow consistent ordering.
    - Resource guards ensure proper release through RAII patterns.

---

# Core Agents

Agents are specialized resources that hold capabilities and perform operations within the system. All agents use content-addressing for identification and are integrated with the resource system.

## Users

- Own programs.
- Deploy new programs.
- Propose schema and logic upgrades.
- Maintain full sovereignty over programs — no forced upgrades.
- Submit external deposits into account programs.
- Own account programs that hold and transfer assets.
- Validate content addressing of critical operations.

## Committees

- One per Domain.
- Observe external facts (balances, prices, inclusion proofs).
- Sign observation proofs.
- Append facts to **FactLog**.
- Validate external messages before accepting into a Domain.
- Respond to fact queries from programs and Operators.
- Manage per-Domain clocks.
- Generate content-addressed fact records.
- Verify cross-domain content hashes.
- Provide time attestations with appropriate trust levels.

## Operators

- Operate the **execution network**.
- Execute program logic and generate **zero-knowledge proofs** of execution.
- Gossip facts and effects across the network.
- Maintain a content-addressed, append-only log of all:
    - Effects.
    - Facts.
    - Events.
- Enforce **safe state rules** before accepting upgrades.
- Enforce **schema compatibility** before running programs.
- Synchronize across Domains to enforce cross-domain invariants.
- Verify content hashes for all operations.
- Manage content-addressed storage.
- Optimize content hashing through deferred techniques.
- Process time effects and maintain time state.

---

# Core Programs

## Program (Logic Program)

- Defines **causal effect pipeline** (business logic).
- Declares:
    - Schema (state structure).
    - Protocol compatibility version.
    - Evolution rules (what schema changes are allowed).
- Does **not own resources directly** — interacts with resources via account programs.
- Includes:
    - Effect DAG (causal history).
    - FactSnapshots (external dependencies).
    - Current schema version.
    - Declared safe state policy.
    - Content addressing policies.
    - Verification requirements.
    - Time effect dependencies.

## Account Program

- Each User has one account program per deployment context.
- Holds all User's cross-domain balances.
- Exposes:
    - Deposit API (Domain-specific).
    - Withdraw API (Domain-specific).
    - Transfer API (to programs).
    - Query API (balance reporting).
- Maintains its own **Effect DAG** (resource history).
- Separately replayable from logic programs.
- Schema is **stable and standard** across all account programs.
- Implements content addressing for all operations.
- Verifies cryptographic proofs for cross-domain operations.

---

# Core Data Structures

## Effect

```rust
/// Effect interface defining common operations
pub trait Effect<R>: ContentAddressed {
    /// Execute the effect with the given handler
    fn execute(self, handler: &dyn EffectHandler) -> EffectOutcome<R>;
    
    /// Get the effect's unique identifier
    fn effect_id(&self) -> EffectId;
    
    /// Get the resources this effect requires
    fn resources(&self) -> Vec<ResourceId>;
    
    /// Get the capabilities required for this effect
    fn required_capabilities(&self) -> Vec<Capability>;
    
    /// Compose with another effect
    fn and_then<U, F>(self, f: F) -> ComposedEffect<Self, F, R, U>
    where
        F: FnOnce(R) -> Box<dyn Effect<U>>,
        Self: Sized;
}

/// Core effect enum - system-wide effects
pub enum CoreEffect<R> {
    // External effects
    Deposit {
        domain: DomainId,
        asset: Asset,
        amount: Amount,
        continuation: Box<dyn Continuation<DepositResult, R>>,
    },
    Withdraw {
        domain: DomainId,
        asset: Asset,
        amount: Amount,
        continuation: Box<dyn Continuation<WithdrawResult, R>>,
    },
    Transfer {
        from_program: ProgramId,
        to_program: ProgramId,
        asset: Asset,
        amount: Amount,
        continuation: Box<dyn Continuation<TransferResult, R>>,
    },
    
    // Fact observation effects
    ObserveFact {
        fact_id: FactId,
        continuation: Box<dyn Continuation<ObservationResult, R>>,
    },
    
    // Internal system effects
    AcquireResource {
        resource_id: ResourceId,
        mode: AccessMode,
        continuation: Box<dyn Continuation<ResourceGuard, R>>,
    },
    Invoke {
        target_program: ProgramId,
        invocation: Invocation,
        continuation: Box<dyn Continuation<InvocationResult, R>>,
    },
    EvolveSchema {
        old_schema: Schema,
        new_schema: Schema,
        continuation: Box<dyn Continuation<EvolutionResult, R>>,
    },
    
    // Time effects
    TimeEffect {
        time_operation: TimeOperation,
        continuation: Box<dyn Continuation<TimeResult, R>>,
    },
    
    // Zero-knowledge effects
    GenerateProof {
        statement: Statement,
        witness: Witness,
        continuation: Box<dyn Continuation<ProofResult, R>>,
    },
    VerifyProof {
        statement: Statement,
        proof: Proof,
        continuation: Box<dyn Continuation<VerificationResult, R>>,
    },
    
    // Content addressing effects
    ContentHash {
        data: Vec<u8>,
        continuation: Box<dyn Continuation<ContentHash, R>>,
    },
    VerifyContent {
        data: Vec<u8>,
        expected_hash: ContentHash,
        continuation: Box<dyn Continuation<VerificationResult, R>>,
    },
}

/// Effect outcome type
pub enum EffectOutcome<T> {
    /// Effect completed successfully
    Success(T),
    /// Effect failed with error
    Error(EffectError),
    /// Effect requires additional context
    NeedsContext(ContextRequest<T>),
    /// Effect will continue asynchronously
    Pending(PendingEffect<T>),
}

/// Continuation trait for effect chaining
pub trait Continuation<I, O>: ContentAddressed {
    /// Apply the continuation to an input value
    fn apply(self: Box<Self>, input: I) -> O;
}

impl ContentAddressed for Effect {
    fn content_hash(&self) -> Result<ContentHash, ContentAddressingError> {
        ContentHash::for_object(self)
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>, ContentAddressingError> {
        Serializer::to_bytes(self)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, ContentAddressingError> {
        Deserializer::from_bytes(bytes)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))
    }
}
```

---

## Fact

```rust
struct Fact {
    fact_id: FactId,
    domain: DomainId,
    fact_type: FactType,
    fact_value: FactValue,
    observed_at: LamportTime,
    observation_proof: ObservationProof,
    content_hash: ContentHash,       // Content hash of this fact
    verification_proof: VerificationProof,  // Proof that this fact was verified
    time_attestation: Option<TimeAttestation>, // Temporal attestation for this fact
}

impl ContentAddressed for Fact {
    fn content_hash(&self) -> Result<ContentHash, ContentAddressingError> {
        ContentHash::for_object(self)
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>, ContentAddressingError> {
        Serializer::to_bytes(self)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, ContentAddressingError> {
        Deserializer::from_bytes(bytes)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))
    }
}

// Time attestation for a fact
struct TimeAttestation {
    timestamp: u64,
    source: AttestationSource,
    signature: String,
    confidence: f64,
}
```

---

## Program

```rust
struct Program {
    program_id: ProgramId,
    user_id: UserId,
    schema: Schema,
    safe_state_policy: SafeStatePolicy,
    effect_dag: EffectDAG,
    content_addressing_policy: ContentAddressingPolicy,  // Policy for content addressing
    verification_requirements: VerificationRequirements,  // Requirements for verification
    time_requirements: TimeRequirements,                 // Requirements for time attestations
}

impl ContentAddressed for Program {
    fn content_hash(&self) -> Result<ContentHash, ContentAddressingError> {
        ContentHash::for_object(self)
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>, ContentAddressingError> {
        Serializer::to_bytes(self)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, ContentAddressingError> {
        Deserializer::from_bytes(bytes)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))
    }
}

// Time requirements for a program
struct TimeRequirements {
    // Minimum confidence level for clock time attestations
    min_confidence: f64,
    // Acceptable attestation sources
    accepted_sources: Vec<AttestationSourceType>,
    // Maximum clock drift allowed between domains
    max_clock_drift: Duration,
    // Whether strict causal ordering is required
    strict_causal_ordering: bool,
}
```

---

## AccountProgram

```rust
struct AccountProgram {
    account_id: AccountId,
    owner: UserId,
    balances: HashMap<(DomainId, Asset), Amount>,
    effect_dag: EffectDAG,
    content_verification_proofs: HashMap<ContentHash, VerificationProof>,  // Proofs for content verification
    time_attestations: HashMap<EffectId, TimeAttestation>, // Time attestations for effects
}

impl ContentAddressed for AccountProgram {
    fn content_hash(&self) -> Result<ContentHash, ContentAddressingError> {
        ContentHash::for_object(self)
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>, ContentAddressingError> {
        Serializer::to_bytes(self)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, ContentAddressingError> {
        Deserializer::from_bytes(bytes)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))
    }
}
```

---

## Agent

```rust
struct Agent {
    // Base resource implementation
    resource: Resource,
    // Identity information
    identity: Identity,
    // Capabilities that define what this agent can do
    capabilities: Vec<Capability>,
    // State information
    state: AgentState,
    // Relationship to other agents and resources
    relationships: Vec<ResourceRelationship>,
}

impl ContentAddressed for Agent {
    fn content_hash(&self) -> Result<ContentHash, ContentAddressingError> {
        ContentHash::for_object(self)
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>, ContentAddressingError> {
        Serializer::to_bytes(self)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, ContentAddressingError> {
        Deserializer::from_bytes(bytes)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))
    }
}

// State information for an agent
enum AgentState {
    Created,
    Initialized,
    Active,
    Suspended { reason: String },
    Upgraded { previous_version: ContentHash },
    Terminated { reason: String },
}

// Relationship between an agent and another resource
struct ResourceRelationship {
    relationship_type: RelationshipType,
    target_resource: ResourceId,
    capabilities: Vec<Capability>,
    metadata: HashMap<String, Value>,
}

enum RelationshipType {
    Owns,
    Parent,
    Child,
    Peer,
    Delegate,
    DependsOn,
    Custom(String),
}
```

---

# Effect System

The effect system is based on a three-layer architecture that unifies all operations, both internal and external, under a consistent algebraic model:

## Algebraic Effect Layer

- **Effect Trait**: Core interface for all effect operations
- **Effect Identification**: All effects have a unique content-addressed ID
- **Continuation Model**: Effects use explicit continuations for composability
- **Effect Outcomes**: Standardized result types for all effects
- **Effect Composition**: Effects can be composed into complex pipelines
- **Error Handling**: Comprehensive typed error handling for all effects

## Effect Constraints Layer

- **Resource Requirements**: Effects declare the resources they need access to
- **Capability Requirements**: Effects declare the capabilities required for execution
- **Type Constraints**: Static typing ensures effects are used correctly
- **Validation Rules**: Effects undergo validation before execution
- **Concurrency Control**: Resource locking ensures safe concurrent access
- **Cross-Domain Validation**: Effects that cross domains undergo special validation

## Domain Implementation Layer

- **Domain Adapters**: Domain-specific implementations of effect handlers
- **Effect Handlers**: Concrete implementations of effect execution logic
- **ZK Integration**: Effects support zero-knowledge proof generation and verification
- **Time Integration**: Effects can interact with the time system
- **Resource Management**: Effects manipulate resources safely through guards
- **Cross-Domain Operations**: Effects can operate across domain boundaries

## Resource-Scoped Concurrency

Resources are protected by explicit locks with deterministic wait queues:

```rust
// Resource lock manager
pub struct ResourceLockManager {
    locks: Mutex<HashMap<ResourceId, LockEntry>>,
}

struct LockEntry {
    holder: Option<TaskId>,
    wait_queue: VecDeque<WaitingTask>,
}

// Resource guard that auto-releases on drop (RAII pattern)
pub struct ResourceGuard {
    manager: Arc<ResourceLockManager>,
    resource: ResourceId,
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        self.manager.release(self.resource.clone());
    }
}

// Effect for resource acquisition
pub struct AcquireResourceEffect<R> {
    resource_id: ResourceId,
    mode: AccessMode,
    continuation: Box<dyn Continuation<ResourceGuard, R>>,
}

impl<R> Effect<R> for AcquireResourceEffect<R> {
    fn execute(self, handler: &dyn EffectHandler) -> EffectOutcome<R> {
        let guard = handler.handle_acquire_resource(self.resource_id, self.mode)?;
        EffectOutcome::Success(self.continuation.apply(guard))
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        vec![self.resource_id.clone()]
    }
    
    // Other implementations
}
```

## ZK VM Integration

Effects support compilation to ZK-VM compatible code:

```rust
// ZK-VM integration for effects
pub trait ZkVmEffect<R>: Effect<R> {
    // Convert the effect to RISC-V code for ZK-VM execution
    fn to_risc_v<W: RiscVWriter>(&self, writer: &mut W) -> Result<(), RiscVError>;
    
    // Generate a witness for the effect execution
    fn generate_witness(&self, inputs: &[Value]) -> Result<Witness, WitnessError>;
    
    // Verify the effect execution with a proof
    fn verify_execution(&self, proof: &Proof) -> Result<bool, VerificationError>;
}

// ZK-VM execution environment
pub struct ZkVmEnvironment {
    vm: ZkVm,
    verification_keys: HashMap<EffectType, VerificationKey>,
}

impl ZkVmEnvironment {
    // Execute an effect in the ZK-VM
    pub fn execute<R>(&self, effect: &dyn ZkVmEffect<R>) -> Result<(R, Proof), ZkVmError> {
        // Implementation
    }
    
    // Verify an effect execution
    pub fn verify<R>(&self, effect: &dyn ZkVmEffect<R>, proof: &Proof) -> Result<bool, ZkVmError> {
        // Implementation
    }
}
```

---

# Concurrency Model

## System-Level Concurrency

- Causality executes **multiple programs concurrently**.
- Each account program acts as a **synchronization point** for resources.
- Programs interacting with the **same account** contend for resource access.
- Programs can operate concurrently if they do not depend on the same facts/resources.
- Content addressing enables optimistic concurrency through hash-based validation.
- Time effects provide synchronization points for temporal operations.

## Program-Level Concurrency

- Programs can spawn **child programs** and wait for their results.
- Programs can split into independent concurrent branches, provided:
    - Each branch works on a **disjoint fact/resource set**.
- Programs receive **fact and effect streams** in causal order.
- Content-addressed references allow safe concurrent access.
- Causal time effects enforce happens-before relationships between concurrent operations.

## Resource-Scoped Concurrency

- Resources are protected by explicit locks with deterministic wait queues.
- Lock acquisition follows RAII patterns through resource guards.
- Resource access modes (read, write, exclusive) control concurrency levels.
- Deadlock prevention through ordered lock acquisition.
- Wait queues ensure fairness and deterministic scheduling.
- Resource state transitions are atomic and verifiable.

---

# Invocation Model

- Programs **invoke** other programs using an **invocation effect**.
- Invocations:
    - Reference **fact snapshots** (what was known at invocation time).
    - Include proof of **current state** of the caller.
    - Include content hashes for verification.
    - Include time attestations for temporal ordering.
- Cross-program calls are **asynchronous** — programs receive results via observed facts.
- Content addressing enables cross-program verification.

---

# Schema Evolution Rules

| Change | Allowed by Default? |
|---|---|
| Add optional field | ✅ |
| Add field with default | ✅ |
| Remove unused field | ✅ |
| Rename field | ❌ |
| Change field type | ❌ |

Programs can override these defaults via their declared **evolution rules**.

## Content-Addressed Schema Evolution

- Schema changes generate a new content hash.
- Evolution is tracked via content addressing.
- Schema history is immutable and verifiable.
- Schema compatibility verification uses content hashing.

---

# Content Addressing System

The content addressing system is a fundamental component of Causality, providing cryptographic guarantees for all stateful objects.

## Core Principles

- **Universal Content Addressing**: All state objects are uniquely identified by their content hash.
- **Deterministic Hashing**: The same content always produces the same hash.
- **Cryptographic Verification**: Content can be verified against its hash.
- **Immutability Guarantee**: Content-addressed objects are immutable; changes create new objects.
- **Canonical Serialization**: Deterministic serialization ensures consistent hashing.

## Content Hash Calculation

Content hashes are calculated as follows:

1. **Serialize**: Object is serialized using canonical serialization.
2. **Normalize**: Serialized data is normalized to ensure consistent representation.
3. **Hash**: A cryptographic hash function (Blake3/Poseidon) is applied to the normalized data.
4. **Verify**: The resulting hash uniquely identifies the content.

## Verification Protocol

1. **Hash Verification**: Verify that object content matches its claimed hash.
2. **Recursive Verification**: Verify all content-addressed references within an object.
3. **Cross-Domain Verification**: Verify content hashes across domain boundaries.
4. **Proof Verification**: Verify cryptographic proofs associated with content.

## Deferred Hashing

For performance optimization, Causality implements deferred hashing:

1. **Deferred Execution**: Hash computation can be deferred to optimize performance.
2. **Batch Processing**: Multiple hash operations can be batched for efficiency.
3. **Verification Guarantees**: Deferred hashing maintains all verification guarantees.

---

# Agent System

The agent system is built on top of the resource system, where agents are specialized resource types that hold capabilities and perform operations.

## Agent Types

1. **User Agent**: Represents an end user with identity and authorization.
2. **Operator Agent**: Performs system operations and maintenance.
3. **Committee Agent**: Acts as a validator for a domain.

## Agent Capabilities

Agents hold capabilities that grant them authority to perform specific operations:

1. **Resource Capabilities**: Authority over resources.
2. **Operational Capabilities**: Authority to execute operations.
3. **Delegation Capabilities**: Authority to delegate capabilities to other agents.
4. **Administrative Capabilities**: Authority to manage the system.

## Agent State Transitions

Agents follow a well-defined lifecycle:

1. **Created**: Initial state when an agent is created.
2. **Initialized**: Agent has been initialized with capabilities.
3. **Active**: Agent is actively performing operations.
4. **Suspended**: Agent is temporarily inactive.
5. **Upgraded**: Agent has been upgraded to a new version.
6. **Terminated**: Agent has been permanently deactivated.

## Agent Relationships

Agents can form relationships with other resources:

1. **Ownership**: Agent owns and has full control over a resource.
2. **Parent/Child**: Hierarchical relationship between agents.
3. **Delegation**: Agent delegates capabilities to another agent.
4. **Dependency**: Agent requires another resource to function.

---

# Safe State Definition

A program is in a **safe state** if:
- No pending cross-program calls.
- No pending resource withdrawals.
- All external facts referenced in the current effect are fully observed.
- All concurrent branches have terminated.
- All content hashes are verified.
- All cross-domain verifications are complete.
- All time effects have been processed and validated.
- All resource locks have been released.
- All capabilities have been verified.

---

# Time Model

## Dual Time Model

Causality implements a dual time model that distinguishes between two fundamental concepts of time:

1. **Causal Time**
   - Defines logical "happens-before" relationships between operations
   - Implemented using Lamport clocks and vector clocks
   - Guarantees that dependent operations respect causal ordering
   - High-trust model (internally derived and verified)
   - Used for enforcing transaction ordering and data dependencies

2. **Clock Time**
   - Represents wall-clock timestamps from external sources
   - Sourced from various attestation providers with different trust levels
   - Includes confidence metrics for each time attestation
   - Lower-trust model (externally influenced)
   - Used for deadlines, timeouts, and real-world clock synchronization

## Time as an Effect

All time changes in Causality are modeled as explicit effects:

1. **Causal Update Effects**
   - Update the causal ordering between operations
   - Specify explicit happens-before relationships
   - Affect the logical clocks in the system
   - Enforced by the effect system

2. **Clock Attestation Effects**
   - Provide external timestamps with varying confidence levels
   - Include source information for trust evaluation
   - Can be verified based on attestation signatures
   - Processed through the effect system with appropriate validation

3. **Time Map Update Effects**
   - Synchronize time across different domains
   - Include proofs for cross-domain time verification
   - Enable consistent time views across the system

## Attestation Sources and Trust Model

Time attestations come from various sources with different trust levels:

1. **Blockchain Sources** (high trust)
   - Timestamps derived from finalized blocks
   - Include block height and hash for verification
   - Trust level based on consensus security and finality guarantees

2. **Committee Sources** (high to medium trust)
   - Threshold-signed timestamps from validator committees
   - Trust level varies based on committee composition and size
   - Verified through threshold signature validation

3. **Operator Sources** (medium trust)
   - Timestamps provided by system operators
   - Trust level depends on operator reputation and validation
   - Verified through operator signatures

4. **Oracle Sources** (medium to low trust)
   - Timestamps from external oracle services
   - Trust level varies based on oracle reputation and data sources
   - Validated through oracle-specific verification mechanisms

5. **User Sources** (lowest trust)
   - User-provided timestamps
   - Minimal trust without additional verification
   - Used primarily for user-specific operations

## Time Integration with Effects

- Programs can **require specific time attestation sources** for critical operations
- Effects can **include time dependencies** as preconditions
- The **effect system validates temporal ordering** before executing effects
- Time effects are **content-addressed** for verifiability and immutability
- Cross-domain time synchronization occurs through **time map updates**

## Program Requirements

Programs can specify their time requirements:
- Minimum confidence level for accepted attestations
- Acceptable attestation sources
- Maximum allowed clock drift
- Whether strict causal ordering is required

---

# Replay Model

- Replay reconstructs:
    - All programs.
    - All account programs.
    - All facts.
    - All effects.
- Replay requires:
    - Complete log history.
    - Content-addressed storage.
    - Verification capabilities.
- Content addressing guarantees replay integrity.
- Cross-domain verification ensures consistent replay across domains.
- Time effects ensure proper temporal replay ordering.
- Effect execution is deterministic for consistent replay results.

---

# ZK Proof Generation and Verification

Causality integrates Zero-Knowledge proofs throughout the system:

## ZK-VM Integration

- Effects can be compiled to RISC-V code compatible with ZK-VMs
- Execution can generate zero-knowledge proofs of correctness
- Proofs can be verified without revealing execution details
- ZK proofs are content-addressed for auditability and verification

## Proof Generation Process

1. **Effect Compilation**: Effects are compiled to ZK-VM compatible RISC-V code
2. **Witness Generation**: System generates a witness for the execution
3. **Proof Generation**: ZK-VM executes the code and generates a proof
4. **Proof Storage**: Proofs are stored with content addressing
5. **Verification**: Proofs can be verified by any party with the verification key

## Cross-Domain ZK Verification

- ZK proofs can be verified across domain boundaries
- Verification keys are published and content-addressed
- Proof verification results are stored as facts
- Cross-domain verification uses standardized zero-knowledge protocols

## ZK Privacy Guarantees

- Effect inputs can remain private while proving correct execution
- Resource state can be proven valid without revealing details
- Authentication can occur without revealing identity information
- Capabilities can be verified without disclosing privileges

---

# Cross-Domain Verification

Causality implements robust cross-domain verification using content addressing:

## Content-Addressed References

- **Content IDs**: Resources are referenced by content hash across domains.
- **Cross-Domain Proofs**: Content verification uses domain-specific verification contexts.
- **Verification Protocols**: Standardized verification across all domains.

## Verification Process

1. **Generate Content Hash**: Generate a cryptographic hash of the content.
2. **Create Verification Context**: Establish a domain-specific verification context.
3. **Generate Verification Proof**: Create a proof of content validity.
4. **Cross-Domain Verification**: Verify the content in the target domain.
5. **Validate Results**: Ensure verification success before proceeding.

## Verification Guarantees

- **Cryptographic Security**: Verification uses strong cryptographic primitives.
- **Tamper Resistance**: Content changes invalidate verification.
- **Cross-Domain Consistency**: Same content verifies consistently across domains.
- **Audit Trail**: Verification history is content-addressed and immutable.
- **Temporal Consistency**: Time attestations ensure cross-domain time alignment.

---

# Content-Addressed Storage

Causality provides a content-addressed storage system with the following characteristics:

## Storage Interface

- **Store**: Store content by its hash.
- **Retrieve**: Retrieve content by its hash.
- **Verify**: Verify content against its hash.
- **Enumerate**: List content matching criteria.

## Storage Guarantees

- **Immutability**: Stored content cannot be modified.
- **Deduplication**: Identical content is stored only once.
- **Verification**: Content is verified on retrieval.
- **Persistence**: Content remains available as needed.

## Performance Optimization

- **Caching**: Frequently accessed content is cached.
- **Lazy Loading**: Content is loaded on demand.
- **Batched Operations**: Storage operations are batched for efficiency.
- **Content Compression**: Content may be compressed for storage efficiency.

---

# Security Guarantees

## Content Addressing Security

- **Collision Resistance**: Content hashes are cryptographically collision-resistant.
- **Tamper Evidence**: Content tampering is immediately detectable.
- **Non-Repudiation**: Content creation is cryptographically verifiable.
- **Zero-Trust Verification**: Content can be verified without trusting its source.

## Cross-Domain Security

- **Domain Isolation**: Domains maintain security isolation.
- **Cryptographic Boundaries**: Cross-domain operations use cryptographic verification.
- **Capability-Based Security**: Operations require explicit capabilities.
- **Content-Based Authorization**: Authorization uses content addressing.

## Temporal Security

- **Time Attestation Verification**: All time attestations are cryptographically verified.
- **Trust-Based Time Acceptance**: Time sources are evaluated based on trust models.
- **Causal Order Enforcement**: Causal dependencies are cryptographically enforced.
- **Clock Drift Detection**: Cross-domain clock drift is monitored and limited.

---

# Audit Trail and Privacy Controls

## Immutable Audit Trail

- All operations are recorded in a content-addressed, append-only log.
- Content addressing ensures auditability and non-repudiation.
- Verification ensures integrity of the audit trail.
- Time attestations provide temporal context for auditing.
- Nullifiers enable selective privacy while preventing double-spending:
  - A nullifier is a unique cryptographic identifier that marks a resource as spent.
  - Nullifiers can be verified without revealing the underlying data.
  - ZK proofs confirm nullifier validity without exposing transaction details.
  - The system maintains a registry of all used nullifiers to prevent replay attacks.
  - Private transactions generate and verify nullifiers through zero-knowledge operations.

---

# Implementation Requirements

Systems implementing this contract must provide:

1. **Complete ContentAddressed Trait**: Full implementation of content addressing for all state objects.
2. **Consistent Verification**: Consistent cryptographic verification throughout the system.
3. **Cross-Domain Proofs**: Robust cross-domain content verification.
4. **Canonical Serialization**: Deterministic serialization for content addressing.
5. **Deferred Hashing**: Performance optimizations for content hashing.
6. **Sparse Merkle Tree Integration**: Full content addressing for SMT nodes and proofs.
7. **Content-Addressed Storage**: High-performance content-addressed storage.
8. **Verification Metrics**: Tracking and reporting of verification statistics.
9. **Content Normalization**: Robust normalization for consistent hashing.
10. **Effect System Implementation**: Complete implementation of the three-layer effect architecture.
11. **Resource-Scoped Concurrency**: Deterministic resource locks with explicit wait queues.
12. **ZK-VM Integration**: Compilation of effects to ZK-compatible RISC-V code.
13. **Agent Resource Model**: Implementation of agents as specialized resource types.
14. **Capability System**: Complete capability-based security implementation.
15. **Documentation**: Comprehensive documentation of effect system and agent protocols.

---

# Conclusion

This system contract defines the fundamental guarantees and requirements for the Causality system. By adhering to these principles, Causality provides a secure, verifiable, and consistent platform for cross-domain operations with strong cryptographic guarantees.
