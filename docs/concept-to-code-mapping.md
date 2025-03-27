# Concept to Code Mapping

*This document maps architectural concepts to their implementations in the codebase.*

*Last updated: 2023-08-20*

## Core Systems

| **Architectural Concept** | **Crate** | **Module/File** | **Primary Types** |
|---------------------------|-----------|----------------|------------------|
| Resource System | `causality-core` | `causality_core::resource` | `Resource`, `ResourceId`, `ResourceAccessor` |
| Effect System | `causality-core` | `causality_core::effect` | `Effect`, `EffectHandler`, `EffectOutcome` |
| Capability System | `causality-core` | `causality_core::capability` | `Capability`, `CapabilityRegistry`, `CapabilityConstraint` |
| Content Addressing | `causality-core` | `causality_core::content` | `ContentAddressed`, `ContentHash`, `ContentHasher` |
| Time System | `causality-core` | `causality_core::time` | `TimeMap`, `Clock`, `TemporalFact` |
| Agent-Based Resources | `causality-core` | `causality_core::agent` | `Agent`, `AgentType`, `AgentState` |
| Operation System | `causality-core` | `causality_core::operation` | `Operation`, `OperationResult`, `Authorization` |

## Integration and Execution

| **Architectural Concept** | **Crate** | **Module/File** | **Primary Types** |
|---------------------------|-----------|----------------|------------------|
| Effect Interpreter | `causality-core` | `causality_core::effect::interpreter` | `Interpreter`, `InterpreterContext`, `ExecutionEnvironment` |
| Resource Concurrency | `causality-core` | `causality_core::resource::concurrency` | `ResourceLockManager`, `ResourceGuard`, `AccessMode` |
| Domain Adapters | `causality-domain` | `causality_domain::adapters` | `DomainAdapter`, `DomainRegistry`, `DomainConnection` |
| Cross-Domain Operations | `causality-domain` | `causality_domain::crossing` | `DomainCrossing`, `CrossDomainProof`, `DomainBridge` |
| Engine | `causality-engine` | `causality_engine` | `Engine`, `EngineConfig`, `EngineState` |

## Domain Adapters

| **Architectural Concept** | **Crate** | **Module/File** | **Primary Types** |
|---------------------------|-----------|----------------|------------------|
| Ethereum Domain | `causality-domain-ethereum` | `causality_domain_ethereum` | `EthereumDomainAdapter`, `EthereumTransaction`, `EthereumEvent` |
| CosmWasm Domain | `causality-domain-cosmwasm` | `causality_domain_cosmwasm` | `CosmWasmDomainAdapter`, `CosmWasmContract`, `CosmWasmMessage` |
| Local Domain | `causality-domain-local` | `causality_domain_local` | `LocalDomainAdapter`, `LocalStorage`, `LocalOperation` |

## Agent System

| **Architectural Concept** | **Crate** | **Module/File** | **Primary Types** |
|---------------------------|-----------|----------------|------------------|
| User Agent | `causality-agent` | `causality_agent::user` | `User`, `UserProfile`, `UserAccessor` |
| Committee Agent | `causality-agent` | `causality_agent::committee` | `Committee`, `ValidatorInfo`, `CommitteeAccessor` |
| Operator Agent | `causality-agent` | `causality_agent::operator` | `Operator`, `OperatorAccessor`, `OperatorConfig` |
| Service Status | `causality-agent` | `causality_agent::service` | `ServiceStatus`, `ServiceDiscovery`, `ServiceAnnouncement` |
| Capability Bundles | `causality-agent` | `causality_agent::capability` | `CapabilityBundle`, `BundleTemplate`, `RoleDefinition` |

## Programs

| **Architectural Concept** | **Crate** | **Module/File** | **Primary Types** |
|---------------------------|-----------|----------------|------------------|
| Program Definition | `causality-program` | `causality_program::definition` | `Program`, `ProgramId`, `ProgramLogic` |
| Program Execution | `causality-program` | `causality_program::execution` | `ProgramExecutor`, `ExecutionContext`, `ExecutionResult` |
| Program Storage | `causality-program` | `causality_program::storage` | `ProgramStorage`, `ProgramRegistrar`, `ProgramMetadata` |

## Zero-Knowledge Components

| **Architectural Concept** | **Crate** | **Module/File** | **Primary Types** |
|---------------------------|-----------|----------------|------------------|
| ZK Execution | `causality-zkvm` | `causality_zkvm::execution` | `ZkVm`, `ZkExecution`, `ZkProof` |
| ZK Verification | `causality-zkvm` | `causality_zkvm::verification` | `ZkVerifier`, `VerificationKey`, `VerificationResult` |
| Circuit Generation | `causality-zkvm` | `causality_zkvm::circuit` | `CircuitGenerator`, `CircuitDefinition`, `CircuitCompiler` |

## Resource Concurrency

| **Architectural Concept** | **Crate** | **Module/File** | **Primary Types** |
|---------------------------|-----------|----------------|------------------|
| Resource Locking | `causality-core` | `causality_core::resource::concurrency` | `ResourceLockManager`, `ResourceGuard`, `LockStrategy` |
| Lock Wait Queues | `causality-core` | `causality_core::resource::concurrency::queue` | `WaitQueue`, `WaitQueueEntry`, `QueuePolicy` |
| Deadlock Detection | `causality-core` | `causality_core::resource::concurrency::deadlock` | `DeadlockDetector`, `WaitForGraph`, `CycleDetection` |

## Notes on Crate Structure

The codebase follows the unified architecture approach from ADR-032:

1. `causality-types`: Core type definitions and traits
2. `causality-core`: Integrated core functionality (resources, capabilities, effects, agents)
3. `causality-agent`: Agent-specific functionality building on core
4. `causality-domain`: Domain-specific adapters and integrations
5. `causality-engine`: Orchestration layer that ties everything together

The effect system is integrated directly into `causality-core` rather than being a separate crate, enabling tighter integration with resources and capabilities while reducing cross-crate dependencies.
