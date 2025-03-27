# causality-types Library Reference

*This document provides reference information for the `causality-types` crate.*

*Last updated: 2023-08-20*

## Overview

The `causality-types` crate provides shared types and interfaces used throughout the Causality system. It serves as a foundation for all other crates in the system, defining common data structures, identifiers, and trait definitions without implementing complex logic.

## Key Modules

### causality_types::resource

Types related to resources and resource identification.

```rust
use causality_types::resource::{
    ResourceId,
    ResourceType,
    ResourceMetadata,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `ResourceId` | Unique identifier for a resource |
| `ResourceType` | Enumeration of resource types |
| `ResourceMetadata` | Metadata associated with a resource |

### causality_types::content

Types related to content addressing.

```rust
use causality_types::content::{
    ContentHash,
    HashAlgorithm,
    ContentAddressed,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `ContentHash` | Cryptographic hash of content-addressed data |
| `HashAlgorithm` | Enumeration of supported hash algorithms |
| `ContentAddressed` | Trait for types that can be content-addressed |

### causality_types::agent

Types related to agents and agent identification.

```rust
use causality_types::agent::{
    AgentId,
    AgentType,
    AgentContext,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `AgentId` | Unique identifier for an agent |
| `AgentType` | Enumeration of agent types (User, Committee, Operator) |
| `AgentContext` | Context information for agent operations |

### causality_types::capability

Types related to capabilities and authorization.

```rust
use causality_types::capability::{
    Capability,
    CapabilityType,
    CapabilityConstraint,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `Capability` | Token granting authorization to perform operations |
| `CapabilityType` | Enumeration of capability types |
| `CapabilityConstraint` | Constraint on capability usage |

### causality_types::effect

Types related to effects and effect execution.

```rust
use causality_types::effect::{
    EffectId,
    EffectType,
    EffectError,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `EffectId` | Unique identifier for an effect |
| `EffectType` | Enumeration of effect types |
| `EffectError` | Error types for effect execution |

### causality_types::operation

Types related to operations.

```rust
use causality_types::operation::{
    OperationId,
    OperationType,
    OperationResult,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `OperationId` | Unique identifier for an operation |
| `OperationType` | Enumeration of operation types |
| `OperationResult` | Result of an operation execution |

### causality_types::time

Types related to time and temporal facts.

```rust
use causality_types::time::{
    TimeStamp,
    CausalRelation,
    TemporalFact,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `TimeStamp` | Timestamp for events |
| `CausalRelation` | Relationship between temporal facts |
| `TemporalFact` | Fact with temporal information |

### causality_types::domain

Types related to domains and domain adapters.

```rust
use causality_types::domain::{
    DomainId,
    DomainType,
    DomainEvent,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `DomainId` | Unique identifier for a domain |
| `DomainType` | Enumeration of domain types |
| `DomainEvent` | Event from a domain |

### causality_types::common

Common utility types used throughout the system.

```rust
use causality_types::common::{
    Serializable,
    Deserializable,
    Result,
    Error,
};
```

#### Primary Types

| Type | Description |
|------|-------------|
| `Serializable` | Trait for types that can be serialized |
| `Deserializable` | Trait for types that can be deserialized |
| `Result<T, E>` | Standard result type |
| `Error` | Common error type |

## Trait Definitions

### ContentAddressed

Trait for types that can be content-addressed.

```rust
pub trait ContentAddressed {
    /// Calculate the content hash for this object
    fn calculate_content_hash(&self) -> Result<ContentHash, ContentHashError>;
    
    /// Get the pre-calculated content hash
    fn content_hash(&self) -> &ContentHash;
    
    /// Create a new instance with the specified content hash
    fn with_content_hash(self, hash: ContentHash) -> Self;
}
```

### Serializable

Trait for types that can be serialized.

```rust
pub trait Serializable {
    /// Serialize to bytes
    fn to_bytes(&self) -> Result<Vec<u8>, SerializationError>;
    
    /// Serialize to JSON
    fn to_json(&self) -> Result<serde_json::Value, SerializationError>;
}
```

### Deserializable

Trait for types that can be deserialized.

```rust
pub trait Deserializable: Sized {
    /// Deserialize from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, DeserializationError>;
    
    /// Deserialize from JSON
    fn from_json(json: &serde_json::Value) -> Result<Self, DeserializationError>;
}
```

## Usage Example

```rust
use causality_types::{
    resource::{ResourceId, ResourceType},
    content::{ContentHash, ContentAddressed},
    agent::{AgentId, AgentType},
    capability::{Capability, CapabilityType},
    common::Result,
};

// Create a resource ID
let resource_id = ResourceId::new(
    "resource", 
    DomainId::local(),
    "my-resource"
)?;

// Create an agent ID
let agent_id = AgentId::new(
    AgentType::User,
    DomainId::local(),
    "alice"
)?;

// Create a capability
let capability = Capability::new(
    resource_id.clone(),
    CapabilityType::Write,
)?;

// Content-address a structure
let my_struct = MyStruct::new(42, "meaning of life");
let content_hash = my_struct.calculate_content_hash()?;
let my_struct_with_hash = my_struct.with_content_hash(content_hash);
```

## References

- [ADR-032: Role-Based Resource System](../../../spec/adr_032-role-based-resource-system.md)
- [System Contract](../../../spec/system_contract.md)
- [Core Architecture](../../architecture/core) 