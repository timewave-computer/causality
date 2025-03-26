# ADR 032: Role-Based Resource System

## Status

| Status    | Date       | Revision |
|:----------|:-----------|:---------|
| Accepted  | 2023-08-15 | 1.2      |

## Context

ADR-005 introduced the concept of "actors" within the Causality system, defining them as one of three roles: user, operator, or committee. This has led to some confusion as the term "actor" is often associated with the "Actor Model" pattern in distributed systems. This model involves independent computational units that communicate exclusively through message passing.

Upon review, we have determined that these roles are fundamentally resource types with specific capabilities rather than independent computational units in the Actor Model sense. The Resource System introduced in ADR-002 and enhanced in ADR-030 already provides mechanisms for content-addressed resource identification, capability-based access control, and state management.

Additionally, we need to clarify the distinction between a "committee" and a "domain":

1. A **committee** is a specific set of validators that materialize chain state
2. A **domain** has an address that serves as an abstraction point for interaction:
   - Publishing to a domain address is equivalent to posting a transaction to the chain
   - Subscribing to a domain address is equivalent to using the chain observation system

The existing actor system:
1. Uses a different addressing mechanism than our core content-addressed resources
2. Introduces its own lifecycle and state management patterns
3. Duplicates concurrency control mechanisms already present in the resource system
4. Creates confusion about the boundary between actors and resources

## Decision

We will integrate role-based entities directly into the Resource System rather than creating a separate Actor System. Specifically:

1. Define `User` and `Operator` as specialized resource types within the Resource System
2. Represent a `Domain` as a resource with an address, rather than treating committees as message recipients
3. Use content-addressed resource identifiers for addressing these entities
4. Leverage the existing capability system for role-based permissions
5. Extend the Resource System to handle necessary message routing between entities
6. Ensure all entities remain content-addressed with no UUIDs

This approach unifies our architecture around the Resource System as the primary mechanism for identity and access control.

### Domain Address Model

Instead of messaging a committee directly, the system will:

1. Represent each domain as a first-class resource with a content-addressed identifier
2. Implement domain publication as transaction submission to the underlying chain
3. Implement domain subscription as chain observation through the time system
4. Map domain resources to the specific validator committees that maintain them, but treat this mapping as an implementation detail

This model better reflects the reality of blockchain-based systems where transactions are notionally submitted to chains rather than a set of validators.

### Resource-Based Actor Architecture

Building on this foundation, we will implement a comprehensive resource-based actor architecture that:

1. Implements all actor functionality as specialized resource types
2. Leverages content addressing for all entity identification
3. Uses the unified capability system for permissions
4. Defines clear resource state transition rules
5. Establishes deterministic resource references and relationship patterns

#### Core Resource Actor Model

At the heart of the architecture is the `ResourceActor` trait that builds on the existing resource system:

```rust
/// A resource-based actor in the system
#[async_trait]
pub trait ResourceActor: ResourceAccessor + ContentAddressed {
    /// Get the actor's identity
    fn actor_id(&self) -> ContentId {
        self.content_id()
    }
    
    /// Get the actor's type
    fn actor_type(&self) -> &str;
    
    /// Process a message sent to this actor
    async fn handle_message(&mut self, message: Message, context: &Context) 
        -> Result<MessageResponse, ActorError>;
    
    /// Handle a state transition request
    async fn handle_state_transition(&mut self, transition: StateTransition, context: &Context)
        -> Result<StateTransitionResult, ActorError>;
    
    /// Get a reference to another actor
    async fn get_actor_reference(&self, actor_id: &ContentId) 
        -> Result<ActorReference, ActorError>;
}
```

The `ResourceActor` trait extends the existing `ResourceAccessor` pattern, ensuring that all actors are also resources, while adding actor-specific behaviors.

#### Resource State Transition Model

Actor lifecycles are mapped directly to resource states with well-defined transitions:

```rust
/// State transitions for resource actors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateTransition {
    /// Initialize the actor
    Initialize {
        /// Initial parameters
        params: HashMap<String, Value>,
    },
    
    /// Activate the actor
    Activate,
    
    /// Suspend the actor temporarily
    Suspend {
        /// Reason for suspension
        reason: String,
    },
    
    /// Resume a suspended actor
    Resume,
    
    /// Upgrade the actor implementation
    Upgrade {
        /// New implementation version
        version: String,
        /// Migration parameters
        migration_params: HashMap<String, Value>,
    },
    
    /// Terminate the actor
    Terminate {
        /// Reason for termination
        reason: String,
    },
}

/// Valid state transitions
///
/// This table defines the allowed transitions between states:
///
/// | Current State | Valid Transitions                |
/// |---------------|----------------------------------|
/// | Created       | Initialize                       |
/// | Initialized   | Activate                         |
/// | Active        | Suspend, Upgrade, Terminate      |
/// | Suspended     | Resume, Terminate                |
/// | Upgraded      | Activate                         |
/// | Terminated    | None                             |
```

#### Permission Model

Resource actors use the capability-based permission system with actor-specific capabilities:

```rust
/// Actor-specific capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorCapability {
    /// Send messages to the actor
    SendMessage {
        /// Types of messages that can be sent
        message_types: Vec<String>,
    },
    
    /// Call specific methods on the actor
    CallMethod {
        /// Methods that can be called
        methods: Vec<String>,
    },
    
    /// Manage the actor's lifecycle
    ManageLifecycle {
        /// Allowed state transitions
        allowed_transitions: Vec<StateTransition>,
    },
    
    /// Manage the actor's relationships
    ManageRelationships {
        /// Types of relationships that can be managed
        relationship_types: Vec<String>,
    },
    
    /// Access the actor's internal state
    AccessState {
        /// State properties that can be accessed
        properties: Vec<String>,
        /// Whether read-only or read-write access
        readonly: bool,
    },
    
    /// Full control of the actor
    FullControl,
}
```

#### Reference and Relationship Model

Resource actors use a deterministic reference system based on content addresses:

```rust
/// A reference to another actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorReference {
    /// Target actor ID (content address)
    pub target_id: ContentId,
    
    /// Type of the target actor
    pub target_type: String,
    
    /// Relationship type
    pub relationship: RelationshipType,
    
    /// Reference capabilities
    pub capabilities: Vec<ActorCapability>,
    
    /// Reference metadata
    pub metadata: HashMap<String, Value>,
}

/// Types of relationships between actors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Direct ownership
    Owns,
    
    /// Parent/child relationship
    Parent,
    
    /// Child/parent relationship
    Child,
    
    /// Peer relationship
    Peer,
    
    /// Delegate relationship (can act on behalf of)
    Delegate,
    
    /// Dependency relationship
    DependsOn,
    
    /// Custom relationship type
    Custom(String),
}
```

Actor references are themselves content-addressed, allowing for deterministic traversal of actor relationship graphs.

#### Communication Pattern

Resource actors communicate through a message passing pattern built on content addressing:

```rust
/// A message sent to an actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID (content address)
    pub id: ContentId,
    
    /// Sender of the message
    pub sender: ContentId,
    
    /// Receiver of the message
    pub receiver: ContentId,
    
    /// Message type
    pub message_type: String,
    
    /// Message payload
    pub payload: Value,
    
    /// Message metadata
    pub metadata: HashMap<String, Value>,
    
    /// Timestamp when the message was created
    pub timestamp: DateTime<Utc>,
    
    /// Message expiration
    pub expires_at: Option<DateTime<Utc>>,
}

impl ContentAddressed for Message {
    // Implementation details...
}
```

#### Specialized Actor Types

The architecture defines several specialized actor types that extend the base ResourceActor trait:

1. **User Actor**
   
```rust
/// User actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActor {
    /// Base resource fields
    pub resource: Resource,
    
    /// User-specific identities
    pub identities: Vec<UserIdentity>,
    
    /// User profile information
    pub profile: UserProfile,
    
    /// User preferences
    pub preferences: HashMap<String, Value>,
    
    /// User capabilities
    pub capabilities: Vec<UserCapability>,
}

#[async_trait]
impl ResourceActor for UserActor {
    // Implementation details...
}
```

2. **Domain Actor**

```rust
/// Domain actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainActor {
    /// Base resource fields
    pub resource: Resource,
    
    /// Domain address
    pub address: DomainAddress,
    
    /// Domain type
    pub domain_type: String,
    
    /// Domain-specific parameters
    pub parameters: HashMap<String, Value>,
    
    /// Associated validator committee
    pub committee: Option<CommitteeInfo>,
    
    /// Domain state root
    pub state_root: Option<ContentId>,
}

#[async_trait]
impl ResourceActor for DomainActor {
    // Implementation details...
}
```

3. **Service Actor**

```rust
/// Service actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceActor {
    /// Base resource fields
    pub resource: Resource,
    
    /// Service type
    pub service_type: String,
    
    /// Service endpoints
    pub endpoints: Vec<ServiceEndpoint>,
    
    /// Service configuration
    pub configuration: HashMap<String, Value>,
    
    /// Service metrics
    pub metrics: Option<ServiceMetrics>,
    
    /// Service dependencies
    pub dependencies: Vec<ActorReference>,
}

#[async_trait]
impl ResourceActor for ServiceActor {
    // Implementation details...
}
```

## Consequences

### Positive

- **Architectural Cohesion**: Unifies actor and resource concepts under a single consistent model
- **Content Addressing**: All entities use the same content addressing mechanism, eliminating UUID dependencies
- **Simplified Mental Model**: Developers only need to understand one system (resources) rather than two separate systems
- **Consistent Permissions**: Uses the unified capability system for all entity types
- **Clear State Machine**: Provides well-defined state transitions for all actor resources
- **Reuse of Patterns**: Leverages existing resource patterns for lifecycles, relationships, and access control
- **Architectural Simplification**: Reduces the number of core systems, providing a clearer and more cohesive architecture
- **Consistency**: Uses the same mechanisms for identification, access control, and state management across all entities
- **Reduced Cognitive Load**: Developers only need to understand one system (Resource) rather than two separate systems
- **Content Addressing Consistency**: All entities will use the same content addressing mechanism, simplifying verification
- **Domain Clarity**: Provides a more accurate model of how domains and validator committees actually work

### Negative

- **Migration Effort**: Significant effort required to migrate from the current actor system to the resource-based architecture
- **Learning Curve**: Teams familiar with the existing actor system will need to learn the new resource-based approach
- **Implementation Complexity**: More complex implementation in the short term as we transition between architectures
- **Lost Separation**: Some benefits of the Actor Model's strict isolation are lost, though these are less relevant given our actual use cases
- **Extension Required**: The Resource System will need to be extended to handle message routing capabilities

### Neutral

- **Performance Tradeoffs**: Some performance characteristics may change, with potential improvements in some areas and degradation in others
- **API Changes**: Client APIs will change, but in ways that are generally more consistent with the rest of the system
- **Resource Complexity**: The Resource System gains additional responsibility but remains focused on its core concern of managing identifiable entities

## Alternatives Considered

### 1. Implement a Traditional Actor System

We considered implementing a full Actor Model-based system as originally interpreted. This would provide stronger isolation guarantees but would introduce unnecessary complexity given our actual needs.

### 2. Hybrid Approach

A hybrid approach would maintain separate systems but create bridges between them. This was rejected as it would increase complexity without providing clear benefits.

### 3. Status Quo with Clarification

We could keep the current design but clarify terminology. However, this would perpetuate the architectural confusion and missed opportunity for simplification.

## Implementation Plan

1. **Define Core Types** (2 weeks)
   - Create the `ResourceActor` trait and core interfaces
   - Define state transition model and state machine
   - Implement permission model with actor-specific capabilities
   - Define actor reference and relationship pattern

2. **Implement Base Abstractions** (3 weeks)
   - Create abstract base implementations of resource actors
   - Implement actor state management through resource states
   - Create message handling infrastructure
   - Implement actor reference system

3. **Specialized Actor Types** (2 weeks)
   - Implement User actor type
   - Implement Domain actor type
   - Implement Service actor types
   - Create testing utilities for each actor type

4. **Adaptation & Migration** (3 weeks)
   - Create adapter layer for existing actor system
   - Migrate core system services to resource actors
   - Update API interfaces for resource actor pattern
   - Create documentation and examples

5. **Ecosystem Utilities** (2 weeks)
   - Create builder patterns for resource actors
   - Implement serialization and deserialization utilities
   - Create actor relationship graph utilities
   - Implement actor discovery mechanisms

6. **Testing & Optimization** (2 weeks)
   - Create comprehensive test suite for resource actors
   - Benchmark performance and optimize critical paths
   - Test migration paths and backward compatibility
   - Address any issues discovered during testing

## Implementation Guidelines

1. Define standard resource types for User, Operator, and Domain resources

```rust
/// User resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID (content-addressed)
    id: ResourceId,
    /// Public keys
    public_keys: Vec<PublicKey>,
    /// Metadata
    metadata: HashMap<String, Value>,
    /// Content hash
    content_hash: ContentHash,
}

/// Domain resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    /// Domain ID (content-addressed)
    id: ResourceId,
    /// Domain address (used for transactions and observations)
    address: DomainAddress,
    /// Domain type (e.g., "ethereum", "solana")
    domain_type: String,
    /// Associated validator committee
    committee: Option<CommitteeInfo>,
    /// Domain metadata
    metadata: HashMap<String, Value>,
    /// Content hash
    content_hash: ContentHash,
}

/// Committee information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeInfo {
    /// Committee members (validators)
    members: Vec<ValidatorInfo>,
    /// Required signatures
    threshold: u32,
    /// Committee epoch
    epoch: u64,
}

/// Validator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// Validator ID
    id: ResourceId,
    /// Public key
    public_key: PublicKey,
    /// Voting power
    voting_power: u64,
}
```

2. Extend ResourceAccessor for domain-specific operations

```rust
/// Domain resource accessor
#[async_trait]
pub trait DomainAccessor: ResourceAccessor<Resource = Domain> {
    /// Submit a transaction to the domain
    async fn submit_transaction(&self, tx: Transaction) 
        -> Result<TransactionReceipt, DomainError>;
    
    /// Subscribe to domain events
    async fn subscribe(&self, filter: EventFilter) 
        -> Result<EventSubscription, DomainError>;
    
    /// Get the current state of the domain
    async fn get_state(&self) -> Result<DomainState, DomainError>;
}
```

3. Define messaging patterns for domain interaction

```rust
/// Transaction to a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Source resource
    from: ResourceId,
    /// Target domain
    to: DomainAddress,
    /// Transaction data
    data: Vec<u8>,
    /// Transaction type
    tx_type: String,
    /// Timestamp
    timestamp: DateTime<Utc>,
    /// Content hash
    content_hash: ContentHash,
}

impl ContentAddressed for Transaction {
    // Implementation...
}
```

## References

- ADR-002: Resource System
- ADR-003: Capability System
- ADR-005: Actor Specification
- ADR-030: Resource Accessor Pattern 

### User Privacy Support

The Role-Based Resource System includes comprehensive privacy support for user data and operations:

#### Privacy-Preserving Resource Model

Resources can be created with privacy-preserving attributes:

```rust
/// A resource with privacy-preserving attributes
pub struct PrivategResource {
    /// Public attributes (visible on-chain)
    pub public_attributes: HashMap<String, Value>,
    
    /// Private attributes (stored encrypted or as commitments)
    pub private_attributes: HashMap<String, PrivateValue>,
    
    /// Verification keys for the resource
    pub verification_keys: HashMap<String, VerificationKey>,
}

/// A private value
pub enum PrivateValue {
    /// Merkle root of a set of values
    MerkleRoot(String),
    
    /// Pedersen commitment to a value
    Commitment(String),
    
    /// Encrypted value
    Encrypted {
        /// Ciphertext
        ciphertext: String,
        
        /// Encryption scheme
        scheme: EncryptionScheme,
    },
}
```

#### Zero-Knowledge Operations

Users can perform operations while keeping sensitive data private:

```rust
/// Create a privacy-preserving operation
pub fn create_privacy_preserving_operation(
    operation_type: OperationType,
    statement: ProofStatement,
    proof: Proof,
    public_inputs: Vec<Value>,
) -> Result<Operation> {
    // Create the operation
    let mut operation = Operation::new(operation_type);
    
    // Add ZK proof as evidence
    operation.add_evidence(Evidence::ZeroKnowledgeProof {
        statement_id: statement.id(),
        proof_id: proof.id(),
        proof_type: proof.proof_type().clone(),
    });
    
    // Add public inputs as metadata
    for (i, input) in public_inputs.iter().enumerate() {
        operation.add_metadata(
            format!("public_input_{}", i),
            input.clone(),
        );
    }
    
    // Mark as privacy-preserving
    operation.add_metadata(
        "privacy_preserving",
        serde_json::json!(true),
    );
    
    Ok(operation)
}
```

#### Field-Level Encryption

Sensitive user data can be encrypted at the field level:

```rust
/// User profile with encrypted fields
pub struct UserProfile {
    /// Public fields
    pub display_name: String,
    pub profile_picture: Option<String>,
    
    /// Encrypted fields
    pub email: Option<EncryptedField>,
    pub phone: Option<EncryptedField>,
    pub address: Option<EncryptedField>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Field encryption policy
pub struct FieldEncryptionPolicy {
    /// Encryption algorithm
    pub algorithm: EncryptionAlgorithm,
    
    /// Whether encryption is required
    pub required: bool,
}
```

#### Privacy-Preserving Authentication

The system supports multiple privacy-preserving authentication methods:

```rust
/// Authentication methods
pub enum AuthMethod {
    /// Public key authentication
    PublicKey,
    /// Username and password
    Password,
    /// Multi-factor authentication
    MFA,
    /// OAuth provider
    OAuth(String),
    /// Zero-knowledge authentication
    ZeroKnowledge(ZkAuth),
}
```

#### Cross-Domain Privacy

Privacy is maintained across domain boundaries through:

1. **Capability-Based Authorization**: Programs receive capabilities that grant specific rights without revealing underlying resources
2. **Private Inputs to Shared Circuits**: Programs compose by agreeing on circuit interfaces while keeping inputs private
3. **Revealed Outputs with Hidden Internals**: Programs can reveal operation outputs while keeping inputs and intermediate steps private
4. **Multi-Party Computation**: Advanced use cases can use MPC techniques to compute over encrypted data

This privacy support ensures that users can interact with the system while maintaining control over their sensitive data and operations. 