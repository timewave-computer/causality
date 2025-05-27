// Fact Replay Engine for Causality
//
// This module provides mechanisms for replaying facts in chronological
// order to reconstruct state and verify operation correctness.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use causality_types::{DomainId, ContentId, Timestamp}; // Import Timestamp
use causality_error::{EngineError, Result};
use crate::log::{FactEntry, LogStorage, EntryData, LogEntry}; // Import EntryData and LogEntry
use crate::log::fact_types::{FactType, RegisterFact};
use crate::log::fact::{FactId, FactSnapshot};
use std::time::Duration;
use async_trait::async_trait; // Import async_trait
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver}; // Import channels
use tokio::sync::Mutex as TokioMutex; // Import tokio Mutex
use tracing::{info, error, debug, trace}; // Import tracing macros

#[cfg(feature = "md5")]
use causality_crypto::md5::Md5ChecksumFunction;
#[cfg(not(feature = "md5"))]
use causality_crypto::hash::HashFactory;

/// Status of a fact replay operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FactReplayStatus {
    /// The replay is running
    Running,
    /// The replay is paused
    Paused,
    /// The replay is completed
    Completed,
    /// The replay has failed
    Failed(String),
}

/// Configuration for fact replay
#[derive(Debug, Clone)]
pub struct FactReplayConfig {
    /// Whether to verify facts during replay
    pub verify_facts: bool,
    /// Whether to apply register state updates
    pub apply_register_updates: bool,
    /// Whether to stop on errors
    pub stop_on_error: bool,
    /// Maximum number of facts to replay
    pub max_facts: Option<usize>,
    /// Filter by domain IDs (empty means all domains)
    pub domain_filter: HashSet<DomainId>,
    /// Filter by resource IDs (empty means all resources)
    pub resource_filter: HashSet<ContentId>,
    /// Domains to replay facts for
    pub domains: HashSet<DomainId>,
    /// Resources to filter facts by
    pub resources: Option<HashSet<ContentId>>,
    /// Start timestamp for replay (optional)
    pub start_time: Option<Timestamp>, // Use imported Timestamp
    /// End timestamp for replay (optional)
    pub end_time: Option<Timestamp>, // Use imported Timestamp
    /// Batch size for reading log entries
    pub batch_size: usize,
    /// Interval for polling new entries
    pub poll_interval: Duration,
    /// Callback channel for replayed facts
    pub callback_tx: UnboundedSender<(FactId, FactEntry, Timestamp)>, // Use imported types
}

impl Default for FactReplayConfig {
    fn default() -> Self {
        FactReplayConfig {
            verify_facts: true,
            apply_register_updates: true,
            stop_on_error: true,
            max_facts: None,
            domain_filter: HashSet::new(),
            resource_filter: HashSet::new(),
            domains: HashSet::new(),
            resources: None,
            start_time: None,
            end_time: None,
            batch_size: 100,
            poll_interval: Duration::from_secs(1),
            callback_tx: tokio::sync::mpsc::unbounded_channel().0,
        }
    }
}

/// Engine for replaying facts to reconstruct state
pub struct FactReplayEngine {
    /// The storage to read facts from
    storage: Arc<Mutex<dyn LogStorage + Send>>,
    /// Configuration for replay
    config: FactReplayConfig,
    /// Current status of replay
    status: FactReplayStatus,
    /// Map of register IDs to their current state
    register_states: HashMap<ContentId, Vec<u8>>,
    /// Map of fact IDs to their entries
    fact_cache: HashMap<FactId, FactEntry>,
    /// Callbacks for fact replay events (Using Arc for shared ownership)
    callbacks: Vec<Arc<dyn FactReplayCallback>>,
    /// Domain ID
    domain_id: DomainId,
}

impl std::fmt::Debug for FactReplayEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FactReplayEngine")
            .field("config", &self.config)
            .field("status", &self.status)
            .field("register_states_count", &self.register_states.len())
            .field("fact_cache_count", &self.fact_cache.len())
            .field("callbacks_count", &self.callbacks.len())
            .field("domain_id", &self.domain_id)
            .finish()
    }
}

impl FactReplayEngine {
    /// Create a new fact replay engine
    pub fn new(
        storage: Arc<Mutex<dyn LogStorage + Send>>,
        config: FactReplayConfig,
        domain_id: DomainId,
    ) -> Self {
        FactReplayEngine {
            storage,
            config,
            status: FactReplayStatus::Paused,
            register_states: HashMap::new(),
            fact_cache: HashMap::new(),
            callbacks: Vec::new(),
            domain_id,
        }
    }
    
    /// Add a callback for fact replay events
    pub fn add_callback(&mut self, callback: Arc<dyn FactReplayCallback>) {
        self.callbacks.push(callback);
    }
    
    /// Start or resume the replay
    pub async fn start(&mut self) -> Result<()> {
        self.status = FactReplayStatus::Running;
        self.replay_facts().await
    }
    
    /// Pause the replay
    pub fn pause(&mut self) {
        if self.status == FactReplayStatus::Running {
            self.status = FactReplayStatus::Paused;
        }
    }
    
    /// Get the current status
    pub fn status(&self) -> &FactReplayStatus {
        &self.status
    }
    
    /// Replay facts from storage
    async fn replay_facts(&mut self) -> Result<()> {
        // First, collect all the *entries* we need to process
        let entries_to_process = {
            let storage = self.storage.lock()
                .map_err(|_| EngineError::LogError("Failed to lock storage".to_string()))?;
            
            let entry_count = storage.entry_count()?;
            storage.read(0, entry_count)? // Read all entries
        };
        
        // Now process the log entries, extracting facts
        let mut facts_processed = 0;
        // Iterate over entries, not just facts
        for entry in entries_to_process { 
            // Check if it's a Fact entry
            if let EntryData::Fact(fact) = &entry.data { 
                 // Check if we should stop
                if let Some(max_facts) = self.config.max_facts {
                    if facts_processed >= max_facts {
                        self.status = FactReplayStatus::Completed;
                        return Ok(());
                    }
                }
                
                // Process the fact
                if let Err(e) = self.process_fact(fact) {
                    if self.config.stop_on_error {
                        self.status = FactReplayStatus::Failed(e.to_string());
                        return Err(e);
                    }
                }
                
                // Cache the fact
                let fact_id = self.build_fact_id(fact);
                self.fact_cache.insert(fact_id.clone(), fact.clone());
                
                // Invoke callbacks asynchronously, passing the entry's timestamp
                for callback in &self.callbacks {
                    // Clone Arc for the async call
                    let cb = callback.clone(); 
                     // Use entry.timestamp
                    if let Err(e) = cb.on_fact_replayed(fact_id.clone(), fact.clone(), entry.timestamp).await {
                        if self.config.stop_on_error {
                            self.status = FactReplayStatus::Failed(e.to_string());
                            return Err(e);
                        }
                        // TODO: Log non-fatal callback error?
                    }
                }
                
                facts_processed += 1;
            }
            
            // Check if we've been paused (after processing each entry)
            if self.status == FactReplayStatus::Paused {
                return Ok(());
            }
        }
        
        self.status = FactReplayStatus::Completed;
        Ok(())
    }
    
    /// Process a single fact
    fn process_fact(&mut self, fact: &FactEntry) -> Result<()> {
        // Placeholder for fact verification logic
        if self.config.verify_facts {
            // In a real implementation, this would verify the fact's proof
        }
        
        // Handle register-related facts if configured
        if self.config.apply_register_updates {
            match self.extract_fact_type(fact) {
                Some(FactType::RegisterState) => {
                    if let Some(register_fact) = self.extract_register_fact(fact) {
                        self.apply_register_fact(register_fact)?;
                    }
                }
                _ => {},
            }
        }
        
        Ok(())
    }
    
    /// Extract the fact type from a fact entry
    fn extract_fact_type(&self, _fact: &FactEntry) -> Option<FactType> {
        // In a real implementation, this would deserialize the fact data
        // and extract the fact type
        Some(FactType::RegisterState) // For testing purposes
    }
    
    /// Extract a RegisterFact from a FactEntry
    fn extract_register_fact(&self, _fact: &FactEntry) -> Option<RegisterFact> {
        // In a real implementation, this would deserialize the fact data
        // For now, we create a dummy RegisterFact
        Some(RegisterFact {
            register_id: ContentId::new("dummy_register"),
            state: Vec::new(),
            version: 1,
            previous_hash: None,
        })
    }
    
    /// Apply a register fact to update register state
    fn apply_register_fact(&mut self, register_fact: RegisterFact) -> Result<()> {
        // Update the register state with the provided fact data
        self.register_states.insert(register_fact.register_id, register_fact.state);
        Ok(())
    }
    
    /// Get the current state of a register
    pub fn get_register_state(&self, register_id: &ContentId) -> Option<&Vec<u8>> {
        self.register_states.get(register_id)
    }
    
    /// Check if a fact has been observed in the replay
    pub fn has_fact(&self, fact_id: &FactId) -> bool {
        self.fact_cache.contains_key(fact_id)
    }
    
    /// Get a fact from the cache
    pub fn get_fact(&self, fact_id: &FactId) -> Option<&FactEntry> {
        self.fact_cache.get(fact_id)
    }
    
    /// Create a fact snapshot from the current state
    pub fn create_snapshot(&self, observer: &str) -> FactSnapshot {
        let mut snapshot = FactSnapshot::new(observer);
        
        // Add all facts in the cache
        for (fact_id, fact_entry) in &self.fact_cache {
            snapshot.add_fact(fact_id.clone(), fact_entry.domain_id.clone());
        }
        
        // Add all register states
        for (register_id, state) in &self.register_states {
            let _fact_id = FactId(format!("register:{}", register_id));
            
            // Create a fake domain ID for simplicity
            let _domain_id = DomainId::new("fact-replay");
            
            // Create a data hash from the state
            #[cfg(feature = "md5")]
            let _data_hash = Md5ChecksumFunction::compute(state).to_hex();
            
            #[cfg(not(feature = "md5"))]
            let _data_hash = {
                let hash_factory = HashFactory::default();
                let mut hasher = hash_factory.default_content_hasher().unwrap();
                hasher.update(state);
                hasher.finalize().to_hex()
            };
            
            snapshot.add_register_observation(
                register_id.to_string().as_str(),
                FactId(format!("register:{}", register_id)),
                self.domain_id.clone(),
                0, // Value as a placeholder
            );
        }
        
        snapshot
    }
    
    /// Build a unique ID for this fact
    fn build_fact_id(&self, fact: &FactEntry) -> FactId {
        // Format fact ID as "domain:fact_id" as resources/type/timestamp are gone
        FactId(format!("{}:{}", fact.domain_id, fact.fact_id))
        /*
        // Format fact ID as "type:resource_id" if resources are available
        if let Some(resources) = &fact.resources {
            if let Some(first_resource) = resources.first() {
                return FactId(format!("{}:{}", fact.fact_type, first_resource));
            }
        }
        
        // Fallback to just using the fact type with a timestamp
        FactId(format!("{}:{}", fact.fact_type, fact.timestamp))
        */
    }
    
    /// Calculate a hash for a register using the given hash factory
    fn hash_register(&self, hash_factory: &HashFactory, register_id: &ContentId, data: &[u8]) -> String {
        // Use the hash factory to create a content hasher with the default algorithm
        let hasher = hash_factory.default_content_hasher().unwrap();
        
        // Update the hasher with register ID and data
        let mut hasher_box = hasher;
        hasher_box.update(register_id.to_string().as_bytes());
        hasher_box.update(data);
        
        // Finalize and return the hash
        hasher_box.finalize().to_string()
    }
}

/// Service that replays facts from the log based on configuration
pub struct FactReplayService<S: LogStorage> {
    config: FactReplayConfig,
    storage: Arc<S>,
    // Use Tokio Mutex for async locking
    processed_offset: Arc<TokioMutex<usize>>,
    active_facts: Arc<TokioMutex<HashMap<FactId, (FactEntry, Timestamp)>>>,
}

#[async_trait]
pub trait FactReplayCallback: Send + Sync {
    async fn on_fact_replayed(&self, fact_id: FactId, fact_entry: FactEntry, timestamp: Timestamp) -> Result<()>; // Use imported Timestamp
}

impl<S: LogStorage + Send + Sync + 'static> FactReplayService<S> {
    pub fn new(config: FactReplayConfig, storage: Arc<S>) -> Self {
        Self {
            config,
            storage,
            // Initialize Tokio Mutex
            processed_offset: Arc::new(TokioMutex::new(0)),
            active_facts: Arc::new(TokioMutex::new(HashMap::new())), 
        }
    }

    pub async fn run(&self, mut stop_rx: UnboundedReceiver<()>) -> Result<()> {
        // Use imported info macro
        info!("Starting fact replay service...");
        let mut interval = tokio::time::interval(self.config.poll_interval);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.process_log_entries().await {
                        // Use imported error macro
                        error!(error = %e, "Error processing log entries");
                        // Decide if error is fatal or recoverable
                        // For now, continue loop
                    }
                }
                _ = stop_rx.recv() => {
                    // Use imported info macro
                    info!("Stop signal received, shutting down fact replay service.");
                    break;
                }
            }
        }
        Ok(())
    }

    async fn process_log_entries(&self) -> Result<()> {
        // Lock Tokio Mutex asynchronously
        let mut current_offset_guard = self.processed_offset.lock().await;
        let current_offset = *current_offset_guard;
         // Use imported debug macro
        debug!(current_offset, batch_size = self.config.batch_size, "Processing log entries batch");

        // Use get_entries (async) instead of read
        let entries = self.storage.get_entries(current_offset, current_offset + self.config.batch_size).await?;
        if entries.is_empty() {
             // Use imported trace macro
            trace!("No new entries to process.");
            return Ok(());
        }

        let num_entries = entries.len();
         // Use imported trace macro
        trace!(num_entries, "Read entries from log");

        let mut new_facts = Vec::new();
        for entry in entries {
            // Basic time filtering
            if let Some(start) = self.config.start_time {
                if entry.timestamp < start {
                    continue;
                }
            }
            if let Some(end) = self.config.end_time {
                if entry.timestamp > end {
                    continue;
                }
            }

            // Use imported EntryData
            if let EntryData::Fact(fact) = entry.data {
                // Domain filtering
                if !self.config.domains.contains(&fact.domain_id) {
                    continue;
                }
                
                // Resource filtering
                // TODO: Revisit resource filtering logic
                /*
                let include_by_resource = self.config.resources.as_ref().map_or(true, |filter_resources| {
                    fact.resources.as_ref().map_or(false, |fact_resources| {
                        fact_resources.iter().any(|r| filter_resources.contains(r))
                    })
                });
                if !include_by_resource {
                    continue;
                }
                */

                // If filters pass, add fact
                let fact_id = self.determine_fact_id(&fact);
                new_facts.push((fact_id, fact, entry.timestamp));
            }
        }

        if !new_facts.is_empty() {
            // Use imported debug macro
            debug!(num_new_facts = new_facts.len(), "Found new facts matching criteria");
            // Lock Tokio Mutex asynchronously
            let mut active_facts_guard = self.active_facts.lock().await;
            for (fact_id, fact, timestamp) in new_facts {
                // TODO: Need logic to handle fact updates/superseding if IDs aren't unique
                 let should_send = true;
                 // Simple check: only insert/update if timestamp is newer?
                 // Or always replace?
                /*
                 if let Some((_existing_fact, existing_ts)) = active_facts_guard.get(&fact_id) {
                    if timestamp <= *existing_ts {
                        should_send = false; // Don't send older or same timestamp fact
                    }
                 }
                 */

                if should_send {
                    // Use imported trace macro
                    trace!(%fact_id, "Updating active fact and sending callback");
                     active_facts_guard.insert(fact_id.clone(), (fact.clone(), timestamp));
                    if let Err(e) = self.config.callback_tx.send((fact_id, fact, timestamp)) {
                        // Use imported error macro
                        error!(error = %e, "Failed to send fact to callback channel");
                        // Potentially break or handle channel closure
                    }
                }
            }
        }

        // Update offset after processing the batch
        // Lock Tokio Mutex asynchronously
        *current_offset_guard = current_offset + num_entries;
        // Use imported trace macro
        trace!(new_offset = current_offset + num_entries, "Updated processed offset");
        Ok(())
    }
    
    /// Gets the currently active (latest replayed) facts.
    pub async fn get_active_facts(&self) -> HashMap<FactId, (FactEntry, Timestamp)> {
        // Lock Tokio Mutex asynchronously
        self.active_facts.lock().await.clone()
    }

    /// Determines a unique identifier for a fact entry.
    /// TODO: This needs a more robust implementation based on actual content or context.
    fn determine_fact_id(&self, fact: &FactEntry) -> FactId {
        // Simple ID based on domain and fact ID string for now
        FactId(format!("{}:{}", fact.domain_id, fact.fact_id))
        // Alternative: Use resource if available?
        /*
        if let Some(resources) = &fact.resources {
            if let Some(first_resource) = resources.first() {
                // TODO: Revisit - fact_type no longer exists
                // return FactId(format!("{}:{}", fact.fact_type, first_resource));
                return FactId(format!("{}:{}", "unknown_fact_type", first_resource)); 
            }
        }
        // Fallback if no resources
        // TODO: Revisit - fact_type and timestamp no longer exist on FactEntry
        // FactId(format!("{}:{}", fact.fact_type, fact.timestamp))
         FactId(format!("{}:{}", "unknown_fact_type", "unknown_timestamp"))
         */
    }
}

/// Runs the fact replay service in the background.
pub fn run_fact_replay_service<S: LogStorage + Send + Sync + 'static>(
    config: FactReplayConfig,
    storage: Arc<S>,
) -> (tokio::task::JoinHandle<Result<()>>, UnboundedSender<()>) {
    let (stop_tx, stop_rx) = tokio::sync::mpsc::unbounded_channel();
    let service = Arc::new(FactReplayService::new(config, storage));

    let handle = tokio::spawn(async move {
        service.run(stop_rx).await
    });

    (handle, stop_tx)
} 
