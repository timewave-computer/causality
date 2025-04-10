# Causality Architectural Layers: Building from Resources to Effects

*A layered analysis of the foundational components in Causality's architecture*

## Introduction

Causality's architecture is elegantly recursive, building higher-order abstractions from fundamental primitives, then representing those abstractions using the same primitives. This document clarifies the conceptual layers and their relationships, showing how each builds upon and extends the previous layer.

## 1. Resources: The Fundamental Units of State

At the very foundation of Causality's architecture are resources - the fundamental units of state. Resources are content-addressed, immutable, and cryptographically verifiable objects.

```rust
struct ResourceRegister {
    // Identity
    id: RegisterId,
    
    // State and properties
    fungibility_domain: FungibilityDomain,
    quantity: Quantity,
    metadata: Value,
    
    // Physical properties
    state: RegisterState,
    nullifier_key: NullifierKey,
    
    // Provenance tracking
    controller_label: ControllerLabel,
    
    // Temporal context
    observed_at: TimeMapSnapshot,
}
```

Resources have these key characteristics:

- **Content-addressed identification**: Resources are identified by cryptographic hashes of their content
- **Immutability**: Resource state changes create new resources with new identifiers
- **Verifiability**: Resources can be cryptographically verified
- **Composition**: Resources can reference other resources
- **Persistence**: Resources exist independent of who can access them

Resources represent the "nouns" in our system - they are the things that exist: accounts, tokens, programs, metadata, etc. They don't define behavior or access control; they simply represent state.

## 2. Resource Logic: Behavior and Rules

The next layer, Resource Logic, defines how resources behave - their valid state transitions, business rules, and domain-specific constraints.

```rust
trait ResourceLogic {
    // What operations can be performed
    fn available_operations(&self) -> Vec<OperationType>;
    
    // Validate an operation against current state
    fn validate_operation(&self, operation: &Operation) -> Result<(), ResourceError>;
    
    // Execute an operation, producing a new state
    fn execute_operation(&self, operation: &Operation) -> Result<ResourceState, ResourceError>;
    
    // Check if resource is in a safe state for schema evolution
    fn is_in_safe_state(&self) -> bool;
}
```

Resource Logic adds:

- **Valid state transitions**: Defines what changes are valid for a resource
- **Validation rules**: Enforces constraints specific to the resource type
- **Business logic**: Implements domain-specific behavior
- **Safe state definitions**: Determines when schema changes are permitted

Resource Logic represents the "verbs" as they apply to specific resources - what operations make sense and when they're valid. This layer is agnostic to who is performing operations; it only cares about the operations themselves.

For example, a token's Resource Logic would define that transfers can't exceed available balance, but wouldn't specify who can initiate transfers.

## 3. Agents: Actors as Specialized Resources

Agents build upon resources and resource logic, representing entities that can act within the system. Agents are themselves modeled as specialized resources with specific properties and relationships.

```rust
struct Agent {
    // Base resource implementation
    resource: Resource,
    
    // Identity information
    identity: Identity,
    
    // Capabilities that define what this agent can do
    capabilities: Vec<Capability>,
    
    // State information
    state: AgentState,
    
    // Relationship to other agents and resources
    relationships: Vec<ResourceRelationship>,
}
```

Agents add:

- **Identity with cryptographic verification**: Who the agent is and how they authenticate
- **State transitions**: Activation, suspension, termination
- **Relationship tracking**: How this agent relates to other resources and agents
- **Capability containers**: Stores and manages the agent's capabilities

What's fascinating about agents is that they're implemented as resources with specific resource logic, but they also represent the actors that can interact with other resources. This recursive relationship creates an elegant symmetry in the system.

The system defines three primary agent types:
- **User Agents**: Human end-users interacting with the system
- **Committee Agents**: Validator groups for domains like blockchains
- **Operator Agents**: System administrators and automated operators

## 4. Capabilities: Permission Tokens

Capabilities are unforgeable tokens of authority that grant specific rights to perform operations on resources. They build on the lower layers by formalizing who can do what with resources.

```rust
struct Capability {
    // Target resource
    target: ResourceId,
    
    // Capability type (read, write, etc.)
    capability_type: CapabilityType,
    
    // Constraints on this capability
    constraints: Vec<CapabilityConstraint>,
    
    // Expiration time (if any)
    expires_at: Option<DateTime<Utc>>,
    
    // Content hash
    content_hash: ContentHash,
}
```

Capabilities add:

- **Explicit permission tokens**: Reified permissions as transferable objects
- **Delegation chains**: Ability to transfer authority with constraints
- **Fine-grained access control**: Precisely defined permissions
- **Capability constraints**: Time, usage, or state conditions for capability use
- **Revocation mechanisms**: Ways to revoke authority

Capabilities bridge resource logic and agents by explicitly defining which agents can perform which operations on which resources. They transform implicit access rules into explicit, transferable tokens.

The capability-based security model ensures:
1. Only agents with appropriate capabilities can perform operations
2. Delegation must be explicit and recorded
3. The principle of least privilege is enforced

## 5. Effects & Operations: Actions and Intent

At the highest conceptual layer, Effects and Operations represent concrete actions in the system. They build on all previous layers by combining:
- What operation to perform (referencing resource logic)
- On which resource (referencing resources)
- By which agent (referencing agents)
- With what authorization (referencing capabilities)

```rust
struct Operation {
    // Unique ID for the operation
    id: OperationId,
    
    // Agent that initiated the operation
    agent_id: AgentId,
    
    // Target resource
    target_resource_id: ResourceId,
    
    // Operation type
    operation_type: OperationType,
    
    // Operation parameters
    parameters: HashMap<String, String>,
    
    // Effects to be executed as part of this operation
    effects: Vec<Box<dyn Effect>>,
    
    // Required capabilities for this operation
    required_capabilities: Vec<Capability>,
    
    // Operation metadata
    metadata: HashMap<String, String>,
}

trait Effect<R> {
    // Execute the effect with the given handler
    fn execute(self, handler: &dyn EffectHandler) -> EffectOutcome<R>;
    
    // Get the resources this effect requires
    fn resources(&self) -> Vec<ResourceId>;
    
    // Get the capabilities required for this effect
    fn required_capabilities(&self) -> Vec<Capability>;
}
```

Effects and Operations add:

- **Intent declaration**: Explicit statement of what an agent wants to do
- **Authorization requirements**: Declaration of what capabilities are needed
- **Execution semantics**: How the action should be processed
- **Composition**: How multiple effects combine into transactions
- **Resource requirements**: Which resources will be affected

This layer ties everything together - agents use their capabilities to authorize operations on resources, which are validated against resource logic and executed to produce new resource states.

## Relationships Between Layers

The relationships between these layers form a cohesive, self-reinforcing system:

### Resources and Resource Logic
Resource Logic depends directly on Resources, defining the rules and behaviors specific to resource types. Resource Logic cannot exist without Resources to operate on.

### Agents and Resources
Agents are implemented as specialized Resources with specific Resource Logic that governs agent lifecycle and behavior. This creates a recursive relationship where Resources are used to model the entities that interact with Resources.

### Agents and Resource Logic
Agents interact with Resource Logic when performing operations, but their own behavior is defined by Agent-specific Resource Logic. Agents both use Resource Logic and are governed by it.

### Capabilities and Resources
Capabilities reference Resources as their targets and are themselves content-addressed objects with similar properties to Resources. However, Capabilities add the semantics of permission and delegation.

### Capabilities and Agents
Capabilities are held by Agents and determine what operations those Agents can perform. Agents are both the holders of Capabilities and can themselves be the targets of Capabilities (when one Agent is granted authority over another).

### Effects/Operations and Resource Logic
Effects leverage Resource Logic to define validation rules and execution semantics. The Effect system provides a framework for expressing operations, while Resource Logic determines if those operations are valid for specific resources.

### Effects/Operations and Capabilities
Effects declare what Capabilities they require, and Operations include proofs that the Agent has those Capabilities. This creates the authorization check that bridges agent identity with permitted actions.

## Building Effects from Resources

To see how the entire stack builds up from Resources to Effects, consider a token transfer operation:

1. **Resources**: Token accounts exist as resources with state (balances)
2. **Resource Logic**: Defines that transfers cannot exceed balance and require recipient existence
3. **Agents**: The user agent exists as a specialized resource with identity
4. **Capabilities**: The user agent holds a transfer capability for their token account
5. **Effects**: The user agent creates a transfer effect, presenting their capability as authorization

The entire operation would flow through the system like this:

1. User agent initiates a transfer operation
2. Operation includes the transfer effect and references the transfer capability
3. Capability system validates that the capability is valid for this operation
4. Resource logic validates that the transfer satisfies business rules (sufficient balance)
5. Effect is executed, producing new token account resources with updated balances
6. The operation result is returned to the user agent

This layered approach creates a secure, expressive, and composable architecture where high-level concepts like operations cleanly build upon foundational primitives like resources.

## Conclusion

Understanding Causality's architecture as a layered system helps clarify how its components interact. While there's a conceptual hierarchy from Resources up to Effects, what makes the architecture special is how higher layers are represented using the same primitives as lower layers, creating a self-describing system with powerful recursive properties.

This architectural pattern enables Causality to achieve its goals of cross-domain operation with strong guarantees about resource conservation, temporal consistency, and verifiable execution.
