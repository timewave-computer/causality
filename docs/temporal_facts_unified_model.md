# Unified Temporal Facts Model

This document outlines the unified temporal facts model within the Causality system, which consolidates the previously separate concepts of facts and time maps into a cohesive framework for managing temporal relationships and dependencies.

## Core Concepts

### Temporal Facts

A **Temporal Fact** is the fundamental unit of temporal information in the Causality system, representing a piece of data with an associated temporal context. Key characteristics include:

1. **Identity**: Each temporal fact has a unique identifier
2. **Data**: The actual information or state being recorded
3. **Temporal Context**: When the fact was observed or becomes valid
4. **Provenance**: The source or origin of the fact
5. **Dependencies**: Other facts this fact depends on

This unified model combines the previously separate concepts of facts (which represented data from external domains) and time maps (which tracked temporal relationships between operations), providing a single, consistent approach to temporal information management.

### Temporal Relationships

Temporal facts establish causal relationships through their dependencies:

1. **Happened-Before**: Fact A happened before Fact B
2. **Depends-On**: Fact B depends on Fact A
3. **Concurrent-With**: Facts that occurred concurrently
4. **Observed-At**: When a fact was observed

These relationships form a directed acyclic graph (DAG) of temporal dependencies, enabling consistent ordering and validation across domains.

## Structure

```rust
/// A unified temporal fact
pub struct TemporalFact {
    /// Unique identifier for the fact
    id: TemporalFactId,
    
    /// The data content of the fact
    data: FactData,
    
    /// The temporal context of the fact
    temporal_context: TemporalContext,
    
    /// Provenance information for the fact
    provenance: FactProvenance,
    
    /// Dependencies on other facts
    dependencies: Vec<TemporalFactId>,
    
    /// Metadata associated with this fact
    metadata: Option<MetadataMap>,
}

/// Data contained in a temporal fact
pub enum FactData {
    /// Resource state data
    ResourceState {
        /// Resource identifier
        resource_id: ResourceId,
        
        /// State of the resource
        state: ResourceState,
    },
    
    /// Operation execution data
    Operation {
        /// Operation identifier
        operation_id: OperationId,
        
        /// Operation type
        operation_type: OperationType,
        
        /// Operation status
        status: OperationStatus,
    },
    
    /// External domain observation
    ExternalObservation {
        /// Domain identifier
        domain_id: DomainId,
        
        /// Observation identifier in the external domain
        external_id: String,
        
        /// Observation data
        data: Vec<u8>,
    },
    
    /// Cross-domain reference
    CrossDomainReference {
        /// Source domain
        source_domain: DomainId,
        
        /// Target domain
        target_domain: DomainId,
        
        /// Reference identifier
        reference_id: String,
    },
    
    /// Custom fact data
    Custom {
        /// Type of the custom data
        data_type: String,
        
        /// Custom data
        data: Vec<u8>,
    },
}

/// Temporal context for a fact
pub struct TemporalContext {
    /// When the fact was observed
    observed_at: Timestamp,
    
    /// When the fact becomes valid
    valid_from: Option<Timestamp>,
    
    /// When the fact expires
    valid_until: Option<Timestamp>,
    
    /// Logical timestamp or sequence number
    logical_timestamp: Option<u64>,
    
    /// Causal vector clock
    vector_clock: Option<VectorClock>,
}

/// Provenance information for a fact
pub struct FactProvenance {
    /// Who or what created the fact
    creator: ResourceId,
    
    /// Domain where the fact originated
    origin_domain: DomainId,
    
    /// Verification proof for the fact
    verification: Option<VerificationProof>,
    
    /// Signatures on the fact
    signatures: Vec<Signature>,
}

/// Temporal fact store
pub struct TemporalFactStore {
    /// Stored facts by ID
    facts: HashMap<TemporalFactId, TemporalFact>,
    
    /// Fact dependencies as a graph
    dependency_graph: DirectedGraph<TemporalFactId>,
    
    /// Facts indexed by resource ID
    resource_facts: HashMap<ResourceId, Vec<TemporalFactId>>,
    
    /// Facts indexed by operation ID
    operation_facts: HashMap<OperationId, Vec<TemporalFactId>>,
    
    /// Facts indexed by domain ID
    domain_facts: HashMap<DomainId, Vec<TemporalFactId>>,
    
    /// Configuration for the fact store
    config: TemporalFactStoreConfig,
}
```

## Integration with Resource System

The unified temporal facts model integrates with the resource system:

1. **Resource Lifecycle**: Resource state changes create temporal facts
2. **Operation Validation**: Operations are validated against temporal facts
3. **Cross-Domain Synchronization**: Temporal facts enable consistent cross-domain state
4. **Capability Verification**: Temporal context affects capability validation
5. **Effect Templates**: Templates include temporal dependencies

## Usage Examples

### Creating and Observing Facts

```rust
// Create a temporal fact store
let mut fact_store = TemporalFactStore::new(
    TemporalFactStoreConfig::default()
);

// Create a resource state fact
let resource_state_fact = TemporalFact::new(
    FactData::ResourceState {
        resource_id: resource_id.clone(),
        state: resource_state.clone(),
    },
    TemporalContext::new()
        .with_observed_at(time::now())
        .with_valid_from(time::now())
        .with_logical_timestamp(1),
    FactProvenance::new(
        observer_id.clone(),
        local_domain_id.clone(),
        None,
        vec![]
    ),
    vec![] // No dependencies
)?;

// Store the fact
let fact_id = fact_store.store(resource_state_fact)?;

// Retrieve the fact
let retrieved_fact = fact_store.get(&fact_id)?;
println!("Fact: {:?}", retrieved_fact);

// Query facts for a resource
let resource_facts = fact_store.get_facts_for_resource(&resource_id)?;
for fact in resource_facts {
    println!("Resource fact: {:?}", fact);
}
```

### Establishing Temporal Dependencies

```rust
// Create a fact with dependencies
let dependent_fact = TemporalFact::new(
    FactData::Operation {
        operation_id: operation_id.clone(),
        operation_type: OperationType::UpdateResource,
        status: OperationStatus::Success,
    },
    TemporalContext::new()
        .with_observed_at(time::now())
        .with_valid_from(time::now())
        .with_logical_timestamp(2),
    FactProvenance::new(
        executor_id.clone(),
        local_domain_id.clone(),
        None,
        vec![]
    ),
    vec![
        prerequisite_fact_id1.clone(),
        prerequisite_fact_id2.clone(),
    ]
)?;

// Store the fact
let dependent_fact_id = fact_store.store(dependent_fact)?;

// Check if all dependencies are satisfied
let dependencies_satisfied = fact_store.are_dependencies_satisfied(
    &dependent_fact_id
)?;

if dependencies_satisfied {
    println!("All dependencies are satisfied");
} else {
    let missing_dependencies = fact_store.get_missing_dependencies(
        &dependent_fact_id
    )?;
    println!("Missing dependencies: {:?}", missing_dependencies);
}
```

### Validating Temporal Consistency

```rust
// Create a temporal validator
let temporal_validator = TemporalValidator::new(
    fact_store.clone(),
    TemporalValidatorConfig::default()
);

// Create an operation with temporal dependencies
let operation = Operation::new(OperationType::UpdateResource)
    .with_inputs([
        ResourceState::from_id(
            resource_id.clone(),
            RegisterState::Active
        )
    ])
    .with_outputs([
        ResourceState::from_id_with_properties(
            resource_id.clone(),
            RegisterState::Active,
            updated_properties.clone()
        )
    ])
    .with_dependencies([
        Dependency::TemporalFact(prerequisite_fact_id.clone())
    ])
    .with_temporal_context(
        TemporalContext::new()
            .with_observed_at(time::now())
            .with_valid_from(time::now())
    )
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(updater_id.clone()));

// Validate temporal consistency
let validation_result = temporal_validator.validate(&operation)?;
if validation_result.is_valid {
    // Operation is temporally consistent
    execute_operation(operation, &context).await?;
    
    // Record the operation as a new temporal fact
    let operation_fact = TemporalFact::new(
        FactData::Operation {
            operation_id: operation.id().clone(),
            operation_type: operation.operation_type().clone(),
            status: OperationStatus::Success,
        },
        operation.temporal_context().clone(),
        FactProvenance::new(
            updater_id.clone(),
            local_domain_id.clone(),
            None,
            vec![]
        ),
        vec![prerequisite_fact_id.clone()]
    )?;
    
    fact_store.store(operation_fact)?;
} else {
    println!("Temporal validation failed: {:?}", validation_result.reason());
}
```

### Cross-Domain Fact Synchronization

```rust
// Create a cross-domain observer
let observer = CrossDomainObserver::new(
    local_domain_id.clone(),
    ObserverConfig::default()
);

// Observe an external fact
let external_fact = observer.observe_external_fact(
    external_domain_id.clone(),
    external_transaction_id.clone(),
    external_data.clone()
)?;

// Store the external observation as a fact
let external_fact_id = fact_store.store(external_fact)?;

// Create a cross-domain reference fact
let reference_fact = TemporalFact::new(
    FactData::CrossDomainReference {
        source_domain: local_domain_id.clone(),
        target_domain: external_domain_id.clone(),
        reference_id: cross_domain_reference_id.clone(),
    },
    TemporalContext::new()
        .with_observed_at(time::now())
        .with_valid_from(time::now()),
    FactProvenance::new(
        observer_id.clone(),
        local_domain_id.clone(),
        None,
        vec![]
    ),
    vec![external_fact_id.clone()]
)?;

// Store the reference fact
let reference_fact_id = fact_store.store(reference_fact)?;

// Create an operation that depends on the cross-domain fact
let cross_domain_operation = Operation::new(OperationType::CreateResource)
    .with_dependencies([
        Dependency::TemporalFact(reference_fact_id.clone())
    ])
    // ... other operation details ...
    .with_context(ExecutionContext::new(ExecutionPhase::Planning))
    .with_authorization(Authorization::from(creator_id.clone()));

// Validate and execute the operation
let validation_result = temporal_validator.validate(&cross_domain_operation)?;
if validation_result.is_valid {
    execute_operation(cross_domain_operation, &context).await?;
}
```

### Temporal Fact Queries

```rust
// Query facts by time range
let time_range_facts = fact_store.query_facts(
    FactQuery::new()
        .with_time_range(
            time::now() - Duration::hours(1),
            time::now()
        )
)?;

// Query facts by domain
let domain_facts = fact_store.query_facts(
    FactQuery::new()
        .with_domain(external_domain_id.clone())
)?;

// Query facts by resource and state
let resource_state_facts = fact_store.query_facts(
    FactQuery::new()
        .with_resource(resource_id.clone())
        .with_state(RegisterState::Active)
)?;

// Query facts by operation type
let operation_facts = fact_store.query_facts(
    FactQuery::new()
        .with_operation_type(OperationType::TransferResource)
)?;

// Combine multiple criteria
let complex_query_facts = fact_store.query_facts(
    FactQuery::new()
        .with_resource(resource_id.clone())
        .with_time_range(
            time::now() - Duration::days(1),
            time::now()
        )
        .with_creator(creator_id.clone())
        .with_dependency(prerequisite_fact_id.clone())
)?;
```

### Building a Temporal Graph

```rust
// Build a graph of temporal relationships
let graph_builder = TemporalGraphBuilder::new(
    fact_store.clone(),
    GraphConfig::default()
);

// Create a graph from a set of facts
let temporal_graph = graph_builder.build_graph(
    vec![fact_id1.clone(), fact_id2.clone(), fact_id3.clone()]
)?;

// Analyze the graph
let roots = temporal_graph.find_roots()?;
let leaves = temporal_graph.find_leaves()?;
let paths = temporal_graph.find_all_paths(
    &fact_id1,
    &fact_id3
)?;

// Check temporal properties
let is_consistent = temporal_graph.is_consistent()?;
let has_cycles = temporal_graph.has_cycles()?;
let causal_order = temporal_graph.compute_causal_order()?;

// Visualize the graph
let dot_representation = temporal_graph.to_dot()?;
println!("Temporal graph: {}", dot_representation);
```

### Temporal Snapshots

```rust
// Create a snapshot of the temporal state
let snapshot = TemporalSnapshot::create(
    fact_store.clone(),
    time::now(),
    SnapshotConfig::default()
)?;

// Validate a snapshot
let is_valid_snapshot = temporal_validator.validate_snapshot(&snapshot)?;
if is_valid_snapshot {
    // Store the snapshot for later reference
    let snapshot_id = fact_store.store_snapshot(snapshot.clone())?;
    
    // Create a fact referencing the snapshot
    let snapshot_fact = TemporalFact::new(
        FactData::Custom {
            data_type: "temporal_snapshot".to_string(),
            data: snapshot.encode()?,
        },
        TemporalContext::new()
            .with_observed_at(time::now())
            .with_valid_from(time::now()),
        FactProvenance::new(
            system_id.clone(),
            local_domain_id.clone(),
            None,
            vec![]
        ),
        vec![] // Snapshot implicitly depends on all included facts
    )?;
    
    fact_store.store(snapshot_fact)?;
}
```

## Temporal Verification

The unified model enables comprehensive temporal verification:

```rust
// Create a temporal proof system
let proof_system = TemporalProofSystem::new(
    ProofSystemConfig::default()
);

// Create a proof for a set of facts
let temporal_proof = proof_system.create_proof(
    fact_store.clone(),
    vec![fact_id1.clone(), fact_id2.clone()],
    ProofCreationConfig::default()
)?;

// Verify the proof
let verification_result = proof_system.verify_proof(
    &temporal_proof,
    ProofVerificationConfig::default()
)?;

if verification_result.is_valid {
    println!("Temporal proof verified successfully");
} else {
    println!("Proof verification failed: {:?}", verification_result.reason());
}

// Create and verify a cross-domain proof
let cross_domain_proof = proof_system.create_cross_domain_proof(
    fact_store.clone(),
    vec![external_fact_id.clone(), reference_fact_id.clone()],
    external_domain_id.clone(),
    CrossDomainProofConfig::default()
)?;

let external_verification = external_verifier.verify_cross_domain_proof(
    &cross_domain_proof
)?;

println!("External verification: {:?}", external_verification);
```

## Best Practices

1. **Record Temporal Context**: Always include accurate temporal context with facts.

2. **Establish Clear Dependencies**: Explicitly define fact dependencies for proper causal ordering.

3. **Validate Temporal Consistency**: Validate temporal consistency before executing operations.

4. **Use Vector Clocks**: Use vector clocks for distributed systems to establish partial ordering.

5. **Record Operation Facts**: Always record operations as facts for audit and consistency.

6. **Query Efficiently**: Use specific queries rather than retrieving all facts.

7. **Manage Fact Lifecycle**: Implement archiving strategies for old facts.

8. **Verify External Facts**: Always verify external domain facts before using them.

9. **Use Snapshots**: Create temporal snapshots for efficient verification.

10. **Design for Scalability**: Consider performance implications of fact storage and queries.

## Security Considerations

1. **Fact Provenance**: Verify fact provenance to prevent unauthorized fact creation.

2. **Temporal Consistency**: Validate temporal consistency to prevent time-based attacks.

3. **Authorization**: Ensure proper authorization for fact creation and observation.

4. **Cross-Domain Verification**: Implement robust verification for cross-domain facts.

5. **Dependency Validation**: Validate dependencies to prevent temporal spoofing.

## Implementation Status

The unified temporal facts model is fully implemented in the Causality system:

- ✅ Core `TemporalFact` structure
- ✅ Fact data types and temporal context
- ✅ Fact storage and retrieval
- ✅ Dependency management
- ✅ Temporal validation
- ✅ Cross-domain observation
- ✅ Query capabilities
- ✅ Temporal graph analysis
- ✅ Snapshot management
- ✅ Proof system integration

## Future Enhancements

1. **Distributed Fact Consensus**: Enhanced consensus mechanisms for distributed fact verification
2. **Privacy-Preserving Facts**: Support for zero-knowledge proofs in temporal facts
3. **Temporal Compression**: Efficient compression of temporal fact history
4. **Adaptive Synchronization**: Adaptive cross-domain synchronization based on fact importance
5. **Temporal Prediction**: Prediction of future facts based on historical patterns
6. **Conflict Resolution**: Advanced conflict resolution for concurrent fact creation
7. **Fact Streaming**: Real-time streaming of temporal facts for live systems
8. **Temporal Schema Validation**: Schema-based validation for fact data
9. **Optimized Storage**: Enhanced storage strategies for high-throughput systems
10. **Advanced Pruning**: Intelligent pruning algorithms that maintain validation correctness 