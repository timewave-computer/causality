<!-- Constraints in the effect system -->
<!-- Original file: docs/src/effect_constraints.md -->

# Effect Constraint Traits System

## Overview

The Effect Constraint Traits system is a fundamental part of the three-layer effect architecture, providing type-safe interfaces for defining, composing, and validating effects. This document explains the constraint trait system and how it enables powerful abstractions while maintaining type safety and domain independence.

## What Are Effect Constraints?

Effect constraints are Rust traits that define the behavior and requirements for different types of effects. They sit above the base `Effect` trait in the type hierarchy, providing specialized interfaces for common effect categories:

```rust
pub trait Effect {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn required_capabilities(&self) -> Vec<(ResourceId, Right)>;
    
    async fn execute(&self, context: EffectContext) -> EffectResult<EffectOutcome>;
    
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool;
    fn preferred_boundary(&self) -> ExecutionBoundary;
}

// Example constraint trait
pub trait TransferEffect: Effect {
    fn source(&self) -> &Address;
    fn destination(&self) -> &Address;
    fn amount(&self) -> &Quantity;
    fn token(&self) -> &ResourceId;
}
```

## Benefits of the Constraint System

The constraint system provides several key benefits:

### 1. Type Safety

Constraints enforce type safety across the effect system, ensuring that:
- Effects can be validated at compile time
- Required methods are implemented
- Return types are consistent

### 2. Domain Independence

Constraints abstract away domain-specific details, allowing:
- Domain-agnostic effect definitions
- Code that works across multiple domains
- Easy migration between domains

### 3. Composability

Constraints enable safe effect composition:
- Effects with compatible constraints can be composed
- Constraint checking prevents invalid compositions
- Meta-effects can be built from primitive effects

### 4. Documentation and Discoverability

Constraints serve as self-documenting interfaces:
- Clear expectations for what an effect does
- Standardized method names and signatures
- Hierarchical organization of effect types

## Core Constraint Traits

The system defines several core constraint traits:

### TransferEffect

Represents the transfer of assets between addresses:

```rust
pub trait TransferEffect: Effect {
    fn source(&self) -> &Address;
    fn destination(&self) -> &Address;
    fn amount(&self) -> &Quantity;
    fn token(&self) -> &ResourceId;
    
    // Optional methods with default implementations
    fn requires_approval(&self) -> bool { false }
    fn fee(&self) -> Option<Quantity> { None }
}
```

### StorageEffect

Represents operations that store data:

```rust
pub trait StorageEffect: Effect {
    fn register_id(&self) -> &ResourceId;
    fn fields(&self) -> &HashSet<String>;
    fn visibility(&self) -> &StateVisibility;
    
    // Optional methods with default implementations
    fn is_update(&self) -> bool { false }
    fn previous_state_hash(&self) -> Option<&str> { None }
}
```

### QueryEffect

Represents read-only data queries:

```rust
pub trait QueryEffect: Effect {
    fn query_type(&self) -> &str;
    fn parameters(&self) -> &QueryParameters;
    fn timeout(&self) -> Duration;
    
    // Optional methods with default implementations
    fn cache_ttl(&self) -> Option<Duration> { None }
    fn requires_fresh_data(&self) -> bool { false }
}
```

### VerificationEffect

Represents cryptographic verification operations:

```rust
pub trait VerificationEffect: Effect {
    fn verification_type(&self) -> &str;
    fn data(&self) -> &[u8];
    fn proof(&self) -> &ProofData;
    
    // Optional methods with default implementations
    fn scheme(&self) -> &str { "default" }
    fn options(&self) -> &VerificationOptions { &DEFAULT_OPTIONS }
}
```

## Constraint Composition

Constraints can be composed to create more specific effect types:

```rust
// Composition example
pub trait TokenApprovalEffect: TransferEffect {
    fn spender(&self) -> &Address;
    fn expiration(&self) -> Option<Timestamp>;
}

pub trait CrossDomainTransferEffect: TransferEffect {
    fn source_domain(&self) -> &DomainId;
    fn target_domain(&self) -> &DomainId;
    fn bridge_address(&self) -> &Address;
}
```

## Using Constraints in Code

### Implementing a Constrained Effect

```rust
// Example implementation of a constrained effect
pub struct EthereumTransferEffect {
    source: Address,
    destination: Address,
    amount: Quantity,
    token: ResourceId,
    domain_id: DomainId,
    // EVM-specific fields
    gas_limit: u64,
    gas_price: u64,
}

impl Effect for EthereumTransferEffect {
    fn name(&self) -> &str {
        "ethereum_transfer"
    }
    
    fn description(&self) -> &str {
        "Transfers tokens on Ethereum network"
    }
    
    fn required_capabilities(&self) -> Vec<(ResourceId, Right)> {
        vec![(self.token.clone(), Right::Transfer)]
    }
    
    async fn execute(&self, context: EffectContext) -> EffectResult<EffectOutcome> {
        // EVM-specific implementation
        // ...
    }
    
    fn can_execute_in(&self, boundary: ExecutionBoundary) -> bool {
        boundary == ExecutionBoundary::OutsideSystem
    }
    
    fn preferred_boundary(&self) -> ExecutionBoundary {
        ExecutionBoundary::OutsideSystem
    }
}

impl TransferEffect for EthereumTransferEffect {
    fn source(&self) -> &Address {
        &self.source
    }
    
    fn destination(&self) -> &Address {
        &self.destination
    }
    
    fn amount(&self) -> &Quantity {
        &self.amount
    }
    
    fn token(&self) -> &ResourceId {
        &self.token
    }
    
    // Override optional method
    fn fee(&self) -> Option<Quantity> {
        Some(Quantity((self.gas_limit * self.gas_price) as u128))
    }
}
```

### Using Constraints with Generics

```rust
// Function that accepts any effect implementing TransferEffect
async fn process_transfer<T: TransferEffect + Send + Sync>(
    transfer_effect: &T,
    context: &mut EffectContext
) -> EffectResult<EffectOutcome> {
    // We can use any methods from the TransferEffect trait
    println!("Processing transfer from {} to {} for {} of token {}",
        transfer_effect.source(),
        transfer_effect.destination(),
        transfer_effect.amount(),
        transfer_effect.token()
    );
    
    // Execute the effect
    transfer_effect.execute(context.clone()).await
}
```

## Constraint Validation and Orchestration

The effect system includes a robust validation and orchestration layer to ensure effects are used correctly:

### EffectValidator

The `EffectValidator` provides comprehensive validation for effects based on their constraint traits:

```rust
pub struct EffectValidator {
    domain_registry: Arc<DomainRegistry>,
    capability_repo: Arc<dyn CapabilityRepository>,
    resource_api: Arc<dyn ResourceAPI>,
}

impl EffectValidator {
    // Validate an effect based on its constraints
    pub async fn validate_effect(&self, effect: &dyn Effect, context: &EffectContext) -> Result<(), EffectError>;
    
    // Validate capabilities for an effect
    async fn validate_capabilities(&self, effect: &dyn Effect, context: &EffectContext) -> Result<(), EffectError>;
    
    // Constraint-specific validation
    async fn validate_transfer_effect(&self, effect: &dyn TransferEffect, context: &EffectContext) -> Result<(), EffectError>;
    async fn validate_storage_effect(&self, effect: &dyn StorageEffect, context: &EffectContext) -> Result<(), EffectError>;
    async fn validate_query_effect(&self, effect: &dyn QueryEffect, context: &EffectContext) -> Result<(), EffectError>;
}
```

The validator ensures:
- Required capabilities are present in the execution context
- Effect parameters are valid and consistent
- Resources referenced by the effect exist
- Domain-specific constraints are met

### EffectOrchestrator

The `EffectOrchestrator` provides execution services with built-in validation:

```rust
pub struct EffectOrchestrator {
    validator: EffectValidator,
}

impl EffectOrchestrator {
    // Execute a single effect with validation
    pub async fn execute_effect<E: Effect + ?Sized>(&self, effect: &E, context: EffectContext) -> EffectResult<EffectOutcome>;
    
    // Execute a sequence of effects
    pub async fn execute_sequence(&self, effects: Vec<Arc<dyn Effect>>, context: EffectContext) -> EffectResult<Vec<EffectOutcome>>;
    
    // Execute effects in parallel
    pub async fn execute_parallel(&self, effects: Vec<Arc<dyn Effect>>, context: EffectContext) -> EffectResult<Vec<EffectOutcome>>;
    
    // Execute an effect conditionally
    pub async fn execute_conditional(
        &self,
        condition: Arc<dyn Effect>,
        then_effect: Arc<dyn Effect>,
        else_effect: Option<Arc<dyn Effect>>,
        context: EffectContext
    ) -> EffectResult<EffectOutcome>;
}
```

This orchestrator provides:
- Pre-execution validation for all effects
- Sequential, parallel, and conditional execution patterns
- Consistent error handling across execution modes
- Capability verification at execution time

### Example Usage

```rust
// Create validator with required services
let validator = EffectValidator::new(
    domain_registry.clone(),
    capability_repo.clone(),
    resource_api.clone(),
);

// Create orchestrator
let orchestrator = EffectOrchestrator::new(validator);

// Execute a transfer effect with validation
let transfer_effect = create_transfer_effect(
    source, destination, amount, token, domain_id
).await?;

let outcome = orchestrator.execute_effect(
    transfer_effect.as_ref(),
    context
).await?;

// Execute a sequence of effects
let sequence_outcome = orchestrator.execute_sequence(
    vec![effect1, effect2, effect3],
    context
).await?;
```

## Constraint Validation

The system provides tools to validate effects against constraints:

```rust
// Validate that an effect implements a constraint
fn validate_transfer_effect(effect: &dyn Effect) -> Result<(), ValidationError> {
    if let Some(transfer_effect) = effect.as_any().downcast_ref::<dyn TransferEffect>() {
        // Effect implements TransferEffect
        // Additional validation
        if transfer_effect.amount().is_zero() {
            return Err(ValidationError::new("Transfer amount cannot be zero"));
        }
        Ok(())
    } else {
        Err(ValidationError::new("Effect does not implement TransferEffect"))
    }
}
```

## Type Erasure and Runtime Checking

While Rust's trait system provides compile-time checking, we sometimes need to work with dynamic effects:

```rust
// Working with dynamic effects
fn process_dynamic_effect(effect: Arc<dyn Effect>) -> Result<(), Error> {
    // Try to use as different constraint types
    if let Some(transfer) = effect.as_any().downcast_ref::<dyn TransferEffect>() {
        println!("Transfer effect: {} -> {}", transfer.source(), transfer.destination());
    } else if let Some(storage) = effect.as_any().downcast_ref::<dyn StorageEffect>() {
        println!("Storage effect for register: {}", storage.register_id());
    } else {
        println!("Unknown effect type: {}", effect.name());
    }
    
    Ok(())
}
```

## Constraint-Based Effect Factories

Constraints allow for powerful factory patterns:

```rust
// Factory for creating transfer effects
struct TransferEffectFactory {
    domain_registry: Arc<DomainRegistry>,
}

impl TransferEffectFactory {
    fn create_transfer_effect(
        &self,
        source: Address,
        destination: Address,
        amount: Quantity,
        token: ResourceId,
        domain_id: DomainId,
    ) -> Result<Arc<dyn TransferEffect>, Error> {
        // Get domain info
        let domain_info = self.domain_registry.get_domain_info(&domain_id)?;
        
        // Create domain-specific implementation
        match domain_info.domain_type {
            DomainType::EVM => {
                let effect = EthereumTransferEffect::new(
                    source, destination, amount, token, domain_id
                )?;
                Ok(Arc::new(effect))
            },
            DomainType::CosmWasm => {
                let effect = CosmWasmTransferEffect::new(
                    source, destination, amount, token, domain_id
                )?;
                Ok(Arc::new(effect))
            },
            // Other domain types...
            _ => Err(Error::UnsupportedDomain(domain_id))
        }
    }
}
```

## TEL Integration

The constraint system integrates with TEL for type-safe, domain-independent effect creation:

```rust
// TEL program that uses constraints
let tx = tel! {
    transfer(
        from: account,
        to: recipient,
        amount: 100,
        token: "ETH",
        domain: eth_domain
    )
}

// TEL compiler generates constraint-based validation
let effect = compile_and_validate_tel(tx)?;

// The effect implements TransferEffect
let transfer_effect = effect.as_any().downcast_ref::<dyn TransferEffect>()
    .expect("Effect should implement TransferEffect");
```

## Best Practices

### 1. Design Constraint Hierarchies Carefully

- Start with minimal core constraints
- Add specialized constraints through composition
- Avoid deep inheritance hierarchies
- Consider using marker traits for categories

### 2. Use Default Implementations

- Provide sensible defaults for optional methods
- Allow selective customization
- Document default behavior clearly

### 3. Ensure Domain Independence

- Keep constraints domain-agnostic
- Avoid domain-specific types in constraint interfaces
- Use adaptors to bridge domain-specific details

### 4. Document Constraints

- Document the purpose of each constraint
- Explain the semantics of each method
- Provide examples of proper implementation

### 5. Test Constraint Implementations

- Create test helpers for constraint verification
- Test each implementation against its constraints
- Ensure constraint invariants are maintained

## Advanced Patterns

### Multi-Constraint Effects

Effects can implement multiple constraints:

```rust
// Multi-constraint effect
pub struct SwapEffect {
    // ... fields
}

impl Effect for SwapEffect {
    // ... implementation
}

impl TransferEffect for SwapEffect {
    // ... implementation
}

impl QueryEffect for SwapEffect {
    // ... implementation
}
```

### Conditional Constraints

Effects can conditionally implement constraints:

```rust
// Conditional constraint implementation
pub struct FlexibleEffect {
    mode: EffectMode,
    // ... other fields
}

impl Effect for FlexibleEffect {
    // ... implementation
}

impl TransferEffect for FlexibleEffect {
    // Only valid in transfer mode
    fn source(&self) -> &Address {
        if self.mode != EffectMode::Transfer {
            panic!("Not in transfer mode");
        }
        &self.transfer_source
    }
    
    // ... other methods
}
```

### Constraint Adapters

Adapters can add constraint implementations to existing effects:

```rust
// Constraint adapter
pub struct TransferAdapter<E: Effect> {
    inner: E,
    source: Address,
    destination: Address,
    amount: Quantity,
    token: ResourceId,
}

impl<E: Effect> Effect for TransferAdapter<E> {
    // Delegate to inner effect
    fn name(&self) -> &str {
        self.inner.name()
    }
    
    // ... other delegations
}

impl<E: Effect> TransferEffect for TransferAdapter<E> {
    fn source(&self) -> &Address {
        &self.source
    }
    
    // ... other implementations
}
```

## Future Directions

The constraint system will continue to evolve:

1. **Effect Composition Operators**: Standard operators for composing effects
2. **Constraint-Based Optimization**: Optimize effects based on constraints
3. **Static Analysis Tools**: Verify constraint correctness statically
4. **Cross-Constraint Validation**: Validate interactions between constraints
5. **Domain-Specific Extensions**: Standardized extension points for domains 