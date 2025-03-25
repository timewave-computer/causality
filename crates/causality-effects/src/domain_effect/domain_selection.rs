// Domain Selection Effect
//
// This module implements effects for dynamically selecting domains based on criteria.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use causality_domain::{
    domain::{DomainId, DomainType, DomainInfo},
    adapter::DomainAdapterRegistry,
    types::{Result as DomainResult, Error as DomainError},
};

use crate::effect::{Effect, EffectContext, EffectResult, EffectError, EffectOutcome};
use crate::effect_id::EffectId;
use crate::domain_effect::{DomainAdapterEffect, DomainContext};

/// Criteria for domain selection
#[derive(Debug, Clone)]
pub enum SelectionCriteria {
    /// Select by domain type
    DomainType(DomainType),
    
    /// Select by capability
    Capability(String),
    
    /// Select by both domain type and capability
    TypeAndCapability(DomainType, String),
    
    /// Select by name pattern
    NamePattern(String),
    
    /// Custom selection with a predicate function
    Custom(String), // The string is a serialized representation of selection criteria
}

/// Domain selection effect
#[derive(Debug)]
pub struct DomainSelectionEffect {
    /// Effect ID
    id: EffectId,
    
    /// Selection criteria
    criteria: SelectionCriteria,
    
    /// Limit the number of results
    limit: Option<usize>,
    
    /// Additional parameters
    parameters: HashMap<String, String>,
}

impl DomainSelectionEffect {
    /// Create a new domain selection effect
    pub fn new(criteria: SelectionCriteria) -> Self {
        Self {
            id: EffectId::new(),
            criteria,
            limit: None,
            parameters: HashMap::new(),
        }
    }
    
    /// Set the maximum number of results to return
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
    
    /// Get the selection criteria
    pub fn criteria(&self) -> &SelectionCriteria {
        &self.criteria
    }
}

#[async_trait]
impl Effect for DomainSelectionEffect {
    fn id(&self) -> &EffectId {
        &self.id
    }
    
    fn effect_type(&self) -> &str {
        "domain_selection"
    }
    
    fn description(&self) -> &str {
        "Select domains based on criteria"
    }
    
    async fn execute(&self, context: &EffectContext) -> EffectResult<EffectOutcome> {
        // This will be implemented by a DomainSelectionHandler
        Err(EffectError::Unimplemented)
    }
}

/// Handler for domain selection effects
///
/// This trait extends the DomainEffectHandler to add methods
/// for executing domain selection effects.
#[async_trait]
pub trait DomainSelectionHandler {
    /// Execute a domain selection effect
    async fn execute_domain_selection(
        &self, 
        effect: &DomainSelectionEffect, 
        context: &EffectContext
    ) -> EffectResult<EffectOutcome>;
    
    /// Find domains matching criteria
    async fn find_matching_domains(
        &self,
        criteria: &SelectionCriteria
    ) -> DomainResult<Vec<DomainInfo>>;
}

/// Implementation of DomainSelectionHandler for any type
/// that implements DomainAdapterRegistry
#[async_trait]
impl<T> DomainSelectionHandler for T 
where 
    T: DomainAdapterRegistry + Send + Sync + 'static
{
    async fn execute_domain_selection(
        &self, 
        effect: &DomainSelectionEffect, 
        context: &EffectContext
    ) -> EffectResult<EffectOutcome> {
        // Find matching domains
        let domains = self.find_matching_domains(&effect.criteria()).await
            .map_err(|e| EffectError::ExecutionError(format!("Domain selection failed: {}", e)))?;
        
        // Apply limit if specified
        let domains = if let Some(limit) = effect.limit {
            domains.into_iter().take(limit).collect()
        } else {
            domains
        };
        
        // Create outcome with domain info
        let mut outcome = EffectOutcome::success(effect.id().clone())
            .with_data("count", domains.len().to_string());
        
        // Add domain details to outcome
        for (i, domain) in domains.iter().enumerate() {
            outcome = outcome.with_data(
                format!("domain_{}_id", i),
                domain.domain_id.to_string()
            );
            outcome = outcome.with_data(
                format!("domain_{}_type", i),
                domain.domain_type.clone()
            );
            outcome = outcome.with_data(
                format!("domain_{}_name", i),
                domain.name.clone()
            );
        }
        
        Ok(outcome)
    }
    
    async fn find_matching_domains(
        &self,
        criteria: &SelectionCriteria
    ) -> DomainResult<Vec<DomainInfo>> {
        // Get all adapters
        let adapters = self.get_all_adapters().await;
        let mut result = Vec::new();
        
        // Filter based on criteria
        for adapter in adapters {
            let domain_info = adapter.domain_info();
            
            let matches = match criteria {
                SelectionCriteria::DomainType(domain_type) => {
                    domain_info.domain_type == *domain_type
                },
                SelectionCriteria::Capability(capability) => {
                    // Check if adapter has this capability
                    match adapter.has_capability(capability).await {
                        Ok(has_capability) => has_capability,
                        Err(_) => false
                    }
                },
                SelectionCriteria::TypeAndCapability(domain_type, capability) => {
                    domain_info.domain_type == *domain_type && 
                    match adapter.has_capability(capability).await {
                        Ok(has_capability) => has_capability,
                        Err(_) => false
                    }
                },
                SelectionCriteria::NamePattern(pattern) => {
                    domain_info.name.contains(pattern) || 
                    domain_info.domain_id.contains(pattern)
                },
                SelectionCriteria::Custom(criteria_str) => {
                    // Simple contains check for custom criteria
                    // In a real implementation, this would use a more sophisticated
                    // deserialization and evaluation mechanism
                    domain_info.domain_id.contains(criteria_str) ||
                    domain_info.name.contains(criteria_str) ||
                    domain_info.domain_type.contains(criteria_str)
                }
            };
            
            if matches {
                result.push(domain_info.clone());
            }
        }
        
        Ok(result)
    }
}

// Utility functions for domain selection

/// Select domains by type
pub fn select_domains_by_type(domain_type: impl Into<String>) -> DomainSelectionEffect {
    DomainSelectionEffect::new(SelectionCriteria::DomainType(domain_type.into()))
}

/// Select domains by capability
pub fn select_domains_by_capability(capability: impl Into<String>) -> DomainSelectionEffect {
    DomainSelectionEffect::new(SelectionCriteria::Capability(capability.into()))
}

/// Select domains by both type and capability
pub fn select_domains_by_type_and_capability(
    domain_type: impl Into<String>,
    capability: impl Into<String>
) -> DomainSelectionEffect {
    DomainSelectionEffect::new(SelectionCriteria::TypeAndCapability(
        domain_type.into(), 
        capability.into()
    ))
}

/// Select domains by name pattern
pub fn select_domains_by_name(pattern: impl Into<String>) -> DomainSelectionEffect {
    DomainSelectionEffect::new(SelectionCriteria::NamePattern(pattern.into()))
}

/// Select domains by custom criteria
pub fn select_domains_custom(criteria: impl Into<String>) -> DomainSelectionEffect {
    DomainSelectionEffect::new(SelectionCriteria::Custom(criteria.into()))
} 