// Time System Adapter
//
// This module provides adapters that connect the domain effect-based time system
// with the existing time provider implementation.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fmt::Debug;
use std::time::SystemTime;
use std::any::Any;

use async_trait::async_trait;

use crate::effect::{
    Effect, DomainEffect, EffectContext, EffectError, EffectOutcome, 
    EffectResult,
    domain::{DomainEffectHandler, DomainEffectOutcome, DomainId, DomainEffectError},
    EffectExecutor, EffectRegistrar, EffectType,
};

use crate::time::{
    TimeProvider, ClockTime, TimeMap,
    effect::{TimeEffectType, TimeEffect},
    effect_handler::TimeEffectTrait,
    effect::AttestationSource as EffectAttestationSource,
    effect::TimeAttestation
};

use causality_types::time_snapshot::{
    AttestationSource as TimeSnapshotAttestationSource
};

use super::types::DomainAttestationSource;

// Define the effect data structures
#[derive(Debug, Clone)]
pub struct AdvanceCausalTimeEffect {
    pub domain_id: String,
    pub logical_clock: Option<u64>,
    pub vector_clock_updates: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct SetClockTimeEffect {
    pub timestamp: u64,
    pub source: TimeSnapshotAttestationSource,
}

#[derive(Debug, Clone)]
pub struct RegisterAttestationEffect {
    pub attestation: DomainTimeAttestation,
}

#[derive(Debug)]
pub enum TimeEffectVariant {
    AdvanceCausalTime(AdvanceCausalTimeEffect),
    SetClockTime(SetClockTimeEffect),
    RegisterAttestation(RegisterAttestationEffect),
    Other(String),
}

/// Domain time attestation structure
#[derive(Debug, Clone)]
pub struct DomainTimeAttestation {
    pub timestamp: u64,
    pub source: DomainAttestationSource,
    pub attestation_time: SystemTime,
    pub signature: Option<Vec<u8>>,
    pub metadata: HashMap<String, String>,
}

// Create a concrete implementation of TimeEffectHandler
#[derive(Debug)]
pub struct BasicTimeEffectHandlerImpl {
    pub domain_id: DomainId,
    pub time_effect_handler: Option<Arc<dyn TimeEffectTrait>>,
}

impl BasicTimeEffectHandlerImpl {
    pub fn new(domain_id: DomainId) -> Self {
        Self { 
            domain_id,
            time_effect_handler: None,
        }
    }
    
    pub fn with_handler(mut self, handler: Arc<dyn TimeEffectTrait>) -> Self {
        self.time_effect_handler = Some(handler);
        self
    }
    
    /// Convert from snapshot attestation source to effect attestation source
    pub fn convert_from_snapshot_to_effect_source(source: &TimeSnapshotAttestationSource) -> EffectAttestationSource {
        match source {
            TimeSnapshotAttestationSource::NTP => EffectAttestationSource::NTP,
            TimeSnapshotAttestationSource::External(s) => EffectAttestationSource::ExternalSource(s.clone()),
            TimeSnapshotAttestationSource::Consensus(s) => EffectAttestationSource::Consensus(s.clone()),
            TimeSnapshotAttestationSource::User => EffectAttestationSource::UserProvided,
            TimeSnapshotAttestationSource::Custom(s) => EffectAttestationSource::Custom(s.clone()),
            TimeSnapshotAttestationSource::SystemClock => EffectAttestationSource::SystemClock,
            TimeSnapshotAttestationSource::Blockchain { chain_id, block_number: _ } => {
                EffectAttestationSource::Custom(format!("blockchain:{}", chain_id))
            },
            TimeSnapshotAttestationSource::Operator { operator_id, signature: _ } => {
                EffectAttestationSource::Custom(format!("operator:{}", operator_id))
            },
            TimeSnapshotAttestationSource::Committee { committee_id, signatures: _ } => {
                EffectAttestationSource::Custom(format!("committee:{}", committee_id))
            },
            TimeSnapshotAttestationSource::Oracle { oracle_id, data: _ } => {
                EffectAttestationSource::Custom(format!("oracle:{}", oracle_id))
            },
        }
    }
    
    /// Convert domain attestation source to effect attestation source
    pub fn convert_domain_to_effect_source(source: &DomainAttestationSource) -> EffectAttestationSource {
        match source {
            DomainAttestationSource::NTP => EffectAttestationSource::NTP,
            DomainAttestationSource::External(s) => EffectAttestationSource::ExternalSource(s.clone()),
            DomainAttestationSource::Consensus(s) => EffectAttestationSource::Consensus(s.clone()),
            DomainAttestationSource::User => EffectAttestationSource::UserProvided,
            DomainAttestationSource::Custom(s) => EffectAttestationSource::Custom(s.clone()),
            DomainAttestationSource::System => EffectAttestationSource::SystemClock,
        }
    }
    
    /// Convert effect attestation source to domain attestation source
    pub fn convert_effect_to_domain_source(source: &EffectAttestationSource) -> DomainAttestationSource {
        match source {
            EffectAttestationSource::NTP => DomainAttestationSource::NTP,
            EffectAttestationSource::ExternalSource(s) => DomainAttestationSource::External(s.clone()),
            EffectAttestationSource::Consensus(s) => DomainAttestationSource::Consensus(s.clone()),
            EffectAttestationSource::UserProvided => DomainAttestationSource::User,
            EffectAttestationSource::Custom(s) => DomainAttestationSource::Custom(s.clone()),
            EffectAttestationSource::SystemClock => DomainAttestationSource::System,
        }
    }
    
    /// Convert the attestation from domain type to core type
    pub fn convert_attestation(attestation: &DomainTimeAttestation) -> TimeAttestation {
        let clock_time = ClockTime::from_unix_timestamp(attestation.timestamp as i64);
        let source = Self::convert_domain_to_effect_source(&attestation.source);
        
        let mut time_attestation = TimeAttestation::new(clock_time, source);
        
        // Add signature if available
        if let Some(sig) = &attestation.signature {
            time_attestation = time_attestation.with_signature(sig.clone());
        }
        
        // Add metadata
        for (key, value) in &attestation.metadata {
            time_attestation = time_attestation.with_metadata(key, value);
        }
        
        // Set attestation time
        time_attestation.attestation_time = attestation.attestation_time;
        
        time_attestation
    }
    
    /// Handle advancing causal time
    async fn handle_advance_causal_time(
        &self,
        effect: &AdvanceCausalTimeEffect,
        context: &dyn EffectContext,
    ) -> Result<DomainEffectOutcome, DomainEffectError> {
        // Get the time effect handler
        match &self.time_effect_handler {
            Some(handler) => {
                // Extract relevant data from the effect
                let result = handler.handle_advance_causal_time(
                    &effect.domain_id,
                    effect.logical_clock.unwrap_or(0),
                    effect.vector_clock_updates.clone(),
                    Vec::new(), // Typically we'd extract dependencies here
                ).await.map_err(|e| 
                    DomainEffectError::OperationError(format!("Failed to advance causal time: {}", e))
                )?;

                // Create outcome from the result
                Ok(DomainEffectOutcome::success(
                    effect.domain_id.clone(),
                    "advance_causal_time".to_string(),
                    Some(result),
                ))
            }
            None => Err(DomainEffectError::OperationError("No time effect handler available".to_string()))
        }
    }

    /// Handle setting clock time
    async fn handle_set_clock_time(
        &self,
        effect: &SetClockTimeEffect,
        _context: &dyn EffectContext,
    ) -> Result<DomainEffectOutcome, DomainEffectError> {
        // Get the time effect handler
        match &self.time_effect_handler {
            Some(handler) => {
                // Convert timestamp to DateTime
                let timestamp = effect.timestamp;
                let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0)
                    .ok_or_else(|| DomainEffectError::OperationError(
                        format!("Invalid timestamp: {}", timestamp)
                    ))?;

                // Convert AttestationSource to the correct type
                let source = Self::convert_from_snapshot_to_effect_source(&effect.source);

                // Set the clock time
                let result = handler.handle_set_clock_time(
                    &self.domain_id,
                    datetime,
                    source,
                ).await.map_err(|e| 
                    DomainEffectError::OperationError(format!("Failed to set clock time: {}", e))
                )?;

                // Create outcome from the result
                Ok(DomainEffectOutcome::success(
                    self.domain_id.clone(),
                    "set_clock_time".to_string(),
                    Some(result),
                ))
            }
            None => Err(DomainEffectError::OperationError("No time effect handler available".to_string()))
        }
    }

    /// Handle registering time attestation
    async fn handle_register_attestation(
        &self,
        effect: &RegisterAttestationEffect,
        _context: &dyn EffectContext,
    ) -> Result<DomainEffectOutcome, DomainEffectError> {
        // Get the time effect handler
        match &self.time_effect_handler {
            Some(handler) => {
                // Register the attestation
                let attestation = Self::convert_attestation(&effect.attestation);
                
                let result = handler.handle_register_attestation(
                    &self.domain_id,
                    attestation,
                ).await.map_err(|e| 
                    DomainEffectError::OperationError(format!("Failed to register attestation: {}", e))
                )?;

                // Create outcome from the result
                Ok(DomainEffectOutcome::success(
                    self.domain_id.clone(),
                    "register_attestation".to_string(),
                    Some(result),
                ))
            },
            None => Err(DomainEffectError::OperationError("No time effect handler available".to_string()))
        }
    }
}

#[async_trait]
impl DomainEffectHandler for BasicTimeEffectHandlerImpl {
    fn domain_id(&self) -> &str {
        self.domain_id.as_str()
    }

    async fn handle_domain_effect(
        &self,
        effect: &dyn DomainEffect,
        context: &dyn EffectContext,
    ) -> Result<DomainEffectOutcome, DomainEffectError> {
        let time_effect = match effect.as_any().downcast_ref::<TimeEffectImpl>() {
            Some(e) => e,
            None => {
                return Err(DomainEffectError::OperationError(
                    format!("Unsupported effect type: {}", effect.effect_type().to_string())
                ));
            }
        };

        match &time_effect.variant {
            TimeEffectVariant::AdvanceCausalTime(e) => self.handle_advance_causal_time(e, context).await,
            TimeEffectVariant::SetClockTime(e) => self.handle_set_clock_time(e, context).await,
            TimeEffectVariant::RegisterAttestation(e) => self.handle_register_attestation(e, context).await,
            TimeEffectVariant::Other(s) => Err(DomainEffectError::OperationError(format!("Unsupported time effect: {}", s))),
        }
    }
}

// Define a domain-specific time effect trait
pub trait TimeEffectDomain: DomainEffect {
    fn get_variant(&self) -> &TimeEffectVariant;
}

/// Adapter that connects the domain effect-based time system with the provider-based system
#[derive(Debug)]
pub struct TimeSystemAdapter {
    /// Domain handler
    domain_handler: Arc<BasicTimeEffectHandlerImpl>,
    
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
        time_effect_handler: Arc<dyn TimeEffectTrait>,
    ) -> Self {
        // Create the domain handler
        let domain_handler = BasicTimeEffectHandlerImpl::new(domain_id.clone())
            .with_handler(time_effect_handler);
        
        // Create the time map
        let time_map = Arc::new(RwLock::new(TimeMap::new()));
        
        Self {
            domain_handler: Arc::new(domain_handler),
            time_provider,
            time_map,
            domain_id,
        }
    }
    
    /// Register this adapter with the effect registry
    pub fn register(&self, registry: &mut (impl EffectExecutor + EffectRegistrar)) -> Result<(), EffectError> {
        // Create a new handler using a copy of the state from the Arc'd handler
        let handler = BasicTimeEffectHandlerImpl::new(self.domain_id.clone());
        
        // Register the domain handler directly
        registry.register_domain_handler(handler)
            .map_err(|e| EffectError::ExecutionError(format!("Failed to register domain handler: {}", e)))?;
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct TimeEffectImpl {
    pub variant: TimeEffectVariant,
    pub effect_id: String,
    pub domain_id_str: String,
}

impl TimeEffectImpl {
    pub fn new(variant: TimeEffectVariant, effect_id: String) -> Self {
        let domain_id = match &variant {
            TimeEffectVariant::AdvanceCausalTime(effect) => effect.domain_id.clone(),
            // Default to empty domain ID for other variants
            _ => "".to_string(),
        };
        
        Self { 
            variant, 
            effect_id,
            domain_id_str: domain_id,
        }
    }
}

impl TimeEffect for TimeEffectImpl {
    fn get_time_effect_type(&self) -> TimeEffectType {
        match self.variant {
            TimeEffectVariant::AdvanceCausalTime(_) => TimeEffectType::AdvanceCausalTime,
            TimeEffectVariant::SetClockTime(_) => TimeEffectType::SetClockTime,
            TimeEffectVariant::RegisterAttestation(_) => TimeEffectType::RegisterAttestation,
            TimeEffectVariant::Other(_) => TimeEffectType::Custom("other".to_string()),
        }
    }
    
    fn get_domain_id(&self) -> &String {
        &self.domain_id_str
    }
}

#[async_trait]
impl Effect for TimeEffectImpl {
    fn effect_type(&self) -> EffectType {
        match &self.variant {
            TimeEffectVariant::AdvanceCausalTime(_) => EffectType::Custom("advance_causal_time".to_string()),
            TimeEffectVariant::SetClockTime(_) => EffectType::Custom("set_clock_time".to_string()),
            TimeEffectVariant::RegisterAttestation(_) => EffectType::Custom("register_attestation".to_string()),
            TimeEffectVariant::Other(name) => EffectType::Custom(name.clone()),
        }
    }

    fn description(&self) -> String {
        format!("Time effect: {:?}", self.effect_type())
    }
    
    async fn execute(&self, _context: &dyn EffectContext) -> EffectResult<EffectOutcome> {
        Ok(EffectOutcome::success(HashMap::new()))
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl DomainEffect for TimeEffectImpl {
    fn domain_id(&self) -> &DomainId {
        &self.domain_id_str
    }
    
    fn validate(&self) -> Result<(), String> {
        // Basic validation - in real implementation, add more checks
        if self.domain_id_str.is_empty() {
            return Err("Domain ID cannot be empty".to_string());
        }
        Ok(())
    }
    
    fn domain_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        match &self.variant {
            TimeEffectVariant::AdvanceCausalTime(effect) => {
                params.insert("domain_id".to_string(), effect.domain_id.clone());
                if let Some(lc) = effect.logical_clock {
                    params.insert("logical_clock".to_string(), lc.to_string());
                }
                for (k, v) in &effect.vector_clock_updates {
                    params.insert(format!("vector_clock.{}", k), v.to_string());
                }
            },
            TimeEffectVariant::SetClockTime(effect) => {
                params.insert("timestamp".to_string(), effect.timestamp.to_string());
                params.insert("source".to_string(), format!("{:?}", effect.source));
            },
            TimeEffectVariant::RegisterAttestation(effect) => {
                params.insert("timestamp".to_string(), effect.attestation.timestamp.to_string());
                params.insert("source".to_string(), format!("{:?}", effect.attestation.source));
            },
            TimeEffectVariant::Other(name) => {
                params.insert("effect_type".to_string(), name.clone());
            }
        }
        params
    }
    
    fn adapt_context(&self, _context: &dyn EffectContext) -> Result<(), String> {
        // Simple adaptation for now
        Ok(())
    }
}

impl TimeEffectDomain for TimeEffectImpl {
    fn get_variant(&self) -> &TimeEffectVariant {
        &self.variant
    }
} 