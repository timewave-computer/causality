# Time System as an Effect

This implementation refactors the time module to use the Effect pattern, allowing time-related operations to be processed through the effect system. 

## Components

1. **TimeEffect Types** (in `time_snapshot.rs`):
   - `TimeEffect`: An enum representing different time-related operations
     - `CausalUpdate`: Updates causal relationships between operations
     - `ClockAttestation`: Records a timestamp attestation from a domain
     - `TimeMapUpdate`: Updates the entire time map with domain positions
   - `AttestationSource`: Sources of time attestations with different trust levels
   - `TimeEffectResult`: Results of time effect operations

2. **Effect Handler Interface** (in `time_effect_system.rs`):
   - `TimeEffectHandler`: Trait for handling time-related effects
     - `handle_causal_update`: Handle causal time updates
     - `handle_clock_attestation`: Handle clock time attestations
     - `handle_time_map_update`: Handle time map updates
   - `TimeEffectHandlerWrapper`: A wrapper for integrating with the effect system

3. **Time Service Implementation** (in `time_effect_system.rs`):
   - `TimeService`: Concrete implementation of the `TimeEffectHandler` trait
   - `TimeMap`: A map for tracking relative positions across domains
   - `DomainPosition`: Position within a domain's time

## Integration with Effect System

The `TimeEffectHandlerWrapper` acts as an adapter to the general effect system, converting between the general effect interface and the specific `TimeEffectHandler` interface.

## Usage Example

```rust
// Create a time service
let time_service = Arc::new(TimeService::new());

// Create the effect handler wrapper
let effect_handler = TimeEffectHandlerWrapper::new(time_service);

// Create a time effect
let effect = TimeEffect::ClockAttestation {
    domain_id: "domain1".to_string(),
    timestamp: current_time(),
    source: AttestationSource::Operator {
        operator_id: "trusted-operator".to_string(),
        signature: "signature-data".to_string(),
    },
    confidence: 0.95,
};

// Handle the effect
let result = effect_handler.handle(effect).await?;

// Use the result
match result {
    TimeEffectResult::ClockUpdate { domain_id, timestamp, confidence } => {
        println!("Updated domain {} to timestamp {} with confidence {}", 
            domain_id, timestamp, confidence);
    },
    _ => (),
}
```

## Benefits of This Approach

1. **Separation of Concerns**: Time-related operations are now cleanly separated and handled through effects
2. **Modularity**: The time system can be extended with new effect types without changing the core interface
3. **Testability**: The effect-based approach makes it easier to test time operations in isolation
4. **Integration with Other Systems**: Effects can trigger cross-system changes in a structured way

## Implementation Notes

This implementation is designed to work with the existing effect system but is structured to avoid cyclic dependencies between the domain and effects crates. The time effect system is implemented in the types crate so that it can be used by both domain and effects crates without creating cycles. 