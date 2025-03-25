# ADR-031: Domain Adapter as Effect

## Status

Proposed

## Context

The Causality system currently has two parallel subsystems for interacting with blockchains and executing operations:

1. **Domain Adapters**: Defined in `crates/causality-domain/src/adapter.rs`, these provide a unified interface for connecting to and interacting with different blockchain systems (EVM, CosmWasm, etc.).

2. **Effect System**: Defined in `crates/causality-effects/src/effect.rs`, this provides a framework for executing actions with proper authorization, validation, and context.

While there is an extension trait `EffectHandlerAdapter` for domain adapters, the integration is incomplete and one-directional. This creates several challenges:

- **Duplicated Functionality**: Similar code exists in both systems
- **Inconsistent Authorization Models**: Each system has its own approach to authorization
- **Poor Composability**: Difficult to build higher-level operations that span both systems
- **Redundant Implementation Paths**: Developers must implement features in both systems
- **Limited Cross-Domain Operations**: No standard way to coordinate actions across domains

Previous work in [ADR-023: Three-Layer Effect Architecture with TEL Integration](./adr_023_domain_adapter_effect_handler_unification.md) started to unify handlers and adapters but did not fully address the bidirectional integration of domain adapters and effects.

## Decision Drivers

1. **Unified Programming Model**: Provide a single, consistent approach for all operations
2. **Reduced Duplication**: Eliminate parallel code paths and duplicate functionality
3. **Improved Type Safety**: Leverage Rust's type system for domain-specific operations
4. **Enhanced Composability**: Enable seamless composition of effects across domains
5. **Streamlined Developer Experience**: Simplify the process of adding new domain support
6. **Robust Authorization**: Ensure proper authorization across all operations

## Considered Options

### Option 1: Maintain Separate Systems with Better Integration

Improve the existing integration between domain adapters and effects while keeping them as separate subsystems. This would involve:

- Enhancing the `EffectHandlerAdapter` trait
- Creating better bridging mechanisms
- Standardizing data formats
- Providing helper utilities for cross-system operations

### Option 2: Fold Domain Adapters into Effects (Domain-to-Effect)

Make domain adapters a specialized type of effect. This would involve:

- Redefining domain adapter methods as effects
- Implementing domain adapter functionality within the effect system
- Deprecated standalone domain adapter interfaces
- Migrating all domain adapter usage to effect-based patterns

### Option 3: Extend Effects with Domain Capabilities (Effect-to-Domain)

Enhance the effect system to handle domain-specific operations directly. This would involve:

- Adding domain-specific traits to effects
- Implementing domain operations within effects
- Maintaining domain adapters as the underlying implementation
- Treating domain adapters as an implementation detail

### Option 4: Bidirectional Integration (Domain ⇄ Effect)

Create a complete bidirectional integration where domain adapters can be used as effects and effects can leverage domain adapters seamlessly. This would involve:

- Implementing domain adapter methods as effects
- Creating effect wrappers for domain operations
- Enabling effects to directly use domain adapters
- Building cross-domain orchestration capabilities

## Decision

We will implement **Option 4: Bidirectional Integration** between domain adapters and effects. This approach provides the best balance of backward compatibility, developer experience, and architectural coherence.

The integration will have these key components:

1. **Domain Adapter Effect Wrappers**: Each domain adapter method will be available as a corresponding effect
2. **Effect-Based Domain Registry**: The domain registry will function as an effect handler
3. **Domain-Specific Effect Types**: Custom effect types for domain-specific operations
4. **Cross-Domain Effect Composition**: Utilities for composing effects across domains
5. **Unified Authorization Model**: Consistent capability-based authorization across both systems

## Architectural Design

### 1. Domain Adapter Effect Layer

We will introduce a new set of effect types that wrap domain adapter functionality:

```rust
// Core domain effect trait
pub trait DomainAdapterEffect: Effect {
    fn domain_id(&self) -> &DomainId;
    fn create_context(&self, base_context: &EffectContext) -> DomainContext;
    fn map_outcome(&self, domain_result: Result<impl Any, DomainError>) -> EffectResult<EffectOutcome>;
}

// Domain operation effects
pub struct DomainQueryEffect {
    id: EffectId,
    domain_id: DomainId,
    query: FactQuery,
    parameters: HashMap<String, String>,
}

pub struct DomainTransactionEffect {
    id: EffectId,
    domain_id: DomainId,
    transaction: Transaction,
    wait_for_confirmation: bool,
    timeout_ms: Option<u64>,
}

pub struct DomainTimeMapEffect {
    id: EffectId,
    domain_id: DomainId,
    height: Option<BlockHeight>,
}

pub struct DomainCapabilityEffect {
    id: EffectId,
    domain_id: DomainId,
    capability: String,
}
```

### 2. Domain Effect Registry

The domain registry will be extended to function as an effect handler:

```rust
pub struct DomainEffectRegistry {
    adapter_registry: Arc<DomainAdapterRegistry>,
    domain_factories: HashMap<String, Arc<dyn DomainAdapterFactory>>,
    capability_manager: Arc<DomainCapabilityManager>,
}

impl EffectHandler for DomainEffectRegistry {
    async fn execute_effect(&self, effect: &dyn Effect, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Dispatch to the appropriate handler based on effect type
        match effect.effect_type() {
            "domain_query" => self.handle_query_effect(effect, context).await,
            "domain_transaction" => self.handle_transaction_effect(effect, context).await,
            "domain_time_map" => self.handle_time_map_effect(effect, context).await,
            "domain_capability" => self.handle_capability_effect(effect, context).await,
            _ => Err(EffectError::UnsupportedEffect(effect.effect_type().to_string())),
        }
    }
    
    fn can_handle_effect(&self, effect_type: &str) -> bool {
        matches!(
            effect_type,
            "domain_query" | "domain_transaction" | 
            "domain_time_map" | "domain_capability"
        )
    }
}
```

### 3. Domain-Specific Effect Implementations

For EVM-specific operations:

```rust
pub struct EvmContractCallEffect {
    id: EffectId,
    domain_id: DomainId,
    contract_address: String,
    function_name: String,
    function_args: Vec<Value>,
    gas_limit: Option<u64>,
}

impl Effect for EvmContractCallEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "evm_contract_call"
    }
    
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    async fn execute(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // Implementation that leverages domain adapter functionality
    }
}
```

### 4. Cross-Domain Effect Orchestration

```rust
pub struct CrossDomainTransferEffect {
    id: EffectId,
    source_domain_id: DomainId,
    target_domain_id: DomainId,
    source_account: String,
    target_account: String,
    amount: u64,
    asset_id: String,
}

impl Effect for CrossDomainTransferEffect {
    // Implementation that coordinates across domains
    // Uses domain adapter effects internally
}
```

### 5. Unified Authorization Model

Authorization will be unified around capabilities:

```rust
pub struct CapabilityContext {
    domain_capabilities: HashMap<DomainId, HashSet<DomainCapability>>,
    effect_capabilities: HashSet<EffectCapability>,
}

impl CapabilityContext {
    // Check if the context allows a specific domain capability
    pub fn has_domain_capability(&self, domain_id: &DomainId, capability: &DomainCapability) -> bool {
        self.domain_capabilities
            .get(domain_id)
            .map(|caps| caps.contains(capability))
            .unwrap_or(false)
    }
    
    // Check if the context allows a specific effect capability
    pub fn has_effect_capability(&self, capability: &EffectCapability) -> bool {
        self.effect_capabilities.contains(capability)
    }
    
    // Check if the context allows executing a specific effect
    pub fn can_execute_effect(&self, effect: &dyn Effect) -> bool {
        // Implementation that checks both domain and effect capabilities
    }
}
```

## Consequences

### Positive

1. **Unified Programming Model**: Developers only need to learn one approach (effects) for all operations
2. **Enhanced Composability**: Effects can be easily composed, including across domain boundaries
3. **Type Safety**: Stronger type checking for domain-specific operations
4. **Reuse of Infrastructure**: Leverages existing effect mechanisms for authorization, validation, etc.
5. **Better Testability**: Can use the same testing approaches for all operations
6. **Streamlined Integration**: Easier to integrate with other systems that use the effect model
7. **Clear Authorization**: A single, consistent capability-based authorization model

### Negative

1. **Migration Effort**: Existing code that uses domain adapters directly will need updates
2. **Learning Curve**: Developers familiar with only one system will need to learn the integration
3. **Performance Overhead**: Slight overhead from the additional abstraction layer
4. **Implementation Complexity**: Bidirectional integration is more complex than one-way approaches

### Neutral

1. **API Surface Growth**: New APIs for the integration, but existing APIs remain available
2. **Documentation Updates**: Comprehensive documentation will be required for the new approach

## Implementation Strategy

The implementation will follow this phased approach:

1. **Core Infrastructure**: Implement the foundational traits and data structures
2. **Domain Effect Wrappers**: Create effect wrappers for domain adapter methods
3. **Registry Integration**: Extend domain registry with effect handling capabilities
4. **Domain-Specific Effects**: Implement domain-specific effect types
5. **Cross-Domain Operations**: Build cross-domain effect composition utilities
6. **Test Suite**: Create comprehensive tests for the integration
7. **Documentation**: Update documentation with usage examples and best practices

## Examples

### Example 1: Querying a Domain Fact Using an Effect

```rust
// Create a domain query effect
let query_effect = DomainQueryEffect::new(
    ethereum_domain_id.clone(),
    FactQuery::new()
        .with_fact_type("balance")
        .with_parameter("account", "0x1234...")
);

// Execute the effect
let outcome = effect_system.execute(&query_effect, &context).await?;

// Extract the balance from the outcome
let balance = outcome.data.get("balance")
    .and_then(|v| v.parse::<u64>().ok())
    .unwrap_or(0);
```

### Example 2: Cross-Domain Token Transfer

```rust
// Create a cross-domain transfer effect
let transfer_effect = CrossDomainTransferEffect::new(
    ethereum_domain_id.clone(),
    cosmwasm_domain_id.clone(),
    "0xalice...",
    "cosmos1bob...",
    1000,
    "USDC"
);

// Execute the effect
let outcome = effect_system.execute(&transfer_effect, &context).await?;

// Check if the transfer was successful
if outcome.success {
    println!("Transfer completed successfully!");
    println!("Ethereum transaction: {}", outcome.data.get("source_tx").unwrap());
    println!("CosmWasm transaction: {}", outcome.data.get("target_tx").unwrap());
} else {
    println!("Transfer failed: {}", outcome.error.unwrap());
}
```

## Future Work

This integration lays the groundwork for several future enhancements:

1. **Domain-Agnostic Programming Model**: Further abstractions that allow domain-independent business logic
2. **Automatic Domain Selection**: Intelligent selection of domains based on operation requirements
3. **Effect Templates**: Parameterized effect templates for common operations
4. **Cross-Domain Atomic Operations**: Mechanisms for ensuring atomicity across domains
5. **Resource Abstraction Layer**: Domain-independent resource abstractions
6. **Effect Monitoring and Observability**: Unified monitoring of all operations

## References

- [ADR-023: Three-Layer Effect Architecture with TEL Integration](./adr_023_domain_adapter_effect_handler_unification.md)
- [ADR-018: Domain Adapter](./adr_018_domain_adapter.md)
- [Domain Integration Patterns](../docs/src/domains/integration.md)
- [Effect System Overview](../docs/src/components/effects/overview.md)
- [Work Plan: Domain Adapter Effect Implementation](../work/d_effect.md) 