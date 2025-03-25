// Committee indexing functionality
// Original file: src/committee/indexer.rs

//! Chain Indexing
//!
//! This module provides functionality for indexing external blockchains and
//! retrieving data from them.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use causality_core::Result;

/// A block from an external chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainBlock {
    /// The block hash
    pub hash: String,
    /// The block height/number
    pub height: u64,
    /// The block timestamp
    pub timestamp: u64,
    /// The parent block hash
    pub parent_hash: String,
    /// The block data
    pub data: serde_json::Value,
    /// Chain-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Transaction within a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainTransaction {
    /// The transaction hash
    pub hash: String,
    /// The block hash this transaction belongs to
    pub block_hash: String,
    /// The sender of the transaction
    pub from: Option<String>,
    /// The recipient of the transaction
    pub to: Option<String>,
    /// The transaction data
    pub data: serde_json::Value,
    /// Chain-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Configuration for a chain indexer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerConfig {
    /// The chain ID
    pub chain_id: String,
    /// The chain type (e.g., "ethereum", "solana", etc.)
    pub chain_type: String,
    /// The RPC endpoint URL
    pub rpc_url: String,
    /// Optional authentication token
    pub auth_token: Option<String>,
    /// Connection timeout
    pub timeout: Duration,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Additional configuration parameters
    pub params: HashMap<String, String>,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        IndexerConfig {
            chain_id: "unknown".to_string(),
            chain_type: "unknown".to_string(),
            rpc_url: "http://localhost:8545".to_string(),
            auth_token: None,
            timeout: Duration::from_secs(30),
            max_retries: 3,
            params: HashMap::new(),
        }
    }
}

/// Status of a chain indexer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexerStatus {
    /// Indexer is not started
    NotStarted,
    /// Indexer is connecting to the chain
    Connecting,
    /// Indexer is syncing data
    Syncing,
    /// Indexer is up to date and actively monitoring
    Active,
    /// Indexer is paused
    Paused,
    /// Indexer encountered an error
    Error,
}

/// Options for indexing blocks
#[derive(Debug, Clone)]
pub struct IndexingOptions {
    /// The starting block height (inclusive)
    pub start_height: Option<u64>,
    /// The ending block height (inclusive)
    pub end_height: Option<u64>,
    /// Whether to follow the chain head
    pub follow_head: bool,
    /// Maximum number of blocks to fetch in one batch
    pub batch_size: u32,
    /// Whether to index transactions
    pub include_transactions: bool,
    /// Filter for specific transaction types or addresses
    pub transaction_filter: Option<String>,
    /// Additional options
    pub options: HashMap<String, String>,
}

impl Default for IndexingOptions {
    fn default() -> Self {
        IndexingOptions {
            start_height: None,
            end_height: None,
            follow_head: true,
            batch_size: 100,
            include_transactions: true,
            transaction_filter: None,
            options: HashMap::new(),
        }
    }
}

/// Interface for chain indexers
#[async_trait]
pub trait ChainIndexer: Send + Sync {
    /// Initialize the indexer
    async fn initialize(&self) -> Result<()>;
    
    /// Start indexing the chain
    async fn start_indexing(&self, options: IndexingOptions) -> Result<()>;
    
    /// Stop indexing the chain
    async fn stop_indexing(&self) -> Result<()>;
    
    /// Get the current status of the indexer
    async fn get_status(&self) -> Result<IndexerStatus>;
    
    /// Get the latest indexed block height
    async fn get_latest_height(&self) -> Result<u64>;
    
    /// Get a block by height
    async fn get_block_by_height(&self, height: u64) -> Result<ChainBlock>;
    
    /// Get a block by hash
    async fn get_block_by_hash(&self, hash: &str) -> Result<ChainBlock>;
    
    /// Get transactions for a block
    async fn get_transactions(&self, block_hash: &str) -> Result<Vec<ChainTransaction>>;
    
    /// Get a transaction by hash
    async fn get_transaction_by_hash(&self, hash: &str) -> Result<ChainTransaction>;
    
    /// Get transactions by address
    async fn get_transactions_by_address(
        &self, 
        address: &str, 
        start_height: Option<u64>, 
        end_height: Option<u64>
    ) -> Result<Vec<ChainTransaction>>;
}

/// A factory for creating chain indexers
pub struct IndexerFactory {
    /// Registry of available indexer types
    indexer_constructors: HashMap<String, Box<dyn Fn(IndexerConfig) -> Arc<dyn ChainIndexer> + Send + Sync>>,
}

impl IndexerFactory {
    /// Create a new indexer factory
    pub fn new() -> Self {
        IndexerFactory {
            indexer_constructors: HashMap::new(),
        }
    }
    
    /// Register an indexer constructor for a chain type
    pub fn register<F>(&mut self, chain_type: &str, constructor: F)
    where
        F: Fn(IndexerConfig) -> Arc<dyn ChainIndexer> + Send + Sync + 'static,
    {
        self.indexer_constructors.insert(chain_type.to_string(), Box::new(constructor));
    }
    
    /// Create an indexer for a given config
    pub fn create_indexer(&self, config: IndexerConfig) -> Result<Arc<dyn ChainIndexer>> {
        if let Some(constructor) = self.indexer_constructors.get(&config.chain_type) {
            Ok(constructor(config))
        } else {
            Err(causality_core::Error::Configuration(format!(
                "Unsupported chain type: {}", config.chain_type
            )))
        }
    }
}

impl Default for IndexerFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// A registry of active chain indexers
pub struct IndexerRegistry {
    /// Map of chain IDs to indexers
    indexers: RwLock<HashMap<String, Arc<dyn ChainIndexer>>>,
    /// Factory for creating indexers
    factory: IndexerFactory,
}

impl IndexerRegistry {
    /// Create a new indexer registry
    pub fn new(factory: IndexerFactory) -> Self {
        IndexerRegistry {
            indexers: RwLock::new(HashMap::new()),
            factory,
        }
    }
    
    /// Register an indexer for a chain
    pub async fn register_indexer(&self, config: IndexerConfig) -> Result<()> {
        let chain_id = config.chain_id.clone();
        let indexer = self.factory.create_indexer(config)?;
        
        // Initialize the indexer
        indexer.initialize().await?;
        
        // Add to registry
        let mut indexers = self.indexers.write().map_err(|_| {
            causality_core::Error::Internal("Failed to acquire write lock on indexers".to_string())
        })?;
        
        indexers.insert(chain_id, indexer);
        
        Ok(())
    }
    
    /// Get an indexer by chain ID
    pub fn get_indexer(&self, chain_id: &str) -> Result<Arc<dyn ChainIndexer>> {
        let indexers = self.indexers.read().map_err(|_| {
            causality_core::Error::Internal("Failed to acquire read lock on indexers".to_string())
        })?;
        
        if let Some(indexer) = indexers.get(chain_id) {
            Ok(indexer.clone())
        } else {
            Err(causality_core::Error::Configuration(format!(
                "No indexer registered for chain ID: {}", chain_id
            )))
        }
    }
    
    /// Remove an indexer by chain ID
    pub async fn remove_indexer(&self, chain_id: &str) -> Result<()> {
        let mut indexers = self.indexers.write().map_err(|_| {
            causality_core::Error::Internal("Failed to acquire write lock on indexers".to_string())
        })?;
        
        if let Some(indexer) = indexers.remove(chain_id) {
            // Stop indexing before removing
            indexer.stop_indexing().await?;
            Ok(())
        } else {
            Err(causality_core::Error::Configuration(format!(
                "No indexer registered for chain ID: {}", chain_id
            )))
        }
    }
    
    /// Get all registered chain IDs
    pub fn get_chain_ids(&self) -> Result<Vec<String>> {
        let indexers = self.indexers.read().map_err(|_| {
            causality_core::Error::Internal("Failed to acquire read lock on indexers".to_string())
        })?;
        
        Ok(indexers.keys().cloned().collect())
    }
} 