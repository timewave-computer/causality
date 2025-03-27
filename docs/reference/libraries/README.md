# Library Reference

*This documentation provides reference information for the Causality libraries.*

*Last updated: 2023-08-20*

## Overview

Causality is organized into several libraries, each providing specific functionality. This reference documentation provides information about the available libraries, their purpose, public interfaces, and usage examples.

## Core Libraries

### [causality_types](./causality-types.md)

The `causality_types` crate provides shared types and interfaces used throughout the Causality system. It serves as a foundation for all other crates, defining common data structures without implementing complex logic.

```rust
// Import core types
use causality_types::{
    resource::{ResourceId, ResourceType},
    content::{ContentHash, ContentAddressed},
    agent::{AgentId, AgentType},
    capability::{Capability, CapabilityType},
    effect::{EffectId, EffectType},
    operation::{OperationId, OperationType},
};
```

Key components:
- Content addressing types and traits
- Domain types and identifiers
- Resource types and identifiers
- Agent types and identifiers
- Capability types and constraints
- Effect types and interfaces
- Operation types and parameters

### [causality_core](./causality-core.md)

The `causality_core` crate implements the foundational systems of Causality, including the unified resource system, effect system, and capability system. As per ADR-032, it integrates effects directly into the core architecture.

```rust
// Import core functionality
use causality_core::{
    resource::{ResourceManager, ResourceGuard, LockMode},
    effect::{EffectSystem, Effectful, EffectContext},
    capability::{CapabilityVerifier, CapabilityChain},
    content::{ContentStore, ContentVerifier},
    time::{TimeSystem, CausalClock},
};
```

Key components:
- Resource management and concurrency control
- Effect system implementation and execution
- Capability verification and delegation
- Content addressing and verification
- Time services and causal relations
- Type-safe registry for heterogeneous types

### [causality_agent](./causality-agent.md)

The `causality_agent` crate implements the agent system in Causality, providing the specialized resource types that can initiate operations and interact with the system.

```rust
// Import agent components
use causality_agent::{
    agent::{Agent, AgentManager, AgentBuilder},
    user::{UserAgent, UserCredentials},
    committee::{CommitteeAgent, CommitteePolicy},
    operator::{OperatorAgent, OperatorConfig},
    operation::{Operation, OperationContext},
};
```

Key components:
- Agent implementation and management
- User, Committee, and Operator agents
- Operation execution and lifecycle
- Agent capability management
- Agent authentication and sessions

### causality_crypto

The `causality_crypto` crate provides cryptographic primitives used throughout the Causality system.

```rust
// Import cryptographic primitives
use causality_crypto::{
    hash::PoseidonHasher,
    signature::EddsaSignature,
    verify::SignatureVerifier,
    commitment::PedersenCommitment,
    smt::SparseMerkleTree,
};
```

Key components:
- Poseidon hash implementation
- Signature generation and verification
- Zero-knowledge commitment schemes
- Sparse Merkle Tree implementation
- Key management utilities

## Domain-Specific Libraries

### causality_domain

The `causality_domain` crate provides functionality for integrating with external blockchains and data sources.

```rust
// Import domain adapter components
use causality_domain::{
    adapter::DomainAdapter,
    boundary::BoundaryCrossing,
    events::EventSubscription,
    blockchain::{
        ethereum::EthereumAdapter,
        solana::SolanaAdapter,
    },
};
```

Key components:
- Domain adapter interfaces
- Boundary crossing protocols
- Event subscription mechanisms
- Blockchain integrations
- Database integrations

### causality_vm

The `causality_vm` crate provides virtual machine support for executing operations securely.

```rust
// Import VM components
use causality_vm::{
    VirtualMachine,
    VMState,
    Instruction,
    proof::ProofGenerator,
};
```

Key components:
- Virtual machine implementation
- Instruction set and execution
- Secure operation execution
- Proof generation and verification
- State transition validation

## Integration and Execution Libraries

### [causality_engine](./causality-engine.md)

The `causality_engine` crate implements the execution engine for Causality, providing the runtime environment for executing operations and managing system resources.

```rust
// Import engine components
use causality_engine::{
    engine::{CausalityEngine, EngineConfig},
    vm::{VirtualMachine},
    facts::{FactStore, FactVerifier},
    transaction::{Transaction, TransactionManager},
    domains::{DomainRegistry, DomainAdapter},
};
```

Key components:
- System integration and orchestration
- Virtual machine for secure execution
- Fact management and verification
- Transaction management and lifecycle
- Domain integration and management

## Client Libraries

### causality_client

The `causality_client` crate provides client functionality for interacting with the Causality API.

```rust
// Import client components
use causality_client::{
    CausalityClient,
    resources::ResourceClient,
    capabilities::CapabilityClient,
    effects::EffectClient,
    domains::DomainClient,
};
```

Key components:
- API client configuration
- Resource management
- Capability management
- Effect execution
- Domain integration

### causality_sdk

The `causality_sdk` crate provides a high-level SDK for building applications on top of Causality.

```rust
// Import SDK components
use causality_sdk::{
    CausalityApp,
    resource::ResourceBuilder,
    effect::EffectBuilder,
    capability::CapabilityBuilder,
    transaction::TransactionBuilder,
};
```

Key components:
- Application configuration
- Resource builder patterns
- Effect composition utilities
- Capability delegation helpers
- Transaction management

## References

- [ADR-032: Role-Based Resource System](../../../spec/adr_032-role-based-resource-system.md)
- [System Contract](../../../spec/system_contract.md)
- [Data Structures](../data-structures/README.md) 