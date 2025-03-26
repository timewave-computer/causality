# Causality System Contract

---

## Version

**Current Revision:** 2025-03-19

---

## Purpose

This document defines the **system contract** for Causality, establishing the fundamental invariants, roles, ownership rules, and guarantees that underpin the system. It serves as both a specification and a **social/legal contract** between participants — Users, Users, Operators, and the broader execution network.

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
- Role separation between actors.

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

---

# Core Actors

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
enum Effect {
    Deposit {
        domain: DomainId,
        asset: Asset,
        amount: Amount,
    },
    Withdraw {
        domain: DomainId,
        asset: Asset,
        amount: Amount,
    },
    Transfer {
        from_program: ProgramId,
        to_program: ProgramId,
        asset: Asset,
        amount: Amount,
    },
    ObserveFact {
        fact_id: FactId,
    },
    Invoke {
        target_program: ProgramId,
        invocation: Invocation,
    },
    EvolveSchema {
        old_schema: Schema,
        new_schema: Schema,
        evolution_result: EvolutionResult,
    },
    RegisterOp {
        register_id: RegisterId,
        operation: RegisterOperation,
        auth_method: AuthorizationMethod,
    },
    RegisterCreate {
        owner: Address,
        contents: RegisterContents,
    },
    CustomEffect {
        name: String,
        value: Value,
    },
    ContentHash(ContentHash),      // Content hash of this effect
    VerificationProof(VerificationProof),  // Proof that this effect was verified
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

# Concurrency Model

## System-Level Concurrency

- Causality executes **multiple programs concurrently**.
- Each account program acts as a **synchronization point** for resources.
- Programs interacting with the **same account** contend for resource access.
- Programs can operate concurrently if they do not depend on the same facts/resources.
- Content addressing enables optimistic concurrency through hash-based validation.

## Program-Level Concurrency

- Programs can spawn **child programs** and wait for their results.
- Programs can split into independent concurrent branches, provided:
    - Each branch works on a **disjoint fact/resource set**.
- Programs receive **fact and effect streams** in causal order.
- Content-addressed references allow safe concurrent access.

---

# Invocation Model

- Programs **invoke** other programs using an **invocation effect**.
- Invocations:
    - Reference **fact snapshots** (what was known at invocation time).
    - Include proof of **current state** of the caller.
    - Include content hashes for verification.
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

# Safe State Definition

A program is in a **safe state** if:
- No pending cross-program calls.
- No pending resource withdrawals.
- All external facts referenced in the current effect are fully observed.
- All concurrent branches have terminated.
- All content hashes are verified.
- All cross-domain verifications are complete.

---

# Time Model

- Every Domain has its own **Lamport Clock**.
- Program facts and effects are timestamped using:
    - Domain clock (external facts).
    - Program-local Lamport clock (internal effects).
- Programs can only advance **after observing facts with non-decreasing timestamps**.
- Causality ensures that cross-domain events respect:
    - External Domain ordering (via fact observation).
    - Internal causal ordering (via effect DAG).
- Content addressing enables verification of temporal ordering.

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

---

# Compliance and Audit

## Immutable Audit Trail

- All operations are recorded in a content-addressed, append-only log.
- Content addressing ensures auditability and non-repudiation.
- Verification ensures integrity of the audit trail.

## Regulatory Compliance

- Content addressing enables stronger compliance guarantees.
- Immutable history supports regulatory requirements.
- Cryptographic verification provides evidence of compliance.

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
10. **Documentation**: Comprehensive documentation of content addressing protocols.

---

# Conclusion

This system contract defines the fundamental guarantees and requirements for the Causality system. By adhering to these principles, Causality provides a secure, verifiable, and consistent platform for cross-domain operations with strong cryptographic guarantees.
