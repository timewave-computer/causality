# ADR-000: Time Model

## Status

Accepted

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
