// Observation proxy functionality
//
// This module provides a proxy for external chain interaction and fact extraction.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use thiserror::Error;

use crate::log::{LogStorage, LogStorageError};
use crate::observation::extraction::{
    BlockData, ExtractedFact, FactExtractor, RuleEngine,
    BasicExtractor, ExtractionRule
};
use crate::observation::indexer::{
    IndexerConfig, ChainIndexer, 
    IndexerFactory, IndexerError
};

/// Configuration for the observation proxy
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// The size of the fact buffer
    pub fact_buffer_size: usize,
    /// The polling interval in seconds
    pub polling_interval: u64,
}

/// Error type for proxy operations
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Data error: {0}")]
    Data(String),
    
    #[error("Extraction error: {0}")]
    Extraction(String),
    
    #[error("Indexer error: {0}")]
    Indexer(#[from] IndexerError),
    
    #[error("Storage error: {0}")]
    Storage(#[from] LogStorageError),
}

/// Status of a chain
#[derive(Debug, Clone)]
pub struct ChainStatus {
    /// The chain ID
    pub chain_id: String,
    /// Whether the chain is being observed
    pub is_observed: bool,
    /// Latest processed block height
    pub latest_processed_height: u64,
    /// Latest known block height
    pub latest_known_height: u64,
    /// Number of facts extracted
    pub facts_extracted: u64,
    /// Last update timestamp
    pub last_updated: u64,
}

/// Proxy event types
#[derive(Debug, Clone)]
pub enum ProxyEvent {
    /// New facts extracted
    FactsExtracted {
        /// The chain ID
        chain_id: String,
        /// The block height
        block_height: u64,
        /// The number of facts
        count: usize,
    },
    /// Chain sync status updated
    ChainSyncUpdated {
        /// The chain ID
        chain_id: String,
        /// The latest processed height
        processed_height: u64,
        /// The latest known height
        known_height: u64,
    },
    /// Error occurred
    Error {
        /// The chain ID
        chain_id: Option<String>,
        /// The error message
        message: String,
    },
}

/// Trait for proxy event handlers
#[async_trait::async_trait]
pub trait ProxyEventHandler: Send + Sync {
    /// Handle a proxy event
    async fn handle_event(&self, event: ProxyEvent) -> std::result::Result<(), ProxyError>;
}

/// A simple logging event handler
pub struct LoggingEventHandler {
    /// The log storage
    log: Arc<dyn LogStorage>,
}

impl LoggingEventHandler {
    /// Create a new logging event handler
    pub fn new(log: Arc<dyn LogStorage>) -> Self {
        LoggingEventHandler { log }
    }
}

#[async_trait::async_trait]
impl ProxyEventHandler for LoggingEventHandler {
    async fn handle_event(&self, event: ProxyEvent) -> std::result::Result<(), ProxyError> {
        match event {
            ProxyEvent::FactsExtracted { chain_id, block_height, count } => {
                let entry = crate::log::LogEntry {
                    log_id: "observation_proxy".to_string(),
                    sequence: 0, // Will be set by storage
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    data: serde_json::json!({
                        "event_type": "facts_extracted",
                        "chain_id": chain_id,
                        "block_height": block_height,
                        "count": count,
                    }),
                    metadata: HashMap::new(),
                };
                
                self.log.append(entry)?;
            }
            ProxyEvent::ChainSyncUpdated { chain_id, processed_height, known_height } => {
                let entry = crate::log::LogEntry {
                    log_id: "observation_proxy".to_string(),
                    sequence: 0, // Will be set by storage
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    data: serde_json::json!({
                        "event_type": "chain_sync_updated",
                        "chain_id": chain_id,
                        "processed_height": processed_height,
                        "known_height": known_height,
                    }),
                    metadata: HashMap::new(),
                };
                
                self.log.append(entry)?;
            }
            ProxyEvent::Error { chain_id, message } => {
                let entry = crate::log::LogEntry {
                    log_id: "observation_proxy".to_string(),
                    sequence: 0, // Will be set by storage
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    data: serde_json::json!({
                        "event_type": "error",
                        "chain_id": chain_id,
                        "message": message,
                    }),
                    metadata: HashMap::new(),
                };
                
                self.log.append(entry)?;
            }
        }
        
        Ok(())
    }
}

/// Observation proxy for blockchain data
pub struct ObservationProxy {
    /// Configuration for the proxy
    config: ProxyConfig,
    /// Factory for creating indexers
    indexer_factory: Arc<IndexerFactory>,
    /// Rule engine for fact extraction
    rule_engine: Arc<RuleEngine>,
    /// Indexers for each chain
    indexers: Mutex<HashMap<String, Arc<dyn ChainIndexer>>>,
    /// Extractors for each chain
    extractors: Mutex<HashMap<String, Arc<dyn FactExtractor>>>,
    /// Chain statuses
    statuses: Mutex<HashMap<String, ChainStatus>>,
    /// Sender for extracted facts
    fact_sender: mpsc::Sender<ExtractedFact>,
    /// Receiver for extracted facts
    fact_receiver: Arc<Mutex<Option<mpsc::Receiver<ExtractedFact>>>>,
    /// Event handlers
    event_handlers: Mutex<Vec<Arc<dyn ProxyEventHandler>>>,
    /// Whether the proxy is running
    running: Mutex<bool>,
}

impl ObservationProxy {
    /// Create a new observation proxy
    pub fn new(config: ProxyConfig) -> std::result::Result<Self, ProxyError> {
        // Create channels for facts
        let (fact_sender, fact_receiver) = mpsc::channel(config.fact_buffer_size);
        
        Ok(ObservationProxy {
            config,
            indexer_factory: Arc::new(IndexerFactory::new()),
            rule_engine: Arc::new(RuleEngine::new()),
            indexers: Mutex::new(HashMap::new()),
            extractors: Mutex::new(HashMap::new()),
            statuses: Mutex::new(HashMap::new()),
            fact_sender,
            fact_receiver: Arc::new(Mutex::new(Some(fact_receiver))),
            event_handlers: Mutex::new(Vec::new()),
            running: Mutex::new(false),
        })
    }

    /// Initialize the proxy with a collection of indexer configurations
    pub fn initialize(&self, indexer_configs: Vec<IndexerConfig>) -> std::result::Result<(), ProxyError> {
        // Create indexers and extractors for each chain
        let mut indexers = self.indexers.lock().map_err(|e| 
            ProxyError::Internal(format!("Failed to lock indexers: {}", e)))?;
            
        let mut extractors = self.extractors.lock().map_err(|e| 
            ProxyError::Internal(format!("Failed to lock extractors: {}", e)))?;
            
        let mut statuses = self.statuses.lock().map_err(|e| 
            ProxyError::Internal(format!("Failed to lock statuses: {}", e)))?;
            
        for config in indexer_configs {
            let chain_id = config.chain_id.clone();
            
            // Create indexer
            let indexer = self.indexer_factory.create(config)?;
            
            // Create extractor
            let extractor = Arc::new(BasicExtractor::new(
                chain_id.clone(),
                self.rule_engine.clone()
            ));
            
            // Create chain status
            let status = ChainStatus {
                chain_id: chain_id.clone(),
                is_observed: false,
                latest_processed_height: 0,
                latest_known_height: 0,
                facts_extracted: 0,
                last_updated: chrono::Utc::now().timestamp() as u64,
            };
            
            indexers.insert(chain_id.clone(), indexer);
            extractors.insert(chain_id.clone(), extractor);
            statuses.insert(chain_id, status);
        }
        
        Ok(())
    }

    /// Start observing chains
    pub fn start(&self) -> std::result::Result<(), ProxyError> {
        let mut running = self.running.lock().map_err(|e| 
            ProxyError::Internal(format!("Failed to lock running: {}", e)))?;
            
        if *running {
            return Ok(());
        }
        
        *running = true;
        
        // Start indexers
        let indexers = self.indexers.lock().map_err(|e| 
            ProxyError::Internal(format!("Failed to lock indexers: {}", e)))?;
            
        let mut statuses = self.statuses.lock().map_err(|e| 
            ProxyError::Internal(format!("Failed to lock statuses: {}", e)))?;
            
        for (chain_id, indexer) in indexers.iter() {
            // Initialize and start the indexer
            tokio::spawn({
                let indexer = indexer.clone();
                async move {
                    if let Err(e) = indexer.initialize().await {
                        log::error!("Failed to initialize indexer for chain {}: {:?}", chain_id, e);
                    }
                    if let Err(e) = indexer.start().await {
                        log::error!("Failed to start indexer for chain {}: {:?}", chain_id, e);
                    }
                }
            });
            
            // Update status
            if let Some(status) = statuses.get_mut(chain_id) {
                status.is_observed = true;
                status.last_updated = chrono::Utc::now().timestamp() as u64;
            }
            
            // Start polling for this chain
            tokio::spawn({
                let proxy = self.clone();
                let chain_id = chain_id.clone();
                async move {
                    loop {
                        // Check if we're still running
                        if !proxy.is_running().unwrap_or(false) {
                            break;
                        }
                        
                        // Poll the chain
                        if let Err(e) = proxy.poll_chain(&chain_id).await {
                            log::error!("Error polling chain {}: {:?}", chain_id, e);
                            
                            // Emit error event
                            proxy.emit_event(ProxyEvent::Error {
                                chain_id: Some(chain_id.clone()),
                                message: format!("Error polling chain: {:?}", e),
                            }).await;
                        }
                        
                        // Sleep for polling interval
                        tokio::time::sleep(tokio::time::Duration::from_secs(
                            proxy.config.polling_interval
                        )).await;
                    }
                }
            });
        }
        
        Ok(())
    }

    /// Stop observing chains
    pub fn stop(&self) -> std::result::Result<(), ProxyError> {
        let mut running = self.running.lock().map_err(|e| 
            ProxyError::Internal(format!("Failed to lock running: {}", e)))?;
            
        if !*running {
            return Ok(());
        }
        
        *running = false;
        
        // Stop indexers
        let indexers = self.indexers.lock().map_err(|e| 
            ProxyError::Internal(format!("Failed to lock indexers: {}", e)))?;
            
        let mut statuses = self.statuses.lock().map_err(|e| 
            ProxyError::Internal(format!("Failed to lock statuses: {}", e)))?;
            
        for (chain_id, indexer) in indexers.iter() {
            // Stop the indexer
            tokio::spawn({
                let indexer = indexer.clone();
                async move {
                    if let Err(e) = indexer.stop().await {
                        log::error!("Failed to stop indexer for chain {}: {:?}", chain_id, e);
                    }
                }
            });
            
            // Update status
            if let Some(status) = statuses.get_mut(chain_id) {
                status.is_observed = false;
                status.last_updated = chrono::Utc::now().timestamp() as u64;
            }
        }
        
        Ok(())
    }

    /// Clone the proxy
    fn clone(&self) -> Self {
        let fact_receiver_lock = self.fact_receiver.lock().unwrap();
        let (new_sender, new_receiver) = mpsc::channel(self.config.fact_buffer_size);
        
        ObservationProxy {
            config: self.config.clone(),
            indexer_factory: self.indexer_factory.clone(),
            rule_engine: self.rule_engine.clone(),
            indexers: Mutex::new(HashMap::new()),
            extractors: Mutex::new(HashMap::new()),
            statuses: Mutex::new(HashMap::new()),
            fact_sender: new_sender,
            fact_receiver: Arc::new(Mutex::new(Some(new_receiver))),
            event_handlers: Mutex::new(Vec::new()),
            running: Mutex::new(false),
        }
    }

    /// Emit an event
    async fn emit_event(&self, event: ProxyEvent) {
        let handlers = self.event_handlers.lock().unwrap_or_else(|_| {
            log::error!("Failed to lock event handlers");
            return Vec::new().into();
        });
        
        for handler in handlers.iter() {
            if let Err(e) = handler.handle_event(event.clone()).await {
                log::error!("Error in event handler: {:?}", e);
            }
        }
    }

    /// Poll a chain for new blocks
    async fn poll_chain(&self, chain_id: &str) -> std::result::Result<(), ProxyError> {
        // Get the indexer and extractor
        let (indexer, extractor) = {
            let indexers = self.indexers.lock().map_err(|e| 
                ProxyError::Internal(format!("Failed to lock indexers: {}", e)))?;
                
            let extractors = self.extractors.lock().map_err(|e| 
                ProxyError::Internal(format!("Failed to lock extractors: {}", e)))?;
                
            let indexer = indexers.get(chain_id)
                .ok_or_else(|| 
                    ProxyError::Internal(format!("Indexer not found: {}", chain_id))
                )?
                .clone();
                
            let extractor = extractors.get(chain_id)
                .ok_or_else(|| 
                    ProxyError::Internal(format!("Extractor not found: {}", chain_id))
                )?
                .clone();
                
            (indexer, extractor)
        };
        
        // Get latest heights
        let latest_height = indexer.get_latest_height().await?;
        let synced_height = indexer.get_synced_height().await?;
        
        // Update chain status
        {
            let mut statuses = self.statuses.lock().map_err(|e| 
                ProxyError::Internal(format!("Failed to lock statuses: {}", e)))?;
                
            if let Some(status) = statuses.get_mut(chain_id) {
                status.latest_processed_height = synced_height;
                status.latest_known_height = latest_height;
                status.last_updated = chrono::Utc::now().timestamp() as u64;
            }
        }
        
        // Emit update event
        self.emit_event(ProxyEvent::ChainSyncUpdated {
            chain_id: chain_id.to_string(),
            processed_height: synced_height,
            known_height: latest_height,
        }).await;
        
        // If we're already synced, nothing to do
        if synced_height >= latest_height {
            return Ok(());
        }
        
        // Get the next block
        let next_height = synced_height + 1;
        let block = indexer.get_block(next_height).await?;
        
        // Extract facts from the block
        let facts = extractor.extract_facts(&block).await
            .map_err(|e| ProxyError::Extraction(format!("Failed to extract facts: {}", e)))?;
        
        // If we found facts, send them
        if !facts.is_empty() {
            for fact in &facts {
                if let Err(e) = self.fact_sender.send(fact.clone()).await {
                    log::error!("Failed to send fact: {:?}", e);
                }
            }
            
            // Update facts extracted count
            {
                let mut statuses = self.statuses.lock().map_err(|e| 
                    ProxyError::Internal(format!("Failed to lock statuses: {}", e)))?;
                    
                if let Some(status) = statuses.get_mut(chain_id) {
                    status.facts_extracted += facts.len() as u64;
                }
            }
            
            // Emit event
            self.emit_event(ProxyEvent::FactsExtracted {
                chain_id: chain_id.to_string(),
                block_height: next_height,
                count: facts.len(),
            }).await;
        }
        
        // Update synced height
        indexer.set_synced_height(next_height).await?;
        
        Ok(())
    }
    
    /// Check if the proxy is running
    fn is_running(&self) -> std::result::Result<bool, ProxyError> {
        let running = self.running.lock().map_err(|e| 
            ProxyError::Internal(format!("Failed to lock running: {}", e)))?;
            
        Ok(*running)
    }
    
    /// Add an event handler
    pub fn add_event_handler(&self, handler: Arc<dyn ProxyEventHandler>) -> std::result::Result<(), ProxyError> {
        let mut handlers = self.event_handlers.lock().map_err(|e| 
            ProxyError::Internal(format!("Failed to lock event handlers: {}", e)))?;
            
        handlers.push(handler);
        
        Ok(())
    }
    
    /// Add a rule to the rule engine
    pub fn add_rule(&self, rule: ExtractionRule) -> std::result::Result<(), ProxyError> {
        self.rule_engine.add_rule(rule).map_err(|e|
            ProxyError::Extraction(format!("Failed to add rule: {}", e)))
    }
    
    /// Load rules from a TOML string
    pub fn load_rules_from_toml(&self, toml_str: &str) -> std::result::Result<(), ProxyError> {
        self.rule_engine.load_rules_from_toml(toml_str).map_err(|e|
            ProxyError::Extraction(format!("Failed to load rules: {}", e)))
    }
    
    /// Take the fact receiver
    pub fn take_fact_receiver(&self) -> std::result::Result<mpsc::Receiver<ExtractedFact>, ProxyError> {
        let mut receiver = self.fact_receiver.lock().map_err(|e| 
            ProxyError::Internal(format!("Failed to lock fact receiver: {}", e)))?;
            
        receiver.take().ok_or_else(|| 
            ProxyError::Internal("Fact receiver already taken".to_string())
        )
    }
    
    /// Get a chain status
    pub fn get_chain_status(&self, chain_id: &str) -> std::result::Result<ChainStatus, ProxyError> {
        let statuses = self.statuses.lock().map_err(|e| 
            ProxyError::Internal(format!("Failed to lock statuses: {}", e)))?;
            
        statuses.get(chain_id)
            .cloned()
            .ok_or_else(|| ProxyError::Data(format!("Chain not found: {}", chain_id)))
    }
    
    /// Get all chain statuses
    pub fn get_all_chain_statuses(&self) -> std::result::Result<Vec<ChainStatus>, ProxyError> {
        let statuses = self.statuses.lock().map_err(|e| 
            ProxyError::Internal(format!("Failed to lock statuses: {}", e)))?;
            
        Ok(statuses.values().cloned().collect())
    }
} 