# ADR-008: Fact Management

## Status

Implemented

## Implementation Status

The Fact Management system described in this ADR has been fully implemented across several modules:

1. **Core Fact Types**: 
   - `src/log/fact_types.rs` implements the `FactType` enum with all variants described in the ADR, including specialized `RegisterFact` and `ZKProofFact` types
   - Each fact type has comprehensive subtype variants with proper serialization support

2. **Fact Observation and Logging**:
   - `src/log/fact.rs` implements the `FactLogger` for logging different types of facts (state, relationship, property, constraint)
   - `FactMetadata` supports confidence levels, verification status, and expiration times
   - Implements a query builder pattern for flexible fact retrieval

3. **Fact Snapshots**:
   - `src/log/fact_snapshot.rs` implements the `FactSnapshot` structure as described in the ADR
   - Includes register observations and domain tracking
   - `FactDependency` and `FactDependencyType` manage explicit dependencies between effects and facts

4. **Content-Addressed Facts**:
   - `src/log/content_addressed_fact.rs` implements content-addressing for facts
   - Supports verification of fact integrity through content hashing

5. **Fact Observation and Verification**:
   - `src/domain/fact/observer.rs` implements the observer pattern for fact collection
   - `src/domain/fact/verification.rs` provides fact verification mechanisms
   - `src/domain/fact/zkproof_observer.rs` specializes in ZK proof observation

6. **Fact Replay and Simulation**:
   - `src/log/fact_replay.rs` implements fact replay for deterministic execution
   - `src/log/fact_simulator.rs` supports fact simulation for testing
   - `src/log/fact_dependency_validator.rs` validates dependencies between facts

The implementation closely adheres to the design described in the ADR, establishing facts as first-class entities in the system with proper observation, verification, and dependency tracking.

## Context

Causality programs need to observe and react to **facts** from external Domains. Facts could be:

1. Account balances
2. Price data
3. Transaction confirmations
4. State transitions
5. Register state updates
6. ZK proof verifications

Previously, facts were treated as secondary to effects - programs would apply effects and then check facts. This led to:

- Unclear causality between facts and effects
- Difficulties in replay and simulation
- Inconsistent fact verification across Domains
- Challenges in tracking fact dependencies

## Decision

We will transition to treating **facts as first-class causal entities** with the following approach:

1. Facts are **observed by Users**
2. Facts are **signed and timestamped**
3. Facts are **content-addressed**
4. Programs **depend on facts explicitly**
5. Facts include **register observations and proofs**

### Core Data Structures

```rust
use std::collections::HashMap;

type FactID = String;
type DomainID = String;
type LamportTime = u64;
type Proof = String;
type RegisterID = String;
type ControllerLabel = String;
type VerificationKey = String;
type ProofID = String;
type CircuitType = String;
type Inputs = Vec<u8>;
type Outputs = Vec<u8>;
type CommitteeID = String;
type Value = serde_json::Value;

struct Fact {
    fact_id: FactID,                // Content hash of the fact
    domain: DomainID,               // Which Domain the fact comes from
    fact_type: FactType,            // Categorization of fact
    fact_value: Value,              // The actual data
    observed_at: LamportTime,       // When the fact was observed
    observation_proof: Proof,       // Proof of observation
}

enum FactType {
    BalanceFact,                    // Token or native currency balance
    TransactionFact,                // Transaction completion on external Domain
    OracleFact,                     // Data from external oracle
    BlockFact,                      // Block information
    TimeFact,                       // Time observation
    RegisterFact,                   // Register state or operation
    ZKProofFact,                    // ZK proof verification result
    Custom(String),                 // Custom fact type
}

enum RegisterFact {
    RegisterCreation {
        register_id: RegisterID,
        register_contents: RegisterContents,
    },
    RegisterUpdate {
        register_id: RegisterID,
        register_contents: RegisterContents,
    },
    RegisterTransfer {
        register_id: RegisterID,
        domain_id: DomainID,
        controller_label: ControllerLabel,
    },
    RegisterMerge {
        source_registers: Vec<RegisterID>,
        target_register: RegisterID,
    },
    RegisterSplit {
        source_register: RegisterID,
        target_registers: Vec<RegisterID>,
    },
}

enum ZKProofFact {
    ProofVerification {
        verification_key: VerificationKey,
        proof: Proof,
    },
    BatchVerification {
        verification_keys: Vec<VerificationKey>,
        proofs: Vec<Proof>,
    },
    CircuitExecution {
        circuit_type: CircuitType,
        inputs: Inputs,
        outputs: Outputs,
    },
    ProofComposition {
        proof_id: ProofID,
        component_proof_ids: Vec<ProofID>,
    },
}

struct RegisterContents {
    // Register contents structure
    // (Definition would go here but is not shown in the original ADR)
}
```

### Fact Observation Workflow

1. **External Observation**: Committees observe external events
2. **Proof Generation**: Committees generate observation proofs
3. **Fact Creation**: Committees create facts with proofs
4. **Fact Propagation**: Facts are gossiped to Operators
5. **Fact Verification**: Operators verify fact proofs
6. **Fact Storage**: Facts are stored in fact logs
7. **Register Observation**: Register states are observed and recorded as facts
8. **ZK Proof Verification**: ZK proofs are verified and recorded as facts

### FactSnapshot and Effect Dependencies

Programs explicitly depend on facts through fact snapshots:

```rust
struct FactSnapshot {
    observed_facts: Vec<FactID>,                 // Facts observed before effect
    observer: CommitteeID,                       // Who observed the facts
    register_observations: HashMap<RegisterID, Vec<u8>>,  // Register state observations
}

struct Effect<T> {
    effect_type: EffectType<T>,                  // Type of effect
    fact_snapshot: FactSnapshot,                 // Facts depended on
    effect_value: T,                             // Effect payload
    applied_at: LamportTime,                     // When effect was applied
}

enum EffectType<T> {
    // Effect type variants would go here
    // (Not defined in the original ADR, so placeholder)
}
```

### Register-Related Facts

Register facts have special handling:

1. **Register Creation**: When a register is created, a RegisterCreation fact is observed
2. **Register Updates**: When a register is updated, a RegisterUpdate fact is observed
3. **Register Transfers**: When a register moves across Domains, a RegisterTransfer fact is observed
4. **ZK Proofs**: When a ZK proof is verified, a ZKProofFact is observed

These facts enable programs to track register state and verify operations without direct chain queries.

### Fact Replay and Simulation

During replay and simulation:

1. Facts are replayed in observed order
2. Register facts are used to reconstruct register state
3. ZK proof facts are used to verify operation correctness
4. Programs only see facts they explicitly depended on
5. Fact snapshots establish clear causal relationships

## Consequences

### Positive

- Clear causality between facts and effects
- Improved replay and simulation fidelity
- Consistent fact verification across Domains
- Better tracking of fact dependencies
- Register operations can be tracked with explicit facts
- ZK proofs can be verified and recorded as facts

### Negative

- Additional complexity in handling fact dependencies
- Potential overhead from fact storage and propagation
- Learning curve for developers used to checking facts after effects

### Neutral

- Requires standardized fact formats across Domains
- May need periodic updates as new fact types emerge
- Register and ZK proof facts require specialized handling