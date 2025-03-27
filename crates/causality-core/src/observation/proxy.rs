// Observation proxy functionality
//
// This module provides a proxy for external chain interaction and fact extraction.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;

use causality_types::{Error, Result};
use crate::indexer::{
    ChainIndexer, IndexerConfig, IndexerFactory, IndexerStatus, IndexingOptions
};
use crate::extraction::{
    FactExtractor, ExtractedFact, ExtractionRule, RuleEngine, BasicExtractor
};
use crate::log::{LogEntry, LogStorage};

/// Configuration for an observation proxy
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

/// Event emitted by the observation proxy
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
    async fn handle_event(&self, event: ProxyEvent) -> Result<(), ProxyError>;
}

/// A simple event handler that logs events
pub struct LoggingEventHandler;

#[async_trait]
impl ProxyEventHandler for LoggingEventHandler {
    async fn handle_event(&self, event: ProxyEvent) -> Result<(), ProxyError> {
        match event {
            ProxyEvent::IndexerStatusChanged { chain_id, status } => {
                log::info!("Chain {}: Status changed to {:?}", chain_id, status);
            },
            ProxyEvent::BlockProcessed { chain_id, block_height, block_hash, fact_count } => {
                log::info!(
                    "Chain {}: Processed block {} ({}), extracted {} facts",
                    chain_id, block_height, block_hash, fact_count
                );
            },
            ProxyEvent::FactsExtracted { chain_id, facts } => {
                log::debug!(
                    "Chain {}: Extracted {} facts",
                    chain_id, facts.len()
                );
            },
            ProxyEvent::Error { chain_id, message } => {
                log::error!("Chain {}: Error: {}", chain_id, message);
            },
        }
        
        Ok(())
    }
}

/// An observation proxy that interacts with external chains
pub struct ObservationProxy {
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

impl ObservationProxy {
    /// Create a new observation proxy
    pub fn new(config: ProxyConfig) -> Result<Self> {
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
    
    /// Initialize the proxy
    pub async fn initialize(&self) -> Result<()> {
        // Create indexers and extractors for each chain
        for chain_config in &self.config.chains {
            // Create indexer
            let indexer = self.indexer_factory.create_indexer(chain_config.clone())?;
            
            // Create extractor
            let extractor = Arc::new(BasicExtractor::new(
                chain_config.chain_id.clone(),
                self.rule_engine.clone(),
            ));
            
            // Store indexer and extractor
            {
                let mut indexers = self.indexers.lock().map_err(|_| 
                    Error::Internal("Failed to lock indexers".to_string()))?;
                    
                indexers.insert(chain_config.chain_id.clone(), indexer);
            }
            
            {
                let mut extractors = self.extractors.lock().map_err(|_| 
                    Error::Internal("Failed to lock extractors".to_string()))?;
                    
                extractors.insert(chain_config.chain_id.clone(), extractor);
            }
            
            // Initialize status
            {
                let mut statuses = self.statuses.lock().map_err(|_| 
                    Error::Internal("Failed to lock statuses".to_string()))?;
                    
                statuses.insert(chain_config.chain_id.clone(), ChainStatus {
                    chain_id: chain_config.chain_id.clone(),
                    indexer_status: IndexerStatus::Initializing,
                    latest_processed_height: 0,
                    chain_head_height: 0,
                    facts_extracted: 0,
                    last_processed_time: None,
                });
            }
        }
        
        Ok(())
    }
    
    /// Start the proxy
    pub async fn start(&self) -> Result<()> {
        // Check if already running
        {
            let running = self.running.lock().map_err(|_| 
                Error::Internal("Failed to lock running".to_string()))?;
                
            if *running {
                return Err(Error::Internal("Proxy already running".to_string()));
            }
        }
        
        // Set running flag
        {
            let mut running = self.running.lock().map_err(|_| 
                Error::Internal("Failed to lock running".to_string()))?;
                
            *running = true;
        }
        
        // Connect to chains and start indexing
        for chain_config in &self.config.chains {
            // Get the indexer
            let indexer = {
                let indexers = self.indexers.lock().map_err(|_| 
                    Error::Internal("Failed to lock indexers".to_string()))?;
                    
                indexers.get(&chain_config.chain_id)
                    .ok_or_else(|| 
                        Error::Internal(format!("Indexer not found: {}", chain_config.chain_id))
                    )?
                    .clone()
            };
            
            // Connect to the chain
            indexer.connect().await.map_err(|e| 
                Error::Connection(format!("Failed to connect to chain {}: {}", 
                                         chain_config.chain_id, e)))?;
            
            // Update status
            {
                let mut statuses = self.statuses.lock().map_err(|_| 
                    Error::Internal("Failed to lock statuses".to_string()))?;
                    
                if let Some(status) = statuses.get_mut(&chain_config.chain_id) {
                    status.indexer_status = IndexerStatus::Connected;
                }
            }
            
            // Emit event
            self.emit_event(ProxyEvent::IndexerStatusChanged {
                chain_id: chain_config.chain_id.clone(),
                status: IndexerStatus::Connected,
            }).await?;
            
            // Start polling for new blocks
            let proxy = self.clone();
            let chain_id = chain_config.chain_id.clone();
            
            tokio::spawn(async move {
                if let Err(e) = proxy.poll_chain(&chain_id).await {
                    log::error!("Error polling chain {}: {}", chain_id, e);
                }
            });
        }
        
        Ok(())
    }
    
    /// Stop the proxy
    pub async fn stop(&self) -> Result<()> {
        // Check if running
        {
            let running = self.running.lock().map_err(|_| 
                Error::Internal("Failed to lock running".to_string()))?;
                
            if !*running {
                return Ok(());
            }
        }
        
        // Set running flag
        {
            let mut running = self.running.lock().map_err(|_| 
                Error::Internal("Failed to lock running".to_string()))?;
                
            *running = false;
        }
        
        // Disconnect from chains
        for chain_config in &self.config.chains {
            // Get the indexer
            let indexer = {
                let indexers = self.indexers.lock().map_err(|_| 
                    Error::Internal("Failed to lock indexers".to_string()))?;
                    
                indexers.get(&chain_config.chain_id)
                    .ok_or_else(|| 
                        Error::Internal(format!("Indexer not found: {}", chain_config.chain_id))
                    )?
                    .clone()
            };
            
            // Disconnect from the chain
            indexer.disconnect().await.map_err(|e| 
                Error::Connection(format!("Failed to disconnect from chain {}: {}", 
                                         chain_config.chain_id, e)))?;
            
            // Update status
            {
                let mut statuses = self.statuses.lock().map_err(|_| 
                    Error::Internal("Failed to lock statuses".to_string()))?;
                    
                if let Some(status) = statuses.get_mut(&chain_config.chain_id) {
                    status.indexer_status = IndexerStatus::Disconnected;
                }
            }
            
            // Emit event
            self.emit_event(ProxyEvent::IndexerStatusChanged {
                chain_id: chain_config.chain_id.clone(),
                status: IndexerStatus::Disconnected,
            }).await?;
        }
        
        Ok(())
    }
    
    /// Poll a chain for new blocks
    async fn poll_chain(&self, chain_id: &str) -> Result<()> {
        // Get the indexer and extractor
        let (indexer, extractor) = {
            let indexers = self.indexers.lock().map_err(|_| 
                Error::Internal("Failed to lock indexers".to_string()))?;
                
            let extractors = self.extractors.lock().map_err(|_| 
                Error::Internal("Failed to lock extractors".to_string()))?;
                
            let indexer = indexers.get(chain_id)
                .ok_or_else(|| 
                    Error::Internal(format!("Indexer not found: {}", chain_id))
                )?
                .clone();
                
            let extractor = extractors.get(chain_id)
                .ok_or_else(|| 
                    Error::Internal(format!("Extractor not found: {}", chain_id))
                )?
                .clone();
                
            (indexer, extractor)
        };
        
        // Start from current head
        let mut current_height = indexer.get_synced_height().await.map_err(|e| 
            Error::Data(format!("Failed to get synced height: {}", e)))?;
            
        // Update status
        {
            let mut statuses = self.statuses.lock().map_err(|_| 
                Error::Internal("Failed to lock statuses".to_string()))?;
                
            if let Some(status) = statuses.get_mut(chain_id) {
                status.latest_processed_height = current_height;
            }
        }
        
        // Polling loop
        while self.is_running()? {
            // Get the chain head
            let chain_head = indexer.get_chain_head().await.map_err(|e| 
                Error::Data(format!("Failed to get chain head: {}", e)))?;
                
            // Update status
            {
                let mut statuses = self.statuses.lock().map_err(|_| 
                    Error::Internal("Failed to lock statuses".to_string()))?;
                    
                if let Some(status) = statuses.get_mut(chain_id) {
                    status.chain_head_height = chain_head;
                }
            }
            
            // Process new blocks
            if chain_head > current_height {
                let blocks_to_process = std::cmp::min(
                    chain_head - current_height,
                    self.config.max_blocks_per_batch as u64
                );
                
                for height in current_height + 1..=current_height + blocks_to_process {
                    // Get block data
                    let block_data = indexer.get_block(height).await.map_err(|e| 
                        Error::Data(format!("Failed to get block data: {}", e)))?;
                        
                    // Extract facts
                    let facts = extractor.extract_facts(&block_data).await.map_err(|e| 
                        Error::Extraction(format!("Failed to extract facts: {}", e)))?;
                        
                    // Send facts to reconstructor
                    for fact in &facts {
                        if let Err(e) = self.fact_sender.send(fact.clone()).await {
                            log::error!("Failed to send fact: {}", e);
                        }
                    }
                    
                    // Update status
                    {
                        let mut statuses = self.statuses.lock().map_err(|_| 
                            Error::Internal("Failed to lock statuses".to_string()))?;
                            
                        if let Some(status) = statuses.get_mut(chain_id) {
                            status.latest_processed_height = height;
                            status.facts_extracted += facts.len() as u64;
                            status.last_processed_time = Some(Instant::now());
                        }
                    }
                    
                    // Emit events
                    self.emit_event(ProxyEvent::BlockProcessed {
                        chain_id: chain_id.to_string(),
                        block_height: height,
                        block_hash: block_data.hash.clone(),
                        fact_count: facts.len(),
                    }).await?;
                    
                    if !facts.is_empty() {
                        self.emit_event(ProxyEvent::FactsExtracted {
                            chain_id: chain_id.to_string(),
                            facts: facts.clone(),
                        }).await?;
                    }
                }
                
                current_height += blocks_to_process;
            }
            
            // Wait before polling again
            tokio::time::sleep(Duration::from_secs(self.config.poll_interval_secs)).await;
        }
        
        Ok(())
    }
    
    /// Check if the proxy is running
    fn is_running(&self) -> Result<bool> {
        let running = self.running.lock().map_err(|_| 
            Error::Internal("Failed to lock running".to_string()))?;
            
        Ok(*running)
    }
    
    /// Emit an event to all registered handlers
    async fn emit_event(&self, event: ProxyEvent) -> Result<()> {
        // Get event handlers
        let handlers = {
            let handlers = self.event_handlers.lock().map_err(|_| 
                Error::Internal("Failed to lock event handlers".to_string()))?;
                
            handlers.clone()
        };
        
        // Call each handler
        for handler in handlers {
            if let Err(e) = handler.handle_event(event.clone()).await {
                log::error!("Error in event handler: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Add an event handler
    pub fn add_event_handler(&self, handler: Arc<dyn ProxyEventHandler>) -> Result<()> {
        let mut handlers = self.event_handlers.lock().map_err(|_| 
            Error::Internal("Failed to lock event handlers".to_string()))?;
            
        handlers.push(handler);
        
        Ok(())
    }
    
    /// Take the fact receiver
    pub fn take_fact_receiver(&self) -> Result<mpsc::Receiver<ExtractedFact>> {
        let mut receiver = self.fact_receiver.lock().map_err(|_| 
            Error::Internal("Failed to lock fact receiver".to_string()))?;
            
        receiver.take().ok_or_else(|| 
            Error::Internal("Fact receiver already taken".to_string())
        )
    }
    
    /// Add a rule to the rule engine
    pub fn add_rule(&self, rule: ExtractionRule) -> Result<()> {
        self.rule_engine.add_rule(rule)
    }
    
    /// Load rules from a TOML string
    pub fn load_rules_from_toml(&self, toml_str: &str) -> Result<()> {
        self.rule_engine.load_rules_from_toml(toml_str)
    }
    
    /// Get a chain status
    pub fn get_chain_status(&self, chain_id: &str) -> Result<ChainStatus> {
        let statuses = self.statuses.lock().map_err(|_| 
            Error::Internal("Failed to lock statuses".to_string()))?;
            
        statuses.get(chain_id)
            .cloned()
            .ok_or_else(|| Error::Data(format!("Chain not found: {}", chain_id)))
    }
    
    /// Get all chain statuses
    pub fn get_all_chain_statuses(&self) -> Result<Vec<ChainStatus>> {
        let statuses = self.statuses.lock().map_err(|_| 
            Error::Internal("Failed to lock statuses".to_string()))?;
            
        Ok(statuses.values().cloned().collect())
    }
}

impl Clone for ObservationProxy {
    fn clone(&self) -> Self {
        // Create a new sender that shares the same channel
        let fact_sender = self.fact_sender.clone();
        
        ObservationProxy {
            config: self.config.clone(),
            indexer_factory: self.indexer_factory.clone(),
            rule_engine: self.rule_engine.clone(),
            indexers: Mutex::new(HashMap::new()),
            extractors: Mutex::new(HashMap::new()),
            statuses: Mutex::new(HashMap::new()),
            fact_sender,
            fact_receiver: Arc::new(Mutex::new(None)),
            event_handlers: Mutex::new(Vec::new()),
            running: Mutex::new(false),
        }
    }
} 