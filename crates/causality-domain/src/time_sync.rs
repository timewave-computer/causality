// Time synchronization for domains
// Original file: src/domain/time_sync.rs

// Time synchronization system for Causality
//
// This module provides time synchronization management
// across multiple Domains.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time;
use chrono::{DateTime, Utc};
use std::sync::atomic::{AtomicBool, Ordering};
use log::{info, warn, error, debug};
use std::future::Future;

use async_trait::async_trait;

use causality_types::{Error, Result};
use crate::domain::{TimeMapEntry, SharedTimeMap};
use crate::domain::DomainStatus;
use crate::domain::{DomainAdapter, DomainId};
use crate::domain::DomainRegistry;

/// Configuration for time map synchronization
#[derive(Clone)]
pub struct SyncConfig {
    /// Default sync interval in seconds
    pub default_interval: u64,
    /// Domain-specific sync intervals in seconds
    pub domain_intervals: HashMap<DomainId, u64>,
    /// Maximum history size per Domain
    pub max_history_size: usize,
    /// Number of attempts before giving up on a Domain
    pub max_retry_attempts: usize,
    /// Delay between retries in seconds
    pub retry_delay: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        SyncConfig {
            default_interval: 60, // 1 minute default
            domain_intervals: HashMap::new(),
            max_history_size: 100,
            max_retry_attempts: 3,
            retry_delay: 5, // 5 seconds default
        }
    }
}

/// Status of a Domain's synchronization
#[derive(Debug, Clone, PartialEq)]
pub enum DomainSyncStatus {
    /// Domain is being synced successfully
    Active,
    /// Domain sync is temporarily paused
    Paused,
    /// Domain sync has permanently failed
    Failed,
}

/// Status of Domain time sync
#[derive(Debug, Clone)]
pub struct DomainSyncInfo {
    /// Domain ID
    pub domain_id: DomainId,
    /// Current sync status
    pub status: DomainSyncStatus,
    /// Last successful sync time
    pub last_sync: Option<DateTime<Utc>>,
    /// Last sync error, if any
    pub last_error: Option<String>,
    /// Current retry count
    pub retry_count: usize,
    /// Sync interval in seconds
    pub sync_interval: u64,
    /// Consecutive failures count
    pub consecutive_failures: usize,
}

/// Time map synchronization manager
pub struct TimeSyncManager {
    /// Configuration
    config: SyncConfig,
    /// Time map to synchronize
    time_map: SharedTimeMap,
    /// Domain adapters
    domain_registry: Arc<DomainRegistry>,
    /// Control channel
    control_tx: Option<mpsc::Sender<TimeSyncControl>>,
    /// Status tracking
    sync_status: Arc<RwLock<HashMap<DomainId, DomainSyncInfo>>>,
    /// Running background tasks
    tasks: HashMap<DomainId, JoinHandle<()>>,
    /// Running flag
    running: AtomicBool,
}

/// Control messages for the sync manager
#[derive(Debug)]
enum TimeSyncControl {
    /// Stop sync for a Domain
    StopDomain(DomainId),
    /// Start sync for a Domain
    StartDomain(DomainId),
    /// Update sync interval for a Domain
    UpdateInterval(DomainId, u64),
    /// Force sync for a Domain
    ForceSync(Vec<DomainId>),
    /// Shutdown the manager
    Shutdown,
    /// Manual sync request
    SyncNow(DomainId),
    /// Time updated notification
    TimeUpdated(Vec<TimeMapEntry>),
    /// Get status for all domains
    GetStatus(Sender<HashMap<DomainId, DomainSyncInfo>>),
}

impl TimeSyncManager {
    /// Create a new time sync manager
    pub fn new(
        config: SyncConfig, 
        time_map: SharedTimeMap,
        domain_registry: Arc<DomainRegistry>,
    ) -> Self {
        TimeSyncManager {
            config,
            time_map,
            domain_registry,
            control_tx: Some(mpsc::channel(100).0),
            sync_status: Arc::new(RwLock::new(HashMap::new())),
            tasks: HashMap::new(),
            running: AtomicBool::new(true),
        }
    }
    
    /// Start synchronization for all Domains in the registry
    pub async fn start_all(&mut self) -> Result<()> {
        // Get all Domains from registry
        let domains = match self.domain_registry.list_domains() {
            Ok(domains) => domains,
            Err(e) => return Err(e),
        };
        
        // Start sync for each Domain
        for domain_id in domains {
            self.start_domain_sync(&domain_id)?;
        }
        
        Ok(())
    }
    
    /// Start synchronization for a specific Domain
    pub fn start_domain_sync(&mut self, domain_id: &DomainId) -> Result<()> {
        // Check if sync is already running for this Domain
        if self.tasks.contains_key(domain_id) {
            return Ok(());
        }
        
        // Get the Domain adapter
        let adapter = self.domain_registry.get_domain(domain_id)
            .ok_or_else(|| Error::DomainNotFound(domain_id.clone()))?;
        
        // Get sync interval for this Domain
        let interval = self.config.domain_intervals
            .get(domain_id)
            .copied()
            .unwrap_or(self.config.default_interval);
        
        // Create initial sync info
        let sync_info = DomainSyncInfo {
            domain_id: domain_id.clone(),
            status: DomainSyncStatus::Active,
            last_sync: None,
            last_error: None,
            retry_count: 0,
            sync_interval: interval,
            consecutive_failures: 0,
        };
        
        // Update status
        {
            let mut status_map = self.sync_status.write().unwrap();
            status_map.insert(domain_id.clone(), sync_info);
        }
        
        // Start sync task for this Domain
        let task = self.spawn_domain_sync_task(adapter, interval, domain_id.clone());
        self.tasks.insert(domain_id.clone(), task);
        
        Ok(())
    }
    
    /// Stop synchronization for a specific Domain
    pub async fn stop_domain_sync(&mut self, domain_id: &DomainId) -> Result<()> {
        // Send stop message
        if let Some(tx) = &self.control_tx {
            tx.send(TimeSyncControl::StopDomain(domain_id.clone())).await
                .map_err(|_| Error::SyncManagerError("Failed to send control message".to_string()))?;
        }
        
        // Update status
        {
            let mut status_map = self.sync_status.write().unwrap();
            if let Some(info) = status_map.get_mut(domain_id) {
                info.status = DomainSyncStatus::Paused;
            }
        }
        
        // Remove task
        if let Some(handle) = self.tasks.remove(domain_id) {
            // Attempt to abort the task, but ignore errors since it may have already completed
            handle.abort();
        }
        
        Ok(())
    }
    
    /// Update sync interval for a specific Domain
    pub async fn update_domain_interval(&mut self, domain_id: &DomainId, interval: u64) -> Result<()> {
        // Send update interval message
        if let Some(tx) = &self.control_tx {
            tx.send(TimeSyncControl::UpdateInterval(domain_id.clone(), interval)).await
                .map_err(|_| Error::SyncManagerError("Failed to send control message".to_string()))?;
        }
        
        // Update status
        {
            let mut status_map = self.sync_status.write().unwrap();
            if let Some(info) = status_map.get_mut(domain_id) {
                // Reset retry count when interval is updated
                info.retry_count = 0;
            }
        }
        
        // Update config
        self.config.domain_intervals.insert(domain_id.clone(), interval);
        
        Ok(())
    }
    
    /// Force synchronization for a specific Domain
    pub async fn force_domain_sync(&self, domain_id: &DomainId) -> Result<()> {
        // Send force sync message
        if let Some(tx) = &self.control_tx {
            tx.send(TimeSyncControl::ForceSync(vec![domain_id.clone()])).await
                .map_err(|_| Error::SyncManagerError("Failed to send control message".to_string()))?;
        }
        
        Ok(())
    }
    
    /// Get current status for all Domains
    pub fn get_all_domain_status(&self) -> Result<HashMap<DomainId, DomainSyncInfo>> {
        let status_map = self.sync_status.read()
            .map_err(|_| Error::LockError("Failed to acquire read lock on sync status".to_string()))?;
        
        Ok(status_map.clone())
    }
    
    /// Get status for a specific Domain
    pub fn get_domain_status(&self, domain_id: &DomainId) -> Result<Option<DomainSyncInfo>> {
        let status_map = self.sync_status.read()
            .map_err(|_| Error::LockError("Failed to acquire read lock on sync status".to_string()))?;
        
        Ok(status_map.get(domain_id).cloned())
    }
    
    /// Start the control loop
    fn start_control_loop(&self, mut rx: Receiver<TimeSyncControl>) -> JoinHandle<()> {
        // Create clones of the required fields to avoid capturing self
        let domain_registry = Arc::clone(&self.domain_registry);
        // Explicitly clone as Arc<SharedTimeMap>
        let timemap = self.time_map.clone();
        let domain_sync_state = Arc::clone(&self.sync_status);
        let config = self.config.clone();
        
        tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    TimeSyncControl::Shutdown => {
                        break;
                    },
                    TimeSyncControl::StartDomain(domain_id) => {
                        let domain = match domain_registry.get_domain(&domain_id) {
                            Some(adapter) => adapter,
                            None => {
                                error!("Domain not found: {:?}", domain_id);
                                continue;
                            }
                        };
                        
                        // Get interval for this domain
                        let interval = config.domain_intervals
                            .get(&domain_id)
                            .copied()
                            .unwrap_or(config.default_interval);
                        
                        // Update sync state
                        if let Ok(mut state) = domain_sync_state.write() {
                            if let Some(info) = state.get_mut(&domain_id) {
                                info.status = DomainSyncStatus::Active;
                                info.last_sync = Some(Utc::now());
                                info.consecutive_failures = 0;
                                info.last_error = None;
                            } else {
                                let sync_info = DomainSyncInfo {
                                    domain_id: domain_id.clone(),
                                    status: DomainSyncStatus::Active,
                                    last_sync: Some(Utc::now()),
                                    consecutive_failures: 0,
                                    last_error: None,
                                    sync_interval: interval,
                                    retry_count: 0,
                                };
                                state.insert(domain_id.clone(), sync_info);
                            }
                        }
                    },
                    TimeSyncControl::StopDomain(_domain_id) => {
                        // Implementation for stopping domain syncs would go here
                    },
                    TimeSyncControl::SyncNow(domain_id) => {
                        // Handle a single domain for SyncNow
                        if let Some(adapter) = domain_registry.get_domain(&domain_id) {
                            // Perform synchronization
                            match perform_domain_sync(adapter, &timemap).await {
                                Ok(entry) => {
                                    // Update sync state on success
                                    if let Ok(mut state) = domain_sync_state.write() {
                                        if let Some(info) = state.get_mut(&domain_id) {
                                            info.status = DomainSyncStatus::Active;
                                            info.last_sync = Some(entry.observed_at);
                                            info.retry_count = 0;
                                            info.consecutive_failures = 0;
                                            info.last_error = None;
                                        }
                                    }
                                },
                                Err(e) => {
                                    error!("Failed to sync domain {:?}: {}", domain_id, e);
                                    
                                    // Update retry count
                                    if let Ok(mut state) = domain_sync_state.write() {
                                        if let Some(info) = state.get_mut(&domain_id) {
                                            info.retry_count += 1;
                                            info.consecutive_failures += 1;
                                            info.last_error = Some(e.to_string());
                                            
                                            // After too many retries, pause syncing
                                            if info.retry_count > config.max_retry_attempts {
                                                info.status = DomainSyncStatus::Failed;
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            error!("Domain not found: {:?}", domain_id);
                        }
                    },
                    TimeSyncControl::ForceSync(domain_ids) => {
                        // Handle a Vec<DomainId> for ForceSync
                        for domain_id in domain_ids {
                            if let Some(adapter) = domain_registry.get_domain(&domain_id) {
                                if let Err(e) = perform_domain_sync(adapter, &timemap).await {
                                    error!("Failed to force sync domain {:?}: {}", domain_id, e);
                                }
                            } else {
                                error!("Domain not found for force sync: {:?}", domain_id);
                            }
                        }
                    },
                    TimeSyncControl::UpdateInterval(_domain_id, _interval) => {
                        // Implementation for updating sync intervals would go here
                    },
                    TimeSyncControl::GetStatus(tx) => {
                        if let Ok(state) = domain_sync_state.read() {
                            let _ = tx.send(state.clone());
                        }
                    },
                    TimeSyncControl::TimeUpdated(_) => {
                        // This is used for notifications, no action needed here
                    },
                }
            }
        })
    }
    
    /// Spawn a domain sync task
    fn spawn_domain_sync_task(
        &self,
        adapter: Arc<dyn DomainAdapter>,
        interval: u64,
        domain_id: DomainId,
    ) -> JoinHandle<()> {
        let sync_status = self.sync_status.clone();
        let time_map = self.time_map.clone();
        let retry_attempts = self.config.max_retry_attempts;
        let retry_delay = self.config.retry_delay;
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(interval));
            
            loop {
                interval.tick().await;
                
                // Check if we should still sync this domain
                let should_sync = {
                    if let Ok(status_map) = sync_status.read() {
                        match status_map.get(&domain_id) {
                            Some(info) => info.status == DomainSyncStatus::Active,
                            None => false,
                        }
                    } else {
                        false
                    }
                };
                
                if !should_sync {
                    break;
                }
                
                // Perform sync
                match perform_domain_sync(adapter.clone(), &time_map).await {
                    Ok(entry) => {
                        // Update status on success
                        if let Ok(mut status_map) = sync_status.write() {
                            if let Some(info) = status_map.get_mut(&domain_id) {
                                info.status = DomainSyncStatus::Active;
                                info.last_sync = Some(entry.observed_at);
                                info.consecutive_failures = 0;
                                info.last_error = None;
                            }
                        }
                    }
                    Err(e) => {
                        // Update status on failure
                        let should_retry = {
                            if let Ok(mut status_map) = sync_status.write() {
                                if let Some(info) = status_map.get_mut(&domain_id) {
                                    info.consecutive_failures += 1;
                                    info.last_error = Some(e.to_string());
                                    
                                    // Check if we should mark as failed
                                    if info.consecutive_failures >= retry_attempts {
                                        info.status = DomainSyncStatus::Failed;
                                        false
                                    } else {
                                        true
                                    }
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        };
                        
                        // Retry if needed
                        if should_retry {
                            time::sleep(Duration::from_secs(retry_delay)).await;
                        } else {
                            break;
                        }
                    }
                }
            }
        })
    }
    
    /// Synchronize a specific domain's time
    async fn sync_domain_time(&self, adapter: Arc<dyn DomainAdapter>) -> Result<TimeMapEntry> {
        let entry = perform_domain_sync(adapter, &self.time_map).await?;
        
        // Update status
        let domain_id = entry.domain_id.clone();
        {
            let mut status_map = self.sync_status.write().map_err(|_| {
                Error::LockError("Failed to acquire write lock on sync status".to_string())
            })?;
            
            if let Some(info) = status_map.get_mut(&domain_id) {
                info.last_sync = Some(Utc::now());
                info.retry_count = 0;
                info.consecutive_failures = 0;
                info.last_error = None;
            }
        }
        
        Ok(entry)
    }
    
    /// Stop all domain synchronization
    pub async fn stop_all(&mut self) -> Result<()> {
        // Get all running domains
        let domain_ids: Vec<DomainId> = {
            let status_map = self.sync_status.read().map_err(|_| {
                Error::LockError("Failed to acquire read lock on sync status".to_string())
            })?;
            status_map.keys().cloned().collect()
        };
        
        // Stop each domain
        for domain_id in domain_ids {
            self.stop_domain_sync(&domain_id).await?;
        }
        
        // Shutdown control loop if needed
        if let Some(tx) = &self.control_tx {
            let _ = tx.send(TimeSyncControl::Shutdown).await;
        }
        
        Ok(())
    }
}

/// Perform synchronization for a single domain
pub async fn perform_domain_sync(
    adapter: Arc<dyn DomainAdapter>,
    time_map: &SharedTimeMap,
) -> Result<TimeMapEntry> {
    // Get current time map from the domain
    let entry = adapter.get_time_map().await?;
    
    // Update shared time map
    time_map.update_entry(entry.clone()).await?;
    
    Ok(entry)
}

/// Periodically sync time maps for a given list of domains
pub async fn sync_domains(
    domains: &[Arc<dyn DomainAdapter>],
    time_map: &SharedTimeMap,
) -> Result<Vec<Result<TimeMapEntry>>> {
    let mut results = Vec::new();
    
    for domain in domains {
        let result = perform_domain_sync(domain.clone(), time_map).await;
        results.push(result);
    }
    
    Ok(results)
}

/// Utility for monitoring domain availability and sync status
pub struct DomainMonitor {
    /// Sync manager
    sync_manager: Arc<RwLock<TimeSyncManager>>,
    /// Last check times
    last_checks: HashMap<DomainId, Instant>,
    /// Check interval in seconds
    check_interval: u64,
}

impl DomainMonitor {
    /// Create a new domain monitor
    pub fn new(sync_manager: Arc<RwLock<TimeSyncManager>>, check_interval: u64) -> Self {
        DomainMonitor {
            sync_manager,
            last_checks: HashMap::new(),
            check_interval,
        }
    }
    
    /// Start monitoring domains
    pub fn start(&self) -> tokio::task::JoinHandle<()> {
        let sync_manager = Arc::clone(&self.sync_manager);
        let check_interval = self.check_interval;
        let mut last_checks = self.last_checks.clone();
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(check_interval));
            
            loop {
                interval.tick().await;
                
                // Collect everything we need while holding the read lock
                let (domain_status, domain_registry, control_tx) = {
                    let guard = match sync_manager.read() {
                        Ok(guard) => guard,
                        Err(_) => {
                            error!("Failed to acquire read lock on sync manager");
                            continue;
                        }
                    };
                    
                    let status = match guard.get_all_domain_status() {
                        Ok(status) => status,
                        Err(e) => {
                            error!("Failed to get domain status: {}", e);
                            continue;
                        }
                    };
                    
                    let registry = Arc::clone(&guard.domain_registry);
                    let tx = guard.control_tx.clone();
                    
                    (status, registry, tx)
                };
                
                let now = Instant::now();
                
                for (domain_id, info) in domain_status {
                    // Check if it's time to check this domain
                    let should_check = match last_checks.get(&domain_id) {
                        Some(last_check) => {
                            now.duration_since(*last_check).as_secs() >= check_interval
                        },
                        None => true,
                    };
                    
                    if !should_check {
                        continue;
                    }
                    
                    // Get domain adapter
                    if let Some(adapter) = domain_registry.get_domain(&domain_id) {
                        // Check connectivity
                        let connected = match adapter.check_connectivity().await {
                            Ok(c) => c,
                            Err(e) => {
                                error!("Failed to check connectivity for domain {:?}: {}", domain_id, e);
                                false
                            }
                        };
                        
                        // Update last check time
                        last_checks.insert(domain_id.clone(), now);
                        
                        // If not connected and currently active, update status
                        if !connected && info.status == DomainSyncStatus::Active {
                            if let Some(tx) = &control_tx {
                                let _ = tx.send(TimeSyncControl::StopDomain(domain_id.clone())).await;
                            }
                        }
                        
                        // If connected but currently paused, start sync again
                        if connected && info.status == DomainSyncStatus::Paused {
                            if let Some(tx) = &control_tx {
                                let _ = tx.send(TimeSyncControl::StartDomain(domain_id.clone())).await;
                            }
                        }
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use async_trait::async_trait;
    use crate::domain::{DomainType, DomainInfo, DomainStatus, FactQuery};
    use causality_types::{BlockHash, BlockHeight, Timestamp};
    use crate::domain::{DomainAdapter, DomainId, Transaction, TransactionId, TransactionReceipt};
    
    /// Mock domain adapter for testing
    #[derive(Debug)]
    struct MockDomainAdapter {
        domain_id: DomainId,
        time_map_entry: Arc<Mutex<TimeMapEntry>>,
        connectivity: Arc<Mutex<bool>>,
    }
    
    #[async_trait]
    impl DomainAdapter for MockDomainAdapter {
        fn domain_id(&self) -> &DomainId {
            &self.domain_id
        }
        
        async fn domain_info(&self) -> Result<DomainInfo> {
            Ok(DomainInfo {
                id: self.domain_id.clone(),
                domain_type: DomainType::Custom("mock".to_string()),
                name: format!("Mock Domain {}", self.domain_id),
                description: Some("Mock domain for testing".to_string()),
                rpc_url: None,
                explorer_url: None,
                chain_id: None,
                native_currency: None,
                status: DomainStatus::Active,
                metadata: HashMap::new(),
            })
        }
        
        async fn current_height(&self) -> Result<BlockHeight> {
            let entry = self.time_map_entry.lock().unwrap();
            Ok(entry.height.clone())
        }
        
        async fn current_hash(&self) -> Result<BlockHash> {
            let entry = self.time_map_entry.lock().unwrap();
            Ok(entry.hash.clone())
        }
        
        async fn current_timestamp(&self) -> Result<Timestamp> {
            let entry = self.time_map_entry.lock().unwrap();
            Ok(entry.timestamp.clone())
        }
        
        async fn observe_fact(&self, _query: FactQuery) -> Result<FactType> {
            Err(Error::NotImplemented("observe_fact not implemented in mock".to_string()))
        }
        
        async fn submit_transaction(&self, _tx: Transaction) -> Result<TransactionId> {
            Err(Error::NotImplemented("submit_transaction not implemented in mock".to_string()))
        }
        
        async fn get_transaction_receipt(&self, _tx_id: &TransactionId) -> Result<TransactionReceipt> {
            Err(Error::NotImplemented("get_transaction_receipt not implemented in mock".to_string()))
        }
        
        async fn get_time_map(&self) -> Result<TimeMapEntry> {
            let entry = self.time_map_entry.lock().unwrap();
            Ok(entry.clone())
        }
        
        async fn verify_block(&self, _height: BlockHeight, _hash: &BlockHash) -> Result<bool> {
            Ok(true)
        }
        
        async fn check_connectivity(&self) -> Result<bool> {
            let connected = self.connectivity.lock().unwrap();
            Ok(*connected)
        }
    }
    
    impl MockDomainAdapter {
        fn new(domain_id: DomainId, height: u64, hash: Vec<u8>, timestamp: u64) -> Self {
            let entry = TimeMapEntry {
                domain_id: domain_id.clone(),
                height: BlockHeight::new(height),
                hash: BlockHash::new(hash),
                timestamp: Timestamp::new(timestamp),
                observed_at: chrono::Utc::now(),
                confidence: 1.0,
                verified: true,
                source: "mock".to_string(),
                metadata: HashMap::new(),
            };
            
            MockDomainAdapter {
                domain_id,
                time_map_entry: Arc::new(Mutex::new(entry)),
                connectivity: Arc::new(Mutex::new(true)),
            }
        }
        
        fn set_connectivity(&self, connected: bool) {
            let mut connectivity = self.connectivity.lock().unwrap();
            *connectivity = connected;
        }
        
        fn update_height(&self, height: u64) {
            let mut entry = self.time_map_entry.lock().unwrap();
            entry.height = BlockHeight::new(height);
        }
    }
    
    /// Create test domains with string IDs
    #[tokio::test]
    async fn test_time_sync_manager() {
        // Create mock domains
        let domain1 = DomainId("domain1".to_string());
        let domain2 = DomainId("domain2".to_string());
        
        let adapter1 = Arc::new(MockDomainAdapter::new(
            domain1.clone(),
            100,
            vec![1, 2, 3, 4],
            1000,
        ));
        
        let adapter2 = Arc::new(MockDomainAdapter::new(
            domain2.clone(),
            200,
            vec![5, 6, 7, 8],
            2000,
        ));
        
        // Create domain registry
        let mut registry = DomainRegistry::new();
        registry.register_domain(adapter1.clone());
        registry.register_domain(adapter2.clone());
        
        let registry_arc = Arc::new(registry);
        
        // Create shared time map
        let time_map = SharedTimeMap::new();
        
        // Create sync manager with fast intervals for testing
        let config = SyncConfig {
            default_interval: 1,
            domain_intervals: {
                let mut intervals = HashMap::new();
                intervals.insert(domain1.clone(), 1);
                intervals.insert(domain2.clone(), 2);
                intervals
            },
            max_history_size: 10,
            max_retry_attempts: 3,
            retry_delay: 5,
        };
        
        let mut sync_manager = TimeSyncManager::new(
            time_map.clone(),
            registry_arc.clone(),
            config,
        );
        
        // Start sync
        sync_manager.start_all().await.unwrap();
        
        // Wait for initial sync
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Check that time map has entries
        let entries = time_map.get_all_entries().await.unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.contains_key(&domain1));
        assert!(entries.contains_key(&domain2));
        
        // Check sync status
        let status = sync_manager.get_all_domain_status().unwrap();
        assert_eq!(status.len(), 2);
        assert_eq!(status.get(&domain1).unwrap().status, DomainSyncStatus::Active);
        assert_eq!(status.get(&domain2).unwrap().status, DomainSyncStatus::Active);
        
        // Update domain1 height and force sync
        adapter1.update_height(150);
        sync_manager.force_domain_sync(&domain1).await.unwrap();
        
        // Wait for sync to complete
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Check that time map has updated
        let entries = time_map.get_all_entries().await.unwrap();
        assert_eq!(entries.get(&domain1).unwrap().height, 150);
        
        // Test disconnection
        adapter2.set_connectivity(false);
        
        // Wait for sync to retry and fail
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        // Check that domain2 is now failed
        let status = sync_manager.get_all_domain_status().unwrap();
        assert_eq!(status.get(&domain2).unwrap().status, DomainSyncStatus::Failed);
        
        // Reconnect domain2
        adapter2.set_connectivity(true);
        
        // Manually restart sync for domain2
        if let Some(tx) = &sync_manager.control_tx {
            tx.send(TimeSyncControl::StartDomain(domain2.clone())).await.unwrap();
        }
        
        // Wait for sync to restart
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Check that domain2 is active again
        let status = sync_manager.get_all_domain_status().unwrap();
        assert_eq!(status.get(&domain2).unwrap().status, DomainSyncStatus::Active);
        
        // Stop sync
        sync_manager.stop_all().await.unwrap();
    }
    
    /// Create test domains with string IDs
    #[tokio::test]
    async fn test_domain_monitor() {
        // Create mock domains
        let domain1 = DomainId("domain1".to_string());
        let domain2 = DomainId("domain2".to_string());
        
        let adapter1 = Arc::new(MockDomainAdapter::new(
            domain1.clone(),
            100,
            vec![1, 2, 3, 4],
            1000,
        ));
        
        let adapter2 = Arc::new(MockDomainAdapter::new(
            domain2.clone(),
            200,
            vec![5, 6, 7, 8],
            2000,
        ));
        
        // Set domain2 as disconnected
        adapter2.set_connectivity(false);
        
        // Create domain registry
        let mut registry = DomainRegistry::new();
        registry.register_domain(adapter1.clone());
        registry.register_domain(adapter2.clone());
        
        let registry_arc = Arc::new(registry);
        
        // Create shared time map
        let time_map = SharedTimeMap::new();
        
        // Create sync manager
        let config = SyncConfig {
            default_interval: 60,
            domain_intervals: HashMap::new(),
            max_history_size: 10,
            max_retry_attempts: 3,
            retry_delay: 5,
        };
        
        let sync_manager = TimeSyncManager::new(
            time_map.clone(),
            registry_arc.clone(),
            config,
        );
        
        let sync_manager_arc = Arc::new(RwLock::new(sync_manager));
        
        // Create domain monitor
        let mut monitor = DomainMonitor::new(sync_manager_arc.clone(), 1);
        
        // Check all domains
        let results = monitor.start().await.unwrap();
        
        // Verify results
        assert_eq!(results.len(), 2);
        assert_eq!(*results.get(&domain1).unwrap(), true);
        assert_eq!(*results.get(&domain2).unwrap(), false);
    }
} 