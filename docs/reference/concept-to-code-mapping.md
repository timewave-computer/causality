# Concept-to-Code Mapping

*This document maps architectural concepts to their implementations in the codebase.*

*Last updated: 2023-09-05*

## Overview

This guide helps developers navigate between high-level architectural concepts and their concrete implementations in the Causality codebase. It shows where to find the code that implements each architectural component and how these components relate to each other across crates.

## Crate Structure

The Causality codebase follows the unified architecture defined in ADR-032, with these main crates:

| Crate | Description | Core Responsibility |
|-------|-------------|---------------------|
| `causality-types` | Core types and traits | Defines shared interfaces and data structures |
| `causality-core` | Core implementations | Implements resources, effects, capabilities |
| `causality-domain` | Domain adapters | Implements blockchain and storage integrations |
| `causality-agent` | Agent implementations | Implements agent profiles and status |
| `causality-engine` | Orchestration | Orchestrates system components |
| `causality-cli` | Command-line interface | Provides CLI access to functionality |
| `causality-api` | API server | Exposes REST API for services |

## Resource System

| Concept | Implementation | Crate | Path |
|---------|---------------|-------|------|
| Resource Trait | `Resource` trait | `causality-types` | `src/resource/mod.rs` |
| Resource ID | `ResourceId` struct | `causality-types` | `src/resource/id.rs` |
| Resource Registry | `ResourceRegistry` trait | `causality-core` | `src/resource/registry.rs` |
| Resource Manager | `ResourceManager` struct | `causality-core` | `src/resource/manager.rs` |
| Resource Versioning | `VersionedResource` trait | `causality-types` | `src/resource/versioned.rs` |
| Resource Accessor | `ResourceAccessor` trait | `causality-core` | `src/resource/accessor.rs` |
| Resource Events | `ResourceEvent` enum | `causality-types` | `src/resource/events.rs` |

## Effect System

| Concept | Implementation | Crate | Path |
|---------|---------------|-------|------|
| Effect Trait | `Effect` trait | `causality-types` | `src/effect/mod.rs` |
| Effect ID | `EffectId` struct | `causality-types` | `src/effect/id.rs` |
| Effect Engine | `EffectEngine` struct | `causality-core` | `src/effect/engine.rs` |
| Continuation | `Continuation` trait | `causality-types` | `src/effect/continuation.rs` |
| Effect Composition | `compose`, `sequence`, `parallel` functions | `causality-core` | `src/effect/composition.rs` |
| Effect Handler | `EffectHandler` trait | `causality-core` | `src/effect/handler.rs` |
| Effect Validator | `EffectValidator` struct | `causality-core` | `src/effect/validator.rs` |
| Effect Context | `EffectContext` struct | `causality-core` | `src/effect/context.rs` |
| Cross-Domain Effect | `CrossDomainEffect` trait | `causality-types` | `src/effect/cross_domain.rs` |

## Three-Layer Effect Architecture

| Layer | Concept | Implementation | Crate | Path |
|-------|---------|---------------|-------|------|
| **Algebraic Layer** | Effect Trait | `Effect` trait | `causality-types` | `src/effect/mod.rs` |
| | Continuation Model | `Continuation` trait | `causality-types` | `src/effect/continuation.rs` |
| | Effect Composition | Composition functions | `causality-core` | `src/effect/composition.rs` |
| **Constraints Layer** | Resource Requirements | `ResourceValidator` | `causality-core` | `src/effect/validator/resource.rs` |
| | Capability Requirements | `CapabilityValidator` | `causality-core` | `src/effect/validator/capability.rs` |
| | Cross-Domain Validation | `DomainValidator` | `causality-core` | `src/effect/validator/domain.rs` |
| **Domain Layer** | Domain Adapters | `DomainAdapter` trait | `causality-types` | `src/domain/adapter.rs` |
| | Effect Handlers | `EffectHandler` trait | `causality-core` | `src/effect/handler.rs` |
| | ZK Integration | `ZkEffect` trait | `causality-types` | `src/effect/zk.rs` |

## Capability System

| Concept | Implementation | Crate | Path |
|---------|---------------|-------|------|
| Capability | `Capability` trait | `causality-types` | `src/capability/mod.rs` |
| Capability Type | `CapabilityType` enum | `causality-types` | `src/capability/type.rs` |
| Capability Registry | `CapabilityRegistry` trait | `causality-types` | `src/capability/registry.rs` |
| Capability Validation | `CapabilityValidator` struct | `causality-core` | `src/capability/validator.rs` |
| Capability Grant | `CapabilityGrant` struct | `causality-types` | `src/capability/grant.rs` |
| Capability Delegation | `delegate_capability` function | `causality-core` | `src/capability/delegation.rs` |
| Capability Chain | `CapabilityChain` struct | `causality-core` | `src/capability/chain.rs` |

## Time System

| Concept | Implementation | Crate | Path |
|---------|---------------|-------|------|
| Timestamp | `Timestamp` struct | `causality-types` | `src/time/timestamp.rs` |
| TimeRange | `TimeRange` struct | `causality-types` | `src/time/range.rs` |
| Clock | `Clock` trait | `causality-types` | `src/time/clock.rs` |
| SystemClock | `SystemClock` struct | `causality-core` | `src/time/system_clock.rs` |
| MockClock | `MockClock` struct | `causality-core` | `src/time/mock_clock.rs` |
| TimeMap | `TimeMap` struct | `causality-types` | `src/time/time_map.rs` |
| TimeEvents | `TimeEvent` enum | `causality-types` | `src/time/events.rs` |

## Content Addressing

| Concept | Implementation | Crate | Path |
|---------|---------------|-------|------|
| ContentHash | `ContentHash` struct | `causality-types` | `src/content/hash.rs` |
| ContentHasher | `ContentHasher` trait | `causality-types` | `src/content/hasher.rs` |
| ContentAddressed | `ContentAddressed` trait | `causality-types` | `src/content/addressed.rs` |
| ContentStore | `ContentStore` trait | `causality-types` | `src/content/store.rs` |
| HashAlgorithm | `HashAlgorithm` enum | `causality-types` | `src/content/algorithm.rs` |
| IPFS Integration | `IpfsContentStore` struct | `causality-core` | `src/content/ipfs.rs` |
| ContentID | `ContentId` struct | `causality-types` | `src/content/id.rs` |

## Agent-Based Resources

| Concept | Implementation | Crate | Path |
|---------|---------------|-------|------|
| Agent | `Agent` trait | `causality-types` | `src/agent/mod.rs` |
| Agent ID | `AgentId` struct | `causality-types` | `src/agent/id.rs` |
| User Agent | `UserAgent` struct | `causality-agent` | `src/agent/user.rs` |
| Operator Agent | `OperatorAgent` struct | `causality-agent` | `src/agent/operator.rs` |
| Committee Agent | `CommitteeAgent` struct | `causality-agent` | `src/agent/committee.rs` |
| Agent Registry | `AgentRegistry` trait | `causality-types` | `src/agent/registry.rs` |
| Agent Status | `AgentStatus` enum | `causality-types` | `src/agent/status.rs` |
| Agent Capabilities | `AgentCapability` trait | `causality-agent` | `src/agent/capability.rs` |

## Domain System

| Concept | Implementation | Crate | Path |
|---------|---------------|-------|------|
| Domain | `Domain` trait | `causality-types` | `src/domain/mod.rs` |
| Domain ID | `DomainId` struct | `causality-types` | `src/domain/id.rs` |
| Domain Adapter | `DomainAdapter` trait | `causality-types` | `src/domain/adapter.rs` |
| Domain Registry | `DomainRegistry` trait | `causality-types` | `src/domain/registry.rs` |
| Ethereum Domain | `EthereumDomain` struct | `causality-domain` | `src/domain/ethereum/mod.rs` |
| CosmWasm Domain | `CosmWasmDomain` struct | `causality-domain` | `src/domain/cosmwasm/mod.rs` |
| Local Domain | `LocalDomain` struct | `causality-domain` | `src/domain/local/mod.rs` |
| Domain Bridge | `DomainBridge` trait | `causality-types` | `src/domain/bridge.rs` |

## Cross-Cutting Concerns

| Concept | Implementation | Crate | Path |
|---------|---------------|-------|------|
| Error Handling | Error types | Multiple | `src/error.rs` in each crate |
| Serialization | Serde implementations | Multiple | Throughout codebase |
| Logging | Log implementations | Multiple | Throughout codebase |
| Configuration | Config structs | Multiple | `src/config` in each crate |
| Testing Utilities | Test helpers | Multiple | `src/test_utils` in each crate |

## Key Interfaces

This section details important interfaces that bridge between architectural components:

### Resource System ↔ Effect System

| Interface | Description |
|-----------|-------------|
| `Effect::resources()` | Effects declare which resources they need |
| `ResourceValidator` | Validates that resources exist and are accessible |
| `ResourceAccessor` | Domain-specific access to resources |

### Effect System ↔ Domain System

| Interface | Description |
|-----------|-------------|
| `DomainAdapter::handle_effect()` | Domains handle effects through adapters |
| `CrossDomainEffect` | Interface for effects that operate across domains |
| `DomainValidator` | Validates cross-domain effect constraints |

### Capability System ↔ Effect System

| Interface | Description |
|-----------|-------------|
| `Effect::required_capabilities()` | Effects declare their required capabilities |
| `CapabilityValidator` | Validates agent has required capabilities |
| `EffectContext::agent_id()` | Provides agent identity during effect execution |

### Agent System ↔ Capability System

| Interface | Description |
|-----------|-------------|
| `CapabilityRegistry::verify_capability()` | Verifies agent has capability |
| `CapabilityGrant::agent_id` | Capabilities are granted to specific agents |
| `delegate_capability()` | Agents can delegate capabilities to other agents |

## Practical Examples

### Example 1: Asset Transfer Effect Flow

The following shows how an asset transfer flows through the system:

1. Client code creates a `TransferAssetEffect` (Algebraic Layer)
2. Effect is submitted to the `EffectEngine` with an `EffectContext`
3. `EffectValidator` validates resource and capability requirements (Constraints Layer)
4. `EffectEngine` routes the effect to appropriate domain adapter
5. Domain adapter's `AssetEffectHandler` executes the transfer (Domain Layer)
6. Result is passed back through continuation chain
7. Client receives final result

```rust
// Client code example showing the flow
let effect = transfer_asset(
    source_account,
    destination_account,
    asset_id,
    amount,
    |result| format!("Transferred {} units", result.amount)
);

let context = EffectContext::new()
    .with_agent_id(agent_id)
    .with_timestamp(current_time);

let result = effect_engine.execute(effect, context).await?;
```

### Example 2: Cross-Domain Effect Flow

For a cross-domain transfer:

1. Client creates a `CrossDomainTransferEffect`
2. Effect declares multiple domains via `involved_domains()`
3. Effect is validated by `EffectValidator`
4. `DomainValidator` specifically checks cross-domain constraints
5. `EffectEngine` orchestrates execution across multiple domains
6. Domain bridges handle cross-domain proof generation and verification
7. Result combines outcomes from both domains

```rust
// Cross-domain effect example
let effect = cross_domain_transfer(
    ethereum_account,
    cosmwasm_account,
    token_id,
    amount,
    identity
);

// Engine handles the complexity of cross-domain execution
let result = effect_engine.execute(effect, context).await?;
```

## Further Reading

- [Three-Layer Effect Architecture](../architecture/core/three-layer-effect-architecture.md)
- [Resource System](../architecture/core/resource-system.md)
- [Effect System](../architecture/core/effect-system.md)
- [Domain System](../architecture/core/domain-system.md)
- [Capability System](../architecture/core/capability-system.md)
- [API Reference](libraries/causality-core.md) 