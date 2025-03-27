# Effect System

*This document is derived from [ADR-001](../../../spec/adr_001_effects.md) and the [System Specification](../../../spec/system_contract.md).*

*Last updated: 2023-09-05*

## Overview

The Effect System in Causality provides a framework for modeling and executing operations in a controlled, composable manner. It enables programs to express and reason about both internal and external effects while maintaining referential transparency and enabling powerful abstractions for handling effects.

## Three-Layer Architecture

The Effect System is based on a three-layer architecture that unifies all operations under a consistent algebraic model. For a detailed explanation of this architecture, see the [Three-Layer Effect Architecture](./three-layer-effect-architecture.md) document.

### Algebraic Effect Layer

- **Effect Trait**: Core interface for all effect operations
- **Effect Identification**: All effects have a unique content-addressed ID
- **Continuation Model**: Effects use explicit continuations for composability
- **Effect Outcomes**: Standardized result types for all effects
- **Effect Composition**: Effects can be composed into complex pipelines
- **Error Handling**: Comprehensive typed error handling for all effects

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

/// Effect outcome type
pub enum EffectOutcome<T> {
    /// Effect completed successfully
    Success(T),
    /// Effect failed with error
    Error(EffectError),
    /// Effect requires additional context
    NeedsContext(ContextRequest<T>),
    /// Effect will continue asynchronously
    Pending(PendingEffect<T>),
}
```

### Effect Constraints Layer

- **Resource Requirements**: Effects declare the resources they need access to
- **Capability Requirements**: Effects declare the capabilities required for execution
- **Type Constraints**: Static typing ensures effects are used correctly
- **Validation Rules**: Effects undergo validation before execution
- **Concurrency Control**: Resource locking ensures safe concurrent access
- **Cross-Domain Validation**: Effects that cross domains undergo special validation

```rust
/// Validate an effect against constraints
pub async fn validate_effect<E: Effect<R>, R>(
    effect: &E,
    context: &EffectContext,
    validator: &dyn EffectValidator,
) -> Result<(), EffectError> {
    // Validate resource requirements
    for resource_id in effect.resources() {
        validator.validate_resource_access(
            resource_id, 
            context.caller_id(), 
            effect.required_capabilities()
        ).await?;
    }
    
    // Validate capabilities
    validator.validate_capabilities(
        context.caller_id(),
        effect.required_capabilities()
    ).await?;
    
    // Validate effect-specific constraints
    validator.validate_effect_constraints(effect).await?;
    
    // Validate cross-domain operations
    if effect.crosses_domains() {
        validator.validate_cross_domain(effect).await?;
    }
    
    Ok(())
}
```

### Domain Implementation Layer

- **Domain Adapters**: Domain-specific implementations of effect handlers
- **Effect Handlers**: Concrete implementations of effect execution logic
- **ZK Integration**: Effects support zero-knowledge proof generation and verification
- **Time Integration**: Effects can interact with the time system
- **Resource Management**: Effects manipulate resources safely through guards
- **Cross-Domain Operations**: Effects can operate across domain boundaries

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

/// Registry for domain adapters
pub struct DomainAdapterRegistry {
    adapters: HashMap<DomainId, Arc<dyn DomainAdapter>>,
}
```

## Resource-Scoped Concurrency

Resources are protected by explicit locks with deterministic wait queues:

```rust
/// Resource lock manager
pub struct ResourceLockManager {
    locks: Mutex<HashMap<ResourceId, LockEntry>>,
}

struct LockEntry {
    holder: Option<TaskId>,
    wait_queue: VecDeque<WaitingTask>,
}

/// Resource guard that auto-releases on drop (RAII pattern)
pub struct ResourceGuard {
    manager: Arc<ResourceLockManager>,
    resource: ResourceId,
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        self.manager.release(self.resource.clone());
    }
}

/// Effect for resource acquisition
pub struct AcquireResourceEffect<R> {
    resource_id: ResourceId,
    mode: AccessMode,
    continuation: Box<dyn Continuation<ResourceGuard, R>>,
}

impl<R> Effect<R> for AcquireResourceEffect<R> {
    fn execute(self, handler: &dyn EffectHandler) -> EffectOutcome<R> {
        let guard = handler.handle_acquire_resource(self.resource_id, self.mode)?;
        EffectOutcome::Success(self.continuation.apply(guard))
    }
    
    fn resources(&self) -> Vec<ResourceId> {
        vec![self.resource_id.clone()]
    }
}

/// Resource access modes
pub enum AccessMode {
    /// Read-only access
    Read,
    /// Read-write access
    Write,
    /// Exclusive access
    Exclusive,
}
```

## Explicit Continuation Model

Effects use explicit continuation objects for composability:

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

/// Continuation factory
pub struct ContinuationFactory;

impl ContinuationFactory {
    pub fn from_fn<I, O, F: FnOnce(I) -> O + Send + 'static>(f: F) -> Box<dyn Continuation<I, O>> {
        Box::new(FnContinuation { f })
    }
}

/// Example effect with continuation
pub struct DepositEffect<R> {
    domain: DomainId,
    asset: Asset,
    amount: Amount,
    continuation: Box<dyn Continuation<DepositResult, R>>,
}

impl<R> Effect<R> for DepositEffect<R> {
    fn execute(self, handler: &dyn EffectHandler) -> EffectOutcome<R> {
        let result = handler.handle_deposit(self.domain, self.asset, self.amount)?;
        EffectOutcome::Success(self.continuation.apply(result))
    }
}
```

## ZK-VM Integration

Effects support compilation to ZK-VM compatible code:

```rust
/// ZK-VM integration for effects
pub trait ZkVmEffect<R>: Effect<R> {
    /// Convert the effect to RISC-V code for ZK-VM execution
    fn to_risc_v<W: RiscVWriter>(&self, writer: &mut W) -> Result<(), RiscVError>;
    
    /// Generate a witness for the effect execution
    fn generate_witness(&self, inputs: &[Value]) -> Result<Witness, WitnessError>;
    
    /// Verify the effect execution with a proof
    fn verify_execution(&self, proof: &Proof) -> Result<bool, VerificationError>;
}

/// RISC-V code generator
pub struct RiscVGenerator {
    target: RiscVTarget,
    optimizations: Vec<Box<dyn RiscVOptimization>>,
}

impl RiscVGenerator {
    /// Generate RISC-V code for an effect
    pub fn generate_code<R>(&self, effect: &dyn ZkVmEffect<R>) -> Result<RiscVProgram, RiscVError> {
        let mut program = RiscVProgram::new();
        effect.to_risc_v(&mut program)?;
        
        // Apply optimizations
        for opt in &self.optimizations {
            opt.optimize(&mut program)?;
        }
        
        Ok(program)
    }
}

/// ZK-VM execution environment
pub struct ZkVmEnvironment {
    vm: ZkVm,
    verification_keys: HashMap<EffectType, VerificationKey>,
}

impl ZkVmEnvironment {
    /// Execute an effect in the ZK-VM
    pub async fn execute<R>(&self, effect: &dyn ZkVmEffect<R>) -> Result<(R, Proof), ZkVmError> {
        // Generate RISC-V code
        let program = RiscVGenerator::new(self.vm.target())
            .generate_code(effect)?;
        
        // Execute in VM
        let (result, witness) = self.vm.execute(&program)?;
        
        // Generate proof
        let proof = self.vm.generate_proof(&program, &witness)?;
        
        Ok((result, proof))
    }
    
    /// Verify an effect execution
    pub async fn verify<R>(&self, effect: &dyn ZkVmEffect<R>, proof: &Proof) -> Result<bool, ZkVmError> {
        let effect_type = effect.effect_type();
        let verification_key = self.verification_keys.get(&effect_type)
            .ok_or(ZkVmError::MissingVerificationKey(effect_type))?;
        
        // Verify the proof
        let valid = self.vm.verify_proof(verification_key, proof)?;
        
        Ok(valid)
    }
}
```

## Core Effect Types

The system includes a rich set of effect types:

```rust
/// Core effect enum - system-wide effects
pub enum CoreEffect<R> {
    // External effects
    Deposit {
        domain: DomainId,
        asset: Asset,
        amount: Amount,
        continuation: Box<dyn Continuation<DepositResult, R>>,
    },
    Withdraw {
        domain: DomainId,
        asset: Asset,
        amount: Amount,
        continuation: Box<dyn Continuation<WithdrawResult, R>>,
    },
    Transfer {
        from_program: ProgramId,
        to_program: ProgramId,
        asset: Asset,
        amount: Amount,
        continuation: Box<dyn Continuation<TransferResult, R>>,
    },
    
    // Fact observation effects
    ObserveFact {
        fact_id: FactId,
        continuation: Box<dyn Continuation<ObservationResult, R>>,
    },
    
    // Internal system effects
    AcquireResource {
        resource_id: ResourceId,
        mode: AccessMode,
        continuation: Box<dyn Continuation<ResourceGuard, R>>,
    },
    Invoke {
        target_program: ProgramId,
        invocation: Invocation,
        continuation: Box<dyn Continuation<InvocationResult, R>>,
    },
    EvolveSchema {
        old_schema: Schema,
        new_schema: Schema,
        continuation: Box<dyn Continuation<EvolutionResult, R>>,
    },
    
    // Time effects
    TimeEffect {
        time_operation: TimeOperation,
        continuation: Box<dyn Continuation<TimeResult, R>>,
    },
    
    // Zero-knowledge effects
    GenerateProof {
        statement: Statement,
        witness: Witness,
        continuation: Box<dyn Continuation<ProofResult, R>>,
    },
    VerifyProof {
        statement: Statement,
        proof: Proof,
        continuation: Box<dyn Continuation<VerificationResult, R>>,
    },
    
    // Content addressing effects
    ContentHash {
        data: Vec<u8>,
        continuation: Box<dyn Continuation<ContentHash, R>>,
    },
    VerifyContent {
        data: Vec<u8>,
        expected_hash: ContentHash,
        continuation: Box<dyn Continuation<VerificationResult, R>>,
    },
}
```

## Effect Execution Flow

The execution flow for effects involves several steps:

1. **Effect Creation**: An effect is created with a specific operation and continuation
2. **Validation**: The effect is validated against constraints
3. **Resource Acquisition**: Required resources are locked
4. **Execution**: The effect is executed via the appropriate handler
5. **Continuation Application**: The result is passed to the continuation
6. **Resource Release**: Resources are released via RAII guards
7. **Result Handling**: The final result is returned

```rust
/// Execute an effect with full validation and resource management
pub async fn execute_effect<E: Effect<R>, R>(
    effect: E,
    context: &EffectContext,
    engine: &EffectEngine,
) -> Result<R, EffectError> {
    // Validate the effect
    validate_effect(&effect, context, &engine.validator).await?;
    
    // Acquire resources
    let resource_guards = acquire_resources(
        &effect.resources(),
        effect.required_capabilities(),
        &engine.resource_manager
    ).await?;
    
    // Execute the effect (resource guards are automatically released when dropped)
    let outcome = engine.execute_effect(effect, context).await?;
    
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

## Where Implemented

The Effect System is implemented in the following crates and modules:

| Component | Crate | Module |
|-----------|-------|--------|
| Effect Traits | `causality-core` | `causality_core::effect` |
| Effect Handlers | `causality-core` | `causality_core::effect::handler` |
| Continuations | `causality-core` | `causality_core::effect::continuation` |
| Effect Engine | `causality-core` | `causality_core::effect::engine` |
| Resource Concurrency | `causality-core` | `causality_core::resource::concurrency` |
| ZK-VM Integration | `causality-zkvm` | `causality_zkvm::effect` |
| Domain Adapters | `causality-domains` | `causality_domains::adapters` |

## References

- [ADR-001: Rust Algebraic Effects Library](../../../spec/adr_001_effects.md)
- [System Contract](../../../spec/system_contract.md)
- [Resource System](./resource-system.md)
- [Domain System](./domain-system.md)
