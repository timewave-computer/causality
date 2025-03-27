# causality-agent Library Reference

*This document provides reference information for the `causality-agent` crate.*

*Last updated: 2023-08-20*

## Overview

The `causality-agent` crate implements the agent system in Causality, providing the specialized resource types that can initiate operations and interact with the system. Agents are the primary actors in the system, holding capabilities and performing operations on resources.

## Key Modules

### causality_agent::agent

Core agent implementation and management.

```rust
use causality_agent::agent::{
    Agent,
    AgentManager,
    AgentBuilder,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `Agent` | Core agent implementation |
| `AgentManager` | Manager for agent instances |
| `AgentBuilder` | Builder for creating new agents |
| `AgentState` | State of an agent |

### causality_agent::user

User agent implementation.

```rust
use causality_agent::user::{
    UserAgent,
    UserAgentBuilder,
    UserCredentials,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `UserAgent` | Agent representing a user |
| `UserAgentBuilder` | Builder for user agents |
| `UserCredentials` | Credentials for user authentication |

### causality_agent::committee

Committee agent implementation.

```rust
use causality_agent::committee::{
    CommitteeAgent,
    CommitteePolicy,
    CommitteeMember,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `CommitteeAgent` | Agent representing a committee |
| `CommitteePolicy` | Policy for committee decisions |
| `CommitteeMember` | Member of a committee |

### causality_agent::operator

Operator agent implementation.

```rust
use causality_agent::operator::{
    OperatorAgent,
    OperatorConfig,
    OperatorRole,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `OperatorAgent` | Agent representing a system operator |
| `OperatorConfig` | Configuration for an operator |
| `OperatorRole` | Role of an operator |

### causality_agent::operation

Operation execution and lifecycle.

```rust
use causality_agent::operation::{
    Operation,
    OperationContext,
    OperationBuilder,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `Operation` | Operations that can be executed by agents |
| `OperationContext` | Context for operation execution |
| `OperationBuilder` | Builder for creating operations |

### causality_agent::capabilities

Agent capability management.

```rust
use causality_agent::capabilities::{
    AgentCapability,
    CapabilityManager,
    CapabilityProof,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `AgentCapability` | Capability specific to agents |
| `CapabilityManager` | Manager for agent capabilities |
| `CapabilityProof` | Proof of capability ownership |

## Agent Resource Model

Agents are implemented as specialized resources within the unified resource system, as described in ADR-032. This allows them to be managed consistently with other resources while providing specialized functionality.

### Agent Structure

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

### Agent Creation

```rust
// Create a new user agent
let user_agent = AgentBuilder::new()
    .user("alice")
    .with_capability(read_capability)
    .with_capability(write_capability)
    .build()?;

// Register the agent with the resource manager
resource_manager.register_agent(user_agent)?;
```

### Agent Operations

Agents can perform operations on resources:

```rust
// Create an operation
let operation = OperationBuilder::new()
    .target_resource(database_id)
    .action("query")
    .parameters(params)
    .build()?;

// Execute the operation
let result = agent.execute(operation, context)?;
```

## Effect Usage by Agents

Agents use the effect system to interact with resources:

```rust
impl Agent {
    pub fn read_resource<T: Resource>(&self, id: ResourceId) 
        -> impl Effectful<ResourceGuard<T>, ResourceEffect> 
    {
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
}
```

## Agent Relationships

Agents can have relationships with other resources:

```rust
// Define a relationship between an agent and a resource
agent.add_relationship(
    ResourceRelationship::new(
        database_id,
        "owner",
        relationship_metadata
    )?
)?;
```

## Usage Example

```rust
use causality_agent::{
    agent::{AgentBuilder, AgentManager},
    operation::{OperationBuilder},
    user::{UserCredentials},
};
use causality_core::{
    resource::{ResourceManager},
    effect::{EffectSystem},
};

// Create a resource manager
let mut resource_manager = ResourceManager::new();

// Create an agent manager
let mut agent_manager = AgentManager::new(&resource_manager);

// Create an effect system
let mut effect_system = EffectSystem::new();

// Create a user agent
let user_credentials = UserCredentials::new("alice", "password123")?;
let user_agent = AgentBuilder::new()
    .user("alice")
    .with_credentials(user_credentials)
    .build()?;

// Register the agent
agent_manager.register(user_agent)?;

// Create a database resource
let database = Database::new("customer_data")?;
resource_manager.create("customer_db", database)?;

// Create an operation
let operation = OperationBuilder::new()
    .target_resource("customer_db")
    .action("query")
    .parameters(json!({ "query": "SELECT * FROM customers" }))
    .build()?;

// Get the agent
let alice = agent_manager.get("alice")?;

// Execute the operation
let result = effect_system.execute(
    alice.execute(operation),
    EffectContext::new()
)?;
```

## Agent Authentication

Agents can be authenticated using various methods:

```rust
// Authenticate a user agent
let agent = agent_manager.authenticate("alice", 
    AuthMethod::Password("password123".to_string())
)?;

// Authenticate with a token
let agent = agent_manager.authenticate("alice", 
    AuthMethod::Token("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")
)?;
```

## References

- [ADR-032: Role-Based Resource System](../../../spec/adr_032-role-based-resource-system.md)
- [System Contract](../../../spec/system_contract.md) 