# Agent

*This document provides reference information for the `Agent` data structure.*

*Last updated: 2023-09-05*

## Overview

An `Agent` is a specialized resource that represents an entity in the system such as users, committees, and operators. As defined in ADR-032, agents are the primary actors in the system and form a key part of the unified system components architecture.

## Type Definition

```rust
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

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `resource` | `Box<dyn Resource>` | The underlying resource implementation for the agent |
| `identity` | `AgentId` | The unique identifier for the agent |
| `capabilities` | `Vec<Capability>` | The capabilities held by the agent |
| `state` | `AgentState` | The current state of the agent |
| `relationships` | `Vec<ResourceRelationship>` | The agent's relationships with other resources |

## Agent Identity

The `AgentId` uniquely identifies an agent in the system:

```rust
pub struct AgentId {
    /// The type of agent
    agent_type: AgentType,
    
    /// The domain the agent belongs to
    domain_id: DomainId,
    
    /// The unique name within the domain
    name: String,
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

## Agent State

The state of an agent represents its current operational status:

```rust
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

## Capabilities

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

## Operations

Agents can execute operations on resources:

```rust
impl Agent {
    /// Execute an operation
    pub fn execute(&self, operation: Operation, context: OperationContext) 
        -> Result<OperationResult, OperationError>;
    
    /// Create an operation builder for this agent
    pub fn create_operation(&self) -> OperationBuilder;
    
    /// Schedule an operation for future execution
    pub fn schedule_operation(
        &self, 
        operation: Operation, 
        schedule: OperationSchedule
    ) -> Result<OperationId, OperationError>;
}
```

## Resource Relationships

Agents can have relationships with other resources:

```rust
impl Agent {
    /// Add a relationship with another resource
    pub fn add_relationship(
        &mut self, 
        relationship: ResourceRelationship
    ) -> Result<(), RelationshipError>;
    
    /// Remove a relationship
    pub fn remove_relationship(
        &mut self, 
        resource_id: &ResourceId, 
        relationship_type: &str
    ) -> Result<(), RelationshipError>;
    
    /// Get all relationships
    pub fn relationships(&self) -> &[ResourceRelationship];
    
    /// Find relationships of a specific type
    pub fn find_relationships(
        &self, 
        relationship_type: &str
    ) -> Vec<&ResourceRelationship>;
}
```

## Effect Usage

Agents use effects to interact with the system. As per ADR-032, effects are now integrated directly into the core crate:

```rust
impl Agent {
    /// Read a resource with effects
    pub fn read_resource<T: Resource>(&self, id: ResourceId) 
        -> impl Effectful<ResourceGuard<T>, ResourceEffect> {
        Effectful::new(move |ctx| {
            // Verify capabilities
            ctx.perform(VerifyCapability { 
                capability: self.capabilities.find("read")?,
                resource_id: id.clone(),
            })?;
            
            // Access the resource
            let guard = ctx.perform(GetResource { 
                id: id.clone(), 
                lock_mode: LockMode::Read 
            })?;
            
            Ok(guard)
        })
    }
    
    /// Write to a resource with effects
    pub fn write_resource<T: Resource>(&self, id: ResourceId) 
        -> impl Effectful<ResourceGuard<T>, ResourceEffect> {
        Effectful::new(move |ctx| {
            // Verify capabilities
            ctx.perform(VerifyCapability { 
                capability: self.capabilities.find("write")?,
                resource_id: id.clone(),
            })?;
            
            // Access the resource
            let guard = ctx.perform(GetResource { 
                id: id.clone(), 
                lock_mode: LockMode::Write 
            })?;
            
            Ok(guard)
        })
    }
}
```

## Agent Authentication

Agents can be authenticated using various methods:

```rust
impl AgentManager {
    /// Authenticate an agent
    pub fn authenticate(&self, id: &str, method: AuthMethod) 
        -> Result<AgentSession, AuthError>;
}

/// Authentication methods
pub enum AuthMethod {
    /// Password authentication
    Password(String),
    
    /// Token authentication
    Token(String),
    
    /// Key-based authentication
    Key(PublicKey, Signature),
}
```

## Agent Builder

Agents can be created using the builder pattern:

```rust
/// Create a new user agent
let user_agent = AgentBuilder::new()
    .user("alice")
    .with_capability(read_capability)
    .with_capability(write_capability)
    .with_relationship(ResourceRelationship::new(
        database_id,
        "owner",
        relationship_metadata
    )?)
    .build()?;
```

## Usage Example

```rust
use causality_agent::{
    agent::{Agent, AgentBuilder, AgentType},
    operation::{Operation, OperationBuilder},
};
use causality_core::{
    resource::{ResourceId, ResourceManager},
    capability::{Capability, CapabilityType},
    effect::{EffectSystem, Effectful},
};

// Create a resource manager
let resource_manager = ResourceManager::new();

// Create an effect system
let effect_system = EffectSystem::new();

// Create a resource ID
let database_id = ResourceId::new("database", domain_id, "customer_db")?;

// Create capabilities
let read_capability = Capability::new(
    database_id.clone(),
    CapabilityType::Read,
)?;

let write_capability = Capability::new(
    database_id.clone(),
    CapabilityType::Write,
)?;

// Create a user agent
let user_agent = AgentBuilder::new()
    .user("alice")
    .with_capability(read_capability)
    .with_capability(write_capability)
    .build()?;

// Register the agent
resource_manager.register_agent(user_agent)?;

// Get the agent
let alice = resource_manager.get_agent("alice")?;

// Create an operation
let operation = alice.create_operation()
    .target_resource(database_id)
    .action("query")
    .parameters(json!({ "query": "SELECT * FROM customers" }))
    .build()?;

// Execute the operation
let result = effect_system.execute(
    alice.execute(operation),
    EffectContext::new()
)?;
```

## References

- [ADR-032: Role-Based Resource System](../../../spec/adr_032_consolidated_agent_resource_system.md)
- [System Contract](../../../spec/system_contract.md)
- [Causality Agent Library](../../reference/libraries/causality-agent.md) 