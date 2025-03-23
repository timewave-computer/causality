# ADR-022: Rigorous Resource and Capability Model

## Status

Proposed

## Context

The Causality system employs several interrelated concepts around resources, capabilities, and authorization:

1. **Resources**: Core abstractions representing assets, data, and capabilities
2. **Resource Logic**: Rules governing how resources behave and transform
3. **Capabilities**: Unforgeable references that grant rights to perform operations
4. **Authorization**: Validation that operations are properly permitted

While these terms are appropriate and well-established in our codebase, their precise definitions, relationships, and implementation patterns have sometimes been inconsistent or ambiguous. This creates several challenges:

- **Conceptual Blurriness**: Unclear boundaries between these related concepts
- **Implementation Inconsistency**: Different subsystems implement similar patterns differently
- **Documentation Gaps**: Difficult to provide complete explanations without precise definitions
- **Developer Onboarding**: New developers struggle to understand the conceptual framework
- **Code Organization**: Unclear where specific functionality belongs

The lack of rigorous definitions and consistent implementation patterns has led to duplicated logic, architectural inconsistencies, and unnecessary complexity.

## Decision

We will establish rigorous definitions and implementation patterns for our existing terminology, creating clear conceptual boundaries while preserving familiar terms:

### 1. Resources

```rust
/// A resource in the Causality system
pub struct Resource {
    /// Unique identifier for this resource
    pub id: ResourceId,
    
    /// Core properties that define this resource
    pub properties: ResourceProperties,
    
    /// Logic governing this resource's behavior
    pub resource_logic: ResourceLogic,
    
    /// Current state of this resource
    pub state: ResourceState,
    
    /// Metadata about this resource
    pub metadata: HashMap<String, Value>,
}
```

**Definition**: A Resource is an addressable entity with intrinsic properties, governed by specific logic, that represents something of value in the system.

**Responsibility**: Resources encapsulate the "what" of the system—the digital assets, data, and rights that programs manipulate.

### 2. Resource Logic

```rust
/// Logic governing a resource's behavior
pub trait ResourceLogic: Send + Sync {
    /// Validate a proposed operation on this resource
    fn validate_operation(&self, resource: &Resource, operation: &Operation) -> Result<(), LogicError>;
    
    /// Apply an operation to this resource, transforming it as needed
    fn apply_operation(&self, resource: &mut Resource, operation: &Operation) -> Result<Vec<Effect>, LogicError>;
    
    /// Check conservation rules for this resource
    fn check_conservation(&self, resources: &[Resource], delta: &ResourceDelta) -> Result<(), ConservationError>;
    
    /// Returns the type identifier for this logic
    fn logic_type(&self) -> LogicType;
}
```

**Definition**: Resource Logic defines the rules governing how a resource behaves, transforms, and interacts with other resources. It implements the "physics" of our digital world.

**Responsibility**: Resource Logic encapsulates the "how" of resources—their transformation rules, validation logic, and conservation laws.

### 3. Capabilities

```rust
/// A capability in the Causality system
pub struct Capability {
    /// Unique identifier for this capability
    pub id: CapabilityId,
    
    /// The rights this capability grants
    pub rights: HashSet<Right>,
    
    /// Targets this capability applies to
    pub targets: Vec<Target>,
    
    /// Constraints on using this capability
    pub constraints: CapabilityConstraints,
    
    /// How this capability can be delegated
    pub delegation_rules: DelegationRules,
    
    /// When this capability expires (if ever)
    pub expiration: Option<Expiration>,
}
```

**Definition**: A Capability is an unforgeable reference that grants specific rights to perform operations on targeted resources, subject to constraints.

**Responsibility**: Capabilities implement the "who can do what" of the system—the access control mechanism that secures resources.

### 4. Authorization

```rust
/// Authorization service for validating operations
pub trait AuthorizationService: Send + Sync {
    /// Authorize an operation given capabilities
    fn authorize(&self, 
        entity: &EntityId, 
        operation: &Operation, 
        capabilities: &[Capability]) -> Result<AuthorizationResult, AuthError>;
    
    /// Check if an entity has a specific right
    fn has_right(&self, 
        entity: &EntityId, 
        right: &Right, 
        target: &Target, 
        capabilities: &[Capability]) -> bool;
    
    /// Validate capability delegation
    fn validate_delegation(&self, 
        delegator: &EntityId, 
        delegatee: &EntityId, 
        capability: &Capability, 
        constraints: &CapabilityConstraints) -> Result<(), DelegationError>;
}
```

**Definition**: Authorization is the process of validating that an entity possesses the required capabilities to perform a specific operation on specific resources.

**Responsibility**: Authorization implements the "validation" aspect of security—ensuring operations only proceed when properly permitted.

## Consequences

### Positive

1. **Conceptual Clarity**: Clear boundaries between related concepts while preserving familiar terms
2. **Consistent Implementation**: Standardized patterns for implementing each concept
3. **Better Encapsulation**: Clearer responsibilities for each component
4. **Improved Documentation**: Precise definitions make documentation more effective
5. **Better Developer Experience**: Clear mental model without terminology changes
6. **Enhanced Security**: Rigorous capability model leads to fewer security bugs

### Negative

1. **Refactoring Effort**: Some existing code will need updates to match rigorous patterns
2. **Initial Overhead**: Stricter definitions may require more explicit code in some cases
3. **Transition Period**: Parts of the codebase will use old patterns until fully refactored

### Neutral

1. **Documentation Updates**: All documentation will need to incorporate the rigorous definitions
2. **Testing Implications**: Tests may need updates to reflect more precise behaviors

## Implementation Plan

1. **Phase 1**: Establish Rigorous Definitions
   - Document precise definitions for each concept
   - Create glossary with examples
   - Identify implementation patterns for each concept

2. **Phase 2**: Standardize Core Interfaces
   ```rust
   // Define standard interfaces for resource logic
   pub trait ResourceLogic {
       fn validate_operation(&self, resource: &Resource, operation: &Operation) -> Result<(), LogicError>;
       fn apply_operation(&self, resource: &mut Resource, operation: &Operation) -> Result<Vec<Effect>, LogicError>;
       fn check_conservation(&self, resources: &[Resource], delta: &ResourceDelta) -> Result<(), ConservationError>;
       fn logic_type(&self) -> LogicType;
   }
   
   // Define standard interfaces for capabilities
   pub trait CapabilitySystem {
       fn verify_capability(&self, capability: &Capability) -> Result<(), CapabilityError>;
       fn delegate_capability(&self, source: &Capability, constraints: &CapabilityConstraints) -> Result<Capability, CapabilityError>;
       fn revoke_capability(&self, capability: &Capability) -> Result<(), CapabilityError>;
   }
   ```

3. **Phase 3**: Implement Reference Implementations
   - Create reference implementations of core interfaces
   - Build utilities for common patterns
   - Develop testing utilities for each concept

4. **Phase 4**: Refactor Existing Components
   - Start with self-contained systems
   - Gradually update more integrated components
   - Ensure all tests pass with updated implementations

5. **Phase 5**: Finalize Documentation and Examples
   - Update all developer documentation
   - Create pattern examples for each concept
   - Provide migration guides for legacy code

## Examples

### Current (Inconsistent) Implementation

```rust
// Create a token resource
let token_resource = Resource::new(
    ResourceLogic::Fungible, 
    "ETH", 
    1000
);

// Create a capability with unclear semantics
let capability = Capability::new(
    "transfer",
    vec!["token_resource"],
    Some(Constraints::new().with_max_amount(500))
);

// Ambiguous authorization check
if auth_system.check_authorization(user, "transfer", &token_resource) {
    // Perform transfer
}
```

### Rigorous Implementation

```rust
// Create a token resource with explicit logic
let token_resource = Resource::new(
    ResourceProperties::new()
        .with_fungibility_domain("ETH")
        .with_quantity(1000),
    FungibleTokenLogic::new()  // Implements ResourceLogic trait
);

// Create a properly defined capability
let transfer_capability = Capability::new(
    Rights::from([Right::Transfer]),
    Targets::Resource(token_resource.id),
    CapabilityConstraints::new()
        .with_max_quantity(500)
        .with_expiration(time::now() + Duration::hours(24))
);

// Explicit authorization check
let operation = Operation::Transfer {
    source: token_resource.id,
    destination: recipient.id,
    quantity: 100,
};

let auth_result = authorization_service.authorize(
    user.id,
    &operation,
    &[transfer_capability]
);

if auth_result.is_ok() {
    // Perform transfer using validated operation
    resource_system.apply_operation(&operation)?;
}
```

## Rigorous Definitions

### Resource-Related Terms

- **Resource**: An addressable entity with intrinsic properties, governed by specific logic, that represents something of value in the system
- **Resource Properties**: Intrinsic characteristics of a resource that define its identity and state
- **Resource Logic**: The rules governing how a resource behaves, transforms, and interacts with other resources
- **Resource State**: The current status of a resource, including its lifecycle stage and operational status
- **Conservation Law**: An invariant that ensures resources are neither created nor destroyed inappropriately (ΔTX = 0)

### Capability-Related Terms

- **Capability**: An unforgeable reference that grants specific rights to perform operations on targeted resources, subject to constraints
- **Right**: A specific permission to perform a category of operations (e.g., Transfer, Read, Write)
- **Target**: A resource or collection of resources that a capability applies to
- **Constraint**: A limitation on how a capability can be used (e.g., quantity limits, time restrictions)
- **Delegation**: The controlled transfer of a capability to another entity, possibly with additional constraints

### Authorization-Related Terms

- **Authorization**: The process of validating that an entity possesses the required capabilities to perform a specific operation
- **Operation**: A specific action performed on one or more resources (e.g., Transfer, Update, Create)
- **Entity**: A user, program, committee, or other actor in the system
- **Authorization Service**: A system component that evaluates operations against capabilities to determine permission
- **Validation**: The process of checking that an operation satisfies all constraints and requirements

## Implementation Patterns

### Resource Implementation Pattern

```rust
// Core resource pattern
struct MyResource {
    id: ResourceId,
    properties: ResourceProperties,
    state: ResourceState,
}

// Resource logic pattern
struct MyResourceLogic;

impl ResourceLogic for MyResourceLogic {
    fn validate_operation(&self, resource: &Resource, operation: &Operation) -> Result<(), LogicError> {
        // Validate the operation against resource rules
        match operation {
            Operation::Transfer { quantity, .. } => {
                // Check resource-specific rules
                if quantity > resource.properties.get_quantity() {
                    return Err(LogicError::InsufficientQuantity);
                }
                Ok(())
            }
            // Other operations...
            _ => Err(LogicError::UnsupportedOperation),
        }
    }
    
    // Other required methods...
}
```

### Capability Implementation Pattern

```rust
// Creating a capability
let capability = Capability::new()
    .with_rights([Right::Transfer, Right::Read])
    .with_target(Target::Resource(resource_id))
    .with_constraints(
        CapabilityConstraints::new()
            .with_max_quantity(100)
            .with_expiration(time::now() + Duration::days(7))
    )
    .with_delegation_rules(
        DelegationRules::new()
            .with_allowed_delegatees([entity_id])
            .with_required_attestation()
    );

// Using a capability
let operation = Operation::Transfer { /* ... */ };
let result = capability_system.use_capability(capability, &operation);
```

### Authorization Implementation Pattern

```rust
// Authorization service pattern
struct MyAuthorizationService {
    // Dependencies
}

impl AuthorizationService for MyAuthorizationService {
    fn authorize(&self, 
                 entity: &EntityId, 
                 operation: &Operation, 
                 capabilities: &[Capability]) -> Result<AuthorizationResult, AuthError> {
        // Find relevant capabilities
        let relevant_caps = self.find_relevant_capabilities(capabilities, operation);
        
        if relevant_caps.is_empty() {
            return Err(AuthError::MissingCapability);
        }
        
        // Check if any capability authorizes this operation
        for cap in relevant_caps {
            if self.can_authorize(cap, entity, operation) {
                return Ok(AuthorizationResult::Authorized);
            }
        }
        
        Err(AuthError::Unauthorized)
    }
    
    // Other required methods...
}
```

## References

1. [ADR-003: Resource System](./adr_003_resource.md)
2. [ADR-001: Effects Library](./adr_001_effects.md)
3. [Capability-based Security](https://en.wikipedia.org/wiki/Capability-based_security)
4. [Object-capability model](https://en.wikipedia.org/wiki/Object-capability_model)
5. [Principle of Least Privilege](https://en.wikipedia.org/wiki/Principle_of_least_privilege)

## Appendix: Complete Interface Reference

### ResourceLogic Interface

```rust
/// Logic governing a resource's behavior
pub trait ResourceLogic: Send + Sync {
    /// Validate a proposed operation on this resource
    fn validate_operation(&self, resource: &Resource, operation: &Operation) -> Result<(), LogicError>;
    
    /// Apply an operation to this resource, transforming it as needed
    fn apply_operation(&self, resource: &mut Resource, operation: &Operation) -> Result<Vec<Effect>, LogicError>;
    
    /// Check conservation rules for this resource
    fn check_conservation(&self, resources: &[Resource], delta: &ResourceDelta) -> Result<(), ConservationError>;
    
    /// Get constraints for operations on this resource
    fn get_constraints(&self, operation_type: OperationType) -> Vec<Constraint>;
    
    /// Get the resource schema
    fn get_schema(&self) -> ResourceSchema;
    
    /// Returns the type identifier for this logic
    fn logic_type(&self) -> LogicType;
}
```

### CapabilitySystem Interface

```rust
/// System for managing capabilities
pub trait CapabilitySystem: Send + Sync {
    /// Create a new capability
    fn create_capability(&self, 
                        creator: &EntityId,
                        rights: &[Right],
                        targets: &[Target],
                        constraints: &CapabilityConstraints) -> Result<Capability, CapabilityError>;
    
    /// Verify a capability is valid and not revoked
    fn verify_capability(&self, capability: &Capability) -> Result<(), CapabilityError>;
    
    /// Delegate a capability to another entity
    fn delegate_capability(&self,
                          source: &Capability,
                          delegator: &EntityId,
                          delegatee: &EntityId,
                          constraints: &CapabilityConstraints) -> Result<Capability, CapabilityError>;
    
    /// Revoke a capability
    fn revoke_capability(&self, capability: &Capability, revoker: &EntityId) -> Result<(), CapabilityError>;
    
    /// Query capabilities for an entity
    fn get_capabilities(&self, entity: &EntityId, filter: &CapabilityFilter) -> Vec<Capability>;
}
```

### AuthorizationService Interface

```rust
/// Service for authorizing operations
pub trait AuthorizationService: Send + Sync {
    /// Authorize an operation given capabilities
    fn authorize(&self, 
        entity: &EntityId, 
        operation: &Operation, 
        capabilities: &[Capability]) -> Result<AuthorizationResult, AuthError>;
    
    /// Check if an entity has a specific right
    fn has_right(&self, 
}
```