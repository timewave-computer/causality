# Architectural Decision Records (ADRs) Index

This document serves as an index for the canonical Architectural Decision Records (ADRs) maintained in the [spec/](../../../spec/) directory.

## What is an ADR?

An Architectural Decision Record (ADR) is a document that captures an important architectural decision, including the context, the decision itself, and the consequences.

## ADRs by Topic

The following is a list of all ADRs, organized by topic and showing relationships between them.

### Time System

- [ADR-000: Time Model](../../../spec/adr_000_time.md) - Defines the core time model for Causality
- [ADR-024: Fact and Time Map Unification](../../../spec/adr_024_fact_time_map_unification.md) (supersedes parts of ADR-000) - Unifies fact management with the time system

### Effect System

- [ADR-001: Effects](../../../spec/adr_001_effects.md) - Establishes the core effect system
- [ADR-002: Effect Adapters](../../../spec/adr_002_effect_adapters.md) - Defines effect adapters for cross-domain effects
- [ADR-023: Domain Adapter and Effect Handler Unification](../../../spec/adr_023_domain_adapter_effect_handler_unification.md) (extends ADR-001 and ADR-002) - Unifies domain adapters with effect handlers
- [ADR-031: Domain Adapter as Effect](../../../spec/adr_031_domain_adapter_as_effect.md) (supersedes parts of ADR-023) - Treats domain adapters as effects themselves
- [ADR-033: Unified System Components](../../../spec/adr_033_unified_system_components.md) (extends ADR-031 and ADR-032) - Integrates effects directly into the core crate

### Resource System

- [ADR-003: Resource](../../../spec/adr_003_resource.md) - Defines the core resource system
- [ADR-021: Resource Register Unification](../../../spec/adr_021_resource_register_unification.md) (extends ADR-003) - Unifies resource management with registers
- [ADR-030: Resource Accessor Pattern](../../../spec/adr_030_deffered_hashing_out_of_vm.md) (extends ADR-021) - Introduces the resource accessor pattern and deferred hashing
- [ADR-032: Agent-Based Resource System](../../../spec/adr_032_consolidated_agent_resource_system.md) (supersedes parts of ADR-003) - Redefines resources with agent-based model

### Agent-Based Resource System

- [ADR-005: Invocation Model](../../../spec/adr_005_invocation.md) - Original definition of actors in the system
- [ADR-032: Agent-Based Resource System](../../../spec/adr_032_consolidated_agent_resource_system.md) (supersedes parts of ADR-005) - Redefines actors as resources within the resource system
- [ADR-033: Unified System Components](../../../spec/adr_033_unified_system_components.md) (extends ADR-032) - Defines the unified architecture of system components

### Content Addressing

- [ADR-007: Content Addressing](../../../spec/adr_007_content_addressing.md) - Defines the content addressing model
- [ADR-028: Universal Content Addressing](../../../spec/adr_028_universal_content_addressing.md) (extends ADR-007) - Expands content addressing to all objects
- [ADR-029: SMT Integration](../../../spec/adr_029_smt_integration.md) (extends ADR-028) - Integrates Sparse Merkle Trees with content addressing
- [ADR-030: Deferred Hashing Out of VM](../../../spec/adr_030_deffered_hashing_out_of_vm.md) (extends ADR-028) - Moves hashing operations outside the VM

### Concurrency

- [ADR-004: Concurrency](../../../spec/adr_004_concurrency.md) - Defines the concurrency model
- [ADR-032: Agent-Based Resource System](../../../spec/adr_032_consolidated_agent_resource_system.md) (extends ADR-004) - Enhances concurrency with agent-based resources
- [ADR-033: Unified System Components](../../../spec/adr_033_unified_system_components.md) (extends ADR-004) - Defines resource locking with unified architecture

### Fact Management

- [ADR-008: Fact Management](../../../spec/adr_008_fact_management.md) - Establishes fact management
- [ADR-024: Fact and Time Map Unification](../../../spec/adr_024_fact_time_map_unification.md) (extends ADR-008) - Unifies facts with time maps

### Logging and Reconstruction

- [ADR-009: Log](../../../spec/adr_009_log.md) - Defines the logging system
- [ADR-010: Log Reconstruction](../../../spec/adr_010_log_reconstruction.md) (extends ADR-009) - Adds log reconstruction capabilities

### Schema and Program Management

- [ADR-011: Schema Evolution](../../../spec/adr_011_schema_evolution.md) - Handles schema evolution
- [ADR-012: AST Resource Graph](../../../spec/adr_012_ast_resource_graph.md) - Defines AST resources
- [ADR-013: TEL](../../../spec/adr_013_tel.md) - Timewave Expression Language
- [ADR-014: Compiler](../../../spec/adr_014_compiler.md) - Program compilation system
- [ADR-026: Program and Program Account Unification](../../../spec/adr_026_program_program_account_unification.md) - Unifies programs with program accounts

### Deployment and Operations

- [ADR-015: Geo Distributed](../../../spec/adr_015_geo_distributed.md) - Geo-distribution architecture
- [ADR-019: Deployer](../../../spec/adr_019_deployer.md) - Deployment system
- [ADR-020: Observer](../../../spec/adr_020_observer.md) - System observation

### Domain Adaptation

- [ADR-016: Network Adapter](../../../spec/adr_016_network_adapter.md) - Network adapters
- [ADR-018: Domain Adapter](../../../spec/adr_018_domain_adapter.md) - Domain adapter model
- [ADR-023: Domain Adapter and Effect Handler Unification](../../../spec/adr_023_domain_adapter_effect_handler_unification.md) (extends ADR-018) - Unifies domain adapters with effect handlers
- [ADR-031: Domain Adapter as Effect](../../../spec/adr_031_domain_adapter_as_effect.md) (supersedes parts of ADR-023) - Treats domain adapters as effects

### Security and Verification

- [ADR-006: ZK Registers](../../../spec/adr_006_zk_registers.md) - Zero-knowledge registers
- [ADR-022: Permissioning Unification](../../../spec/adr_022_permissioning_unification.md) - Unified permission model
- [ADR-027: Unified Verification](../../../spec/adr_027_unified_verification.md) - Verification system

### Simulation

- [ADR-017: Simulation System](../../../spec/adr_017_simulation_system.md) - System simulation capabilities

### Unified Operation Model

- [ADR-025: Unified Operation Model](../../../spec/adr_025_unified_operation_model.md) - Defines a unified approach to operations
- [ADR-033: Unified System Components](../../../spec/adr_033_unified_system_components.md) (extends ADR-025) - Enhances operations with unified architecture

## Chronological List of All ADRs

| Number | Title | Status | Superseded By |
|--------|-------|--------|--------------|
| 000 | [Time Model](../../../spec/adr_000_time.md) | Active | Partially by ADR-024 |
| 001 | [Effects](../../../spec/adr_001_effects.md) | Active | Extended by ADR-033 |
| 002 | [Effect Adapters](../../../spec/adr_002_effect_adapters.md) | Active | Partially by ADR-023, Extended by ADR-033 |
| 003 | [Resource](../../../spec/adr_003_resource.md) | Active | Partially by ADR-021, Superseded by ADR-032 |
| 004 | [Concurrency](../../../spec/adr_004_concurrency.md) | Active | Extended by ADR-032 and ADR-033 |
| 005 | [Invocation Model](../../../spec/adr_005_invocation.md) | Active | Superseded by ADR-032 |
| 006 | [ZK Registers](../../../spec/adr_006_zk_registers.md) | Active | - |
| 007 | [Content Addressing](../../../spec/adr_007_content_addressing.md) | Active | Extended by ADR-028 |
| 008 | [Fact Management](../../../spec/adr_008_fact_management.md) | Active | Extended by ADR-024 |
| 009 | [Log](../../../spec/adr_009_log.md) | Active | Extended by ADR-010 |
| 010 | [Log Reconstruction](../../../spec/adr_010_log_reconstruction.md) | Active | - |
| 011 | [Schema Evolution](../../../spec/adr_011_schema_evolution.md) | Active | - |
| 012 | [AST Resource Graph](../../../spec/adr_012_ast_resource_graph.md) | Active | - |
| 013 | [TEL](../../../spec/adr_013_tel.md) | Active | - |
| 014 | [Compiler](../../../spec/adr_014_compiler.md) | Active | - |
| 015 | [Geo Distributed](../../../spec/adr_015_geo_distributed.md) | Active | - |
| 016 | [Network Adapter](../../../spec/adr_016_network_adapter.md) | Active | - |
| 017 | [Simulation System](../../../spec/adr_017_simulation_system.md) | Active | - |
| 018 | [Domain Adapter](../../../spec/adr_018_domain_adapter.md) | Active | Superseded by ADR-031 |
| 019 | [Deployer](../../../spec/adr_019_deployer.md) | Active | - |
| 020 | [Observer](../../../spec/adr_020_observer.md) | Active | - |
| 021 | [Resource Register Unification](../../../spec/adr_021_resource_register_unification.md) | Active | Extended by ADR-030 |
| 022 | [Permissioning Unification](../../../spec/adr_022_permissioning_unification.md) | Active | - |
| 023 | [Domain Adapter and Effect Handler Unification](../../../spec/adr_023_domain_adapter_effect_handler_unification.md) | Active | Partially by ADR-031 |
| 024 | [Fact and Time Map Unification](../../../spec/adr_024_fact_time_map_unification.md) | Active | - |
| 025 | [Unified Operation Model](../../../spec/adr_025_unified_operation_model.md) | Active | Extended by ADR-033 |
| 026 | [Program and Program Account Unification](../../../spec/adr_026_program_program_account_unification.md) | Active | - |
| 027 | [Unified Verification](../../../spec/adr_027_unified_verification.md) | Active | - |
| 028 | [Universal Content Addressing](../../../spec/adr_028_universal_content_addressing.md) | Active | Extended by ADR-029, ADR-030 |
| 029 | [SMT Integration](../../../spec/adr_029_smt_integration.md) | Active | - |
| 030 | [Deferred Hashing Out of VM](../../../spec/adr_030_deffered_hashing_out_of_vm.md) | Active | - |
| 031 | [Domain Adapter as Effect](../../../spec/adr_031_domain_adapter_as_effect.md) | Active | Extended by ADR-033 |
| 032 | [Agent-Based Resource System](../../../spec/adr_032_consolidated_agent_resource_system.md) | Active | Extended by ADR-033 |
| 033 | [Unified System Components](../../../spec/adr_033_unified_system_components.md) | Active | - |

This index includes all ADRs as of the last update. For the most current list, please check the [spec/](../../../spec/) directory.

## Visual Relationship Diagram

```
+----------------+        +----------------+        +----------------+
| ADR-000        |<-------| ADR-024        |------->| ADR-008        |
| Time Model     |        | Fact & Time    |        | Fact Management|
+----------------+        | Unification    |        +----------------+
                          +----------------+

+----------------+        +----------------+        +----------------+
| ADR-001        |<-------| ADR-023        |<-------| ADR-018        |
| Effects        |        | Domain & Effect|        | Domain Adapter |
+----------------+        | Unification    |        +----------------+
       ^                  +----------------+
       |                         ^
       |                         |
       |                  +----------------+        +----------------+
       |                  | ADR-002        |        | ADR-031        |
       |                  | Effect Adapters|<-------| Domain as      |
       |                  +----------------+        | Effect         |
       |                                            +----------------+
       |                                                    ^
       |                                                    |
+----------------+                                   +----------------+
| ADR-033        |<----------------------------------| ADR-032        |
| Unified System |                                   | Agent-Based    |
| Components     |                                   | Resources      |
+----------------+                                   +----------------+
       ^                                                    ^
       |                                                    |
       |                                             +----------------+
       |                                             | ADR-005        |
       |                                             | Invocation     |
       |                                             | Model          |
       |                                             +----------------+
       |
       |                  +----------------+
       |                  | ADR-025        |
       +------------------| Unified        |
                          | Operation      |
                          +----------------+

+----------------+        +----------------+        +----------------+
| ADR-003        |<-------| ADR-021        |<-------| ADR-030        |
| Resource       |        | Resource       |        | Resource       |
|                |        | Register       |        | Accessor       |
+----------------+        +----------------+        +----------------+
       ^
       |
       |
+----------------+
| ADR-032        |
| Agent-Based    |
| Resources      |
+----------------+

+----------------+        +----------------+        +----------------+
| ADR-007        |<-------| ADR-028        |<-------| ADR-029        |
| Content        |        | Universal      |        | SMT Integration|
| Addressing     |        | Content        |        +----------------+
+----------------+        | Addressing     |                ^
                          +----------------+                |
                                  ^                         |
                                  |                         |
                                  |                 +----------------+
                                  +---------------->| ADR-030        |
                                                    | Deferred       |
                                                    | Hashing        |
                                                    +----------------+

+----------------+        +----------------+        +----------------+
| ADR-004        |<-------| ADR-032        |<-------| ADR-033        |
| Concurrency    |        | Agent-Based    |        | Unified System |
|                |        | Resources      |        | Components     |
+----------------+        +----------------+        +----------------+
```

## ADR Lifecycle and Supersession

When an ADR supersedes or extends another, the following principles apply:

1. **Superseded ADRs**: These remain valuable as historical documents showing the evolution of ideas
2. **Extended ADRs**: The core principles remain, but specific aspects are enhanced or expanded
3. **Partial Supersession**: Only specified parts are superseded, while other aspects remain active

For current architectural understanding, always prefer the most recent ADR on a given topic unless there is an addendum added at a later date.
