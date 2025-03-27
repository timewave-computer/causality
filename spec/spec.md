# Causality System Specification

**Current Revision: 2023-11-15**

## Introduction

Causality is a distributed, causal computation system designed for secure, robust, and consistent state management across heterogeneous domains. This specification document serves as the comprehensive reference for the architecture, design principles, and implementation patterns of the Causality system.

The Causality system enables reliable computation in environments with partial trust by maintaining causal consistency across domains, ensuring that effects and their outcomes can be verified, and enforcing capability-based security for all operations.

### Purpose and Scope

This specification:

1. Defines the core architectural components of the Causality system
2. Documents the data structures, interfaces, and protocols used throughout the system
3. Provides implementation guidance with concrete code examples
4. Establishes the formal requirements and invariants that must be maintained
5. References authoritative Architectural Decision Records (ADRs) for the rationale behind design choices

### System Architecture Overview

Causality is organized into interconnected components that form a cohesive system:

1. **Resources**: Represent domain entities and stateful objects
2. **Capabilities**: Grant precise, contextual authority to interact with resources
3. **Effects**: Abstract, composable actions that can change system state
4. **Operations**: Requests to perform effects with authorization
5. **Agents**: Entities that hold capabilities and perform operations
6. **Service Status**: Signals that agents are actively offering services
7. **Obligation Manager**: Enforces usage-based expectations on capabilities
8. **Messaging**: Enables asynchronous interaction between agents
9. **Fact System**: Tracks temporal and logical dependencies between actions

Each component provides clear abstractions and interfaces that allow them to work together while maintaining separation of concerns.

### Design Principles

All components within Causality adhere to these fundamental principles:

1. **Content Addressing**: All data structures have cryptographically secure content hashes that uniquely identify them based on their content rather than location or arbitrary identifiers.
2. **Capability-Based Security**: Access to resources is governed by unforgeable capabilities with explicit delegation paths.
3. **Temporal Consistency**: Operations maintain causal ordering through logical clocks and temporal facts.
4. **Compositional Effects**: Complex operations are composed of simpler, well-defined effects with clear semantics.
5. **Domain Independence**: The core system is agnostic to specific blockchain protocols or data stores while providing adapters for each.
6. **Resource Centricity**: All stateful objects are modeled as resources with well-defined lifecycles.
7. **Explicit Authorization**: Operations require explicit capabilities to perform effects.
8. **Separation of Concerns**: System components have clear responsibilities and interactions.

## Table of Contents

- [Causality System Specification](#causality-system-specification)
  - [Introduction](#introduction)
    - [Purpose and Scope](#purpose-and-scope)
    - [System Architecture Overview](#system-architecture-overview)
    - [Design Principles](#design-principles)
  - [Table of Contents](#table-of-contents)
  - [Core Principles](#core-principles)
  - [Architecture Overview](#architecture-overview)
  - [1. Content Addressing System \[ADR-007, ADR-028, ADR-029, ADR-030\]](#1-content-addressing-system-adr-007-adr-028-adr-029-adr-030)
    - [1.1 Core Components](#11-core-components)
    - [1.2 Content Hash Calculation](#12-content-hash-calculation)
    - [1.3 Deferred Hashing \[ADR-030\]](#13-deferred-hashing-adr-030)
    - [1.4 SMT Integration \[ADR-029\]](#14-smt-integration-adr-029)
  - [2. Time System \[ADR-000, ADR-024\]](#2-time-system-adr-000-adr-024)
    - [2.1 Temporal Model Foundations](#21-temporal-model-foundations)
    - [2.2 Distinct Notions of Time](#22-distinct-notions-of-time)
    - [2.3 Lamport Clock](#23-lamport-clock)
      - [Usage Example: Enforcing Causal Order](#usage-example-enforcing-causal-order)
    - [2.4 Time Map and Temporal Facts \[ADR-024\]](#24-time-map-and-temporal-facts-adr-024)
    - [2.5 Time as an Effect](#25-time-as-an-effect)
    - [2.6 Time Services](#26-time-services)
    - [2.7 Temporal Fact Validation](#27-temporal-fact-validation)
    - [2.8 Integration with Effect System](#28-integration-with-effect-system)
  - [3. Effect System \[ADR-001, ADR-023, ADR-031, ADR-032\]](#3-effect-system-adr-001-adr-023-adr-031-adr-032)
    - [3.1 Algebraic Effects \[ADR-023\]](#31-algebraic-effects-adr-023)
      - [Usage Example: Creating a Custom Effect](#usage-example-creating-a-custom-effect)
    - [3.2 Effect Constraints \[ADR-023\]](#32-effect-constraints-adr-023)
    - [3.3 Domain Adapter as Effect \[ADR-031\]](#33-domain-adapter-as-effect-adr-031)
    - [3.4 Effect Execution Lifecycle](#34-effect-execution-lifecycle)
    - [3.5 Cross-Domain Effect Composition](#35-cross-domain-effect-composition)
    - [3.6 Effect Interpreter \[ADR-032\]](#36-effect-interpreter-adr-032)
    - [3.7 Operation and Effect Integration \[ADR-032\]](#37-operation-and-effect-integration-adr-032)
  - [4. Resource System \[ADR-002, ADR-030, ADR-032\]](#4-resource-system-adr-002-adr-030-adr-032)
    - [4.1 Resource Model \[ADR-002, ADR-032\]](#41-resource-model-adr-002-adr-032)
    - [4.2 Resource Operations \[ADR-032\]](#42-resource-operations-adr-032)
    - [4.3 Resource Logic \[ADR-032\]](#43-resource-logic-adr-032)
    - [4.4 Resource Lifecycle \[ADR-032\]](#44-resource-lifecycle-adr-032)
    - [4.5 Resource System Interaction Diagram](#45-resource-system-interaction-diagram)
    - [4.6 Resource Query Language](#46-resource-query-language)
  - [5. Capability System \[ADR-003, ADR-032, ADR-032\]](#5-capability-system-adr-003-adr-032-adr-032)
    - [5.1 Capability Model \[ADR-003\]](#51-capability-model-adr-003)
    - [5.2 Capability Delegation \[ADR-032\]](#52-capability-delegation-adr-032)
      - [Usage Example: Creating and Delegating Capabilities](#usage-example-creating-and-delegating-capabilities)
    - [5.3 Capability Store](#53-capability-store)
    - [5.4 Capability Integration with Effects](#54-capability-integration-with-effects)
    - [5.5 Capability Registry \[ADR-032\]](#55-capability-registry-adr-032)
    - [5.6 Capability Constraints](#56-capability-constraints)
    - [5.7 Capability System Interaction Diagram](#57-capability-system-interaction-diagram)
    - [5.8 Capability-based Security Model](#58-capability-based-security-model)
  - [6. Agent System \[ADR-005, ADR-032, ADR-032\]](#6-agent-system-adr-005-adr-032-adr-032)
    - [6.1 Agent Definition \[ADR-032\]](#61-agent-definition-adr-032)
    - [6.2 Agent Profiles \[ADR-032\]](#62-agent-profiles-adr-032)
    - [6.3 Service Status \[ADR-032\]](#63-service-status-adr-032)
    - [6.4 Obligation Manager \[ADR-032\]](#64-obligation-manager-adr-032)
    - [6.5 Messaging \[ADR-032\]](#65-messaging-adr-032)
    - [6.6 Agent System Diagram](#66-agent-system-diagram)
  - [7. Operation System \[ADR-032\]](#7-operation-system-adr-032)
    - [7.1 Operation Model](#71-operation-model)
    - [7.2 Authorization](#72-authorization)
    - [7.3 Operation Execution](#73-operation-execution)
    - [7.4 Operation Composition](#74-operation-composition)
    - [7.5 Operation Interaction Diagram](#75-operation-interaction-diagram)
  - [Conclusion](#conclusion)
  - [Codebase Structure](#codebase-structure)
    - [Core Crates](#core-crates)
    - [Utility Crates](#utility-crates)
    - [Frontend Crates](#frontend-crates)
    - [Integration Crates](#integration-crates)
    - [Dependencies and Build System](#dependencies-and-build-system)
    - [Development Guidelines](#development-guidelines)

## Core Principles

1. **Universal Content Addressing**: All stateful objects are content-addressed with cryptographic hashes [ADR-007, ADR-028]
2. **Causal Consistency**: Every effect has a complete causal history [ADR-000, ADR-009]
3. **Explicit Fact Observations**: Programs cannot act on external state without explicit observation [ADR-008, ADR-024]
4. **User Sovereignty**: Programs cannot be forcibly upgraded [ADR-019]
5. **Deterministic Replay**: All program state is fully reconstructible from logs [ADR-009, ADR-017]
6. **Domain Agnosticism**: Programs do not require domain-specific logic [ADR-018, ADR-031]
7. **Capability-Based Security**: Operations require explicit capabilities [ADR-022, ADR-032]
8. **Algebraic Effect Model**: Effects have explicit inputs, outputs, and constraints [ADR-001, ADR-023]
9. **Resource Centricity**: All stateful objects are modeled as resources [ADR-032]
10. **Explicit Authorization**: Operations require explicit validation of capabilities [ADR-032]

## Architecture Overview

Causality is built around these core components:

```
┌─────────────────────────────────────────────────────────────────┐
│                      Operation System                           │
│   ┌─────────────┐   ┌─────────────┐   ┌─────────────────────┐   │
│   │ Operations  │   │Authorization│   │  Execution Context  │   │
│   └─────────────┘   └─────────────┘   └─────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                         Effect System                           │
│   ┌─────────────┐   ┌─────────────┐   ┌─────────────────────┐   │
│   │  Effects    │   │ Interpreter │   │  Domain Adapters    │   │
│   └─────────────┘   └─────────────┘   └─────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                      Resource System                            │
│   ┌─────────────┐   ┌─────────────┐   ┌─────────────────────┐   │
│   │ Resources   │   │ResourceLogic│   │  Resource Effects   │   │
│   └─────────────┘   └─────────────┘   └─────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                       Time System                               │
│   ┌─────────────┐   ┌─────────────┐   ┌─────────────────────┐   │
│   │ LamportClock│   │ Time Maps   │   │  Temporal Facts     │   │
│   └─────────────┘   └─────────────┘   └─────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                  Content Addressing System                      │
│   ┌─────────────┐   ┌─────────────┐   ┌─────────────────────┐   │
│   │ ContentHash │   │ Storage     │   │  Verification       │   │
│   └─────────────┘   └─────────────┘   └─────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                     Capability System                           │
│   ┌─────────────┐   ┌─────────────┐   ┌─────────────────────┐   │
│   │ Capabilities│   │CapabilityRegistry│ Verification       │   │
│   └─────────────┘   └─────────────┘   └─────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                       Agent System                              │
│   ┌─────────────┐   ┌─────────────┐   ┌─────────────────────┐   │
│   │ Agents      │   │ Messaging   │   │  Service Status     │   │
│   └─────────────┘   └─────────────┘   └─────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## 1. Content Addressing System [ADR-007, ADR-028, ADR-029, ADR-030]

The content addressing system provides cryptographic guarantees for all stateful objects through content-derived identifiers.

### 1.1 Core Components

```rust
/// A cryptographic hash that uniquely identifies content
pub struct ContentHash {
    /// Raw hash bytes
    bytes: [u8; 32],
}

/// Trait for objects that are content-addressed
pub trait ContentAddressed: Sized {
    /// Get the content hash of this object
    fn content_hash(&self) -> Result<ContentHash, ContentAddressingError>;
    
    /// Verify that this object matches its expected hash
    fn verify(&self, expected_hash: &ContentHash) -> Result<bool, ContentAddressingError>;
    
    /// Convert to a serialized form
    fn to_bytes(&self) -> Result<Vec<u8>, ContentAddressingError>;
    
    /// Create from serialized form
    fn from_bytes(bytes: &[u8]) -> Result<Self, ContentAddressingError>;
}

/// Storage for content-addressed objects
pub trait ContentAddressedStorage: Send + Sync {
    /// Store an object by its content hash
    fn store<T: ContentAddressed>(&self, object: &T) -> Result<ContentHash, ContentAddressingError>;
    
    /// Retrieve an object by its content hash
    fn get<T: ContentAddressed>(&self, hash: &ContentHash) -> Result<T, ContentAddressingError>;
    
    /// Check if an object exists
    fn exists(&self, hash: &ContentHash) -> Result<bool, ContentAddressingError>;
}
```

### 1.2 Content Hash Calculation

Content hashes are calculated via:
1. **Serialize**: Object is serialized using canonical serialization
2. **Normalize**: Serialized data is normalized to ensure consistent representation
3. **Hash**: A cryptographic hash function (Blake3/Poseidon) is applied to the normalized data

### 1.3 Deferred Hashing [ADR-030]

For performance optimization, Causality implements deferred hashing outside ZK execution environments:

```rust
/// Context for deferred hash operations
pub struct DeferredHashingContext {
    /// Queue of pending hash operations
    pending_operations: Vec<HashOperation>,
    /// Computed results
    results: HashMap<OperationId, ContentHash>,
}

/// Deferred hash operation
pub enum HashOperation {
    /// Directly compute a hash
    Direct { data: Vec<u8> },
    /// Combine multiple hashes
    Combine { sources: Vec<OperationId> },
}
```

### 1.4 SMT Integration [ADR-029]

Content addressing integrates with Sparse Merkle Trees for efficient storage and verification:

```rust
/// Content-addressed SMT
pub struct ContentAddressedSmt<T: ContentAddressed> {
    /// Root hash of the tree
    root: ContentHash,
    /// Storage backend
    storage: Arc<dyn ContentAddressedStorage>,
}

impl<T: ContentAddressed> ContentAddressedSmt<T> {
    /// Insert a value into the tree
    fn insert(&mut self, key: &[u8], value: &T) -> Result<(), SmtError>;
    
    /// Get a value from the tree
    fn get(&self, key: &[u8]) -> Result<Option<T>, SmtError>;
    
    /// Generate a proof of inclusion
    fn prove(&self, key: &[u8]) -> Result<InclusionProof, SmtError>;
    
    /// Verify a proof
    fn verify_proof(&self, proof: &InclusionProof, key: &[u8], value: &T) -> Result<bool, SmtError>;
}
```

## 2. Time System [ADR-000, ADR-024]

The time system provides a unified framework for tracking time across domains, enabling causal consistency and temporal validation in a distributed environment.

### 2.1 Temporal Model Foundations

Causality resolves the challenge of coordinating time across independent domains through a multi-layered approach:

1. **Domain-local time**: Each domain maintains its own clock, block height, and finality guarantees
2. **Logical time**: A Lamport clock ensures causal ordering of internal events
3. **Time maps**: Capture relative positions across all domains at observation time
4. **Temporal facts**: Observations tied to specific points in domain timelines

This approach enables programs to reason about time without requiring global synchronization, while still maintaining strong causal consistency guarantees.

### 2.2 Distinct Notions of Time

Based on implementation experience, we have formalized two distinct notions of time in the system:

1. **Causal Time**: A materialization of operations that are partially ordered with respect to others in the Causality system. Causal time represents the logical ordering of events and captures the "happens-before" relationship between operations.

2. **Clock Time**: Attestations by outside parties about when events occurred. These could come from users, operators, blockchain timestamps, or other external sources. Clock time involves different trust models depending on the source.

This distinction enables programs to reason explicitly about different forms of temporal relationships, with different trust and verification requirements.

```rust
/// Sources of time attestations with varying trust levels
pub enum AttestationSource {
    /// Blockchain timestamp (trust depends on consensus model)
    Blockchain {
        height: BlockHeight,
        block_hash: BlockHash,
    },
    
    /// User attestation (low trust)
    User {
        user_id: UserId,
        signature: Signature,
    },
    
    /// Operator attestation (medium trust)
    Operator {
        operator_id: OperatorId,
        signature: Signature,
    },
    
    /// Committee attestation (higher trust)
    Committee {
        committee_id: CommitteeId,
        threshold_signature: ThresholdSignature,
    },
    
    /// External oracle (trust depends on oracle reputation)
    Oracle {
        oracle_id: OracleId,
        signature: Signature,
    },
}
```

### 2.3 Lamport Clock

```rust
/// Logical clock for ordering events
pub struct LamportClock {
    /// Current clock value
    value: u64,
}

impl LamportClock {
    /// Create a new Lamport clock
    pub fn new() -> Self {
        Self { value: 0 }
    }
    
    /// Increment and get the clock value
    pub fn tick(&mut self) -> u64 {
        self.value += 1;
        self.value
    }
    
    /// Update the clock based on a received timestamp
    pub fn update(&mut self, received: u64) {
        self.value = self.value.max(received) + 1;
    }
    
    /// Generate a timestamp for an outgoing message
    pub fn timestamp(&mut self) -> u64 {
        self.tick()
    }
    
    /// Process an incoming message with timestamp
    pub fn process_message(&mut self, message_time: u64) -> u64 {
        self.update(message_time);
        self.value
    }
}
```

#### Usage Example: Enforcing Causal Order

```rust
fn ensure_causal_order(mut clock: LamportClock, operations: &[Operation]) -> Result<Vec<Operation>, Error> {
    let mut ordered_ops = Vec::new();
    
    for op in operations {
        if let Some(timestamp) = op.timestamp {
            // Update our clock based on the operation's timestamp
            clock.update(timestamp);
        }
        
        // Assign a new timestamp to the operation
        let new_timestamp = clock.tick();
        let mut ordered_op = op.clone();
        ordered_op.timestamp = Some(new_timestamp);
        
        ordered_ops.push(ordered_op);
    }
    
    Ok(ordered_ops)
}
```

### 2.4 Time Map and Temporal Facts [ADR-024]

Time maps provide a snapshot of domain positions across all observed domains:

```rust
/// A snapshot of domain positions
pub struct TimeMap {
    /// Map of domain IDs to their positions
    pub positions: HashMap<DomainId, TimePosition>,
    
    /// When this time map was observed
    pub observed_at: Timestamp,
    
    /// Content hash of this time map
    pub content_hash: ContentHash,
}

/// Position in a domain's timeline
pub struct TimePosition {
    /// Block height
    pub height: BlockHeight,
    
    /// Block hash
    pub hash: BlockHash,
    
    /// Block timestamp
    pub timestamp: Timestamp,
}
```

Temporal facts (ADR-024) extend this model by including the complete temporal context with each fact:

```rust
/// A fact with its complete temporal context
pub struct TemporalFact {
    /// Unique identifier
    pub id: FactId,
    
    /// Domain this fact comes from
    pub domain_id: DomainId,
    
    /// Type of this fact
    pub fact_type: FactType,
    
    /// The actual fact data
    pub value: Value,
    
    /// Time position when observed
    pub time_position: TimePosition,
    
    /// Enhanced time model
    pub causal_position: CausalPosition,
    pub clock_attestation: ClockAttestation,
    
    /// Proof of observation
    pub observation_proof: ObservationProof,
    
    /// Observer identity
    pub observer: String,
}

/// Causal position in the operation graph
pub struct CausalPosition {
    /// Operations that happened before this fact
    pub happens_after: Vec<OperationId>,
    /// Operations that happened after this fact
    pub happens_before: Vec<OperationId>,
    /// Lamport timestamp
    pub lamport_time: u64,
}

/// Clock time attestation
pub struct ClockAttestation {
    /// Attested timestamp
    pub timestamp: Timestamp,
    /// Source of attestation
    pub source: AttestationSource,
    /// Confidence level (0.0-1.0)
    pub confidence: f64,
}
```

### 2.5 Time as an Effect

Time changes are modeled as effects in the system, allowing them to be composed, validated, and processed through the same mechanisms as other operations:

```rust
/// Time effect types
pub enum TimeEffect {
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

/// Time effect result types
pub enum TimeEffectResult {
    /// Result of a causal update
    CausalUpdate {
        graph_hash: ContentHash,
        affected_operations: Vec<OperationId>,
    },
    
    /// Result of a clock attestation
    ClockUpdate {
        domain_id: DomainId,
        timestamp: Timestamp,
        confidence: f64,
    },
    
    /// Result of a time map update
    TimeMapUpdate {
        map_hash: ContentHash,
        domains_updated: Vec<DomainId>,
    },
}
```

### 2.6 Time Services

The system provides dedicated time services that programs can choose from based on their trust requirements:

```rust
/// Base time service trait
pub trait TimeService: Send + Sync + 'static {
    /// Service name
    fn name(&self) -> &str;
    
    /// Supported capabilities
    fn capabilities(&self) -> Vec<TimeCapability>;
    
    /// Confidence level provided by this service
    fn confidence_level(&self) -> f64;
}

/// Causal time service
pub trait CausalTimeService: TimeService {
    /// Order operations according to constraints
    async fn order_operations(
        &self,
        operations: Vec<OperationId>,
        ordering: Vec<(OperationId, OperationId)>
    ) -> Result<CausalUpdateResult, TimeError>;
    
    /// Get the causal position of an operation
    async fn get_causal_position(
        &self,
        operation_id: &OperationId
    ) -> Result<Option<CausalPosition>, TimeError>;
}

/// Clock time service
pub trait ClockTimeService: TimeService {
    /// Attest to a time value
    async fn attest_time(
        &self,
        timestamp: Timestamp,
        source: AttestationSource
    ) -> Result<ClockAttestation, TimeError>;
    
    /// Verify an attestation
    async fn verify_attestation(
        &self,
        attestation: &ClockAttestation
    ) -> Result<bool, TimeError>;
}
```

### 2.7 Temporal Fact Validation

Temporal validation ensures that operations are executed with respect to the correct causal ordering and with proper time attestations:

```rust
/// Validate temporal facts
pub struct TemporalFactValidator {
    /// Time service registry
    time_service_registry: Arc<TimeServiceRegistry>,
    
    /// Current temporal context
    current_context: RwLock<TemporalContext>,
}

impl TemporalFactValidator {
    /// Validate a temporal fact against constraints
    pub async fn validate(
        &self,
        fact: &TemporalFact,
        constraints: &TemporalConstraints
    ) -> Result<bool, ValidationError> {
        // Get the current temporal context
        let context = self.current_context.read().unwrap().clone();
        
        // Validate causal ordering
        let causal_valid = self.validate_causal_ordering(
            &fact.causal_position,
            &constraints.causal_constraints
        ).await?;
        
        // Validate clock attestation
        let clock_valid = self.validate_clock_attestation(
            &fact.clock_attestation,
            &constraints.clock_constraints
        ).await?;
        
        // Validate time position
        let position_valid = self.validate_time_position(
            &fact.time_position,
            &context,
            &constraints.position_constraints
        ).await?;
        
        Ok(causal_valid && clock_valid && position_valid)
    }
    
    // Other validation methods...
}
```

### 2.8 Integration with Effect System

The time system integrates with the effect system in two ways:

1. **Time Effects**: Time updates are explicitly modeled as effects
2. **Temporal Validation**: All effects undergo temporal validation against temporal facts

```rust
/// Time effect handler
pub struct TimeEffectHandler {
    /// Causal time service
    causal_service: Arc<dyn CausalTimeService>,
    
    /// Clock time service
    clock_service: Arc<dyn ClockTimeService>,
    
    /// Time service registry
    service_registry: Arc<TimeServiceRegistry>,
}

#[async_trait]
impl EffectHandler<TimeEffect> for TimeEffectHandler {
    async fn handle(&self, effect: TimeEffect) -> Result<EffectResult, EffectError> {
        match effect {
            TimeEffect::CausalUpdate { operations, ordering } => {
                // Handle causal update through the causal time service
                let result = self.causal_service.order_operations(operations, ordering).await
                    .map_err(|e| EffectError::HandlerError(e.to_string()))?;
                
                Ok(EffectResult::Value(TimeEffectResult::CausalUpdate {
                    graph_hash: result.graph_hash,
                    affected_operations: result.affected_operations,
                }))
            },
            
            TimeEffect::ClockAttestation { domain_id, timestamp, source, confidence } => {
                // Handle clock attestation through the clock time service
                let attestation = self.clock_service.attest_time(timestamp, source).await
                    .map_err(|e| EffectError::HandlerError(e.to_string()))?;
                
                Ok(EffectResult::Value(TimeEffectResult::ClockUpdate {
                    domain_id,
                    timestamp,
                    confidence: attestation.confidence,
                }))
            },
            
            TimeEffect::TimeMapUpdate { positions, proofs } => {
                // Validate all proofs
                // Update the time map
                // ...implementation details...
                
                Ok(EffectResult::Value(TimeEffectResult::TimeMapUpdate {
                    map_hash: ContentHash::default(), // Placeholder
                    domains_updated: positions.keys().cloned().collect(),
                }))
            },
        }
    }
}

/// Execute any effect with temporal validation
pub async fn execute_effect_with_temporal_validation<E: Effect>(
    effect: &E,
    context: &EffectContext,
    executor: &dyn EffectExecutor,
    validator: &TemporalFactValidator,
) -> Result<EffectResult, EffectError> {
    // Extract fact dependencies from the effect context
    let fact_snapshot = context.fact_snapshot()
        .ok_or(EffectError::MissingFactSnapshot)?;
    
    // Validate all fact dependencies
    for fact_id in &fact_snapshot.observed_facts {
        let fact = fact_snapshot.get_fact(fact_id)
            .ok_or(EffectError::MissingFact(fact_id.clone()))?;
        
        // Validate the fact against temporal constraints
        let constraints = effect.temporal_constraints();
        let is_valid = validator.validate(fact, &constraints).await
            .map_err(|e| EffectError::ValidationError(e.to_string()))?;
        
        if !is_valid {
            return Err(EffectError::InvalidFact(fact_id.clone()));
        }
    }
    
    // Execute the effect
    executor.execute_effect(effect, context).await
}
```

## 3. Effect System [ADR-001, ADR-023, ADR-031, ADR-032]

The effect system provides a framework for expressing and executing operations with proper authorization and validation. Effects are the primary mechanism for defining state transitions and actions within Causality.

### 3.1 Algebraic Effects [ADR-023]

Causality implements a three-layer algebraic effect architecture:

1. **Effect Layer**: Core effect abstractions and interfaces
2. **Constraint Layer**: Type constraints and validation rules
3. **Implementation Layer**: Domain-specific implementations

This approach leverages Rust's type system to ensure strong compile-time guarantees while providing flexibility for domain-specific behaviors.

```rust
/// Base trait for all effects
pub trait Effect: Send + Sync + 'static {
    /// Output type produced by this effect
    type Output;
    
    /// Get the unique ID of this effect
    fn id(&self) -> &EffectId;
    
    /// Get the type of this effect
    fn effect_type(&self) -> &str;
    
    /// Get the domains this effect interacts with
    fn domains(&self) -> Vec<DomainId>;
    
    /// Get the resources this effect accesses
    fn resources(&self) -> Vec<ResourceId>;
    
    /// Get fact snapshot dependencies
    fn fact_snapshot(&self) -> Option<FactSnapshot>;
    
    /// Validate fact dependencies
    fn validate_fact_dependencies(&self) -> Result<(), EffectError>;
    
    /// Execute this effect with the given context
    fn execute(&self, context: &EffectContext) -> Result<Self::Output, EffectError>;
    
    /// Convert to any
    fn as_any(&self) -> &dyn Any;
}

/// Extension trait for effects that can be executed asynchronously
#[async_trait]
pub trait AsyncEffect: Effect {
    /// Execute this effect asynchronously
    async fn execute_async(&self, context: &EffectContext) 
        -> Result<Self::Output, EffectError>;
}

/// Registry for effect handlers
pub struct EffectRegistry {
    /// Registered handlers by effect type
    handlers: RwLock<HashMap<String, Box<dyn EffectHandler>>>,
}

impl EffectRegistry {
    /// Create a new effect registry
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a handler for an effect type
    pub fn register<H: EffectHandler + 'static>(&self, handler: H) -> Result<(), EffectError> {
        let mut handlers = self.handlers.write().map_err(|_| EffectError::LockError)?;
        
        // Register the handler for all effect types it can handle
        for effect_type in handler.handled_effect_types() {
            handlers.insert(effect_type.to_string(), Box::new(handler.clone()));
        }
        
        Ok(())
    }
    
    /// Get a handler for an effect type
    pub fn get_handler(&self, effect_type: &str) -> Result<Box<dyn EffectHandler>, EffectError> {
        let handlers = self.handlers.read().map_err(|_| EffectError::LockError)?;
        
        handlers.get(effect_type)
            .cloned()
            .ok_or_else(|| EffectError::HandlerNotFound(effect_type.to_string()))
    }
    
    /// Get all registered handlers
    pub fn get_all(&self) -> Vec<Box<dyn EffectHandler>> {
        let handlers = self.handlers.read().unwrap_or_else(|_| panic!("Failed to acquire read lock"));
        handlers.values().cloned().collect()
    }
}
```

#### Usage Example: Creating a Custom Effect

```rust
/// Custom effect for updating user preferences
#[derive(Debug, Clone)]
pub struct UpdatePreferencesEffect {
    /// Effect ID
    id: EffectId,
    /// User ID
    user_id: UserId,
    /// Preferences to update
    preferences: HashMap<String, Value>,
    /// Fact snapshot
    fact_snapshot: Option<FactSnapshot>,
}

impl Effect for UpdatePreferencesEffect {
    type Output = bool;
    
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "update_preferences"
    }
    
    fn domains(&self) -> Vec<DomainId> {
        vec![DomainId::from("user_service")]
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        vec![ResourceId::from_parts("user", &self.user_id)]
    }
    
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        self.fact_snapshot.clone()
    }
    
    fn validate_fact_dependencies(&self) -> Result<(), EffectError> {
        // No fact dependencies to validate
        Ok(())
    }
    
    fn execute(&self, context: &EffectContext) -> Result<bool, EffectError> {
        // Check if the context has the necessary capabilities
        if !context.has_capability(&Capability::new(
            self.resources()[0].clone(),
            CapabilityType::Write,
        )) {
            return Err(EffectError::MissingCapability);
        }
        
        // Execute the effect (in a real implementation, this would update a database)
        println!("Updating preferences for user {}: {:?}", self.user_id, self.preferences);
        
        Ok(true)
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Handler for user preference effects
#[derive(Clone)]
pub struct UserPreferencesHandler {
    /// User database
    user_db: Arc<UserDatabase>,
}

impl EffectHandler for UserPreferencesHandler {
    fn handled_effect_types(&self) -> Vec<&'static str> {
        vec!["update_preferences"]
    }
    
    fn can_handle_effect(&self, effect_type: &str) -> bool {
        effect_type == "update_preferences"
    }
    
    fn execute_effect(&self, effect: &dyn Effect, context: &EffectContext) 
        -> Result<EffectOutcome, EffectError> {
        // Cast to the specific effect type
        let update_effect = effect.as_any()
            .downcast_ref::<UpdatePreferencesEffect>()
            .ok_or_else(|| EffectError::TypeMismatch)?;
        
        // Execute the effect
        let success = update_effect.execute(context)?;
        
        // Create the outcome
        let outcome = EffectOutcome {
            effect: Box::new(update_effect.clone()),
            result: Value::Bool(success),
            affected_resources: update_effect.resources(),
            observed_facts: vec![],
            child_effects: vec![],
            content_hash: ContentHash::default(), // Would calculate in real implementation
        };
        
        Ok(outcome)
    }
}
```

### 3.2 Effect Constraints [ADR-023]

Effects are constrained by domain-specific traits that ensure type safety and proper validation:

```rust
/// Constraint for transfer effects
pub trait TransferEffect: Effect {
    /// Source of the transfer
    fn from(&self) -> ResourceId;
    
    /// Destination of the transfer
    fn to(&self) -> ResourceId;
    
    /// Amount to transfer
    fn quantity(&self) -> u64;
    
    /// Domain where the transfer happens
    fn domain(&self) -> DomainId;
    
    /// Asset being transferred
    fn asset(&self) -> Asset;
    
    /// Validate the transfer
    fn validate_transfer(&self, context: &EffectContext) -> Result<(), TransferError> {
        // Check if the context has the necessary capabilities
        if !context.has_capability(&Capability::new(
            self.from().clone(),
            CapabilityType::Write,
        )) {
            return Err(TransferError::MissingSourceCapability);
        }
        
        // Check for destination capability if needed
        if !self.to().is_public() && !context.has_capability(&Capability::new(
            self.to().clone(),
            CapabilityType::Write,
        )) {
            return Err(TransferError::MissingDestinationCapability);
        }
        
        // Validate the amount
        if self.quantity() == 0 {
            return Err(TransferError::ZeroAmount);
        }
        
        Ok(())
    }
}

/// Constraint for query effects
pub trait QueryEffect: Effect {
    /// Query to execute
    fn query(&self) -> &Query;
    
    /// Parameters for the query
    fn parameters(&self) -> &HashMap<String, String>;
    
    /// Validate the query
    fn validate_query(&self, context: &EffectContext) -> Result<(), QueryError> {
        // Check if the context has the necessary capabilities
        for resource in self.resources() {
            if !context.has_capability(&Capability::new(
                resource.clone(),
                CapabilityType::Read,
            )) {
                return Err(QueryError::MissingReadCapability(resource));
            }
        }
        
        // Validate query syntax
        match self.query().validate() {
            Ok(_) => Ok(()),
            Err(e) => Err(QueryError::InvalidQuery(e.to_string())),
        }
    }
}

/// Constraint for register operations
pub trait RegisterOperationEffect: Effect {
    /// Register to operate on
    fn register_id(&self) -> &RegisterId;
    
        /// Operation to perform
    fn operation(&self) -> &RegisterOperation;
    
        /// Authorization method
    fn auth_method(&self) -> &AuthorizationMethod;
    
    /// Validate the register operation
    fn validate_register_operation(&self, context: &EffectContext) -> Result<(), RegisterOperationError> {
        // Check if the context has the necessary capabilities
        let capability_type = match self.operation() {
            RegisterOperation::Read => CapabilityType::Read,
            RegisterOperation::Write(_) => CapabilityType::Write,
            RegisterOperation::Delete => CapabilityType::Write,
            RegisterOperation::Transfer(_) => CapabilityType::Owner,
        };
        
        if !context.has_capability(&Capability::new(
            ResourceId::from(self.register_id().clone()),
            capability_type,
        )) {
            return Err(RegisterOperationError::MissingCapability);
        }
        
        // Validate authorization method
        self.auth_method().validate()?;
        
        Ok(())
    }
}
```

### 3.3 Domain Adapter as Effect [ADR-031]

Domain adapters integrate with the effect system to provide seamless cross-domain operations:

```rust
/// Core domain effect trait
pub trait DomainAdapterEffect: Effect {
    /// Get the domain ID
    fn domain_id(&self) -> &DomainId;
    
    /// Create a domain context from an effect context
    fn create_context(&self, base_context: &EffectContext) -> DomainContext;
    
    /// Map domain result to effect outcome
    fn map_outcome(&self, domain_result: Result<impl Any, DomainError>) -> EffectResult<EffectOutcome>;
}

/// Domain query effect
#[derive(Debug, Clone)]
pub struct DomainQueryEffect {
    /// Effect ID
    id: EffectId,
    /// Domain ID
    domain_id: DomainId,
    /// Query to execute
    query: FactQuery,
    /// Query parameters
    parameters: HashMap<String, String>,
    /// Fact snapshot
    fact_snapshot: Option<FactSnapshot>,
}

impl Effect for DomainQueryEffect {
    type Output = QueryResult;
    
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "domain_query"
    }
    
    fn domains(&self) -> Vec<DomainId> {
        vec![self.domain_id.clone()]
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        // Derive resources from the query
        self.query.resources()
    }
    
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        self.fact_snapshot.clone()
    }
    
    fn validate_fact_dependencies(&self) -> Result<(), EffectError> {
        // Delegate to the query validator
        Ok(())
    }
    
    fn execute(&self, context: &EffectContext) -> Result<QueryResult, EffectError> {
        // This would delegate to the domain adapter
        Err(EffectError::Unimplemented("Synchronous execution not supported".to_string()))
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl DomainAdapterEffect for DomainQueryEffect {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    fn create_context(&self, base_context: &EffectContext) -> DomainContext {
        DomainContext::from_effect_context(base_context, self.domain_id.clone())
    }
    
    fn map_outcome(&self, domain_result: Result<impl Any, DomainError>) -> EffectResult<EffectOutcome> {
        match domain_result {
            Ok(result) => {
                // Try to cast the result to QueryResult
                let query_result = result.downcast_ref::<QueryResult>()
                    .ok_or_else(|| EffectError::TypeMismatch)?
                    .clone();
                
                // Create the outcome
                let outcome = EffectOutcome {
                    effect: Box::new(self.clone()),
                    result: serde_json::to_value(&query_result).unwrap_or(Value::Null),
                    affected_resources: self.resources(),
                    observed_facts: query_result.observed_facts(),
                    child_effects: vec![],
                    content_hash: ContentHash::default(), // Would calculate in real implementation
                };
                
                Ok(outcome)
            },
            Err(e) => Err(EffectError::DomainError(e.to_string())),
        }
    }
}

/// Registry for domain effects
pub struct DomainEffectRegistry {
    /// Underlying adapter registry
    adapter_registry: Arc<DomainAdapterRegistry>,
    /// Domain factories
    domain_factories: HashMap<String, Arc<dyn DomainAdapterFactory>>,
    /// Capability manager
    capability_manager: Arc<DomainCapabilityManager>,
}

impl DomainEffectRegistry {
    /// Create a new domain effect registry
    pub fn new(
        adapter_registry: Arc<DomainAdapterRegistry>,
        domain_factories: HashMap<String, Arc<dyn DomainAdapterFactory>>,
        capability_manager: Arc<DomainCapabilityManager>,
    ) -> Self {
        Self {
            adapter_registry,
            domain_factories,
            capability_manager,
        }
    }
    
    /// Register a domain adapter
    pub fn register_adapter(&self, adapter: Arc<dyn DomainAdapter>) -> Result<(), EffectError> {
        self.adapter_registry.register_adapter(adapter)
            .map_err(|e| EffectError::RegistrationError(e.to_string()))
    }
    
    /// Create a domain query effect
    pub fn create_query_effect(
        &self,
        domain_id: DomainId,
        query: FactQuery,
        parameters: HashMap<String, String>,
    ) -> Result<DomainQueryEffect, EffectError> {
        // Check if the domain is supported
        if !self.adapter_registry.has_adapter(&domain_id) {
            return Err(EffectError::UnsupportedDomain(domain_id.to_string()));
        }
        
        // Create the effect
        let effect = DomainQueryEffect {
            id: EffectId::new(),
            domain_id,
            query,
            parameters,
            fact_snapshot: None,
        };
        
        Ok(effect)
    }
}

impl EffectHandler for DomainEffectRegistry {
    fn handled_effect_types(&self) -> Vec<&'static str> {
        vec!["domain_query", "domain_transaction", "domain_time_map", "domain_capability"]
    }
    
    fn can_handle_effect(&self, effect_type: &str) -> bool {
        matches!(
            effect_type,
            "domain_query" | "domain_transaction" | 
            "domain_time_map" | "domain_capability"
        )
    }
    
    fn execute_effect(&self, effect: &dyn Effect, context: &EffectContext) 
        -> Result<EffectOutcome, EffectError> {
        match effect.effect_type() {
            "domain_query" => self.handle_query_effect(effect, context),
            "domain_transaction" => self.handle_transaction_effect(effect, context),
            "domain_time_map" => self.handle_time_map_effect(effect, context),
            "domain_capability" => self.handle_capability_effect(effect, context),
            _ => Err(EffectError::UnsupportedEffect(effect.effect_type().to_string())),
        }
    }
}
```

### 3.4 Effect Execution Lifecycle

Effects go through a well-defined lifecycle during execution:

1. **Creation**: An effect is created with specific parameters and dependencies
2. **Validation**: The effect is validated for proper capabilities and constraints
3. **Execution**: The effect is executed to produce an outcome
4. **Recording**: The effect and its outcome are recorded in the effect log
5. **Notification**: Observers are notified of the effect's completion

This lifecycle ensures reliable, auditable execution with proper authorization checks.

```rust
/// Engine for executing effects
pub struct EffectEngine {
    /// Registry of effect handlers
    registry: Arc<EffectRegistry>,
    /// Capability verifier
    capability_verifier: Arc<CapabilityVerifier>,
    /// Temporal fact validator
    temporal_validator: Arc<TemporalFactValidator>,
    /// Effect log
    effect_log: Arc<dyn EffectLog>,
}

impl EffectEngine {
    /// Create a new effect engine
    pub fn new(
        registry: Arc<EffectRegistry>,
        capability_verifier: Arc<CapabilityVerifier>,
        temporal_validator: Arc<TemporalFactValidator>,
        effect_log: Arc<dyn EffectLog>,
    ) -> Self {
        Self {
            registry,
            capability_verifier,
            temporal_validator,
            effect_log,
        }
    }
    
    /// Execute an effect
    pub async fn execute(&self, effect: &dyn Effect, context: &EffectContext) 
        -> Result<EffectOutcome, EffectError> {
        // 1. Validate capabilities
        self.validate_capabilities(effect, context)?;
        
        // 2. Validate fact dependencies
        self.validate_fact_dependencies(effect).await?;
        
        // 3. Get the appropriate handler
        let handler = self.registry.get_handler(effect.effect_type())?;
        
        // 4. Execute the effect
        let outcome = handler.execute_effect(effect, context)?;
        
        // 5. Record the effect and outcome
        self.effect_log.record(effect, &outcome).await?;
        
        // 6. Notify observers
        self.notify_observers(effect, &outcome).await?;
        
        Ok(outcome)
    }
    
    /// Validate that the context has the necessary capabilities for the effect
    fn validate_capabilities(&self, effect: &dyn Effect, context: &EffectContext) 
        -> Result<(), EffectError> {
        // Check capabilities for each resource
        for resource in effect.resources() {
            let capability_type = match effect.effect_type() {
                "transfer" => CapabilityType::Write,
                "query" => CapabilityType::Read,
                "invoke" => CapabilityType::Execute,
                _ => CapabilityType::from_effect_type(effect.effect_type())?,
            };
            
            let has_capability = self.capability_verifier.has_capability(
                &context.identity(),
                &resource,
                &capability_type,
            )?;
            
            if !has_capability {
                return Err(EffectError::MissingCapability);
            }
        }
        
        Ok(())
    }
    
    /// Validate fact dependencies for an effect
    async fn validate_fact_dependencies(&self, effect: &dyn Effect) 
        -> Result<(), EffectError> {
        // Get the fact snapshot
        let fact_snapshot = effect.fact_snapshot()
            .ok_or(EffectError::MissingFactSnapshot)?;
        
        // Validate each fact
        for fact_id in &fact_snapshot.observed_facts {
            let fact = fact_snapshot.get_fact(fact_id)
                .ok_or(EffectError::MissingFact(fact_id.clone()))?;
            
            let is_valid = self.temporal_validator.validate(
                fact,
                &TemporalConstraints::default(),
            ).await.map_err(|e| EffectError::ValidationError(e.to_string()))?;
            
            if !is_valid {
                return Err(EffectError::InvalidFact(fact_id.clone()));
            }
        }
        
        Ok(())
    }
    
    /// Notify observers of an effect's completion
    async fn notify_observers(&self, effect: &dyn Effect, outcome: &EffectOutcome) 
        -> Result<(), EffectError> {
        // In a real implementation, this would notify interested observers
        Ok(())
    }
}
```

### 3.5 Cross-Domain Effect Composition

Effects can be composed across domains for complex workflows:

```rust
/// Effect for cross-domain asset transfer
#[derive(Debug, Clone)]
pub struct CrossDomainTransferEffect {
    /// Effect ID
    id: EffectId,
    /// Source domain
    source_domain: DomainId,
        /// Target domain
        target_domain: DomainId,
    /// Source account
    source_account: ResourceId,
    /// Target account
    target_account: ResourceId,
    /// Asset to transfer
    asset: Asset,
    /// Amount to transfer
    amount: u64,
    /// Fact snapshot
    fact_snapshot: Option<FactSnapshot>,
}

impl Effect for CrossDomainTransferEffect {
    type Output = TransferResult;
    
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "cross_domain_transfer"
    }
    
    fn domains(&self) -> Vec<DomainId> {
        vec![self.source_domain.clone(), self.target_domain.clone()]
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        vec![self.source_account.clone(), self.target_account.clone()]
    }
    
    fn fact_snapshot(&self) -> Option<FactSnapshot> {
        self.fact_snapshot.clone()
    }
    
    fn validate_fact_dependencies(&self) -> Result<(), EffectError> {
        // This would validate that the source account has sufficient balance
        Ok(())
    }
    
    fn execute(&self, context: &EffectContext) -> Result<TransferResult, EffectError> {
        // This would delegate to a handler
        Err(EffectError::Unimplemented("Synchronous execution not supported".to_string()))
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Handler for cross-domain transfers
pub struct CrossDomainTransferHandler {
    /// Domain effect registry
    domain_registry: Arc<DomainEffectRegistry>,
    /// Time map manager
    time_map_manager: Arc<TimeMapManager>,
}

impl EffectHandler for CrossDomainTransferHandler {
    fn handled_effect_types(&self) -> Vec<&'static str> {
        vec!["cross_domain_transfer"]
    }
    
    fn can_handle_effect(&self, effect_type: &str) -> bool {
        effect_type == "cross_domain_transfer"
    }
    
    async fn execute_effect_async(&self, effect: &dyn Effect, context: &EffectContext) 
        -> Result<EffectOutcome, EffectError> {
        // Cast to the specific effect type
        let transfer_effect = effect.as_any()
            .downcast_ref::<CrossDomainTransferEffect>()
            .ok_or_else(|| EffectError::TypeMismatch)?;
        
        // 1. Create a withdrawal effect for the source domain
        let withdrawal_effect = self.domain_registry.create_withdrawal_effect(
            transfer_effect.source_domain.clone(),
            transfer_effect.source_account.clone(),
            transfer_effect.asset.clone(),
            transfer_effect.amount,
        )?;
        
        // 2. Execute the withdrawal
        let withdrawal_outcome = self.domain_registry.execute_effect(&withdrawal_effect, context).await?;
        
        // 3. Create a deposit effect for the target domain
        let deposit_effect = self.domain_registry.create_deposit_effect(
            transfer_effect.target_domain.clone(),
            transfer_effect.target_account.clone(),
            transfer_effect.asset.clone(),
            transfer_effect.amount,
        )?;
        
        // 4. Execute the deposit
        let deposit_outcome = self.domain_registry.execute_effect(&deposit_effect, context).await?;
        
        // 5. Create the final outcome
        let outcome = EffectOutcome {
            effect: Box::new(transfer_effect.clone()),
            result: TransferResult {
                source_transaction: withdrawal_outcome.result["transaction_id"].as_str().unwrap_or("").to_string(),
                target_transaction: deposit_outcome.result["transaction_id"].as_str().unwrap_or("").to_string(),
                amount: transfer_effect.amount,
                status: TransferStatus::Completed,
            }.into(),
            affected_resources: transfer_effect.resources(),
            observed_facts: vec![],
            child_effects: vec![
                Box::new(withdrawal_effect),
                Box::new(deposit_effect),
            ],
            content_hash: ContentHash::default(), // Would calculate in real implementation
        };
        
        Ok(outcome)
    }
}
```

### 3.6 Effect Interpreter [ADR-032]

The effect interpreter is responsible for executing effects and producing outcomes:

```rust
/// Effect interpreter
pub struct EffectInterpreter {
    /// Effect registry
    registry: Arc<EffectRegistry>,
    /// Capability verifier
    capability_verifier: Arc<CapabilityVerifier>,
    /// Temporal fact validator
    temporal_validator: Arc<TemporalFactValidator>,
    /// Effect log
    effect_log: Arc<dyn EffectLog>,
}

impl EffectInterpreter {
    /// Create a new effect interpreter
    pub fn new(
        registry: Arc<EffectRegistry>,
        capability_verifier: Arc<CapabilityVerifier>,
        temporal_validator: Arc<TemporalFactValidator>,
        effect_log: Arc<dyn EffectLog>,
    ) -> Self {
        Self {
            registry,
            capability_verifier,
            temporal_validator,
            effect_log,
        }
    }
    
    /// Execute an effect
    pub async fn execute(&self, effect: &dyn Effect, context: &EffectContext) 
        -> Result<EffectOutcome, EffectError> {
        // 1. Validate capabilities
        self.validate_capabilities(effect, context)?;
        
        // 2. Validate fact dependencies
        self.validate_fact_dependencies(effect).await?;
        
        // 3. Get the appropriate handler
        let handler = self.registry.get_handler(effect.effect_type())?;
        
        // 4. Execute the effect
        let outcome = handler.execute_effect(effect, context)?;
        
        // 5. Record the effect and outcome
        self.effect_log.record(effect, &outcome).await?;
        
        // 6. Notify observers
        self.notify_observers(effect, &outcome).await?;
        
        Ok(outcome)
    }
    
    /// Validate that the context has the necessary capabilities for the effect
    fn validate_capabilities(&self, effect: &dyn Effect, context: &EffectContext) 
        -> Result<(), EffectError> {
        // Check capabilities for each resource
        for resource in effect.resources() {
            let capability_type = match effect.effect_type() {
                "read" | "query" => CapabilityType::Read,
                "write" | "update" => CapabilityType::Write,
                "execute" | "invoke" => CapabilityType::Execute,
                "admin" | "delegate" => CapabilityType::Admin,
                "transfer" | "ownership" => CapabilityType::Owner,
                _ => {
                    // Default to read for unknown effect types
                    CapabilityType::Read
                }
            };
            
            // Verify the capability
            let has_capability = self.capability_verifier.has_capability(
                &context.identity(),
                &resource,
                &capability_type,
            )?;
            
            if !has_capability {
                // Missing capability
                return Err(EffectError::MissingCapability);
            }
        }
        
        Ok(())
    }
    
    /// Validate fact dependencies for an effect
    async fn validate_fact_dependencies(&self, effect: &dyn Effect) 
        -> Result<(), EffectError> {
        // Get the fact snapshot
        let fact_snapshot = effect.fact_snapshot()
            .ok_or(EffectError::MissingFactSnapshot)?;
        
        // Validate each fact
        for fact_id in &fact_snapshot.observed_facts {
            let fact = fact_snapshot.get_fact(fact_id)
                .ok_or(EffectError::MissingFact(fact_id.clone()))?;
            
            let is_valid = self.temporal_validator.validate(
                fact,
                &TemporalConstraints::default(),
            ).await.map_err(|e| EffectError::ValidationError(e.to_string()))?;
            
            if !is_valid {
                return Err(EffectError::InvalidFact(fact_id.clone()));
            }
        }
        
        Ok(())
    }
    
    /// Notify observers of an effect's completion
    async fn notify_observers(&self, effect: &dyn Effect, outcome: &EffectOutcome) 
        -> Result<(), EffectError> {
        // In a real implementation, this would notify interested observers
        Ok(())
    }
}
```

### 3.7 Operation and Effect Integration [ADR-032]

Operations are defined as requests to perform effects with explicit authorization:

```rust
/// Operation for cross-domain asset transfer
#[derive(Debug, Clone)]
pub struct CrossDomainTransferOperation {
    /// Source domain
    source_domain: DomainId,
    /// Target domain
    target_domain: DomainId,
    /// Source account
    source_account: ResourceId,
    /// Target account
    target_account: ResourceId,
    /// Asset to transfer
    asset: Asset,
    /// Amount to transfer
    amount: u64,
    /// Authorization
    authorization: Capability,
}

impl Operation for CrossDomainTransferOperation {
    type Effect = CrossDomainTransferEffect;
    
    fn id(&self) -> &OperationId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "cross_domain_transfer"
    }
    
    fn domains(&self) -> Vec<DomainId> {
        vec![self.source_domain.clone(), self.target_domain.clone()]
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        vec![self.source_account.clone(), self.target_account.clone()]
    }
    
    fn authorization(&self) -> &Capability {
        &self.authorization
    }
    
    fn validate(&self, context: &EffectContext) -> Result<(), OperationError> {
        // Validate the authorization
        if !context.has_capability(&self.authorization) {
            return Err(OperationError::MissingCapability);
        }
        
        // Validate the amount
        if self.amount == 0 {
            return Err(OperationError::ZeroAmount);
        }
        
        Ok(())
    }
    
    fn execute(&self, context: &EffectContext) -> Result<Self::Effect, OperationError> {
        // Create the effect
        let effect = CrossDomainTransferEffect {
            id: EffectId::new(),
            source_domain: self.source_domain.clone(),
            target_domain: self.target_domain.clone(),
            source_account: self.source_account.clone(),
            target_account: self.target_account.clone(),
            asset: self.asset.clone(),
            amount: self.amount,
            fact_snapshot: None,
        };
        
        Ok(effect)
    }
}
```

## 4. Resource System [ADR-002, ADR-030, ADR-032]

The resource system manages access to state across domains through a unified abstraction layer that ensures consistency and proper authorization.

### 4.1 Resource Model [ADR-002, ADR-032]

Resources are the fundamental unit of state in Causality. Each resource:

1. Is uniquely identified by a content-addressed ID
2. Has a well-defined type (account, token, contract, etc.)
3. Can be accessed through capabilities
4. Maintains its own state history

```rust
/// Unique identifier for a resource
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId {
    /// Type of the resource
    resource_type: String,
    /// Domain where the resource is located
    domain: DomainId,
    /// Unique identifier within the domain
    id: String,
    /// Content hash of this resource ID
    content_hash: ContentHash,
}

impl ResourceId {
    /// Create a new resource ID
    pub fn new(resource_type: &str, domain: DomainId, id: &str) -> Result<Self, ResourceError> {
        let mut result = Self {
            resource_type: resource_type.to_string(),
            domain,
            id: id.to_string(),
            content_hash: ContentHash::default(),
        };
        
        // Calculate the content hash
        result.content_hash = result.calculate_content_hash()?;
        
        Ok(result)
    }
    
    /// Create a new resource ID from parts
    pub fn from_parts(resource_type: &str, id: &str) -> Result<Self, ResourceError> {
        // Extract domain from ID if it contains a separator
        if let Some(pos) = id.find(':') {
            let domain = DomainId::from(&id[..pos]);
            let local_id = &id[pos+1..];
            Self::new(resource_type, domain, local_id)
        } else {
            // Use default domain
            Self::new(resource_type, DomainId::from("default"), id)
        }
    }
    
    /// Get the resource type
    pub fn resource_type(&self) -> &str {
        &self.resource_type
    }
    
    /// Get the domain
    pub fn domain(&self) -> &DomainId {
        &self.domain
    }
    
    /// Get the ID
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Get the fully qualified path
    pub fn path(&self) -> String {
        format!("{}/{}/{}", self.resource_type, self.domain, self.id)
    }
    
    /// Check if this is a public resource
    pub fn is_public(&self) -> bool {
        self.id == "public" || self.id.starts_with("public/")
    }
}

impl ContentAddressed for ResourceId {
    fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError> {
        let mut hasher = ContentHasher::new();
        hasher.update("ResourceId");
        hasher.update(&self.resource_type);
        hasher.update(self.domain.as_str());
        hasher.update(&self.id);
        Ok(hasher.finalize())
    }
    
    fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }
    
    fn with_content_hash(mut self, hash: ContentHash) -> Self {
        self.content_hash = hash;
        self
    }
    
    fn verify_content_hash(&self) -> bool {
        match self.calculate_content_hash() {
            Ok(hash) => &hash == &self.content_hash,
            Err(_) => false,
        }
    }
}
```

### 4.2 Resource Operations [ADR-032]

Resource operations are defined as requests to perform effects with explicit authorization:

```rust
/// Resource operation for cross-domain asset transfer
#[derive(Debug, Clone)]
pub struct CrossDomainTransferOperation {
    /// Source domain
    source_domain: DomainId,
    /// Target domain
    target_domain: DomainId,
    /// Source account
    source_account: ResourceId,
    /// Target account
    target_account: ResourceId,
    /// Asset to transfer
    asset: Asset,
    /// Amount to transfer
    amount: u64,
    /// Authorization
    authorization: Capability,
}

impl Operation for CrossDomainTransferOperation {
    type Effect = CrossDomainTransferEffect;
    
    fn id(&self) -> &OperationId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "cross_domain_transfer"
    }
    
    fn domains(&self) -> Vec<DomainId> {
        vec![self.source_domain.clone(), self.target_domain.clone()]
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        vec![self.source_account.clone(), self.target_account.clone()]
    }
    
    fn authorization(&self) -> &Capability {
        &self.authorization
    }
    
    fn validate(&self, context: &EffectContext) -> Result<(), OperationError> {
        // Validate the authorization
        if !context.has_capability(&self.authorization) {
            return Err(OperationError::MissingCapability);
        }
        
        // Validate the amount
        if self.amount == 0 {
            return Err(OperationError::ZeroAmount);
        }
        
        Ok(())
    }
    
    fn execute(&self, context: &EffectContext) -> Result<Self::Effect, OperationError> {
        // Create the effect
        let effect = CrossDomainTransferEffect {
            id: EffectId::new(),
            source_domain: self.source_domain.clone(),
            target_domain: self.target_domain.clone(),
            source_account: self.source_account.clone(),
            target_account: self.target_account.clone(),
            asset: self.asset.clone(),
            amount: self.amount,
            fact_snapshot: None,
        };
        
        Ok(effect)
    }
}
```

### 4.3 Resource Logic [ADR-032]

Resource logic is defined as the rules and behaviors associated with resources:

```rust
/// Resource logic for cross-domain asset transfer
#[derive(Debug, Clone)]
pub struct CrossDomainTransferLogic {
    /// Source domain
    source_domain: DomainId,
    /// Target domain
    target_domain: DomainId,
    /// Source account
    source_account: ResourceId,
    /// Target account
    target_account: ResourceId,
    /// Asset to transfer
    asset: Asset,
    /// Amount to transfer
    amount: u64,
}

impl ResourceLogic for CrossDomainTransferLogic {
    type Effect = CrossDomainTransferEffect;
    
    fn id(&self) -> &ResourceId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "cross_domain_transfer"
    }
    
    fn domains(&self) -> Vec<DomainId> {
        vec![self.source_domain.clone(), self.target_domain.clone()]
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        vec![self.source_account.clone(), self.target_account.clone()]
    }
    
    fn authorization(&self) -> &Capability {
        // This would be determined by the resource logic
        unimplemented!()
    }
    
    fn validate(&self, context: &EffectContext) -> Result<(), ResourceLogicError> {
        // Validate the authorization
        if !context.has_capability(&self.authorization) {
            return Err(ResourceLogicError::MissingCapability);
        }
        
        // Validate the amount
        if self.amount == 0 {
            return Err(ResourceLogicError::ZeroAmount);
        }
        
        Ok(())
    }
    
    fn execute(&self, context: &EffectContext) -> Result<Self::Effect, ResourceLogicError> {
        // Create the effect
        let effect = CrossDomainTransferEffect {
            id: EffectId::new(),
            source_domain: self.source_domain.clone(),
            target_domain: self.target_domain.clone(),
            source_account: self.source_account.clone(),
            target_account: self.target_account.clone(),
            asset: self.asset.clone(),
            amount: self.amount,
            fact_snapshot: None,
        };
        
        Ok(effect)
    }
}
```

### 4.4 Resource Lifecycle [ADR-032]

Resource lifecycle management is defined as the rules and behaviors associated with the creation, update, and deletion of resources:

```rust
/// Resource lifecycle for cross-domain asset transfer
#[derive(Debug, Clone)]
pub struct CrossDomainTransferLifecycle {
    /// Source domain
    source_domain: DomainId,
    /// Target domain
    target_domain: DomainId,
    /// Source account
    source_account: ResourceId,
    /// Target account
    target_account: ResourceId,
    /// Asset to transfer
    asset: Asset,
    /// Amount to transfer
    amount: u64,
}

impl ResourceLifecycle for CrossDomainTransferLifecycle {
    type Effect = CrossDomainTransferEffect;
    
    fn id(&self) -> &ResourceId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "cross_domain_transfer"
    }
    
    fn domains(&self) -> Vec<DomainId> {
        vec![self.source_domain.clone(), self.target_domain.clone()]
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        vec![self.source_account.clone(), self.target_account.clone()]
    }
    
    fn authorization(&self) -> &Capability {
        // This would be determined by the resource lifecycle
        unimplemented!()
    }
    
    fn validate(&self, context: &EffectContext) -> Result<(), ResourceLifecycleError> {
        // Validate the authorization
        if !context.has_capability(&self.authorization) {
            return Err(ResourceLifecycleError::MissingCapability);
        }
        
        // Validate the amount
        if self.amount == 0 {
            return Err(ResourceLifecycleError::ZeroAmount);
        }
        
        Ok(())
    }
    
    fn execute(&self, context: &EffectContext) -> Result<Self::Effect, ResourceLifecycleError> {
        // Create the effect
        let effect = CrossDomainTransferEffect {
            id: EffectId::new(),
            source_domain: self.source_domain.clone(),
            target_domain: self.target_domain.clone(),
            source_account: self.source_account.clone(),
            target_account: self.target_account.clone(),
            asset: self.asset.clone(),
            amount: self.amount,
            fact_snapshot: None,
        };
        
        Ok(effect)
    }
}
```

### 4.5 Resource System Interaction Diagram

```
┌──────────────────────────┐     ┌──────────────────────────┐
│                          │     │                          │
│     Effect System        │     │    Capability System     │
│                          │     │                          │
└───────────┬──────────────┘     └───────────┬──────────────┘
            │                                 │
            │                                 │
            │    ┌─────────────────────┐      │
            └───►│                     │◄─────┘
                 │   Resource System   │
                 │                     │
                 └──────┬──────┬──────┘
                        │      │
                        │      │
            ┌───────────┘      └───────────┐
            │                              │
┌───────────▼────────────┐    ┌───────────▼────────────┐
│                        │    │                        │
│  Domain Adapter A      │    │  Domain Adapter B      │
│  (Blockchain)          │    │  (Database)            │
│                        │    │                        │
└────────────────────────┘    └────────────────────────┘
```

The Resource System serves as a mediator between the Effect System, Capability System, and Domain Adapters:

1. **Effect Execution**: When an effect is executed, it requests resources through the Resource System
2. **Capability Verification**: The Resource System verifies capabilities through the Capability System
3. **Resource Access**: The Resource System delegates to the appropriate Domain Adapter for resource access
4. **State Changes**: Resource state changes are propagated back through the system

### 4.6 Resource Query Language

Resources can be queried using a flexible query language:

```rust
/// Resource query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuery {
    /// Resource type
    resource_type: String,
    /// Domain
    domain: Option<DomainId>,
    /// Filters
    filters: Vec<Filter>,
    /// Sort order
    sort: Vec<Sort>,
    /// Limit
    limit: Option<usize>,
    /// Offset
    offset: Option<usize>,
}

/// Filter for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Filter {
    /// Equals
    Eq(String, Value),
    /// Not equals
    Ne(String, Value),
    /// Greater than
    Gt(String, Value),
    /// Greater than or equal
    Ge(String, Value),
    /// Less than
    Lt(String, Value),
    /// Less than or equal
    Le(String, Value),
    /// In a set
    In(String, Vec<Value>),
    /// Contains a substring
    Contains(String, String),
    /// Starts with a prefix
    StartsWith(String, String),
    /// Ends with a suffix
    EndsWith(String, String),
    /// And
    And(Vec<Filter>),
    /// Or
    Or(Vec<Filter>),
    /// Not
    Not(Box<Filter>),
}

/// Sort order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    /// Field
    field: String,
    /// Direction
    direction: SortDirection,
}

/// Sort direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SortDirection {
    /// Ascending
    Asc,
    /// Descending
    Desc,
}

impl ResourceQuery {
    /// Create a new resource query
    pub fn new(resource_type: &str) -> Self {
        Self {
            resource_type: resource_type.to_string(),
            domain: None,
            filters: Vec::new(),
            sort: Vec::new(),
            limit: None,
            offset: None,
        }
    }
    
    /// Set the domain
    pub fn with_domain(mut self, domain: DomainId) -> Self {
        self.domain = Some(domain);
        self
    }
    
    /// Add a filter
    pub fn with_filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }
    
    /// Add a sort
    pub fn with_sort(mut self, field: &str, direction: SortDirection) -> Self {
        self.sort.push(Sort {
            field: field.to_string(),
            direction,
        });
        self
    }
    
    /// Set the limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Set the offset
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
    
    /// Check if a resource matches this query
    pub fn matches<T: Serialize>(&self, resource: &T) -> bool {
        // Convert to a Value
        let value = match serde_json::to_value(resource) {
            Ok(v) => v,
            Err(_) => return false,
        };
        
        // Apply filters
        self.apply_filters(&value)
    }
    
    /// Apply filters to a value
    fn apply_filters(&self, value: &Value) -> bool {
        // If no filters, match everything
        if self.filters.is_empty() {
            return true;
        }
        
        // Apply all filters (AND)
        for filter in &self.filters {
            if !Self::apply_filter(filter, value) {
                return false;
            }
        }
        
        true
    }
    
    /// Apply a filter to a value
    fn apply_filter(filter: &Filter, value: &Value) -> bool {
        match filter {
            Filter::Eq(field, val) => Self::get_field(value, field) == Some(val),
            Filter::Ne(field, val) => Self::get_field(value, field) != Some(val),
            Filter::Gt(field, val) => {
                if let Some(v) = Self::get_field(value, field) {
                    v > val
                } else {
                    false
                }
            },
            // ... other filter types ...
            Filter::And(filters) => {
                for f in filters {
                    if !Self::apply_filter(f, value) {
                        return false;
                    }
                }
                true
            },
            Filter::Or(filters) => {
                for f in filters {
                    if Self::apply_filter(f, value) {
                        return true;
                    }
                }
                false
            },
            Filter::Not(f) => !Self::apply_filter(f, value),
            // ... other filter types ...
            _ => false,
        }
    }
    
    /// Get a field from a value
    fn get_field<'a>(value: &'a Value, field: &str) -> Option<&'a Value> {
        if let Value::Object(obj) = value {
            obj.get(field)
        } else {
            None
        }
    }
}
```

## 5. Capability System [ADR-003, ADR-032, ADR-032]

The capability system provides a secure authorization model that governs access to resources based on unforgeable capability tokens with explicit delegation paths.

### 5.1 Capability Model [ADR-003]

Capabilities in Causality follow these core principles:

1. **Unforgeable**: Each capability has a cryptographically secure content hash
2. **Delegatable**: Capabilities can be delegated to create capability chains
3. **Revocable**: Capabilities can be revoked without affecting other capability chains
4. **Attenuable**: Capabilities can be restricted when delegated

```rust
/// Capability token that grants access to a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Target resource
    target: ResourceId,
    /// Capability type (read, write, etc.)
    capability_type: CapabilityType,
    /// Constraints on this capability
    constraints: Vec<CapabilityConstraint>,
    /// Expiration time (if any)
    expires_at: Option<DateTime<Utc>>,
    /// Content hash of this capability
    content_hash: ContentHash,
}

/// Type of capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityType {
    /// Read access
    Read,
    /// Write access
    Write,
    /// Execute access
    Execute,
    /// Admin access (can delegate)
    Admin,
    /// Owner access (can transfer ownership)
    Owner,
}

/// Constraint on a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityConstraint {
    /// Time constraint
    Time(TimeConstraint),
    /// Resource field constraint
    Field(FieldConstraint),
    /// Operation constraint
    Operation(OperationConstraint),
    /// Custom constraint
    Custom(CustomConstraint),
}

impl Capability {
    /// Create a new capability
    pub fn new(target: ResourceId, capability_type: CapabilityType) -> Result<Self, CapabilityError> {
        let mut result = Self {
            target,
            capability_type,
            constraints: Vec::new(),
            expires_at: None,
            content_hash: ContentHash::default(),
        };
        
        // Calculate the content hash
        result.content_hash = result.calculate_content_hash()?;
        
        Ok(result)
    }
    
    /// Add a constraint
    pub fn with_constraint(mut self, constraint: CapabilityConstraint) -> Result<Self, CapabilityError> {
        self.constraints.push(constraint);
        
        // Recalculate the content hash
        self.content_hash = self.calculate_content_hash()?;
        
        Ok(self)
    }
    
    /// Set an expiration time
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Result<Self, CapabilityError> {
        self.expires_at = Some(expires_at);
        
        // Recalculate the content hash
        self.content_hash = self.calculate_content_hash()?;
        
        Ok(self)
    }
    
    /// Get the target resource
    pub fn target(&self) -> &ResourceId {
        &self.target
    }
    
    /// Get the capability type
    pub fn capability_type(&self) -> CapabilityType {
        self.capability_type
    }
    
    /// Get the constraints
    pub fn constraints(&self) -> &[CapabilityConstraint] {
        &self.constraints
    }
    
    /// Get the expiration time
    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.expires_at
    }
    
    /// Check if this capability has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < Utc::now()
        } else {
            false
        }
    }
    
    /// Check if this capability subsumes another
    pub fn subsumes(&self, other: &Capability) -> bool {
        // Check target and type
        if self.target != other.target {
            return false;
        }
        
        if !self.capability_type_subsumes(other.capability_type) {
            return false;
        }
        
        // Check expiration
        if let Some(self_expires) = self.expires_at {
            if let Some(other_expires) = other.expires_at {
                if self_expires < other_expires {
                    return false;
                }
            }
        }
        
        // Check constraints
        for other_constraint in &other.constraints {
            if !self.constraints.iter().any(|self_constraint| {
                self_constraint.subsumes(other_constraint)
            }) {
                return false;
            }
        }
        
        true
    }
    
    /// Check if this capability type subsumes another
    fn capability_type_subsumes(&self, other: CapabilityType) -> bool {
        match self.capability_type {
            CapabilityType::Owner => true, // Owner can do anything
            CapabilityType::Admin => {
                // Admin can do anything except transfer ownership
                other != CapabilityType::Owner
            },
            CapabilityType::Write => {
                // Write can do write and read
                matches!(other, CapabilityType::Write | CapabilityType::Read)
            },
            CapabilityType::Execute => {
                // Execute can only execute
                other == CapabilityType::Execute
            },
            CapabilityType::Read => {
                // Read can only read
                other == CapabilityType::Read
            },
        }
    }
}

impl ContentAddressed for Capability {
    fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError> {
        let mut hasher = ContentHasher::new();
        hasher.update("Capability");
        hasher.update(self.target.content_hash().as_bytes());
        hasher.update(&format!("{:?}", self.capability_type));
        
        for constraint in &self.constraints {
            hasher.update(&format!("{:?}", constraint));
        }
        
        if let Some(expires_at) = self.expires_at {
            hasher.update(&expires_at.to_rfc3339());
        }
        
        Ok(hasher.finalize())
    }
    
    fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }
    
    fn with_content_hash(mut self, hash: ContentHash) -> Self {
        self.content_hash = hash;
        self
    }
    
    fn verify_content_hash(&self) -> bool {
        match self.calculate_content_hash() {
            Ok(hash) => &hash == &self.content_hash,
            Err(_) => false,
        }
    }
}
```

### 5.2 Capability Delegation [ADR-032]

Capabilities can be delegated to create verifiable authorization chains:

```rust
/// Capability delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDelegation {
    /// Delegator identity
    delegator: Identity,
    /// Delegatee identity
    delegatee: Identity,
    /// Source capability
    source: ContentHash,
    /// Derived capability
    derived: Capability,
    /// Signature from the delegator
    signature: Signature,
    /// Content hash of this delegation
    content_hash: ContentHash,
}

impl CapabilityDelegation {
    /// Create a new capability delegation
    pub fn new(
        delegator: Identity,
        delegatee: Identity,
        source: ContentHash,
        derived: Capability,
    ) -> Result<Self, CapabilityError> {
        let mut result = Self {
            delegator,
            delegatee,
            source,
            derived,
            signature: Signature::default(),
            content_hash: ContentHash::default(),
        };
        
        // Calculate the content hash
        result.content_hash = result.calculate_content_hash()?;
        
        Ok(result)
    }
    
    /// Sign this delegation
    pub fn sign(mut self, signer: &dyn Signer) -> Result<Self, CapabilityError> {
        // Create a serialized representation for signing
        let serialized = serde_json::to_string(&self)
            .map_err(|e| CapabilityError::SerializationError(e.to_string()))?;
        
        // Sign the serialized delegation
        self.signature = signer.sign(serialized.as_bytes())
            .map_err(|e| CapabilityError::SigningError(e.to_string()))?;
        
        // Recalculate the content hash
        self.content_hash = self.calculate_content_hash()?;
        
        Ok(self)
    }
    
    /// Verify this delegation
    pub fn verify(&self, verifier: &dyn Verifier) -> Result<bool, CapabilityError> {
        // Create a serialized representation for verification
        let mut verification_copy = self.clone();
        verification_copy.signature = Signature::default();
        
        let serialized = serde_json::to_string(&verification_copy)
            .map_err(|e| CapabilityError::SerializationError(e.to_string()))?;
        
        // Verify the signature
        let is_valid = verifier.verify(
            serialized.as_bytes(),
            &self.signature,
            &self.delegator,
        ).map_err(|e| CapabilityError::VerificationError(e.to_string()))?;
        
        Ok(is_valid)
    }
    
    /// Get the delegator
    pub fn delegator(&self) -> &Identity {
        &self.delegator
    }
    
    /// Get the delegatee
    pub fn delegatee(&self) -> &Identity {
        &self.delegatee
    }
    
    /// Get the source capability
    pub fn source(&self) -> &ContentHash {
        &self.source
    }
    
    /// Get the derived capability
    pub fn derived(&self) -> &Capability {
        &self.derived
    }
}

impl ContentAddressed for CapabilityDelegation {
    fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError> {
        let mut hasher = ContentHasher::new();
        hasher.update("CapabilityDelegation");
        hasher.update(self.delegator.content_hash().as_bytes());
        hasher.update(self.delegatee.content_hash().as_bytes());
        hasher.update(self.source.as_bytes());
        hasher.update(self.derived.content_hash().as_bytes());
        hasher.update(self.signature.as_bytes());
        
        Ok(hasher.finalize())
    }
    
    fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }
    
    fn with_content_hash(mut self, hash: ContentHash) -> Self {
        self.content_hash = hash;
        self
    }
    
    fn verify_content_hash(&self) -> bool {
        match self.calculate_content_hash() {
            Ok(hash) => &hash == &self.content_hash,
            Err(_) => false,
        }
    }
}
```

#### Usage Example: Creating and Delegating Capabilities

```rust
/// Create and delegate capabilities
async fn create_capability_example() -> Result<(), CapabilityError> {
    // Initialize components
    let store = Arc::new(InMemoryCapabilityStore::new());
    let crypto = Arc::new(Ed25519Crypto::new());
    let system_key = crypto.generate_key_pair()?;
    let system_identity = Identity::from_public_key(&system_key.public_key);
    
    let manager = CapabilityManager::new(
        store.clone(),
        crypto.clone(),
        Arc::new(system_key),
        system_identity.clone(),
    );
    
    // Create a user
    let user_key = crypto.generate_key_pair()?;
    let user_identity = Identity::from_public_key(&user_key.public_key);
    
    // Issue a capability to the user
    let resource_id = ResourceId::from_parts("account", "user1")?;
    let capability = manager.issue_capability(
        resource_id,
        CapabilityType::Owner,
        user_identity.clone(),
    ).await?;
    
    // User delegates a more restricted capability to an application
    let app_key = crypto.generate_key_pair()?;
    let app_identity = Identity::from_public_key(&app_key.public_key);
    
    // Create a derived capability with constraints
    let derived = Capability::new(
        resource_id.clone(),
        CapabilityType::Write,
    )?.with_constraint(
        CapabilityConstraint::Time(TimeConstraint {
            start: Some(Utc::now()),
            end: Some(Utc::now() + chrono::Duration::days(30)),
        }),
    )?.with_constraint(
        CapabilityConstraint::Operation(OperationConstraint {
            allowed_operations: vec!["update_balance".to_string()],
        }),
    )?;
    
    // Delegate the capability
    let delegation = manager.delegate_capability(
        &capability,
        derived,
        user_identity,
        app_identity.clone(),
        &*user_key,
    ).await?;
    
    // Later, verify the app's capability
    let has_capability = manager.verify_capability(
        &app_identity,
        &resource_id,
        CapabilityType::Write,
    ).await?;
    
    assert!(has_capability);
    
    Ok(())
}
```

### 5.3 Capability Store

Capabilities and their delegations are persisted in a capability store:

```rust
/// Storage for capabilities and delegations
#[async_trait]
pub trait CapabilityStore: Send + Sync + 'static {
    /// Store a capability delegation
    async fn store_delegation(&self, delegation: &CapabilityDelegation) 
        -> Result<(), CapabilityError>;
    
    /// Find delegations for an identity
    async fn find_delegations_for_identity(&self, identity: &Identity) 
        -> Result<Vec<CapabilityDelegation>, CapabilityError>;
    
    /// Find a delegation chain
    async fn find_delegation_chain(&self, delegation_hash: &ContentHash) 
        -> Result<Vec<CapabilityDelegation>, CapabilityError>;
    
    /// Revoke a capability
    async fn revoke_capability(&self, capability_hash: &ContentHash) 
        -> Result<(), CapabilityError>;
    
    /// Check if a capability is revoked
    async fn is_revoked(&self, capability_hash: &ContentHash) 
        -> Result<bool, CapabilityError>;
}

/// In-memory capability store
pub struct InMemoryCapabilityStore {
    /// Stored delegations
    delegations: RwLock<HashMap<ContentHash, CapabilityDelegation>>,
    /// Delegations by identity
    delegations_by_identity: RwLock<HashMap<Identity, Vec<ContentHash>>>,
    /// Revoked capabilities
    revoked_capabilities: RwLock<HashSet<ContentHash>>,
}

impl InMemoryCapabilityStore {
    /// Create a new in-memory capability store
    pub fn new() -> Self {
        Self {
            delegations: RwLock::new(HashMap::new()),
            delegations_by_identity: RwLock::new(HashMap::new()),
            revoked_capabilities: RwLock::new(HashSet::new()),
        }
    }
}

#[async_trait]
impl CapabilityStore for InMemoryCapabilityStore {
    async fn store_delegation(&self, delegation: &CapabilityDelegation) 
        -> Result<(), CapabilityError> {
        let content_hash = delegation.content_hash().clone();
        
        // Store the delegation
        let mut delegations = self.delegations.write()
            .map_err(|_| CapabilityError::LockError)?;
        
        delegations.insert(content_hash.clone(), delegation.clone());
        
        // Update the identity index
        let mut delegations_by_identity = self.delegations_by_identity.write()
            .map_err(|_| CapabilityError::LockError)?;
        
        let delegatee = delegation.delegatee().clone();
        let entry = delegations_by_identity.entry(delegatee).or_insert_with(Vec::new);
        entry.push(content_hash);
        
        Ok(())
    }
    
    async fn find_delegations_for_identity(&self, identity: &Identity) 
        -> Result<Vec<CapabilityDelegation>, CapabilityError> {
        // Get delegation hashes for this identity
        let delegations_by_identity = self.delegations_by_identity.read()
            .map_err(|_| CapabilityError::LockError)?;
        
        let hashes = match delegations_by_identity.get(identity) {
            Some(h) => h.clone(),
            None => return Ok(Vec::new()),
        };
        
        // Get the delegations
        let delegations = self.delegations.read()
            .map_err(|_| CapabilityError::LockError)?;
        
        let mut result = Vec::new();
        for hash in hashes {
            if let Some(delegation) = delegations.get(&hash) {
                result.push(delegation.clone());
            }
        }
        
        Ok(result)
    }
    
    async fn find_delegation_chain(&self, delegation_hash: &ContentHash) 
        -> Result<Vec<CapabilityDelegation>, CapabilityError> {
        let delegations = self.delegations.read()
            .map_err(|_| CapabilityError::LockError)?;
        
        // Start with the target delegation
        let mut chain = Vec::new();
        let mut current_hash = delegation_hash.clone();
        
        // Build the chain
        while let Some(delegation) = delegations.get(&current_hash) {
            chain.push(delegation.clone());
            
            // Move to the source capability
            current_hash = delegation.source().clone();
            
            // Check if we've reached the root
            if current_hash == ContentHash::from_bytes(&[0; 32]) {
                break;
            }
        }
        
        // Reverse the chain to start with the root
        chain.reverse();
        
        Ok(chain)
    }
    
    async fn revoke_capability(&self, capability_hash: &ContentHash) 
        -> Result<(), CapabilityError> {
        let mut revoked = self.revoked_capabilities.write()
            .map_err(|_| CapabilityError::LockError)?;
        
        revoked.insert(capability_hash.clone());
        
        Ok(())
    }
    
    async fn is_revoked(&self, capability_hash: &ContentHash) 
        -> Result<bool, CapabilityError> {
        let revoked = self.revoked_capabilities.read()
            .map_err(|_| CapabilityError::LockError)?;
        
        Ok(revoked.contains(capability_hash))
    }
}
```

### 5.4 Capability Integration with Effects

Capabilities are verified during effect execution to ensure proper authorization:

```rust
/// Execute an effect with capability verification
pub async fn execute_effect_with_capabilities(
    effect: &dyn Effect,
    context: &EffectContext,
    executor: &EffectEngine,
    capability_manager: &CapabilityManager,
) -> Result<EffectOutcome, EffectError> {
    // Get the identity
    let identity = context.identity();
    
    // Verify capabilities for each resource
    for resource in effect.resources() {
        // Determine the required capability type based on the effect
        let capability_type = match effect.effect_type() {
            "read" | "query" => CapabilityType::Read,
            "write" | "update" => CapabilityType::Write,
            "execute" | "invoke" => CapabilityType::Execute,
            "admin" | "delegate" => CapabilityType::Admin,
            "transfer" | "ownership" => CapabilityType::Owner,
            _ => {
                // Default to read for unknown effect types
                CapabilityType::Read
            }
        };
        
        // Verify the capability
        let has_capability = capability_manager.verify_capability(
            &identity,
            &resource,
            capability_type,
        ).await.map_err(|e| EffectError::CapabilityError(e.to_string()))?;
        
        if !has_capability {
            // Missing capability
            return Err(EffectError::MissingCapability);
        }
    }
    
    // Execute the effect
    executor.execute(effect, context).await
}
```

### 5.5 Capability Registry [ADR-032]

The capability registry is responsible for managing capabilities and their relationships:

```rust
/// Capability registry
pub struct CapabilityRegistry {
    /// Capabilities by ID
    capabilities: RwLock<HashMap<ContentHash, Capability>>,
    /// Delegations by ID
    delegations: RwLock<HashMap<ContentHash, CapabilityDelegation>>,
    /// Revoked capabilities
    revoked_capabilities: RwLock<HashSet<ContentHash>>,
}

impl CapabilityRegistry {
    /// Create a new capability registry
    pub fn new() -> Self {
        Self {
            capabilities: RwLock::new(HashMap::new()),
            delegations: RwLock::new(HashMap::new()),
            revoked_capabilities: RwLock::new(HashSet::new()),
        }
    }
    
    /// Add a capability
    pub fn add_capability(&self, capability: Capability) -> Result<(), CapabilityError> {
        let mut capabilities = self.capabilities.write().map_err(|_| CapabilityError::LockError)?;
        let mut delegations = self.delegations.write().map_err(|_| CapabilityError::LockError)?;
        
        // Check if the capability already exists
        if capabilities.contains_key(capability.content_hash()) {
            return Err(CapabilityError::AlreadyExists);
        }
        
        // Add the capability
        capabilities.insert(capability.content_hash().clone(), capability);
        
        // Add the delegation
        delegations.insert(capability.content_hash().clone(), CapabilityDelegation::new(
            self.system_identity.clone(),
            self.system_identity.clone(),
            ContentHash::from_bytes(&[0; 32]), // Root capability
            capability.clone(),
        )?.sign(&*self.signer)?);
        
        Ok(())
    }
    
    /// Find a capability
    pub fn find_capability(&self, capability_hash: &ContentHash) -> Result<Option<Capability>, CapabilityError> {
        let capabilities = self.capabilities.read().map_err(|_| CapabilityError::LockError)?;
        Ok(capabilities.get(capability_hash).cloned())
    }
    
    /// Find a delegation
    pub fn find_delegation(&self, delegation_hash: &ContentHash) -> Result<Option<CapabilityDelegation>, CapabilityError> {
        let delegations = self.delegations.read().map_err(|_| CapabilityError::LockError)?;
        Ok(delegations.get(delegation_hash).cloned())
    }
    
    /// Revoke a capability
    pub fn revoke_capability(&self, capability_hash: &ContentHash) -> Result<(), CapabilityError> {
        let mut revoked = self.revoked_capabilities.write().map_err(|_| CapabilityError::LockError)?;
        revoked.insert(capability_hash.clone());
        Ok(())
    }
    
    /// Check if a capability is revoked
    pub fn is_revoked(&self, capability_hash: &ContentHash) -> Result<bool, CapabilityError> {
        let revoked = self.revoked_capabilities.read().map_err(|_| CapabilityError::LockError)?;
        Ok(revoked.contains(capability_hash))
    }
}
```

### 5.6 Capability Constraints

Constraints can be applied to capabilities to limit their scope:

```rust
/// Time constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConstraint {
    /// Start time
    start: Option<DateTime<Utc>>,
    /// End time
    end: Option<DateTime<Utc>>,
}

impl TimeConstraint {
    /// Check if this constraint is satisfied
    pub fn is_satisfied(&self, now: DateTime<Utc>) -> bool {
        if let Some(start) = self.start {
            if now < start {
                return false;
            }
        }
        
        if let Some(end) = self.end {
            if now > end {
                return false;
            }
        }
        
        true
    }
    
    /// Check if this constraint subsumes another
    pub fn subsumes(&self, other: &TimeConstraint) -> bool {
        // Start time: self.start <= other.start (or other.start is Some and self.start is None)
        let start_subsumes = match (self.start, other.start) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(self_start), Some(other_start)) => self_start <= other_start,
        };
        
        // End time: self.end >= other.end (or other.end is Some and self.end is None)
        let end_subsumes = match (self.end, other.end) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(self_end), Some(other_end)) => self_end >= other_end,
        };
        
        start_subsumes && end_subsumes
    }
}

/// Field constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConstraint {
    /// Field path
    path: String,
    /// Allowed values
    allowed_values: Vec<Value>,
}

impl FieldConstraint {
    /// Check if this constraint is satisfied
    pub fn is_satisfied(&self, resource: &Value) -> bool {
        // Get the field value
        let field_value = self.get_field_value(resource);
        
        // Check if the field value is in the allowed values
        match field_value {
            Some(value) => self.allowed_values.iter().any(|allowed| allowed == value),
            None => false,
        }
    }
    
    /// Get the value of a field
    fn get_field_value<'a>(&self, resource: &'a Value) -> Option<&'a Value> {
        let mut current = resource;
        
        for part in self.path.split('.') {
            match current {
                Value::Object(obj) => {
                    current = obj.get(part)?;
                },
                _ => return None,
            }
        }
        
        Some(current)
    }
    
    /// Check if this constraint subsumes another
    pub fn subsumes(&self, other: &FieldConstraint) -> bool {
        // Paths must be the same
        if self.path != other.path {
            return false;
        }
        
        // All values allowed by the other must be allowed by this one
        other.allowed_values.iter().all(|other_value| {
            self.allowed_values.iter().any(|self_value| self_value == other_value)
        })
    }
}

/// Operation constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationConstraint {
    /// Allowed operations
    allowed_operations: Vec<String>,
}

impl OperationConstraint {
    /// Check if this constraint is satisfied
    pub fn is_satisfied(&self, operation: &str) -> bool {
        self.allowed_operations.iter().any(|allowed| allowed == operation)
    }
    
    /// Check if this constraint subsumes another
    pub fn subsumes(&self, other: &OperationConstraint) -> bool {
        // All operations allowed by the other must be allowed by this one
        other.allowed_operations.iter().all(|other_op| {
            self.allowed_operations.iter().any(|self_op| self_op == other_op)
        })
    }
}

/// Rate limit constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConstraint {
    /// Maximum number of operations
    max_operations: u64,
    /// Time window in seconds
    time_window_seconds: u64,
}

impl RateLimitConstraint {
    /// Check if this constraint is satisfied
    pub fn is_satisfied(&self, operation_count: u64, window_seconds: u64) -> bool {
        if window_seconds > self.time_window_seconds {
            return false;
        }
        
        let rate = (operation_count as f64) / (window_seconds as f64);
        let max_rate = (self.max_operations as f64) / (self.time_window_seconds as f64);
        
        rate <= max_rate
    }
    
    /// Check if this constraint subsumes another
    pub fn subsumes(&self, other: &RateLimitConstraint) -> bool {
        // This constraint's rate must be >= other's rate
        let self_rate = (self.max_operations as f64) / (self.time_window_seconds as f64);
        let other_rate = (other.max_operations as f64) / (other.time_window_seconds as f64);
        
        self_rate >= other_rate
    }
}

/// Custom constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomConstraint {
    /// Constraint type
    constraint_type: String,
    /// Constraint data
    data: Value,
}

impl CustomConstraint {
    /// Check if this constraint subsumes another
    pub fn subsumes(&self, other: &CustomConstraint) -> bool {
        // Custom constraints must be handled by specific verifiers
        self.constraint_type == other.constraint_type
    }
}

impl CapabilityConstraint {
    /// Check if this constraint subsumes another
    pub fn subsumes(&self, other: &CapabilityConstraint) -> bool {
        match (self, other) {
            (CapabilityConstraint::Time(self_time), CapabilityConstraint::Time(other_time)) => {
                self_time.subsumes(other_time)
            },
            (CapabilityConstraint::Field(self_field), CapabilityConstraint::Field(other_field)) => {
                self_field.subsumes(other_field)
            },
            (CapabilityConstraint::Operation(self_op), CapabilityConstraint::Operation(other_op)) => {
                self_op.subsumes(other_op)
            },
            (CapabilityConstraint::Custom(self_custom), CapabilityConstraint::Custom(other_custom)) => {
                self_custom.subsumes(other_custom)
            },
            _ => false, // Different constraint types never subsume each other
        }
    }
}
```

### 5.7 Capability System Interaction Diagram

```
┌─────────────────────────┐     ┌─────────────────────────┐
│                         │     │                         │
│      Actor System       │     │     Resource System     │
│                         │     │                         │
└───────────┬─────────────┘     └────────────┬────────────┘
            │                                │
            │                                │
            │     ┌────────────────────┐     │
            └────►│                    │◄────┘
                  │ Capability System  │
                  │                    │
                  └─────┬──────┬───────┘
                        │      │
                        │      │
            ┌───────────┘      └──────────┐
            │                             │
┌───────────▼────────────┐    ┌───────────▼────────────┐
│                        │    │                        │
│  Capability Storage    │    │  Cryptographic         │
│                        │    │  Verification          │
│                        │    │                        │
└────────────────────────┘    └────────────────────────┘
```

The Capability System serves as the central authorization layer:

1. **Identity Association**: Actors present their identity to access resources
2. **Capability Issuance**: The system issues capabilities to actors for specific resources
3. **Delegation Chains**: Actors can delegate capabilities to other actors, forming verifiable chains
4. **Authorization Verification**: When an actor attempts to access a resource, the capability system verifies that the actor has a valid capability

### 5.8 Capability-based Security Model

The capability-based security model enables secure, fine-grained access control:

1. **No Ambient Authority**: All authority is explicitly represented by capabilities
2. **Least Privilege**: Effects only receive the capabilities they need to perform their function
3. **Capability Attenuation**: Capabilities can be restricted when delegated to limit their scope
4. **Revocation**: Capabilities can be revoked without affecting other authorization chains

```rust
/// Example capability-based authorization flow
async fn capability_based_authorization(
    effect: &dyn Effect,
    context: &EffectContext,
    capability_registry: &CapabilityRegistry,
) -> Result<bool, CapabilityError> {
    // Get the agent identity
    let identity = context.identity();
    
    // Check each resource the effect needs to access
    for resource in effect.resources() {
        // Determine the required capability type based on the effect
        let capability_type = match effect.effect_type() {
            "read" => CapabilityType::Read,
            "write" => CapabilityType::Write,
            "execute" => CapabilityType::Execute,
            "admin" => CapabilityType::Admin,
            "transfer" => CapabilityType::Owner,
            _ => return Err(CapabilityError::UnknownEffectType),
        };
        
        // Verify the capability
        let has_capability = capability_registry.verify_capability(
            &identity,
            &resource,
            capability_type,
        ).await?;
        
        if !has_capability {
            // Authorization failed
            return Ok(false);
        }
    }
    
    // All capabilities verified
    Ok(true)
}
```

## 6. Agent System [ADR-005, ADR-032, ADR-032]

The Agent System represents the evolution of the previous Actor System into a fully integrated, resource-based model. As described in ADR-032 and ADR-003, agents are content-addressed specialized resource types that hold capabilities and perform operations:

### 6.1 Agent Definition [ADR-032]

An agent is a specialized resource that holds capabilities and performs operations:

```rust
/// Agent definition - implemented as a specialized resource type
pub struct Agent {
    /// Base resource implementation with content addressing
    resource: Resource,
    /// Identity
    identity: Identity,
    /// Capabilities
    capabilities: Arc<CapabilityRegistry>,
    /// Obligation manager
    obligation_manager: Arc<ObligationManager>,
    /// Message queue
    message_queue: Arc<MessageQueue>,
}

impl ContentAddressed for Agent {
    fn content_id(&self) -> ContentId {
        // Derive content ID from fields
        self.resource.content_id()
    }
}

impl ResourceAccessor for Agent {
    // Implementation connecting agent to the resource system
}

impl Agent {
    /// Create a new agent
    pub fn new(
        identity: Identity,
        capabilities: Arc<CapabilityRegistry>,
        obligation_manager: Arc<ObligationManager>,
        message_queue: Arc<MessageQueue>,
    ) -> Self {
        // Create the underlying resource
        let resource = Resource::new_with_type("Agent");
        
        Self {
            resource,
            identity,
            capabilities,
            obligation_manager,
            message_queue,
        }
    }
    
    /// Get the identity
    pub fn identity(&self) -> &Identity {
        &self.identity
    }
    
    /// Get the capabilities
    pub fn capabilities(&self) -> &Arc<CapabilityRegistry> {
        &self.capabilities
    }
    
    /// Get the obligation manager
    pub fn obligation_manager(&self) -> &Arc<ObligationManager> {
        &self.obligation_manager
    }
    
    /// Get the message queue
    pub fn message_queue(&self) -> &Arc<MessageQueue> {
        &self.message_queue
    }
}
```

### 6.2 Agent Profiles [ADR-032]

Agent profiles are specialized resources that define rules and behaviors associated with agents:

```rust
/// Agent profile as a specialized resource
pub struct AgentProfile {
    /// Base resource implementation with content addressing
    resource: Resource,
    /// Capabilities
    capabilities: Vec<Capability>,
    /// Obligation manager
    obligation_manager: Arc<ObligationManager>,
    /// Message queue
    message_queue: Arc<MessageQueue>,
}

impl ContentAddressed for AgentProfile {
    fn content_id(&self) -> ContentId {
        // Derive content ID from fields
        self.resource.content_id()
    }
}

impl ResourceAccessor for AgentProfile {
    // Implementation connecting agent profile to the resource system
}

impl AgentProfile {
    /// Create a new agent profile
    pub fn new(
        capabilities: Vec<Capability>,
        obligation_manager: Arc<ObligationManager>,
        message_queue: Arc<MessageQueue>,
    ) -> Self {
        // Create the underlying resource
        let resource = Resource::new_with_type("AgentProfile");
        
        Self {
            resource,
            capabilities,
            obligation_manager,
            message_queue,
        }
    }
    
    /// Get the capabilities
    pub fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }
    
    /// Get the obligation manager
    pub fn obligation_manager(&self) -> &Arc<ObligationManager> {
        &self.obligation_manager
    }
    
    /// Get the message queue
    pub fn message_queue(&self) -> &Arc<MessageQueue> {
        &self.message_queue
    }
}
```

### 6.3 Service Status [ADR-032]

Service status is a specialized resource that defines the rules and behaviors associated with agents offering services:

```rust
/// Service status as a specialized resource
pub struct ServiceStatus {
    /// Base resource implementation with content addressing
    resource: Resource,
    /// Capabilities
    capabilities: Vec<Capability>,
    /// Obligation manager
    obligation_manager: Arc<ObligationManager>,
    /// Message queue
    message_queue: Arc<MessageQueue>,
}

impl ContentAddressed for ServiceStatus {
    fn content_id(&self) -> ContentId {
        // Derive content ID from fields
        self.resource.content_id()
    }
}

impl ResourceAccessor for ServiceStatus {
    // Implementation connecting service status to the resource system
}

impl ServiceStatus {
    /// Create a new service status
    pub fn new(
        capabilities: Vec<Capability>,
        obligation_manager: Arc<ObligationManager>,
        message_queue: Arc<MessageQueue>,
    ) -> Self {
        // Create the underlying resource
        let resource = Resource::new_with_type("ServiceStatus");
        
        Self {
            resource,
            capabilities,
            obligation_manager,
            message_queue,
        }
    }
    
    /// Get the capabilities
    pub fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }
    
    /// Get the obligation manager
    pub fn obligation_manager(&self) -> &Arc<ObligationManager> {
        &self.obligation_manager
    }
    
    /// Get the message queue
    pub fn message_queue(&self) -> &Arc<MessageQueue> {
        &self.message_queue
    }
}
```

### 6.4 Obligation Manager [ADR-032]

The obligation manager is a component responsible for managing usage-based expectations on capabilities:

```rust
/// Obligation manager
pub struct ObligationManager {
    /// Usage expectations
    expectations: RwLock<HashMap<ResourceId, u64>>,
    /// Usage history
    history: RwLock<HashMap<ResourceId, Vec<UsageEvent>>>,
}

impl ObligationManager {
    /// Create a new obligation manager
    pub fn new() -> Self {
        Self {
            expectations: RwLock::new(HashMap::new()),
            history: RwLock::new(HashMap::new()),
        }
    }
    
    /// Add an expectation
    pub fn add_expectation(&self, resource: ResourceId, quantity: u64) -> Result<(), ObligationError> {
        let mut expectations = self.expectations.write().map_err(|_| ObligationError::LockError)?;
        let mut history = self.history.write().map_err(|_| ObligationError::LockError)?;
        
        // Check if the resource already has an expectation
        if let Some(current_expectation) = expectations.get(&resource) {
            if *current_expectation >= quantity {
                return Err(ObligationError::AlreadyMet);
            }
        }
        
        // Add the new expectation
        expectations.insert(resource, quantity);
        
        // Add a usage event to the history
        history.entry(resource).or_insert_with(Vec::new).push(UsageEvent::new());
        
        Ok(())
    }
    
    /// Check if an expectation is met
    pub fn is_expectation_met(&self, resource: &ResourceId) -> Result<bool, ObligationError> {
        let expectations = self.expectations.read().map_err(|_| ObligationError::LockError)?;
        let history = self.history.read().map_err(|_| ObligationError::LockError)?;
        
        // Check if the resource has an expectation
        if let Some(expectation) = expectations.get(resource) {
            // Check if the usage history meets the expectation
            let usage_history = history.get(resource).unwrap_or(&[]);
            let usage_count = usage_history.len() as u64;
            if usage_count >= *expectation {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
}
```

### 6.5 Messaging [ADR-032]

Messaging is defined as a resource-based system for asynchronous communication between agents:

```rust
/// Message as a specialized resource
pub struct Message {
    /// Base resource implementation with content addressing
    resource: Resource,
    /// Message content
    content: Vec<u8>,
    /// Sender
    sender: ResourceId,
    /// Recipient
    recipient: ResourceId,
    /// Message type
    message_type: String,
}

impl ContentAddressed for Message {
    fn content_id(&self) -> ContentId {
        // Derive content ID from fields
        self.resource.content_id()
    }
}

impl ResourceAccessor for Message {
    // Implementation connecting message to the resource system
}

/// Message queue
pub struct MessageQueue {
    /// Messages
    messages: RwLock<Vec<ResourceId>>, // Now stores ResourceIds of Message resources
}

impl MessageQueue {
    /// Create a new message queue
    pub fn new() -> Self {
        Self {
            messages: RwLock::new(Vec::new()),
        }
    }
    
    /// Send a message
    pub fn send(&self, message: Message) -> Result<(), MessageError> {
        let message_id = message.content_id();
        let mut messages = self.messages.write().map_err(|_| MessageError::LockError)?;
        messages.push(message_id);
        Ok(())
    }
    
    /// Receive a message
    pub fn receive(&self) -> Result<Option<ResourceId>, MessageError> {
        let mut messages = self.messages.write().map_err(|_| MessageError::LockError)?;
        Ok(messages.pop())
    }
}
```

### 6.6 Agent System Diagram

The Agent System integrates fully with the Resource System:

```
┌────────────────────────────────────────────────────────────────────────┐
│                       Agent System                                     │
│                                                                        │
│   ┌────────────┐        ┌────────────┐        ┌────────────────────┐   │
│   │ Agent      │◄──────►│ Messaging  │◄──────►│  Service Status    │   │
│   │ (Resource) │        │ (Resource) │        │  (Resource)        │   │
│   └────┬───────┘        └────────────┘        └────────────────────┘   │
│        │                                                               │
│        │                    ┌───────────────┐    ┌────────────────┐    │
│        └───────────────────►│ Capabilities  │◄───┤ ObligationMgr  │    │
│                             │               │    │                │    │
│                             └───────┬───────┘    └────────────────┘    │
│                                     │                                  │
│                                     ▼                                  │
│                             ┌───────────────┐                          │
│                             │   Resource    │                          │
│                             │    System     │                          │
│                             └───────────────┘                          │
└────────────────────────────────────────────────────────────────────────┘
```

## 7. Operation System [ADR-032]

The operation system is responsible for defining and executing operations:

### 7.1 Operation Model

Operations are defined as requests to perform effects with explicit authorization:

```rust
/// Operation for cross-domain asset transfer
#[derive(Debug, Clone)]
pub struct CrossDomainTransferOperation {
    /// Source domain
    source_domain: DomainId,
    /// Target domain
    target_domain: DomainId,
    /// Source account
    source_account: ResourceId,
    /// Target account
    target_account: ResourceId,
    /// Asset to transfer
    asset: Asset,
    /// Amount to transfer
    amount: u64,
    /// Authorization
    authorization: Capability,
}

impl Operation for CrossDomainTransferOperation {
    type Effect = CrossDomainTransferEffect;
    
    fn id(&self) -> &OperationId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "cross_domain_transfer"
    }
    
    fn domains(&self) -> Vec<DomainId> {
        vec![self.source_domain.clone(), self.target_domain.clone()]
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        vec![self.source_account.clone(), self.target_account.clone()]
    }
    
    fn authorization(&self) -> &Capability {
        &self.authorization
    }
    
    fn validate(&self, context: &EffectContext) -> Result<(), OperationError> {
        // Validate the authorization
        if !context.has_capability(&self.authorization) {
            return Err(OperationError::MissingCapability);
        }
        
        // Validate the amount
        if self.amount == 0 {
            return Err(OperationError::ZeroAmount);
        }
        
        Ok(())
    }
    
    fn execute(&self, context: &EffectContext) -> Result<Self::Effect, OperationError> {
        // Create the effect
        let effect = CrossDomainTransferEffect {
            id: EffectId::new(),
            source_domain: self.source_domain.clone(),
            target_domain: self.target_domain.clone(),
            source_account: self.source_account.clone(),
            target_account: self.target_account.clone(),
            asset: self.asset.clone(),
            amount: self.amount,
            fact_snapshot: None,
        };
        
        Ok(effect)
    }
}
```

### 7.2 Authorization

Authorization is defined as the rules and behaviors associated with granting explicit capabilities to agents:

```rust
/// Authorization for operations
pub struct Authorization {
    /// Capabilities
    capabilities: Vec<Capability>,
    /// Required roles
    required_roles: Vec<Role>,
    /// Verification context
    verification_context: VerificationContext,
}

impl Authorization {
    /// Create a new authorization
    pub fn new(
        capabilities: Vec<Capability>,
        required_roles: Vec<Role>,
        verification_context: VerificationContext,
    ) -> Self {
        Self {
            capabilities,
            required_roles,
            verification_context,
        }
    }
    
    /// Verify authorization
    pub fn verify(&self, operation: &Operation) -> Result<bool, AuthorizationError> {
        // Verify capabilities
        for capability in &self.capabilities {
            if !self.verify_capability(capability, operation)? {
                return Ok(false);
            }
        }
        
        // Verify roles
        for role in &self.required_roles {
            if !self.verify_role(role, operation)? {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Verify a capability
    fn verify_capability(&self, capability: &Capability, operation: &Operation) -> Result<bool, AuthorizationError> {
        // Check if the capability applies to the operation
        if !capability.matches_operation(operation)? {
            return Ok(false);
        }
        
        // Verify the capability
        self.verification_context.verify_capability(capability)
    }
    
    /// Verify a role
    fn verify_role(&self, role: &Role, operation: &Operation) -> Result<bool, AuthorizationError> {
        // Check if the agent has the required role
        self.verification_context.verify_role(role)
    }
}
```

### 7.3 Operation Execution

Operation execution is defined as the rules and behaviors associated with executing operations:

```rust
/// Operation executor
pub struct OperationExecutor {
    /// Effect interpreter
    interpreter: Arc<EffectInterpreter>,
    /// Capability manager
    capability_manager: Arc<CapabilityManager>,
}

impl OperationExecutor {
    /// Create a new operation executor
    pub fn new(
        interpreter: Arc<EffectInterpreter>,
        capability_manager: Arc<CapabilityManager>,
    ) -> Self {
        Self {
            interpreter,
            capability_manager,
        }
    }
    
    /// Execute an operation
    pub async fn execute(&self, operation: &dyn Operation, context: &EffectContext) 
        -> Result<EffectOutcome, OperationError> {
        // Validate the operation
        operation.validate(context)?;
        
        // Execute the operation
        let effect = operation.execute(context)?;
        
        // Execute the effect
        self.interpreter.execute(&effect, context).await
    }
}
```

### 7.4 Operation Composition

Operation composition is defined as the rules and behaviors associated with composing operations:

```rust
/// Operation composer
pub struct OperationComposer {
    /// Operation registry
    registry: Arc<OperationRegistry>,
}

impl OperationComposer {
    /// Create a new operation composer
    pub fn new(registry: Arc<OperationRegistry>) -> Self {
        Self { registry }
    }
    
    /// Compose operations
    pub fn compose(&self, operations: Vec<&dyn Operation>) -> Result<Operation, OperationError> {
        // Validate all operations
        for operation in &operations {
            operation.validate(context)?;
        }
        
        // Create a new operation
        let operation = Operation::new(
            EffectId::new(),
            "composed_operation",
            EffectType::from_str("composed_operation")?,
            operations.into_iter().map(|operation| operation.id().clone()).collect(),
            operations.into_iter().map(|operation| operation.resources().clone()).collect(),
            operations.into_iter().map(|operation| operation.authorization().clone()).collect(),
        )?;
        
        Ok(operation)
    }
}
```

### 7.5 Operation Interaction Diagram

```
┌──────────────────────────┐     ┌──────────────────────────┐
│                          │     │                          │
│     Effect System        │     │    Capability System     │
│                          │     │                          │
└───────────┬──────────────┘     └───────────┬──────────────┘
            │                                │
            │                                │
            │    ┌─────────────────────┐     │
            └───►│                     │◄────┘
                 │   Resource System   │
                 │                     │
                 └──────┬──────┬───────┘
                        │      │
                        │      │
            ┌───────────┘      └──────────┐
            │                             │
┌───────────▼────────────┐    ┌───────────▼────────────┐
│                        │    │                        │
│  Domain Adapter A      │    │  Domain Adapter B      │
│  (Blockchain)          │    │  (Database)            │
│                        │    │                        │
└────────────────────────┘    └────────────────────────┘
```

The Operation System serves as the entry point for all operations:

1. **Operation Definition**: Operations are defined as requests to perform effects with explicit authorization
2. **Operation Validation**: Operations undergo validation to ensure they meet the requirements of the system
3. **Operation Execution**: Operations are executed through the Effect Interpreter
4. **Operation Composition**: Operations can be composed to form more complex workflows

## Conclusion

This specification document provides a comprehensive reference for the Causality system. It integrates the architectural decisions documented in the ADRs and serves as the authoritative source for all crate implementations.

For detailed implementation guidelines, refer to the individual ADRs referenced throughout this document.

## Codebase Structure

The Causality codebase is organized into a modular structure of Rust crates, each with a specific responsibility within the system. This section provides an overview of the major crates and their relationships.

### Core Crates

- **causality-core**: The foundation library containing all core data structures, traits, and interfaces used throughout the system.
  - **causality-core/src/content**: Content addressing implementation
  - **causality-core/src/time**: Time system implementation
  - **causality-core/src/effect**: Effect system implementation
  - **causality-core/src/resource**: Resource system implementation
  - **causality-core/src/capability**: Capability system implementation
  - **causality-core/src/role**: Role-Based Resource System implementation
  - **causality-core/src/domain**: Domain and validator committee implementation

- **causality-domain**: Implementations of domain adapters for various blockchains and data stores.
  - **causality-domain/src/ethereum**: Ethereum domain adapter
  - **causality-domain/src/cosmwasm**: CosmWasm domain adapter
  - **causality-domain/src/local**: Local storage adapter

- **causality-program**: Program composition and execution.
  - **causality-program/src/account**: Account program implementation
  - **causality-program/src/execution**: Program execution environment
  - **causality-program/src/composition**: Program composition patterns
  - **causality-program/src/state**: State management primitives

### Utility Crates

- **causality-common**: Shared utilities and common functionality.
  - **causality-common/src/error**: Error handling patterns
  - **causality-common/src/logging**: Logging infrastructure
  - **causality-common/src/config**: Configuration management
  - **causality-common/src/testing**: Testing utilities

- **causality-crypto**: Cryptographic primitives and implementations.
  - **causality-crypto/src/hash**: Content hashing algorithms
  - **causality-crypto/src/signature**: Signature schemes
  - **causality-crypto/src/key**: Key management
  - **causality-crypto/src/zkp**: Zero-knowledge proof primitives

### Frontend Crates

- **causality-cli**: Command-line interface for interacting with Causality.
  - **causality-cli/src/commands**: CLI commands
  - **causality-cli/src/interactive**: Interactive shell
  - **causality-cli/src/config**: CLI configuration

- **causality-api**: HTTP/WebSocket API for interacting with Causality.
  - **causality-api/src/routes**: API routes
  - **causality-api/src/middleware**: API middleware
  - **causality-api/src/client**: Client library

### Integration Crates

- **causality-integration-tests**: End-to-end and integration tests.
- **causality-benchmarks**: Performance benchmarking suite.
- **causality-examples**: Example applications and use cases.

### Dependencies and Build System

The Causality system uses:

- **Cargo**: Rust package manager for build and dependency management
- **Nix**: Reproducible build environment
- **GitHub Actions**: CI/CD pipeline

### Development Guidelines

Development of new components should adhere to these guidelines:

1. **Modular Design**: Each component should have a clear responsibility
2. **Stable Interfaces**: Public interfaces should be stable and well-documented
3. **Comprehensive Documentation**: All public APIs must be documented
4. **Test Coverage**: All core functionality must have tests
5. **Error Handling**: Errors should be properly propagated and documented

