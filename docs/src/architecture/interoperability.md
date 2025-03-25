<!-- Interoperability between systems -->
<!-- Original file: docs/src/interoperability.md -->

# Cross-Domain Interoperability

This document outlines the interoperability mechanisms within the Causality architecture, focusing on how resources, operations, and facts are shared and synchronized across different domains.

## Core Concepts

### Domains

A **Domain** represents a distinct execution environment with its own:
- Consensus mechanism
- State management
- Resource governance
- Security model

Domains can be heterogeneous (e.g., EVM chains, CosmWasm chains, TEL environments) or homogeneous (e.g., different instances of the same chain type).

### Cross-Domain Communication

The architecture supports several communication patterns:
- **Message Passing**: Direct transfer of information between domains
- **Fact Observation**: Observation of facts across domain boundaries
- **Shared Resources**: Resources accessible from multiple domains
- **Cross-Domain Operations**: Operations that affect resources in multiple domains

## Domain Bridge Architecture

### Components

1. **Domain Adapter**: Provides a standardized interface to domain-specific functionality
2. **Bridge Protocol**: Defines how messages are exchanged between domains
3. **Translator**: Converts between domain-specific and canonical data formats
4. **Verifier**: Validates cross-domain messages and state proofs
5. **Synchronizer**: Ensures temporal consistency across domains

### Bridge Protocol

```rust
/// Core protocol for cross-domain messaging
pub trait BridgeProtocol: Send + Sync {
    /// Send a message to another domain
    fn send_message(&self, target_domain: &DomainId, message: Message) -> Result<MessageId>;
    
    /// Receive messages from another domain
    fn receive_messages(&self, source_domain: &DomainId) -> Result<Vec<Message>>;
    
    /// Verify a message from another domain
    fn verify_message(&self, message: &Message) -> Result<VerificationResult>;
    
    /// Check if a domain is supported
    fn supports_domain(&self, domain_id: &DomainId) -> bool;
}

/// A cross-domain message
pub struct Message {
    /// Unique identifier
    id: MessageId,
    /// Source domain
    source_domain: DomainId,
    /// Target domain
    target_domain: DomainId,
    /// Message payload
    payload: MessagePayload,
    /// Verification data
    proof: Option<Proof>,
    /// Message metadata
    metadata: HashMap<String, Value>,
}
```

## Resource Interoperability

### Resource Projection

Resources can be projected across domains through:

1. **Shadow Resources**: Read-only projections of resources from another domain
2. **Bridged Resources**: Mutable projections with synchronized state
3. **Locked Resources**: Resources locked in one domain and represented in another

### Implementation

```rust
/// Manager for cross-domain resources
pub struct CrossDomainResourceManager {
    /// Domain registry
    domain_registry: Arc<DomainRegistry>,
    /// Resource registry
    resource_registry: Arc<ResourceRegistry>,
    /// Bridge protocols by domain pair
    bridge_protocols: HashMap<(DomainId, DomainId), Box<dyn BridgeProtocol>>,
}

impl CrossDomainResourceManager {
    /// Project a resource from one domain to another
    pub fn project_resource(
        &self,
        resource_id: &ResourceId,
        source_domain: &DomainId,
        target_domain: &DomainId,
        projection_type: ProjectionType,
    ) -> Result<ResourceId>;
    
    /// Synchronize a resource across domains
    pub fn synchronize_resource(
        &self,
        resource_id: &ResourceId,
        domains: &[DomainId],
    ) -> Result<Vec<ResourceSyncResult>>;
}
```

## Operation Interoperability

Cross-domain operations ensure that actions affecting resources in multiple domains maintain consistency.

### Operation Types

1. **Atomic Cross-Domain Operations**: Operations that either succeed in all domains or fail in all
2. **Sequential Cross-Domain Operations**: Operations executed in a specific order across domains
3. **Conditional Cross-Domain Operations**: Operations in one domain that depend on conditions in another

### Implementation

```rust
/// Service for cross-domain operations
pub struct CrossDomainOperationService {
    /// Domain registry
    domain_registry: Arc<DomainRegistry>,
    /// Operation transformation service
    transformation_service: Arc<OperationTransformationService>,
    /// Cross-domain verifier
    verifier: Arc<CrossDomainVerifier>,
}

impl CrossDomainOperationService {
    /// Execute an operation across multiple domains
    pub fn execute_cross_domain(
        &self,
        operation: &Operation,
        domains: &[DomainId],
        strategy: CrossDomainStrategy,
    ) -> Result<CrossDomainExecutionResult>;
}
```

## Fact Interoperability

Facts can be observed and synchronized across domains to maintain temporal consistency.

### Fact Propagation

1. **Fact Broadcasting**: Facts are broadcast to relevant domains
2. **Fact Subscription**: Domains subscribe to facts from other domains
3. **Fact Aggregation**: Facts from multiple domains are aggregated

### Implementation

```rust
/// Service for cross-domain fact synchronization
pub struct CrossDomainFactService {
    /// Fact store
    fact_store: Arc<FactStore>,
    /// Domain registry
    domain_registry: Arc<DomainRegistry>,
    /// Fact observer registry
    observer_registry: Arc<FactObserverRegistry>,
}

impl CrossDomainFactService {
    /// Observe facts from another domain
    pub fn observe_facts(
        &self,
        source_domain: &DomainId,
        filter: Option<FactFilter>,
    ) -> Result<Vec<TemporalFact>>;
    
    /// Propagate facts to another domain
    pub fn propagate_facts(
        &self,
        facts: &[TemporalFact],
        target_domain: &DomainId,
    ) -> Result<Vec<FactPropagationResult>>;
}
```

## Security Model

Cross-domain interoperability relies on several security mechanisms:

1. **Capability-Based Authorization**: Validates capabilities across domain boundaries
2. **Cryptographic Verification**: Ensures messages are authentic and unaltered
3. **Temporal Validation**: Maintains happened-before relationships across domains
4. **Domain Trust Levels**: Defines how much one domain trusts another

## Usage Examples

### Bridging a Resource

```rust
// Create a bridge between two domains
let bridge = CrossDomainResourceManager::new(
    domain_registry.clone(),
    resource_registry.clone()
);

// Project a resource from one domain to another
let projected_resource_id = bridge.project_resource(
    &resource_id,
    &source_domain.id(),
    &target_domain.id(),
    ProjectionType::Bridged {
        update_strategy: UpdateStrategy::Immediate,
        conflict_resolution: ConflictResolution::SourceDomainPriority,
    }
)?;

// Now we can access the resource in the target domain
let resource = resource_registry.get_resource(&projected_resource_id)?;
```

### Cross-Domain Transfer

```rust
// Create a cross-domain operation service
let cross_domain_service = CrossDomainOperationService::new(
    domain_registry.clone(),
    transformation_service.clone(),
    verifier.clone()
);

// Create a transfer operation
let transfer_effect = TransferEffect::new(
    source_resource.id(),
    destination_resource.id(),
    100,
    HashMap::new()
);

let operation = Operation::new(OperationType::TransferResource)
    .with_input(source_resource.clone())
    .with_output(destination_resource.clone())
    .with_abstract_representation(Box::new(transfer_effect))
    .with_authorization(Authorization::with_capabilities(
        invoker.clone(),
        vec![transfer_capability]
    ));

// Execute across domains
let result = cross_domain_service.execute_cross_domain(
    &operation,
    &[source_domain.id(), destination_domain.id()],
    CrossDomainStrategy::AtomicCommit {
        timeout: Duration::from_secs(30),
        verification_level: VerificationLevel::Full,
    }
)?;

// Check the result
if result.success {
    println!("Cross-domain transfer successful");
} else {
    println!("Cross-domain transfer failed: {}", result.error.unwrap_or_default());
}
```

### Observing Cross-Domain Facts

```rust
// Create a cross-domain fact service
let fact_service = CrossDomainFactService::new(
    fact_store.clone(),
    domain_registry.clone(),
    observer_registry.clone()
);

// Create a fact filter for specific resource facts
let filter = FactFilter::new()
    .with_resource(resource_id.clone())
    .with_fact_type("transfer")
    .with_time_range(TimeRange::since(timestamp));

// Observe facts from another domain
let facts = fact_service.observe_facts(
    &source_domain.id(),
    Some(filter)
)?;

// Process the observed facts
for fact in facts {
    println!("Observed fact: {} at {}", fact.fact_type(), fact.timestamp());
    
    // Validate temporal consistency
    if fact_validator.validate_temporal_consistency(&fact, &local_facts)? {
        // Fact is temporally consistent, apply it locally
        fact_store.store_fact(fact)?;
    } else {
        // Handle temporal inconsistency
        conflict_resolver.resolve_temporal_conflict(&fact, &local_facts)?;
    }
}
```

## Implementation Status

The cross-domain interoperability framework is partially implemented:

- ✅ Domain adapter interface
- ✅ Resource projection mechanisms
- ✅ Cross-domain fact observation
- ✅ Basic cross-domain operations
- ✅ Capability-based security model
- ⚠️ Atomic cross-domain operations (in progress)
- ⚠️ Advanced conflict resolution (in progress)
- ❌ Cross-domain transaction rollback
- ❌ Optimistic cross-domain execution

## Future Enhancements

1. **Cross-Domain Transaction Isolation**: Ensure transactions spanning multiple domains maintain isolation properties
2. **Optimistic Execution**: Execute operations optimistically and roll back if necessary
3. **Domain-Specific Adapters**: Specialized adapters for common blockchain platforms
4. **Performance Optimizations**: Batching and caching for cross-domain operations
5. **Heterogeneous Domain Consensus**: Consensus protocols spanning different domain types 