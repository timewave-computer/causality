// Indexer module
//
// This module provides blockchain data indexing capabilities for the causality core.

use std::fmt::Debug;
use thiserror::Error;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

use crate::observation::extraction::BlockData;

/// Error type for indexer operations
#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Data error: {0}")]
    Data(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Type alias for indexer results
pub type Result<T> = std::result::Result<T, IndexerError>;

/// Configuration for chain indexers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerConfig {
    /// The chain ID
    pub chain_id: String,
    
    /// Indexer type (e.g., "ethereum", "near", etc.)
    pub indexer_type: String,
    
    /// Chain RPC URL
    pub rpc_url: String,
    
    /// Starting block height (optional)
    pub start_block: Option<u64>,
    
    /// Contract addresses to watch (optional)
    pub contracts: Option<Vec<String>>,
    
    /// Authentication token (optional)
    pub auth_token: Option<String>,
    
    /// Polling interval in seconds
    pub polling_interval: u64,
    
    /// Additional configuration options
    pub options: HashMap<String, String>,
}

/// Status of a chain indexer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerStatus {
    /// The chain ID
    pub chain_id: String,
    
    /// Whether the indexer is running
    pub is_running: bool,
    
    /// Latest block height on the chain
    pub latest_block_height: u64,
    
    /// Latest synced block height
    pub latest_synced_height: u64,
    
    /// Number of blocks processed
    pub blocks_processed: u64,
    
    /// Number of errors encountered
    pub errors: u64,
    
    /// Last error message
    pub last_error: Option<String>,
    
    /// Last update timestamp
    pub last_updated: u64,
}

/// Trait for chain indexers
#[async_trait]
pub trait ChainIndexer: Send + Sync + Debug {
    /// Get the chain ID
    fn chain_id(&self) -> &str;
    
    /// Initialize the indexer
    async fn initialize(&self) -> Result<()>;
    
    /// Start the indexer
    async fn start(&self) -> Result<()>;
    
    /// Stop the indexer
    async fn stop(&self) -> Result<()>;
    
    /// Get the current status
    async fn get_status(&self) -> Result<IndexerStatus>;
    
    /// Get the latest block height on the chain
    async fn get_latest_height(&self) -> Result<u64>;
    
    /// Get the synced block height
    async fn get_synced_height(&self) -> Result<u64>;
    
    /// Set the synced block height
    async fn set_synced_height(&self, height: u64) -> Result<()>;
    
    /// Get a block by height
    async fn get_block(&self, height: u64) -> Result<BlockData>;
    
    /// Get blocks in a range
    async fn get_blocks(&self, start: u64, end: u64) -> Result<Vec<BlockData>> {
        let mut blocks = Vec::with_capacity((end - start) as usize);
        for height in start..=end {
            blocks.push(self.get_block(height).await?);
        }
        Ok(blocks)
    }
}

/// Basic implementation of a chain indexer
pub struct BasicIndexer {
    /// Indexer configuration
    config: IndexerConfig,
    
    /// Indexer status
    status: std::sync::RwLock<IndexerStatus>,
}

impl BasicIndexer {
    /// Create a new basic indexer
    pub fn new(config: IndexerConfig) -> Self {
        let status = IndexerStatus {
            chain_id: config.chain_id.clone(),
            is_running: false,
            latest_block_height: 0,
            latest_synced_height: config.start_block.unwrap_or(0),
            blocks_processed: 0,
            errors: 0,
            last_error: None,
            last_updated: chrono::Utc::now().timestamp() as u64,
        };
        
        BasicIndexer {
            config,
            status: std::sync::RwLock::new(status),
        }
    }
    
    /// Update the status
    fn update_status<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut IndexerStatus),
    {
        let mut status = self.status.write().map_err(|e| 
            IndexerError::Internal(format!("Failed to lock status: {}", e)))?;
            
        updater(&mut status);
        status.last_updated = chrono::Utc::now().timestamp() as u64;
        
        Ok(())
    }
}

impl Debug for BasicIndexer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BasicIndexer")
            .field("config", &self.config)
            .finish()
    }
}

#[async_trait]
impl ChainIndexer for BasicIndexer {
    fn chain_id(&self) -> &str {
        &self.config.chain_id
    }
    
    async fn initialize(&self) -> Result<()> {
        // Simple initialization - could be extended for real implementations
        self.update_status(|status| {
            status.is_running = false;
        })?;
        
        Ok(())
    }
    
    async fn start(&self) -> Result<()> {
        self.update_status(|status| {
            status.is_running = true;
        })?;
        
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        self.update_status(|status| {
            status.is_running = false;
        })?;
        
        Ok(())
    }
    
    async fn get_status(&self) -> Result<IndexerStatus> {
        let status = self.status.read().map_err(|e| 
            IndexerError::Internal(format!("Failed to lock status: {}", e)))?;
            
        Ok(status.clone())
    }
    
    async fn get_latest_height(&self) -> Result<u64> {
        let status = self.status.read().map_err(|e| 
            IndexerError::Internal(format!("Failed to lock status: {}", e)))?;
            
        Ok(status.latest_block_height)
    }
    
    async fn get_synced_height(&self) -> Result<u64> {
        let status = self.status.read().map_err(|e| 
            IndexerError::Internal(format!("Failed to lock status: {}", e)))?;
            
        Ok(status.latest_synced_height)
    }
    
    async fn set_synced_height(&self, height: u64) -> Result<()> {
        self.update_status(|status| {
            status.latest_synced_height = height;
        })?;
        
        Ok(())
    }
    
    async fn get_block(&self, _height: u64) -> Result<BlockData> {
        // This is a placeholder implementation
        // Real implementation would connect to blockchain RPC
        Err(IndexerError::Internal("Not implemented".to_string()))
    }
}

/// Factory for creating chain indexers
pub struct IndexerFactory {
    /// Registered indexer creators
    creators: std::sync::RwLock<HashMap<String, Box<dyn IndexerCreator>>>,
}

impl IndexerFactory {
    /// Create a new indexer factory
    pub fn new() -> Self {
        IndexerFactory {
            creators: std::sync::RwLock::new(HashMap::new()),
        }
    }
    
    /// Register an indexer creator
    pub fn register<T: IndexerCreator + 'static>(&self, creator: T) -> Result<()> {
        let indexer_type = creator.indexer_type();
        
        let mut creators = self.creators.write().map_err(|e| 
            IndexerError::Internal(format!("Failed to lock creators: {}", e)))?;
            
        creators.insert(indexer_type.to_string(), Box::new(creator));
        
        Ok(())
    }
    
    /// Create an indexer from configuration
    pub fn create(&self, config: IndexerConfig) -> Result<Arc<dyn ChainIndexer>> {
        let creators = self.creators.read().map_err(|e| 
            IndexerError::Internal(format!("Failed to lock creators: {}", e)))?;
            
        let creator = creators.get(&config.indexer_type).ok_or_else(|| 
            IndexerError::Configuration(format!("Unsupported indexer type: {}", config.indexer_type)))?;
            
        creator.create(config)
    }
}

impl Default for IndexerFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for creating indexers
pub trait IndexerCreator: Send + Sync {
    /// Get the indexer type
    fn indexer_type(&self) -> &str;
    
    /// Create an indexer from configuration
    fn create(&self, config: IndexerConfig) -> Result<Arc<dyn ChainIndexer>>;
}

/// Basic indexer creator
pub struct BasicIndexerCreator;

impl IndexerCreator for BasicIndexerCreator {
    fn indexer_type(&self) -> &str {
        "basic"
    }
    
    fn create(&self, config: IndexerConfig) -> Result<Arc<dyn ChainIndexer>> {
        Ok(Arc::new(BasicIndexer::new(config)))
    }
} 