# Resource Interfaces in Causality

## Overview

This document details the resource interface model within the Causality architecture. Resource interfaces define the standard ways resources can be accessed, manipulated, and integrated with other components of the system. They establish protocols for interaction, enabling consistent access patterns and interoperability between different resource types and domains.

## Core Concepts

### Interface Definition

A resource interface is a collection of operations, methods, and events that a resource supports:

```rust
pub struct ResourceInterface {
    /// Unique identifier for the interface
    id: InterfaceId,
    
    /// Human-readable name
    name: String,
    
    /// Supported operations
    operations: Vec<InterfaceOperation>,
    
    /// Events that can be emitted
    events: Vec<InterfaceEvent>,
    
    /// Required attributes for implementing resources
    required_attributes: Vec<AttributeDefinition>,
    
    /// Optional attributes for implementing resources
    optional_attributes: Vec<AttributeDefinition>,
    
    /// Interface version
    version: Version,
    
    /// Interface extensions
    extensions: Vec<InterfaceId>,
}
```

### Interface Operations

An interface defines the operations that can be performed on a resource:

```rust
pub struct InterfaceOperation {
    /// Operation name
    name: String,
    
    /// Operation inputs
    inputs: Vec<ParameterDefinition>,
    
    /// Operation outputs
    outputs: Vec<ParameterDefinition>,
    
    /// Required capabilities to execute this operation
    required_capabilities: Vec<CapabilityRequirement>,
    
    /// Operation constraints
    constraints: Vec<OperationConstraint>,
    
    /// Optional operation metadata
    metadata: HashMap<String, String>,
}
```

### Interface Events

Events that can be emitted by resources implementing the interface:

```rust
pub struct InterfaceEvent {
    /// Event name
    name: String,
    
    /// Event data fields
    fields: Vec<ParameterDefinition>,
    
    /// Optional event metadata
    metadata: HashMap<String, String>,
}
```

### Parameter and Attribute Definitions

Parameters and attributes are defined with types and constraints:

```rust
pub struct ParameterDefinition {
    /// Parameter name
    name: String,
    
    /// Parameter type
    param_type: ParameterType,
    
    /// Optional default value
    default_value: Option<Value>,
    
    /// Parameter constraints
    constraints: Vec<ParameterConstraint>,
    
    /// Whether parameter is optional
    optional: bool,
}

pub struct AttributeDefinition {
    /// Attribute name
    name: String,
    
    /// Attribute type
    attr_type: AttributeType,
    
    /// Attribute constraints
    constraints: Vec<AttributeConstraint>,
    
    /// Whether attribute is mutable
    mutable: bool,
}
```

## Interface Registry

Interfaces are registered in a central registry for discoverability:

```rust
pub struct InterfaceRegistry {
    interfaces: HashMap<InterfaceId, ResourceInterface>,
    implementations: HashMap<ResourceType, Vec<InterfaceId>>,
}

impl InterfaceRegistry {
    /// Register a new interface
    pub fn register_interface(&mut self, interface: ResourceInterface) -> Result<(), InterfaceError> {
        if self.interfaces.contains_key(&interface.id) {
            return Err(InterfaceError::AlreadyExists);
        }
        
        // Validate interface definition
        self.validate_interface(&interface)?;
        
        // Add to registry
        self.interfaces.insert(interface.id, interface);
        
        Ok(())
    }
    
    /// Register a resource type as implementing an interface
    pub fn register_implementation(
        &mut self, 
        resource_type: ResourceType, 
        interface_id: InterfaceId
    ) -> Result<(), InterfaceError> {
        if !self.interfaces.contains_key(&interface_id) {
            return Err(InterfaceError::InterfaceNotFound);
        }
        
        self.implementations
            .entry(resource_type)
            .or_insert_with(Vec::new)
            .push(interface_id);
        
        Ok(())
    }
    
    /// Get all interfaces implemented by a resource type
    pub fn get_implemented_interfaces(&self, resource_type: &ResourceType) -> Vec<InterfaceId> {
        self.implementations
            .get(resource_type)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Check if a resource type implements an interface
    pub fn implements_interface(
        &self, 
        resource_type: &ResourceType, 
        interface_id: &InterfaceId
    ) -> bool {
        self.implementations
            .get(resource_type)
            .map(|interfaces| interfaces.contains(interface_id))
            .unwrap_or(false)
    }
    
    // Additional methods...
}
```

## Standard Interfaces

Causality defines several standard interfaces for common resource behaviors:

### Transferable Interface

```rust
pub fn create_transferable_interface() -> ResourceInterface {
    ResourceInterface {
        id: InterfaceId::from_str("causality:interface:transferable:v1"),
        name: "Transferable".to_string(),
        operations: vec![
            InterfaceOperation {
                name: "transfer".to_string(),
                inputs: vec![
                    ParameterDefinition {
                        name: "to".to_string(),
                        param_type: ParameterType::AccountId,
                        default_value: None,
                        constraints: vec![],
                        optional: false,
                    },
                    ParameterDefinition {
                        name: "amount".to_string(),
                        param_type: ParameterType::U64,
                        default_value: None,
                        constraints: vec![
                            ParameterConstraint::MinValue(Value::U64(1)),
                        ],
                        optional: false,
                    },
                ],
                outputs: vec![
                    ParameterDefinition {
                        name: "success".to_string(),
                        param_type: ParameterType::Bool,
                        default_value: None,
                        constraints: vec![],
                        optional: false,
                    },
                ],
                required_capabilities: vec![
                    CapabilityRequirement::OwnerOrDelegate,
                ],
                constraints: vec![],
                metadata: HashMap::new(),
            },
        ],
        events: vec![
            InterfaceEvent {
                name: "Transfer".to_string(),
                fields: vec![
                    ParameterDefinition {
                        name: "from".to_string(),
                        param_type: ParameterType::AccountId,
                        default_value: None,
                        constraints: vec![],
                        optional: false,
                    },
                    ParameterDefinition {
                        name: "to".to_string(),
                        param_type: ParameterType::AccountId,
                        default_value: None,
                        constraints: vec![],
                        optional: false,
                    },
                    ParameterDefinition {
                        name: "amount".to_string(),
                        param_type: ParameterType::U64,
                        default_value: None,
                        constraints: vec![],
                        optional: false,
                    },
                ],
                metadata: HashMap::new(),
            },
        ],
        required_attributes: vec![
            AttributeDefinition {
                name: "owner".to_string(),
                attr_type: AttributeType::AccountId,
                constraints: vec![],
                mutable: true,
            },
            AttributeDefinition {
                name: "balance".to_string(),
                attr_type: AttributeType::U64,
                constraints: vec![],
                mutable: true,
            },
        ],
        optional_attributes: vec![],
        version: Version::new(1, 0, 0),
        extensions: vec![],
    }
}
```

### Mintable Interface

```rust
pub fn create_mintable_interface() -> ResourceInterface {
    ResourceInterface {
        id: InterfaceId::from_str("causality:interface:mintable:v1"),
        name: "Mintable".to_string(),
        operations: vec![
            InterfaceOperation {
                name: "mint".to_string(),
                inputs: vec![
                    ParameterDefinition {
                        name: "to".to_string(),
                        param_type: ParameterType::AccountId,
                        default_value: None,
                        constraints: vec![],
                        optional: false,
                    },
                    ParameterDefinition {
                        name: "amount".to_string(),
                        param_type: ParameterType::U64,
                        default_value: None,
                        constraints: vec![
                            ParameterConstraint::MinValue(Value::U64(1)),
                        ],
                        optional: false,
                    },
                ],
                outputs: vec![
                    ParameterDefinition {
                        name: "success".to_string(),
                        param_type: ParameterType::Bool,
                        default_value: None,
                        constraints: vec![],
                        optional: false,
                    },
                ],
                required_capabilities: vec![
                    CapabilityRequirement::Named("mint".to_string()),
                ],
                constraints: vec![],
                metadata: HashMap::new(),
            },
        ],
        events: vec![
            InterfaceEvent {
                name: "Mint".to_string(),
                fields: vec![
                    ParameterDefinition {
                        name: "to".to_string(),
                        param_type: ParameterType::AccountId,
                        default_value: None,
                        constraints: vec![],
                        optional: false,
                    },
                    ParameterDefinition {
                        name: "amount".to_string(),
                        param_type: ParameterType::U64,
                        default_value: None,
                        constraints: vec![],
                        optional: false,
                    },
                ],
                metadata: HashMap::new(),
            },
        ],
        required_attributes: vec![
            AttributeDefinition {
                name: "total_supply".to_string(),
                attr_type: AttributeType::U64,
                constraints: vec![],
                mutable: true,
            },
        ],
        optional_attributes: vec![
            AttributeDefinition {
                name: "max_supply".to_string(),
                attr_type: AttributeType::U64,
                constraints: vec![],
                mutable: false,
            },
        ],
        version: Version::new(1, 0, 0),
        extensions: vec![],
    }
}
```

## Interface Implementation

Resources implement interfaces to declare supported functionality:

```rust
pub struct ResourceImplementation {
    /// Resource type
    resource_type: ResourceType,
    
    /// Implemented interfaces
    interfaces: Vec<InterfaceImplementation>,
}

pub struct InterfaceImplementation {
    /// Interface ID being implemented
    interface_id: InterfaceId,
    
    /// Custom operation handlers
    operation_handlers: HashMap<String, OperationHandlerId>,
    
    /// Attribute mappings (interface attribute → resource attribute)
    attribute_mappings: HashMap<String, String>,
    
    /// Custom validation rules
    custom_validations: Vec<ValidationRuleFn>,
}

/// Define how a resource type implements an interface
pub fn implement_interface(
    resource_type: ResourceType,
    interface_id: InterfaceId,
    implementation: InterfaceImplementation,
) -> Result<(), InterfaceError> {
    // Verify interface exists
    let interface = interface_registry.get_interface(&interface_id)?;
    
    // Verify all required operations are implemented
    for op in &interface.operations {
        if !implementation.operation_handlers.contains_key(&op.name) {
            return Err(InterfaceError::MissingOperationHandler(op.name.clone()));
        }
    }
    
    // Verify all required attributes are mapped
    for attr in &interface.required_attributes {
        if !implementation.attribute_mappings.contains_key(&attr.name) {
            return Err(InterfaceError::MissingAttributeMapping(attr.name.clone()));
        }
    }
    
    // Register implementation
    interface_registry.register_implementation(resource_type, interface_id)?;
    implementation_registry.register_implementation(resource_type, interface_id, implementation)?;
    
    Ok(())
}
```

## Interface Discovery and Introspection

Services can discover what interfaces resources implement:

```rust
/// Get all interfaces implemented by a resource
pub fn get_resource_interfaces(resource_id: ResourceId) -> Result<Vec<ResourceInterface>, ResourceError> {
    // Get the resource
    let resource = registry.get_resource(resource_id)?;
    
    // Get the resource type
    let resource_type = resource.resource_type();
    
    // Get all interfaces implemented by this resource type
    let interface_ids = interface_registry.get_implemented_interfaces(&resource_type);
    
    // Fetch the interface definitions
    let mut interfaces = Vec::with_capacity(interface_ids.len());
    
    for id in interface_ids {
        if let Ok(interface) = interface_registry.get_interface(&id) {
            interfaces.push(interface);
        }
    }
    
    Ok(interfaces)
}

/// Check if a resource implements a specific interface
pub fn resource_implements_interface(
    resource_id: ResourceId, 
    interface_id: InterfaceId,
) -> Result<bool, ResourceError> {
    // Get the resource
    let resource = registry.get_resource(resource_id)?;
    
    // Get the resource type
    let resource_type = resource.resource_type();
    
    // Check if the resource type implements the interface
    let implements = interface_registry.implements_interface(
        &resource_type, 
        &interface_id,
    );
    
    Ok(implements)
}
```

## Interface-Based Operations

Operations can be performed via interfaces without knowledge of the specific resource type:

```rust
/// Perform an operation through an interface
pub fn perform_interface_operation(
    resource_id: ResourceId,
    interface_id: InterfaceId,
    operation_name: String,
    inputs: HashMap<String, Value>,
    auth_context: AuthContext,
) -> Result<HashMap<String, Value>, OperationError> {
    // Verify the resource exists
    let resource = registry.get_resource(resource_id)?;
    
    // Get the resource type
    let resource_type = resource.resource_type();
    
    // Verify the resource implements the interface
    if !interface_registry.implements_interface(&resource_type, &interface_id) {
        return Err(OperationError::InterfaceNotImplemented);
    }
    
    // Get the interface definition
    let interface = interface_registry.get_interface(&interface_id)?;
    
    // Find the operation in the interface
    let operation = interface.operations.iter()
        .find(|op| op.name == operation_name)
        .ok_or(OperationError::OperationNotFound)?;
    
    // Validate inputs
    validate_operation_inputs(&operation, &inputs)?;
    
    // Verify authorization
    if !auth_system.authorize_interface_operation(
        resource_id, 
        &interface_id, 
        &operation_name, 
        &auth_context
    ) {
        return Err(OperationError::Unauthorized);
    }
    
    // Get the implementation
    let implementation = implementation_registry.get_implementation(
        &resource_type, 
        &interface_id,
    )?;
    
    // Get the operation handler
    let handler_id = implementation.operation_handlers.get(&operation_name)
        .ok_or(OperationError::OperationHandlerNotFound)?;
    
    // Execute the operation
    let operation_handler = operation_handler_registry.get_handler(handler_id)?;
    let result = operation_handler.execute(resource_id, inputs, auth_context)?;
    
    // Validate outputs
    validate_operation_outputs(&operation, &result)?;
    
    Ok(result)
}
```

## Cross-Domain Interface Usage

Interfaces enable consistent access patterns across domains:

```rust
/// Perform a cross-domain interface operation
pub fn perform_cross_domain_interface_operation(
    resource_id: ResourceId,
    target_domain: DomainId,
    interface_id: InterfaceId,
    operation_name: String,
    inputs: HashMap<String, Value>,
    auth_context: AuthContext,
) -> Result<HashMap<String, Value>, OperationError> {
    // Verify the interface exists
    let interface = interface_registry.get_interface(&interface_id)?;
    
    // Find the operation in the interface
    let operation = interface.operations.iter()
        .find(|op| op.name == operation_name)
        .ok_or(OperationError::OperationNotFound)?;
    
    // Validate inputs
    validate_operation_inputs(&operation, &inputs)?;
    
    // Verify cross-domain capabilities
    if !cross_domain_registry.can_perform_interface_operation(target_domain, &interface_id) {
        return Err(OperationError::CrossDomainOperationNotSupported);
    }
    
    // Create cross-domain operation message
    let operation_msg = CrossDomainMessage::InterfaceOperation {
        origin_domain: system.domain_id(),
        resource_id,
        interface_id,
        operation_name,
        inputs,
        auth_context: auth_context.to_cross_domain(),
        timestamp: system.current_time(),
    };
    
    // Send operation message to target domain and await result
    let result = cross_domain_messenger.send_operation_and_wait_result(
        target_domain, 
        operation_msg,
    )?;
    
    // Validate outputs
    validate_operation_outputs(&operation, &result)?;
    
    Ok(result)
}
```

## Interface Evolution and Versioning

Interfaces can evolve over time with proper versioning:

```rust
/// Register a new version of an existing interface
pub fn register_interface_version(
    base_interface_id: InterfaceId,
    new_interface: ResourceInterface,
) -> Result<InterfaceId, InterfaceError> {
    // Verify base interface exists
    let base_interface = interface_registry.get_interface(&base_interface_id)?;
    
    // Verify version increment is valid
    if !is_valid_version_increment(&base_interface.version, &new_interface.version) {
        return Err(InterfaceError::InvalidVersionIncrement);
    }
    
    // Validate backwards compatibility if minor version increment
    if is_minor_version_increment(&base_interface.version, &new_interface.version) {
        validate_backwards_compatibility(&base_interface, &new_interface)?;
    }
    
    // Register the new interface version
    interface_registry.register_interface(new_interface.clone())?;
    
    // Create compatibility mapping
    interface_registry.register_compatibility(base_interface_id, new_interface.id)?;
    
    Ok(new_interface.id)
}
```

## Usage Examples

### Defining a Custom Interface

```rust
// Define a custom data access interface
let data_access_interface = ResourceInterface {
    id: InterfaceId::from_str("myapp:interface:data-access:v1"),
    name: "DataAccess".to_string(),
    operations: vec![
        InterfaceOperation {
            name: "read_data".to_string(),
            inputs: vec![
                ParameterDefinition {
                    name: "key".to_string(),
                    param_type: ParameterType::String,
                    default_value: None,
                    constraints: vec![],
                    optional: false,
                },
            ],
            outputs: vec![
                ParameterDefinition {
                    name: "data".to_string(),
                    param_type: ParameterType::Bytes,
                    default_value: None,
                    constraints: vec![],
                    optional: false,
                },
            ],
            required_capabilities: vec![
                CapabilityRequirement::Named("data.read".to_string()),
            ],
            constraints: vec![],
            metadata: HashMap::new(),
        },
        InterfaceOperation {
            name: "write_data".to_string(),
            inputs: vec![
                ParameterDefinition {
                    name: "key".to_string(),
                    param_type: ParameterType::String,
                    default_value: None,
                    constraints: vec![],
                    optional: false,
                },
                ParameterDefinition {
                    name: "data".to_string(),
                    param_type: ParameterType::Bytes,
                    default_value: None,
                    constraints: vec![],
                    optional: false,
                },
            ],
            outputs: vec![
                ParameterDefinition {
                    name: "success".to_string(),
                    param_type: ParameterType::Bool,
                    default_value: None,
                    constraints: vec![],
                    optional: false,
                },
            ],
            required_capabilities: vec![
                CapabilityRequirement::Named("data.write".to_string()),
            ],
            constraints: vec![],
            metadata: HashMap::new(),
        },
    ],
    events: vec![
        InterfaceEvent {
            name: "DataChanged".to_string(),
            fields: vec![
                ParameterDefinition {
                    name: "key".to_string(),
                    param_type: ParameterType::String,
                    default_value: None,
                    constraints: vec![],
                    optional: false,
                },
                ParameterDefinition {
                    name: "changed_by".to_string(),
                    param_type: ParameterType::AccountId,
                    default_value: None,
                    constraints: vec![],
                    optional: false,
                },
            ],
            metadata: HashMap::new(),
        },
    ],
    required_attributes: vec![
        AttributeDefinition {
            name: "data_owner".to_string(),
            attr_type: AttributeType::AccountId,
            constraints: vec![],
            mutable: true,
        },
    ],
    optional_attributes: vec![
        AttributeDefinition {
            name: "access_control_list".to_string(),
            attr_type: AttributeType::Map(Box::new(AttributeType::AccountId), Box::new(AttributeType::U8)),
            constraints: vec![],
            mutable: true,
        },
    ],
    version: Version::new(1, 0, 0),
    extensions: vec![],
};

// Register the interface
interface_registry.register_interface(data_access_interface)?;
```

### Implementing a Standard Interface

```rust
// Define how our token resource implements the Transferable interface
let transferable_implementation = InterfaceImplementation {
    interface_id: InterfaceId::from_str("causality:interface:transferable:v1"),
    operation_handlers: HashMap::from([
        ("transfer".to_string(), token_transfer_handler_id),
    ]),
    attribute_mappings: HashMap::from([
        ("owner".to_string(), "token_owner".to_string()),
        ("balance".to_string(), "token_balance".to_string()),
    ]),
    custom_validations: vec![
        validate_token_transfer,
    ],
};

// Register the implementation
implement_interface(
    ResourceType::Token,
    InterfaceId::from_str("causality:interface:transferable:v1"),
    transferable_implementation,
)?;
```

### Using Interfaces for Resource Discovery

```rust
// Find all resources that implement a specific interface
fn find_resources_implementing_interface(
    interface_id: InterfaceId,
    auth_context: AuthContext,
) -> Result<Vec<ResourceId>, ResourceError> {
    // Get all resource types that implement this interface
    let implementing_types = interface_registry.get_implementing_types(&interface_id);
    
    // Query resources of those types
    let mut result = Vec::new();
    
    for resource_type in implementing_types {
        let matching_resources = registry.query_resources_by_type(
            &resource_type,
            None, // No additional filters
            auth_context.clone(),
        )?;
        
        result.extend(matching_resources);
    }
    
    Ok(result)
}

// Example usage
let transferable_resources = find_resources_implementing_interface(
    InterfaceId::from_str("causality:interface:transferable:v1"),
    auth_context,
)?;

println!("Found {} transferable resources", transferable_resources.len());
```

### Performing Interface Operations

```rust
// Transfer tokens using the Transferable interface
let transfer_inputs = HashMap::from([
    ("to".to_string(), Value::AccountId(recipient_account_id)),
    ("amount".to_string(), Value::U64(100)),
]);

let result = perform_interface_operation(
    token_id,
    InterfaceId::from_str("causality:interface:transferable:v1"),
    "transfer".to_string(),
    transfer_inputs,
    auth_context,
)?;

let success = result.get("success")
    .and_then(|v| v.as_bool())
    .unwrap_or(false);

if success {
    println!("Transfer successful");
} else {
    println!("Transfer failed");
}
```

## Implementation Status

The following components of the resource interface system have been implemented:

- ✅ Core interface definition model
- ✅ Interface registry
- ✅ Basic interface discovery
- ✅ Standard interfaces for common resource types
- ⚠️ Interface-based operations (partially implemented)
- ⚠️ Interface versioning (partially implemented)
- ❌ Interface compatibility validation (not yet implemented)
- ❌ Cross-domain interface operations (not yet implemented)

## Future Enhancements

Future enhancements to the resource interface system include:

1. **Interface Composition**: Enable interfaces to be composed from smaller interface components
2. **Interface Inheritance**: Support formal interface inheritance relationships
3. **Dynamic Interface Discovery**: Runtime discovery of interfaces based on resource capabilities
4. **Interface Adapters**: Adapters to map between different interface versions or incompatible interfaces
5. **Schema Evolution**: Advanced schema evolution rules for interface versioning
6. **Interface Analytics**: Usage analytics for interface adoption and performance
7. **Interface Standardization Process**: Formal process for proposing and standardizing new interfaces 