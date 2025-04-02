// Time Facade
//
// This module provides a high-level facade for interacting with the time system.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::fmt::Debug;
use std::time::Duration;
use std::any::Any;

use async_trait::async_trait;
use thiserror::Error;
use anyhow::Result;

// Internal imports
use crate::effect::{EffectExecutor, EffectContext as CoreEffectContext, EffectOutcome, EffectResult};
use crate::effect::types::EffectId;
use crate::effect::context::Capability;
use crate::ResourceId;

// External types from causality-types
use causality_types::time_snapshot::{TimeEffect, TimeEffectResult, AttestationSource};

use super::TimeProvider;
use crate::time::timestamp::Timestamp;

/// Time facade for working with time effects
pub struct TimeFacade {
    /// Effect executor for executing time effects
    effect_executor: Arc<dyn EffectExecutor>,
    /// Time provider for direct time operations
    time_provider: Arc<dyn TimeProvider>,
}

impl TimeFacade {
    /// Create a new time facade
    pub fn new(
        effect_executor: Arc<dyn EffectExecutor>,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        Self {
            effect_executor,
            time_provider,
        }
    }
    
    /// Get the current time
    pub async fn now(&self) -> Result<Timestamp> {
        self.time_provider.now().await
    }
    
    /// Get the time for a specific domain
    pub async fn domain_time(&self, domain_id: &str) -> Result<Option<Timestamp>> {
        self.time_provider.domain_timestamp(domain_id).await
    }
    
    /// Create a causal update effect
    pub fn causal_update(&self) -> CausalUpdateBuilder {
        CausalUpdateBuilder::new(self.effect_executor.clone())
    }
    
    /// Create a clock attestation effect
    pub fn clock_attestation(&self) -> ClockAttestationBuilder {
        ClockAttestationBuilder::new(self.effect_executor.clone())
    }
    
    /// Create a time map update effect
    pub fn time_map_update(&self) -> TimeMapUpdateBuilder {
        TimeMapUpdateBuilder::new(self.effect_executor.clone())
    }
}

/// Builder for causal update effects
pub struct CausalUpdateBuilder {
    effect_executor: Arc<dyn EffectExecutor>,
    operations: Vec<String>,
    ordering: Vec<(String, String)>,
}

impl CausalUpdateBuilder {
    /// Create a new causal update builder
    pub fn new(effect_executor: Arc<dyn EffectExecutor>) -> Self {
        Self {
            effect_executor,
            operations: Vec::new(),
            ordering: Vec::new(),
        }
    }
    
    /// Add an operation to the causal update
    pub fn add_operation(mut self, operation: impl Into<String>) -> Self {
        self.operations.push(operation.into());
        self
    }
    
    /// Add operations to the causal update
    pub fn add_operations(mut self, operations: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.operations.extend(operations.into_iter().map(|op| op.into()));
        self
    }
    
    /// Add a causal ordering between operations
    pub fn add_ordering(mut self, before: impl Into<String>, after: impl Into<String>) -> Self {
        self.ordering.push((before.into(), after.into()));
        self
    }
    
    /// Execute the causal update effect
    pub fn execute(self) -> Result<TimeEffectResult> {
        let effect = TimeEffect::CausalUpdate {
            operations: self.operations,
            ordering: self.ordering,
        };
        
        let ctx = EffectContext::with_executor(self.effect_executor.clone());
        self.effect_executor.execute_effect(&effect, &ctx)
            .map_err(|e| anyhow::anyhow!("Effect execution failed: {}", e))?
            .into_result()
            .ok_or_else(|| anyhow::anyhow!("No result returned from effect execution"))
    }
}

/// Builder for clock attestation effects
pub struct ClockAttestationBuilder {
    effect_executor: Arc<dyn EffectExecutor>,
    domain_id: Option<String>,
    timestamp: Option<u64>,
    source: Option<AttestationSource>,
    confidence: Option<f64>,
}

impl ClockAttestationBuilder {
    /// Create a new clock attestation builder
    pub fn new(effect_executor: Arc<dyn EffectExecutor>) -> Self {
        Self {
            effect_executor,
            domain_id: None,
            timestamp: None,
            source: None,
            confidence: None,
        }
    }
    
    /// Set the domain ID
    pub fn domain(mut self, domain_id: impl Into<String>) -> Self {
        self.domain_id = Some(domain_id.into());
        self
    }
    
    /// Set the timestamp
    pub fn timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
    
    /// Set the source to blockchain
    pub fn blockchain_source(mut self, chain_id: impl Into<String>, block_number: Option<u64>) -> Self {
        self.source = Some(AttestationSource::Blockchain {
            chain_id: chain_id.into(),
            block_number,
        });
        self
    }
    
    /// Set the source to user
    pub fn user_source(mut self) -> Self {
        self.source = Some(AttestationSource::User);
        self
    }
    
    /// Set the source to operator
    pub fn operator_source(mut self, operator_id: impl Into<String>, signature: impl Into<String>) -> Self {
        self.source = Some(AttestationSource::Operator {
            operator_id: operator_id.into(),
            signature: signature.into(),
        });
        self
    }
    
    /// Set the source to committee
    pub fn committee_source(mut self, committee_id: impl Into<String>, signatures: Vec<String>) -> Self {
        self.source = Some(AttestationSource::Committee {
            committee_id: committee_id.into(),
            signatures,
        });
        self
    }
    
    /// Set the source to oracle
    pub fn oracle_source(mut self, oracle_id: impl Into<String>, data: impl Into<String>) -> Self {
        self.source = Some(AttestationSource::Oracle {
            oracle_id: oracle_id.into(),
            data: data.into(),
        });
        self
    }
    
    /// Set the confidence level
    pub fn confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }
    
    /// Execute the clock attestation effect
    pub fn execute(self) -> Result<TimeEffectResult> {
        let domain_id = self.domain_id.ok_or_else(|| anyhow::anyhow!("Domain ID is required"))?;
        let timestamp = self.timestamp.ok_or_else(|| anyhow::anyhow!("Timestamp is required"))?;
        let source = self.source.ok_or_else(|| anyhow::anyhow!("Source is required"))?;
        let confidence = self.confidence.unwrap_or(1.0);
        
        let effect = TimeEffect::ClockAttestation {
            domain_id,
            timestamp,
            source,
            confidence,
        };
        
        let ctx = EffectContext::with_executor(self.effect_executor.clone());
        self.effect_executor.execute_effect(&effect, &ctx)
            .map_err(|e| anyhow::anyhow!("Effect execution failed: {}", e))?
            .into_result()
            .ok_or_else(|| anyhow::anyhow!("No result returned from effect execution"))
    }
}

/// Builder for time map update effects
pub struct TimeMapUpdateBuilder {
    effect_executor: Arc<dyn EffectExecutor>,
    positions: HashMap<String, u64>,
    proofs: HashMap<String, String>,
}

impl TimeMapUpdateBuilder {
    /// Create a new time map update builder
    pub fn new(effect_executor: Arc<dyn EffectExecutor>) -> Self {
        Self {
            effect_executor,
            positions: HashMap::new(),
            proofs: HashMap::new(),
        }
    }
    
    /// Add a position to the time map update
    pub fn add_position(mut self, domain_id: impl Into<String>, timestamp: u64) -> Self {
        self.positions.insert(domain_id.into(), timestamp);
        self
    }
    
    /// Add a proof to the time map update
    pub fn add_proof(mut self, domain_id: impl Into<String>, proof: impl Into<String>) -> Self {
        self.proofs.insert(domain_id.into(), proof.into());
        self
    }
    
    /// Add positions to the time map update
    pub fn add_positions(mut self, positions: HashMap<String, u64>) -> Self {
        self.positions.extend(positions);
        self
    }
    
    /// Add proofs to the time map update
    pub fn add_proofs(mut self, proofs: HashMap<String, String>) -> Self {
        self.proofs.extend(proofs);
        self
    }
    
    /// Execute the time map update effect
    pub fn execute(self) -> Result<TimeEffectResult> {
        let effect = TimeEffect::TimeMapUpdate {
            positions: self.positions,
            proofs: self.proofs,
        };
        
        let ctx = EffectContext::with_executor(self.effect_executor.clone());
        self.effect_executor.execute_effect(&effect, &ctx)
            .map_err(|e| anyhow::anyhow!("Effect execution failed: {}", e))?
            .into_result()
            .ok_or_else(|| anyhow::anyhow!("No result returned from effect execution"))
    }
}

/// Simple effect context for use with time facade operations
#[derive(Debug)]
pub struct EffectContext {
    // Simple identifier for the effect
    id: EffectId,
    // Additional metadata
    metadata: HashMap<String, String>,
    // Effect executor
    executor: Option<Arc<dyn EffectExecutor>>,
    // Resources
    resources: HashSet<ResourceId>,
}

impl EffectContext {
    /// Create a new effect context
    pub fn new() -> Self {
        Self {
            id: EffectId::from("time-facade-effect".to_string()),
            metadata: HashMap::new(),
            executor: None,
            resources: HashSet::new(),
        }
    }
    
    /// Create a new effect context with an executor
    pub fn with_executor(executor: Arc<dyn EffectExecutor>) -> Self {
        Self {
            id: EffectId::from("time-facade-effect".to_string()),
            metadata: HashMap::new(),
            executor: Some(executor),
            resources: HashSet::new(),
        }
    }
}

impl crate::effect::context::EffectContext for EffectContext {
    fn effect_id(&self) -> &EffectId {
        &self.id
    }
    
    fn capabilities(&self) -> &[Capability] {
        &[]
    }
    
    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    
    fn resources(&self) -> &HashSet<ResourceId> {
        &self.resources
    }
    
    fn parent_context(&self) -> Option<&Arc<dyn crate::effect::context::EffectContext>> {
        None
    }
    
    fn has_capability(&self, _capability: &Capability) -> bool {
        // For simplicity, allow all capabilities
        true
    }
    
    fn get_registry(&self) -> Option<Arc<dyn EffectExecutor>> {
        self.executor.clone()
    }
    
    fn derive_context(&self, effect_id: EffectId) -> Box<dyn EffectContext> {
        Box::new(Self {
            id: effect_id,
            metadata: self.metadata.clone(),
            executor: self.executor.clone(),
            resources: self.resources.clone(),
        })
    }
    
    fn with_additional_capabilities(&self, _capabilities: Vec<Capability>) -> Box<dyn EffectContext> {
        Box::new(self.clone())
    }
    
    fn with_additional_resources(&self, resources: HashSet<ResourceId>) -> Box<dyn EffectContext> {
        let mut new_resources = self.resources.clone();
        new_resources.extend(resources);
        Box::new(Self {
            id: self.id.clone(),
            metadata: self.metadata.clone(),
            executor: self.executor.clone(),
            resources: new_resources,
        })
    }
    
    fn with_additional_metadata(&self, metadata: HashMap<String, String>) -> Box<dyn EffectContext> {
        let mut new_metadata = self.metadata.clone();
        new_metadata.extend(metadata);
        Box::new(Self {
            id: self.id.clone(),
            metadata: new_metadata,
            executor: self.executor.clone(),
            resources: self.resources.clone(),
        })
    }
    
    fn clone_context(&self) -> Box<dyn EffectContext> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clone for EffectContext {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            metadata: self.metadata.clone(),
            executor: self.executor.clone(),
            resources: self.resources.clone(),
        }
    }
} 