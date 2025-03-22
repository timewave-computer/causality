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

use crate::error::{Error, Result};
use crate::domain::{
    DomainId, DomainRegistry, DomainInfo, DomainType, DomainStatus, 
    DomainAdapter, FactQuery
};
use crate::domain::SharedTimeMap;
use crate::domain::{DomainSyncInfo, DomainSyncStatus};
use crate::types::{BlockHeight, BlockHash, Timestamp};

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
        
        async fn get_time_map(&self) -> Result<crate::domain::map_map::TimeMapEntry> {
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