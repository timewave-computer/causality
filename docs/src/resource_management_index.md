# Resource Management Documentation Index

This index provides an overview of the documentation for the unified resource management system in the Causality framework.

## Core Documentation

- [**Unified ResourceRegister Model**](resource_register_unified_model.md) - Comprehensive documentation of the core resource management model, including lifecycle and relationship management
- [**Architecture Design Decisions**](architecture_design_decisions.md) - Detailed explanation of the architectural decisions behind the resource management system

## Examples

- [**Relationship Tracking Example**](examples/relationship_tracking_example.md) - Example showing how to use relationship tracking for dependency management in a cross-chain asset transaction scenario
- [**Lifecycle Management Example**](examples/lifecycle_management_example.md) - Example illustrating how to implement a document workflow system with custom states and transitions
- [**Integrated Resource Management**](examples/integrated_resource_management_example.md) - Example demonstrating how to combine lifecycle management and relationship tracking in a data processing pipeline

## Implementation

The resource management system consists of several core components:

1. **ResourceRegister** - The central component for resource management
2. **ResourceRegisterLifecycleManager** - Manages resource states and transitions
3. **RelationshipTracker** - Tracks relationships between resources
4. **StorageStrategies** - Provides different storage backends for resources

## Key Concepts

### Resource Lifecycle States

Resources in the system can be in one of the following states:

- **Pending** - Initial state after registration
- **Active** - Resource is active and available for use
- **Locked** - Resource is temporarily locked for modifications
- **Frozen** - Resource is immutable but can be read
- **Consumed** - Resource has been used and is no longer available
- **Archived** - Resource is preserved for historical purposes
- **Inactive** - Resource is not currently active

### Relationship Types

The system supports various types of relationships between resources:

- **Parent-Child** - Hierarchical relationships
- **Dependencies** - Resources that depend on other resources
- **Custom** - User-defined relationship types for domain-specific needs

### State Transitions

The lifecycle manager enforces valid state transitions and records transition history. Some key transitions include:

- Pending → Active (Activation)
- Active → Locked (Locking for processing)
- Active → Frozen (Freezing to prevent changes)
- Active/Frozen → Consumed (Consumption after use)
- Any state → Archived (Archiving for historical record)

## Integration Points

The resource management system integrates with other components of the Causality framework:

- **Effect System** - Resource operations are implemented as effects
- **Capability System** - Authorization for resource operations
- **TEL Integration** - Scripting interface for resource management
- **Domain Adaptations** - Chain-specific resource implementations

## Usage Guidelines

When using the resource management system, follow these best practices:

1. **Define clear resource lifecycles** - Map your domain states to the system's lifecycle states
2. **Use relationships for dependencies** - Model dependencies explicitly using relationships
3. **Validate state transitions** - Ensure transitions are valid for your domain
4. **Record metadata during transitions** - Capture relevant information during state changes
5. **Use appropriate storage strategies** - Choose storage backends based on your requirements

## Related Documentation

- [Effect Templates Documentation](effect_templates.md)
- [Capability System Documentation](capability_system.md)
- [TEL Integration Guide](tel_integration.md)

## API Documentation

For detailed API documentation, see the Rust API docs for the `resource` module. 