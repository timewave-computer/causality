<!-- Execution of scripts -->
<!-- Original file: docs/src/script_execution.md -->

# Script Execution in the Unified Architecture

## Overview

Script execution in the unified Causality architecture leverages the Temporal Effect Language (TEL) to provide a robust, content-addressed, capability-based framework for defining and executing complex effects across multiple domains. This document explains how scripts are executed, verified, and integrated with the three-layer effect architecture and unified ResourceRegister model.

## Script Execution Pipeline

The script execution pipeline consists of several stages that transform a TEL script into executed effects:

```
┌───────────────┐     ┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│ TEL Script    │────>│ Compilation   │────>│ Validation    │────>│ Domain Binding│
│ Definition    │     │ & Parsing     │     │ & Constraint  │     │ & Resolution  │
└───────────────┘     └───────────────┘     └───────────────┘     └───────────────┘
                                                                         │
                                                                         ▼
┌───────────────┐     ┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│ Content       │<────│ Proof         │<────│ Capability    │<────│ Effect Graph  │
│ Commitment    │     │ Generation    │     │ Verification  │     │ Construction  │
└───────────────┘     └───────────────┘     └───────────────┘     └───────────────┘
        │                                                                  
        ▼                                                                  
┌───────────────┐     ┌───────────────┐     ┌───────────────┐             
│ Execution     │────>│ State         │────>│ Observation   │             
│ & Effects     │     │ Transition    │     │ & Logging     │             
└───────────────┘     └───────────────┘     └───────────────┘             
```

## Script Definition with TEL

TEL scripts are defined using a declarative syntax that specifies the effects to be performed:

```rust
// Simple transfer script
let transfer_script = tel! {
    transfer(
        from: sender_account,
        to: recipient_account,
        amount: 100,
        token: "ETH",
        domain: ethereum_domain
    )
}

// Cross-domain operation script
let cross_domain_script = tel! {
    sequence {
        // First transfer ETH on Ethereum
        transfer(
            from: eth_account,
            to: bridge_contract,
            amount: 100,
            token: "ETH",
            domain: eth_domain
        ),
        
        // Then mint tokens on CosmWasm chain
        mint(
            to: cosmos_account,
            amount: 100,
            token: "wETH",
            domain: cosmos_domain
        )
    }
}
```

## Script Compilation

The TEL compiler translates scripts into an intermediate representation:

1. **Parsing**: Converts the script syntax into an abstract syntax tree (AST)
2. **Type-checking**: Validates types and resolves identifiers
3. **Effect Graph Generation**: Creates a directed acyclic graph (DAG) of effects
4. **Optimization**: Performs initial optimizations on the effect graph

```rust
// Script compilation process
let effect_graph = tel_compiler.compile(script_source)?;
```

## Three-Layer Architecture Integration

Scripts integrate with the three-layer effect architecture:

### 1. Abstract Effect Layer

At this layer, scripts define what operations will be performed using abstract effect interfaces:

```rust
// Abstract effect definition in the script
pub trait TransferEffect: Effect {
    fn source(&self) -> &ContentRef<Address>;
    fn destination(&self) -> &ContentRef<Address>;
    fn amount(&self) -> &Quantity;
}
```

### 2. Effect Constraints Layer

The constraint system validates that the script's effects satisfy all constraints:

```rust
// Constraint validation during script execution
pub trait BalanceConstraint: Constraint {
    fn validate_transfer(&self, effect: &dyn TransferEffect) -> Result<(), ConstraintViolation>;
}

// Applying constraints to script effects
let constraint_violations = constraint_system.validate_graph(effect_graph)?;
if !constraint_violations.is_empty() {
    return Err(ScriptValidationError::ConstraintViolations(constraint_violations));
}
```

### 3. Domain Implementation Layer

Domain-specific implementations are bound to abstract effects:

```rust
// Domain binding for script effects
let bound_graph = domain_binder.bind_effects(effect_graph, domain_registry)?;

// Example of domain-specific implementation
impl TransferEffect for EvmTransferEffect {
    fn source(&self) -> &ContentRef<Address> {
        &self.source_address
    }
    
    fn destination(&self) -> &ContentRef<Address> {
        &self.destination_address
    }
    
    fn amount(&self) -> &Quantity {
        &self.transfer_amount
    }
}
```

## Integration with ResourceRegister

Scripts interact with the unified ResourceRegister model through ContentRef references:

```rust
// Script accessing ResourceRegisters
let tx = tel! {
    // Create a new resource register
    let register = create_resource_register(
        id: register_id,
        logic: ResourceLogic::Fungible,
        owner: account_address,
        quantity: 100,
        domain: eth_domain
    );
    
    // Store it using the StorageEffect
    store(
        register: register,
        fields: ["id", "quantity", "owner"],
        domain: eth_domain
    )
}
```

All ResourceRegister operations in scripts are converted to content-addressed effects:

```rust
pub struct ResourceRegisterEffect<C: ExecutionContext> {
    operation: Operation<C>,
    register_ref: ContentRef<ResourceRegister<C>>,
    operation_type: ResourceRegisterOperationType,
}

// ContentRef implementation ensures immutability and content addressing
pub struct ContentRef<T> {
    content_hash: Hash,
    content: Arc<T>,
}
```

## Capability-Based Authorization

Script execution leverages the capability-based authorization model:

```rust
// Script using capabilities
let tx = tel! {
    with_capability(transfer_capability) {
        transfer(
            from: account,
            to: recipient,
            amount: 100,
            token: "ETH",
            domain: eth_domain
        )
    }
}
```

Capability verification occurs during script execution:

```rust
// Capability verification
struct CapabilityVerifier;

impl CapabilityVerifier {
    fn verify_capability(&self, capability: &ContentRef<Capability>, operation: &dyn Effect) -> Result<(), CapabilityError> {
        // 1. Verify the capability hasn't expired
        if capability.has_expired() {
            return Err(CapabilityError::Expired);
        }
        
        // 2. Verify the capability grants rights for this operation
        if !capability.grants_rights_for(operation) {
            return Err(CapabilityError::InsufficientRights);
        }
        
        // 3. Verify the capability's signature is valid
        if !capability.verify_signature() {
            return Err(CapabilityError::InvalidSignature);
        }
        
        Ok(())
    }
}
```

## Content Addressing in Script Execution

All aspects of script execution use content addressing:

1. **Script Content Hash**: Scripts themselves are content-addressed
2. **Effect Content Hash**: Each effect in the script is content-addressed
3. **ResourceRegister Content Hash**: All resources referenced are content-addressed
4. **Capability Content Hash**: All capabilities are content-addressed

```rust
// Content addressing in script execution
let script_hash = ContentHasher::hash(script);
let effect_hash = ContentHasher::hash(effect);
let register_hash = ContentHasher::hash(resource_register);
let capability_hash = ContentHasher::hash(capability);

// Store script execution in content-addressed storage
content_addressed_storage.put(script_hash, script)?;
```

## Unified Verification Framework

Script execution integrates with the unified verification framework:

```rust
// Verification during script execution
struct UnifiedProof {
    zk_proof: Option<ZkProof>,
    temporal_proof: Option<TemporalProof>,
    capability_proof: Option<CapabilityProof>,
    logical_proof: Option<LogicalProof>,
}

// Generating proofs during script execution
let unified_proof = proof_generator.generate_unified_proof(
    effect_graph,
    verification_context
)?;

// Verifying proofs during script execution
let verification_result = verifier.verify_unified_proof(
    effect_graph,
    unified_proof,
    verification_context
)?;
```

## Cross-Domain Script Execution

Scripts can span multiple domains, with special handling for cross-domain operations:

```rust
// Cross-domain script execution
let cross_domain_script = tel! {
    // Define a cross-domain transfer
    cross_domain_transfer(
        source_domain: eth_domain,
        source_account: eth_account,
        target_domain: cosmos_domain,
        target_account: cosmos_account,
        amount: 100,
        token: "ETH"
    )
}

// Cross-domain handler
struct CrossDomainHandler;

impl CrossDomainHandler {
    fn handle_cross_domain_operation(&self, operation: &dyn CrossDomainEffect) -> Result<Vec<Effect>, CrossDomainError> {
        // 1. Extract source and target domain information
        let source_domain = operation.source_domain();
        let target_domain = operation.target_domain();
        
        // 2. Generate domain-specific effects for both domains
        let source_effects = self.generate_source_domain_effects(operation, source_domain)?;
        let target_effects = self.generate_target_domain_effects(operation, target_domain)?;
        
        // 3. Create verification proofs that link the effects
        let cross_domain_proof = self.generate_cross_domain_proof(
            operation, source_effects.clone(), target_effects.clone()
        )?;
        
        // 4. Combine all effects
        let mut all_effects = source_effects;
        all_effects.extend(target_effects);
        all_effects.push(cross_domain_proof);
        
        Ok(all_effects)
    }
}
```

## Execution Context

Scripts execute within an `ExecutionContext` that provides:

1. **Authentication information**: Who is executing the script
2. **Capabilities**: What operations are permitted
3. **Domain information**: Which domains are involved
4. **Temporal context**: When the script is executing
5. **Resource state**: Current state of referenced resources

```rust
// ExecutionContext for script execution
pub struct ExecutionContext {
    executor: ContentRef<Address>,
    timestamp: Timestamp,
    capabilities: Vec<ContentRef<Capability>>,
    domains: Vec<ContentRef<Domain>>,
    resource_state: HashMap<ContentRef<ResourceId>, ResourceState>,
}

// Using the context during execution
let result = effect_runtime.execute_graph(bound_graph, execution_context).await?;
```

## Script Lifecycle Events

Script execution generates lifecycle events for observation and auditing:

1. **ScriptCompiled**: Script was successfully compiled
2. **ConstraintsValidated**: All constraints were successfully validated
3. **DomainBound**: Effects were bound to domain-specific implementations
4. **CapabilitiesVerified**: All required capabilities were verified
5. **ProofsGenerated**: All required proofs were generated
6. **ExecutionStarted**: Script execution has started
7. **EffectApplied**: An individual effect was applied
8. **StateTransitionCompleted**: A state transition was completed
9. **ExecutionCompleted**: Script execution was completed
10. **ExecutionFailed**: Script execution failed

```rust
// Lifecycle events during script execution
pub enum ScriptLifecycleEvent {
    ScriptCompiled { script_hash: Hash, effect_graph: EffectGraph },
    ConstraintsValidated { script_hash: Hash },
    DomainBound { script_hash: Hash, bound_graph: BoundEffectGraph },
    CapabilitiesVerified { script_hash: Hash },
    ProofsGenerated { script_hash: Hash, proofs: Vec<UnifiedProof> },
    ExecutionStarted { script_hash: Hash, timestamp: Timestamp },
    EffectApplied { script_hash: Hash, effect_hash: Hash, result: EffectResult },
    StateTransitionCompleted { script_hash: Hash, state_hash: Hash },
    ExecutionCompleted { script_hash: Hash, result: ExecutionResult },
    ExecutionFailed { script_hash: Hash, error: ExecutionError },
}
```

## Best Practices for Script Development

1. **Use ContentRef for Immutable References**: Always use `ContentRef<T>` for references to ensure content-addressing.

2. **Define Clear Domain Boundaries**: Explicitly specify domains for all operations in your scripts.

3. **Leverage the Three-Layer Architecture**:
   - Abstract effects for domain-agnostic operations
   - Constraints for validation rules
   - Domain implementations for specific chains

4. **Handle Cross-Domain Operations Carefully**: Use the cross-domain handlers for operations that span multiple domains.

5. **Design for Verification**: Structure scripts to make verification straightforward.

6. **Use Capability-Based Authorization**: Explicitly include capability requirements in your scripts.

7. **Optimize for Composability**: Build scripts from smaller, reusable components.

8. **Test with Mock Domain Adapters**: Test scripts with mock domain adapters before deploying.

## Conclusion

Script execution in the unified Causality architecture leverages content addressing, capability-based authorization, and the three-layer effect architecture to provide a powerful, verifiable framework for defining and executing complex effects across domains. TEL serves as the declarative language that enables developers to express complex workflows while the underlying architecture handles the complexities of verification, authorization, and cross-domain operations.