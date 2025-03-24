# ADR-021: Unified Resource-Register Model with Storage Effects

## Status

Accepted

## Implementation Status

This ADR has been fully implemented. The unified ResourceRegister model is now the standard approach for resource management in Causality. Key implementation components include:

- Unified ResourceRegister data structure that combines logical and physical properties
- Storage effect system integrated with the algebraic effect system
- Flexible storage strategies for different domain requirements
- Integration with the capability-based security model

The implementation preserves backward compatibility while providing a cleaner abstraction for developers. Documentation is available in [docs/src/resource_register_unified_model.md](/docs/src/resource_register_unified_model.md).

## Context

Currently, Causality implements two distinct abstractions for representing assets across domains:

1. **Resources**: Logical abstractions representing assets with properties (fungibility domain, quantity, etc.)
2. **Registers**: Physical on-chain state with commitments, nullifiers, and proofs

This separation creates several challenges:

- **Mental Overhead**: Developers must constantly map between these two abstractions
- **Redundant Logic**: Operations require coordinating changes across both systems
- **Error-Prone Coordination**: Changes must be synchronized correctly across abstractions
- **Conceptual Disconnect**: The logical resource does not directly correspond to its physical storage
- **Implementation Complexity**: Cross-domain operations require maintaining two parallel models

Additionally, on-chain storage strategies vary widely across domains, with some requiring direct state access (e.g., for EVM contract interoperability) while others benefit from commitment-based private storage.

## Decision

We will implement two related changes to simplify this architecture:

1. **Unify Resources and Registers into a single ResourceRegister abstraction**
2. **Represent on-chain storage as explicit Effects in our algebraic effect system**

### Unified ResourceRegister Abstraction

The unified model will combine both logical resource properties and physical register characteristics:

```rust
struct ResourceRegister {
    // Identity
    id: RegisterId,
    
    // Logical properties (previously in Resource)
    resource_logic: ResourceLogic,
    fungibility_domain: FungibilityDomain,
    quantity: Quantity,
    metadata: Value,
    
    // Physical properties (previously in Register)
    state: RegisterState,
    nullifier_key: NullifierKey,
    
    // Provenance tracking
    controller_label: ControllerLabel,
    
    // Temporal context
    observed_at: TimeMapSnapshot,
    
    // Additional fields as needed
}
```

This unified approach will:
- Provide a single coherent abstraction for developers
- Ensure logical and physical representations stay in sync
- Simplify cross-domain operations
- Reduce duplication in the codebase

### Storage as an Effect

Storage operations will be modeled as explicit effects in our algebraic effect system:

```rust
enum StorageEffect<R> {
    // New storage effects
    StoreOnChain {
        register_id: RegisterId,
        fields: HashSet<FieldName>,
        domain_id: DomainId,
        continuation: Box<dyn Continuation<StoreResult, R>>,
    },
    
    ReadFromChain {
        register_id: RegisterId,
        fields: HashSet<FieldName>,
        domain_id: DomainId,
        continuation: Box<dyn Continuation<ReadResult, R>>,
    },
    
    StoreCommitment {
        register_id: RegisterId,
        commitment: Commitment,
        domain_id: DomainId,
        continuation: Box<dyn Continuation<StoreResult, R>>,
    },
    
    // Other storage effects...
}
```

Storage strategies will be explicit choices rather than implicit in the data structure:

```rust
enum StorageStrategy {
    // Full on-chain storage - all fields available to EVM
    FullyOnChain {
        visibility: StateVisibility,
    },
    
    // Commitment-based with ZK proofs - minimal on-chain footprint
    CommitmentBased {
        commitment: Option<Commitment>,
        nullifier: Option<NullifierId>,
    },
    
    // Hybrid - critical fields on-chain, others as commitments
    Hybrid {
        on_chain_fields: HashSet<FieldName>,
        remaining_commitment: Option<Commitment>,
    },
}
```

This approach will:
- Make storage operations explicit IO boundaries
- Allow different domains to implement storage differently
- Support diverse on-chain availability requirements
- Bring storage operations into our existing effect handler infrastructure

## Consequences

### Positive

1. **Simplified Mental Model**: Developers work with a single `ResourceRegister` abstraction
2. **Reduced Cognitive Load**: No need to constantly translate between resource and register concepts
3. **Improved Cross-Domain Operations**: Single unified type crossing domain boundaries
4. **Storage Flexibility**: Different domains can implement storage strategies as needed
5. **Consistent Programming Model**: WHAT (ResourceRegister) is separate from HOW (storage effect) and WHERE (domain)
6. **Better Testability**: Storage effects can be easily mocked for testing
7. **Clear IO Boundaries**: Explicit effects for all storage operations

### Negative

1. **Data Structure Size**: Unified structure may be larger than either individual structure
2. **Migration Cost**: Substantial refactoring required to implement the unified model
3. **Performance Considerations**: May need optimization for common operations

### Neutral

1. **API Changes**: Public APIs will need to be updated to use the unified model
2. **Documentation Updates**: Comprehensive updates required to reflect new model

## Implementation Plan

1. **Phase 1**: Create a facade layer that presents a unified API while delegating to existing components
   ```rust
   struct ResourceRegister {
       resource: Resource,
       register: Register,
   }
   ```

2. **Phase 2**: Implement storage effects in the effect system
   - Add new effect types for storage operations
   - Implement handlers for these effects in domain adapters
   - Update existing code to use storage effects

3. **Phase 3**: Gradually refactor core components to use the unified model
   - Replace dual processing with unified operations
   - Convert validation logic to use the unified model
   - Update domain adapters to work with the unified type

4. **Phase 4**: Complete migration and cleanup
   - Remove redundant abstractions
   - Update documentation
   - Optimize performance

## Examples

### Current (Separated) Model

```rust
// Create a resource
let resource = Resource::new(logic, domain, quantity);
resource_manager.create_resource(resource)?;

// Create a register to hold it
let register = Register::new();
register_system.create_register(register.id, resource.id)?;

// Update - requires coordinating both systems
// Step 1: Update the resource
let new_resource = resource.with_quantity(new_amount);
resource_manager.update_resource(resource.id, new_resource)?;

// Step 2: Update the register
register_system.update_register(register.id, new_resource.id)?;
```

### Unified Model with Storage Effects

```rust
// Create a resource register
let token = ResourceRegister::new(logic, domain, quantity);

// Store it on-chain as appropriate for the domain
effect_system.execute_effect(StorageEffect::StoreOnChain {
    register_id: token.id,
    fields: token.all_fields(),
    domain_id: ethereum_domain,
    continuation: Box::new(|result| {
        // Handle result
        println!("Storage result: {:?}", result)
    }),
}).await?;

// Update quantity
token.update_quantity(new_amount)?;

// Update on-chain storage
effect_system.execute_effect(StorageEffect::StoreOnChain {
    register_id: token.id,
    fields: HashSet::from([String::from("quantity")]),  // Just update quantity
    domain_id: ethereum_domain,
    continuation: Box::new(|result| {
        // Handle result
        println!("Update result: {:?}", result)
    }),
}).await?;
```

## References

1. [ADR-003: Resource System](./adr_003_resource.md)
2. [ADR-006: ZK-Based Register System](./adr_006_zk_registers.md)
3. [ADR-001: Effects Library](./adr_001_effects.md)