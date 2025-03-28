use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;

use crate::effect::{Effect, EffectContext, EffectError, EffectHandler, EffectOutcome};
use crate::types::DomainId;
use crate::time::effect::{CausalTimeEffect, ClockTimeEffect, TimeError, TemporalQueryEffect};
use crate::time::service::{CausalTimeService, ClockTimeService, FactTimeStore, TimeAttestationStore};

/// Handler for causal time effects
pub struct CausalTimeEffectHandler {
    causal_time_service: Arc<dyn CausalTimeService>,
    fact_time_store: Arc<dyn FactTimeStore>,
}

impl CausalTimeEffectHandler {
    /// Create a new causal time effect handler
    pub fn new(
        causal_time_service: Arc<dyn CausalTimeService>,
        fact_time_store: Arc<dyn FactTimeStore>,
    ) -> Self {
        Self {
            causal_time_service,
            fact_time_store,
        }
    }
}

#[async_trait]
impl EffectHandler for CausalTimeEffectHandler {
    async fn handle(
        &self,
        effect: &dyn Effect,
        context: &EffectContext,
    ) -> Result<EffectOutcome, EffectError> {
        // Cast effect to CausalTimeEffect
        let causal_effect = match effect.as_any().downcast_ref::<CausalTimeEffect>() {
            Some(effect) => effect,
            None => return Err(EffectError::InvalidEffectType(
                "Expected CausalTimeEffect".to_string()
            )),
        };

        // Verify dependencies
        for dependency in &causal_effect.dependencies {
            match self.fact_time_store.get_logical_time(dependency, &causal_effect.domain_id).await {
                Ok(Some(_)) => { /* Fact exists in the store */ },
                Ok(None) => return Err(EffectError::DependencyNotMet(
                    format!("Missing fact dependency: {}", dependency)
                )),
                Err(e) => return Err(EffectError::Other(
                    format!("Error checking dependency: {}", e)
                )),
            }
        }
        
        // Update logical clock
        match self.causal_time_service.update_vector_clock(
            &causal_effect.domain_id,
            causal_effect.vector_clock_updates.clone(),
        ).await {
            Ok(_) => {},
            Err(e) => return Err(EffectError::Other(
                format!("Failed to update vector clock: {}", e)
            )),
        }
        
        // Return success outcome
        Ok(EffectOutcome::Success(serde_json::json!({
            "domain_id": causal_effect.domain_id,
            "logical_clock": causal_effect.logical_clock,
            "updated_domains": causal_effect.vector_clock_updates.keys(),
        })))
    }
}

/// Handler for clock time effects
pub struct ClockTimeEffectHandler {
    clock_time_service: Arc<dyn ClockTimeService>,
    time_attestation_store: Arc<dyn TimeAttestationStore>,
    fact_time_store: Arc<dyn FactTimeStore>,
}

impl ClockTimeEffectHandler {
    /// Create a new clock time effect handler
    pub fn new(
        clock_time_service: Arc<dyn ClockTimeService>,
        time_attestation_store: Arc<dyn TimeAttestationStore>,
        fact_time_store: Arc<dyn FactTimeStore>,
    ) -> Self {
        Self {
            clock_time_service,
            time_attestation_store,
            fact_time_store,
        }
    }
}

#[async_trait]
impl EffectHandler for ClockTimeEffectHandler {
    async fn handle(
        &self,
        effect: &dyn Effect,
        context: &EffectContext,
    ) -> Result<EffectOutcome, EffectError> {
        // Cast effect to ClockTimeEffect
        let clock_effect = match effect.as_any().downcast_ref::<ClockTimeEffect>() {
            Some(effect) => effect,
            None => return Err(EffectError::InvalidEffectType(
                "Expected ClockTimeEffect".to_string()
            )),
        };
        
        // Verify attestation if provided
        if let Some(attestation) = &clock_effect.attestation {
            match self.clock_time_service.verify_attestation(attestation).await {
                Ok(true) => {
                    // Store the attestation
                    if let Err(e) = self.time_attestation_store.store_attestation(
                        clock_effect.domain_id.clone(),
                        attestation.clone(),
                    ).await {
                        return Err(EffectError::Other(
                            format!("Failed to store attestation: {}", e)
                        ));
                    }
                },
                Ok(false) => return Err(EffectError::ValidationFailed(
                    "Invalid time attestation".to_string()
                )),
                Err(e) => return Err(EffectError::Other(
                    format!("Error verifying attestation: {}", e)
                )),
            }
        }
        
        // Return success outcome
        Ok(EffectOutcome::Success(serde_json::json!({
            "domain_id": clock_effect.domain_id,
            "wall_time": clock_effect.wall_time,
            "time_source": format!("{:?}", clock_effect.time_source),
            "has_attestation": clock_effect.attestation.is_some(),
        })))
    }
}

/// Handler for temporal query effects
pub struct TemporalQueryEffectHandler {
    fact_time_store: Arc<dyn FactTimeStore>,
}

impl TemporalQueryEffectHandler {
    /// Create a new temporal query effect handler
    pub fn new(fact_time_store: Arc<dyn FactTimeStore>) -> Self {
        Self {
            fact_time_store,
        }
    }
}

#[async_trait]
impl EffectHandler for TemporalQueryEffectHandler {
    async fn handle(
        &self,
        effect: &dyn Effect,
        context: &EffectContext,
    ) -> Result<EffectOutcome, EffectError> {
        // Cast effect to TemporalQueryEffect
        let query_effect = match effect.as_any().downcast_ref::<TemporalQueryEffect>() {
            Some(effect) => effect,
            None => return Err(EffectError::InvalidEffectType(
                "Expected TemporalQueryEffect".to_string()
            )),
        };
        
        // Handle different query types
        match &query_effect.query_type {
            // For now, return placeholder responses for each query type
            // Full implementation would involve complex causal logic
            _ => Ok(EffectOutcome::Success(serde_json::json!({
                "query_type": format!("{:?}", query_effect.query_type),
                "result": "Query executed (placeholder)",
                "facts": query_effect.facts,
            })))
        }
    }
}

/// Register time effect handlers with the effect system
pub fn register_time_effect_handlers(
    registry: &mut impl crate::effect::EffectRegistry,
    causal_time_service: Arc<dyn CausalTimeService>,
    clock_time_service: Arc<dyn ClockTimeService>,
    fact_time_store: Arc<dyn FactTimeStore>,
    time_attestation_store: Arc<dyn TimeAttestationStore>,
) {
    // Create handlers
    let causal_time_handler = CausalTimeEffectHandler::new(
        causal_time_service,
        fact_time_store.clone(),
    );
    
    let clock_time_handler = ClockTimeEffectHandler::new(
        clock_time_service,
        time_attestation_store,
        fact_time_store.clone(),
    );
    
    let temporal_query_handler = TemporalQueryEffectHandler::new(
        fact_time_store,
    );
    
    // Register handlers
    let _ = registry.register_handler(causal_time_handler);
    let _ = registry.register_handler(clock_time_handler);
    let _ = registry.register_handler(temporal_query_handler);
}

impl TimeEffectHandlerImpl {
    // ... existing code ...

    // Update the method that uses dyn EffectRegistry
    pub async fn register_with_effect_registry(
        &self,
        effect_registry: &mut impl EffectRegistry,
    ) -> Result<(), TimeError> {
        // Register handlers for all time effect types
        let handler = Arc::new(self.clone());
        effect_registry.register_handler(handler)
            .map_err(|e| TimeError::Registration(format!("Failed to register time effect handler: {}", e)))?;
        
        Ok(())
    }

    // ... existing code ...
} 