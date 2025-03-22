// Time Synchronization Implementation
//
// This module provides functionality for synchronizing time across different
// domains and ensuring consistent timelines.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Duration as StdDuration;
use chrono::{DateTime, Duration, Utc};
use tokio::sync::broadcast;
use tokio::time;
use tokio::time::timeout as tokio_timeout;
use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};
use crate::types::Timestamp;
use crate::domain::DomainId;
use super::{TimeMap, TimeMapEntry, SharedTimeMap};
use super::TimePoint;

/// Configuration for the time synchronization manager
#[derive(Debug, Clone)]
pub struct TimeSyncConfig {
    /// How often to attempt synchronization (in seconds)
    pub sync_interval: u64,
    /// Timeout for sync operations (in seconds)
    pub sync_timeout: u64,
    /// Maximum time difference allowed between domains (in seconds)
    pub max_time_difference: u64,
    /// Minimum confidence threshold for accepting time points
    pub min_confidence: f64,
    /// Number of history points to maintain
    pub history_size: usize,
    /// Whether to verify time points cryptographically
    pub verify_time_points: bool,
}

impl Default for TimeSyncConfig {
    fn default() -> Self {
        TimeSyncConfig {
            sync_interval: 60,
            sync_timeout: 30,
            max_time_difference: 300,
            min_confidence: 0.7,
            history_size: 100,
            verify_time_points: true,
        }
    }
}

/// Status of a time synchronization
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStatus {
    /// Sync is in progress
    InProgress,
    /// Sync completed successfully
    Completed,
    /// Sync failed
    Failed(String),
    /// Sync was skipped
    Skipped(String),
}

/// Result of a time synchronization operation
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Status of the sync operation
    pub status: SyncStatus,
    /// Domains that were synchronized
    pub synced_domains: HashSet<DomainId>,
    /// Domains that failed to synchronize
    pub failed_domains: HashMap<DomainId, String>,
    /// Updated time map (if successful)
    pub time_map: Option<TimeMap>,
    /// When the sync was started
    pub started_at: DateTime<Utc>,
    /// When the sync completed
    pub completed_at: DateTime<Utc>,
    /// Duration of the sync operation
    pub duration_ms: u64,
}

impl SyncResult {
    /// Create a new sync result
    pub fn new(started_at: DateTime<Utc>) -> Self {
        SyncResult {
            status: SyncStatus::InProgress,
            synced_domains: HashSet::new(),
            failed_domains: HashMap::new(),
            time_map: None,
            started_at,
            completed_at: Utc::now(),
            duration_ms: 0,
        }
    }
    
    /// Mark the sync as completed
    pub fn complete(mut self, time_map: TimeMap) -> Self {
        self.status = SyncStatus::Completed;
        self.time_map = Some(time_map);
        self.completed_at = Utc::now();
        self.duration_ms = self.completed_at.timestamp_millis() as u64 - 
                         self.started_at.timestamp_millis() as u64;
        self
    }
    
    /// Mark the sync as failed
    pub fn fail(mut self, reason: &str) -> Self {
        self.status = SyncStatus::Failed(reason.to_string());
        self.completed_at = Utc::now();
        self.duration_ms = self.completed_at.timestamp_millis() as u64 - 
                         self.started_at.timestamp_millis() as u64;
        self
    }
    
    /// Mark the sync as skipped
    pub fn skip(mut self, reason: &str) -> Self {
        self.status = SyncStatus::Skipped(reason.to_string());
        self.completed_at = Utc::now();
        self.duration_ms = self.completed_at.timestamp_millis() as u64 - 
                         self.started_at.timestamp_millis() as u64;
        self
    }
    
    /// Add a successfully synced domain
    pub fn add_synced_domain(mut self, domain_id: DomainId) -> Self {
        self.synced_domains.insert(domain_id);
        self
    }
    
    /// Add a failed domain with reason
    pub fn add_failed_domain(mut self, domain_id: DomainId, reason: &str) -> Self {
        self.failed_domains.insert(domain_id, reason.to_string());
        self
    }
    
    /// Check if all domains were successfully synced
    pub fn is_fully_successful(&self) -> bool {
        self.status == SyncStatus::Completed && self.failed_domains.is_empty()
    }
    
    /// Get the number of synced domains
    pub fn synced_count(&self) -> usize {
        self.synced_domains.len()
    }
    
    /// Get the number of failed domains
    pub fn failed_count(&self) -> usize {
        self.failed_domains.len()
    }
}

/// Source type for time points
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeSource {
    /// Local time observation
    Local,
    /// Remote peer provided this time point
    Peer(String),
    /// External source (e.g., API, oracle)
    External(String),
    /// Derived through consensus
    Consensus,
}

impl ToString for TimeSource {
    fn to_string(&self) -> String {
        match self {
            TimeSource::Local => "local".to_string(),
            TimeSource::Peer(id) => format!("peer:{}", id),
            TimeSource::External(src) => format!("external:{}", src),
            TimeSource::Consensus => "consensus".to_string(),
        }
    }
}

/// Time synchronization strategy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStrategy {
    /// Pull time from specified sources
    Pull(Vec<TimeSource>),
    /// Push time to specified targets
    Push(Vec<DomainId>),
    /// Coordinate time with peers
    Coordinate,
    /// Use consensus to establish time
    Consensus,
}

/// Function type for time point providers
pub type TimePointProvider = Box<dyn Fn(DomainId) -> Result<TimePoint> + Send + Sync>;

/// Time verification status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// Not verified
    Unverified,
    /// Verification in progress
    InProgress,
    /// Verified successfully
    Verified,
    /// Verification failed
    Failed(String),
}

/// A cryptographic commitment for a time point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeCommitment {
    /// The time point being committed to
    pub time_point: TimeMapEntry,
    /// Signature over the time point by a trusted authority
    pub signature: Vec<u8>,
    /// Public key of the signer
    pub public_key: Vec<u8>,
    /// When this commitment was created
    pub created_at: DateTime<Utc>,
    /// Verification status
    pub status: VerificationStatus,
}

impl TimeCommitment {
    /// Create a new time commitment with an unverified status
    pub fn new(time_point: TimeMapEntry, signature: Vec<u8>, public_key: Vec<u8>) -> Self {
        TimeCommitment {
            time_point,
            signature,
            public_key,
            created_at: Utc::now(),
            status: VerificationStatus::Unverified,
        }
    }
    
    /// Set the verification status
    pub fn with_status(mut self, status: VerificationStatus) -> Self {
        self.status = status;
        self
    }
    
    /// Check if this commitment is verified
    pub fn is_verified(&self) -> bool {
        self.status == VerificationStatus::Verified
    }
}

/// Time verification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    /// Whether to verify time points
    pub verify_enabled: bool,
    /// Minimum number of verifiers required for consensus
    pub min_verifiers: usize,
    /// List of trusted public keys for verification
    pub trusted_keys: Vec<Vec<u8>>,
    /// Verification timeout in seconds
    pub timeout: u64,
    /// Maximum accepted time difference (in seconds) between verifiers
    pub max_drift: u64,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        VerificationConfig {
            verify_enabled: false,
            min_verifiers: 1,
            trusted_keys: Vec::new(),
            timeout: 30,
            max_drift: 60,
        }
    }
}

/// A service for verifying time points
#[derive(Debug)]
pub struct TimeVerificationService {
    /// Configuration for this verification service
    config: VerificationConfig,
    /// Cache of verified time points
    verified_cache: HashMap<String, TimeCommitment>,
}

impl TimeVerificationService {
    /// Create a new time verification service
    pub fn new(config: VerificationConfig) -> Self {
        TimeVerificationService {
            config,
            verified_cache: HashMap::new(),
        }
    }
    
    /// Create a key for the cache
    fn cache_key(domain_id: &DomainId, timestamp: Timestamp) -> String {
        format!("{}:{}", domain_id, timestamp)
    }
    
    /// Check if a time point is already verified
    pub fn is_verified(&self, domain_id: &DomainId, timestamp: Timestamp) -> bool {
        let key = Self::cache_key(domain_id, timestamp);
        self.verified_cache.get(&key)
            .map(|commitment| commitment.is_verified())
            .unwrap_or(false)
    }
    
    /// Verify a time point using cryptographic attestation
    /// 
    /// This verifies that the given time point is attested by a trusted authority.
    /// In a real implementation, this would validate cryptographic signatures.
    pub async fn verify_time_point(&mut self, time_point: &TimeMapEntry) -> Result<VerificationStatus> {
        // If verification is not enabled, return as verified
        if !self.config.verify_enabled {
            return Ok(VerificationStatus::Verified);
        }
        
        let domain_id = &time_point.domain_id;
        let timestamp = time_point.timestamp;
        let key = Self::cache_key(domain_id, timestamp);
        
        // If already verified, return from cache
        if let Some(commitment) = self.verified_cache.get(&key) {
            if commitment.is_verified() {
                return Ok(VerificationStatus::Verified);
            }
        }
        
        // Mark as in progress
        let mut status = VerificationStatus::InProgress;
        
        // Simulate verification using consensus
        // In a real implementation, this would verify cryptographic signatures from multiple sources
        let signatures_valid = true;  // Placeholder for signature verification
        let sufficient_verifiers = true;  // Placeholder for consensus check
        
        if signatures_valid && sufficient_verifiers {
            status = VerificationStatus::Verified;
        } else {
            status = VerificationStatus::Failed("Verification failed".to_string());
        }
        
        // Update cache with result
        let commitment = TimeCommitment::new(
            time_point.clone(),
            vec![],  // Placeholder for signature
            vec![],  // Placeholder for public key
        ).with_status(status.clone());
        
        self.verified_cache.insert(key, commitment);
        
        Ok(status)
    }
    
    /// Verify multiple time points for consensus
    pub async fn verify_time_map(&mut self, time_map: &TimeMap) -> Result<HashMap<DomainId, VerificationStatus>> {
        let mut results = HashMap::new();
        
        for (domain_id, entry) in &time_map.entries {
            let status = self.verify_time_point(entry).await?;
            results.insert(domain_id.clone(), status);
        }
        
        Ok(results)
    }
}

/// Consensus-based verification manager that coordinates multiple verifiers
#[derive(Debug)]
pub struct ConsensusVerificationManager {
    /// The verification service
    service: TimeVerificationService,
    /// Consensus threshold (0.0 - 1.0)
    threshold: f64,
    /// Cache of consensus results
    consensus_cache: HashMap<String, VerificationStatus>,
}

impl ConsensusVerificationManager {
    /// Create a new consensus verification manager
    pub fn new(config: VerificationConfig, threshold: f64) -> Self {
        ConsensusVerificationManager {
            service: TimeVerificationService::new(config),
            threshold: threshold.max(0.0).min(1.0),
            consensus_cache: HashMap::new(),
        }
    }
    
    /// Create a cache key
    fn cache_key(domain_id: &DomainId, timestamp: Timestamp) -> String {
        format!("consensus:{}:{}", domain_id, timestamp)
    }
    
    /// Verify a time point using consensus among verifiers
    /// 
    /// This would typically gather attestations from multiple sources and require
    /// a threshold of them to agree before considering the time point verified.
    pub async fn verify_with_consensus(&mut self, time_point: &TimeMapEntry) -> Result<VerificationStatus> {
        let domain_id = &time_point.domain_id;
        let timestamp = time_point.timestamp;
        let key = Self::cache_key(domain_id, timestamp);
        
        // If already in cache, return result
        if let Some(status) = self.consensus_cache.get(&key) {
            return Ok(status.clone());
        }
        
        // Use the verification service to verify the time point
        let status = self.service.verify_time_point(time_point).await?;
        
        // In a real implementation, this would gather reports from multiple sources
        // and check if a threshold of them agree on the time point.
        
        // Cache the consensus result
        self.consensus_cache.insert(key, status.clone());
        
        Ok(status)
    }
}

/// Time synchronization manager
pub struct TimeSyncManager {
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

impl TimeSyncManager {
    /// Create a new time synchronization manager
    pub fn new(config: TimeSyncConfig, time_map: SharedTimeMap) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        
        TimeSyncManager {
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
            let interval_duration = StdDuration::from_secs(config.sync_interval);
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
        let started_at = Utc::now();
        let mut result = SyncResult::new(started_at);
        
        // Get the list of domains to sync
        let domains = match providers.read() {
            Ok(providers) => providers.keys().cloned().collect::<Vec<_>>(),
            Err(_) => {
                return result.fail("Failed to lock providers");
            }
        };
        
        if domains.is_empty() {
            return result.skip("No domains registered for synchronization");
        }
        
        // Create a copy of the current time map
        let current_map = match time_map.get() {
            Ok(map) => map,
            Err(_) => {
                return result.fail("Failed to read current time map");
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
                    result = result.add_failed_domain(domain_id, "Failed to lock providers");
                    continue;
                }
            };
            
            // Fetch the time point with a timeout
            let time_point_result = tokio_timeout(
                StdDuration::from_secs(config.sync_timeout),
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
                    if time_point.confidence < config.min_confidence {
                        result = result.add_failed_domain(
                            domain_id.clone(),
                            &format!("Insufficient confidence: {}", time_point.confidence),
                        );
                        continue;
                    }
                    
                    // Check if the time point is verifiable (if required)
                    if config.verify_time_points && !time_point.verified {
                        result = result.add_failed_domain(
                            domain_id.clone(),
                            "Time point not verified",
                        );
                        continue;
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
                        if recent.len() > config.history_size {
                            recent.sort_by(|a, b| a.1.timestamp.cmp(&b.1.timestamp));
                            recent.drain(0..recent.len() - config.history_size);
                        }
                    }
                    
                    // Update the time map
                    updated_map.update_domain(entry);
                    
                    // Mark the domain as synced
                    result = result.add_synced_domain(domain_id);
                }
                Ok(Err(e)) => {
                    result = result.add_failed_domain(domain_id, &format!("Provider error: {}", e));
                }
                Err(_) => {
                    result = result.add_failed_domain(domain_id, "Operation timed out");
                }
            }
        }
        
        // Update the shared time map if we have any successful syncs
        if !result.synced_domains.is_empty() {
            if let Err(e) = time_map.merge(&updated_map) {
                return result.fail(&format!("Failed to update time map: {}", e));
            }
            
            result = result.complete(updated_map);
        } else {
            result = result.skip("No domains were successfully synced");
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
        let max_drift = Duration::seconds(self.config.max_time_difference as i64);
        
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
    use crate::types::BlockHash;
    
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
        let mut result = SyncResult::new(started_at);
        
        // Check initial state
        assert_eq!(result.status, SyncStatus::InProgress);
        assert_eq!(result.synced_count(), 0);
        assert_eq!(result.failed_count(), 0);
        assert!(result.time_map.is_none());
        
        // Add a synced domain
        let domain1 = create_test_domain_id(1);
        result = result.add_synced_domain(domain1.clone());
        
        // Add a failed domain
        let domain2 = create_test_domain_id(2);
        result = result.add_failed_domain(domain2.clone(), "Test failure");
        
        // Check counts
        assert_eq!(result.synced_count(), 1);
        assert_eq!(result.failed_count(), 1);
        
        // Complete the result
        let time_map = TimeMap::new();
        result = result.complete(time_map);
        
        // Check final state
        assert_eq!(result.status, SyncStatus::Completed);
        assert!(result.time_map.is_some());
        assert!(!result.is_fully_successful()); // Has failed domains
    }
    
    #[tokio::test]
    async fn test_time_sync_manager_basic() -> Result<()> {
        // Create a shared time map
        let shared_map = SharedTimeMap::new();
        
        // Create a config with fast sync
        let config = TimeSyncConfig {
            sync_interval: 1,
            sync_timeout: 5,
            ..Default::default()
        };
        
        // Create a sync manager
        let manager = TimeSyncManager::new(config, shared_map);
        
        // Register a provider for a domain
        let domain1 = create_test_domain_id(1);
        
        let provider = TimeSyncManager::create_provider(move |_| {
            Ok(create_test_time_point(100, 1000, 1.0, true))
        });
        
        manager.register_provider(domain1.clone(), provider)?;
        
        // Manually trigger a sync
        let result = manager.sync_now().await?;
        
        // Check the result
        assert_eq!(result.status, SyncStatus::Completed);
        assert_eq!(result.synced_count(), 1);
        assert_eq!(result.failed_count(), 0);
        assert!(result.synced_domains.contains(&domain1));
        
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
            max_time_difference: 100,
            ..Default::default()
        };
        
        // Create a sync manager
        let manager = TimeSyncManager::new(config, shared_map);
        
        // Register providers for two domains
        let domain1 = create_test_domain_id(1);
        let domain2 = create_test_domain_id(2);
        
        let provider1 = TimeSyncManager::create_provider(move |_| {
            Ok(create_test_time_point(100, 1000, 1.0, true))
        });
        
        let provider2 = TimeSyncManager::create_provider(move |_| {
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
        let provider2 = TimeSyncManager::create_provider(move |_| {
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