# Time as an Effect System Guide

This guide explains how to use the Time as an Effect system in Causality applications.

## Overview

The Time as an Effect system models both causal time and clock time as effects within the Effect system. This enables composition, validation, and processing of time-related operations through the unified Effect system framework.

## Key Concepts

### Causal Time vs. Clock Time

1. **Causal Time**: The logical ordering of operations that establishes "happens-before" relationships between events. This is represented by logical clocks and vector clocks.

2. **Clock Time**: Attestations from external sources about when events occurred in real, physical time. This is represented by wall clock timestamps with attestation sources.

### Time Effects

The Time as an Effect system provides three main effect types:

1. **CausalTimeEffect**: For updating logical clocks and establishing causal dependencies between events.
2. **ClockTimeEffect**: For recording wall clock time attestations from various sources.
3. **TemporalQueryEffect**: For querying temporal relationships between facts, such as "happened before" or temporal distance.

### Time Services

1. **CausalTimeService**: Manages logical clocks and vector clocks for domains.
2. **ClockTimeService**: Manages wall clock time and attestations.
3. **TimeService**: Combines both services and provides higher-level temporal operations.

## Setting Up the Time Effect System

### Basic Setup

To set up the Time Effect system in your application:

```rust
use causality_core::{
    crypto::KeyPair,
    effect::registry::EffectRegistryBuilder,
    time::integration::TimeEffectIntegration,
};

// Generate a key pair for signing time attestations
let key_pair = KeyPair::generate()?;

// Create a time effect integration with memory-based services
let time_integration = TimeEffectIntegration::new_memory_based(key_pair);

// Create an effect registry and register time effect handlers
let mut registry_builder = EffectRegistryBuilder::new();
time_integration.register_handlers(&mut registry_builder);
let registry = registry_builder.build();

// Now you can use time_integration to create time effects
// and registry to execute them
```

### Custom Time Services

You can also use custom implementations of the time services:

```rust
use causality_core::{
    time::{
        service::{CausalTimeService, ClockTimeService, TimeService},
        integration::TimeEffectIntegration,
    },
};
use std::sync::Arc;

// Create custom time services
let causal_time_service = Arc::new(MyCustomCausalTimeService::new());
let clock_time_service = Arc::new(MyCustomClockTimeService::new());
let time_service = Arc::new(MyCustomTimeService::new(
    causal_time_service.clone(),
    clock_time_service.clone(),
));

// Create time stores
let attestation_store = Arc::new(MyCustomAttestationStore::new());
let fact_store = Arc::new(MyCustomFactTimeStore::new());

// Create time integration with custom services
let time_integration = TimeEffectIntegration::new(
    time_service,
    attestation_store,
    fact_store,
);
```

## Using Time Effects

### Creating and Executing Causal Time Effects

```rust
use causality_core::types::{DomainId, FactId};

// Create a domain ID
let domain_id = DomainId::new("my_domain");

// Create fact IDs for dependencies
let fact1 = FactId::new("fact1");
let fact2 = FactId::new("fact2");

// Create a causal time effect
let causal_effect = time_integration.create_causal_time_effect(
    domain_id.clone(),
    vec![fact1, fact2],
).await?;

// Create an effect context
let context = EffectContext::new();

// Execute the effect
let outcome = registry.execute(&causal_effect, &context).await?;
```

### Creating and Executing Clock Time Effects

```rust
// Create a clock time effect for the domain
let clock_effect = time_integration.create_clock_time_effect(
    domain_id.clone(),
).await?;

// Execute the effect
let outcome = registry.execute(&clock_effect, &context).await?;
```

### Querying Temporal Relationships

```rust
// Create a happened-before query effect
let happened_before_effect = time_integration.create_happened_before_effect(
    domain_id.clone(),
    fact1.clone(),
    fact2.clone(),
);

// Execute the query effect
let outcome = registry.execute(&happened_before_effect, &context).await?;

// Parse the result
let result = outcome.result().as_object().unwrap();
let happened_before = result.get("result").unwrap().as_bool().unwrap();
```

### Combining Time Effects with Other Effects

```rust
use causality_core::{
    effect::EffectBuilder,
    time::integration::TimeEffectBuilderExt,
};

// Create a domain operation effect (example)
let domain_effect = create_domain_effect();

// Create a causal time effect
let causal_effect = time_integration.create_causal_time_effect(
    domain_id.clone(),
    vec![fact1],
).await?;

// Create a composite effect
let composite_effect = EffectBuilder::new("domain_operation_with_time")
    .add_effect(domain_effect)
    .add_causal_time_effect(causal_effect)
    .build();

// Execute the composite effect
let outcome = registry.execute(&composite_effect, &context).await?;
```

## Time Attestations

Time attestations provide a way to verify the source and authenticity of clock time claims:

```rust
use causality_core::time::effect::TimeSource;

// Get an attestation from a trusted source
let attestation = time_service.clock_time().get_time_attestation().await?;

// Verify the attestation
let is_valid = time_service.clock_time().verify_attestation(&attestation).await?;

// Check the source
match attestation.source {
    "system_time" => println!("Local system time"),
    "ntp" => println!("Network Time Protocol"),
    source if source.starts_with("trusted_") => println!("Trusted external source"),
    _ => println!("Unknown source"),
}
```

## Domain-Specific Time Management

Each domain in Causality can have its own logical clock and vector clock:

```rust
// Create multiple domain IDs
let domain1 = DomainId::new("domain1");
let domain2 = DomainId::new("domain2");

// Get logical clocks for domains
let clock1 = time_service.causal_time().get_logical_clock(&domain1).await?;
let clock2 = time_service.causal_time().get_logical_clock(&domain2).await?;

// Advance logical clock for a domain
let new_clock = time_service.causal_time().advance_logical_clock(&domain1).await?;

// Get vector clocks for domains
let vclock1 = time_service.causal_time().get_vector_clock(&domain1).await?;
let vclock2 = time_service.causal_time().get_vector_clock(&domain2).await?;
```

## Fact Timing

You can record and query timing information for facts:

```rust
use chrono::Utc;

// Record logical time for a fact
fact_store.record_logical_time(
    &fact_id,
    &domain_id,
    42, // logical time
).await?;

// Record wall clock time for a fact
fact_store.record_wall_time(
    &fact_id,
    &domain_id,
    Utc::now(),
).await?;

// Record dependencies between facts
fact_store.record_dependencies(
    &fact_id,
    &[dependency1, dependency2],
).await?;

// Query fact timing
let logical_time = fact_store.get_logical_time(&fact_id, &domain_id).await?;
let wall_time = fact_store.get_wall_time(&fact_id, &domain_id).await?;
let dependencies = fact_store.get_dependencies(&fact_id).await?;
let dependents = fact_store.get_dependents(&fact_id).await?;
```

## Best Practices

1. **Use Causal Time for Ordering**: Rely on causal time (logical clocks) for establishing the order of operations within and across domains.

2. **Use Clock Time for External Interactions**: Use wall clock time with attestations for interactions with external systems or when real-time information is needed.

3. **Verify Time Attestations**: Always verify time attestations before trusting them, especially for sensitive operations.

4. **Combine Time Effects with Domain Effects**: Use the builder pattern to combine time effects with domain-specific effects for atomic operations.

5. **Record Fact Dependencies**: Explicitly record dependencies between facts to establish a clear causal history.

6. **Use Vector Clocks for Cross-Domain Causality**: When working across domains, use vector clocks to track causal relationships between events.

7. **Isolate Time Domains**: Each logical domain should have its own logical clock and vector clock to maintain proper causality.

## Examples

### Recording an Event with Time

```rust
async fn record_event_with_time(
    event_data: &[u8],
    domain_id: DomainId,
    time_integration: &TimeEffectIntegration,
    registry: &EffectRegistry,
) -> Result<FactId, Error> {
    // Create an event fact ID
    let fact_id = FactId::new_from_content(event_data);
    
    // Create a domain effect to store the event
    let store_effect = create_store_effect(event_data, &fact_id);
    
    // Create a causal time effect
    let causal_effect = time_integration.create_causal_time_effect(
        domain_id.clone(),
        vec![], // no dependencies for this event
    ).await?;
    
    // Create a clock time effect
    let clock_effect = time_integration.create_clock_time_effect(
        domain_id.clone(),
    ).await?;
    
    // Create a composite effect
    let composite_effect = EffectBuilder::new("record_event_with_time")
        .add_effect(store_effect)
        .add_causal_time_effect(causal_effect)
        .add_clock_time_effect(clock_effect)
        .build();
    
    // Create a context
    let context = EffectContext::new();
    
    // Execute the composite effect
    let outcome = registry.execute(&composite_effect, &context).await?;
    
    // Return the fact ID
    Ok(fact_id)
}
```

### Checking Causal Relationship Between Events

```rust
async fn are_causally_related(
    fact1: &FactId,
    fact2: &FactId,
    domain_id: &DomainId,
    time_service: &dyn TimeService,
) -> Result<bool, Error> {
    // Check if fact1 happened before fact2
    let happened_before = time_service.happened_before(fact1, fact2).await?;
    
    if happened_before {
        return Ok(true);
    }
    
    // Check if fact2 happened before fact1
    let happened_after = time_service.happened_before(fact2, fact1).await?;
    
    if happened_after {
        return Ok(true);
    }
    
    // If neither, they are concurrent (not causally related)
    Ok(false)
}
```

## Conclusion

The Time as an Effect system provides a powerful way to manage time in Causality applications, enabling both causal ordering and real-time attestations. By modeling time as effects, it integrates seamlessly with the rest of the Effect system, allowing for composition and consistent processing of time-related operations. 