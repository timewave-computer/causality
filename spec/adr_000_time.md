# ADR-000: Time Model

## Status

Accepted and Extended

*Note: This ADR has been extended with the "Time as an Effect" concept in March 2023 while maintaining backward compatibility with the original decision.*

## Implementation Status

This ADR has been fully implemented. The time model is a core component of the Causality system, providing a unified framework for tracking time across domains. Key implementation components include:

- TimeMap structure for tracking domain positions
- LamportClock implementation for logical time
- Register-based time commitments for cross-domain verification
- Time map integration with the content addressing system
- Cross-domain temporal validation system
- Time integration with the fact observation system
- ZK proof generation for time map verification

The time system forms the foundation for temporal validation, fact observation, and cross-domain operations. Documentation is available in [docs/src/temporal_model.md](/docs/src/temporal_model.md) and [docs/src/temporal_validation.md](/docs/src/temporal_validation.md).

## Update: Time as an Effect

*This section represents an extension to the original ADR, reflecting the evolution of the time model.*

Our implementation experience has revealed two distinct notions of time that should be explicitly modeled in the system:

1. **Causal Time**: A materialization of operations that are partially ordered with respect to others in the Causality system. This represents the logical ordering of events and captures the "happens-before" relationship between operations.

2. **Clock Time**: Attestations by outside parties about when events occurred. These could come from users, operators, blockchain timestamps, or other external sources. Clock time involves different trust models depending on the source.

To better represent these distinct time concepts and provide a more integrated approach, we propose treating time as an effect in the system.

### Time Effect Model

```rust
/// Time effect types
enum TimeEffect {
    /// Update to causal ordering
    CausalUpdate {
        /// Operations being causally ordered
        operations: Vec<OperationId>,
        /// Causal ordering constraints
        ordering: Vec<(OperationId, OperationId)>, // (before, after)
    },
    
    /// Clock time attestation
    ClockAttestation {
        /// Domain providing the clock time
        domain_id: DomainId,
        /// Actual timestamp value
        timestamp: Timestamp,
        /// Source of the attestation
        source: AttestationSource,
        /// Confidence level (0.0-1.0)
        confidence: f64,
    },
    
    /// Time map update
    TimeMapUpdate {
        /// New domain positions
        positions: HashMap<DomainId, Height>,
        /// Proof of domain positions
        proofs: HashMap<DomainId, PositionProof>,
    },
}

/// Sources of time attestations
enum AttestationSource {
    /// Blockchain timestamp
    Blockchain {
        /// Block height
        height: BlockHeight,
        /// Block hash
        block_hash: BlockHash,
    },
    
    /// User attestation
    User {
        /// User identifier
        user_id: UserId,
        /// User signature
        signature: Signature,
    },
    
    /// Operator attestation
    Operator {
        /// Operator identifier
        operator_id: OperatorId,
        /// Operator signature
        signature: Signature,
    },
    
    /// Committee attestation
    Committee {
        /// Committee identifier
        committee_id: CommitteeId,
        /// Threshold signature
        threshold_signature: ThresholdSignature,
    },
    
    /// External oracle
    Oracle {
        /// Oracle identifier
        oracle_id: OracleId,
        /// Oracle signature
        signature: Signature,
    },
}
```

### Trust Models for Time

Different sources of time attestations carry different trust assumptions:

1. **Causal Time Trust**: Based on internal system invariants and cryptographic verification of operation ordering, providing high confidence.

2. **Clock Time Trust**: Varies by source:
   - **Blockchain Timestamps**: Trust depends on the consensus model of the specific blockchain.
   - **User Attestations**: Low trust, useful primarily for user-centric operations.
   - **Operator Attestations**: Medium trust, requiring operator honesty.
   - **Committee Attestations**: Higher trust through threshold signatures and Byzantine fault tolerance.
   - **Oracle Attestations**: Trust depends on the specific oracle's reputation and security model.

### Time Effect Handlers

Time effects will be processed by dedicated handlers that maintain the system's temporal state:

```rust
/// Handle a time effect
async fn handle_time_effect(effect: TimeEffect) -> Result<TimeEffectResult, TimeError> {
    match effect {
        TimeEffect::CausalUpdate { operations, ordering } => {
            // Update the causal graph
            let updated_graph = update_causal_graph(operations, ordering).await?;
            
            // Return the updated causal state
            Ok(TimeEffectResult::CausalUpdate {
                graph_hash: updated_graph.hash(),
                affected_operations: operations,
            })
        },
        
        TimeEffect::ClockAttestation { domain_id, timestamp, source, confidence } => {
            // Validate the attestation
            validate_attestation(&domain_id, &timestamp, &source).await?;
            
            // Update the time model with the attestation
            let updated_model = update_clock_attestation(
                &domain_id, 
                timestamp, 
                &source, 
                confidence
            ).await?;
            
            // Return the updated time state
            Ok(TimeEffectResult::ClockUpdate {
                domain_id,
                timestamp,
                confidence,
            })
        },
        
        TimeEffect::TimeMapUpdate { positions, proofs } => {
            // Validate all proofs
            validate_position_proofs(&positions, &proofs).await?;
            
            // Update the time map
            let updated_map = update_time_map(positions).await?;
            
            // Return the updated time map
            Ok(TimeEffectResult::TimeMapUpdate {
                map_hash: updated_map.hash(),
                domains_updated: updated_map.domains_updated(),
            })
        },
    }
}
```

### Integration with Unified Temporal Facts

This model integrates with the unified temporal facts approach (described in ADR-024), as each fact now carries both causal and clock time information:

```rust
/// A unified temporal fact with enhanced time model
struct TemporalFact {
    // Core fact fields
    fact_id: FactId,
    domain_id: DomainId,
    fact_type: FactType,
    fact_value: Value,
    
    // Enhanced time model
    causal_position: CausalPosition,
    clock_attestation: ClockAttestation,
    time_position: TimePosition,
    
    // Verification
    observation_proof: ObservationProof,
    observer: String,
    metadata: HashMap<String, String>,
}

/// Causal position in the operation graph
struct CausalPosition {
    /// Operations that happened before this fact
    happens_after: Vec<OperationId>,
    /// Operations that happened after this fact
    happens_before: Vec<OperationId>,
    /// Lamport timestamp
    lamport_time: u64,
}

/// Clock time attestation
struct ClockAttestation {
    /// Attested timestamp
    timestamp: Timestamp,
    /// Source of attestation
    source: AttestationSource,
    /// Confidence level
    confidence: f64,
}
```

### Benefits of Time as an Effect

Treating time as an effect provides several key advantages:

1. **Unified Processing Model**: Time updates use the same effect system as other operations.
2. **Explicit Trust Models**: Different time sources have clear trust models.
3. **Composability**: Time effects compose with other effects in the system.
4. **Auditability**: All time changes are explicitly logged as effects.
5. **Programmability**: Programs can explicitly reason about and manipulate time concepts.

## Context

Causality programs operate across multiple independent Domains — each corresponding to a chain, rollup, or distributed ledger. Each Domain advances independently and asynchronously, with its own:

- Consensus process.
- Block height and timestamps.
- Inclusion and proof mechanisms.
- Finality guarantees.

This makes it essential for Causality to maintain a unified and causally consistent view of time across all Domains participating in a program's execution. This view must:
- Capture external observations from each Domain.
- Provide replayable proofs of external facts.
- Preserve internal causal ordering between program effects.

Programs need to reason about time across Domains to ensure:

1. Causal consistency
2. Temporal ordering
3. Finality tracking
4. Cross-domain coordination


## Decision

We will use a unified time model that combines:

1. Domain-local Time: Each Domain maintains its own Lamport clock
2. Cross-domain Time Maps: Programs track relative time positions across Domains
3. Register-Based Time Commitments: Time maps stored in registers and verified with ZK proofs

### Time Model Components

```rust
// Domain-local Lamport clock
struct LamportClock {
    Domain_id: DomainId,
    counter: u64,
}

// Map of Domain positions
struct TimeMap {
    positions: HashMap<DomainId, Height>,
    observed_at: LamportTime,
    commitments: HashMap<DomainId, Commitment>,
}

// Register-based time commitment
struct TimeMapCommitment {
    register_id: RegisterId,
    time_map: TimeMap,
    proof: Proof,
    last_updated: BlockHeight,
}
```

### Time Operations

```rust
// Update local Lamport clock
fn tick_clock(mut clock: LamportClock) -> LamportClock {
    clock.counter += 1;
    clock
}

// Merge two time maps, taking the later position for each Domain
fn merge_time_maps(tm1: &TimeMap, tm2: &TimeMap) -> TimeMap {
    let mut merged_positions = tm1.positions.clone();
    
    for (tid, height) in &tm2.positions {
        merged_positions.entry(tid.clone())
            .and_modify(|h| *h = (*h).max(*height))
            .or_insert(*height);
    }
    
    TimeMap {
        positions: merged_positions,
        observed_at: tm1.observed_at.max(tm2.observed_at),
        commitments: tm1.commitments.clone().into_iter()
            .Domain(tm2.commitments.clone())
            .collect(),
    }
}

// Check if a time map is ahead of another for all Domains
fn is_ahead_of(tm1: &TimeMap, tm2: &TimeMap) -> bool {
    tm2.positions.iter().all(|(tid, h)| {
        let tm1_height = tm1.positions.get(tid).unwrap_or(&0);
        tm1_height >= h
    })
}

// Create a time map commitment in a register
async fn commit_time_map(tm: &TimeMap) -> Result<TimeMapCommitment, Error> {
    let proof = generate_time_map_proof(tm).await?;
    let reg_id = create_register(TimeMapContents {
        positions: tm.positions.clone(),
        commitments: tm.commitments.clone(),
    }).await?;
    
    Ok(TimeMapCommitment {
        register_id: reg_id,
        time_map: tm.clone(),
        proof,
        last_updated: get_current_height().await?,
    })
}

// Verify a time map commitment
async fn verify_time_map_commitment(tmc: &TimeMapCommitment) -> Result<bool, Error> {
    let register_exists = check_register_exists(&tmc.register_id).await?;
    let proof_valid = verify_proof(&tmc.proof, &tmc.time_map).await?;
    
    Ok(register_exists && proof_valid)
}
```

### Register-Based Time Maps

Time maps will be stored in registers to enable:

1. On-domain verification: Domains can verify time maps in smart contracts
2. ZK proof generation: Generate ZK proofs of time map correctness
3. Cross-domain coordination: Share time maps between domains securely
4. Temporal validation: Verify temporal ordering of operations
5. Auditability: Track when domains have been observed

### ZK Circuit for Time Map Verification

```rust
// ZK circuit for verifying time map updates
struct Circuit {
    name: String,
    inputs: Vec<CircuitInput>,
    outputs: Vec<CircuitOutput>,
    constraints: Vec<Constraint>,
}

// Generate a proof for a time map update
async fn generate_time_map_proof(
    old_tm: &TimeMap,
    new_tm: &TimeMap,
    updates: &[(DomainId, Height)]
) -> Result<Proof, Error> {
    // Generate ZK proof that new_tm is a valid update to old_tm
    let circuit = compile_circuit(&verify_time_map_update())?;
    let mut witness = HashMap::new();
    
    witness.insert("oldTimeMap".to_string(), old_tm.clone());
    witness.insert("newTimeMap".to_string(), new_tm.clone());
    witness.insert("domainUpdates".to_string(), updates.to_vec());
    
    let witness = generate_witness(&circuit, &witness).await?;
    generate_proof(&circuit, &witness).await
}

fn verify_time_map_update() -> Circuit {
    Circuit {
        name: "TimeMapUpdate".to_string(),
        inputs: vec![
            CircuitInput::new("oldTimeMap", InputType::Commitment),
            CircuitInput::new("newTimeMap", InputType::Commitment),
            CircuitInput::new("domainUpdates", InputType::List(Box::new(InputType::Pair(
                Box::new(InputType::DomainId),
                Box::new(InputType::Height)
            )))),
        ],
        outputs: vec![
            CircuitOutput::new("valid", OutputType::Boolean),
        ],
        constraints: vec![
            Constraint::Equal(
                "valid".to_string(),
                format!("allUpdatesValid oldTimeMap newTimeMap domainUpdates")
            ),
        ],
    }
}
```

### Cross-domain Temporal Validation

```rust
// Validate that an operation respects temporal ordering
async fn validate_temporal_ordering(
    required_tm: &TimeMap,
    op: &Operation,
    actual_tm: &TimeMap
) -> Result<bool, Error> {
    // Check if actual time map is ahead of required time map
    let temporally_valid = is_ahead_of(actual_tm, required_tm);
    
    // For register operations, verify time map commitment
    match op {
        Operation::RegisterOp { reg_id, .. } => {
            let commitment = get_register_time_map_commitment(reg_id).await?;
            let commitment_valid = verify_time_map_commitment(&commitment).await?;
            Ok(temporally_valid && commitment_valid)
        },
        _ => Ok(temporally_valid),
    }
}
```

## Time Map Components

| Field | Description |
|---|---|
| Domain ID | Ethereum, Solana, Celestia, etc. |
| Height | Current block height or slot number. |
| Hash | Block hash or equivalent commitment. |
| Timestamp | Block timestamp (if provided by the domain). |


## Example Time Map

```toml
[time_map.Ethereum]
height = 123456
hash = "0xabc123"
timestamp = 1710768000

[time_map.Celestia]
height = 98765
hash = "0xdef456"
timestamp = 1710768005
```


## Observed Time Map (Per Effect)

Every proposed effect — whether originating from a User, account program, or program-to-program invocation — records the time map snapshot that was observed when the effect was proposed.

This observed Time Map is part of the effect proof, ensuring that:

- Each effect is tied to a specific set of external facts.
- Each precondition check (e.g., balance proofs) is tied to the exact external state at the time of proposal.
- Replay and audit can reconstruct the same snapshot to check for validity.


## Time Map in the Effect Pipeline

| Stage | Role of Time Map |
|---|---|
| Proposal | Proposing actor queries latest Time Map and embeds it in effect proposal. |
| Application | Effect is re-validated against current Time Map before application. |
| Replay | Replay reconstructs each observed Time Map to re-run all precondition checks. |


## Internal Logical Time: Lamport Clock

Each program maintains an internal Lamport clock, which tracks:

- Total causal ordering of all effects applied within the program.
- Monotonic sequence number for each applied effect.
- Links to the per-resource effect log.

This ensures internal time is totally ordered within each program — even if external domains advance asynchronously.


## Precondition Horizon

Every effect records:
- The observed time map (snapshot at proposal time).
- The external facts it depended on (balance proofs, inclusion proofs).

At the time of application, the current time map is compared to the observed one:

- If the time map advanced (new blocks observed), external preconditions are revalidated.
- If preconditions still hold, the effect applies.
- If preconditions fail under the new time map, the effect is rejected.

This protects programs against:

- Reorgs.  
- Double-spends (withdrawals already processed).  
- External state drift (balance changes, price changes).  


## Time Map Hashing

Each Time Map is content-addressed:

```rust
let time_map_hash = hash(&[
    &all_domains_heights, 
    &all_domains_hashes, 
    &all_domains_timestamps
]);
```

This hash is:

- Stored directly in every applied effect's log entry.
- Included in every effect proof.
- Passed into proof-of-correct-execution for zk generation.

This guarantees:

- Effects are cryptographically linked to external state.
- Time consistency is independently verifiable.
- Effects cannot retroactively depend on altered facts.


## Time Map and the Unified Log

Every applied effect in the unified log includes:

- Observed Time Map.
- Time Map hash.
- Parent effect hash (causal link).
- Logical timestamp (Lamport clock tick).

This ensures the unified log records:

- Causal history of effects.
- External domain observations at each step.
- Causal consistency with both internal and external time.


## Replay and Time Reconstruction

Replay must reconstruct:

- Full sequence of applied effects from the unified log.  
- Exact time map snapshots that were observed at proposal time.  
- Precondition checks against reconstructed time maps.  

This makes Causality fully replayable and auditable from first principles, even if no live chain connection exists during replay.


## Watches and Observations

The watch primitive (e.g., "wait for a deposit") works by:

1. Querying the current time map.
2. Proposing an effect that observes the desired event at a known block height.
3. Validating that the event still exists at effect application time.

This provides:

- Causal consistency between observation and program state.  
- Replayable proof that the observation was valid.  
- Defense against reorg-based ambiguity.  


## Summary - What Each Effect Carries

| Field | Purpose |
|---|---|
| Observed Time Map | Declares external state known at proposal time. |
| Time Map Hash | Commit to specific external snapshot. |
| Parent Effect Hash | Causal predecessor. |
| Lamport Clock | Internal ordering. |
| Proof | Proves valid state transition given observed facts. |


## Cross-domain Consistency

The Time Map serves as the global clock boundary across all Domains:

- Internal causal order (Lamport clock) applies within programs.
- External domain order (Time Map snapshots) applies across programs and domains.
- Cross-domain consistency is ensured by:
    - Observing facts via Committees.
    - Embedding observed facts into effects.
    - Linking effects to the observed Time Map.


## Time in Simulation and Production

This model applies equally to:

| Mode | Time Source |
|---|---|
| In-Memory Simulation | Synthetic Time Map generated by controller. |
| Multi-Process Local | Each process queries local Committee for Time Map. |
| Geo-Distributed | Each actor queries remote Committee for Time Map. |

Everywhere, the Time Map API is:

```rust
async fn get_latest_time_map(domain_id: &DomainId) -> Result<TimeMap, Error>;
async fn observe_fact(query: &FactQuery) -> Result<(ObservedFact, TimeMap), Error>;
```


## Example Time Map Evolution Flow

1. User proposes effect at block 100.
    - Observed Time Map includes block 100 hash.
    - Balance proof at block 100.
2. By the time the effect applies, block 102 is observed.
    - Precondition check:
        - Re-fetch balance at block 102.
        - Check if balance still meets preconditions.
        - Check inclusion proof is still valid in canonical domain.
    - If valid, apply.
    - If invalid, reject.


## Time Map Consistency Invariants

- Every effect carries exactly one observed time map.  
- Every applied effect records the time map hash.  
- No effect can apply unless preconditions hold against the current time map.  
- Every fact and observation passes through Committees — no direct domain RPC in programs.  
- Time Maps are content-addressed and signed.

## Mock ZK Proof Implementation

For development and testing purposes, the Time Map system integrates with a mock ZK proof and verification system that provides the same logical interfaces as actual ZK proofs while simplifying the cryptographic aspects.

### Mock ZK Store

Each domain maintains a key-value store for verification keys and proofs:

```rust
// Mock ZK system key-value store
struct MockZkStore {
    verification_keys: HashMap<VerificationKey, CircuitType>,
    proof_pairs: HashMap<VerificationKey, ProofData>,
    validation_results: HashMap<(ProofData, Vec<u8>), bool>,
}

type VerificationKey = Vec<u8>; // Random bytes mimicking a real verification key
type ProofData = Vec<u8>; // Random bytes mimicking a real ZK proof
```

### Verification Key Generation

Prior to domain instantiation, the system generates random strings to serve as verification keys:
- Each verification key is associated with a specific circuit type
- These keys are stored in the domain's key-value store
- The keys mimic real ZK verification keys without requiring actual cryptographic operations

### Proof Generation and Validation

For each verification key, the system generates a corresponding proof string:

```rust
// Generate mock proof
async fn generate_mock_proof(
    time_map: &TimeMap,
    verification_key: &VerificationKey
) -> Result<ProofData, Error> {
    // Generate random bytes as mock proof
    let proof_data = generate_random_bytes(32)?;
    
    // Store association between verification key and proof
    store_proof_pair(verification_key, &proof_data).await?;
    
    // Return the mock proof
    Ok(proof_data)
}

// Mock prove function
async fn mock_prove(
    time_map: &TimeMap,
    computation_output: &[u8],
    verification_key: &VerificationKey,
    proof_data: &ProofData
) -> Result<bool, Error> {
    // Look up verification key
    let found_key = lookup_verification_key(verification_key).await?;
    
    // Get expected proof for this key
    let expected_proof = lookup_proof_for_key(&found_key).await?;
    
    // Validate computation against Time Map
    let time_map_valid = validate_against_time_map(time_map, computation_output).await?;
    
    // Check if proof matches and computation is valid
    let result = proof_data == &expected_proof && time_map_valid;
    
    // Record validation result
    store_validation_result(&(*proof_data, computation_output.to_vec()), result).await?;
    
    Ok(result)
}
```

### Time Map Integration

The mock ZK system integrates with the Time Map in several key ways:

1. Time Map Inclusion: Each mock proof contains a reference to the Time Map hash, ensuring that proofs are associated with a specific observed state.

2. Temporal Validation: The mock proof system validates that the computation output is consistent with the observed Time Map.

3. Cross-domain Operations: For operations spanning multiple domains, the system verifies that Time Maps across domains are consistent, as part of the validation process.

### Resource Conservation Validation

For resource operations that must maintain conservation (ΔTX = 0):

```rust
async fn validate_resource_operation(
    time_map: &TimeMap,
    operations: &[ResourceOp]
) -> Result<Result<ProofData, ValidationError>, Error> {
    // Calculate resource delta
    let delta = calculate_delta(operations);
    
    // Choose appropriate verification key
    let vk = get_verification_key_for_resource_ops(operations).await?;
    
    if delta == 0 {
        // Generate valid proof for conservative operations
        let proof = generate_mock_proof(time_map, &vk).await?;
        Ok(Ok(proof))
    } else {
        // Return error for non-conservative operations
        Ok(Err(ValidationError::ConservationViolation(delta)))
    }
}
```

This integration ensures that all temporal proofs (based on the Time Map) and resource conservation proofs work together, maintaining the same logical guarantees that real ZK proofs would provide, while simplifying development and testing.

## Benefits

- Works across any chain (domain-agnostic).  
- Replayable even with no live chain access.  
- Handles reorgs gracefully.  
- No program ever queries domains directly.  
- Fully auditable causal link between program state and external reality.  
- Compatible with zk proof generation.


This time model ensures:

- Internal causal consistency.
- External factual consistency.
- Replayable proofs of all observations.
- Auditability across program boundaries.

It is foundational to the Causality architecture and is required for secure, auditable, cross-domain programs.

## Architectural Implications

Formalizing causal time and clock time as distinct concepts and implementing them as effects has significant architectural implications:

### System Components

1. **Effect Handlers**: The system requires dedicated time effect handlers that:
   - Process `CausalUpdate` effects to maintain the causal operation graph
   - Handle `ClockAttestation` effects to update clock time models
   - Apply `TimeMapUpdate` effects to maintain consistent time maps

2. **Time Service**: A dedicated time service component is required to:
   - Maintain the current causal graph state
   - Store and verify clock attestations from different sources
   - Track confidence levels for different time attestations
   - Provide a unified API for time-related operations

3. **Attestation Verification**: New verification components are needed to:
   - Verify signatures from different attestation sources
   - Calculate and track confidence metrics for time attestations
   - Detect and resolve conflicting time attestations

### Integration Points

1. **Effect System**: The time effects will integrate with the existing effect system:
   - Time effects use the same handler registration mechanisms
   - Time effects participate in the same transaction boundaries
   - Time effects can be composed with other effects

2. **Fact System**: The unified temporal facts system needs to:
   - Include both causal and clock time information
   - Track provenance of time information
   - Maintain verifiable proofs of time attestations

3. **Register System**: Registers that store temporal information need to:
   - Support both causal and clock time representations
   - Provide verification of time-based register operations
   - Enable ZK proofs for time-related operations

### Dependency Models

1. **Time Dependency Tracking**: The system must track:
   - Which operations depend on specific causal orderings
   - Which operations depend on clock time attestations
   - Confidence requirements for specific time-dependent operations

2. **Trust-Level Awareness**: Operations can specify:
   - Minimum required trust levels for time attestations
   - Required attestation sources for critical operations
   - Fallback behaviors for low-confidence time information

### Programming Model

1. **Effect-Based Time API**: Programs interact with time through:
   - Explicit time effect proposals
   - Time effect handlers
   - Time service queries

2. **Time-Aware Programming**: Programs can now:
   - Explicitly reason about different notions of time
   - Choose appropriate time sources based on trust requirements
   - Implement custom time-dependent logic

### Operational Considerations

1. **Cross-Domain Time Consistency**: The system must:
   - Maintain consistent time representations across domains
   - Reconcile different time models from different domains
   - Resolve conflicts in reported domain times

2. **Time Synchronization**: Time components need:
   - Mechanisms to handle clock drift between domains
   - Synchronization protocols for causal time tracking
   - Conflict resolution strategies for attestation discrepancies

3. **Replay and Audit**: Replay systems must:
   - Reconstruct both causal and clock time states
   - Replay time effects in the correct order
   - Verify time attestations during replay

4. **Effect-Based Time Services**: The system should provide:
   - Effect-based services for resolving causal time
   - Effect-based services for resolving clock time
   - A registry of available time services with their trust levels
   - Service discovery mechanisms for programs to locate time services

5. **Service Selection Flexibility**: Programs should:
   - Be able to choose which time services they want to use
   - Select services based on their specific trust requirements
   - Explicitly declare their time service dependencies
   - Fallback to alternative services when preferred ones are unavailable

6. **System Epoch Exception**: While programs generally have flexibility in choosing time services, system epochs are an exception:
   - System epochs must use a consistent, system-wide time service
   - All programs must recognize and respect system epoch boundaries
   - Epoch transitions are managed by the system, not individual programs
   - Critical system-wide operations are anchored to epoch transitions

This architectural evolution maintains backward compatibility with existing time-dependent components while enabling a more explicit and flexible approach to temporal modeling and reasoning.

## Evolution to Effect-Based Time

The evolution of our time model to treat time as an effect represents a natural progression of the original design. By explicitly modeling causal time and clock time as distinct concepts with their own trust models, we better capture the reality of distributed systems. The original time map concept remains valuable as a snapshot of system state, but treating time updates as effects provides greater flexibility, composability, and auditability. This approach maintains all the benefits of the original design while enabling more explicit reasoning about temporal relationships and trust models. Programs can now manipulate time concepts directly through the effect system, using the same patterns they use for other operations, creating a more uniform and intuitive programming model.
