# Causality API Documentation

This index provides a comprehensive overview of the API documentation available for the Causality framework.

## Core Components

- [**ResourceRegister API**](resource_register_api.md) - Documentation for the unified resource management system
- [**Effect System API**](effect_system_api.md) - Documentation for the three-layer effect architecture
- [**Capability System API**](capability_system_api.md) - Documentation for the capability-based authorization system
- [**TEL (Temporal Effect Language) API**](tel_api.md) - Documentation for the scripting language interface

## Resource Management

The ResourceRegister API includes documentation for:

- Resource lifecycle management
- Resource relationship tracking
- Storage strategies
- Integration with the effect system

## Effect System

The Effect System API includes documentation for:

- Foundation layer effects
- Domain-specific effects
- Composite effects
- Effect constraints and validation
- Effect composition (sequential, parallel, conditional)
- Effect templates

## Capability System

The Capability System API includes documentation for:

- Capability types and repository
- Capability validation and constraints
- Capability delegation and revocation
- Integration with authorization systems
- Capability proofs for cross-system authorization

## TEL (Temporal Effect Language)

The TEL API includes documentation for:

- Script parser and representations
- Handler interfaces
- Chain-specific handlers (EVM, CosmWasm)
- Integration with effect execution

## Integration Points

- [**Storage Integration**](storage_integration.md) - How the storage system integrates with other components
- [**Effect Integration**](effect_integration.md) - How effects are integrated throughout the system
- [**TEL Integration**](tel_integration.md) - How TEL scripts integrate with the effect system
- [**Capability Integration**](capability_integration.md) - How capabilities integrate with authorization

## Getting Started

For getting started with the Causality framework, please refer to the following resources:

- [Causality Overview](../overview.md)
- [Getting Started Guide](../getting_started.md)
- [Usage Examples](../examples/index.md)

## Implementation Notes

For detailed implementation notes and design decisions, please refer to:

- [Architecture Design Decisions](../architecture_design_decisions.md)
- [Unified ResourceRegister Model](../resource_register_unified_model.md) 