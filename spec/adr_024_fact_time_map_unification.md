# ADR-024: Unification of Facts and Time Maps

## Status

Proposed

## Context

The Causality system currently maintains two related but separate concepts:

1. **Facts**: Observations about external domains (balances, prices, transactions, register states) that are cryptographically proven, signed, and timestamped.

2. **Time Maps**: Causal ordering structures that track domain positions (block heights, hashes, timestamps) to establish temporal relationships between events.

These concepts are deeply intertwined:
- Every fact corresponds to an observation at a specific point in a domain's timeline
- Every effect depends on facts that existed at specific points in time
- Temporal validation requires checking both facts and their relative positions in time maps

The current architecture maintains these as separate systems with redundant information:
- Facts include timestamps but lack complete temporal context
- Time maps track temporal positions but don't directly incorporate facts
- `FactSnapshot` attempts to bridge this gap but creates a third abstraction

This separation introduces several challenges:
- Developers must explicitly reason about both systems
- Temporal validation requires separate lookups and correlation
- Replay needs to reconstruct both fact logs and time maps
- Register operations must track temporal consistency separately from fact validation

## Decision

We will unify facts and time maps into a single integrated concept:

```rust
/// A unified fact that includes its complete temporal context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalFact {
    /// Unique identifier for this fact
    pub fact_id: FactId,
    
    /// The domain this fact comes from
    pub domain_id: DomainId,
    
    /// Type of this fact (balance, transaction, etc.)
    pub fact_type: FactType,
    
    /// The actual fact data
    pub fact_value: Value,
    
    /// Time map position when this fact was observed
    pub time_position: TimePosition,
    
    /// Cryptographic proof of observation
    pub observation_proof: ObservationProof,
    
    /// Committee that observed this fact
    pub observer: String,
    
    /// Additional metadata for this fact
    pub metadata: HashMap<String, String>,
}

/// Position in a domain's timeline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimePosition {
    /// Block height this fact was observed at
    pub height: BlockHeight,
    
    /// Block hash at this position
    pub hash: BlockHash,
    
    /// Timestamp at this position
    pub timestamp: Timestamp,
    
    /// Confidence in this time position (0.0-1.0)
    pub confidence: f64,
    
    /// Whether this position has been verified
    pub verified: bool,
}

/// A snapshot of the temporal state across all domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalContext {
    /// Map of domain IDs to their time positions
    pub positions: HashMap<DomainId, TimePosition>,
    
    /// When this context was created
    pub observed_at: Timestamp,
    
    /// Hash of this temporal context
    pub context_hash: Hash,
}
```

This unified approach provides several key mechanisms:

### 1. Fact Observation with Temporal Context

When a Committee observes a fact, it automatically includes the complete time position:

```rust
/// Observe a fact with its temporal context
pub async fn observe_fact(
    query: &FactQuery,
    domain_id: &DomainId
) -> Result<TemporalFact, ObservationError> {
    // Get the current time position for this domain
    let time_position = get_domain_time_position(domain_id).await?;
    
    // Observe the fact
    let fact_value = query_domain_fact(domain_id, query).await?;
    
    // Generate proof of observation
    let proof = generate_observation_proof(domain_id, &fact_value, &time_position).await?;
    
    // Create the temporal fact
    let temporal_fact = TemporalFact {
        fact_id: generate_fact_id(&fact_value, &time_position),
        domain_id: domain_id.clone(),
        fact_type: query.fact_type.clone(),
        fact_value,
        time_position,
        observation_proof: proof,
        observer: get_committee_id().to_string(),
        metadata: HashMap::new(),
    };
    
    Ok(temporal_fact)
}
```

### 2. Effect Dependencies with Temporal Context

Effects will directly reference the temporal facts they depend on, implicitly establishing their causal relationship to the time map:

```rust
/// An effect with its dependent temporal facts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    /// Unique identifier for this effect
    pub effect_id: EffectId,
    
    /// Type of effect
    pub effect_type: EffectType,
    
    /// Arguments for this effect
    pub parameters: HashMap<String, Value>,
    
    /// Temporal facts this effect depends on
    pub dependent_facts: Vec<FactId>,
    
    /// Temporal context when this effect was proposed
    pub temporal_context: TemporalContext,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}
```

### 3. Temporal Validation 

Temporal validation becomes more natural, as facts directly encode their position in time:

```rust
/// Validate that an effect's temporal dependencies are satisfied
pub fn validate_temporal_dependencies(
    effect: &Effect,
    facts: &[TemporalFact]
) -> Result<bool, ValidationError> {
    // Extract temporal context from each fact
    let fact_contexts = facts.iter()
        .map(|fact| (fact.domain_id.clone(), fact.time_position.clone()))
        .collect::<HashMap<_, _>>();
    
    // Create a temporal context from the facts
    let observed_context = TemporalContext {
        positions: fact_contexts,
        observed_at: get_current_time(),
        context_hash: calculate_context_hash(&fact_contexts),
    };
    
    // Validate that the current context is at least as advanced as the effect's context
    for (domain_id, required_position) in &effect.temporal_context.positions {
        if let Some(current_position) = observed_context.positions.get(domain_id) {
            if current_position < required_position {
                return Ok(false); // Context isn't advanced enough
            }
        } else {
            return Ok(false); // Missing domain
        }
    }
    
    Ok(true)
}
```

### 4. Register Operations with Temporal Validation

Register operations for the ZK-based register system will directly incorporate temporal validation:

```rust
/// Perform a register operation with temporal validation
pub async fn perform_register_operation(
    operation: RegisterOperation,
    temporal_facts: Vec<TemporalFact>
) -> Result<RegisterOperationResult, OperationError> {
    // Extract the temporal context from the facts
    let temporal_context = extract_temporal_context(&temporal_facts)?;
    
    // Generate inputs for the ZK circuit
    let mut public_inputs = HashMap::new();
    public_inputs.insert("operation".to_string(), serialize_operation(&operation));
    public_inputs.insert("temporal_context".to_string(), serialize_context(&temporal_context));
    
    // Generate the ZK proof
    let proof = generate_zk_proof(
        &CircuitType::RegisterWithTemporal,
        &public_inputs,
        &operation.witness_inputs
    ).await?;
    
    // Execute the register operation with the proof
    let result = execute_register_operation(operation, proof).await?;
    
    Ok(result)
}
```

### 5. Event Handling for Temporal Fact Updates

When new facts are observed, the system will propagate temporal updates more coherently:

```rust
/// Process a new temporal fact and update affected components
pub async fn process_temporal_fact(
    fact: TemporalFact
) -> Result<(), ProcessingError> {
    // Log the fact
    log_temporal_fact(&fact).await?;
    
    // Update the global temporal context
    update_temporal_context(&fact.domain_id, &fact.time_position).await?;
    
    // Notify interested observers
    notify_fact_observers(&fact).await?;
    
    // Check for pending effects that depend on this fact
    process_pending_effects_for_fact(&fact.fact_id).await?;
    
    Ok(())
}
```

## Consequences

### Positive

1. **Simplified Mental Model**: Developers work with a single unified concept for facts and their temporal context.

2. **More Natural Temporal Validation**: Validating causal relationships becomes more intuitive, as facts directly encode their position in time.

3. **Streamlined Replay**: Replay only needs to reconstruct temporal facts, as they contain all necessary temporal information.

4. **Reduced Redundancy**: Eliminates duplicate storage of temporal information across separate systems.

5. **Improved Register Integration**: Register operations can use temporal facts directly for validation, simplifying the ZK circuit interfaces.

6. **Better Cross-Domain Coordination**: Temporal facts provide clear ordering across domains without requiring separate time map lookups.

7. **Enhanced Auditability**: Each fact carries its complete causal context, making audits more straightforward.

### Negative

1. **Migration Complexity**: Existing systems using separate facts and time maps will need to be migrated.

2. **Increased Fact Size**: Temporal facts include more information than simple facts, slightly increasing storage requirements.

3. **Learning Curve**: Developers familiar with the current separation will need to adapt to the unified model.

### Mitigation Strategies

1. **Backward Compatibility Layer**: Provide adapters that allow existing code to interact with the unified system.

2. **Incremental Migration**: Roll out the unification in phases, starting with new components and gradually migrating existing ones.

3. **Efficient Serialization**: Use compact encodings and consider sharing temporal context across related facts to reduce storage overhead.

4. **Extended Documentation**: Provide clear documentation and examples showing the benefits of the unified approach.

## Implementation Plan

1. **Phase 1: Core Data Structures** (2 weeks)
   - Implement `TemporalFact` and `TemporalContext` structures
   - Create serialization/deserialization for these types
   - Build content-addressing for the unified structures

2. **Phase 2: Committee Integration** (2 weeks)
   - Update fact observation methods to include temporal context
   - Modify the fact gossip protocol to handle temporal facts
   - Update the committee fact storage mechanism

3. **Phase 3: Effect System Integration** (3 weeks)
   - Modify the effect system to work with temporal facts
   - Update effect validation to use temporal validation directly
   - Implement new temporal dependency tracking

4. **Phase 4: Register System Updates** (3 weeks)
   - Update register operations to use temporal facts
   - Modify ZK circuits to work with temporal facts
   - Implement updated validation mechanisms

5. **Phase 5: Migration Tools** (2 weeks)
   - Create tools to migrate existing facts and time maps
   - Build conversion utilities for backward compatibility
   - Develop testing tools to validate equivalent behavior

## Example Workflows

### 1. Observe External Fact

```rust
// Committee observes a balance on Ethereum
let query = FactQuery {
    fact_type: "balance".into(),
    parameters: {
        let mut params = HashMap::new();
        params.insert("address".to_string(), "0x1234...".to_string());
        params.insert("token".to_string(), "USDC".to_string());
        params
    },
};

// This now returns a unified temporal fact
let temporal_fact = ethereum_committee.observe_fact(&query, &ethereum_domain).await?;

// Gossip the temporal fact to operators
gossip_temporal_fact(&temporal_fact).await?;
```

### 2. Apply Effect with Temporal Dependencies

```rust
// Create an effect that depends on observed facts
let effect = Effect {
    effect_id: generate_effect_id(),
    effect_type: EffectType::Transfer,
    parameters: {
        let mut params = HashMap::new();
        params.insert("from".to_string(), "alice".to_string());
        params.insert("to".to_string(), "bob".to_string());
        params.insert("amount".to_string(), "100".to_string());
        params
    },
    // Reference the temporal facts this effect depends on
    dependent_facts: vec![balance_fact_id, price_fact_id],
    // Include the temporal context when this effect was created
    temporal_context: current_temporal_context(),
    metadata: HashMap::new(),
};

// Apply the effect
apply_effect(&effect).await?;
```

### 3. Register Operation with Temporal Validation

```rust
// Create a register operation that includes temporal facts
let operation = RegisterOperation {
    op_type: RegisterOpType::Update,
    register_id: "reg123".to_string(),
    new_contents: RegisterContents::TokenBalance {
        token_type: "USDC".to_string(),
        address: "0x1234...".to_string(),
        amount: 1000,
    },
    // Include temporal facts for validation
    temporal_facts: vec![balance_fact_id, allowance_fact_id],
    witness_inputs: generate_witness_inputs(),
};

// Perform the register operation with temporal validation
let result = perform_register_operation(operation, temporal_facts).await?;
```

## Alternatives Considered

### 1. Maintain Separate Systems with Tighter Integration

We could keep facts and time maps separate but create a more formal integration layer.

**Rejected because**: This would maintain the conceptual complexity and still require developers to reason about two separate systems.

### 2. Fact References in Time Maps

We could extend time maps to reference facts directly, rather than unifying the concepts.

**Rejected because**: This would create bidirectional dependencies between the systems, increasing complexity without addressing the fundamental redundancy.

### 3. Context-Free Facts with Central Time Service

We could keep facts simple and provide a central time service for relating them.

**Rejected because**: This would require additional lookups for temporal validation and make replay more complex.

## Conclusion

Unifying facts and time maps into a single integrated concept of "temporal facts" provides a more natural and intuitive approach to handling causal relationships in the Causality system. This unification simplifies validation, reduces redundancy, and improves integration with other components like the register system. While there's a migration cost, the long-term benefits for system clarity, developer productivity, and architectural coherence justify this change.