use std::sync::Arc;
use crate::effect::{
    Effect, EffectRegistry, EffectBuilder, EffectContext, EffectHandler,
    registry::EffectRegistryBuilder,
};
use crate::time::effect::{
    CausalTimeEffect, ClockTimeEffect, TemporalQueryEffect,
    TimeSource, TimeAttestation,
};
use crate::time::service::{
    CausalTimeService, ClockTimeService, TimeService,
    TimeAttestationStore, FactTimeStore,
};
use crate::time::handler::{
    CausalTimeEffectHandler, ClockTimeEffectHandler, TemporalQueryEffectHandler,
};
use crate::time::implementations::{
    MemoryTimeService, MemoryTimeAttestationStore, MemoryFactTimeStore,
};
use causality_crypto::KeyPair;
use crate::types::DomainId;

/// Integration utility for the time effect system
pub struct TimeEffectIntegration {
    /// The time service
    time_service: Arc<dyn TimeService>,
    
    /// The time attestation store
    attestation_store: Arc<dyn TimeAttestationStore>,
    
    /// The fact time store
    fact_store: Arc<dyn FactTimeStore>,
}

impl TimeEffectIntegration {
    /// Create a new time effect integration with the provided services
    pub fn new(
        time_service: Arc<dyn TimeService>,
        attestation_store: Arc<dyn TimeAttestationStore>,
        fact_store: Arc<dyn FactTimeStore>,
    ) -> Self {
        Self {
            time_service,
            attestation_store,
            fact_store,
        }
    }
    
    /// Create a new time effect integration with memory-based services
    pub fn new_memory_based(key_pair: KeyPair) -> Self {
        let time_service = Arc::new(MemoryTimeService::new(key_pair));
        let attestation_store = Arc::new(MemoryTimeAttestationStore::new());
        let fact_store = Arc::new(MemoryFactTimeStore::new());
        
        Self {
            time_service,
            attestation_store,
            fact_store,
        }
    }
    
    /// Get the time service
    pub fn time_service(&self) -> Arc<dyn TimeService> {
        self.time_service.clone()
    }
    
    /// Get the attestation store
    pub fn attestation_store(&self) -> Arc<dyn TimeAttestationStore> {
        self.attestation_store.clone()
    }
    
    /// Get the fact store
    pub fn fact_store(&self) -> Arc<dyn FactTimeStore> {
        self.fact_store.clone()
    }
    
    /// Register time effect handlers with the effect registry
    pub fn register_handlers(&self, registry_builder: &mut EffectRegistryBuilder) {
        // Create handlers
        let causal_time_handler = Arc::new(CausalTimeEffectHandler::new(
            Arc::new(self.time_service.causal_time().clone()),
            self.fact_store.clone(),
        ));
        
        let clock_time_handler = Arc::new(ClockTimeEffectHandler::new(
            Arc::new(self.time_service.clock_time().clone()),
            self.attestation_store.clone(),
            self.fact_store.clone(),
        ));
        
        let temporal_query_handler = Arc::new(TemporalQueryEffectHandler::new(
            self.fact_store.clone(),
        ));
        
        // Register handlers with the registry builder
        registry_builder.register_handler::<CausalTimeEffect, _>(causal_time_handler);
        registry_builder.register_handler::<ClockTimeEffect, _>(clock_time_handler);
        registry_builder.register_handler::<TemporalQueryEffect, _>(temporal_query_handler);
    }
    
    /// Create a causal time effect
    pub async fn create_causal_time_effect(
        &self,
        domain_id: DomainId,
        dependencies: Vec<crate::types::FactId>,
    ) -> Result<CausalTimeEffect, crate::error::Error> {
        Ok(self.time_service.causal_time().create_causal_time_effect(&domain_id, dependencies).await?)
    }
    
    /// Create a clock time effect
    pub async fn create_clock_time_effect(
        &self,
        domain_id: DomainId,
    ) -> Result<ClockTimeEffect, crate::error::Error> {
        Ok(self.time_service.clock_time().create_clock_time_effect(&domain_id).await?)
    }
    
    /// Create a temporal query effect for happened-before relationship
    pub fn create_happened_before_effect(
        &self,
        domain_id: DomainId,
        fact1: crate::types::FactId,
        fact2: crate::types::FactId,
    ) -> TemporalQueryEffect {
        TemporalQueryEffect {
            domain_id,
            facts: vec![fact1.clone(), fact2.clone()],
            query_type: crate::time::effect::TemporalQueryType::HappenedBefore(fact1, fact2),
        }
    }
    
    /// Create a temporal query effect for timeline
    pub fn create_timeline_effect(
        &self,
        domain_id: DomainId,
        facts: Vec<crate::types::FactId>,
    ) -> TemporalQueryEffect {
        TemporalQueryEffect {
            domain_id,
            facts: facts.clone(),
            query_type: crate::time::effect::TemporalQueryType::Timeline(facts),
        }
    }
}

/// Extension trait for EffectBuilder to support time effects
pub trait TimeEffectBuilderExt {
    /// Add a causal time effect to the builder
    fn add_causal_time_effect(
        &mut self,
        effect: CausalTimeEffect,
    ) -> &mut Self;
    
    /// Add a clock time effect to the builder
    fn add_clock_time_effect(
        &mut self,
        effect: ClockTimeEffect,
    ) -> &mut Self;
    
    /// Add a temporal query effect to the builder
    fn add_temporal_query_effect(
        &mut self,
        effect: TemporalQueryEffect,
    ) -> &mut Self;
}

impl<T: Effect + 'static> TimeEffectBuilderExt for EffectBuilder<T> {
    fn add_causal_time_effect(
        &mut self,
        effect: CausalTimeEffect,
    ) -> &mut Self {
        self.add_effect(effect)
    }
    
    fn add_clock_time_effect(
        &mut self,
        effect: ClockTimeEffect,
    ) -> &mut Self {
        self.add_effect(effect)
    }
    
    fn add_temporal_query_effect(
        &mut self,
        effect: TemporalQueryEffect,
    ) -> &mut Self {
        self.add_effect(effect)
    }
} 