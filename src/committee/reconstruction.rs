//! Log Reconstruction
//!
//! This module provides functionality for reconstructing logs from extracted facts.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio::time::interval;
use serde::{Serialize, Deserialize};

use crate::committee::{Result, Error};
use crate::committee::extraction::ExtractedFact;
use crate::log::{LogEntry, LogStorage, MemoryLogStorage, LogSegment};

/// Configuration for a log reconstructor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconstructionConfig {
    /// The domain for which to reconstruct logs
    pub domain: String,
    /// The frequency at which to check for new facts (in seconds)
    pub check_interval_secs: u64,
    /// Buffer size for reconstructed log entries
    pub buffer_size: usize,
    /// Maximum batch size for adding entries to storage
    pub max_batch_size: usize,
    /// Whether to verify integrity of reconstructed logs
    pub verify_integrity: bool,
}

impl Default for ReconstructionConfig {
    fn default() -> Self {
        ReconstructionConfig {
            domain: "default".to_string(),
            check_interval_secs: 5,
            buffer_size: 1000,
            max_batch_size: 100,
            verify_integrity: true,
        }
    }
}

/// Status of a log reconstructor
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconstructorStatus {
    /// The reconstructor has not been started
    NotStarted,
    /// The reconstructor is running
    Running,
    /// The reconstructor has been paused
    Paused,
    /// The reconstructor has encountered an error
    Error,
}

/// Statistics for a log reconstructor
#[derive(Debug, Clone)]
pub struct ReconstructorStats {
    /// The number of facts processed
    pub facts_processed: u64,
    /// The number of entries reconstructed
    pub entries_reconstructed: u64,
    /// The number of facts that failed validation
    pub validation_failures: u64,
    /// The number of duplicate facts detected
    pub duplicates_detected: u64,
    /// The last time a fact was processed
    pub last_processed_time: Option<Instant>,
}

impl Default for ReconstructorStats {
    fn default() -> Self {
        ReconstructorStats {
            facts_processed: 0,
            entries_reconstructed: 0,
            validation_failures: 0,
            duplicates_detected: 0,
            last_processed_time: None,
        }
    }
}

/// A log entry reconstructor that rebuilds logs from extracted facts
pub struct LogReconstructor {
    /// Configuration for the reconstructor
    config: ReconstructionConfig,
    /// Storage for the reconstructed log
    storage: Arc<dyn LogStorage>,
    /// Channel for receiving facts
    fact_receiver: mpsc::Receiver<ExtractedFact>,
    /// Channel for sending reconstructed log entries
    entry_sender: mpsc::Sender<LogEntry>,
    /// Set of processed fact hashes to detect duplicates
    processed_facts: Mutex<HashSet<String>>,
    /// Current status of the reconstructor
    status: Mutex<ReconstructorStatus>,
    /// Statistics for the reconstructor
    stats: Mutex<ReconstructorStats>,
}

impl LogReconstructor {
    /// Create a new log reconstructor
    pub fn new(
        config: ReconstructionConfig,
        storage: Arc<dyn LogStorage>,
        fact_receiver: mpsc::Receiver<ExtractedFact>,
    ) -> Self {
        let (tx, _) = mpsc::channel(config.buffer_size);
        
        LogReconstructor {
            config,
            storage,
            fact_receiver,
            entry_sender: tx,
            processed_facts: Mutex::new(HashSet::new()),
            status: Mutex::new(ReconstructorStatus::NotStarted),
            stats: Mutex::new(ReconstructorStats::default()),
        }
    }
    
    /// Start the log reconstructor
    pub async fn start(&mut self) -> Result<mpsc::Receiver<LogEntry>> {
        {
            let mut status = self.status.lock().map_err(|_| {
                Error::Internal("Failed to acquire lock on status".to_string())
            })?;
            
            if *status == ReconstructorStatus::Running {
                return Err(Error::Internal("Reconstructor is already running".to_string()));
            }
            
            *status = ReconstructorStatus::Running;
        }
        
        // Create a new channel for reconstructed entries
        let (tx, rx) = mpsc::channel(self.config.buffer_size);
        self.entry_sender = tx;
        
        // Start the processing loop
        let config = self.config.clone();
        let storage = self.storage.clone();
        let processed_facts = self.processed_facts.clone();
        let status = self.status.clone();
        let stats = self.stats.clone();
        let mut fact_receiver = std::mem::replace(&mut self.fact_receiver, mpsc::channel(1).1);
        let entry_sender = self.entry_sender.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.check_interval_secs));
            let mut batch = Vec::new();
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Process any pending batch
                        if !batch.is_empty() {
                            if let Err(e) = process_batch(
                                &batch,
                                &storage,
                                &entry_sender,
                                &config,
                                &stats,
                            ).await {
                                log::error!("Error processing batch: {}", e);
                                
                                // Update status to error
                                if let Ok(mut status) = status.lock() {
                                    *status = ReconstructorStatus::Error;
                                }
                                
                                break;
                            }
                            
                            batch.clear();
                        }
                    }
                    result = fact_receiver.recv() => {
                        match result {
                            Some(fact) => {
                                // Check if the reconstructor is still running
                                if let Ok(status) = status.lock() {
                                    if *status != ReconstructorStatus::Running {
                                        break;
                                    }
                                }
                                
                                // Check for duplicate
                                let is_duplicate = {
                                    let mut processed = processed_facts.lock().map_err(|_| {
                                        log::error!("Failed to acquire lock on processed facts");
                                        true
                                    })?;
                                    
                                    if processed.contains(&fact.hash()) {
                                        // Update stats
                                        if let Ok(mut stats) = stats.lock() {
                                            stats.duplicates_detected += 1;
                                        }
                                        
                                        true
                                    } else {
                                        processed.insert(fact.hash());
                                        false
                                    }
                                };
                                
                                if !is_duplicate {
                                    // Add fact to batch
                                    batch.push(fact);
                                    
                                    // Process batch if it reaches the maximum size
                                    if batch.len() >= config.max_batch_size {
                                        if let Err(e) = process_batch(
                                            &batch,
                                            &storage,
                                            &entry_sender,
                                            &config,
                                            &stats,
                                        ).await {
                                            log::error!("Error processing batch: {}", e);
                                            
                                            // Update status to error
                                            if let Ok(mut status) = status.lock() {
                                                *status = ReconstructorStatus::Error;
                                            }
                                            
                                            break;
                                        }
                                        
                                        batch.clear();
                                    }
                                }
                            }
                            None => {
                                // Fact channel closed, exit loop
                                break;
                            }
                        }
                    }
                }
            }
            
            // Process any remaining facts
            if !batch.is_empty() {
                if let Err(e) = process_batch(
                    &batch,
                    &storage,
                    &entry_sender,
                    &config,
                    &stats,
                ).await {
                    log::error!("Error processing final batch: {}", e);
                }
            }
            
            log::info!("Log reconstructor stopped");
            
            // Update status to paused
            if let Ok(mut status) = status.lock() {
                if *status == ReconstructorStatus::Running {
                    *status = ReconstructorStatus::Paused;
                }
            }
        });
        
        Ok(rx)
    }
    
    /// Stop the log reconstructor
    pub async fn stop(&self) -> Result<()> {
        let mut status = self.status.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on status".to_string())
        })?;
        
        if *status != ReconstructorStatus::Running {
            return Ok(());
        }
        
        *status = ReconstructorStatus::Paused;
        
        Ok(())
    }
    
    /// Get the current status of the reconstructor
    pub fn get_status(&self) -> Result<ReconstructorStatus> {
        let status = self.status.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on status".to_string())
        })?;
        
        Ok(status.clone())
    }
    
    /// Get the current statistics of the reconstructor
    pub fn get_stats(&self) -> Result<ReconstructorStats> {
        let stats = self.stats.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on stats".to_string())
        })?;
        
        Ok(stats.clone())
    }
    
    /// Reset the statistics of the reconstructor
    pub fn reset_stats(&self) -> Result<()> {
        let mut stats = self.stats.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on stats".to_string())
        })?;
        
        *stats = ReconstructorStats::default();
        
        Ok(())
    }
    
    /// Clear the set of processed facts
    pub fn clear_processed_facts(&self) -> Result<()> {
        let mut processed = self.processed_facts.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on processed facts".to_string())
        })?;
        
        processed.clear();
        
        Ok(())
    }
    
    /// Get the underlying storage
    pub fn storage(&self) -> Arc<dyn LogStorage> {
        self.storage.clone()
    }
}

/// Process a batch of facts
async fn process_batch(
    facts: &[ExtractedFact],
    storage: &Arc<dyn LogStorage>,
    entry_sender: &mpsc::Sender<LogEntry>,
    config: &ReconstructionConfig,
    stats: &Mutex<ReconstructorStats>,
) -> Result<()> {
    // Convert facts to log entries
    let mut entries = Vec::new();
    
    for fact in facts {
        if fact.domain != config.domain {
            continue; // Skip facts for other domains
        }
        
        match fact.to_log_entry() {
            Ok(entry) => {
                entries.push(entry);
            }
            Err(e) => {
                log::warn!("Failed to convert fact to log entry: {}", e);
                
                // Update stats
                if let Ok(mut stats) = stats.lock() {
                    stats.validation_failures += 1;
                }
            }
        }
    }
    
    // Skip if no entries to add
    if entries.is_empty() {
        return Ok(());
    }
    
    // Add entries to storage
    storage.add_entries(&entries).await?;
    
    // Send entries to channel
    for entry in &entries {
        if let Err(e) = entry_sender.send(entry.clone()).await {
            log::warn!("Failed to send reconstructed entry: {}", e);
        }
    }
    
    // Update stats
    {
        let mut stats_guard = stats.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on stats".to_string())
        })?;
        
        stats_guard.facts_processed += facts.len() as u64;
        stats_guard.entries_reconstructed += entries.len() as u64;
        stats_guard.last_processed_time = Some(Instant::now());
    }
    
    Ok(())
}

/// A factory for creating log reconstructors
pub struct ReconstructorFactory {
    /// Default configuration for reconstructors
    default_config: ReconstructionConfig,
}

impl ReconstructorFactory {
    /// Create a new reconstructor factory with default configuration
    pub fn new(default_config: ReconstructionConfig) -> Self {
        ReconstructorFactory { default_config }
    }
    
    /// Create a new log reconstructor
    pub fn create_reconstructor(
        &self,
        domain: &str,
        fact_receiver: mpsc::Receiver<ExtractedFact>,
    ) -> Result<LogReconstructor> {
        let mut config = self.default_config.clone();
        config.domain = domain.to_string();
        
        // Create storage for the domain
        let storage = Arc::new(MemoryLogStorage::new());
        
        Ok(LogReconstructor::new(config, storage, fact_receiver))
    }
    
    /// Create a new log reconstructor with custom configuration and storage
    pub fn create_reconstructor_with_config(
        &self,
        config: ReconstructionConfig,
        storage: Arc<dyn LogStorage>,
        fact_receiver: mpsc::Receiver<ExtractedFact>,
    ) -> LogReconstructor {
        LogReconstructor::new(config, storage, fact_receiver)
    }
}

/// A registry for managing multiple log reconstructors
pub struct ReconstructorRegistry {
    /// Map of domains to reconstructors
    reconstructors: Mutex<HashMap<String, LogReconstructor>>,
    /// Factory for creating reconstructors
    factory: ReconstructorFactory,
}

impl ReconstructorRegistry {
    /// Create a new reconstructor registry
    pub fn new(factory: ReconstructorFactory) -> Self {
        ReconstructorRegistry {
            reconstructors: Mutex::new(HashMap::new()),
            factory,
        }
    }
    
    /// Register a reconstructor for a domain
    pub fn register(
        &self,
        domain: &str,
        reconstructor: LogReconstructor,
    ) -> Result<()> {
        let mut reconstructors = self.reconstructors.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on reconstructors".to_string())
        })?;
        
        reconstructors.insert(domain.to_string(), reconstructor);
        
        Ok(())
    }
    
    /// Create and register a reconstructor for a domain
    pub fn create_and_register(
        &self,
        domain: &str,
        fact_receiver: mpsc::Receiver<ExtractedFact>,
    ) -> Result<()> {
        let reconstructor = self.factory.create_reconstructor(domain, fact_receiver)?;
        self.register(domain, reconstructor)
    }
    
    /// Get a reconstructor for a domain
    pub fn get(&self, domain: &str) -> Result<LogReconstructor> {
        let reconstructors = self.reconstructors.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on reconstructors".to_string())
        })?;
        
        reconstructors.get(domain).cloned().ok_or_else(|| {
            Error::Configuration(format!("No reconstructor for domain '{}'", domain))
        })
    }
    
    /// Remove a reconstructor for a domain
    pub fn remove(&self, domain: &str) -> Result<()> {
        let mut reconstructors = self.reconstructors.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on reconstructors".to_string())
        })?;
        
        if reconstructors.remove(domain).is_none() {
            return Err(Error::Configuration(format!(
                "No reconstructor for domain '{}'", domain
            )));
        }
        
        Ok(())
    }
    
    /// Get all registered domains
    pub fn get_domains(&self) -> Result<Vec<String>> {
        let reconstructors = self.reconstructors.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on reconstructors".to_string())
        })?;
        
        Ok(reconstructors.keys().cloned().collect())
    }
    
    /// Start all reconstructors
    pub async fn start_all(&self) -> Result<()> {
        let domains = self.get_domains()?;
        
        for domain in domains {
            let mut reconstructor = self.get(&domain)?;
            reconstructor.start().await?;
        }
        
        Ok(())
    }
    
    /// Stop all reconstructors
    pub async fn stop_all(&self) -> Result<()> {
        let domains = self.get_domains()?;
        
        for domain in domains {
            let reconstructor = self.get(&domain)?;
            reconstructor.stop().await?;
        }
        
        Ok(())
    }
} 