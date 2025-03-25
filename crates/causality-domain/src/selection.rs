// Domain selection utilities
// Original file: src/domain/selection.rs

// Domain Selection for Causality
//
// This module provides utilities for selecting optimal domains
// for various operations based on criteria like cost, speed,
// and reliability.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use futures::FutureExt;
use log::{info, debug};
use async_trait::async_trait;

use causality_types::{Error, Result};
use crate::domain::{
    DomainId, DomainRegistry, DomainInfo, DomainType, DomainStatus, 
    DomainAdapter, FactQuery
};
use crate::domain::SharedTimeMap;
use crate::domain::{DomainSyncInfo, DomainSyncStatus};
use causality_types::{BlockHeight, BlockHash, Timestamp};

/// Domain selection criteria
#[derive(Debug, Clone)]
pub struct SelectionCriteria {
    /// Required domain types
    pub required_types: Option<HashSet<DomainType>>,
    /// Preferred domain types
    pub preferred_types: Option<HashSet<DomainType>>,
    /// Required features
    pub required_features: Option<HashSet<String>>,
    /// Preferred features
    pub preferred_features: Option<HashSet<String>>,
    /// Minimum reliability score (0.0-1.0)
    pub min_reliability: Option<f64>,
    /// Maximum latency in ms
    pub max_latency: Option<u64>,
    /// Maximum cost in gas units
    pub max_cost: Option<u64>,
    /// Domains to exclude
    pub excluded_domains: Option<HashSet<DomainId>>,
}

impl Default for SelectionCriteria {
    fn default() -> Self {
        SelectionCriteria {
            required_types: None,
            preferred_types: None,
            required_features: None,
            preferred_features: None,
            min_reliability: None,
            max_latency: None,
            max_cost: None,
            excluded_domains: None,
        }
    }
}

/// Domain selection result
#[derive(Debug, Clone)]
pub struct SelectionResult {
    /// Selected domain
    pub domain_id: DomainId,
    /// Selection score (higher is better)
    pub score: f64,
    /// Domain info
    pub info: DomainInfo,
    /// Estimated cost in gas units
    pub estimated_cost: Option<u64>,
    /// Estimated latency in ms
    pub estimated_latency: Option<u64>,
}

/// Domain metrics for selection
#[derive(Debug, Clone)]
pub struct DomainMetrics {
    /// Domain ID
    pub domain_id: DomainId,
    /// Reliability score (0.0-1.0)
    pub reliability: f64,
    /// Average latency in ms
    pub avg_latency: u64,
    /// Cost factor (higher is more expensive)
    pub cost_factor: f64,
    /// Features supported by this domain
    pub features: HashSet<String>,
    /// Last update time
    pub last_update: chrono::DateTime<chrono::Utc>,
}

impl Default for DomainMetrics {
    fn default() -> Self {
        DomainMetrics {
            domain_id: DomainId::new("default"),
            reliability: 0.95, // Default to high reliability
            avg_latency: 1000, // Default to 1 second
            cost_factor: 1.0,  // Default cost factor
            features: HashSet::new(),
            last_update: chrono::Utc::now(),
        }
    }
}

/// Domain selector service
pub struct DomainSelector {
    /// Domain registry
    domain_registry: Arc<DomainRegistry>,
    /// Shared time map
    time_map: SharedTimeMap,
    /// Domain metrics
    metrics: HashMap<DomainId, DomainMetrics>,
    /// Operation cost estimates by domain type
    operation_costs: HashMap<(DomainType, String), u64>,
}

impl DomainSelector {
    /// Create a new domain selector
    pub fn new(
        domain_registry: Arc<DomainRegistry>,
        time_map: SharedTimeMap,
    ) -> Self {
        DomainSelector {
            domain_registry,
            time_map,
            metrics: HashMap::new(),
            operation_costs: HashMap::new(),
        }
    }
    
    /// Update metrics for a domain
    pub fn update_metrics(&mut self, metrics: DomainMetrics) {
        self.metrics.insert(metrics.domain_id.clone(), metrics);
    }
    
    /// Set operation cost estimate
    pub fn set_operation_cost(&mut self, domain_type: DomainType, operation: &str, cost: u64) {
        self.operation_costs.insert((domain_type, operation.to_string()), cost);
    }
    
    /// Get metrics for a domain
    pub fn get_metrics(&self, domain_id: &DomainId) -> Option<&DomainMetrics> {
        self.metrics.get(domain_id)
    }
    
    /// Get all domain metrics
    pub fn get_all_metrics(&self) -> &HashMap<DomainId, DomainMetrics> {
        &self.metrics
    }
    
    /// Select best domain for an operation based on criteria
    pub async fn select_domain(
        &self,
        operation: &str,
        criteria: Option<&SelectionCriteria>,
    ) -> Result<Option<SelectionResult>> {
        // Get the top domain
        let mut selected = self.select_multiple_domains(operation, criteria, 1).await?;
        
        // Return the first result, if any
        Ok(selected.pop())
    }
    
    /// Select multiple domains for an operation
    pub async fn select_multiple_domains(
        &self,
        operation: &str,
        criteria: Option<&SelectionCriteria>,
        count: usize,
    ) -> Result<Vec<SelectionResult>> {
        // Use default criteria if none provided
        let criteria = criteria.cloned().unwrap_or_default();
        
        // Get all available domains from registry
        let available_domains = self.domain_registry.list_domains()?;
        if available_domains.is_empty() {
            return Err(Error::InvalidArgument("No domains available".to_string()));
        }
        
        // Filter and score domains
        let mut candidates = Vec::new();
        
        for domain_id in available_domains {
            // Skip excluded domains
            if let Some(excluded) = &criteria.excluded_domains {
                if excluded.contains(&domain_id) {
                    continue;
                }
            }
            
            // Get domain adapter
            let adapter = match self.domain_registry.get_domain(&domain_id) {
                Some(a) => a,
                None => continue,
            };
            
            // Check domain connectivity
            let is_connected = adapter.check_connectivity().await?;
            if !is_connected {
                continue;
            }
            
            // Get domain info
            let info = adapter.domain_info().await?;
            
            // Skip if not active
            if info.status != DomainStatus::Active {
                continue;
            }
            
            // Check required domain types
            if let Some(required_types) = &criteria.required_types {
                if !required_types.contains(&info.domain_type) {
                    continue;
                }
            }
            
            // Get domain metrics
            let metrics = self.metrics.get(&domain_id).cloned().unwrap_or_else(|| {
                // Use default metrics if not available
                let mut default_metrics = DomainMetrics::default();
                default_metrics.domain_id = domain_id.clone();
                default_metrics
            });
            
            // Check minimum reliability
            if let Some(min_reliability) = criteria.min_reliability {
                if metrics.reliability < min_reliability {
                    continue;
                }
            }
            
            // Check maximum latency
            if let Some(max_latency) = criteria.max_latency {
                if metrics.avg_latency > max_latency {
                    continue;
                }
            }
            
            // Calculate estimated cost
            let estimated_cost = {
                let domain_type_key = info.domain_type.clone(); // Clone first time for this lookup
                self.operation_costs
                    .get(&(domain_type_key, operation.to_string()))
                    .copied()
            };
            
            // Check maximum cost
            if let (Some(max_cost), Some(est_cost)) = (criteria.max_cost, estimated_cost) {
                if est_cost > max_cost {
                    continue;
                }
            }
            
            // Check required features
            if let Some(required_features) = &criteria.required_features {
                if !required_features.iter().all(|f| metrics.features.contains(f)) {
                    continue;
                }
            }
            
            // Calculate score
            let mut score = 100.0; // Base score
            
            // Adjust score based on preferred domain types
            if let Some(preferred_types) = &criteria.preferred_types {
                let domain_type_check = info.domain_type.clone(); // Clone again for this lookup
                if preferred_types.contains(&domain_type_check) {
                    score += 50.0;
                }
            }
            
            // Adjust score based on preferred features
            if let Some(preferred_features) = &criteria.preferred_features {
                let matching_features = preferred_features.iter()
                    .filter(|f| metrics.features.contains(*f))
                    .count();
                
                score += matching_features as f64 * 10.0;
            }
            
            // Adjust score based on reliability
            score += metrics.reliability * 30.0;
            
            // Adjust score based on latency (lower is better)
            score -= (metrics.avg_latency as f64 / 100.0).min(20.0);
            
            // Adjust score based on cost (lower is better)
            if let Some(est_cost) = estimated_cost {
                score -= (est_cost as f64 / 100000.0).min(20.0);
            }
            
            // Add to candidates
            let registry_info = DomainInfo {
                id: info.id.clone(),
                domain_type: info.domain_type.clone(),
                name: info.name.clone(),
                description: info.description.clone(),
                rpc_url: info.rpc_url.clone(),
                explorer_url: info.explorer_url.clone(),
                chain_id: info.chain_id,
                native_currency: info.native_currency.clone(),
                status: info.status.clone(),
                metadata: info.metadata.clone(),
            };
            
            candidates.push(SelectionResult {
                domain_id: domain_id.clone(),
                score,
                info: registry_info,
                estimated_cost,
                estimated_latency: Some(metrics.avg_latency),
            });
        }
        
        // Sort by score (descending)
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        // Take the requested number of domains
        Ok(candidates.into_iter().take(count).collect())
    }
    
    /// Select domains for fault tolerance
    pub async fn select_for_fault_tolerance(
        &self,
        operation: &str,
        criteria: Option<&SelectionCriteria>,
        redundancy: usize,
    ) -> Result<Vec<SelectionResult>> {
        let selected = self.select_multiple_domains(operation, criteria, redundancy).await?;
        
        // Return the selected domains
        Ok(selected)
    }
    
    /// Update domain metrics from sync status
    pub fn update_metrics_from_sync(&mut self, sync_info: &HashMap<DomainId, DomainSyncInfo>) {
        for (domain_id, info) in sync_info {
            // Get existing metrics or create default
            let metrics = self.metrics.entry(domain_id.clone()).or_insert_with(|| {
                let mut default_metrics = DomainMetrics::default();
                default_metrics.domain_id = domain_id.clone();
                default_metrics
            });
            
            // Update reliability based on sync status
            match info.status {
                DomainSyncStatus::Active => {
                    // Gradually increase reliability
                    metrics.reliability = (metrics.reliability * 0.95 + 0.05).min(1.0);
                }
                DomainSyncStatus::Paused => {
                    // Slightly decrease reliability
                    metrics.reliability = (metrics.reliability * 0.9).max(0.0);
                }
                DomainSyncStatus::Failed => {
                    // Significantly decrease reliability
                    metrics.reliability = (metrics.reliability * 0.5).max(0.0);
                }
            }
            
            // Update last update time
            metrics.last_update = chrono::Utc::now();
        }
    }
    
    /// Estimate latency based on historical data
    pub async fn estimate_latency(
        &self,
        domain_id: &DomainId,
        operation: &str,
    ) -> Result<Duration> {
        // For now, use metrics if available
        if let Some(metrics) = self.metrics.get(domain_id) {
            return Ok(Duration::from_millis(metrics.avg_latency));
        }
        
        // Otherwise return a default
        Ok(Duration::from_secs(1))
    }
    
    /// Estimate cost based on operation and domain type
    pub fn estimate_cost(
        &self,
        domain_id: &DomainId,
        operation: &str,
    ) -> Result<Option<u64>> {
        // Get domain info
        if let Some(adapter) = self.domain_registry.get_domain(domain_id) {
            // This would need to be awaited in a real implementation
            let info = match adapter.domain_info().now_or_never() {
                Some(Ok(info)) => info,
                _ => return Ok(None),
            };
            
            // Look up cost
            let cost = self.operation_costs
                .get(&(info.domain_type, operation.to_string()))
                .cloned();
            
            return Ok(cost);
        }
        
        Ok(None)
    }

    /// Select domains for an operation based on criteria
    pub async fn select_domains(
        &self,
        operation: &str,
        criteria: Option<&SelectionCriteria>,
        count: usize
    ) -> Result<Vec<Arc<dyn DomainAdapter>>> {
        // Get the top domains by criteria
        let results = self.select_multiple_domains(operation, criteria, count).await?;
        
        // Extract adapters from selection results
        let mut adapters = Vec::new();
        for result in results {
            let domain_id = &result.domain_id;
            
            if let Some(c) = criteria {
                if let Some(excluded) = &c.excluded_domains {
                    if excluded.contains(domain_id) {
                        continue;
                    }
                }
            }
            
            if let Some(adapter) = self.domain_registry.get_domain(domain_id) {
                adapters.push(adapter);
            } else {
                debug!("Selected domain {:?} not found in registry", domain_id);
            }
        }
        
        Ok(adapters)
    }
}

// Domain Selection Strategies
//
// This module provides strategies for selecting domains when executing operations
// that can span multiple domains.

/// Domain selection strategy interface
///
/// This trait defines the interface for strategies that select domains
/// for operations that can span multiple domains.
#[async_trait]
pub trait DomainSelectionStrategy: Send + Sync {
    /// Select a domain for the given operation
    ///
    /// # Arguments
    /// * `domains` - The available domains to choose from
    /// * `required_capabilities` - Capabilities that the selected domain must support
    /// * `preferences` - Optional preferences to consider during selection (e.g., cost, latency)
    ///
    /// # Returns
    /// The selected domain ID or an error if no suitable domain is found
    async fn select_domain(
        &self,
        domains: &[Arc<dyn DomainAdapter>],
        required_capabilities: &HashSet<String>,
        preferences: &HashMap<String, String>,
    ) -> Result<DomainId>;
    
    /// Select multiple domains for an operation that spans multiple domains
    ///
    /// # Arguments
    /// * `domains` - The available domains to choose from
    /// * `required_capabilities` - Capabilities that the selected domains must support
    /// * `preferences` - Optional preferences to consider during selection (e.g., cost, latency)
    /// * `count` - Number of domains to select
    ///
    /// # Returns
    /// The selected domain IDs or an error if no suitable domains are found
    async fn select_domains(
        &self,
        domains: &[Arc<dyn DomainAdapter>],
        required_capabilities: &HashSet<String>,
        preferences: &HashMap<String, String>,
        count: usize,
    ) -> Result<Vec<DomainId>>;
}

/// Preferred domain selection strategy
///
/// This strategy selects domains based on a preferred list, falling back to
/// other domains if the preferred ones are not available.
pub struct PreferredDomainStrategy {
    /// List of preferred domain IDs in order of preference
    preferred_domains: Vec<DomainId>,
}

impl PreferredDomainStrategy {
    /// Create a new preferred domain strategy
    pub fn new(preferred_domains: Vec<DomainId>) -> Self {
        Self {
            preferred_domains,
        }
    }
}

#[async_trait]
impl DomainSelectionStrategy for PreferredDomainStrategy {
    async fn select_domain(
        &self,
        domains: &[Arc<dyn DomainAdapter>],
        required_capabilities: &HashSet<String>,
        _preferences: &HashMap<String, String>,
    ) -> Result<DomainId> {
        // First try the preferred domains
        for preferred_id in &self.preferred_domains {
            if let Some(domain) = domains.iter().find(|d| d.domain_id() == preferred_id) {
                // Check if domain has required capabilities
                // For now we don't have a capability check, so just return the domain
                return Ok(preferred_id.clone());
            }
        }
        
        // If no preferred domain is found, use the first available one
        if let Some(domain) = domains.first() {
            Ok(domain.domain_id().clone())
        } else {
            Err(Error::DomainNotFound("No domains available".to_string()))
        }
    }
    
    async fn select_domains(
        &self,
        domains: &[Arc<dyn DomainAdapter>],
        required_capabilities: &HashSet<String>,
        preferences: &HashMap<String, String>,
        count: usize,
    ) -> Result<Vec<DomainId>> {
        let mut selected = Vec::with_capacity(count);
        
        // First add all preferred domains that are available
        for preferred_id in &self.preferred_domains {
            if selected.len() >= count {
                break;
            }
            
            if let Some(domain) = domains.iter().find(|d| d.domain_id() == preferred_id) {
                // Check if domain has required capabilities
                // For now we don't have a capability check, so just add the domain
                selected.push(preferred_id.clone());
            }
        }
        
        // If we need more domains, add any remaining available ones
        for domain in domains {
            if selected.len() >= count {
                break;
            }
            
            if !selected.contains(domain.domain_id()) {
                selected.push(domain.domain_id().clone());
            }
        }
        
        if selected.is_empty() {
            Err(Error::DomainNotFound("No domains available".to_string()))
        } else if selected.len() < count {
            Err(Error::InsufficientDomains(format!("Requested {} domains but only found {}", count, selected.len())))
        } else {
            Ok(selected)
        }
    }
}

/// Latency-based domain selection strategy
///
/// This strategy selects domains based on their current latency, preferring
/// domains with lower latency.
pub struct LatencyBasedStrategy {
    /// Maximum acceptable latency in milliseconds
    max_latency_ms: u64,
}

impl LatencyBasedStrategy {
    /// Create a new latency-based strategy
    pub fn new(max_latency_ms: u64) -> Self {
        Self {
            max_latency_ms,
        }
    }
    
    /// Measure latency to a domain
    async fn measure_latency(&self, domain: &Arc<dyn DomainAdapter>) -> Result<u64> {
        // In a real implementation, we would measure the actual latency
        // For now, just return a placeholder value
        Ok(100)
    }
}

#[async_trait]
impl DomainSelectionStrategy for LatencyBasedStrategy {
    async fn select_domain(
        &self,
        domains: &[Arc<dyn DomainAdapter>],
        required_capabilities: &HashSet<String>,
        _preferences: &HashMap<String, String>,
    ) -> Result<DomainId> {
        if domains.is_empty() {
            return Err(Error::DomainNotFound("No domains available".to_string()));
        }
        
        // Measure latency to each domain
        let mut domain_latencies = Vec::with_capacity(domains.len());
        for domain in domains {
            if let Ok(latency) = self.measure_latency(domain).await {
                if latency <= self.max_latency_ms {
                    domain_latencies.push((domain.domain_id().clone(), latency));
                }
            }
        }
        
        // Sort by latency (lowest first)
        domain_latencies.sort_by_key(|(_, latency)| *latency);
        
        // Return the domain with the lowest latency, or an error if none is found
        if let Some((domain_id, _)) = domain_latencies.first() {
            Ok(domain_id.clone())
        } else {
            Err(Error::DomainNotFound("No domain with acceptable latency found".to_string()))
        }
    }
    
    async fn select_domains(
        &self,
        domains: &[Arc<dyn DomainAdapter>],
        required_capabilities: &HashSet<String>,
        preferences: &HashMap<String, String>,
        count: usize,
    ) -> Result<Vec<DomainId>> {
        if domains.is_empty() {
            return Err(Error::DomainNotFound("No domains available".to_string()));
        }
        
        // Measure latency to each domain
        let mut domain_latencies = Vec::with_capacity(domains.len());
        for domain in domains {
            if let Ok(latency) = self.measure_latency(domain).await {
                if latency <= self.max_latency_ms {
                    domain_latencies.push((domain.domain_id().clone(), latency));
                }
            }
        }
        
        // Sort by latency (lowest first)
        domain_latencies.sort_by_key(|(_, latency)| *latency);
        
        // Take the domains with the lowest latency
        let selected: Vec<DomainId> = domain_latencies
            .iter()
            .take(count)
            .map(|(id, _)| id.clone())
            .collect();
        
        if selected.is_empty() {
            Err(Error::DomainNotFound("No domain with acceptable latency found".to_string()))
        } else if selected.len() < count {
            Err(Error::InsufficientDomains(format!("Requested {} domains but only found {}", count, selected.len())))
        } else {
            Ok(selected)
        }
    }
}

/// Cost-based domain selection strategy
///
/// This strategy selects domains based on their estimated operation cost,
/// preferring domains with lower costs.
pub struct CostBasedStrategy {
    /// Maximum acceptable cost
    max_cost: f64,
}

impl CostBasedStrategy {
    /// Create a new cost-based strategy
    pub fn new(max_cost: f64) -> Self {
        Self {
            max_cost,
        }
    }
    
    /// Estimate the cost of an operation on a domain
    fn estimate_cost(&self, domain: &Arc<dyn DomainAdapter>, capabilities: &HashSet<String>) -> Result<f64> {
        // In a real implementation, we would estimate the actual cost
        // For now, just return a placeholder value
        Ok(1.0)
    }
}

#[async_trait]
impl DomainSelectionStrategy for CostBasedStrategy {
    async fn select_domain(
        &self,
        domains: &[Arc<dyn DomainAdapter>],
        required_capabilities: &HashSet<String>,
        _preferences: &HashMap<String, String>,
    ) -> Result<DomainId> {
        if domains.is_empty() {
            return Err(Error::DomainNotFound("No domains available".to_string()));
        }
        
        // Estimate cost for each domain
        let mut domain_costs = Vec::with_capacity(domains.len());
        for domain in domains {
            if let Ok(cost) = self.estimate_cost(domain, required_capabilities) {
                if cost <= self.max_cost {
                    domain_costs.push((domain.domain_id().clone(), cost));
                }
            }
        }
        
        // Sort by cost (lowest first)
        domain_costs.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return the domain with the lowest cost, or an error if none is found
        if let Some((domain_id, _)) = domain_costs.first() {
            Ok(domain_id.clone())
        } else {
            Err(Error::DomainNotFound("No domain with acceptable cost found".to_string()))
        }
    }
    
    async fn select_domains(
        &self,
        domains: &[Arc<dyn DomainAdapter>],
        required_capabilities: &HashSet<String>,
        preferences: &HashMap<String, String>,
        count: usize,
    ) -> Result<Vec<DomainId>> {
        if domains.is_empty() {
            return Err(Error::DomainNotFound("No domains available".to_string()));
        }
        
        // Estimate cost for each domain
        let mut domain_costs = Vec::with_capacity(domains.len());
        for domain in domains {
            if let Ok(cost) = self.estimate_cost(domain, required_capabilities) {
                if cost <= self.max_cost {
                    domain_costs.push((domain.domain_id().clone(), cost));
                }
            }
        }
        
        // Sort by cost (lowest first)
        domain_costs.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take the domains with the lowest cost
        let selected: Vec<DomainId> = domain_costs
            .iter()
            .take(count)
            .map(|(id, _)| id.clone())
            .collect();
        
        if selected.is_empty() {
            Err(Error::DomainNotFound("No domain with acceptable cost found".to_string()))
        } else if selected.len() < count {
            Err(Error::InsufficientDomains(format!("Requested {} domains but only found {}", count, selected.len())))
        } else {
            Ok(selected)
        }
    }
}

/// Composite domain selection strategy
///
/// This strategy combines multiple strategies with weights to make a selection.
pub struct CompositeStrategy {
    /// Strategies to combine
    strategies: Vec<(Box<dyn DomainSelectionStrategy>, f64)>,
}

impl CompositeStrategy {
    /// Create a new composite strategy
    pub fn new(strategies: Vec<(Box<dyn DomainSelectionStrategy>, f64)>) -> Self {
        Self {
            strategies,
        }
    }
}

#[async_trait]
impl DomainSelectionStrategy for CompositeStrategy {
    async fn select_domain(
        &self,
        domains: &[Arc<dyn DomainAdapter>],
        required_capabilities: &HashSet<String>,
        preferences: &HashMap<String, String>,
    ) -> Result<DomainId> {
        if self.strategies.is_empty() {
            return Err(Error::InvalidArgument("No strategies defined".to_string()));
        }
        
        // Use the strategy with the highest weight
        let (strategy, _) = self.strategies.iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();
        
        strategy.select_domain(domains, required_capabilities, preferences).await
    }
    
    async fn select_domains(
        &self,
        domains: &[Arc<dyn DomainAdapter>],
        required_capabilities: &HashSet<String>,
        preferences: &HashMap<String, String>,
        count: usize,
    ) -> Result<Vec<DomainId>> {
        if self.strategies.is_empty() {
            return Err(Error::InvalidArgument("No strategies defined".to_string()));
        }
        
        // Use the strategy with the highest weight
        let (strategy, _) = self.strategies.iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();
        
        strategy.select_domains(domains, required_capabilities, preferences, count).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tokio::runtime::Runtime;
    use crate::domain::DomainAdapter;
    use crate::domain::TransactionStatus;

    #[derive(Debug)]
    struct MockDomainAdapter {
        domain_id: DomainId,
        domain_type: DomainType,
        status: DomainStatus,
        connected: bool,
    }

    impl MockDomainAdapter {
        fn new(domain_id: DomainId, domain_type: DomainType, status: DomainStatus, connected: bool) -> Self {
            MockDomainAdapter {
                domain_id,
                domain_type,
                status,
                connected,
            }
        }
    }

    #[async_trait::async_trait]
    impl DomainAdapter for MockDomainAdapter {
        fn domain_id(&self) -> &DomainId {
            &self.domain_id
        }
        
        async fn domain_info(&self) -> Result<DomainInfo> {
            Ok(DomainInfo {
                id: self.domain_id.clone(),
                domain_type: self.domain_type.clone(),
                name: "Mock".to_string(),
                description: None,
                rpc_url: Some("http://localhost:8545".to_string()),
                explorer_url: None,
                chain_id: Some(1),
                native_currency: Some("TOKEN".to_string()),
                status: self.status.clone(),
                metadata: HashMap::new(),
            })
        }
        
        async fn current_height(&self) -> Result<BlockHeight> {
            Ok(BlockHeight::new(100))
        }
        
        async fn current_hash(&self) -> Result<BlockHash> {
            // Create a fixed-size array for BlockHash
            let hash_bytes = [1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            Ok(BlockHash(hash_bytes))
        }
        
        async fn current_timestamp(&self) -> Result<Timestamp> {
            Ok(Timestamp::new(1621234567))
        }
        
        async fn observe_fact(&self, _query: FactQuery) -> Result<FactType> {
            unimplemented!()
        }
        
        async fn submit_transaction(&self, _tx: crate::domain::Transaction) -> Result<crate::domain::TransactionId> {
            unimplemented!()
        }
        
        async fn get_transaction_receipt(&self, _tx_id: &crate::domain::TransactionId) -> Result<crate::domain::TransactionReceipt> {
            unimplemented!()
        }
        
        async fn get_time_map(&self) -> Result<causality_domain_map::TimeMapEntry> {
            unimplemented!()
        }
        
        async fn verify_block(&self, _height: BlockHeight, _hash: &BlockHash) -> Result<bool> {
            Ok(true)
        }
        
        async fn check_connectivity(&self) -> Result<bool> {
            Ok(self.connected)
        }
    }
    
    #[tokio::test]
    async fn test_domain_selector() {
        // Create mock domains
        let domain1 = DomainId("domain1".to_string());
        let domain2 = DomainId("domain2".to_string());
        
        let adapter1 = Arc::new(MockDomainAdapter::new(
            domain1.clone(),
            DomainType::Ethereum,
            DomainStatus::Active,
            true,
        ));
        
        let adapter2 = Arc::new(MockDomainAdapter::new(
            domain2.clone(),
            DomainType::Solana,
            DomainStatus::Active,
            true,
        ));
        
        // Create domain registry
        let mut registry = DomainRegistry::new();
        registry.register_domain(adapter1.clone());
        registry.register_domain(adapter2.clone());
        
        let registry_arc = Arc::new(registry);
        
        // Create time map
        let time_map = SharedTimeMap::new();
        
        // Create selector
        let mut selector = DomainSelector::new(registry_arc.clone(), time_map);
        
        // Add metrics
        selector.update_metrics(DomainMetrics {
            domain_id: domain1.clone(),
            reliability: 0.9,
            avg_latency: 500,
            cost_factor: 1.0,
            features: HashSet::from(["transfer".to_string(), "storage".to_string()]),
            last_update: chrono::Utc::now(),
        });
        
        selector.update_metrics(DomainMetrics {
            domain_id: domain2.clone(),
            reliability: 0.8,
            avg_latency: 300,
            cost_factor: 0.8,
            features: HashSet::from(["transfer".to_string()]),
            last_update: chrono::Utc::now(),
        });
        
        // Test selecting with default criteria
        let result = selector.select_domain("transfer", None).await.unwrap();
        assert!(result.is_some());
        
        // Test selection with specific criteria
        let mut criteria = SelectionCriteria::default();
        criteria.max_latency = Some(1000);
        
        let result = selector.select_domain("transfer", Some(&criteria)).await.unwrap();
        assert!(result.is_some());
        let selected = result.unwrap();
        assert_eq!(selected.domain_id, domain1);
        
        // Update criteria to prefer lower latency
        criteria.max_latency = Some(400);
        criteria.preferred_types = Some(HashSet::from([
            DomainType::Solana
        ]));
        
        let result = selector.select_domain("transfer", Some(&criteria)).await.unwrap();
        assert!(result.is_some());
        let selected = result.unwrap();
        assert_eq!(selected.domain_id, domain2);
        
        // Update criteria to require Ethereum
        criteria.required_types = Some(HashSet::from([
            DomainType::Ethereum
        ]));
        
        let result = selector.select_domain("transfer", Some(&criteria)).await.unwrap();
        assert!(result.is_some());
        let selected = result.unwrap();
        assert_eq!(selected.domain_id, domain1);
        
        // Update criteria to require unavailable feature
        criteria.required_features = Some(HashSet::from([
            "unavailable".to_string()
        ]));
        
        let result = selector.select_domain("transfer", Some(&criteria)).await.unwrap();
        assert!(result.is_none());
        
        // Test multiple domain selection
        let result = selector.select_multiple_domains("transfer", None, 2).await.unwrap();
        assert_eq!(result.len(), 2);
        
        // Test fault tolerance selection
        let result = selector.select_for_fault_tolerance("transfer", None, 2).await.unwrap();
        assert_eq!(result.len(), 2);
        
        // Verify domain types are diverse
        assert_ne!(result[0].info.domain_type, result[1].info.domain_type);
    }
} 