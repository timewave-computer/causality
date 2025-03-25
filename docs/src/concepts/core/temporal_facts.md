<!-- Temporal facts -->
<!-- Original file: docs/src/temporal_facts.md -->

# Temporal Facts in Causality

## Overview

This document explains the temporal facts model within the Causality architecture. Temporal facts provide an immutable, time-aware record of events, state changes, and operations across the system. They form the foundation of Causality's temporal consistency model, enabling precise reasoning about causality, state evolution, and cross-domain synchronization.

## Core Concepts

### Temporal Fact Definition

A temporal fact is an immutable record of a specific event or state change that occurred at a particular point in time:

```rust
pub struct TemporalFact {
    /// Unique identifier for this fact
    id: FactId,
    
    /// Type of the fact
    fact_type: FactType,
    
    /// Timestamp when the fact was created
    timestamp: Timestamp,
    
    /// Domain that created the fact
    origin_domain: DomainId,
    
    /// Content of the fact
    content: FactContent,
    
    /// Cryptographic proof of the fact
    proof: Option<FactProof>,
    
    /// References to other facts this fact depends on
    dependencies: Vec<FactId>,
    
    /// Optional metadata associated with this fact
    metadata: HashMap<String, Value>,
}
```

### Fact Types

Causality supports various types of temporal facts:

```rust
pub enum FactType {
    /// A fact about a state change
    StateChange {
        /// Resource ID that changed
        resource_id: ResourceId,
        /// Type of state change
        change_type: StateChangeType,
    },
    
    /// A fact about an operation execution
    Operation {
        /// Operation ID
        operation_id: OperationId,
        /// Operation type
        operation_type: OperationType,
    },
    
    /// A fact about a transaction result
    Transaction {
        /// Transaction ID
        transaction_id: TransactionId,
        /// Transaction status
        status: TransactionStatus,
    },
    
    /// A fact about a cross-domain message
    CrossDomain {
        /// Message ID
        message_id: MessageId,
        /// Target domain
        target_domain: DomainId,
    },
    
    /// A fact about a validation result
    Validation {
        /// Subject of validation
        subject_id: SubjectId,
        /// Validation result
        result: ValidationResult,
    },
    
    /// Custom fact type for domain-specific facts
    Custom {
        /// Type identifier
        type_id: String,
        /// Additional type information
        type_data: Vec<u8>,
    },
}
```

### Fact Content

The content of a fact contains the actual data:

```rust
pub enum FactContent {
    /// JSON-encoded content
    Json(String),
    
    /// Binary content
    Binary(Vec<u8>),
    
    /// Content hash (for large content stored elsewhere)
    ContentHash {
        /// Hash of the content
        hash: Hash,
        /// Location where the full content can be retrieved
        location: ContentLocation,
    },
}
```

### Fact Proofs

Facts can include cryptographic proofs for verification:

```rust
pub enum FactProof {
    /// Signature-based proof
    Signature {
        /// Signature data
        signature: Vec<u8>,
        /// Public key that created the signature
        public_key: PublicKey,
    },
    
    /// Merkle proof
    MerkleProof {
        /// Merkle root
        root: Hash,
        /// Proof path
        path: Vec<(bool, Hash)>,
    },
    
    /// Zero-knowledge proof
    ZkProof {
        /// ZK proof data
        proof: ZkProof,
    },
    
    /// Custom proof type
    Custom {
        /// Proof type identifier
        proof_type: String,
        /// Proof data
        data: Vec<u8>,
    },
}
```

## Temporal Fact System

### Fact Registry

The Fact Registry stores and indexes all temporal facts:

```rust
pub struct FactRegistry {
    /// Storage for all facts
    facts: HashMap<FactId, TemporalFact>,
    
    /// Index of facts by resource
    resource_index: HashMap<ResourceId, Vec<FactId>>,
    
    /// Index of facts by timestamp
    time_index: BTreeMap<Timestamp, Vec<FactId>>,
    
    /// Index of facts by origin domain
    domain_index: HashMap<DomainId, Vec<FactId>>,
    
    /// Index of facts by type
    type_index: HashMap<FactTypeKey, Vec<FactId>>,
}

impl FactRegistry {
    /// Register a new fact
    pub fn register_fact(&mut self, fact: TemporalFact) -> Result<(), FactError> {
        // Validate the fact
        self.validate_fact(&fact)?;
        
        // Store the fact
        let fact_id = fact.id;
        
        // Update indices
        if let FactType::StateChange { resource_id, .. } = &fact.fact_type {
            self.resource_index
                .entry(*resource_id)
                .or_insert_with(Vec::new)
                .push(fact_id);
        }
        
        self.time_index
            .entry(fact.timestamp)
            .or_insert_with(Vec::new)
            .push(fact_id);
            
        self.domain_index
            .entry(fact.origin_domain)
            .or_insert_with(Vec::new)
            .push(fact_id);
            
        let type_key = FactTypeKey::from(&fact.fact_type);
        self.type_index
            .entry(type_key)
            .or_insert_with(Vec::new)
            .push(fact_id);
            
        // Store the fact
        self.facts.insert(fact_id, fact);
        
        Ok(())
    }
    
    /// Query facts by various criteria
    pub fn query_facts(
        &self,
        filter: FactFilter,
        limit: Option<usize>,
    ) -> Vec<&TemporalFact> {
        // Implementation of fact query logic
        // ...
    }
    
    /// Get facts for a specific resource
    pub fn get_facts_for_resource(
        &self,
        resource_id: ResourceId,
        time_range: Option<TimeRange>,
    ) -> Vec<&TemporalFact> {
        let fact_ids = match self.resource_index.get(&resource_id) {
            Some(ids) => ids,
            None => return Vec::new(),
        };
        
        let mut result = Vec::new();
        
        for id in fact_ids {
            if let Some(fact) = self.facts.get(id) {
                if let Some(range) = &time_range {
                    if fact.timestamp < range.start || fact.timestamp > range.end {
                        continue;
                    }
                }
                result.push(fact);
            }
        }
        
        // Sort by timestamp
        result.sort_by_key(|f| f.timestamp);
        
        result
    }
    
    /// Get a specific fact by ID
    pub fn get_fact(&self, fact_id: &FactId) -> Option<&TemporalFact> {
        self.facts.get(fact_id)
    }
    
    // Additional methods...
}
```

### Fact Observer

The Fact Observer handles the creation and propagation of facts:

```rust
pub struct FactObserver {
    registry: FactRegistry,
    fact_validators: Vec<Box<dyn FactValidator>>,
    event_handlers: HashMap<FactTypeKey, Vec<Box<dyn FactEventHandler>>>,
}

impl FactObserver {
    /// Create a new fact about a state change
    pub fn observe_state_change(
        &mut self,
        resource_id: ResourceId,
        change_type: StateChangeType,
        content: FactContent,
        dependencies: Vec<FactId>,
    ) -> Result<FactId, FactError> {
        let fact_id = FactId::generate();
        
        let fact = TemporalFact {
            id: fact_id,
            fact_type: FactType::StateChange {
                resource_id,
                change_type,
            },
            timestamp: system.current_time(),
            origin_domain: system.domain_id(),
            content,
            proof: self.generate_proof(fact_id, &content)?,
            dependencies,
            metadata: HashMap::new(),
        };
        
        // Validate the fact
        for validator in &self.fact_validators {
            validator.validate_fact(&fact)?;
        }
        
        // Register the fact
        self.registry.register_fact(fact.clone())?;
        
        // Notify event handlers
        self.notify_handlers(&fact)?;
        
        Ok(fact_id)
    }
    
    /// Create a new fact about an operation
    pub fn observe_operation(
        &mut self,
        operation_id: OperationId,
        operation_type: OperationType,
        content: FactContent,
        dependencies: Vec<FactId>,
    ) -> Result<FactId, FactError> {
        // Similar implementation to observe_state_change
        // ...
    }
    
    /// Generate a cryptographic proof for a fact
    fn generate_proof(&self, fact_id: FactId, content: &FactContent) -> Result<Option<FactProof>, FactError> {
        // Actual proof generation logic depends on the security model
        // ...
        
        Ok(Some(FactProof::Signature {
            signature: system.sign(content.hash())?,
            public_key: system.public_key(),
        }))
    }
    
    /// Notify relevant event handlers of a new fact
    fn notify_handlers(&self, fact: &TemporalFact) -> Result<(), FactError> {
        let type_key = FactTypeKey::from(&fact.fact_type);
        
        if let Some(handlers) = self.event_handlers.get(&type_key) {
            for handler in handlers {
                handler.handle_fact(fact)?;
            }
        }
        
        Ok(())
    }
    
    // Additional methods...
}
```

## Working with Temporal Facts

### Creating Facts

Temporal facts are created when significant events occur:

```rust
/// Create a fact about a resource state change
pub fn record_resource_state_change(
    resource_id: ResourceId,
    old_state: ResourceState,
    new_state: ResourceState,
    change_reason: &str,
) -> Result<FactId, FactError> {
    // Create the content of the fact
    let content = FactContent::Json(serde_json::to_string(&StateChangeData {
        old_state,
        new_state,
        reason: change_reason.to_string(),
        timestamp: system.current_time(),
    })?);
    
    // Determine dependencies (previous facts about this resource)
    let latest_facts = fact_registry.get_latest_facts_for_resource(resource_id, 1)?;
    let dependencies = latest_facts.into_iter().map(|f| f.id).collect();
    
    // Create the fact
    let fact_id = fact_observer.observe_state_change(
        resource_id,
        StateChangeType::StateUpdate,
        content,
        dependencies,
    )?;
    
    Ok(fact_id)
}
```

### Querying Facts

Facts can be queried to understand the history of the system:

```rust
/// Get the history of a resource's state changes
pub fn get_resource_state_history(
    resource_id: ResourceId,
    time_range: Option<TimeRange>,
) -> Result<Vec<ResourceStateChange>, FactError> {
    // Query facts for this resource
    let facts = fact_registry.get_facts_for_resource(
        resource_id,
        time_range,
    );
    
    // Extract state changes from facts
    let mut state_changes = Vec::new();
    
    for fact in facts {
        if let FactType::StateChange { change_type, .. } = &fact.fact_type {
            if *change_type == StateChangeType::StateUpdate {
                // Parse the fact content
                if let FactContent::Json(json) = &fact.content {
                    let change_data: StateChangeData = serde_json::from_str(json)?;
                    
                    state_changes.push(ResourceStateChange {
                        fact_id: fact.id,
                        timestamp: fact.timestamp,
                        old_state: change_data.old_state,
                        new_state: change_data.new_state,
                        reason: change_data.reason,
                    });
                }
            }
        }
    }
    
    // Sort by timestamp
    state_changes.sort_by_key(|c| c.timestamp);
    
    Ok(state_changes)
}
```

### Temporal Reasoning

Facts enable reasoning about causality and temporal relationships:

```rust
/// Check if one fact happened before another
pub fn is_fact_before(
    fact1_id: FactId,
    fact2_id: FactId,
) -> Result<bool, FactError> {
    let fact1 = fact_registry.get_fact(&fact1_id)
        .ok_or(FactError::FactNotFound(fact1_id))?;
    
    let fact2 = fact_registry.get_fact(&fact2_id)
        .ok_or(FactError::FactNotFound(fact2_id))?;
    
    // Check temporal relationship
    Ok(fact1.timestamp < fact2.timestamp)
}

/// Check if a fact causally depends on another
pub fn is_fact_dependent_on(
    fact_id: FactId,
    dependency_id: FactId,
) -> Result<bool, FactError> {
    let fact = fact_registry.get_fact(&fact_id)
        .ok_or(FactError::FactNotFound(fact_id))?;
    
    // Check direct dependency
    if fact.dependencies.contains(&dependency_id) {
        return Ok(true);
    }
    
    // Check transitive dependencies
    for dep_id in &fact.dependencies {
        if is_fact_dependent_on(*dep_id, dependency_id)? {
            return Ok(true);
        }
    }
    
    Ok(false)
}
```

## Temporal Consistency

### Consistency Validation

Facts are used to validate temporal consistency:

```rust
/// Validate temporal consistency for an operation
pub fn validate_temporal_consistency(
    operation: &Operation,
    auth_context: &AuthContext,
) -> Result<ValidationResult, ValidationError> {
    // Extract resource ID from operation
    let resource_id = operation.resource_id();
    
    // Get the latest facts for this resource
    let latest_facts = fact_registry.get_latest_facts_for_resource(resource_id, 5)?;
    
    // Get the latest state from facts
    let current_state = if let Some(latest) = latest_facts.first() {
        if let FactType::StateChange { .. } = &latest.fact_type {
            if let FactContent::Json(json) = &latest.content {
                let change_data: StateChangeData = serde_json::from_str(json)?;
                Some(change_data.new_state)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    
    // Validate operation against current state
    if let Some(state) = current_state {
        // Check if operation is valid for the current state
        if !is_operation_valid_for_state(operation, &state) {
            return Ok(ValidationResult::Failure(
                "Operation not valid for current resource state".to_string(),
            ));
        }
    }
    
    // Check for concurrent operations
    let concurrent_ops = fact_registry.get_concurrent_operations(
        resource_id,
        operation.timestamp(),
        ConcurrencyWindow::default(),
    )?;
    
    if !concurrent_ops.is_empty() {
        // Check for conflicts with concurrent operations
        if has_conflicts_with_concurrent_ops(operation, &concurrent_ops) {
            return Ok(ValidationResult::Failure(
                "Operation conflicts with concurrent operations".to_string(),
            ));
        }
    }
    
    Ok(ValidationResult::Success)
}
```

### Temporal Invariants

Facts are used to enforce temporal invariants:

```rust
/// Define a temporal invariant
pub struct TemporalInvariant {
    /// Invariant ID
    id: InvariantId,
    
    /// Description
    description: String,
    
    /// Resource types this invariant applies to
    applicable_resources: Vec<ResourceType>,
    
    /// Invariant checking function
    checker: Box<dyn Fn(&[&TemporalFact]) -> Result<bool, ValidationError>>,
}

/// Check if temporal invariants hold for a resource
pub fn check_temporal_invariants(
    resource_id: ResourceId,
) -> Result<Vec<InvariantResult>, ValidationError> {
    // Get resource type
    let resource = registry.get_resource(resource_id)?;
    let resource_type = resource.resource_type();
    
    // Get applicable invariants
    let invariants = invariant_registry.get_invariants_for_resource_type(&resource_type)?;
    
    // Get facts for this resource
    let facts = fact_registry.get_facts_for_resource(
        resource_id,
        None, // Consider all time
    );
    
    // Check each invariant
    let mut results = Vec::new();
    
    for invariant in invariants {
        let holds = (invariant.checker)(&facts)?;
        
        results.push(InvariantResult {
            invariant_id: invariant.id,
            resource_id,
            holds,
            timestamp: system.current_time(),
        });
    }
    
    Ok(results)
}
```

## Cross-Domain Fact Synchronization

Facts are synchronized across domains to maintain global consistency:

```rust
/// Synchronize a fact with another domain
pub fn synchronize_fact(
    fact_id: FactId,
    target_domain: DomainId,
) -> Result<(), SyncError> {
    // Get the fact
    let fact = fact_registry.get_fact(&fact_id)
        .ok_or(SyncError::FactNotFound(fact_id))?;
    
    // Create a synchronization message
    let sync_message = CrossDomainMessage::FactSync {
        fact: fact.clone(),
        origin_domain: system.domain_id(),
        timestamp: system.current_time(),
    };
    
    // Send the message
    cross_domain_messenger.send_message(target_domain, sync_message)?;
    
    // Record the synchronization
    fact_sync_registry.record_sync(fact_id, target_domain, system.current_time())?;
    
    Ok(())
}

/// Process an incoming fact from another domain
pub fn process_remote_fact(
    fact: TemporalFact,
    origin_domain: DomainId,
) -> Result<(), SyncError> {
    // Validate the remote fact
    for validator in &fact_validators {
        validator.validate_remote_fact(&fact, origin_domain)?;
    }
    
    // Check if we already have this fact
    if fact_registry.contains_fact(&fact.id) {
        return Ok(());
    }
    
    // Register the fact
    fact_registry.register_fact(fact.clone())?;
    
    // Process any dependencies
    for dep_id in &fact.dependencies {
        if !fact_registry.contains_fact(dep_id) {
            // Request missing dependency
            request_missing_fact(*dep_id, origin_domain)?;
        }
    }
    
    // Notify handlers about the new fact
    fact_observer.notify_handlers(&fact)?;
    
    Ok(())
}
```

## Usage Examples

### Recording Resource Creation

```rust
// Create a new resource
let resource_id = resource_manager.create_resource(
    resource_type,
    initial_attributes,
    owner_id,
)?;

// Record the creation as a fact
let content = FactContent::Json(serde_json::to_string(&ResourceCreationData {
    resource_type,
    attributes: initial_attributes,
    owner: owner_id,
})?);

let fact_id = fact_observer.observe_state_change(
    resource_id,
    StateChangeType::Creation,
    content,
    Vec::new(), // No dependencies for initial creation
)?;

println!("Resource created and recorded as fact: {}", fact_id);
```

### Querying Operation History

```rust
// Query all operations performed on a resource
let operation_facts = fact_registry.query_facts(
    FactFilter::new()
        .with_resource_id(resource_id)
        .with_fact_type(FactTypeKey::Operation)
        .with_time_range(TimeRange::new(
            system.current_time() - Duration::days(7),
            system.current_time(),
        )),
    Some(100), // Limit to 100 results
);

println!("Found {} operations on resource in the last week", operation_facts.len());

// Extract operation details
for fact in operation_facts {
    if let FactType::Operation { operation_id, operation_type } = &fact.fact_type {
        if let FactContent::Json(json) = &fact.content {
            let operation_data: OperationData = serde_json::from_str(json)?;
            
            println!("Operation {} of type {} performed at {} by {}",
                operation_id,
                operation_type,
                fact.timestamp,
                operation_data.performer,
            );
        }
    }
}
```

### Validating Time-Based Constraints

```rust
// Check if a time-based capability has expired
pub fn is_capability_valid(
    capability_id: CapabilityId,
) -> Result<bool, ValidationError> {
    // Get the capability
    let capability = capability_registry.get_capability(capability_id)?;
    
    // Get the creation fact for this capability
    let creation_facts = fact_registry.query_facts(
        FactFilter::new()
            .with_resource_id(ResourceId::from(capability_id))
            .with_fact_type(FactTypeKey::StateChange)
            .with_change_type(StateChangeType::Creation),
        Some(1),
    );
    
    if let Some(creation_fact) = creation_facts.first() {
        // Get creation time
        let creation_time = creation_fact.timestamp;
        
        // Parse expiration from capability
        if let Some(expiration) = capability.expiration() {
            // Check if the capability has expired
            return Ok(system.current_time() < expiration);
        }
    }
    
    // Default to valid if no time constraints found
    Ok(true)
}
```

## Implementation Status

The following components of the temporal fact system have been implemented:

- ✅ Core temporal fact model
- ✅ Fact registry and storage
- ✅ Basic fact creation and querying
- ⚠️ Fact validation (partially implemented)
- ⚠️ Cross-domain fact synchronization (partially implemented)
- ❌ Advanced temporal reasoning (not yet implemented)
- ❌ Temporal invariant enforcement (not yet implemented)

## Future Enhancements

Future enhancements to the temporal fact system include:

1. **Hierarchical Fact Modeling**: Support for hierarchical relationships between facts
2. **Fact Compression**: Advanced storage and compression techniques for historical facts
3. **Temporal Query Language**: Domain-specific language for complex temporal queries
4. **Probabilistic Temporal Reasoning**: Handling uncertainty in temporal relationships
5. **Zero-Knowledge Fact Proofs**: Privacy-preserving fact verification
6. **Distributed Fact Consensus**: Advanced consensus mechanisms for fact agreement across domains
7. **Temporal Visualization Tools**: Tools for visualizing temporal fact relationships and causality chains 