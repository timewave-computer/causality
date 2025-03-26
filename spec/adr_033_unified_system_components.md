# ADR-033: Unified System Components and Interactions

## Status
Accepted

## Context
The Causality system introduces a number of modular components to manage execution, authorization, resource interaction, and communication. To ensure clarity and long-term maintainability, we are aligning around a unified architectural model that clearly separates concerns, defines the role of each component, and highlights how they interact to form a coherent system.

## Decision
We will adopt a system architecture centered on the following major components:

- **Resources**: Represent domain entities and stateful objects
- **Capabilities**: Grant precise, contextual authority to interact with resources
- **Effects**: Abstract, composable actions that can change system state
- **Operations**: Requests to perform effects with authorization
- **Agents**: Entities that hold capabilities and perform operations
- **Service Status**: Signals that agents are actively offering services
- **Obligation Manager**: Enforces usage-based expectations on capabilities
- **Messaging**: Enables asynchronous interaction between agents
- **Fact System**: Tracks temporal and logical dependencies between actions

This ADR defines the role of each component, how they are modeled, and how they interrelate in the system.

## System Component Overview

| **Component**           | **Description**                                                                 | **Modeled As**                          | **Governed By / Consumes**           |
|-------------------------|----------------------------------------------------------------------------------|------------------------------------------|--------------------------------------|
| **Resource**            | Stateful object with a lifecycle and metadata                                   | `Resource`                               | `ResourceLogic`, `Effect`            |
| **Resource Logic**      | Defines valid transitions and behaviors per resource type                       | Trait or module per type                 | Invoked by `Effect` execution        |
| **Capability**          | Token of authority to perform rights on a target resource                       | `Capability`                             | `CapabilityRegistry`                 |
| **Capability Registry** | Issues, validates, revokes, and delegates capabilities                         | `CapabilityRegistry`                     | Used by `Authorization`, `ObligationManager` |
| **Capability Constraint**| Defines time, usage, or state restrictions on capability usage                  | Struct in `Capability`                   | Validated during operation           |
| **Operation**           | Request to perform one or more effects using presented capabilities             | `Operation`                              | `Authorization`, `Effect`            |
| **Effect**              | Abstract, executable side effect                                                | `Effect` trait                           | Executed by `Interpreter`            |
| **Interpreter**         | Runtime for executing effects with effect handlers and context                 | `Interpreter`                            | Uses `EffectHandler`, `ExecutionContext` |
| **Authorization**       | Proof that the agent has valid capabilities to execute an operation            | `Authorization`                          | Validated before operation execution |
| **Execution Context**   | Encapsulates environment, clock, domain, and metadata                          | `ExecutionContext`                       | Passed to `Effect`, `Validator`      |
| **Fact System**         | Tracks logical and temporal relationships between effects                      | `FactSnapshot`, `FactDependency`         | Used in `Effect` validation          |
| **Service Status**      | Declares that an agent is offering a capability-backed service                 | `Resource::ServiceStatus`                | Governed by service-advertising effects |
| **Obligation Manager**  | Monitors usage and enforces duty-based expiration policies                     | `ObligationManager`                      | Uses logs, revokes via registry      |
| **Message**             | Asynchronous, secure, structured communication between agents                  | `Resource::Message`                      | Sent via message-related capabilities |
| **Agent Profile**       | Identity and metadata container for agents in the system                       | `Resource::AgentProfile`                 | Holds capabilities, service state    |
| **Capability Bundle**   | Reusable template for issuing predefined capability sets                       | `CapabilityBundle`                       | Used to instantiate agents or roles  |

## Component Interaction Model

The Causality system revolves around the following interactions:

1. **Agents** are modeled as resources and hold **capabilities** that grant them rights over other resources.
2. An agent submits an **operation**, which includes:
   - One or more **effects** representing the abstract actions to perform
   - An **authorization** proof using held capabilities
3. The **interpreter** validates and executes effects, optionally consulting:
   - **Resource logic** for domain-specific behaviors
   - The **fact system** to ensure temporal constraints are upheld
   - The **execution context** for runtime conditions (e.g. domain, time)
4. **Service Status** declares that an agent is online and ready to offer specific services (backed by capabilities)
5. **Obligation Manager** enforces expectations that capabilities are actively used or delegated
6. **Messages** are used for capability negotiation, cross-agent coordination, or service announcements

## Separation of Concerns

| **Subsystem**            | **Primary Responsibility**                                           | **Collaborates With**                                 |
|--------------------------|-----------------------------------------------------------------------|--------------------------------------------------------|
| **Authorization**        | Confirms that capability proofs are valid for operations              | `CapabilityRegistry`, `Operation`, `Effect`            |
| **Execution**            | Executes abstract effects and manages state transitions               | `Effect`, `Interpreter`, `ResourceLogic`               |
| **Access Control**       | Issues, tracks, delegates, and revokes capabilities                   | `CapabilityRegistry`, `CapabilityConstraint`           |
| **Service Management**   | Declares available services from capable agents                       | `Service Status`, `AgentProfile`                       |
| **Duty Enforcement**     | Ensures capabilities are actively used                               | `ObligationManager`, `AuditLog`, `CapabilityRegistry`  |
| **Communication**        | Manages agent-to-agent messaging and coordination                     | `Message`, `Authorization`, `Effect`                   |
| **Temporal Validation**  | Ensures causal ordering and dependency tracking                       | `FactSystem`, `Effect`, `ExecutionContext`             |
| **Agent Identity**       | Tracks ownership, metadata, and service state of agents               | `AgentProfile`, `Service Status`, `CapabilityBundle`   |

## Crate-Level Architecture

The relationship between `causality-core` and `causality-effects` is key to modularity:

- `causality-core` defines **the `Operation` type**, which is the unit of execution submitted by agents.
- `Operation` references an abstract `Effect`, which is defined in `causality-effects`.
- `causality-effects` provides the `Effect` trait, base effect types, and the `Interpreter` that can validate and execute effects.

This separation allows:
- `causality-core` to remain free of execution semantics
- `causality-effects` to evolve independently with domain-specific or pluggable behaviors

While it’s possible to move `Operation` into `causality-effects`, keeping it in `core` ensures:
- The data model of an operation is accessible across the system
- Execution logic remains decoupled from representation and authorization

Thus, `Operation` lives in `causality-core`, and its embedded `Effect` is an opaque trait object handled at runtime by `causality-effects`.

To support modularity while avoiding fragmentation, the system will be divided into the following Rust crates:

### Crate Layout

| **Crate Name**            | **Responsibility**                                                                 | **Depends On**                                       |
|---------------------------|------------------------------------------------------------------------------------|------------------------------------------------------|
| `causality-types`         | Shared types and interfaces: IDs, timestamps, trait definitions                   | —                                                    |
| `causality-core`          | Resources, capabilities, operations (references to `Effect`)                      | `causality-types`                                    |
| `causality-effects`       | Effect trait, interpreter, execution context, and built-in effect types           | `causality-core`                                     |
| `causality-agent`         | Agent profiles, service status, obligation manager, capability bundles            | `causality-core`                                     |
| `causality-engine`        | Top-level orchestration: effect validation, operation execution, routing          | All of the above                                     |

- `causality-effects` is split out to allow isolated effect development and support multiple interpreters.
- `causality-engine` glues everything together, enabling simulation, validation, and execution.

## Consequences

- System behavior is defined through a clean pipeline of resource-effect-capability-operation.
- All coordination (presence, duty, messaging) is unified under the resource + effect model.
- No ambient authority; all actions require possession of appropriate capabilities.
- Messaging, availability, and obligations interoperate as first-class system components.
- Effect handlers and resource logic remain modular, enabling domain-specific extensions.

## Future Considerations

- Consider building a schema-based diagram to visualize runtime flow between components.
- Introduce declarative roles or behavior profiles derived from capability bundles.
- Explore protocol-level extensions for federated capability, messaging, or ZK execution.

