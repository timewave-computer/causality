# Three-Layer Effect Architecture

*This document is derived from [ADR-023](../../../spec/adr_023_domain_adapter_effect_handler_unification.md), [ADR-032](../../../spec/adr_032_consolidated_agent_resource_system.md), and the [System Contract](../../../spec/system_contract.md).*

*Last updated: 2023-09-05*

## Overview

The Three-Layer Effect Architecture is a fundamental architectural pattern in Causality that provides a clean separation of concerns for effect handling while enabling powerful abstractions for cross-domain operations. This document explains the architecture, its benefits, and how it's implemented in the codebase.

## Architectural Layers

The architecture consists of three distinct layers, each with specific responsibilities:

```
┌─────────────────────────────────────────────────────────────────┐
│                  1. Algebraic Effect Layer                      │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────────────┐   │
│  │   Effect    │   │Continuation │   │Effect Composition   │   │
│  │   Trait     │   │    Model    │   │    Operators        │   │
│  └─────────────┘   └─────────────┘   └─────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                 2. Effect Constraints Layer                     │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────────────┐   │
│  │  Resource   │   │ Capability  │   │Cross-Domain         │   │
│  │Requirements │   │Requirements │   │   Validation        │   │
│  └─────────────┘   └─────────────┘   └─────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                3. Domain Implementation Layer                   │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────────────┐   │
│  │   Domain    │   │  Effect     │   │Zero-Knowledge       │   │
│  │  Adapters   │   │  Handlers   │   │    Integration      │   │
│  └─────────────┘   └─────────────┘   └─────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 1. Algebraic Effect Layer

The top layer defines the core abstractions for effects and provides the interfaces and types that programs interact with:

#### Effect Trait

The `Effect` trait defines the interface for all effects:

```rust
/// Effect interface defining common operations
pub trait Effect<R>: ContentAddressed {
    /// Execute the effect with the given handler
    fn execute(self, handler: &dyn EffectHandler) -> EffectOutcome<R>;
    
    /// Get the effect's unique identifier
    fn effect_id(&self) -> EffectId;
    
    /// Get the resources this effect requires
    fn resources(&self) -> Vec<ResourceId>;
    
    /// Get the capabilities required for this effect
    fn required_capabilities(&self) -> Vec<Capability>;
    
    /// Compose with another effect
    fn and_then<U, F>(self, f: F) -> ComposedEffect<Self, F, R, U>
    where
        F: FnOnce(R) -> Box<dyn Effect<U>>,
        Self: Sized;
}
```

#### Continuation Model

The continuation model allows for complex effect compositions:

```rust
/// Continuation trait for effect chaining
pub trait Continuation<I, O>: ContentAddressed {
    /// Apply the continuation to an input value
    fn apply(self: Box<Self>, input: I) -> O;
}

/// Simple function continuation
pub struct FnContinuation<I, O, F: FnOnce(I) -> O + Send + 'static> {
    f: F,
}

impl<I, O, F: FnOnce(I) -> O + Send + 'static> Continuation<I, O> for FnContinuation<I, O, F> {
    fn apply(self: Box<Self>, input: I) -> O {
        (self.f)(input)
    }
}
```

#### Effect Composition Operators

The algebraic layer provides operators for composing effects:

```rust
/// Sequence two effects
pub fn sequence<R1, R2, E1: Effect<R1>, E2: Effect<R2>>(
    first: E1,
    second: E2
) -> SequencedEffect<E1, E2, R1, R2> {
    SequencedEffect::new(first, second)
}

/// Run two effects in parallel
pub fn parallel<R1, R2, E1: Effect<R1>, E2: Effect<R2>>(
    first: E1,
    second: E2
) -> ParallelEffect<E1, E2, R1, R2> {
    ParallelEffect::new(first, second)
}

/// Map an effect's result
pub fn map<R1, R2, E: Effect<R1>, F: FnOnce(R1) -> R2 + Send + 'static>(
    effect: E,
    f: F
) -> MappedEffect<E, R1, R2, F> {
    MappedEffect::new(effect, f)
}
```

### 2. Effect Constraints Layer

The middle layer enforces constraints and validates that effects are used correctly:

#### Resource Requirements

Effects declare the resources they need access to:

```rust
/// Get resources required by an effect
pub fn get_required_resources<E: Effect<R>, R>(
    effect: &E
) -> Vec<ResourceId> {
    effect.resources()
}

/// Check if effect has required resources
pub fn validate_resources<E: Effect<R>, R>(
    effect: &E,
    available_resources: &[ResourceId]
) -> Result<(), ResourceError> {
    for resource in effect.resources() {
        if !available_resources.contains(&resource) {
            return Err(ResourceError::MissingResource(resource));
        }
    }
    Ok(())
}
```

#### Capability Requirements

Effects declare the capabilities required for execution:

```rust
/// Capability validation for effects
pub struct CapabilityValidator {
    registry: Arc<CapabilityRegistry>,
}

impl CapabilityValidator {
    /// Validate capabilities for an effect
    pub async fn validate<E: Effect<R>, R>(
        &self,
        effect: &E,
        agent_id: &AgentId
    ) -> Result<(), CapabilityError> {
        for capability in effect.required_capabilities() {
            self.registry.verify_capability(
                agent_id,
                &effect.resources()[0],
                capability
            ).await?;
        }
        Ok(())
    }
}
```

#### Cross-Domain Validation

Effects that cross domains undergo special validation:

```rust
/// Cross-domain effect validator
pub struct CrossDomainValidator {
    domain_registry: Arc<DomainRegistry>,
}

impl CrossDomainValidator {
    /// Validate cross-domain operations
    pub async fn validate<E: Effect<R>, R>(
        &self,
        effect: &E
    ) -> Result<(), CrossDomainError> {
        if !effect.crosses_domains() {
            return Ok(());
        }
        
        let domains = effect.involved_domains();
        
        // Check that all domains are valid
        for domain_id in &domains {
            if !self.domain_registry.has_domain(domain_id) {
                return Err(CrossDomainError::InvalidDomain(domain_id.clone()));
            }
        }
        
        // Check domain compatibility
        self.domain_registry
            .check_cross_domain_compatibility(&domains)
            .await?;
            
        Ok(())
    }
}
```

### 3. Domain Implementation Layer

The bottom layer connects effects to concrete implementations:

#### Domain Adapters

Domain adapters connect effects to specific domains:

```rust
/// Domain adapter for handling effects in a specific domain
pub trait DomainAdapter: Send + Sync + 'static {
    /// Get the domain ID
    fn domain_id(&self) -> DomainId;
    
    /// Handle a domain-specific effect
    async fn handle_effect<R>(
        &self,
        effect: Box<dyn Effect<R>>,
        context: &EffectContext,
    ) -> Result<EffectOutcome<R>, EffectError>;
    
    /// Check if this domain can handle a specific effect type
    fn can_handle(&self, effect_type: &EffectType) -> bool;
    
    /// Get resource accessor for this domain
    fn resource_accessor(&self) -> Box<dyn ResourceAccessor>;
}
```

#### Effect Handlers

Effect handlers implement the logic for executing effects:

```rust
/// Handler for effect execution
pub trait EffectHandler: Send + Sync + 'static {
    /// Handle a specific effect
    fn handle<R>(&self, effect: Box<dyn Effect<R>>) -> Result<EffectOutcome<R>, EffectError>;
    
    /// Get the effect types this handler can handle
    fn supported_effects(&self) -> Vec<EffectType>;
}

/// Handler for domain-specific resource effects
pub struct ResourceEffectHandler {
    resource_manager: Arc<ResourceManager>,
}

impl EffectHandler for ResourceEffectHandler {
    fn handle<R>(&self, effect: Box<dyn Effect<R>>) -> Result<EffectOutcome<R>, EffectError> {
        if let Some(resource_effect) = effect.downcast_ref::<GetResourceEffect<R>>() {
            // Handle resource acquisition
            let guard = self.resource_manager.acquire(
                resource_effect.resource_id.clone(),
                resource_effect.mode
            )?;
            
            let result = resource_effect.continuation.apply(guard);
            return Ok(EffectOutcome::Success(result));
        }
        
        Err(EffectError::UnsupportedEffect)
    }
    
    fn supported_effects(&self) -> Vec<EffectType> {
        vec![
            EffectType::of::<GetResourceEffect<()>>(),
            EffectType::of::<ReleaseResourceEffect<()>>(),
        ]
    }
}
```

#### Zero-Knowledge Integration

Effects support zero-knowledge proof generation and verification:

```rust
/// Zero-knowledge effect trait
pub trait ZkEffect<R>: Effect<R> {
    /// Generate a circuit for the effect
    fn generate_circuit(&self) -> Result<Circuit, CircuitError>;
    
    /// Generate a witness for the effect
    fn generate_witness(&self, inputs: &[Value]) -> Result<Witness, WitnessError>;
    
    /// Verify the effect's execution
    fn verify_execution(
        &self, 
        proof: &Proof, 
        public_inputs: &[Value]
    ) -> Result<bool, VerificationError>;
}

/// Zero-knowledge effect handler
pub struct ZkEffectHandler {
    prover: Arc<Prover>,
    verifier: Arc<Verifier>,
}

impl EffectHandler for ZkEffectHandler {
    fn handle<R>(&self, effect: Box<dyn Effect<R>>) -> Result<EffectOutcome<R>, EffectError> {
        if let Some(zk_effect) = effect.downcast_ref::<dyn ZkEffect<R>>() {
            // Generate circuit
            let circuit = zk_effect.generate_circuit()?;
            
            // Generate witness
            let witness = zk_effect.generate_witness(&[])?;
            
            // Generate proof
            let proof = self.prover.generate_proof(&circuit, &witness)?;
            
            // Verify proof
            let valid = self.verifier.verify_proof(&circuit, &proof, &[])?;
            if !valid {
                return Err(EffectError::InvalidProof);
            }
            
            // Execute the effect
            // ...
        }
        
        Err(EffectError::UnsupportedEffect)
    }
}
```

## Effect Execution Flow

The execution of an effect in the three-layer architecture involves the following steps:

1. An effect is created in the algebraic layer with a specific operation and continuation
2. The effect is validated in the constraints layer against:
   - Required resources
   - Required capabilities
   - Type constraints
   - Cross-domain validation
3. The effect is executed in the domain implementation layer:
   - Domain adapter routes to appropriate handler
   - Handler performs actual operation
   - Result is passed to continuation
   - Resources are released
4. The final result is returned to the caller

```rust
/// Execute an effect with full validation and resource management
pub async fn execute_effect<E: Effect<R>, R>(
    effect: E,
    context: &EffectContext,
    engine: &EffectEngine,
) -> Result<R, EffectError> {
    // 1. Validate the effect (Constraints Layer)
    validate_effect(&effect, context, &engine.validator).await?;
    
    // 2. Acquire resources (Constraints Layer)
    let resource_guards = acquire_resources(
        &effect.resources(),
        effect.required_capabilities(),
        &engine.resource_manager
    ).await?;
    
    // 3. Execute the effect (Domain Implementation Layer)
    let outcome = engine.execute_effect(effect, context).await?;
    
    // 4. Process the outcome (Algebraic Layer)
    match outcome {
        EffectOutcome::Success(result) => Ok(result),
        EffectOutcome::Error(error) => Err(error),
        EffectOutcome::NeedsContext(ctx_req) => {
            // Handle context request
            let additional_context = resolve_context_request(ctx_req).await?;
            execute_effect_with_context(effect, additional_context, engine).await
        },
        EffectOutcome::Pending(pending) => {
            // Handle pending effect
            wait_for_pending_effect(pending).await
        },
    }
}
```

## Integration with Agents

The Three-Layer Effect Architecture is tightly integrated with the Agent-Based Resource System:

1. Agents use the algebraic layer to define and compose effects
2. The constraints layer validates agent capabilities and resource access
3. The domain implementation layer connects agents to specific domains

```rust
impl Agent {
    /// Execute an operation with effects
    pub fn execute_operation(
        &self,
        operation: Operation
    ) -> impl Effectful<OperationResult, ResourceEffect + CapabilityEffect> {
        Effectful::new(move |ctx| {
            // 1. Verify capabilities (constraints layer)
            for cap_type in &operation.required_capabilities {
                ctx.perform(VerifyCapability { 
                    capability: self.find_capability(*cap_type, &operation.target_resource)?,
                    resource_id: operation.target_resource.clone(),
                })?;
            }
            
            // 2. Acquire resources (constraints layer)
            let resource = ctx.perform(GetResource { 
                id: operation.target_resource.clone(), 
                lock_mode: LockMode::Write 
            })?;
            
            // 3. Execute operation (domain implementation layer)
            let result = resource.execute_operation(
                &operation.action,
                &operation.parameters
            )?;
            
            // 4. Return result (algebraic layer)
            Ok(result)
        })
    }
}
```

## Cross-Domain Integration

The Three-Layer Effect Architecture enables cross-domain operations:

1. The algebraic layer defines cross-domain effects
2. The constraints layer validates cross-domain access
3. The domain implementation layer connects to multiple domains

```rust
/// Cross-domain transfer effect
pub struct CrossDomainTransferEffect<R> {
    source_domain: DomainId,
    target_domain: DomainId,
    asset: Asset,
    amount: Amount,
    continuation: Box<dyn Continuation<TransferResult, R>>,
}

impl<R> Effect<R> for CrossDomainTransferEffect<R> {
    fn crosses_domains(&self) -> bool {
        true
    }
    
    fn involved_domains(&self) -> Vec<DomainId> {
        vec![self.source_domain.clone(), self.target_domain.clone()]
    }
    
    fn execute(self, handler: &dyn EffectHandler) -> EffectOutcome<R> {
        // Execute via cross-domain handler
        // ...
    }
}

/// Cross-domain adapter
pub struct CrossDomainAdapter {
    adapters: HashMap<DomainId, Arc<dyn DomainAdapter>>,
    bridge: Arc<DomainBridge>,
}

impl CrossDomainAdapter {
    /// Handle cross-domain transfer
    pub async fn handle_transfer<R>(
        &self,
        effect: CrossDomainTransferEffect<R>
    ) -> Result<EffectOutcome<R>, EffectError> {
        // 1. Lock source asset
        let source_adapter = self.adapters.get(&effect.source_domain)
            .ok_or(EffectError::UnknownDomain(effect.source_domain.clone()))?;
            
        let lock_effect = LockAssetEffect {
            asset: effect.asset.clone(),
            amount: effect.amount,
            continuation: Box::new(|_| ()),
        };
        
        source_adapter.handle_effect(Box::new(lock_effect), &EffectContext::default()).await?;
        
        // 2. Create proof of lock
        let proof = self.bridge.generate_proof(
            effect.source_domain.clone(),
            effect.asset.clone(),
            effect.amount
        ).await?;
        
        // 3. Create asset in target domain
        let target_adapter = self.adapters.get(&effect.target_domain)
            .ok_or(EffectError::UnknownDomain(effect.target_domain.clone()))?;
            
        let create_effect = CreateAssetWithProofEffect {
            asset: effect.asset.clone(),
            amount: effect.amount,
            proof,
            continuation: Box::new(|_| ()),
        };
        
        target_adapter.handle_effect(Box::new(create_effect), &EffectContext::default()).await?;
        
        // 4. Return result via continuation
        let result = TransferResult {
            source_domain: effect.source_domain,
            target_domain: effect.target_domain,
            asset: effect.asset,
            amount: effect.amount,
            status: TransferStatus::Completed,
        };
        
        Ok(EffectOutcome::Success(effect.continuation.apply(result)))
    }
}
```

## Benefits of the Three-Layer Architecture

The Three-Layer Effect Architecture provides several key benefits:

1. **Separation of Concerns**: Each layer has a clear responsibility
2. **Type Safety**: The algebraic layer ensures type safety for effects
3. **Composability**: Effects can be composed into complex pipelines
4. **Validation**: The constraints layer ensures effects are used correctly
5. **Domain Abstraction**: The domain implementation layer abstracts away domain details
6. **Cross-Domain Operations**: The architecture enables seamless cross-domain operations
7. **ZK Integration**: Effects can be integrated with zero-knowledge proofs
8. **Resource Safety**: Resources are managed safely with RAII guards
9. **Capability-Based Security**: Effects are authorized via capabilities

## Where Implemented

The Three-Layer Effect Architecture is implemented across several crates:

| Component | Crate | Module |
|-----------|-------|--------|
| Algebraic Layer | `causality-core` | `causality_core::effect` |
| Effect Traits | `causality-core` | `causality_core::effect::traits` |
| Continuations | `causality-core` | `causality_core::effect::continuation` |
| Constraints Layer | `causality-core` | `causality_core::effect::constraints` |
| Capability Validation | `causality-core` | `causality_core::capability::validation` |
| Domain Implementation | `causality-core` | `causality_core::effect::domain` |
| Domain Adapters | `causality-domain` | `causality_domain::adapters` |
| ZK Integration | `causality-zkvm` | `causality_zkvm::effect` |

## References

- [ADR-023: Three-Layer Effect Architecture with TEL Integration](../../../spec/adr_023_domain_adapter_effect_handler_unification.md)
- [ADR-032: Agent-Based Resource System](../../../spec/adr_032_consolidated_agent_resource_system.md)
- [Effect System](./effect-system.md)
- [Domain System](./domain-system.md)
- [System Contract](../../../spec/system_contract.md) 