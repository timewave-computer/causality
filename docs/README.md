# Causality Documentation

*This documentation provides comprehensive information about the Causality system.*

*Last updated: 2023-09-05*

## Overview

Causality is a system for building cross-domain applications with strong causal consistency guarantees. It provides a framework for representing, managing, and executing operations across different domains while maintaining cryptographic verification of all state transitions.

## Documentation Sections

### Architecture

- [Architecture Overview](architecture/README.md): Overview of the system architecture
  - [Resource System](architecture/resource-system.md): Universal framework for resource management
  - [Effect System](architecture/effect-system.md): Framework for modelling and executing effects
  - [Three-Layer Effect Architecture](architecture/three-layer-effect-architecture.md): Detailed design of the effect architecture
  - [Agent-Based Resources](architecture/agent-based-resources.md): Unified model for system agents
  - [Capability System](architecture/capability-system.md): Capability-based access control
  - [Time System](architecture/time-system.md): Causal and clock time management
  - [Content Addressing](architecture/content-addressing.md): Content-addressed storage and verification
  - [Domain System](architecture/domain-system.md): Cross-domain operations and boundary crossing
  - [Domain Model](architecture/domain-model.md): Unified domain adapter model
  - [TEL System](architecture/tel-system.md): Temporal Effect Language for cross-domain operations
  - [Unified System Components](architecture/unified-system-components.md): Integration of system components
  - [Concept-to-Code Mapping](architecture/concept-to-code-mapping.md): Mapping between concepts and implementations
  - [ADR Index](architecture/adr-index.md): Index of Architectural Decision Records

### Guides

- [Getting Started](guides/getting-started.md): Introduction to Causality
- [Implementation Guides](guides/README.md#implementation-guides): Detailed implementation instructions
  - [Resource System](guides/resource-system.md): Implementing resources
  - [Effect System](guides/effect-system.md): Working with effects 
  - [Three-Layer Effect Architecture](guides/three-layer-effect-architecture.md): Implementing with the three-layer effect architecture
  - [Capability System](guides/capability-system.md): Implementing capabilities
  - [Time System](guides/time-system.md): Working with the time system
  - [Agent-Based Resources](guides/agent-based-resources.md): Working with user, operator, and committee agents
  - [Domain System](guides/domain-system.md): Working with domains
  - [Content Addressing](guides/content-addressing.md): Implementing content addressing

### Reference

- [API Reference](reference/api/README.md): API documentation
  - [REST API](reference/api/rest.md): REST API endpoints
  - [CLI](reference/api/cli.md): Command-line interface
- [Libraries](reference/libraries/README.md): Library documentation
- [Data Structures](reference/data-structures/README.md): Key data structures
- [Concept-to-Code Mapping](reference/concept-to-code-mapping.md): Maps architectural concepts to implementation

## Key Concepts

- **Resources**: Stateful objects with lifecycles and metadata
- **Capabilities**: Tokens of authority to perform operations on resources
- **Effects**: Abstract, composable actions that can change system state
- **Operations**: Requests to perform effects with authorization
- **Agents**: Entities that hold capabilities and perform operations
- **Facts**: Records of temporal and logical dependencies between actions
- **Domains**: External systems integrated through domain adapters
- **TEL**: Temporal Effect Language for expressing cross-domain operations

## System Architecture

Causality is built on a unified component architecture where:

1. **Agents** submit **operations** with capabilities
2. **Operations** contain **effects** representing actions to perform
3. **Effects** modify **resources** according to their logic
4. **Capabilities** authorize operations on resources
5. **Time** tracks causal relationships between operations
6. **Content addressing** provides cryptographic verification
7. **TEL** provides a domain-agnostic language for cross-domain operations

## Additional Resources

- [System Contract](../spec/system_contract.md): Formal specification of system guarantees
- [GitHub Repository](https://github.com/causality/causality): Source code and issues
- [Community Discord](https://discord.gg/causality): Community discussions
