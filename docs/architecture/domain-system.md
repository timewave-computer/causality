# Domain System

*The Domain System enables Causality to interact with external blockchains and data sources in a uniform way.*

*Last updated: 2023-03-26*

## Overview

The Domain System is a core component of Causality that enables interoperability with external blockchains and data sources. It provides a unified interface for interacting with heterogeneous domains, abstracting away their specific implementation details and providing consistent semantics for cross-domain operations.

Key capabilities of the Domain System include:

1. **Uniform Interface**: A consistent API for interacting with diverse blockchains and data sources
2. **Cross-Domain Operations**: The ability to execute operations that span multiple domains
3. **Domain-Specific Adapters**: Translation of Causality operations into domain-specific operations
4. **State Observation**: Reliable observation of external state changes
5. **Boundary Crossing**: Secure transfer of data and effects across domain boundaries

The Domain System enables Causality applications to be domain-agnostic, allowing them to interact with multiple blockchains or data sources without being tightly coupled to any specific implementation.

## Core Concepts

### Domain

A Domain represents an external blockchain or data source with which Causality interacts:

```rust
/// A domain in the Causality system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    /// Domain ID (content-addressed)
    id: DomainId,
    /// Domain name
    name: String,
    /// Domain type
    domain_type: DomainType,
    /// Domain configuration
    config: DomainConfig,
    /// Domain state
    state: DomainState,
    /// Domain public keys
    public_keys: Vec<PublicKey>,
    /// Content hash
    content_hash: ContentHash,
}
```

Each domain has a unique, content-addressed identifier and contains configuration and state information specific to that domain.

### Domain Identifier

```rust
/// A unique identifier for a domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DomainId {
    /// Domain type
    domain_type: String,
    /// Domain name
    name: String,
    /// Content hash
    content_hash: ContentHash,
}
```

Domain identifiers are content-addressed and contain information about the domain type and name, making them self-describing.

### Domain Types

Causality supports various types of domains:

```rust
/// Types of domains
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DomainType {
    /// Blockchain domain
    Blockchain {
        /// Consensus mechanism
        consensus: ConsensusType,
        /// VM type
        vm_type: VmType,
    },
    /// Database domain
    Database {
        /// Database type
        db_type: DatabaseType,
    },
    /// File system domain
    FileSystem,
    /// Memory domain
    Memory,
    /// Custom domain
    Custom(String),
}
```

Each domain type has specific characteristics and capabilities that influence how Causality interacts with it.

## Domain System Architecture

### Domain Registry

The Domain Registry provides a central registry of all domains known to the system:

```rust
/// Registry of domains
#[async_trait]
pub trait DomainRegistry: Send + Sync + 'static {
    /// Register a new domain
    async fn register_domain(&self, domain: Domain) -> Result<DomainId, DomainError>;
    
    /// Get a domain by ID
    async fn get_domain(&self, id: &DomainId) -> Result<Option<Domain>, DomainError>;
    
    /// List all domains
    async fn list_domains(&self) -> Result<Vec<Domain>, DomainError>;
    
    /// Check if a domain exists
    async fn domain_exists(&self, id: &DomainId) -> Result<bool, DomainError>;
    
    /// Update a domain
    async fn update_domain(&self, domain: Domain) -> Result<(), DomainError>;
    
    /// Deactivate a domain
    async fn deactivate_domain(&self, id: &DomainId) -> Result<(), DomainError>;
}
```

The registry maintains information about each domain, including its configuration, state, and capabilities.

### Domain Adapter

Domain Adapters translate Causality operations into domain-specific operations:

```rust
/// Adapter for domain-specific operations
#[async_trait]
pub trait DomainAdapter: Send + Sync + 'static {
    /// Get the domain ID
    fn domain_id(&self) -> &DomainId;
    
    /// Get the domain type
    fn domain_type(&self) -> DomainType;
    
    /// Initialize the adapter
    async fn initialize(&self, config: &DomainConfig) -> Result<(), DomainError>;
    
    /// Perform a domain-specific operation
    async fn execute_operation(
        &self, 
        operation: DomainOperation,
    ) -> Result<DomainOperationResult, DomainError>;
    
    /// Observe domain state
    async fn observe_state(
        &self,
        query: StateQuery,
    ) -> Result<StateObservation, DomainError>;
    
    /// Subscribe to domain events
    async fn subscribe(
        &self,
        filter: EventFilter,
    ) -> Result<EventSubscription, DomainError>;
    
    /// Check domain status
    async fn check_status(&self) -> Result<DomainStatus, DomainError>;
}
```

Different blockchains and data sources have their own adapters that implement this trait, allowing Causality to interact with them through a consistent interface.

### Domain Manager

The Domain Manager coordinates interactions with domains:

```rust
/// Manager for domain interactions
#[async_trait]
pub trait DomainManager: Send + Sync + 'static {
    /// Get an adapter for a domain
    async fn get_adapter(
        &self,
        domain_id: &DomainId,
    ) -> Result<Arc<dyn DomainAdapter>, DomainError>;
    
    /// Execute a cross-domain operation
    async fn execute_cross_domain(
        &self,
        operations: Vec<DomainOperation>,
        context: &OperationContext,
    ) -> Result<Vec<DomainOperationResult>, DomainError>;
    
    /// Register a new domain
    async fn register_domain(
        &self,
        domain: Domain,
    ) -> Result<DomainId, DomainError>;
    
    /// Subscribe to events from multiple domains
    async fn subscribe_multi(
        &self,
        subscriptions: Vec<(DomainId, EventFilter)>,
    ) -> Result<MultiDomainSubscription, DomainError>;
}
```

The Domain Manager serves as the entry point for domain interactions, providing methods for accessing adapters and coordinating cross-domain operations.

### Domain Operations

Operations that can be performed on domains:

```rust
/// An operation to be performed on a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainOperation {
    /// Domain ID
    domain_id: DomainId,
    /// Operation type
    operation_type: OperationType,
    /// Operation parameters
    parameters: HashMap<String, Value>,
    /// Dependencies
    dependencies: Vec<OperationId>,
    /// Content hash
    content_hash: ContentHash,
}

/// Types of domain operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationType {
    /// Read operation
    Read,
    /// Write operation
    Write,
    /// Execute operation
    Execute,
    /// Observe operation
    Observe,
    /// Custom operation
    Custom(String),
}
```

Domain operations are content-addressed and can have dependencies, enabling complex, multi-step operations that maintain causal ordering.

## Domain Boundary Crossing

### Boundary Crossing Protocol

The boundary crossing protocol enables secure transfer of data and effects across domain boundaries:

```rust
/// Protocol for crossing domain boundaries
#[async_trait]
pub trait BoundaryCrossing: Send + Sync + 'static {
    /// Cross from one domain to another
    async fn cross_boundary(
        &self,
        from: &DomainId,
        to: &DomainId,
        payload: &Payload,
        context: &CrossingContext,
    ) -> Result<CrossingReceipt, BoundaryError>;
    
    /// Verify a crossing receipt
    async fn verify_crossing(
        &self,
        receipt: &CrossingReceipt,
    ) -> Result<bool, BoundaryError>;
    
    /// Get the status of a crossing
    async fn crossing_status(
        &self,
        crossing_id: &CrossingId,
    ) -> Result<CrossingStatus, BoundaryError>;
}

/// Context for a boundary crossing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossingContext {
    /// Crossing ID
    id: CrossingId,
    /// Sender identity
    sender: Identity,
    /// Receiver identity
    receiver: Identity,
    /// Timestamp
    timestamp: DateTime<Utc>,
    /// Capabilities
    capabilities: Vec<Capability>,
}
```

Boundary crossings involve cryptographic verification to ensure that data and effects are transferred securely between domains.

### Cross-Domain Effects

Effects can be composed across domains:

```rust
/// An effect that spans multiple domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainEffect {
    /// Source domain
    source_domain: DomainId,
    /// Target domain
    target_domain: DomainId,
    /// Effect to execute
    effect: Box<dyn Effect>,
    /// Boundary crossing parameters
    crossing_params: CrossingParams,
    /// Content hash
    content_hash: ContentHash,
}

impl Effect for CrossDomainEffect {
    // Implementation of Effect trait methods
}
```

Cross-domain effects enable complex operations that span multiple domains while maintaining the semantics of the Effect System.

## Domain State Observation

### State Observation

Causality observes the state of external domains through a consistent interface:

```rust
/// Query for domain state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateQuery {
    /// Path to query
    path: String,
    /// Query parameters
    parameters: HashMap<String, Value>,
    /// Proof requirements
    proof_required: bool,
}

/// Result of state observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateObservation {
    /// Observed data
    data: Value,
    /// Proof of the observation (if requested)
    proof: Option<StateProof>,
    /// Timestamp of the observation
    timestamp: DateTime<Utc>,
    /// Content hash
    content_hash: ContentHash,
}
```

State observations can include cryptographic proofs to verify the authenticity and integrity of the observed data.

### Event Subscription

Causality can subscribe to events from external domains:

```rust
/// Filter for domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    /// Event types
    event_types: Vec<String>,
    /// Filter conditions
    conditions: HashMap<String, Value>,
}

/// Subscription to domain events
#[async_trait]
pub trait EventSubscription: Send + Sync + 'static {
    /// Get the next event
    async fn next_event(&mut self) -> Result<Option<DomainEvent>, DomainError>;
    
    /// Acknowledge an event
    async fn acknowledge(&mut self, event_id: &EventId) -> Result<(), DomainError>;
    
    /// Close the subscription
    async fn close(&mut self) -> Result<(), DomainError>;
}
```

Event subscriptions enable Causality to react to changes in external domains, maintaining causal consistency across the system.

## Domain Adapters

### Blockchain Adapters

Adapters for various blockchain platforms:

```rust
/// Ethereum domain adapter
pub struct EthereumAdapter {
    /// Domain ID
    domain_id: DomainId,
    /// Ethereum client
    client: Arc<dyn EthereumClient>,
    /// Configuration
    config: EthereumConfig,
}

impl DomainAdapter for EthereumAdapter {
    // Implementation of DomainAdapter trait methods
}

/// Solana domain adapter
pub struct SolanaAdapter {
    /// Domain ID
    domain_id: DomainId,
    /// Solana client
    client: Arc<dyn SolanaClient>,
    /// Configuration
    config: SolanaConfig,
}

impl DomainAdapter for SolanaAdapter {
    // Implementation of DomainAdapter trait methods
}
```

These adapters translate Causality operations into blockchain-specific operations, such as smart contract calls or transaction submissions.

### Database Adapters

Adapters for various database systems:

```rust
/// SQL database adapter
pub struct SqlAdapter {
    /// Domain ID
    domain_id: DomainId,
    /// Database connection
    connection: Arc<dyn SqlConnection>,
    /// Configuration
    config: SqlConfig,
}

impl DomainAdapter for SqlAdapter {
    // Implementation of DomainAdapter trait methods
}

/// NoSQL database adapter
pub struct NoSqlAdapter {
    /// Domain ID
    domain_id: DomainId,
    /// Database client
    client: Arc<dyn NoSqlClient>,
    /// Configuration
    config: NoSqlConfig,
}

impl DomainAdapter for NoSqlAdapter {
    // Implementation of DomainAdapter trait methods
}
```

Database adapters enable Causality to interact with various database systems through a uniform interface.

## Integration with Other Systems

### Resource System Integration

Resources can be associated with specific domains:

```rust
/// Resource with domain information
pub struct DomainResource {
    /// Resource ID
    id: ResourceId,
    /// Domain ID
    domain_id: DomainId,
    /// Resource data
    data: Value,
    /// Content hash
    content_hash: ContentHash,
}

impl ContentAddressed for DomainResource {
    // Implementation of ContentAddressed trait methods
}
```

This integration allows resources to be identified by their domain and enables domain-specific resource access patterns.

### Effect System Integration

Effects can be executed in specific domains:

```rust
/// Domain-specific effect
pub struct DomainEffect {
    /// Domain ID
    domain_id: DomainId,
    /// Effect to execute
    effect: Box<dyn Effect>,
    /// Domain-specific parameters
    parameters: HashMap<String, Value>,
    /// Content hash
    content_hash: ContentHash,
}

impl Effect for DomainEffect {
    // Implementation of Effect trait methods
}
```

This integration allows effects to be executed in specific domains, with domain-specific parameters and validation rules.

### Time System Integration

Time observations can be made from specific domains:

```rust
/// Domain-specific time observation
pub struct DomainTimeObservation {
    /// Domain ID
    domain_id: DomainId,
    /// Observed time
    timestamp: DateTime<Utc>,
    /// Proof of observation
    proof: Option<TimeProof>,
    /// Content hash
    content_hash: ContentHash,
}

impl ContentAddressed for DomainTimeObservation {
    // Implementation of ContentAddressed trait methods
}
```

This integration enables time observations to be associated with specific domains, with domain-specific trust models and verification rules.

## Affected Components and Location

The Domain System touches several parts of Causality:

| Component | Purpose | Location |
|-----------|---------|----------|
| Domain Registry | Central registry of domains | `causality_core::domain::DomainRegistry` |
| Domain Adapter | Translation of operations | `causality_domain::adapter::DomainAdapter` |
| Domain Manager | Coordination of domain interactions | `causality_core::domain::DomainManager` |
| Boundary Crossing | Cross-domain data transfer | `causality_domain::boundary::BoundaryCrossing` |
| State Observation | Observation of domain state | `causality_domain::state::StateObservation` |
| Event Subscription | Subscription to domain events | `causality_domain::events::EventSubscription` |
| Blockchain Adapters | Adapters for blockchain domains | `causality_domain::blockchain::*` |
| Database Adapters | Adapters for database domains | `causality_domain::database::*` |

## References

- [ADR-018: Domain Adapter Pattern](../../adrs/adr_018_domain_adapter_pattern.md)
- [ADR-023: Three-Layer Effect Architecture with TEL Integration](../../adrs/adr_023_domain_adapter_effect_handler_unification.md)
- [ADR-031: Domain-Specific Operations](../../adrs/adr_031_domain_specific_operations.md)
- [System Specification: Domain System](../../../../spec/spec.md#domain-system) 