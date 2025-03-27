# Agent Resource System Guide

*This document provides guidance on using the agent resource system, as implemented according to ADR-032: Consolidated Agent-Resource System.*

*Last updated: [Current Date]*

## Overview

The agent resource system is a core component of the Causality platform that models agents (users, committees, and operators) as specialized resources within the unified resource system. This approach provides a consistent way to manage identities, capabilities, and relationships in the system.

## Key Components

### 1. Agent Types

The system defines three main types of agents:

- **User Agents**: Represent human users of the system
- **Committee Agents**: Represent groups of validators materializing chain state
- **Operator Agents**: Represent automated system operators

Each agent type has specific behaviors and capabilities but shares a common interface.

### 2. Agent Resources

Agents are implemented as resources with the following attributes:

```rust
struct Agent {
    // Resource implementation (ID, metadata, etc.)
    resource: Resource,
    
    // Agent identity
    identity: AgentId,
    
    // Capabilities this agent holds
    capabilities: Vec<Capability>,
    
    // Current state
    state: AgentState,
    
    // Relationships with other resources
    relationships: Vec<ResourceRelationship>,
}
```

### 3. Capability Management

Agents hold capabilities that grant them authority to perform operations on resources:

```rust
// Add a capability
agent.add_capability(read_capability).await?;

// Check for a capability
if agent.has_capability("resource:read") {
    // Perform operation
}

// Remove a capability
agent.remove_capability("resource:read").await?;
```

### 4. Operations

Agents can execute operations on resources:

```rust
// Create an operation
let operation = OperationBuilder::new()
    .agent_id(agent.id().clone())
    .target_resource(resource_id)
    .operation_type(OperationType::Read)
    .build()?;

// Execute the operation
let result = agent.execute_operation(operation, context).await?;
```

### 5. Relationships

Agents can have relationships with other resources:

```rust
// Create a relationship
let relationship = AgentRelationship::new(
    RelationshipType::Owns,
    document_id,
    vec![write_capability],
    HashMap::new(),
);

// Add the relationship
agent.add_relationship(relationship).await?;
```

## Usage Patterns

### Creating Agents

```rust
// Create a user agent
let agent = AgentBuilder::new()
    .agent_type(AgentType::User)
    .state(AgentState::Active)
    .with_capability(read_capability)
    .with_capability(write_capability)
    .build()?;
```

### Agent Authentication

```rust
// Authenticate a user agent
let credentials = UserCredentials::new("username", "password");
let authenticated = user_agent.authenticate(&credentials).await?;

if authenticated {
    // Allow access
} else {
    // Deny access
}
```

### Service Advertisement

```rust
// Advertise a service
let service = ServiceStatus::new(
    agent.id().clone(), 
    "document-storage",
    ServiceState::Available,
    HashMap::new(),
);

agent.advertise_service(service).await?;
```

### Agent Communication

```rust
// Send a message to another agent
let message = Message::new(
    sender.id().clone(),
    recipient.id().clone(),
    "Hello, world!".as_bytes().to_vec(),
    HashMap::new(),
);

sender.send_message(message).await?;
```

## Integration with Other Systems

### Effect System Integration

Agents interact with the effect system to perform operations:

```rust
// Create an effect
let effect = ReadResourceEffect::new(resource_id);

// Add the effect to an operation
let operation = OperationBuilder::new()
    .agent_id(agent.id().clone())
    .target_resource(resource_id)
    .operation_type(OperationType::Read)
    .add_effect(Box::new(effect))
    .build()?;

// Execute the operation
let result = agent.execute_operation(operation, context).await?;
```

### Resource System Integration

Agents are themselves resources, enabling consistent management:

```rust
// Store an agent in resource storage
let agent_id = resource_storage.store(agent).await?;

// Retrieve an agent from resource storage
let agent = resource_storage.get::<AgentImpl>(agent_id).await?;
```

## Best Practices

1. **Content Addressing**: Always use content-addressed identifiers for agents and their components.
2. **Capability Checking**: Verify capabilities before performing operations.
3. **State Management**: Always update agent state using the proper methods to ensure content hashes are updated.
4. **Relationship Management**: Use relationships to model ownership and other connections between resources.
5. **Service Advertisement**: Declare services your agent offers using the service status system.

## Implementation Status

The agent resource system is currently in development. The following components are being implemented:

- [x] Core agent types (AgentId, AgentType, AgentState)
- [x] Agent trait and implementation
- [x] Operation system for capability-checked resource operations
- [ ] Authorization system for capability verification
- [ ] Agent registry for managing agents
- [ ] Specialized agent types (User, Committee, Operator)
- [ ] Service status implementation for service advertisement
- [ ] Obligation manager for capability enforcement
- [ ] Messaging system for agent communication
- [ ] Capability bundle implementation for role-based capability sets

## Migration

If you are currently using the actor system, we recommend waiting for the complete implementation of the agent-resource system before migrating. A detailed migration guide will be provided when the implementation is complete. 