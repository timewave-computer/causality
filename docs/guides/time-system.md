# Implementing the Time System

*This guide provides practical implementations for working with the [Time System](../../architecture/core/time-system.md).*

*Last updated: 2023-03-26*

## Overview

This guide covers the practical aspects of implementing and working with the Time System in Causality. It provides code examples, best practices, and implementation patterns for managing both causal time and clock time through the effect system.

## Prerequisites

Before implementing time-related functionality in your code, make sure you're familiar with:

- The [Time System Architecture](../../architecture/core/time-system.md)
- The [Effect System](../../architecture/core/effect-system.md)
- The [Resource System](../../architecture/core/resource-system.md)

## Implementation Guide

### Required Crates and Imports

```rust
// Core time types
use causality_types::{
    time::{
        effects::{CausalTimeEffect, ClockTimeEffect},
        TimeSource, TimeAttestation,
    },
    domain::DomainId,
    fact::FactId,
    common::ContentHash,
};

// Time services
use causality_core::{
    time::{
        TimeService, CausalTimeService, ClockTimeService,
        TimeServiceRegistry, TimeError,
    },
    effects::{EffectContext, EffectOutcome, EffectError},
};
```

### Setting Up Time Services

```rust
/// Set up time services for a domain
async fn setup_time_services(
    domain_id: DomainId,
) -> Result<Arc<dyn TimeService>, TimeError> {
    // Create causal time service
    let causal_service = Arc::new(DefaultCausalTimeService::new());
    
    // Create clock time service
    let clock_service = Arc::new(DefaultClockTimeService::new());
    
    // Create combined time service
    let time_service = Arc::new(DefaultTimeService::new(
        causal_service,
        clock_service,
    ));
    
    // Register services
    let mut registry = TimeServiceRegistry::new();
    registry.register_causal_service(domain_id.clone(), time_service.causal_time());
    registry.register_clock_service(domain_id, time_service.clock_time());
    
    Ok(time_service)
}
```

### Working with Causal Time

```rust
/// Create a causal time effect
async fn create_causal_time_effect(
    time_service: &dyn TimeService,
    domain_id: &DomainId,
    dependencies: Vec<FactId>,
) -> Result<CausalTimeEffect, TimeError> {
    // Get current logical clock
    let logical_clock = time_service.causal_time()
        .get_logical_clock(domain_id)
        .await?;
    
    // Get current vector clock
    let vector_clock = time_service.causal_time()
        .get_vector_clock(domain_id)
        .await?;
    
    // Create the effect
    Ok(CausalTimeEffect {
        domain_id: domain_id.clone(),
        logical_clock: logical_clock + 1,
        vector_clock_updates: vector_clock,
        dependencies,
    })
}

/// Handle a causal time effect
async fn handle_causal_time_effect(
    effect: &CausalTimeEffect,
    time_service: &dyn TimeService,
) -> Result<(), TimeError> {
    // Update logical clock
    time_service.causal_time()
        .update_logical_clock(
            &effect.domain_id,
            effect.logical_clock,
        ).await?;
    
    // Update vector clock
    if !effect.vector_clock_updates.is_empty() {
        time_service.causal_time()
            .update_vector_clock(
                &effect.domain_id,
                effect.vector_clock_updates.clone(),
            ).await?;
    }
    
    Ok(())
}
```

### Working with Clock Time

```rust
/// Create a clock time effect
async fn create_clock_time_effect(
    time_service: &dyn TimeService,
    domain_id: &DomainId,
) -> Result<ClockTimeEffect, TimeError> {
    // Get current time
    let wall_time = time_service.clock_time()
        .get_current_time()
        .await?;
    
    // Get time attestation
    let attestation = time_service.clock_time()
        .get_time_attestation()
        .await?;
    
    // Create the effect
    Ok(ClockTimeEffect {
        domain_id: domain_id.clone(),
        wall_time,
        time_source: TimeSource::LocalSystem,
        attestation: Some(attestation),
    })
}

/// Handle a clock time effect
async fn handle_clock_time_effect(
    effect: &ClockTimeEffect,
    time_service: &dyn TimeService,
) -> Result<(), TimeError> {
    // Verify attestation if provided
    if let Some(attestation) = &effect.attestation {
        let is_valid = time_service.clock_time()
            .verify_attestation(attestation)
            .await?;
        
        if !is_valid {
            return Err(TimeError::InvalidAttestation);
        }
    }
    
    // Update the domain's clock time
    // Implementation depends on how clock time is stored
    
    Ok(())
}
```

### Creating Facts with Time Effects

```rust
/// Create a fact with proper time effects
pub async fn create_fact_with_time_effects(
    domain: &Domain,
    fact_data: &[u8],
    dependencies: Vec<FactId>,
) -> Result<FactId, DomainError> {
    // Create the effect context
    let mut context = EffectContext::new();
    
    // Add causal time effect
    let causal_time_effect = domain.time_service()
        .causal_time()
        .create_causal_time_effect(
            domain.id(),
            dependencies.clone(),
        ).await?;
    
    context.add_effect(causal_time_effect);
    
    // Add clock time effect if needed
    let clock_time_effect = domain.time_service()
        .clock_time()
        .create_clock_time_effect(
            domain.id(),
        ).await?;
    
    context.add_effect(clock_time_effect);
    
    // Add the main fact effect
    let fact_effect = FactCreationEffect {
        domain_id: domain.id().clone(),
        data: fact_data.to_vec(),
        dependencies: dependencies.clone(),
    };
    
    context.add_effect(fact_effect);
    
    // Execute all effects as a transaction
    let result = domain.effect_engine()
        .execute_transaction(context)
        .await?;
    
    // Extract and return the fact ID
    let fact_id = result.get("fact_id")
        .ok_or_else(|| DomainError::MissingData("fact_id not in result".to_string()))?
        .as_str()
        .ok_or_else(|| DomainError::InvalidData("fact_id is not a string".to_string()))?;
    
    Ok(FactId::from_string(fact_id)?)
}
```

### Validating Temporal Dependencies

```rust
/// Validate temporal dependencies for an effect
async fn validate_temporal_dependencies(
    effect: &dyn Effect,
    time_service: &dyn TimeService,
) -> Result<(), EffectError> {
    // Get the effect's dependencies
    let dependencies = effect.dependencies();
    
    // Validate each dependency
    for dependency in dependencies {
        // Check that the dependency exists and is in the causal past
        let is_in_past = time_service.happened_before(
            &dependency,
            &effect.id(),
        ).await
        .map_err(|e| EffectError::TemporalValidationError(e.to_string()))?;
        
        if !is_in_past {
            return Err(EffectError::CausalOrderViolation(
                format!("Dependency {} is not in the causal past", dependency)
            ));
        }
    }
    
    Ok(())
}
```

### Implementing Custom Time Services

```rust
/// Custom causal time service implementation
pub struct CustomCausalTimeService {
    store: Arc<dyn TimeStore>,
}

impl CausalTimeService for CustomCausalTimeService {
    async fn get_logical_clock(&self, domain_id: &DomainId) 
        -> Result<u64, TimeError> {
        // Implementation
        Ok(0)
    }
    
    async fn get_vector_clock(&self, domain_id: &DomainId) 
        -> Result<HashMap<DomainId, u64>, TimeError> {
        // Implementation
        Ok(HashMap::new())
    }
    
    async fn advance_logical_clock(&self, domain_id: &DomainId) 
        -> Result<u64, TimeError> {
        // Implementation
        Ok(0)
    }
    
    async fn update_vector_clock(
        &self,
        domain_id: &DomainId,
        updates: HashMap<DomainId, u64>,
    ) -> Result<(), TimeError> {
        // Implementation
        Ok(())
    }
}

/// Custom clock time service implementation
pub struct CustomClockTimeService {
    time_source: TimeSource,
    attestation_verifier: Arc<dyn AttestationVerifier>,
}

impl ClockTimeService for CustomClockTimeService {
    async fn get_current_time(&self) -> Result<DateTime<Utc>, TimeError> {
        // Implementation
        Ok(Utc::now())
    }
    
    async fn get_time_attestation(&self) -> Result<TimeAttestation, TimeError> {
        // Implementation
        Ok(TimeAttestation::default())
    }
    
    async fn verify_attestation(&self, attestation: &TimeAttestation) 
        -> Result<bool, TimeError> {
        // Implementation
        Ok(true)
    }
}
```

## Best Practices

### Time Service Selection

1. **Choose Appropriate Time Services**
   ```rust
   // GOOD: Select time service based on trust requirements
   let time_service = if requires_high_trust {
       registry.get_causal_service(&domain_id)?
   } else {
       registry.get_clock_service(&domain_id)?
   };
   
   // BAD: Always use the same time service
   let time_service = registry.get_causal_service(&domain_id)?;
   ```

2. **Handle Missing Services Gracefully**
   ```rust
   // GOOD: Provide fallback behavior
   let time_service = registry.get_causal_service(&domain_id)
       .unwrap_or_else(|| Arc::new(FallbackTimeService::new()));
   
   // BAD: Panic on missing service
   let time_service = registry.get_causal_service(&domain_id)
       .expect("Time service must exist");
   ```

### Effect Composition

1. **Compose Time Effects Properly**
   ```rust
   // GOOD: Add time effects in the correct order
   context.add_effect(causal_time_effect);
   context.add_effect(clock_time_effect);
   context.add_effect(main_effect);
   
   // BAD: Add effects in arbitrary order
   context.add_effect(main_effect);
   context.add_effect(causal_time_effect);
   context.add_effect(clock_time_effect);
   ```

2. **Validate Dependencies**
   ```rust
   // GOOD: Validate all temporal dependencies
   for dependency in effect.dependencies() {
       validate_temporal_dependency(dependency, time_service).await?;
   }
   
   // BAD: Skip dependency validation
   // No validation code here
   ```

### Error Handling

1. **Handle Time Errors Appropriately**
   ```rust
   // GOOD: Map time errors to domain errors
   let result = time_service.get_current_time().await
       .map_err(|e| DomainError::TimeError(e.to_string()))?;
   
   // BAD: Ignore time errors
   let result = time_service.get_current_time().await?;
   ```

2. **Provide Context in Errors**
   ```rust
   // GOOD: Include context in error messages
   Err(TimeError::InvalidAttestation(format!(
       "Attestation from {} is invalid",
       attestation.source
   )))
   
   // BAD: Use generic error messages
   Err(TimeError::InvalidAttestation)
   ```

## Testing Time Implementations

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_causal_time_effect() {
        // Set up time service
        let time_service = setup_time_services(DomainId::test()).await.unwrap();
        
        // Create dependencies
        let dependencies = vec![
            FactId::test(),
            FactId::test(),
        ];
        
        // Create causal time effect
        let effect = create_causal_time_effect(
            &*time_service,
            &DomainId::test(),
            dependencies.clone(),
        ).await.unwrap();
        
        // Verify effect fields
        assert_eq!(effect.domain_id, DomainId::test());
        assert_eq!(effect.dependencies, dependencies);
        
        // Handle the effect
        handle_causal_time_effect(&effect, &*time_service).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_clock_time_effect() {
        // Set up time service
        let time_service = setup_time_services(DomainId::test()).await.unwrap();
        
        // Create clock time effect
        let effect = create_clock_time_effect(
            &*time_service,
            &DomainId::test(),
        ).await.unwrap();
        
        // Verify effect fields
        assert_eq!(effect.domain_id, DomainId::test());
        assert!(effect.attestation.is_some());
        
        // Handle the effect
        handle_clock_time_effect(&effect, &*time_service).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_fact_creation_with_time_effects() {
        // Set up domain
        let domain = Domain::test();
        
        // Create fact data
        let fact_data = b"test fact";
        let dependencies = vec![FactId::test()];
        
        // Create fact with time effects
        let fact_id = create_fact_with_time_effects(
            &domain,
            fact_data,
            dependencies.clone(),
        ).await.unwrap();
        
        // Verify fact was created
        assert!(domain.has_fact(&fact_id).await.unwrap());
        
        // Verify temporal dependencies
        let fact = domain.get_fact(&fact_id).await.unwrap();
        assert_eq!(fact.dependencies, dependencies);
    }
}
```

## Troubleshooting

| Problem | Possible Cause | Solution |
|---------|---------------|----------|
| Causal time effect fails | Missing dependencies | Ensure all dependencies exist and are in the causal past |
| | Invalid logical clock | Verify logical clock updates follow Lamport clock rules |
| | Invalid vector clock | Check vector clock updates maintain causal ordering |
| Clock time effect fails | Invalid attestation | Verify attestation source and signature |
| | Time source unavailable | Check time source configuration and connectivity |
| | Clock drift | Implement clock drift detection and correction |
| Fact creation fails | Missing time effects | Ensure both causal and clock time effects are added |
| | Invalid temporal order | Verify effects are added in the correct order |
| | Transaction failure | Check all effects in the transaction succeed |

## References

- [Time System Architecture](../../architecture/core/time-system.md)
- [Effect System](../../architecture/core/effect-system.md)
- [Resource System](../../architecture/core/resource-system.md)
- [ADR-000: Time System](../../../spec/adr_000_time.md)
- [ADR-001: Effect System](../../../spec/adr_001_effect_system.md) 