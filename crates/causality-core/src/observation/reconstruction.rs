// Observation reconstruction functionality
//
// This module provides functionality for reconstructing logs from extracted facts.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use causality_types::{Error, Result};
use crate::observation::extraction::ExtractedFact;
use crate::log::{LogEntry, LogStorage};

/// Configuration for a log reconstructor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconstructionConfig {
    /// The fact types to reconstruct from
    pub fact_types: Vec<String>,
    /// The maximum number of facts to buffer
    pub max_buffer_size: usize,
    /// The target log ID
    pub log_id: String,
    /// Whether to validate reconstruction
    pub validate: bool,
    /// Whether to persist reconstructed logs
    pub persist_logs: bool,
    /// Storage directory for persisted logs
    pub storage_dir: Option<String>,
}

impl Default for ReconstructionConfig {
    fn default() -> Self {
        ReconstructionConfig {
            fact_types: Vec::new(),
            max_buffer_size: 1000,
            log_id: "default".to_string(),
            validate: true,
            persist_logs: true,
            storage_dir: None,
        }
    }
}

/// Status of a log reconstructor
#[derive(Debug, Clone)]
pub struct ReconstructionStatus {
    /// The log ID
    pub log_id: String,
    /// The number of facts processed
    pub facts_processed: u64,
    /// The number of log entries reconstructed
    pub entries_reconstructed: u64,
    /// The number of facts in the buffer
    pub buffer_size: usize,
    /// The latest fact processed time
    pub latest_processed_time: Option<u64>,
}

/// An interface for reconstructing logs from facts
pub trait LogReconstructor: Send + Sync {
    /// Process a fact for reconstruction
    fn process_fact(&self, fact: &ExtractedFact) -> Result<()>;
    
    /// Get the status of the reconstructor
    fn get_status(&self) -> Result<ReconstructionStatus>;
    
    /// Get the reconstructed log
    fn get_log(&self) -> Result<Arc<dyn LogStorage>>;
}

/// Factory for creating log reconstructors
pub struct ReconstructorFactory {
    /// Storage for logs
    log_storage: RwLock<HashMap<String, Arc<dyn LogStorage>>>,
}

impl ReconstructorFactory {
    /// Create a new reconstructor factory
    pub fn new() -> Self {
        ReconstructorFactory {
            log_storage: RwLock::new(HashMap::new()),
        }
    }
    
    /// Create a new log reconstructor
    pub fn create_reconstructor(
        &self,
        config: ReconstructionConfig,
    ) -> Result<Arc<BasicReconstructor>> {
        // Get or create the log storage
        let log_id = config.log_id.clone();
        let log_storage = {
            let mut storages = self.log_storage.write().map_err(|_| 
                Error::Internal("Failed to lock log storage".to_string()))?;
                
            if let Some(storage) = storages.get(&log_id) {
                storage.clone()
            } else {
                // Create new storage
                let storage = Arc::new(crate::log::MemoryLogStorage::new());
                storages.insert(log_id.clone(), storage.clone());
                storage
            }
        };
        
        // Create the reconstructor
        let reconstructor = Arc::new(BasicReconstructor::new(config, log_storage));
        
        Ok(reconstructor)
    }
}

/// A basic implementation of a log reconstructor
pub struct BasicReconstructor {
    /// Configuration for the reconstructor
    config: ReconstructionConfig,
    /// Storage for reconstructed logs
    log_storage: Arc<dyn LogStorage>,
    /// Buffer for processed facts
    fact_buffer: Mutex<VecDeque<ExtractedFact>>,
    /// Status of the reconstructor
    status: Mutex<ReconstructionStatus>,
}

impl BasicReconstructor {
    /// Create a new basic reconstructor
    pub fn new(
        config: ReconstructionConfig,
        log_storage: Arc<dyn LogStorage>,
    ) -> Self {
        // Initialize status
        let status = ReconstructionStatus {
            log_id: config.log_id.clone(),
            facts_processed: 0,
            entries_reconstructed: 0,
            buffer_size: 0,
            latest_processed_time: None,
        };
        
        BasicReconstructor {
            config,
            log_storage,
            fact_buffer: Mutex::new(VecDeque::with_capacity(config.max_buffer_size)),
            status: Mutex::new(status),
        }
    }
    
    /// Start processing facts from a receiver
    pub fn start_processing(
        self: Arc<Self>,
        mut fact_receiver: mpsc::Receiver<ExtractedFact>,
    ) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            while let Some(fact) = fact_receiver.recv().await {
                if let Err(e) = self.process_fact(&fact) {
                    log::error!("Error processing fact: {}", e);
                }
            }
            
            Ok(())
        })
    }
    
    /// Process facts in the buffer
    fn process_buffer(&self) -> Result<()> {
        let mut buffer = self.fact_buffer.lock().map_err(|_| 
            Error::Internal("Failed to lock fact buffer".to_string()))?;
            
        // Sort facts by timestamp or height
        buffer.make_contiguous().sort_by(|a, b| {
            a.block_height.cmp(&b.block_height)
        });
        
        // Process facts in order
        let facts_to_process = buffer.len();
        let mut entries_reconstructed = 0;
        
        for _ in 0..facts_to_process {
            if let Some(fact) = buffer.pop_front() {
                // Reconstruct log entry from fact
                if let Ok(entry) = self.reconstruct_entry(&fact) {
                    // Append to log
                    if let Err(e) = self.log_storage.append(entry.clone()) {
                        log::error!("Failed to append log entry: {}", e);
                    } else {
                        entries_reconstructed += 1;
                    }
                }
                
                // Update status
                {
                    let mut status = self.status.lock().map_err(|_| 
                        Error::Internal("Failed to lock status".to_string()))?;
                        
                    status.facts_processed += 1;
                    status.entries_reconstructed += entries_reconstructed;
                    status.buffer_size = buffer.len();
                    status.latest_processed_time = Some(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    );
                }
            }
        }
        
        Ok(())
    }
    
    /// Reconstruct a log entry from a fact
    fn reconstruct_entry(&self, fact: &ExtractedFact) -> Result<LogEntry> {
        // Create a log entry from the fact
        let entry = LogEntry {
            log_id: self.config.log_id.clone(),
            sequence: fact.block_height,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            data: serde_json::to_value(fact).map_err(|e| 
                Error::Serialization(format!("Failed to serialize fact: {}", e)))?,
            metadata: fact.metadata.clone(),
        };
        
        Ok(entry)
    }
}

impl LogReconstructor for BasicReconstructor {
    /// Process a fact for reconstruction
    fn process_fact(&self, fact: &ExtractedFact) -> Result<()> {
        // Check if this fact type is relevant
        if !self.config.fact_types.is_empty() && 
           !self.config.fact_types.contains(&fact.fact_type) {
            return Ok(());
        }
        
        // Add to buffer
        {
            let mut buffer = self.fact_buffer.lock().map_err(|_| 
                Error::Internal("Failed to lock fact buffer".to_string()))?;
                
            buffer.push_back(fact.clone());
            
            // Process buffer if it's full
            if buffer.len() >= self.config.max_buffer_size {
                drop(buffer); // Release the lock before processing
                self.process_buffer()?;
            }
        }
        
        Ok(())
    }
    
    /// Get the status of the reconstructor
    fn get_status(&self) -> Result<ReconstructionStatus> {
        let status = self.status.lock().map_err(|_| 
            Error::Internal("Failed to lock status".to_string()))?;
            
        Ok(status.clone())
    }
    
    /// Get the reconstructed log
    fn get_log(&self) -> Result<Arc<dyn LogStorage>> {
        Ok(self.log_storage.clone())
    }
} 