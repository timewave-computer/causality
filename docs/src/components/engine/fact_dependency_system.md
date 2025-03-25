<!-- System for fact dependencies -->
<!-- Original file: docs/src/fact_dependency_system.md -->

# Fact Dependency System in Causality

This document describes the implementation of the Fact Dependency System in the Causality Unified Log System.

## Overview

The Fact Dependency System tracks causal relationships between facts observed in external domains and effects executed in the Causality system. These relationships enable important capabilities:

1. **Causal Tracking**: Clear trace of which facts enabled which effects
2. **Verification**: Ensuring effects only execute when required facts exist
3. **Observability**: Monitoring fact dependencies for transparency
4. **Replayability**: Reproducible system state by tracking fact dependencies
5. **Auditability**: Evidence of what caused specific system actions

## Core Components

### FactId and FactDependency

```rust
/// A unique identifier for a fact
pub struct FactId(pub String);

/// A struct representing a dependency on a fact
pub struct FactDependency {
    /// The ID of the fact
    pub fact_id: FactId,
    
    /// The type of dependency
    pub dependency_type: FactDependencyType,
    
    /// The domain the fact comes from
    pub domain_id: DomainId,
    
    /// The type of the fact (optional)
    pub fact_type: Option<FactType>,
}
```

Each fact has a unique identifier. Effects declare dependencies on facts through `FactDependency` structures that specify the nature of the dependency.

### FactDependencyType

```rust
/// Fact dependency type, used to indicate why a fact is needed
pub enum FactDependencyType {
    /// Fact is required for the effect to be valid
    Required,
    
    /// Fact is used by the effect but not strictly required
    Optional,
    
    /// Fact provides additional context for the effect
    Context,
}
```

The system supports different types of dependencies with varying levels of criticality.

### FactSnapshot

```rust
/// A snapshot of facts at a specific point in time
pub struct FactSnapshot {
    /// The facts that are included in this snapshot
    pub observed_facts: HashMap<FactId, String>,
    
    /// The observer that created this snapshot
    pub observer: String,
    
    /// Timestamp when the snapshot was created
    pub creation_timestamp: Timestamp,
    
    /// Observations about registers at this snapshot point
    pub register_observations: HashMap<RegisterId, RegisterObservation>,
    
    /// Domains that contributed to this snapshot
    pub contributing_domains: HashSet<DomainId>,
    
    /// Additional metadata for this snapshot
    pub metadata: HashMap<String, String>,
}

/// Details about an observed register
pub struct RegisterObservation {
    /// The register ID being observed
    pub register_id: RegisterId,
    
    /// The fact ID that last modified this register
    pub fact_id: FactId,
    
    /// The domain ID that wrote the register
    pub domain_id: DomainId,
    
    /// Time when the observation was made
    pub observation_time: Timestamp,
    
    /// Hash of the register data at observation time
    pub data_hash: String,
}
```

A `FactSnapshot` represents a consistent view of facts at a specific point in time, which can be attached to effects to document their dependencies explicitly.

### FactDependencyValidator

```rust
/// Validates fact dependencies for effects
pub struct FactDependencyValidator {
    /// Map of fact IDs to verified status
    fact_cache: HashMap<FactId, bool>,
    /// Map of register IDs to their latest observation
    register_observations: HashMap<ResourceId, RegisterObservation>,
    /// Map of domain IDs to their allowed maximum age (in seconds)
    domain_freshness: HashMap<DomainId, u64>,
}
```

The validator ensures that effects only execute when all their required fact dependencies are satisfied.

### FactEffectTracker

```rust
/// Tracker for fact-effect causal relationships
pub struct FactEffectTracker {
    /// Map of fact IDs to the effects that depend on them
    fact_to_effects: RwLock<HashMap<FactId, HashSet<String>>>,
    /// Map of effect IDs to the facts they depend on
    effect_to_facts: RwLock<HashMap<String, HashSet<FactId>>>,
    /// Map of resource IDs to the fact-effect relations that involve them
    resource_relations: RwLock<HashMap<ResourceId, HashSet<(FactId, String)>>>,
    /// Map of domain IDs to the fact-effect relations that involve them
    domain_relations: RwLock<HashMap<DomainId, HashSet<(FactId, String)>>>,
    /// Map of trace IDs to the fact-effect relations that involve them
    trace_relations: RwLock<HashMap<TraceId, HashSet<(FactId, String)>>>,
    /// Detailed relation information
    relations: RwLock<HashMap<(FactId, String), FactEffectRelation>>,
    /// Time-indexed facts
    time_indexed_facts: RwLock<BTreeMap<Timestamp, HashSet<FactId>>>,
    /// Time-indexed effects
    time_indexed_effects: RwLock<BTreeMap<Timestamp, HashSet<String>>>,
}
```

The tracker monitors and indexes all relationships between facts and effects, enabling efficient querying by fact, effect, resource, domain, trace, or time range.

## How It Works

### 1. Declaring Dependencies

Effects declare their fact dependencies in one of two ways:

#### Method 1: Using the Effect trait

```rust
pub trait Effect: Send + Sync {
    // ... other methods ...
    
    /// Get the fact dependencies for this effect
    fn fact_dependencies(&self) -> Vec<FactDependency> {
        Vec::new()
    }
    
    /// Get the fact snapshot for this effect
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        None
    }
}
```

Effects can override these methods to declare their dependencies.

#### Method 2: Attaching metadata to log entries

```rust
// Create an effect with fact dependencies
let mut metadata = HashMap::new();
metadata.insert(
    "fact_dependencies".to_string(),
    serde_json::to_string(&dependencies).unwrap(),
);

// Or attach a full snapshot
metadata.insert(
    "fact_snapshot".to_string(),
    serde_json::to_string(&snapshot).unwrap(),
);
```

Dependencies can be embedded as metadata in effect log entries.

### 2. Validating Dependencies

Before executing an effect, the system validates that all required fact dependencies are satisfied:

```rust
// Validate an effect's dependencies
validator.validate_effect_dependencies(&effect)?;

// Or validate a single dependency
validator.validate_dependency(&dependency)?;

// Or validate an entire snapshot
validator.validate_snapshot(&snapshot)?;
```

Validation ensures that:
- All required facts exist and are verified
- Register observations are up-to-date
- Domain freshness requirements are met

### 3. Tracking Causal Relationships

The `FactEffectTracker` monitors all log entries and builds comprehensive indexes:

```rust
// Track relationships from a log entry
tracker.track_entry(&entry)?;

// Or build a tracker from storage
let tracker = FactEffectTracker::build_from_storage(&storage)?;
```

This enables powerful queries:

```rust
// Get all effects that depend on a fact
let dependent_effects = tracker.get_dependent_effects(&fact_id)?;

// Get all facts that an effect depends on
let effect_dependencies = tracker.get_effect_dependencies(&effect_id)?;

// Get facts in a time range
let facts = tracker.get_facts_in_time_range(start_time, end_time)?;

// Create a snapshot for specific resources and domains
let snapshot = tracker.create_snapshot(&resources, &domains, "observer")?;
```

### 4. Snapshot Usage

Fact snapshots can be created from the current state of the system, capturing all relevant facts for a particular context:

```rust
// Create a snapshot manually
let mut snapshot = FactSnapshot::new("observer_committee");
snapshot.add_fact(fact_id, domain_id);
snapshot.add_register_observation(register_id, fact_id, domain_id, data_hash);

// Or create from the fact-effect tracker
let snapshot = tracker.create_snapshot(&resources, &domains, "observer_committee")?;

// Verify the snapshot
validator.validate_snapshot(&snapshot)?;
```

Snapshots provide consistent views of facts at specific points in time, enabling deterministic replay and verification.

## Integration with the Log System

The fact dependency system is deeply integrated with the Unified Log System:

1. **Log Entries**: Fact and effect entries in the log have clear relationships via dependencies
2. **Replay**: When replaying logs, fact dependencies are validated to ensure consistency
3. **Time Map**: Fact timestamps are verified against the global Map of Time
4. **Content Addressing**: Fact hashes ensure integrity of observations

## Example Usage Patterns

### Register a Fact and Dependent Effect

```rust
// 1. Observe and log a fact
let fact_id = FactId("ethereum_block_12345".to_string());
let domain_id = DomainId::new(1); // Ethereum
fact_logger.log_fact(&fact_entry)?;

// 2. Add fact to validator
let mut validator = FactDependencyValidator::new();
validator.add_fact(fact_id.clone(), true); // Verified fact

// 3. Create an effect that depends on the fact
let dependency = FactDependency::new(
    fact_id,
    domain_id,
    FactDependencyType::Required,
);

// 4. Validate and execute the effect
validator.validate_dependency(&dependency)?;
effect_logger.log_effect(effect_entry)?;

// 5. Track the relationship
let tracker = FactEffectTracker::new();
tracker.track_entry(&fact_entry)?;
tracker.track_entry(&effect_entry)?;
```

### Query Causal Relationships

```rust
// Find all effects caused by a specific fact
let dependent_effects = tracker.get_dependent_effects(&fact_id)?;

// Find all effects related to a resource
let resource_relations = tracker.get_resource_relations(&resource_id)?;

// Find all fact-effect relationships in a time range
let facts = tracker.get_facts_in_time_range(start_time, end_time)?;
let effects = tracker.get_effects_in_time_range(start_time, end_time)?;
```

### Creating Snapshots for Effects

```rust
// Create a snapshot for resources affected by an operation
let snapshot = tracker.create_snapshot(
    &[resource1, resource2],
    &[domain1, domain2],
    "committee_1",
)?;

// Attach snapshot to an effect
let mut metadata = HashMap::new();
metadata.insert(
    "fact_snapshot".to_string(),
    serde_json::to_string(&snapshot).unwrap(),
);
```

## Benefits

1. **Determinism**: System behavior is fully determined by documented facts and their effects
2. **Verifiability**: All effects have clear, verifiable preconditions
3. **Auditability**: Complete trail of what facts caused which effects
4. **Cross-Domain Consistency**: Facts from different domains are consistently referenced
5. **Replay Accuracy**: Snapshots enable precise state reconstruction

## Future Improvements

1. **Fact Materialization**: Optimized storage for frequently accessed facts
2. **Dependency Graph Visualization**: Visual tools to explore fact-effect relationships
3. **Conflict Detection**: Early detection of conflicting facts or dependencies
4. **Dependency Pruning**: Automatic removal of obsolete dependencies
5. **Probabilistic Dependencies**: Support for uncertain or probabilistic fact dependencies 