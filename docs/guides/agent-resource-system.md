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

# Agent Resource System Integration Guide

This guide explains how to integrate and use the Agent Resource System in your Causality applications. The Agent Resource System provides a unified approach to modeling users, committees, and operators as specialized resources with consistent identity, capabilities, and state management.

## Getting Started

To use the Agent Resource System, you'll need to import the necessary components from the `causality-core` crate:

```rust
use causality_core::resource::agent::{
    Agent, AgentBuilder, AgentId, AgentRegistry, AgentType, AgentState,
    capability::{Capability, CapabilityBundle},
    messaging::{Message, MessageFactory, MessageType},
    operation::{Operation, OperationType},
};
```

## Creating Agents

Agents can be created using the `AgentBuilder`:

```rust
// Create a new user agent
let user_agent = AgentBuilder::new()
    .agent_type(AgentType::User)
    .state(AgentState::Active)
    .with_capability(Capability::new("resource123", "read", None))
    .with_metadata("display_name", "Alice")
    .build()
    .unwrap();

// Create a committee agent
let committee_agent = AgentBuilder::new()
    .agent_type(AgentType::Committee)
    .state(AgentState::Active)
    .with_capability(Capability::new("resource456", "write", None))
    .with_metadata("committee_type", "reviewers")
    .build()
    .unwrap();

// Create an operator agent
let operator_agent = AgentBuilder::new()
    .agent_type(AgentType::Operator)
    .state(AgentState::Active)
    .with_capability(Capability::new("*", "admin", None))
    .with_metadata("service", "content_manager")
    .build()
    .unwrap();
```

## Managing Agents with AgentRegistry

The `AgentRegistry` provides agent registration, lookup, and management:

```rust
// Register agents
registry.register_agent(Box::new(user_agent.clone())).await?;
registry.register_agent(Box::new(committee_agent.clone())).await?;
registry.register_agent(Box::new(operator_agent.clone())).await?;

// Look up agents
let found_agent = registry.get_agent_by_id(&user_agent.agent_id()).await?;
let agents = registry.find_agents_by_type(AgentType::User).await?;
let alice = registry.find_agents_by_metadata("display_name", "Alice").await?;

// Deregister an agent
registry.deregister_agent(&operator_agent.agent_id()).await?;
```

## Working with Agent Operations

Operations allow agents to interact with resources through capability-checked effects:

```rust
// Create an operation
let operation = user_agent.create_operation()
    .target_resource(resource_id)
    .operation_type(OperationType::Read)
    .add_effect(read_effect)
    .build()?;

// Execute the operation
let result = user_agent.execute_operation(operation, context).await?;

// Check operation result
match result {
    OperationResult::Success(data) => println!("Operation succeeded: {:?}", data),
    OperationResult::Failure(error) => println!("Operation failed: {:?}", error),
    OperationResult::Pending => println!("Operation is pending further approval"),
}
```

## Agent Messaging

The Messaging System enables communication between agents:

```rust
// Create a message factory
let message_factory = MessageFactory::new();

// Create a message
let message = message_factory.create_message(
    user_agent.agent_id().clone(),
    committee_agent.agent_id().clone(),
    MessageType::Request,
    "Request for document approval".to_string(),
    Some(json!({
        "document_id": "doc123",
        "urgency": "high"
    })),
)?;

// Send the message
messaging_system.send_message(message).await?;

// Receive messages
let inbox = messaging_system.get_inbox(user_agent.agent_id()).await?;
for message in inbox {
    println!("Message: {:?}", message);
}

// Create and track conversation
let conversation = messaging_system.create_conversation(
    user_agent.agent_id().clone(),
    committee_agent.agent_id().clone(),
    "Document approval".to_string(),
)?;

messaging_system.add_to_conversation(message_id, conversation.id()).await?;
```

## Working with Capability Bundles

Capability bundles allow grouping and delegating capabilities:

```rust
// Create a capability bundle
let bundle = CapabilityBundle::new("document_editor")
    .add_capability(Capability::new("documents", "read", None))
    .add_capability(Capability::new("documents", "write", None))
    .add_capability(Capability::new("comments", "create", None))
    .set_delegation_rules(DelegationRules::AllowWithApproval);

// Apply bundle to an agent
user_agent.apply_capability_bundle(bundle)?;

// Delegate a bundle to another agent
let delegation = user_agent.delegate_capability_bundle(
    bundle.id(),
    another_agent.agent_id(),
    Some(expiration_time),
)?;

// Revoke a delegation
user_agent.revoke_delegation(delegation.id())?;
```

## Checking Agent Status

Service status allows advertising and discovering agent capabilities:

```rust
// Update agent service status
user_agent.update_service_status(
    ServiceStatus::new()
        .add_service("document_processing")
        .set_availability(Availability::Available)
        .with_capacity(0.75)
)?;

// Find agents providing a service
let service_providers = registry.find_agents_by_service("document_processing").await?;
```

## Handling Obligations

Obligation Manager tracks capability-related commitments:

```rust
// Create an obligation
let obligation = Obligation::new(
    user_agent.agent_id().clone(),
    committee_agent.agent_id().clone(),
    "review_document",
    json!({"document_id": "doc123"}),
    expiration_time,
)?;

// Track the obligation
obligation_manager.add_obligation(obligation)?;

// Check agent obligations
let agent_obligations = obligation_manager.get_obligations_for_agent(user_agent.agent_id())?;

// Complete an obligation
obligation_manager.complete_obligation(obligation_id)?;
```

## Error Handling

The Agent Resource System provides structured error types:

```rust
match result {
    Ok(value) => println!("Success: {:?}", value),
    Err(AgentError::CapabilityMissing(details)) => {
        println!("Missing capability: {:?}", details);
    },
    Err(AgentError::InvalidOperation(details)) => {
        println!("Invalid operation: {:?}", details);
    },
    Err(e) => println!("Other error: {:?}", e),
}
```

## Testing with Agent Resources

Test utilities simplify testing with agent resources:

```rust
// Create a test agent
let test_agent = test_utils::create_test_agent(
    AgentType::User,
    vec![
        Capability::new("test_resource", "read", None),
        Capability::new("test_resource", "write", None),
    ],
)?;

// Set up a test registry
let test_registry = test_utils::create_test_registry(vec![test_agent.clone()])?;

// Execute test operations
let result = test_utils::execute_test_operation(
    test_agent,
    resource_id,
    OperationType::Read,
    effect,
)?;
```

## Best Practices

1. **Use Agent Types Appropriately**: Match the agent type to the entity's role - User for human users, Committee for groups, and Operator for automated services.

2. **Scope Capabilities Narrowly**: Provide agents with the minimum capabilities needed for their tasks.

3. **Use Capability Bundles**: Group related capabilities into bundles for easier management.

4. **Leverage the Registry**: Use the agent registry for agent discovery and lifecycle management.

5. **Standardize Messaging**: Define standard message types and formats for consistent communication.

6. **Track Obligations**: Use obligations to manage capability commitments and ensure accountability.

7. **Automate Verifications**: Use the built-in capability verification for all agent operations.

8. **Handle Delegations Carefully**: Set appropriate expiration times for delegated capabilities.

## Migration from Actor System

If you are migrating from the legacy actor system, see the [Actor to Agent Migration Guide](../migrations/actor-to-agent.md) for detailed instructions. 