// Synchronization module for time map
//
// This module provides functionality for synchronizing time maps
// across domains and verifying time commitments.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Duration as StdDuration;
use chrono::{DateTime, Duration, Utc};
use tokio::sync::broadcast;
use tokio::time;
use tokio::time::timeout as tokio_timeout;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

use causality_types::{Error, Result};
use causality_types::Timestamp;
use crate::domain::DomainId;
use super::{TimeMap, TimeMapEntry, SharedTimeMap};
use super::TimePoint;
use crate::selection::TimeRange;
use crate::error::system_error;

/// Time synchronization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSyncConfig {
    /// Interval for sync operations in seconds
    pub sync_interval_seconds: u64,
    /// Maximum time deviation allowed in seconds
    pub max_time_deviation_seconds: u64,
    /// Minimum confidence threshold
    pub min_confidence_threshold: f64,
    /// Required verifications before committing
    pub required_verifications: u32,
    /// Domains to sync with
    pub domains: Vec<DomainId>,
    /// Additional sync parameters
    pub parameters: HashMap<String, String>,
}

impl Default for TimeSyncConfig {
    fn default() -> Self {
        Self {
            sync_interval_seconds: 60,
            max_time_deviation_seconds: 300,
            min_confidence_threshold: 0.7,
            required_verifications: 2,
            domains: Vec::new(),
            parameters: HashMap::new(),
        }
    }
}

/// Synchronization status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    /// Synchronization is in progress
    InProgress,
    /// Synchronization has been completed
    Completed,
    /// Synchronization has failed
    Failed(String),
    /// Synchronization has not been started
    NotStarted,
}

impl ToString for SyncStatus {
    fn to_string(&self) -> String {
        match self {
            SyncStatus::InProgress => "in_progress".to_string(),
            SyncStatus::Completed => "completed".to_string(),
            SyncStatus::Failed(err) => format!("failed: {}", err),
            SyncStatus::NotStarted => "not_started".to_string(),
        }
    }
}

/// Result of a synchronization operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Status of the sync operation
    pub status: SyncStatus,
    /// Domain that was synchronized
    pub domain_id: DomainId,
    /// Time point that was synchronized
    pub time_point: Option<TimePoint>,
    /// Verification status
    pub verification: VerificationStatus,
    /// Timestamp when the sync was started
    pub started_at: u64,
    /// Timestamp when the sync was completed
    pub completed_at: u64,
    /// Duration of the sync operation in seconds
    pub duration_seconds: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Time source
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSource {
    /// Local time source
    Local,
    /// Remote time source
    Remote(DomainId),
    /// Consensus time source (aggregated from multiple domains)
    Consensus(Vec<DomainId>),
    /// External time source
    External(String),
}

impl ToString for TimeSource {
    fn to_string(&self) -> String {
        match self {
            TimeSource::Local => "local".to_string(),
            TimeSource::Remote(domain_id) => format!("remote:{}", domain_id),
            TimeSource::Consensus(domains) => format!("consensus:{}", domains.iter().map(|d| d.to_string()).collect::<Vec<String>>().join(",")),
            TimeSource::External(source) => format!("external:{}", source),
        }
    }
}

/// Synchronization strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStrategy {
    /// Sync with a single domain
    SingleDomain(DomainId),
    /// Sync with multiple domains
    MultiDomain(Vec<DomainId>),
    /// Sync with a consensus of domains
    Consensus {
        domains: Vec<DomainId>,
        threshold: usize,
    },
    /// Sync with an external time source
    External(String),
}

impl ToString for SyncStrategy {
    fn to_string(&self) -> String {
        match self {
            SyncStrategy::SingleDomain(domain_id) => format!("single_domain:{}", domain_id),
            SyncStrategy::MultiDomain(domains) => format!("multi_domain:{}", domains.iter().map(|d| d.to_string()).collect::<Vec<String>>().join(",")),
            SyncStrategy::Consensus { domains, threshold } => format!("consensus:{}:{}", domains.iter().map(|d| d.to_string()).collect::<Vec<String>>().join(","), threshold),
            SyncStrategy::External(source) => format!("external:{}", source),
        }
    }
}

/// Verification status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// Verification is pending
    Pending,
    /// Verification has been completed
    Verified,
    /// Verification has failed
    Failed(String),
    /// Verification has not been started
    NotStarted,
}

impl ToString for VerificationStatus {
    fn to_string(&self) -> String {
        match self {
            VerificationStatus::Pending => "pending".to_string(),
            VerificationStatus::Verified => "verified".to_string(),
            VerificationStatus::Failed(err) => format!("failed: {}", err),
            VerificationStatus::NotStarted => "not_started".to_string(),
        }
    }
}

/// Time commitment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeCommitment {
    /// Domain ID
    pub domain_id: DomainId,
    /// Time point
    pub time_point: TimePoint,
    /// Source of the time commitment
    pub source: TimeSource,
    /// Verification status
    pub verification: VerificationStatus,
    /// Timestamp when the commitment was created
    pub created_at: u64,
    /// Timestamp when the commitment was last updated
    pub updated_at: u64,
    /// Signatures or proofs (if any)
    pub proofs: Vec<Vec<u8>>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Time synchronization manager
#[async_trait]
pub trait TimeSyncManager: Send + Sync {
    /// Initialize the sync manager
    async fn initialize(&self, config: TimeSyncConfig) -> Result<()>;
    
    /// Start synchronization
    async fn start_sync(&self) -> Result<()>;
    
    /// Stop synchronization
    async fn stop_sync(&self) -> Result<()>;
    
    /// Get the current sync status
    async fn get_status(&self) -> Result<SyncStatus>;
    
    /// Get the last sync result
    async fn get_last_result(&self) -> Result<Option<SyncResult>>;
    
    /// Synchronize with a specific domain
    async fn sync_with_domain(&self, domain_id: &DomainId) -> Result<SyncResult>;
    
    /// Synchronize with multiple domains
    async fn sync_with_domains(&self, domain_ids: &[DomainId]) -> Result<Vec<SyncResult>>;
    
    /// Get time commitments for a specific domain
    async fn get_commitments(&self, domain_id: &DomainId) -> Result<Vec<TimeCommitment>>;
    
    /// Get time commitments for a specific time range
    async fn get_commitments_in_range(&self, domain_id: &DomainId, range: &TimeRange) -> Result<Vec<TimeCommitment>>;
    
    /// Verify a time commitment
    async fn verify_commitment(&self, commitment: &TimeCommitment) -> Result<VerificationStatus>;
}

/// Time verification service
#[async_trait]
pub trait TimeVerificationService: Send + Sync {
    /// Verify a time point for a specific domain
    async fn verify_time_point(&self, domain_id: &DomainId, time_point: &TimePoint) -> Result<VerificationStatus>;
    
    /// Verify a time commitment
    async fn verify_commitment(&self, commitment: &TimeCommitment) -> Result<VerificationStatus>;
    
    /// Create a time commitment
    async fn create_commitment(&self, domain_id: &DomainId, time_point: &TimePoint) -> Result<TimeCommitment>;
    
    /// Get supported verification methods
    fn get_verification_methods(&self) -> Vec<String>;
}

/// Consensus verification manager
pub struct ConsensusVerificationManager {
    /// Verification services
    verification_services: RwLock<Vec<Arc<dyn TimeVerificationService>>>,
    /// Consensus threshold
    threshold: usize,
}

impl ConsensusVerificationManager {
    /// Create a new consensus verification manager
    pub fn new(threshold: usize) -> Self {
        Self {
            verification_services: RwLock::new(Vec::new()),
            threshold,
        }
    }
    
    /// Register a verification service
    pub fn register_service(&self, service: Arc<dyn TimeVerificationService>) -> Result<()> {
        let mut services = self.verification_services.write().map_err(|_| system_error("Failed to acquire write lock on verification services"))?;
        services.push(service);
        Ok(())
    }
    
    /// Get registered verification services
    pub fn get_services(&self) -> Result<Vec<Arc<dyn TimeVerificationService>>> {
        let services = self.verification_services.read().map_err(|_| system_error("Failed to acquire read lock on verification services"))?;
        Ok(services.clone())
    }
}

#[async_trait]
impl TimeVerificationService for ConsensusVerificationManager {
    async fn verify_time_point(&self, domain_id: &DomainId, time_point: &TimePoint) -> Result<VerificationStatus> {
        let services = self.get_services()?;
        
        if services.is_empty() {
            return Ok(VerificationStatus::Failed("No verification services registered".to_string()));
        }
        
        let mut verified_count = 0;
        let mut failed_messages = Vec::new();
        
        for service in services.iter() {
            match service.verify_time_point(domain_id, time_point).await {
                Ok(VerificationStatus::Verified) => {
                    verified_count += 1;
                }
                Ok(VerificationStatus::Failed(msg)) => {
                    failed_messages.push(msg);
                }
                Ok(_) => {}
                Err(e) => {
                    failed_messages.push(format!("Verification error: {}", e));
                }
            }
        }
        
        if verified_count >= self.threshold {
            Ok(VerificationStatus::Verified)
        } else {
            Ok(VerificationStatus::Failed(format!(
                "Verification failed: {}/{} services failed ({})",
                services.len() - verified_count,
                services.len(),
                failed_messages.join(", ")
            )))
        }
    }
    
    async fn verify_commitment(&self, commitment: &TimeCommitment) -> Result<VerificationStatus> {
        let services = self.get_services()?;
        
        if services.is_empty() {
            return Ok(VerificationStatus::Failed("No verification services registered".to_string()));
        }
        
        let mut verified_count = 0;
        let mut failed_messages = Vec::new();
        
        for service in services.iter() {
            match service.verify_commitment(commitment).await {
                Ok(VerificationStatus::Verified) => {
                    verified_count += 1;
                }
                Ok(VerificationStatus::Failed(msg)) => {
                    failed_messages.push(msg);
                }
                Ok(_) => {}
                Err(e) => {
                    failed_messages.push(format!("Verification error: {}", e));
                }
            }
        }
        
        if verified_count >= self.threshold {
            Ok(VerificationStatus::Verified)
        } else {
            Ok(VerificationStatus::Failed(format!(
                "Commitment verification failed: {}/{} services failed ({})",
                services.len() - verified_count,
                services.len(),
                failed_messages.join(", ")
            )))
        }
    }
    
    async fn create_commitment(&self, domain_id: &DomainId, time_point: &TimePoint) -> Result<TimeCommitment> {
        // Use the first service that succeeds
        let services = self.get_services()?;
        
        if services.is_empty() {
            return Err(system_error("No verification services registered"));
        }
        
        for service in services.iter() {
            match service.create_commitment(domain_id, time_point).await {
                Ok(commitment) => return Ok(commitment),
                Err(_) => continue,
            }
        }
        
        Err(system_error("Failed to create commitment with any verification service"))
    }
    
    fn get_verification_methods(&self) -> Vec<String> {
        let services = match self.get_services() {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        
        let mut methods = Vec::new();
        for service in services.iter() {
            methods.extend(service.get_verification_methods());
        }
        
        methods
    }
}

/// Time synchronization manager
pub struct TimeSyncManagerImpl {
    /// Configuration for time sync
    config: TimeSyncConfig,
    /// Shared time map
    time_map: SharedTimeMap,
    /// Time point providers by domain
    providers: Arc<RwLock<HashMap<DomainId, TimePointProvider>>>,
    /// Recently observed time points
    recent_points: Arc<RwLock<Vec<(DomainId, TimePoint)>>>,
    /// Broadcast channel for sync events
    event_tx: broadcast::Sender<SyncResult>,
    /// Running flag
    running: Arc<RwLock<bool>>,
}

impl TimeSyncManagerImpl {
    /// Create a new time synchronization manager
    pub fn new(config: TimeSyncConfig, time_map: SharedTimeMap) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        
        TimeSyncManagerImpl {
            config,
            time_map,
            providers: Arc::new(RwLock::new(HashMap::new())),
            recent_points: Arc::new(RwLock::new(Vec::new())),
            event_tx,
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Register a time point provider for a domain
    pub fn register_provider(
        &self,
        domain_id: DomainId,
        provider: TimePointProvider,
    ) -> Result<()> {
        let mut providers = self.providers.write().map_err(|_| 
            Error::InternalError("Failed to lock providers".to_string()))?;
        
        providers.insert(domain_id, provider);
        
        Ok(())
    }
    
    /// Get a subscription for sync events
    pub fn subscribe(&self) -> broadcast::Receiver<SyncResult> {
        self.event_tx.subscribe()
    }
    
    /// Start the synchronization loop
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().map_err(|_| 
            Error::InternalError("Failed to lock running flag".to_string()))?;
        
        if *running {
            return Ok(());
        }
        
        *running = true;
        
        // Clone the necessary components for the task
        let config = self.config.clone();
        let time_map = self.time_map.clone();
        let providers = self.providers.clone();
        let recent_points = self.recent_points.clone();
        let event_tx = self.event_tx.clone();
        let running_flag = self.running.clone();
        
        // Spawn the sync task
        tokio::spawn(async move {
            let interval_duration = StdDuration::from_secs(config.sync_interval_seconds);
            let mut interval = time::interval(interval_duration);
            
            loop {
                interval.tick().await;
                
                // Check if we should continue running
                let should_run = match running_flag.read() {
                    Ok(flag) => *flag,
                    Err(_) => false,
                };
                
                if !should_run {
                    break;
                }
                
                // Perform synchronization
                let result = Self::perform_sync(
                    &config,
                    &time_map,
                    &providers,
                    &recent_points,
                ).await;
                
                // Broadcast the result
                let _ = event_tx.send(result);
            }
        });
        
        Ok(())
    }
    
    /// Stop the synchronization loop
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().map_err(|_| 
            Error::InternalError("Failed to lock running flag".to_string()))?;
        
        *running = false;
        
        Ok(())
    }
    
    /// Perform a synchronization operation
    async fn perform_sync(
        config: &TimeSyncConfig,
        time_map: &SharedTimeMap,
        providers: &Arc<RwLock<HashMap<DomainId, TimePointProvider>>>,
        recent_points: &Arc<RwLock<Vec<(DomainId, TimePoint)>>>,
    ) -> SyncResult {
        let started_at = Utc::now().timestamp();
        let mut result = SyncResult {
            status: SyncStatus::InProgress,
            domain_id: config.domains[0].clone(),
            time_point: None,
            verification: VerificationStatus::NotStarted,
            started_at,
            completed_at: 0,
            duration_seconds: 0,
            metadata: HashMap::new(),
        };
        
        // Get the list of domains to sync
        let domains = match providers.read() {
            Ok(providers) => providers.keys().cloned().collect::<Vec<_>>(),
            Err(_) => {
                result.status = SyncStatus::Failed("Failed to lock providers".to_string());
                return result;
            }
        };
        
        if domains.is_empty() {
            result.status = SyncStatus::Failed("No domains registered for synchronization".to_string());
            return result;
        }
        
        // Create a copy of the current time map
        let current_map = match time_map.get() {
            Ok(map) => map,
            Err(_) => {
                result.status = SyncStatus::Failed("Failed to read current time map".to_string());
                return result;
            }
        };
        
        // Create a new time map for updates
        let mut updated_map = current_map.clone();
        
        // Synchronize each domain
        for domain_id in domains {
            // Skip domains that don't have a provider
            let provider = match providers.read() {
                Ok(providers) => match providers.get(&domain_id) {
                    Some(provider) => provider,
                    None => continue,
                },
                Err(_) => {
                    result.status = SyncStatus::Failed(format!("Failed to lock providers for domain: {}", domain_id));
                    return result;
                }
            };
            
            // Fetch the time point with a timeout
            let time_point_result = tokio_timeout(
                StdDuration::from_secs(config.max_time_deviation_seconds as u64),
                async {
                    match provider(domain_id.clone()) {
                        Ok(point) => Ok(point),
                        Err(e) => Err(e),
                    }
                },
            ).await;
            
            match time_point_result {
                Ok(Ok(time_point)) => {
                    // Check if the time point has sufficient confidence
                    if time_point.confidence < config.min_confidence_threshold {
                        result.status = SyncStatus::Failed(format!("Insufficient confidence: {}", time_point.confidence));
                        return result;
                    }
                    
                    // Check if the time point is verifiable (if required)
                    if config.required_verifications > 0 && !time_point.verified {
                        result.status = SyncStatus::Failed("Time point not verified".to_string());
                        return result;
                    }
                    
                    // Create a time map entry from the time point
                    let entry = TimeMapEntry::new(
                        domain_id.clone(),
                        time_point.height,
                        time_point.hash.clone(),
                        time_point.timestamp,
                    )
                    .with_confidence(time_point.confidence)
                    .with_verification(time_point.verified)
                    .with_source(&time_point.source);
                    
                    // Add time point to recent points
                    if let Ok(mut recent) = recent_points.write() {
                        recent.push((domain_id.clone(), time_point.clone()));
                        
                        // Trim the list if it gets too large
                        if recent.len() > config.required_verifications as usize {
                            recent.sort_by(|a, b| a.1.timestamp.cmp(&b.1.timestamp));
                            recent.drain(0..recent.len() - config.required_verifications as usize);
                        }
                    }
                    
                    // Update the time map
                    updated_map.update_domain(entry);
                    
                    // Mark the domain as synced
                    result.domain_id = domain_id.clone();
                    result.time_point = Some(time_point.clone());
                    result.status = SyncStatus::Completed;
                }
                Ok(Err(e)) => {
                    result.status = SyncStatus::Failed(format!("Provider error: {}", e));
                    return result;
                }
                Err(_) => {
                    result.status = SyncStatus::Failed("Operation timed out".to_string());
                    return result;
                }
            }
        }
        
        // Update the shared time map if we have any successful syncs
        if result.status == SyncStatus::Completed {
            if let Err(e) = time_map.merge(&updated_map) {
                result.status = SyncStatus::Failed(format!("Failed to update time map: {}", e));
            }
            
            result.completed_at = Utc::now().timestamp();
            result.duration_seconds = result.completed_at - result.started_at;
        }
        
        result
    }
    
    /// Manually trigger a synchronization
    pub async fn sync_now(&self) -> Result<SyncResult> {
        let config = self.config.clone();
        let providers = self.providers.clone();
        let recent_points = self.recent_points.clone();
        
        let result = Self::perform_sync(
            &config,
            &self.time_map,
            &providers,
            &recent_points,
        ).await;
        
        // Broadcast the result
        let _ = self.event_tx.send(result.clone());
        
        Ok(result)
    }
    
    /// Get the most recent time points
    pub fn get_recent_points(&self) -> Result<Vec<(DomainId, TimePoint)>> {
        let recent = self.recent_points.read().map_err(|_| 
            Error::InternalError("Failed to lock recent points".to_string()))?;
        
        Ok(recent.clone())
    }
    
    /// Calculate the time drift between domains
    pub fn calculate_drift(
        &self,
        domain1: &DomainId,
        domain2: &DomainId,
    ) -> Result<Duration> {
        let time_map = self.time_map.get()?;
        
        let timestamp1 = time_map.get_timestamp(domain1).ok_or_else(|| 
            Error::NotFound(format!("Domain not found: {}", domain1)))?;
        
        let timestamp2 = time_map.get_timestamp(domain2).ok_or_else(|| 
            Error::NotFound(format!("Domain not found: {}", domain2)))?;
        
        let drift_seconds = timestamp1.value() as i64 - timestamp2.value() as i64;
        
        Ok(Duration::seconds(drift_seconds))
    }
    
    /// Check if the time drift between domains is acceptable
    pub fn is_drift_acceptable(
        &self,
        domain1: &DomainId,
        domain2: &DomainId,
    ) -> Result<bool> {
        let drift = self.calculate_drift(domain1, domain2)?;
        let max_drift = Duration::seconds(self.config.max_time_deviation_seconds as i64);
        
        Ok(drift.abs() <= max_drift)
    }
    
    /// Get the current time map
    pub fn get_time_map(&self) -> Result<TimeMap> {
        self.time_map.get()
    }
    
    /// Create a simple time point provider from a closure
    pub fn create_provider<F>(f: F) -> TimePointProvider 
    where
        F: Fn(DomainId) -> Result<TimePoint> + Send + Sync + 'static,
    {
        Box::new(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causality_types::BlockHash;
    
    fn create_test_domain_id(id: u8) -> DomainId {
        format!("domain_{}", id).into()
    }
    
    fn create_test_time_point(
        height: u64,
        timestamp: u64,
        confidence: f64,
        verified: bool,
    ) -> TimePoint {
        TimePoint {
            height,
            hash: format!("hash_{}", height).into(),
            timestamp,
            confidence,
            verified,
            source: "test".to_string(),
        }
    }
    
    #[test]
    fn test_sync_result() {
        let started_at = Utc::now();
        let mut result = SyncResult {
            status: SyncStatus::InProgress,
            domain_id: create_test_domain_id(1),
            time_point: None,
            verification: VerificationStatus::NotStarted,
            started_at: started_at.timestamp(),
            completed_at: 0,
            duration_seconds: 0,
            metadata: HashMap::new(),
        };
        
        // Check initial state
        assert_eq!(result.status, SyncStatus::InProgress);
        assert_eq!(result.domain_id, create_test_domain_id(1));
        assert!(result.time_point.is_none());
        
        // Add a synced domain
        let domain1 = create_test_domain_id(1);
        result.domain_id = domain1.clone();
        
        // Add a failed domain
        let domain2 = create_test_domain_id(2);
        result.status = SyncStatus::Failed(format!("Test failure: {}", domain2));
        
        // Complete the result
        let time_map = TimeMap::new();
        result.status = SyncStatus::Completed;
        result.time_point = Some(create_test_time_point(100, 1000, 1.0, true));
        result.completed_at = Utc::now().timestamp();
        result.duration_seconds = result.completed_at - result.started_at;
        
        // Check final state
        assert_eq!(result.status, SyncStatus::Completed);
        assert_eq!(result.domain_id, domain1);
        assert!(result.time_point.is_some());
        assert!(!result.is_fully_successful()); // Has failed domains
    }
    
    #[tokio::test]
    async fn test_time_sync_manager_basic() -> Result<()> {
        // Create a shared time map
        let shared_map = SharedTimeMap::new();
        
        // Create a config with fast sync
        let config = TimeSyncConfig {
            sync_interval_seconds: 1,
            max_time_deviation_seconds: 5,
            min_confidence_threshold: 0.7,
            required_verifications: 2,
            domains: vec![create_test_domain_id(1)],
            parameters: HashMap::new(),
        };
        
        // Create a sync manager
        let manager = TimeSyncManagerImpl::new(config, shared_map);
        
        // Register a provider for a domain
        let domain1 = create_test_domain_id(1);
        
        let provider = TimeSyncManagerImpl::create_provider(move |_| {
            Ok(create_test_time_point(100, 1000, 1.0, true))
        });
        
        manager.register_provider(domain1.clone(), provider)?;
        
        // Manually trigger a sync
        let result = manager.sync_now().await?;
        
        // Check the result
        assert_eq!(result.status, SyncStatus::Completed);
        assert_eq!(result.domain_id, domain1);
        assert!(result.time_point.is_some());
        
        // Check the time map was updated
        let time_map = manager.get_time_map()?;
        assert!(time_map.contains_domain(&domain1));
        assert_eq!(time_map.get_height(&domain1), Some(100));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_time_sync_manager_drift() -> Result<()> {
        // Create a shared time map
        let shared_map = SharedTimeMap::new();
        
        // Create a config with specific max drift
        let config = TimeSyncConfig {
            max_time_deviation_seconds: 100,
            min_confidence_threshold: 0.7,
            required_verifications: 2,
            domains: vec![create_test_domain_id(1), create_test_domain_id(2)],
            parameters: HashMap::new(),
        };
        
        // Create a sync manager
        let manager = TimeSyncManagerImpl::new(config, shared_map);
        
        // Register providers for two domains
        let domain1 = create_test_domain_id(1);
        let domain2 = create_test_domain_id(2);
        
        let provider1 = TimeSyncManagerImpl::create_provider(move |_| {
            Ok(create_test_time_point(100, 1000, 1.0, true))
        });
        
        let provider2 = TimeSyncManagerImpl::create_provider(move |_| {
            Ok(create_test_time_point(200, 1050, 1.0, true))
        });
        
        manager.register_provider(domain1.clone(), provider1)?;
        manager.register_provider(domain2.clone(), provider2)?;
        
        // Manually trigger a sync
        let _ = manager.sync_now().await?;
        
        // Calculate drift
        let drift = manager.calculate_drift(&domain1, &domain2)?;
        assert_eq!(drift, Duration::seconds(-50));
        
        // Check if drift is acceptable
        let acceptable = manager.is_drift_acceptable(&domain1, &domain2)?;
        assert!(acceptable);
        
        // Update domain2 with a larger time difference
        let provider2 = TimeSyncManagerImpl::create_provider(move |_| {
            Ok(create_test_time_point(300, 1200, 1.0, true))
        });
        
        manager.register_provider(domain2.clone(), provider2)?;
        
        // Sync again
        let _ = manager.sync_now().await?;
        
        // Calculate drift again
        let drift = manager.calculate_drift(&domain1, &domain2)?;
        assert_eq!(drift, Duration::seconds(-200));
        
        // Check if drift is acceptable
        let acceptable = manager.is_drift_acceptable(&domain1, &domain2)?;
        assert!(!acceptable);
        
        Ok(())
    }
} 