# Time System

*This document is derived from [ADR-000](../../../spec/adr_000_time.md), [ADR-001](../../../spec/adr_001_effect_system.md), and the [System Specification](../../../spec/spec.md).*

*Last updated: 2023-03-26*

## Overview

The Time System in Causality provides a comprehensive framework for managing both causal time (logical ordering) and clock time (wall clock) through an effect-based architecture. This approach ensures that time changes are explicitly modeled, validated, and processed through the effect system while maintaining clear trust models for different time sources.

## Core Concepts

### Distinct Notions of Time

The Time System distinguishes between two fundamental notions of time:

1. **Causal Time**
   - Internal logical time for ordering events
   - Based on Lamport clocks and vector clocks
   - Does not rely on external systems
   - High trust model - internally consistent
   - Used for establishing "happens-before" relationships

2. **Clock Time**
   - External time derived from system clocks
   - May rely on external time sources (NTP, etc.)
   - Subject to clock drift and potential manipulation
   - Lower trust model - externally influenced
   - Used for real-world time tracking

### Time as an Effect

Time changes are modeled as effects within the system:

```rust
/// Effect for updating causal time
pub struct CausalTimeEffect {
    /// Domain for which to update time
    pub domain_id: DomainId,
    /// New logical clock value
    pub logical_clock: u64,
    /// Vector clock updates
    pub vector_clock_updates: HashMap<DomainId, u64>,
    /// Dependencies (facts that must be in the past)
    pub dependencies: Vec<FactId>,
}

/// Effect for updating clock time
pub struct ClockTimeEffect {
    /// Domain for which to update time
    pub domain_id: DomainId,
    /// Wall clock time
    pub wall_time: DateTime<Utc>,
    /// Time source information
    pub time_source: TimeSource,
    /// Time attestation (if available)
    pub attestation: Option<TimeAttestation>,
}
```

### Time Services

The system provides dedicated services for managing different aspects of time:

```rust
/// Service for causal time
pub trait CausalTimeService: Send + Sync + 'static {
    /// Get the current logical clock for a domain
    async fn get_logical_clock(&self, domain_id: &DomainId) -> Result<u64, TimeError>;
    
    /// Get the current vector clock for a domain
    async fn get_vector_clock(&self, domain_id: &DomainId) 
        -> Result<HashMap<DomainId, u64>, TimeError>;
    
    /// Advance the logical clock for a domain
    async fn advance_logical_clock(&self, domain_id: &DomainId) -> Result<u64, TimeError>;
    
    /// Update the vector clock for a domain
    async fn update_vector_clock(
        &self, 
        domain_id: &DomainId,
        updates: HashMap<DomainId, u64>,
    ) -> Result<(), TimeError>;
}

/// Service for clock time
pub trait ClockTimeService: Send + Sync + 'static {
    /// Get the current clock time
    async fn get_current_time(&self) -> Result<DateTime<Utc>, TimeError>;
    
    /// Get a time attestation
    async fn get_time_attestation(&self) -> Result<TimeAttestation, TimeError>;
    
    /// Verify a time attestation
    async fn verify_attestation(&self, attestation: &TimeAttestation) -> Result<bool, TimeError>;
}
```

## Components

### Time Effect Handlers

Time effects are processed by dedicated handlers:

```rust
/// Handler for causal time effects
pub struct CausalTimeEffectHandler {
    causal_time_service: Arc<dyn CausalTimeService>,
    fact_store: Arc<dyn FactStore>,
}

impl EffectHandler<CausalTimeEffect> for CausalTimeEffectHandler {
    async fn handle(
        &self,
        effect: &CausalTimeEffect,
        context: &EffectContext,
    ) -> Result<EffectOutcome, EffectError> {
        // Verify dependencies
        for dependency in &effect.dependencies {
            if !self.fact_store.has_fact(dependency).await? {
                return Err(EffectError::DependencyNotMet(
                    format!("Missing fact dependency: {}", dependency)
                ));
            }
        }
        
        // Update logical clock
        self.causal_time_service.update_logical_clock(
            &effect.domain_id,
            effect.logical_clock,
        ).await?;
        
        // Update vector clock
        if !effect.vector_clock_updates.is_empty() {
            self.causal_time_service.update_vector_clock(
                &effect.domain_id,
                effect.vector_clock_updates.clone(),
            ).await?;
        }
        
        Ok(EffectOutcome::Success(json!({
            "domain_id": effect.domain_id,
            "logical_clock": effect.logical_clock,
            "updated_domains": effect.vector_clock_updates.keys(),
        })))
    }
}
```

### Time Service Registry

The system maintains a registry of available time services:

```rust
/// Registry for time services
pub struct TimeServiceRegistry {
    /// Available causal time services
    causal_services: HashMap<DomainId, Arc<dyn CausalTimeService>>,
    /// Available clock time services
    clock_services: HashMap<DomainId, Arc<dyn ClockTimeService>>,
}

impl TimeServiceRegistry {
    /// Register a causal time service
    pub fn register_causal_service(
        &mut self,
        domain_id: DomainId,
        service: Arc<dyn CausalTimeService>,
    ) {
        self.causal_services.insert(domain_id, service);
    }
    
    /// Register a clock time service
    pub fn register_clock_service(
        &mut self,
        domain_id: DomainId,
        service: Arc<dyn ClockTimeService>,
    ) {
        self.clock_services.insert(domain_id, service);
    }
    
    /// Get a causal time service for a domain
    pub fn get_causal_service(&self, domain_id: &DomainId) 
        -> Option<Arc<dyn CausalTimeService>> {
        self.causal_services.get(domain_id).cloned()
    }
    
    /// Get a clock time service for a domain
    pub fn get_clock_service(&self, domain_id: &DomainId) 
        -> Option<Arc<dyn ClockTimeService>> {
        self.clock_services.get(domain_id).cloned()
    }
}
```

## Integration with Effect System

The Time System integrates with the Effect System in several ways:

1. **Time Effects as First-Class Citizens**
   - Time changes are modeled as effects
   - Time effects can be composed with other effects
   - Time effects participate in transactions

2. **Temporal Validation**
   ```rust
   /// Execute an effect with temporal validation
   pub async fn execute_effect_with_temporal_validation<E: Effect>(
       effect: &E,
       context: &EffectContext,
       executor: &EffectExecutor,
       time_service: &dyn TimeService,
   ) -> Result<EffectOutcome, EffectError> {
       // Get the effect's dependencies
       let dependencies = effect.dependencies();
       
       // Validate all temporal dependencies
       for dependency in dependencies {
           // Check that the dependency exists and is in the causal past
           let is_in_past = time_service.happened_before(
               &dependency, 
               &context.current_fact_id()
           ).await
           .map_err(|e| EffectError::TemporalValidationError(e.to_string()))?;
           
           if !is_in_past {
               return Err(EffectError::CausalOrderViolation(
                   format!("Dependency {} is not in the causal past", dependency)
               ));
           }
       }
       
       // Execute the effect
       executor.execute(effect, context).await
   }
   ```

3. **Time Service Selection**
   - Programs can select time services based on trust requirements
   - Different domains can use different time services
   - Time services can be composed and chained

## Trust Models

### Causal Time Trust Model

Causal time has a high trust model because:

1. It is internally consistent
2. It does not rely on external systems
3. It is based on mathematical principles (Lamport clocks)
4. It can be verified through the effect system

### Clock Time Trust Model

Clock time has a lower trust model because:

1. It relies on external time sources
2. It is subject to clock drift
3. It can be manipulated by external factors
4. It requires attestation for verification

## Implementation Examples

### Creating a Fact with Time Effects

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

### Using Time Services

```rust
/// Example of using time services
async fn example_time_usage(
    domain: &Domain,
) -> Result<(), DomainError> {
    // Get the time service
    let time_service = domain.time_service();
    
    // Get current logical clock
    let logical_clock = time_service.causal_time()
        .get_logical_clock(domain.id())
        .await?;
    
    // Get current wall clock time
    let wall_time = time_service.clock_time()
        .get_current_time()
        .await?;
    
    // Create a time attestation
    let attestation = time_service.clock_time()
        .get_time_attestation()
        .await?;
    
    // Verify the attestation
    let is_valid = time_service.clock_time()
        .verify_attestation(&attestation)
        .await?;
    
    Ok(())
}
```

## Where Implemented

The Time System is implemented in the following crates and modules:

| Component | Crate | Module |
|-----------|-------|--------|
| Time Effect Types | `causality-types` | `causality_types::time::effects` |
| Time Services | `causality-core` | `causality_core::time::services` |
| Time Effect Handlers | `causality-core` | `causality_core::effects::handlers::time` |
| Time Service Registry | `causality-core` | `causality_core::time::registry` |
| Temporal Validation | `causality-core` | `causality_core::effects::validation::temporal` |
| Time Attestation | `causality-crypto` | `causality_crypto::time::attestation` |

## References

- [ADR-000: Time System](../../../spec/adr_000_time.md)
- [ADR-001: Effect System](../../../spec/adr_001_effect_system.md)
- [Implementing the Time System](../../guides/implementation/time-system.md)
