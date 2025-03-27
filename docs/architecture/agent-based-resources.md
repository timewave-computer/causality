# Agent-Based Resource System

*This document is derived from [ADR-032](../../../spec/adr_032_consolidated_agent_resource_system.md) and the [System Contract](../../../spec/system_contract.md).*

*Last updated: 2023-08-15*

## Overview

The Agent-Based Resource System is a core architectural component that implements entities with specific capabilities as specialized resources within the Causality system. It unifies the architecture around the Resource System to provide a consistent model for identity, access control, and entity management.

## Core Concepts

### Agents as Specialized Resources

In Causality, all agent-based entities are implemented as specialized resource types with specific capabilities. The Resource System provides mechanisms for content-addressed resource identification, capability-based access control, and state management, which all agents leverage.

### Agent Types

The Agent-Based Resource System defines three primary agent types:

1. **User**: End users who interact with the system and own programs
2. **Operator**: System administrators who maintain the infrastructure and execute programs
3. **Committee**: Represents a validator group for a blockchain or data source 
   - Committees observe external facts, sign observation proofs, and validate external messages
   - Each committee is associated with a specific domain

### Domain & Agent Interaction Model

The domain-agent interaction model works as follows:

1. Each domain is associated with a committee of agents that observe and validate its state
2. Domain publication is implemented as transaction submission through the committee
3. Domain subscription is implemented as chain observation through the committee
4. Committee agents are responsible for verifying cross-domain content hashes

This model reflects the reality of blockchain-based systems where validators maintain definitive state while enabling cross-domain verification.

## Architecture

### Agent Struct Definition

The Agent system is built around the `Agent` resource type:

```rust
/// Agent structure representing a specialized resource
struct Agent {
    // Base resource implementation
    resource: Resource,
    // Identity information
    identity: Identity,
    // Capabilities that define what this agent can do
    capabilities: Vec<Capability>,
    // State information
    state: AgentState,
    // Relationship to other agents and resources
    relationships: Vec<ResourceRelationship>,
}

/// State information for an agent
enum AgentState {
    Created,
    Initialized,
    Active,
    Suspended { reason: String },
    Upgraded { previous_version: ContentHash },
    Terminated { reason: String },
}

/// Relationship between an agent and another resource
struct ResourceRelationship {
    relationship_type: RelationshipType,
    target_resource: ResourceId,
    capabilities: Vec<Capability>,
    metadata: HashMap<String, Value>,
}

enum RelationshipType {
    Owns,
    Parent,
    Child,
    Peer,
    Delegate,
    DependsOn,
    Custom(String),
}
```

All agents are content-addressed resources, having a unique identifier derived from their content:

```rust
impl ContentAddressed for Agent {
    fn content_hash(&self) -> Result<ContentHash, ContentAddressingError> {
        ContentHash::for_object(self)
    }
    
    fn to_bytes(&self) -> Result<Vec<u8>, ContentAddressingError> {
        Serializer::to_bytes(self)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self, ContentAddressingError> {
        Deserializer::from_bytes(bytes)
            .map_err(|e| ContentAddressingError::SerializationError(e.to_string()))
    }
}
```

### Agent-Specific Resource Accessors

Agent resources are accessed through specialized resource accessors:

```rust
/// User agent accessor
#[async_trait]
pub trait UserAccessor: ResourceAccessor<Resource = User> {
    /// Authenticate a user
    async fn authenticate(&self, id: &ResourceId, credentials: &Credentials) 
        -> Result<bool, ResourceError>;
    
    /// Send a message to a user
    async fn send_message(&self, from: &ResourceId, to: &ResourceId, message: Message)
        -> Result<(), ResourceError>;
    
    /// Get a user's capabilities
    async fn get_capabilities(&self, id: &ResourceId) -> Result<Vec<Capability>, ResourceError>;
}

/// Committee agent accessor
#[async_trait]
pub trait CommitteeAccessor: ResourceAccessor<Resource = Committee> {
    /// Submit an observation for validation
    async fn submit_observation(&self, observation: Observation) 
        -> Result<ObservationId, ResourceError>;
    
    /// Verify a fact with the committee
    async fn verify_fact(&self, fact: Fact) -> Result<VerificationResult, ResourceError>;
    
    /// Get the committee's domain
    async fn get_domain(&self, id: &ResourceId) -> Result<DomainId, ResourceError>;
}
```

### Integration with Effect System

Agents interact with the Effect System to perform operations:

1. Users initiate effects through user interfaces or API calls
2. Operators manage system infrastructure and perform maintenance tasks
3. Committees observe facts and validate cross-domain operations

### Integration with Capability System

The Agent-Based Resource System leverages the Capability System for access control:

1. Users receive capabilities for resources they're allowed to access
2. Operators receive administrative capabilities for system management
3. Committees have specialized capabilities for fact observation and validation

## Agent-Based Security Model

The Agent-Based Resource System provides a comprehensive security model:

1. **Identity Management**: Agents have cryptographic identities
2. **Authentication**: Agent resources authenticate through signatures with their public keys
3. **Authorization**: The Capability System governs access control based on agent capabilities
4. **Audit Trail**: All operations are logged with the initiating agent
5. **Delegation**: Agents can delegate capabilities to other agents within constraints

## System Diagram

```
                     ┌─────────────────┐
                     │  Effect System  │
                     └────────┬────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────┐
│             Agent-Based Resource System            │
│                                                    │
│    ┌──────────┐      ┌──────────┐     ┌──────────┐ │
│    │          │      │          │     │          │ │
│    │   User   │◄────►│ Operator │◄───►│Committee │ │
│    │          │      │          │     │          │ │
│    └────┬─────┘      └────┬─────┘     └────┬─────┘ │
│         │                 │                 │      │
│    ┌────▼─────┐      ┌────▼─────┐     ┌────▼─────┐ │
│    │  User    │      │  System  │     │  Domain  │ │
│    │Resources │      │Resources │     │Resources │ │
│    └──────────┘      └──────────┘     └──────────┘ │
│                                                    │
└────────────────────────┬───────────────────────────┘
                         │
                         ▼
           ┌─────────────────────────┐
           │   Capability System     │
           └────────────┬────────────┘
                        │
                        ▼
           ┌─────────────────────────┐
           │    Resource System      │
           └─────────────────────────┘
```

## Agent State Transitions

Agents follow a well-defined lifecycle:

1. **Created**: Initial state when an agent is created
2. **Initialized**: Agent has been initialized with capabilities
3. **Active**: Agent is actively performing operations
4. **Suspended**: Agent is temporarily inactive
5. **Upgraded**: Agent has been upgraded to a new version
6. **Terminated**: Agent has been permanently deactivated

## Agent Relationships

Agents can form relationships with other resources:

1. **Ownership**: Agent owns and has full control over a resource
2. **Parent/Child**: Hierarchical relationship between agents
3. **Delegation**: Agent delegates capabilities to another agent
4. **Dependency**: Agent requires another resource to function
5. **Peer**: Equal relationship between collaborating agents

## Benefits

The integration of agent-based entities into the Resource System provides several benefits:

1. **Architectural Cohesion**: Unifies entity and resource concepts under a single consistent model
2. **Consistency**: Uses the same mechanisms for identification, access control, and state management
3. **Reduced Cognitive Load**: Developers only need to understand one system (Resource) rather than multiple systems
4. **Content Addressing Consistency**: All entities use the same content addressing mechanism
5. **Domain Clarity**: Provides an accurate model of how domains and validator committees work

## Where Implemented

The Agent-Based Resource System is implemented in the following crates and modules:

| Component | Crate | Module |
|-----------|-------|--------|
| Agent Implementation | `causality-core` | `causality_core::agent` |
| User Implementation | `causality-core` | `causality_core::agent::user` |
| Operator Implementation | `causality-core` | `causality_core::agent::operator` |
| Committee Implementation | `causality-core` | `causality_core::agent::committee` |
| Agent Accessors | `causality-core` | `causality_core::agent::accessors` |

## References

- [ADR-032: Agent-Based Resource System](../../../spec/adr_032_consolidated_agent_resource_system.md)
- [System Contract](../../../spec/system_contract.md)
