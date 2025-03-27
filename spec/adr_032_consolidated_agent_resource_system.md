# ADR-032: Agent-Based Resource System

## Status

| Status    | Date       | Revision |
|:----------|:-----------|:---------|
| Accepted  | 2023-03-27 | 1.0      |

## Context

The Causality system needs a cohesive architectural model that clearly separates concerns, ensures consistency in resource management, and provides a unified framework for authorization, execution, and state management. While previous architecture efforts introduced concepts in isolation, this document presents a complete, integrated approach to the system architecture.

The concept of "actors" (from ADR-005) has been refined to better align with our content-addressed resource system. Through architectural exploration, we've determined that these roles are fundamentally resource types with specific capabilities rather than independent computational units in the Actor Model sense.

## Decision

We will implement a comprehensive agent-based resource system with the following key components:

1. **Resources**: Represent domain entities and stateful objects
2. **Capabilities**: Grant precise, contextual authority to interact with resources
3. **Effects**: Abstract, composable actions that can change system state
4. **Operations**: Requests to perform effects with authorization
5. **Agents**: Entities that hold capabilities and perform operations (specialized resource types)
6. **Service Status**: Signals that agents are actively offering services
7. **Obligation Manager**: Enforces usage-based expectations on capabilities
8. **Messaging**: Enables asynchronous interaction between agents
9. **Fact System**: Tracks temporal and logical dependencies between actions

### Unified System Component Model

| **Component**           | **Description**                                                                 | **Modeled As**                          | **Governed By / Consumes**           |
|-------------------------|----------------------------------------------------------------------------------|------------------------------------------|--------------------------------------|
| **Resource**            | Stateful object with a lifecycle and metadata                                   | `Resource`                               | `ResourceLogic`, `Effect`            |
| **Resource Logic**      | Defines valid transitions and behaviors per resource type                       | Trait or module per type                 | Invoked by `Effect` execution        |
| **Capability**          | Token of authority to perform rights on a target resource                       | `Capability`                             | `CapabilityRegistry`                 |
| **Capability Registry** | Issues, validates, revokes, and delegates capabilities                         | `CapabilityRegistry`                     | Used by `Authorization`, `ObligationManager` |
| **Capability Constraint**| Defines time, usage, or state restrictions on capability usage                  | Struct in `Capability`                   | Validated during operation           |
| **Operation**           | Request to perform one or more effects using presented capabilities             | `Operation`                              | `Authorization`, `Effect`            |
| **Effect**              | Abstract, executable side effect                                                | `Effect` trait                           | Executed by `Interpreter`            |
| **Interpreter**         | Runtime for executing effects with effect handlers and context                 | `Interpreter`                            | Uses `EffectHandler`, `ExecutionContext` |
| **Authorization**       | Proof that the agent has valid capabilities to execute an operation            | `Authorization`                          | Validated before operation execution |
| **Execution Context**   | Encapsulates environment, clock, domain, and metadata                          | `ExecutionContext`                       | Passed to `Effect`, `Validator`      |
| **Fact System**         | Tracks logical and temporal relationships between effects                      | `FactSnapshot`, `FactDependency`         | Used in `Effect` validation          |
| **Service Status**      | Declares that an agent is offering a capability-backed service                 | `Resource::ServiceStatus`                | Governed by service-advertising effects |
| **Obligation Manager**  | Monitors usage and enforces duty-based expiration policies                     | `ObligationManager`                      | Uses logs, revokes via registry      |
| **Message**             | Asynchronous, secure, structured communication between agents                  | `Resource::Message`                      | Sent via message-related capabilities |
| **Agent Profile**       | Identity and metadata container for agents in the system                       | `Resource::AgentProfile`                 | Holds capabilities, service state    |
| **Capability Bundle**   | Reusable template for issuing predefined capability sets                       | `CapabilityBundle`                       | Used to instantiate agents or roles  |

### Specialized Agent Types

Building upon our architectural foundation, we define the following specialized agent types:

1. **User Agent**: Represents human users of the system
2. **Committee Agent**: Represents a group of validators materializing chain state
3. **Operator Agent**: Represents automated system operators

### Component Interaction Model

The system revolves around the following key interactions:

1. **Agents** are modeled as specialized resources and hold **capabilities** that grant them rights over other resources.
2. An agent submits an **operation**, which includes:
   - One or more **effects** representing the abstract actions to perform
   - An **authorization** proof using held capabilities
3. The **interpreter** validates and executes effects, consulting:
   - **Resource logic** for domain-specific behaviors
   - The **fact system** to ensure temporal constraints are upheld
   - The **execution context** for runtime conditions (e.g. domain, time)
4. **Service Status** declares that an agent is online and ready to offer specific services (backed by capabilities)
5. **Obligation Manager** enforces expectations that capabilities are actively used or delegated
6. **Messages** are used for capability negotiation, cross-agent coordination, or service announcements

### Separation of Concerns

| **Subsystem**            | **Primary Responsibility**                                           | **Collaborates With**                                 |
|--------------------------|-----------------------------------------------------------------------|--------------------------------------------------------|
| **Authorization**        | Confirms that capability proofs are valid for operations              | `CapabilityRegistry`, `Operation`, `Effect`            |
| **Execution**            | Executes abstract effects and manages state transitions               | `Effect`, `Interpreter`, `ResourceLogic`               |
| **Access Control**       | Issues, tracks, delegates, and revokes capabilities                   | `CapabilityRegistry`, `CapabilityConstraint`           |
| **Service Management**   | Declares available services from capable agents                       | `Service Status`, `AgentProfile`                       |
| **Duty Enforcement**     | Ensures capabilities are actively used                               | `ObligationManager`, `AuditLog`, `CapabilityRegistry`  |
| **Communication**        | Manages agent-to-agent messaging and coordination                     | `Message`, `Authorization`, `Effect`                   |
| **Temporal Validation**  | Ensures causal ordering and dependency tracking                       | `FactSystem`, `Effect`, `ExecutionContext`             |
| **Agent Identity**       | Tracks ownership, metadata, and service state of agents               | `AgentProfile`, `Service Status`, `CapabilityBundle`   |

### Agent Resource Model

Agents are implemented as specialized resources, allowing them to be managed consistently with other resources while providing specific functionality for initiating operations and holding capabilities.

```rust
/// An agent in the system
pub struct Agent {
    /// Agent's resource implementation
    resource: Box<dyn Resource>,
    
    /// Agent's identity
    identity: AgentId,
    
    /// Agent's capabilities
    capabilities: Vec<Capability>,
    
    /// Agent's current state
    state: AgentState,
    
    /// Agent's relationships with other resources
    relationships: Vec<ResourceRelationship>,
}
```

### Agent State Transitions

Agents follow well-defined state transitions:

```rust
/// State transitions for agents
pub enum AgentState {
    /// Agent is active and can perform operations
    Active,
    
    /// Agent is inactive and cannot perform operations
    Inactive,
    
    /// Agent is suspended and cannot perform operations
    Suspended {
        /// Reason for suspension
        reason: String,
        
        /// Timestamp when suspension occurred
        timestamp: TimeStamp,
    },
}
```

### Agent Types

Agents can be of the following types:

```rust
pub enum AgentType {
    /// Human user of the system
    User,
    
    /// Multi-agent decision-making body
    Committee,
    
    /// Automated system operator
    Operator,
}
```

### Agent Capabilities

Agents hold capabilities that authorize them to perform operations on resources:

```rust
impl Agent {
    /// Add a capability to the agent
    pub fn add_capability(&mut self, capability: Capability) -> Result<(), CapabilityError>;
    
    /// Remove a capability from the agent
    pub fn remove_capability(&mut self, capability_id: &CapabilityId) -> Result<(), CapabilityError>;
    
    /// Check if the agent has a specific capability
    pub fn has_capability(&self, capability_type: CapabilityType, resource_id: &ResourceId) -> bool;
    
    /// Get all capabilities of the agent
    pub fn capabilities(&self) -> &[Capability];
}
```

### Agent Operations

Agents can execute operations on resources:

```rust
impl Agent {
    /// Execute an operation
    pub fn execute(&self, operation: Operation, context: OperationContext) 
        -> Result<OperationResult, OperationError>;
    
    /// Create an operation builder for this agent
    pub fn create_operation(&self) -> OperationBuilder;
}
```

## Crate-Level Architecture

The effect system is integrated directly into `causality-core` to provide a unified approach to system design:

- `causality-core` defines both **the `Operation` type** and the **`Effect` trait**, creating a cohesive model for execution.
- `Effect` is implemented as a module within `causality-core`, ensuring tight integration with resources and capabilities.
- The `effect` module within `causality-core` provides the `Effect` trait, base effect types, and the `Interpreter` that can validate and execute effects.

The system is divided into the following Rust crates:

| **Crate Name**            | **Responsibility**                                                                 | **Depends On**                                       |
|---------------------------|------------------------------------------------------------------------------------|------------------------------------------------------|
| `causality-types`         | Shared types and interfaces: IDs, timestamps, trait definitions                   | —                                                    |
| `causality-core`          | Resources, capabilities, operations, effects, interpreter                         | `causality-types`                                    |
| `causality-agent`         | Agent profiles, service status, obligation manager, capability bundles            | `causality-core`                                     |
| `causality-engine`        | Top-level orchestration: effect validation, operation execution, routing          | All of the above                                     |

## Three-Layer Effect Architecture

The effect system follows a three-layer architecture:

1. **Algebraic Effect Layer**
   - Defines the core algebraic effect abstractions
   - Implementation of effect handlers and continuations
   - Type-safe effect composition

2. **Effect Constraints Layer**
   - Defines constraints on effects based on capabilities
   - Links effect execution to the capability system
   - Provides effect authorization and audit

3. **Domain Implementation Layer**
   - Domain-specific effect implementations
   - Integration with external systems
   - Resource-specific effect handlers

## Consequences

### Positive

- **Unified Architecture**: All components are integrated into a cohesive system with clear interactions
- **Content Addressing**: All entities use content addressing, eliminating UUID dependencies
- **Simplified Mental Model**: Developers only need to understand one resource-based system
- **Consistent Permissions**: Uses the unified capability system for all entity types
- **Clear State Machine**: Provides well-defined state transitions for all agent resources
- **Type-Safe Effects**: The effect system provides type-safe, composable operations
- **Reduced Crate Dependencies**: Integration of effects into `causality-core` reduces dependencies
- **Unified Type System**: Resources, capabilities, and effects share a consistent type system
- **Domain Clarity**: Provides a more accurate model of how domains and validator committees work

### Negative

- **Migration Effort**: Significant effort required to migrate from the current actor system
- **Learning Curve**: Teams will need to learn the new unified architecture
- **Implementation Complexity**: Short-term complexity during transition
- **Lost Separation**: Some benefits of strict isolation are lost, though these are less relevant

## Implementation Plan

The implementation follows these phases:

1. **Define Core Types** - Create the core interfaces and traits
2. **Implement Base Components** - Implement the resource, capability, and effect systems
3. **Develop Agent System** - Implement specialized agent types and their interactions
4. **Integration** - Integrate all components into a unified system
5. **Testing & Optimization** - Create comprehensive tests and optimize performance

## Privacy Considerations

The Agent-Based Resource System includes comprehensive privacy support:

1. **Field-Level Encryption** - Sensitive data can be encrypted at the field level
2. **Zero-Knowledge Operations** - Agents can execute privacy-preserving operations
3. **Capability-Based Authorization** - Access control without revealing underlying resources
4. **Cross-Domain Privacy** - Privacy maintained across domain boundaries

## References

- ADR-005: Actor Specification
- ADR-030: Resource Accessor Pattern 