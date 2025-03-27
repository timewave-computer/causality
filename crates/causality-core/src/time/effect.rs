// Time Effect Module
//
// This module implements time-related effects using the domain effect system.
// It provides effects for advancing time, managing time domains, and time
// attestations.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use async_trait::async_trait;
use thiserror::Error;

use crate::effect::{
    Effect, EffectContext, EffectError, EffectOutcome, EffectResult,
    domain::{DomainEffect, DomainEffectHandler, DomainId},
    outcome::{EffectOutcomeBuilder, ResultData},
    types::{EffectId, EffectTypeId, ExecutionBoundary},
};
use crate::resource::ResourceId;
use super::{ClockTime, Timestamp, TimeDelta, Timer, TimeObserver, Duration as TimeDuration};
use super::map::{DomainPosition, TimeMap};
use crate::crypto::Signature;
use crate::crypto::PublicKey;
use crate::types::FactId;

/// Time domain identifier
pub type TimeDomainId = DomainId;

/// Time effect error
#[derive(Error, Debug)]
pub enum TimeEffectError {
    #[error("Invalid time: {0}")]
    InvalidTime(String),
    
    #[error("Time domain error: {0}")]
    DomainError(String),
    
    #[error("Time attestation error: {0}")]
    AttestationError(String),
    
    #[error("Time validation error: {0}")]
    ValidationError(String),
    
    #[error("Time source error: {0}")]
    SourceError(String),
    
    #[error("Time operation error: {0}")]
    OperationError(String),
}

/// Time effect type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeEffectType {
    /// Advance causal time in a domain
    AdvanceCausalTime,
    
    /// Set clock time in a domain
    SetClockTime,
    
    /// Register time attestation
    RegisterAttestation,
    
    /// Create time domain
    CreateDomain,
    
    /// Merge time domains
    MergeDomains,
    
    /// Synchronize time between domains
    SynchronizeDomains,
    
    /// Custom time effect
    Custom(String),
}

impl TimeEffectType {
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeEffectType::AdvanceCausalTime => "advance_causal_time",
            TimeEffectType::SetClockTime => "set_clock_time",
            TimeEffectType::RegisterAttestation => "register_attestation",
            TimeEffectType::CreateDomain => "create_domain",
            TimeEffectType::MergeDomains => "merge_domains",
            TimeEffectType::SynchronizeDomains => "synchronize_domains",
            TimeEffectType::Custom(_) => "custom_time_effect",
        }
    }
    
    /// Get the effect type ID for this time effect type
    pub fn type_id(&self) -> EffectTypeId {
        let name = match self {
            TimeEffectType::Custom(name) => name.clone(),
            _ => self.as_str().to_string(),
        };
        
        EffectTypeId::new(&name)
    }
}

/// Time attestation source type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttestationSource {
    /// Local system time source
    SystemClock,
    
    /// Network time protocol source
    NTP,
    
    /// Trusted external time source
    ExternalSource(String),
    
    /// Consensus time (e.g., from a blockchain)
    Consensus(String),
    
    /// User-provided time
    UserProvided,
    
    /// Custom attestation source
    Custom(String),
}

impl AttestationSource {
    /// Get the source name
    pub fn name(&self) -> String {
        match self {
            AttestationSource::SystemClock => "system_clock".to_string(),
            AttestationSource::NTP => "ntp".to_string(),
            AttestationSource::ExternalSource(src) => format!("external_{}", src),
            AttestationSource::Consensus(src) => format!("consensus_{}", src),
            AttestationSource::UserProvided => "user_provided".to_string(),
            AttestationSource::Custom(name) => name.clone(),
        }
    }
    
    /// Get the trust level for this source
    pub fn trust_level(&self) -> u8 {
        match self {
            AttestationSource::SystemClock => 50,
            AttestationSource::NTP => 70,
            AttestationSource::ExternalSource(_) => 60,
            AttestationSource::Consensus(_) => 80,
            AttestationSource::UserProvided => 10,
            AttestationSource::Custom(_) => 30,
        }
    }
    
    /// Check if this source is considered trusted
    pub fn is_trusted(&self) -> bool {
        self.trust_level() >= 70
    }
}

/// Time attestation
#[derive(Debug, Clone)]
pub struct TimeAttestation {
    /// The timestamp being attested
    pub timestamp: ClockTime,
    
    /// The source of the attestation
    pub source: AttestationSource,
    
    /// The time the attestation was created
    pub attestation_time: SystemTime,
    
    /// Optional signature for the attestation
    pub signature: Option<Vec<u8>>,
    
    /// Additional metadata for the attestation
    pub metadata: HashMap<String, String>,
}

impl TimeAttestation {
    /// Create a new time attestation
    pub fn new(timestamp: ClockTime, source: AttestationSource) -> Self {
        Self {
            timestamp,
            source,
            attestation_time: SystemTime::now(),
            signature: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the attestation
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Add a signature to the attestation
    pub fn with_signature(mut self, signature: Vec<u8>) -> Self {
        self.signature = Some(signature);
        self
    }
    
    /// Check if this attestation is considered trusted
    pub fn is_trusted(&self) -> bool {
        self.source.is_trusted()
    }
    
    /// Get a string representation of this attestation
    pub fn to_string(&self) -> String {
        format!(
            "TimeAttestation {{ timestamp: {}, source: {:?}, trusted: {} }}",
            self.timestamp, self.source, self.is_trusted()
        )
    }
}

/// Base trait for time effects
#[async_trait]
pub trait TimeEffect: DomainEffect {
    /// Get the time effect type
    fn time_effect_type(&self) -> TimeEffectType;
    
    /// Get the time domain ID
    fn time_domain_id(&self) -> &TimeDomainId;
    
    /// Process any time attestations that may be part of this effect
    fn process_attestation(&self, _context: &dyn EffectContext) -> EffectResult<()> {
        Ok(()) // Default implementation does nothing
    }
    
    /// Validate time constraints for this effect
    fn validate_time_constraints(&self, _context: &dyn EffectContext) -> EffectResult<()> {
        Ok(()) // Default implementation does nothing
    }
}

/// Trait for effects that advance causal time
#[async_trait]
pub trait AdvanceCausalTimeEffect: TimeEffect {
    /// Get the amount to advance time by
    fn advance_by(&self) -> TimeDelta;
    
    /// Get the reason for the time advancement
    fn reason(&self) -> &str;
}

/// Trait for effects that set clock time
#[async_trait]
pub trait SetClockTimeEffect: TimeEffect {
    /// Get the new clock time
    fn new_time(&self) -> ClockTime;
    
    /// Get the attestation for the time
    fn attestation(&self) -> Option<&TimeAttestation>;
}

/// Trait for effects that register time attestations
#[async_trait]
pub trait RegisterAttestationEffect: TimeEffect {
    /// Get the attestation to register
    fn attestation(&self) -> &TimeAttestation;
}

/// Basic time effect implementation
#[derive(Debug, Clone)]
pub struct BasicTimeEffect {
    /// Effect ID
    id: EffectId,
    
    /// Time effect type
    effect_type: TimeEffectType,
    
    /// Time domain ID
    time_domain_id: TimeDomainId,
    
    /// Execution boundary
    boundary: ExecutionBoundary,
    
    /// Effect parameters
    parameters: HashMap<String, String>,
    
    /// Time attestation
    attestation: Option<TimeAttestation>,
}

impl BasicTimeEffect {
    /// Create a new basic time effect
    pub fn new(
        effect_type: TimeEffectType,
        time_domain_id: TimeDomainId,
    ) -> Self {
        Self {
            id: EffectId::new(),
            effect_type,
            time_domain_id,
            boundary: ExecutionBoundary::Inside,
            parameters: HashMap::new(),
            attestation: None,
        }
    }
    
    /// Set the execution boundary
    pub fn with_boundary(mut self, boundary: ExecutionBoundary) -> Self {
        self.boundary = boundary;
        self
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Add multiple parameters
    pub fn with_parameters(mut self, parameters: HashMap<String, String>) -> Self {
        self.parameters.extend(parameters);
        self
    }
    
    /// Add an attestation
    pub fn with_attestation(mut self, attestation: TimeAttestation) -> Self {
        self.attestation = Some(attestation);
        self
    }
    
    /// Get the parameters
    pub fn parameters(&self) -> &HashMap<String, String> {
        &self.parameters
    }
    
    /// Get a parameter value
    pub fn get_parameter(&self, key: &str) -> Option<&String> {
        self.parameters.get(key)
    }
    
    /// Get the attestation if available
    pub fn attestation(&self) -> Option<&TimeAttestation> {
        self.attestation.as_ref()
    }
}

#[async_trait]
impl Effect for BasicTimeEffect {
    /// Get the ID of this effect
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    /// Get the type ID of this effect
    fn type_id(&self) -> EffectTypeId {
        self.effect_type.type_id()
    }
    
    /// Get the execution boundary for this effect
    fn boundary(&self) -> ExecutionBoundary {
        self.boundary
    }
    
    /// Clone this effect into a boxed effect
    fn clone_effect(&self) -> Box<dyn Effect> {
        Box::new(self.clone())
    }
}

#[async_trait]
impl DomainEffect for BasicTimeEffect {
    /// Get the domain ID this effect operates on
    fn domain_id(&self) -> &DomainId {
        &self.time_domain_id
    }
    
    /// Get domain-specific parameters for this effect
    fn domain_parameters(&self) -> HashMap<String, String> {
        let mut params = self.parameters.clone();
        params.insert("time_effect_type".to_string(), self.effect_type.as_str().to_string());
        
        if let Some(attestation) = &self.attestation {
            params.insert("attestation_source".to_string(), attestation.source.name());
            params.insert("attestation_timestamp".to_string(), attestation.timestamp.to_string());
            params.insert("attestation_trusted".to_string(), attestation.is_trusted().to_string());
        }
        
        params
    }
    
    /// Handle this effect within the specified domain using the adapted context
    async fn handle_in_domain(
        &self,
        context: &dyn EffectContext,
        handler: &dyn DomainEffectHandler,
    ) -> EffectResult<EffectOutcome> {
        handler.handle_domain_effect(self, context).await
    }
}

#[async_trait]
impl TimeEffect for BasicTimeEffect {
    /// Get the time effect type
    fn time_effect_type(&self) -> TimeEffectType {
        self.effect_type.clone()
    }
    
    /// Get the time domain ID
    fn time_domain_id(&self) -> &TimeDomainId {
        &self.time_domain_id
    }
    
    /// Process any time attestations that may be part of this effect
    fn process_attestation(&self, context: &dyn EffectContext) -> EffectResult<()> {
        if let Some(attestation) = &self.attestation {
            // For effects that require trusted attestations, validate the trust level
            match self.effect_type {
                TimeEffectType::SetClockTime | TimeEffectType::RegisterAttestation => {
                    if !attestation.is_trusted() {
                        // Check if we have permission to use untrusted sources
                        if context.has_capability("time.use_untrusted_sources").is_err() {
                            return Err(EffectError::ValidationError(
                                format!("Untrusted time source: {}", attestation.source.name())
                            ));
                        }
                    }
                },
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Validate time constraints for this effect
    fn validate_time_constraints(&self, context: &dyn EffectContext) -> EffectResult<()> {
        match self.effect_type {
            TimeEffectType::AdvanceCausalTime => {
                // Validate advance amount
                if let Some(advance_str) = self.get_parameter("advance_by") {
                    let advance = advance_str.parse::<u64>().map_err(|_| {
                        EffectError::ValidationError("Invalid advance_by parameter".to_string())
                    })?;
                    
                    if advance == 0 {
                        return Err(EffectError::ValidationError(
                            "Cannot advance time by zero".to_string()
                        ));
                    }
                } else {
                    return Err(EffectError::ValidationError(
                        "Missing advance_by parameter".to_string()
                    ));
                }
            },
            TimeEffectType::SetClockTime => {
                // Must have an attestation
                if self.attestation.is_none() {
                    return Err(EffectError::ValidationError(
                        "Missing time attestation for SetClockTime effect".to_string()
                    ));
                }
            },
            _ => {}
        }
        
        Ok(())
    }
}

/// Implementation of AdvanceCausalTimeEffect
#[async_trait]
impl AdvanceCausalTimeEffect for BasicTimeEffect {
    /// Get the amount to advance time by
    fn advance_by(&self) -> TimeDelta {
        let advance_str = self.get_parameter("advance_by")
            .map(|s| s.as_str())
            .unwrap_or("1");
        
        let advance = advance_str.parse::<u64>().unwrap_or(1);
        TimeDelta::from(advance)
    }
    
    /// Get the reason for the time advancement
    fn reason(&self) -> &str {
        self.get_parameter("reason")
            .map(|s| s.as_str())
            .unwrap_or("default advancement")
    }
}

/// Implementation of SetClockTimeEffect
#[async_trait]
impl SetClockTimeEffect for BasicTimeEffect {
    /// Get the new clock time
    fn new_time(&self) -> ClockTime {
        if let Some(attestation) = &self.attestation {
            return attestation.timestamp;
        }
        
        // Fallback to parameter
        let time_str = self.get_parameter("new_time")
            .map(|s| s.as_str())
            .unwrap_or("0");
        
        let timestamp = time_str.parse::<u64>().unwrap_or(0);
        ClockTime::from_millis(timestamp)
    }
    
    /// Get the attestation for the time
    fn attestation(&self) -> Option<&TimeAttestation> {
        self.attestation.as_ref()
    }
}

/// Implementation of RegisterAttestationEffect
#[async_trait]
impl RegisterAttestationEffect for BasicTimeEffect {
    /// Get the attestation to register
    fn attestation(&self) -> &TimeAttestation {
        self.attestation.as_ref().expect("Attestation must be present for RegisterAttestationEffect")
    }
}

/// Time effect handler trait
#[async_trait]
pub trait TimeEffectHandler: DomainEffectHandler {
    /// Handle an advance causal time effect
    async fn handle_advance_causal_time(
        &self,
        effect: &dyn AdvanceCausalTimeEffect,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome>;
    
    /// Handle a set clock time effect
    async fn handle_set_clock_time(
        &self,
        effect: &dyn SetClockTimeEffect,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome>;
    
    /// Handle a register attestation effect
    async fn handle_register_attestation(
        &self,
        effect: &dyn RegisterAttestationEffect,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome>;
    
    /// Get the timer for a domain
    fn get_domain_timer(&self, domain_id: &TimeDomainId) -> Option<&Timer>;
}

/// Basic time effect handler implementation
#[derive(Debug)]
pub struct BasicTimeEffectHandler {
    /// The domain ID this handler operates on
    domain_id: TimeDomainId,
    
    /// Timers by domain ID
    timers: HashMap<TimeDomainId, Timer>,
    
    /// Time map for cross-domain time tracking
    time_map: TimeMap,
    
    /// Time observers
    observers: Vec<Arc<dyn TimeObserver>>,
    
    /// Time attestations
    attestations: Vec<TimeAttestation>,
}

impl BasicTimeEffectHandler {
    /// Create a new basic time effect handler
    pub fn new(domain_id: TimeDomainId) -> Self {
        let mut timers = HashMap::new();
        timers.insert(domain_id.clone(), Timer::new());
        
        Self {
            domain_id,
            timers,
            time_map: TimeMap::new(),
            observers: Vec::new(),
            attestations: Vec::new(),
        }
    }
    
    /// Add a timer for a domain
    pub fn add_domain_timer(&mut self, domain_id: TimeDomainId, timer: Timer) {
        self.timers.insert(domain_id, timer);
    }
    
    /// Add a time observer
    pub fn add_observer(&mut self, observer: Arc<dyn TimeObserver>) {
        self.observers.push(observer);
    }
    
    /// Add a time attestation
    pub fn add_attestation(&mut self, attestation: TimeAttestation) {
        self.attestations.push(attestation);
    }
    
    /// Get all attestations for a domain
    pub fn get_attestations(&self) -> &[TimeAttestation] {
        &self.attestations
    }
    
    /// Notify observers of a time change
    async fn notify_observers(&self, domain_id: &TimeDomainId, causal_time: Timestamp, clock_time: Option<ClockTime>) {
        for observer in &self.observers {
            if let Err(e) = observer.on_time_changed(domain_id, causal_time, clock_time).await {
                // Just log the error for now
                eprintln!("Error notifying time observer: {}", e);
            }
        }
    }
    
    /// Handle a generic time effect
    async fn handle_time_effect(
        &self,
        effect: &dyn TimeEffect,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome> {
        // Process attestation and validate constraints first
        effect.process_attestation(context)?;
        effect.validate_time_constraints(context)?;
        
        // Handle based on effect type
        match effect.time_effect_type() {
            TimeEffectType::AdvanceCausalTime => {
                if let Some(advance_effect) = effect.as_any().downcast_ref::<dyn AdvanceCausalTimeEffect>() {
                    self.handle_advance_causal_time(advance_effect, context).await
                } else {
                    Err(EffectError::ExecutionError(
                        "Effect doesn't implement AdvanceCausalTimeEffect".to_string()
                    ))
                }
            },
            TimeEffectType::SetClockTime => {
                if let Some(clock_effect) = effect.as_any().downcast_ref::<dyn SetClockTimeEffect>() {
                    self.handle_set_clock_time(clock_effect, context).await
                } else {
                    Err(EffectError::ExecutionError(
                        "Effect doesn't implement SetClockTimeEffect".to_string()
                    ))
                }
            },
            TimeEffectType::RegisterAttestation => {
                if let Some(attestation_effect) = effect.as_any().downcast_ref::<dyn RegisterAttestationEffect>() {
                    self.handle_register_attestation(attestation_effect, context).await
                } else {
                    Err(EffectError::ExecutionError(
                        "Effect doesn't implement RegisterAttestationEffect".to_string()
                    ))
                }
            },
            _ => {
                Err(EffectError::ExecutionError(
                    format!("Unsupported time effect type: {:?}", effect.time_effect_type())
                ))
            }
        }
    }
}

#[async_trait]
impl DomainEffectHandler for BasicTimeEffectHandler {
    /// Get the domain ID this handler operates on
    fn domain_id(&self) -> &DomainId {
        &self.domain_id
    }
    
    /// Handle a domain effect
    async fn handle_domain_effect(
        &self,
        effect: &dyn DomainEffect,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome> {
        if let Some(time_effect) = effect.as_any().downcast_ref::<dyn TimeEffect>() {
            self.handle_time_effect(time_effect, context).await
        } else {
            Err(EffectError::ExecutionError(
                "Effect is not a TimeEffect".to_string()
            ))
        }
    }
}

#[async_trait]
impl TimeEffectHandler for BasicTimeEffectHandler {
    /// Handle an advance causal time effect
    async fn handle_advance_causal_time(
        &self,
        effect: &dyn AdvanceCausalTimeEffect,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome> {
        let domain_id = effect.time_domain_id();
        let timer = self.get_domain_timer(domain_id).ok_or_else(|| {
            EffectError::ExecutionError(format!("No timer found for domain: {}", domain_id))
        })?;
        
        let advance_by = effect.advance_by();
        let old_time = timer.current_time();
        let new_time = old_time + advance_by;
        
        // Advance the timer
        timer.advance_to(new_time);
        
        // Update the time map
        let position = DomainPosition::from(new_time.as_u64());
        self.time_map.update_position(domain_id.as_str(), position);
        
        // Notify observers
        self.notify_observers(domain_id, new_time, None).await;
        
        // Create the outcome
        let builder = EffectOutcomeBuilder::new()
            .effect_id(effect.id().clone())
            .success()
            .string_result(format!("Advanced causal time from {} to {}", old_time, new_time))
            .data("old_time", old_time.to_string())
            .data("new_time", new_time.to_string())
            .data("advance_by", advance_by.to_string())
            .data("domain_id", domain_id.to_string())
            .data("reason", effect.reason().to_string());
        
        Ok(builder.build())
    }
    
    /// Handle a set clock time effect
    async fn handle_set_clock_time(
        &self,
        effect: &dyn SetClockTimeEffect,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome> {
        let domain_id = effect.time_domain_id();
        let timer = self.get_domain_timer(domain_id).ok_or_else(|| {
            EffectError::ExecutionError(format!("No timer found for domain: {}", domain_id))
        })?;
        
        let new_time = effect.new_time();
        let old_time = timer.current_clock_time();
        
        // Set the new clock time
        timer.set_clock_time(new_time);
        
        // Notify observers
        self.notify_observers(domain_id, timer.current_time(), Some(new_time)).await;
        
        // Get attestation information
        let (attestation_source, trust_level) = if let Some(attestation) = effect.attestation() {
            (attestation.source.name(), attestation.source.trust_level())
        } else {
            ("none".to_string(), 0)
        };
        
        // Create the outcome
        let builder = EffectOutcomeBuilder::new()
            .effect_id(effect.id().clone())
            .success()
            .string_result(format!("Set clock time from {} to {}", old_time, new_time))
            .data("old_time", old_time.to_string())
            .data("new_time", new_time.to_string())
            .data("domain_id", domain_id.to_string())
            .data("attestation_source", attestation_source)
            .data("trust_level", trust_level.to_string());
        
        Ok(builder.build())
    }
    
    /// Handle a register attestation effect
    async fn handle_register_attestation(
        &self,
        effect: &dyn RegisterAttestationEffect,
        context: &dyn EffectContext,
    ) -> EffectResult<EffectOutcome> {
        let domain_id = effect.time_domain_id();
        let attestation = effect.attestation();
        
        // Store the attestation (in a real implementation, this would be more robust)
        let mut attestations = self.attestations.clone();
        attestations.push(attestation.clone());
        
        // Create the outcome
        let builder = EffectOutcomeBuilder::new()
            .effect_id(effect.id().clone())
            .success()
            .string_result(format!("Registered time attestation from {}", attestation.source.name()))
            .data("domain_id", domain_id.to_string())
            .data("timestamp", attestation.timestamp.to_string())
            .data("source", attestation.source.name())
            .data("trust_level", attestation.source.trust_level().to_string())
            .data("is_trusted", attestation.is_trusted().to_string());
        
        Ok(builder.build())
    }
    
    /// Get the timer for a domain
    fn get_domain_timer(&self, domain_id: &TimeDomainId) -> Option<&Timer> {
        self.timers.get(domain_id)
    }
}

/// Factory for creating time effects
pub struct TimeEffectFactory;

impl TimeEffectFactory {
    /// Create an advance causal time effect
    pub fn advance_causal_time(
        domain_id: &TimeDomainId,
        advance_by: TimeDelta,
        reason: &str,
    ) -> BasicTimeEffect {
        BasicTimeEffect::new(
            TimeEffectType::AdvanceCausalTime,
            domain_id.clone(),
        )
        .with_parameter("advance_by", advance_by.to_string())
        .with_parameter("reason", reason.to_string())
    }
    
    /// Create a set clock time effect with attestation
    pub fn set_clock_time(
        domain_id: &TimeDomainId,
        time: ClockTime,
        source: AttestationSource,
    ) -> BasicTimeEffect {
        let attestation = TimeAttestation::new(time, source);
        
        BasicTimeEffect::new(
            TimeEffectType::SetClockTime,
            domain_id.clone(),
        )
        .with_parameter("new_time", time.to_string())
        .with_attestation(attestation)
    }
    
    /// Create a register attestation effect
    pub fn register_attestation(
        domain_id: &TimeDomainId,
        attestation: TimeAttestation,
    ) -> BasicTimeEffect {
        BasicTimeEffect::new(
            TimeEffectType::RegisterAttestation,
            domain_id.clone(),
        )
        .with_attestation(attestation)
    }
}

/// Extension trait for casting to Any
pub trait AsAny {
    /// Cast to Any
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: std::any::Any> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Error type for time-related operations
#[derive(Debug, thiserror::Error)]
pub enum TimeError {
    /// Error related to causal time
    #[error("Causal time error: {0}")]
    CausalTimeError(String),

    /// Error related to clock time
    #[error("Clock time error: {0}")]
    ClockTimeError(String),

    /// Error validating time attestation
    #[error("Time attestation validation error: {0}")]
    AttestationError(String),

    /// Error accessing time data
    #[error("Time data access error: {0}")]
    DataAccessError(String),

    /// Error with temporal relationship
    #[error("Temporal relationship error: {0}")]
    TemporalRelationshipError(String),

    /// General time error
    #[error("Time error: {0}")]
    GeneralError(String),
}

/// Effect for updating causal time
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Sources of time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeSource {
    /// Local system clock
    LocalSystem,
    
    /// Network Time Protocol
    NTP(String),
    
    /// Trusted external source
    TrustedSource(String),
    
    /// Consensus-derived time
    Consensus(Vec<String>),
}

/// Temporal distance between two facts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemporalDistance {
    /// Facts are causally related with a specific distance
    Causal(u64),
    
    /// Facts are temporally related with a specific duration
    Temporal(std::time::Duration),
    
    /// Facts have no known temporal relationship
    Unknown,
}

/// Effect for temporal query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalQueryEffect {
    /// Domain for the query
    pub domain_id: DomainId,
    
    /// Facts to query
    pub facts: Vec<FactId>,
    
    /// Query type
    pub query_type: TemporalQueryType,
}

/// Types of temporal queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemporalQueryType {
    /// Check if fact1 happened before fact2
    HappenedBefore(FactId, FactId),
    
    /// Get the temporal distance between facts
    Distance(FactId, FactId),
    
    /// Check if facts happened concurrently
    Concurrent(Vec<FactId>),
    
    /// Get a timeline of facts
    Timeline(Vec<FactId>),
}

// Future implementation:
// - Effect handler implementations for each effect type
// - Integration with the effect system
// - Time service interfaces 