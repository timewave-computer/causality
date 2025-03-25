// Committee proxy functionality
// Original file: src/committee/proxy.rs

//! Committee Proxy
//!
//! This module provides a proxy for external chain interaction and fact extraction.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;

use causality_core::{Result, Error};
use causality_core::indexer::{
    ChainIndexer, IndexerConfig, IndexerFactory, IndexerStatus, IndexingOptions
};
use causality_core::extraction::{
    FactExtractor, ExtractedFact, ExtractionRule, RuleEngine, BasicExtractor
};
use crate::log::{LogEntry, LogStorage};

/// Configuration for a committee proxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Configuration for each chain to index
    pub chains: Vec<IndexerConfig>,
    /// How often to poll for new blocks (in seconds)
    pub poll_interval_secs: u64,
    /// Maximum number of blocks to process in one batch
    pub max_blocks_per_batch: u32,
    /// Whether to persist extracted facts
    pub persist_facts: bool,
    /// Storage directory for persisted facts
    pub storage_dir: Option<String>,
    /// Buffer size for fact channel
    pub fact_buffer_size: usize,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        ProxyConfig {
            chains: Vec::new(),
            poll_interval_secs: 10,
            max_blocks_per_batch: 100,
            persist_facts: true,
            storage_dir: None,
            fact_buffer_size: 1000,
        }
    }
}

/// Status of a chain being indexed
#[derive(Debug, Clone)]
pub struct ChainStatus {
    /// The chain ID
    pub chain_id: String,
    /// The indexer status
    pub indexer_status: IndexerStatus,
    /// The latest processed block height
    pub latest_processed_height: u64,
    /// The chain head height
    pub chain_head_height: u64,
    /// The number of facts extracted
    pub facts_extracted: u64,
    /// The last time a block was processed
    pub last_processed_time: Option<Instant>,
}

/// Event emitted by the committee proxy
#[derive(Debug, Clone)]
pub enum ProxyEvent {
    /// Indexer status changed
    IndexerStatusChanged {
        chain_id: String,
        status: IndexerStatus,
    },
    /// New block processed
    BlockProcessed {
        chain_id: String,
        block_height: u64,
        block_hash: String,
        fact_count: usize,
    },
    /// Facts extracted
    FactsExtracted {
        chain_id: String,
        facts: Vec<ExtractedFact>,
    },
    /// Error occurred
    Error {
        chain_id: String,
        message: String,
    },
}

/// A handler that receives proxy events
#[async_trait]
pub trait ProxyEventHandler: Send + Sync {
    /// Handle a proxy event
    async fn handle_event(&self, event: ProxyEvent) -> Result<()>;
}

/// A committee proxy that interacts with external chains
pub struct CommitteeProxy {
    /// Configuration for the proxy
    config: ProxyConfig,
    /// Factory for creating indexers
    indexer_factory: Arc<IndexerFactory>,
    /// Engine for managing extraction rules
    rule_engine: Arc<RuleEngine>,
    /// Map of chain IDs to indexers
    indexers: Mutex<HashMap<String, Arc<dyn ChainIndexer>>>,
    /// Map of chain IDs to extractors
    extractors: Mutex<HashMap<String, Arc<dyn FactExtractor>>>,
    /// Map of chain IDs to statuses
    statuses: Mutex<HashMap<String, ChainStatus>>,
    /// Channel for sending extracted facts
    fact_sender: mpsc::Sender<ExtractedFact>,
    /// Channel for receiving extracted facts
    fact_receiver: Arc<Mutex<Option<mpsc::Receiver<ExtractedFact>>>>,
    /// Event handlers
    event_handlers: Mutex<Vec<Arc<dyn ProxyEventHandler>>>,
    /// Whether the proxy is running
    running: Mutex<bool>,
}

impl CommitteeProxy {
    /// Create a new committee proxy
    pub fn new(config: ProxyConfig) -> Result<Self> {
        let (tx, rx) = mpsc::channel(config.fact_buffer_size);
        
        Ok(CommitteeProxy {
            config,
            indexer_factory: Arc::new(IndexerFactory::new()),
            rule_engine: Arc::new(RuleEngine::new()),
            indexers: Mutex::new(HashMap::new()),
            extractors: Mutex::new(HashMap::new()),
            statuses: Mutex::new(HashMap::new()),
            fact_sender: tx,
            fact_receiver: Arc::new(Mutex::new(Some(rx))),
            event_handlers: Mutex::new(Vec::new()),
            running: Mutex::new(false),
        })
    }
    
    /// Initialize the proxy
    pub async fn initialize(&self) -> Result<()> {
        for chain_config in &self.config.chains {
            self.add_chain(chain_config.clone()).await?;
        }
        
        Ok(())
    }
    
    /// Add a chain to the proxy
    pub async fn add_chain(&self, config: IndexerConfig) -> Result<()> {
        let chain_id = config.chain_id.clone();
        
        // Create indexer
        let indexer = self.indexer_factory.create_indexer(config.clone())?;
        
        // Initialize indexer
        indexer.initialize().await?;
        
        // Create extractor
        let extractor = Arc::new(BasicExtractor::new(chain_id.clone(), self.rule_engine.clone()));
        
        // Add to indexers and extractors
        {
            let mut indexers = self.indexers.lock().map_err(|_| {
                Error::Internal("Failed to acquire lock on indexers".to_string())
            })?;
            
            let mut extractors = self.extractors.lock().map_err(|_| {
                Error::Internal("Failed to acquire lock on extractors".to_string())
            })?;
            
            let mut statuses = self.statuses.lock().map_err(|_| {
                Error::Internal("Failed to acquire lock on statuses".to_string())
            })?;
            
            indexers.insert(chain_id.clone(), indexer.clone());
            extractors.insert(chain_id.clone(), extractor);
            
            statuses.insert(chain_id.clone(), ChainStatus {
                chain_id: chain_id.clone(),
                indexer_status: IndexerStatus::NotStarted,
                latest_processed_height: 0,
                chain_head_height: 0,
                facts_extracted: 0,
                last_processed_time: None,
            });
        }
        
        Ok(())
    }
    
    /// Remove a chain from the proxy
    pub async fn remove_chain(&self, chain_id: &str) -> Result<()> {
        // Get indexer
        let indexer = {
            let indexers = self.indexers.lock().map_err(|_| {
                Error::Internal("Failed to acquire lock on indexers".to_string())
            })?;
            
            indexers.get(chain_id).cloned()
        };
        
        // Stop indexing if running
        if let Some(indexer) = indexer {
            indexer.stop_indexing().await?;
        }
        
        // Remove from indexers, extractors, and statuses
        {
            let mut indexers = self.indexers.lock().map_err(|_| {
                Error::Internal("Failed to acquire lock on indexers".to_string())
            })?;
            
            let mut extractors = self.extractors.lock().map_err(|_| {
                Error::Internal("Failed to acquire lock on extractors".to_string())
            })?;
            
            let mut statuses = self.statuses.lock().map_err(|_| {
                Error::Internal("Failed to acquire lock on statuses".to_string())
            })?;
            
            indexers.remove(chain_id);
            extractors.remove(chain_id);
            statuses.remove(chain_id);
        }
        
        Ok(())
    }
    
    /// Start all indexers
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on running".to_string())
        })?;
        
        if *running {
            return Ok(());
        }
        
        *running = true;
        
        // Get indexers
        let indexers = {
            let indexers = self.indexers.lock().map_err(|_| {
                Error::Internal("Failed to acquire lock on indexers".to_string())
            })?;
            
            indexers.clone()
        };
        
        // Start each indexer
        for (chain_id, indexer) in indexers {
            let options = IndexingOptions {
                start_height: None, // Start from the latest indexed height
                end_height: None,   // No end height (follow chain head)
                follow_head: true,
                batch_size: self.config.max_blocks_per_batch,
                include_transactions: true,
                transaction_filter: None,
                options: HashMap::new(),
            };
            
            indexer.start_indexing(options).await?;
            
            // Update status
            self.update_indexer_status(&chain_id, IndexerStatus::Syncing).await?;
        }
        
        // Start polling loop
        let proxy = self.clone();
        tokio::spawn(async move {
            proxy.polling_loop().await;
        });
        
        Ok(())
    }
    
    /// Stop all indexers
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on running".to_string())
        })?;
        
        if !*running {
            return Ok(());
        }
        
        *running = false;
        
        // Get indexers
        let indexers = {
            let indexers = self.indexers.lock().map_err(|_| {
                Error::Internal("Failed to acquire lock on indexers".to_string())
            })?;
            
            indexers.clone()
        };
        
        // Stop each indexer
        for (chain_id, indexer) in indexers {
            indexer.stop_indexing().await?;
            
            // Update status
            self.update_indexer_status(&chain_id, IndexerStatus::Paused).await?;
        }
        
        Ok(())
    }
    
    /// Add an event handler
    pub fn add_event_handler(&self, handler: Arc<dyn ProxyEventHandler>) -> Result<()> {
        let mut handlers = self.event_handlers.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on event handlers".to_string())
        })?;
        
        handlers.push(handler);
        
        Ok(())
    }
    
    /// Get the fact receiver channel
    pub fn take_fact_receiver(&self) -> Result<mpsc::Receiver<ExtractedFact>> {
        let mut receiver = self.fact_receiver.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on fact receiver".to_string())
        })?;
        
        receiver.take().ok_or_else(|| {
            Error::Internal("Fact receiver has already been taken".to_string())
        })
    }
    
    /// Add a rule to the rule engine
    pub fn add_rule(&self, rule: ExtractionRule) -> Result<()> {
        self.rule_engine.add_rule(rule)
    }
    
    /// Remove a rule from the rule engine
    pub fn remove_rule(&self, chain_id: &str, rule_id: &str) -> Result<()> {
        self.rule_engine.remove_rule(chain_id, rule_id)
    }
    
    /// Load rules from a TOML file
    pub fn load_rules_from_toml(&self, toml_str: &str) -> Result<()> {
        self.rule_engine.load_rules_from_toml(toml_str)
    }
    
    /// Get the status of a chain
    pub fn get_chain_status(&self, chain_id: &str) -> Result<ChainStatus> {
        let statuses = self.statuses.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on statuses".to_string())
        })?;
        
        statuses.get(chain_id).cloned().ok_or_else(|| {
            Error::Configuration(format!("No status for chain ID '{}'", chain_id))
        })
    }
    
    /// Get the status of all chains
    pub fn get_all_chain_statuses(&self) -> Result<Vec<ChainStatus>> {
        let statuses = self.statuses.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on statuses".to_string())
        })?;
        
        Ok(statuses.values().cloned().collect())
    }
    
    /// Update the status of an indexer
    async fn update_indexer_status(&self, chain_id: &str, status: IndexerStatus) -> Result<()> {
        // Update status
        {
            let mut statuses = self.statuses.lock().map_err(|_| {
                Error::Internal("Failed to acquire lock on statuses".to_string())
            })?;
            
            if let Some(chain_status) = statuses.get_mut(chain_id) {
                chain_status.indexer_status = status;
            }
        }
        
        // Emit event
        self.emit_event(ProxyEvent::IndexerStatusChanged {
            chain_id: chain_id.to_string(),
            status,
        }).await?;
        
        Ok(())
    }
    
    /// Emit an event to all handlers
    async fn emit_event(&self, event: ProxyEvent) -> Result<()> {
        let handlers = {
            let handlers = self.event_handlers.lock().map_err(|_| {
                Error::Internal("Failed to acquire lock on event handlers".to_string())
            })?;
            
            handlers.clone()
        };
        
        for handler in handlers {
            if let Err(e) = handler.handle_event(event.clone()).await {
                log::error!("Error handling event: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Main polling loop for processing blocks and extracting facts
    async fn polling_loop(&self) {
        let poll_interval = Duration::from_secs(self.config.poll_interval_secs);
        
        loop {
            // Check if we should stop
            {
                let running = self.running.lock().unwrap_or_else(|_| {
                    log::error!("Failed to acquire lock on running");
                    Box::new(false)
                });
                
                if !*running {
                    break;
                }
            }
            
            // Process each chain
            if let Ok(chain_ids) = self.get_chain_ids() {
                for chain_id in chain_ids {
                    if let Err(e) = self.process_chain(&chain_id).await {
                        log::error!("Error processing chain {}: {}", chain_id, e);
                        
                        // Emit error event
                        let _ = self.emit_event(ProxyEvent::Error {
                            chain_id: chain_id.clone(),
                            message: e.to_string(),
                        }).await;
                    }
                }
            }
            
            // Sleep before next poll
            tokio::time::sleep(poll_interval).await;
        }
    }
    
    /// Process a single chain
    async fn process_chain(&self, chain_id: &str) -> Result<()> {
        let (indexer, extractor, mut status) = {
            let indexers = self.indexers.lock().map_err(|_| {
                Error::Internal("Failed to acquire lock on indexers".to_string())
            })?;
            
            let extractors = self.extractors.lock().map_err(|_| {
                Error::Internal("Failed to acquire lock on extractors".to_string())
            })?;
            
            let statuses = self.statuses.lock().map_err(|_| {
                Error::Internal("Failed to acquire lock on statuses".to_string())
            })?;
            
            let indexer = indexers.get(chain_id).cloned().ok_or_else(|| {
                Error::Configuration(format!("No indexer for chain ID '{}'", chain_id))
            })?;
            
            let extractor = extractors.get(chain_id).cloned().ok_or_else(|| {
                Error::Configuration(format!("No extractor for chain ID '{}'", chain_id))
            })?;
            
            let status = statuses.get(chain_id).cloned().ok_or_else(|| {
                Error::Configuration(format!("No status for chain ID '{}'", chain_id))
            })?;
            
            (indexer, extractor, status)
        };
        
        // Get latest chain height
        let latest_height = indexer.get_latest_height().await?;
        status.chain_head_height = latest_height;
        
        // Determine range of blocks to process
        let start_height = status.latest_processed_height + 1;
        let end_height = (start_height + self.config.max_blocks_per_batch as u64 - 1)
            .min(latest_height);
        
        // Skip if no new blocks
        if start_height > end_height {
            return Ok(());
        }
        
        // Process blocks in the range
        for height in start_height..=end_height {
            // Get block
            let block = indexer.get_block_by_height(height).await?;
            
            // Extract facts from block
            let mut block_facts = extractor.extract_from_block(&block).await?;
            
            // Get transactions for the block
            let transactions = indexer.get_transactions(&block.hash).await?;
            
            // Extract facts from transactions
            for tx in &transactions {
                let tx_facts = extractor.extract_from_transaction(tx).await?;
                block_facts.extend(tx_facts);
            }
            
            // Update status
            status.latest_processed_height = height;
            status.facts_extracted += block_facts.len() as u64;
            status.last_processed_time = Some(Instant::now());
            
            // Emit block processed event
            self.emit_event(ProxyEvent::BlockProcessed {
                chain_id: chain_id.to_string(),
                block_height: height,
                block_hash: block.hash.clone(),
                fact_count: block_facts.len(),
            }).await?;
            
            // Emit facts extracted event
            if !block_facts.is_empty() {
                self.emit_event(ProxyEvent::FactsExtracted {
                    chain_id: chain_id.to_string(),
                    facts: block_facts.clone(),
                }).await?;
                
                // Send facts to the channel
                for fact in block_facts {
                    // Convert to log entry if needed
                    if let Err(e) = self.fact_sender.send(fact).await {
                        log::error!("Failed to send fact: {}", e);
                    }
                }
            }
        }
        
        // Update status in storage
        {
            let mut statuses = self.statuses.lock().map_err(|_| {
                Error::Internal("Failed to acquire lock on statuses".to_string())
            })?;
            
            if let Some(s) = statuses.get_mut(chain_id) {
                *s = status;
            }
        }
        
        // Update indexer status if needed
        if status.latest_processed_height == status.chain_head_height && 
           status.indexer_status == IndexerStatus::Syncing {
            self.update_indexer_status(chain_id, IndexerStatus::Active).await?;
        }
        
        Ok(())
    }
    
    /// Get all chain IDs
    fn get_chain_ids(&self) -> Result<Vec<String>> {
        let indexers = self.indexers.lock().map_err(|_| {
            Error::Internal("Failed to acquire lock on indexers".to_string())
        })?;
        
        Ok(indexers.keys().cloned().collect())
    }
}

impl Clone for CommitteeProxy {
    fn clone(&self) -> Self {
        // Note: This only clones the references to shared data
        Self {
            config: self.config.clone(),
            indexer_factory: self.indexer_factory.clone(),
            rule_engine: self.rule_engine.clone(),
            indexers: Mutex::new(HashMap::new()),  // Empty to avoid contention
            extractors: Mutex::new(HashMap::new()), // Empty to avoid contention
            statuses: Mutex::new(HashMap::new()),  // Empty to avoid contention
            fact_sender: self.fact_sender.clone(),
            fact_receiver: self.fact_receiver.clone(),
            event_handlers: Mutex::new(Vec::new()), // Empty to avoid contention
            running: Mutex::new(false),  // Force to false in clone
        }
    }
}

/// A basic implementation of a proxy event handler that logs events
pub struct LoggingEventHandler;

#[async_trait]
impl ProxyEventHandler for LoggingEventHandler {
    async fn handle_event(&self, event: ProxyEvent) -> Result<()> {
        match event {
            ProxyEvent::IndexerStatusChanged { chain_id, status } => {
                log::info!("Chain {} indexer status changed to {:?}", chain_id, status);
            },
            ProxyEvent::BlockProcessed { chain_id, block_height, block_hash, fact_count } => {
                log::info!(
                    "Chain {} processed block {} ({}), extracted {} facts",
                    chain_id, block_height, block_hash, fact_count
                );
            },
            ProxyEvent::FactsExtracted { chain_id, facts } => {
                log::debug!(
                    "Chain {} extracted {} facts",
                    chain_id, facts.len()
                );
            },
            ProxyEvent::Error { chain_id, message } => {
                log::error!("Chain {} error: {}", chain_id, message);
            },
        }
        
        Ok(())
    }
} 