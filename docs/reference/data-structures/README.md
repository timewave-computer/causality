# Data Structures Reference

*This documentation provides reference information for the core data structures used in Causality.*

*Last updated: 2023-09-05*

## Overview

Causality uses a variety of specialized data structures to ensure efficiency, security, and correctness. This reference documentation provides information about the key data structures, their properties, and usage patterns.

## Core Data Structures

### Content Hash

The `ContentHash` structure represents a cryptographic hash of content-addressed data.

```rust
pub struct ContentHash {
    algorithm: HashAlgorithm,
    digest: [u8; 32],
}
```

Properties:
- `algorithm`: The hash algorithm used (e.g., Poseidon, SHA-256)
- `digest`: The 32-byte hash output

### Resource

The `Resource` structure represents a stateful object in the Causality system.

```rust
pub struct Resource {
    id: ResourceId,
    content_hash: ContentHash,
    resource_type: ResourceType,
    data: Vec<u8>,
    metadata: ResourceMetadata,
}
```

Properties:
- `id`: Unique identifier for the resource
- `content_hash`: Cryptographic hash of the resource data
- `resource_type`: Type classification of the resource
- `data`: Serialized resource data
- `metadata`: Additional information about the resource

### [Agent](./agent.md)

The `Agent` structure represents a specialized resource type that can initiate operations and interact with the system. As per ADR-032, agents are a core component of the unified system architecture and the primary actors in the system.

```rust
pub struct Agent {
    resource: Box<dyn Resource>,
    identity: AgentId,
    capabilities: Vec<Capability>,
    state: AgentState,
    relationships: Vec<ResourceRelationship>,
}
```

Properties:
- `resource`: The underlying resource implementation
- `identity`: Unique identifier for the agent
- `capabilities`: List of capabilities held by the agent
- `state`: Current operational state of the agent
- `relationships`: Relationships with other resources

### Capability

The `Capability` structure represents a delegated permission to perform operations.

```rust
pub struct Capability {
    id: CapabilityId,
    issuer: AgentId,
    target: ResourceId,
    capability_type: CapabilityType,
    constraints: Vec<Constraint>,
    expires_at: Option<TimeStamp>,
    signature: Signature,
}
```

Properties:
- `id`: Unique identifier for the capability
- `issuer`: Agent that issued the capability
- `target`: Resource this capability grants access to
- `capability_type`: Type of capability (Read, Write, Execute, etc.)
- `constraints`: List of constraints on capability use
- `expires_at`: Optional expiration timestamp
- `signature`: Cryptographic signature by the issuer

### [Effect](./effect.md)

The `Effect` trait represents side effects or interactions with external systems. As per ADR-032, effects are now integrated directly into the core crate as part of the unified system components architecture.

```rust
pub trait Effect: Clone + Send + Sync + 'static {
    type Output;
    type Error;
    
    fn description(&self) -> String;
}
```

Properties:
- `Output`: The return type of the effect
- `Error`: The error type that can occur
- `description()`: Human-readable description of the effect

### [Operation](./operation.md)

The `Operation` structure represents an action that an agent performs on a resource. Operations are the primary mechanism for changing state in the system and, as per ADR-032, are a core component of the unified system architecture.

```rust
pub struct Operation {
    id: OperationId,
    agent_id: AgentId,
    target_resource: ResourceId,
    action: String,
    parameters: OperationParameters,
    required_capabilities: Vec<CapabilityType>,
    metadata: OperationMetadata,
}
```

Properties:
- `id`: Unique identifier for the operation
- `agent_id`: Agent that initiated the operation
- `target_resource`: Resource that is the target of the operation
- `action`: Action to perform on the resource
- `parameters`: Parameters for the action
- `required_capabilities`: Capabilities required for this operation
- `metadata`: Additional information about the operation

### Fact

The `Fact` structure represents an immutable piece of information, often the result of an operation.

```rust
pub struct Fact {
    id: FactId,
    content_hash: ContentHash,
    fact_type: FactType,
    data: Vec<u8>,
    metadata: FactMetadata,
    dependencies: Vec<FactId>,
    timestamp: TimeStamp,
}
```

Properties:
- `id`: Unique identifier for the fact
- `content_hash`: Cryptographic hash of the fact data
- `fact_type`: Type classification of the fact
- `data`: Serialized fact data
- `metadata`: Additional information about the fact
- `dependencies`: Facts that this fact depends on
- `timestamp`: When the fact was created

## Specialized Data Structures

### Sparse Merkle Tree

```rust
pub struct SparseMerkleTree<K, V> {
    root: ContentHash,
    depth: usize,
    nodes: HashMap<ContentHash, Node<K, V>>,
}
```

Key operations:
- `insert(key, value)`: Insert a key-value pair
- `get(key)`: Retrieve a value by key
- `prove(key)`: Generate a proof for a key
- `verify_proof(key, value, proof)`: Verify a proof

### Vector Commitment

```rust
pub struct VectorCommitment<T> {
    root: ContentHash,
    leaves: Vec<T>,
    nodes: HashMap<ContentHash, Node>,
}
```

Key operations:
- `commit(items)`: Create a commitment to a vector
- `prove(index)`: Generate a proof for an item
- `verify_proof(index, item, proof)`: Verify a proof

### Capability DAG

```rust
pub struct CapabilityDag {
    nodes: HashMap<CapabilityId, CapabilityNode>,
    edges: HashMap<CapabilityId, Vec<CapabilityId>>,
}
```

Key operations:
- `add_capability(capability)`: Add a capability to the DAG
- `add_delegation(from, to)`: Add a delegation edge
- `validate_chain(capability)`: Validate a delegation chain
- `find_path(from, to)`: Find a delegation path

## References

- [ADR-032: Agent-Based Resource System](../../../spec/adr_032_consolidated_agent_resource_system.md)
- [System Contract](../../../spec/system_contract.md)
- [Core Architecture](../../architecture/core)
- [Library References](../libraries/README.md) 