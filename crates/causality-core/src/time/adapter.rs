// Time System Adapter
//
// This module provides adapters that connect the domain effect-based time system
// with the existing time provider implementation.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fmt::Debug;

use async_trait::async_trait;
use anyhow::Result;

use crate::effect::{
    Effect, EffectContext, EffectError, EffectOutcome, EffectResult, EffectRegistry,
    domain::{DomainEffect, DomainEffectHandler, DomainId},
    types::{EffectId, EffectTypeId, ExecutionBoundary},
};
use crate::resource::ResourceId;
use super::{
    effect::{
        TimeEffect, TimeEffectType, TimeEffectHandler, BasicTimeEffectHandler,
        AdvanceCausalTimeEffect, SetClockTimeEffect, RegisterAttestationEffect,
        TimeAttestation as DomainTimeAttestation, AttestationSource as DomainAttestationSource
    },
    effect_handler::{TimeEffectHandlerImpl, AttestationStore, TimeAttestation},
    provider::{TimeProvider, TimeProviderFactory},
    map::TimeMap,
    types::DomainPosition,
    Timer, Timestamp, ClockTime, TimeDelta,
};

/// Adapter that connects the domain effect-based time system with the provider-based system
pub struct TimeSystemAdapter {
    /// Effect-based handler
    domain_handler: Arc<BasicTimeEffectHandler>,
    
    /// Provider-based handler
    provider_handler: Arc<TimeEffectHandlerImpl>,
    
    /// Time provider
    time_provider: Arc<dyn TimeProvider>,
    
    /// Time map
    time_map: Arc<RwLock<TimeMap>>,
    
    /// Domain ID
    domain_id: DomainId,
}

impl TimeSystemAdapter {
    /// Create a new time system adapter
    pub fn new(
        domain_id: DomainId,
        time_provider: Arc<dyn TimeProvider>,
        attestation_store: Arc<dyn AttestationStore>,
    ) -> Self {
        // Create the domain handler
        let mut domain_handler = BasicTimeEffectHandler::new(domain_id.clone());
        
        // Create the provider handler
        let provider_handler = Arc::new(TimeEffectHandlerImpl::new(
            Arc::clone(&time_provider),
            attestation_store,
        ));
        
        // Create the time map
        let time_map = Arc::new(RwLock::new(TimeMap::new()));
        
        Self {
            domain_handler: Arc::new(domain_handler),
            provider_handler,
            time_provider,
            time_map,
            domain_id,
        }
    }
    
    /// Register this adapter with the effect registry
    pub fn register(&self, registry: &mut dyn EffectRegistry) -> Result<(), EffectError> {
        // Register the domain handler
        registry.register_domain_handler(Arc::clone(&self.domain_handler) as Arc<dyn DomainEffectHandler>)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to register domain handler: {}", e)))?;
        
        Ok(())
    }
    
    /// Advance causal time by the specified amount
    pub async fn advance_causal_time(
        &self,
        advance_by: TimeDelta,
        reason: &str,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome> {
        // Create the effect
        let effect = super::effect::TimeEffectFactory::advance_causal_time(
            &self.domain_id,
            advance_by,
            reason,
        );
        
        // Execute the effect using the domain handler
        self.domain_handler.handle_advance_causal_time(&effect, context).await
    }
    
    /// Set clock time with attestation
    pub async fn set_clock_time(
        &self,
        time: ClockTime,
        source: DomainAttestationSource,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome> {
        // Create the effect
        let effect = super::effect::TimeEffectFactory::set_clock_time(
            &self.domain_id,
            time,
            source,
        );
        
        // Execute the effect using the domain handler
        self.domain_handler.handle_set_clock_time(&effect, context).await
    }
    
    /// Register a time attestation
    pub async fn register_attestation(
        &self,
        attestation: DomainTimeAttestation,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome> {
        // Create the effect
        let effect = super::effect::TimeEffectFactory::register_attestation(
            &self.domain_id,
            attestation,
        );
        
        // Execute the effect using the domain handler
        self.domain_handler.handle_register_attestation(&effect, context).await
    }
    
    /// Convert from domain attestation source to provider attestation source
    fn convert_attestation_source(
        source: &DomainAttestationSource,
    ) -> causality_types::time_snapshot::AttestationSource {
        use causality_types::time_snapshot::AttestationSource;
        
        match source {
            DomainAttestationSource::SystemClock => AttestationSource::System,
            DomainAttestationSource::NTP => AttestationSource::NTP,
            DomainAttestationSource::ExternalSource(src) => AttestationSource::External(src.clone()),
            DomainAttestationSource::Consensus(src) => AttestationSource::Consensus(src.clone()),
            DomainAttestationSource::UserProvided => AttestationSource::User,
            DomainAttestationSource::Custom(name) => AttestationSource::Custom(name.clone()),
        }
    }
    
    /// Convert from provider attestation source to domain attestation source
    fn convert_to_domain_source(
        source: &causality_types::time_snapshot::AttestationSource,
    ) -> DomainAttestationSource {
        match source {
            causality_types::time_snapshot::AttestationSource::System => DomainAttestationSource::SystemClock,
            causality_types::time_snapshot::AttestationSource::NTP => DomainAttestationSource::NTP,
            causality_types::time_snapshot::AttestationSource::External(src) => DomainAttestationSource::ExternalSource(src.clone()),
            causality_types::time_snapshot::AttestationSource::Consensus(src) => DomainAttestationSource::Consensus(src.clone()),
            causality_types::time_snapshot::AttestationSource::User => DomainAttestationSource::UserProvided,
            causality_types::time_snapshot::AttestationSource::Custom(name) => DomainAttestationSource::Custom(name.clone()),
        }
    }
    
    /// Convert from domain attestation to provider attestation
    fn convert_attestation(
        attestation: &DomainTimeAttestation,
    ) -> TimeAttestation {
        TimeAttestation {
            domain_id: attestation.timestamp.to_string(),
            timestamp: attestation.timestamp.as_millis() as u64,
            source: Self::convert_attestation_source(&attestation.source),
            confidence: attestation.source.trust_level() as f64 / 100.0,
        }
    }
    
    /// Convert from provider attestation to domain attestation
    fn convert_to_domain_attestation(
        attestation: &TimeAttestation,
    ) -> DomainTimeAttestation {
        let timestamp = ClockTime::from_millis(attestation.timestamp);
        let source = Self::convert_to_domain_source(&attestation.source);
        
        DomainTimeAttestation::new(timestamp, source)
    }
}

/// Factory for creating time system adapters
pub struct TimeSystemAdapterFactory;

impl TimeSystemAdapterFactory {
    /// Create a new time system adapter with in-memory components
    pub fn create_in_memory(domain_id: DomainId) -> TimeSystemAdapter {
        // Create the time provider
        let time_provider = TimeProviderFactory::create_in_memory();
        
        // Create the attestation store
        let attestation_store = Arc::new(super::effect_handler::InMemoryAttestationStore::new());
        
        // Create the adapter
        TimeSystemAdapter::new(domain_id, time_provider, attestation_store)
    }
    
    /// Create a time system adapter with the given components
    pub fn create_with_components(
        domain_id: DomainId,
        time_provider: Arc<dyn TimeProvider>,
        attestation_store: Arc<dyn AttestationStore>,
    ) -> TimeSystemAdapter {
        TimeSystemAdapter::new(domain_id, time_provider, attestation_store)
    }
    
    /// Register a time system adapter with the given registry for the specified domain
    pub fn register_for_domain(
        registry: &mut dyn EffectRegistry, 
        domain_id: DomainId,
    ) -> Result<Arc<TimeSystemAdapter>, EffectError> {
        let adapter = Self::create_in_memory(domain_id);
        adapter.register(registry)?;
        let adapter_arc = Arc::new(adapter);
        Ok(adapter_arc)
    }
} 