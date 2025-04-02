// Domain selection module - implements strategies for domain selection

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

/// Unique identifier for a domain
pub type DomainId = String;

/// Selection criteria for domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionCriteria {
    /// Max acceptable latency in milliseconds
    pub max_latency_ms: Option<u64>,
    /// Max acceptable cost
    pub max_cost: Option<u64>,
    /// Min reliability score (0-100)
    pub min_reliability: Option<u64>,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Preferred domain types
    pub preferred_domain_types: Vec<String>,
    /// Additional constraints as key-value pairs
    pub constraints: HashMap<String, String>,
}

impl Default for SelectionCriteria {
    fn default() -> Self {
        Self {
            max_latency_ms: None,
            max_cost: None,
            min_reliability: None,
            required_capabilities: vec![],
            preferred_domain_types: vec![],
            constraints: HashMap::new(),
        }
    }
}

/// Result of domain selection
#[derive(Debug, Clone)]
pub struct SelectionResult {
    /// Selected domain ID
    pub domain_id: DomainId,
    /// Score assigned by the strategy (higher is better)
    pub score: f64,
    /// Reason for selection
    pub reason: String,
    /// Selection criteria used
    pub criteria: SelectionCriteria,
}

/// Domain information needed for selection
#[derive(Debug, Clone)]
pub struct DomainInfo {
    /// Domain ID
    pub domain_id: DomainId,
    /// Domain type (e.g., "Ethereum", "Solana")
    pub domain_type: String,
    /// Domain capabilities (features it supports)
    pub capabilities: Vec<String>,
    /// Average latency in milliseconds
    pub avg_latency: u64,
    /// Cost per operation (estimate)
    pub cost: u64,
    /// Reliability score (0.0 to 1.0)
    pub reliability: f64,
}

/// Simple domain adapter trait with only methods needed for selection
pub trait DomainAdapter: Send + Sync + fmt::Debug {
    /// Get the domain ID
    fn domain_id(&self) -> &DomainId;
    
    /// Get domain information used for selection
    fn info(&self) -> DomainInfo;
}

/// Domain selection strategy interface
pub trait DomainSelectionStrategy: Send + Sync {
    /// Get the name of this strategy
    fn name(&self) -> &str;
    
    /// Select a domain based on criteria
    fn select_domain(
        &self, 
        domains: &[Arc<dyn DomainAdapter>], 
        criteria: &SelectionCriteria
    ) -> Option<SelectionResult>;
}

/// Domain selection strategy that always prefers a specific domain
pub struct PreferredDomainStrategy {
    /// Name of this strategy
    name: String,
    /// Preferred domain ID
    preferred_domain_id: DomainId,
}

impl PreferredDomainStrategy {
    /// Create a new instance of this strategy
    pub fn new(name: &str, preferred_domain_id: DomainId) -> Self {
        Self {
            name: name.to_string(),
            preferred_domain_id,
        }
    }
}

impl DomainSelectionStrategy for PreferredDomainStrategy {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn select_domain(
        &self, 
        domains: &[Arc<dyn DomainAdapter>], 
        criteria: &SelectionCriteria
    ) -> Option<SelectionResult> {
        // First try to find the preferred domain
        let preferred = domains.iter().find(|d| d.domain_id() == &self.preferred_domain_id);
        
        if let Some(domain) = preferred {
            return Some(SelectionResult {
                domain_id: domain.domain_id().clone(),
                score: 1.0,
                reason: format!("Preferred domain found: {}", domain.domain_id()),
                criteria: criteria.clone(),
            });
        }
        
        // If not found, just take the first one
        domains.first().map(|domain| SelectionResult {
            domain_id: domain.domain_id().clone(),
            score: 0.5,
            reason: format!("Preferred domain not found, using alternative: {}", domain.domain_id()),
            criteria: criteria.clone(),
        })
    }
}

/// Domain selection strategy based on latency
pub struct LatencyBasedStrategy {
    /// Name of this strategy
    name: String,
}

impl LatencyBasedStrategy {
    /// Create a new instance of this strategy
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl DomainSelectionStrategy for LatencyBasedStrategy {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn select_domain(
        &self, 
        domains: &[Arc<dyn DomainAdapter>], 
        criteria: &SelectionCriteria
    ) -> Option<SelectionResult> {
        if domains.is_empty() {
            return None;
        }
        
        // Get domain infos
        let domain_infos: Vec<_> = domains.iter()
            .map(|d| (d.domain_id().clone(), d.info()))
            .collect();
        
        // Filter by max latency if specified
        let mut candidates = domain_infos;
        if let Some(max_latency) = criteria.max_latency_ms {
            candidates.retain(|(_, info)| info.avg_latency <= max_latency);
        }
        
        // Sort by latency (lowest first)
        candidates.sort_by_key(|(_, info)| info.avg_latency);
        
        // Take the lowest latency domain
        candidates.first().map(|(domain_id, info)| {
            let score = if info.avg_latency == 0 { 1.0 } else { 1.0 / (info.avg_latency as f64) };
            
            SelectionResult {
                domain_id: domain_id.clone(),
                score,
                reason: format!("Lowest latency domain: {} ({} ms)", domain_id, info.avg_latency),
                criteria: criteria.clone(),
            }
        })
    }
}

/// Domain selection strategy based on cost
pub struct CostBasedStrategy {
    /// Name of this strategy
    name: String,
}

impl CostBasedStrategy {
    /// Create a new instance of this strategy
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl DomainSelectionStrategy for CostBasedStrategy {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn select_domain(
        &self, 
        domains: &[Arc<dyn DomainAdapter>], 
        criteria: &SelectionCriteria
    ) -> Option<SelectionResult> {
        if domains.is_empty() {
            return None;
        }
        
        // Get domain infos
        let domain_infos: Vec<_> = domains.iter()
            .map(|d| (d.domain_id().clone(), d.info()))
            .collect();
        
        // Filter by max cost if specified
        let mut candidates = domain_infos;
        if let Some(max_cost) = criteria.max_cost {
            candidates.retain(|(_, info)| info.cost <= max_cost);
        }
        
        // Sort by cost (lowest first)
        candidates.sort_by_key(|(_, info)| info.cost);
        
        // Take the lowest cost domain
        candidates.first().map(|(domain_id, info)| {
            let score = if info.cost == 0 { 1.0 } else { 1.0 / (info.cost as f64) };
            
            SelectionResult {
                domain_id: domain_id.clone(),
                score,
                reason: format!("Lowest cost domain: {} (cost: {})", domain_id, info.cost),
                criteria: criteria.clone(),
            }
        })
    }
}

/// Composite domain selection strategy that combines multiple strategies
pub struct CompositeStrategy {
    /// Name of this strategy
    name: String,
    /// Strategies to use, with their weights
    strategies: Vec<(Box<dyn DomainSelectionStrategy>, f64)>,
}

impl CompositeStrategy {
    /// Create a new instance of this strategy
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            strategies: Vec::new(),
        }
    }
    
    /// Add a strategy with a weight
    pub fn add_strategy(&mut self, strategy: Box<dyn DomainSelectionStrategy>, weight: f64) {
        self.strategies.push((strategy, weight));
    }
}

impl DomainSelectionStrategy for CompositeStrategy {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn select_domain(
        &self, 
        domains: &[Arc<dyn DomainAdapter>], 
        criteria: &SelectionCriteria
    ) -> Option<SelectionResult> {
        // Get results from all strategies
        let mut results: Vec<(SelectionResult, f64)> = Vec::new();
        
        for (strategy, weight) in &self.strategies {
            if let Some(result) = strategy.select_domain(domains, criteria) {
                results.push((result, *weight));
            }
        }
        
        if results.is_empty() {
            return None;
        }
        
        // Score domains based on weighted results
        let mut domain_scores: HashMap<DomainId, (f64, String)> = HashMap::new();
        
        for (result, weight) in results {
            let weighted_score = result.score * weight;
            let entry = domain_scores.entry(result.domain_id.clone()).or_insert((0.0, String::new()));
            entry.0 += weighted_score;
            entry.1 = format!("{}, {}", entry.1, result.reason);
        }
        
        // Find the domain with the highest score
        let best = domain_scores.into_iter()
            .max_by(|(_, (score_a, _)), (_, (score_b, _))| {
                score_a.partial_cmp(score_b).unwrap()
            });
        
        best.map(|(domain_id, (score, reasons))| {
            SelectionResult {
                domain_id,
                score,
                reason: format!("Composite selection: {}", reasons),
                criteria: criteria.clone(),
            }
        })
    }
} 