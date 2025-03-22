// Next-generation Domain Adapter trait for standardized fact system

use async_trait::async_trait;
use std::fmt::Debug;
use std::collections::HashMap;

use crate::types::{DomainId, BlockHeight, BlockHash, Timestamp};
use crate::log::fact_types::FactType;
use crate::error::Result;

/// Transaction status codes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Transaction was executed successfully
    Success,
    /// Transaction execution failed
    Failed,
    /// Transaction status is unknown
    Unknown,
    /// Transaction is pending
    Pending,
}

impl ToString for TransactionStatus {
    fn to_string(&self) -> String {
        match self {
            TransactionStatus::Success => "Success".to_string(),
            TransactionStatus::Failed => "Failed".to_string(),
            TransactionStatus::Unknown => "Unknown".to_string(),
            TransactionStatus::Pending => "Pending".to_string(),
        }
    }
}

/// Transaction structure for domain operations
#[derive(Debug, Clone)]
pub struct Transaction {
    /// Domain ID for the transaction
    pub domain_id: DomainId,
    /// Transaction type (e.g., "ethereum_raw", "solana_raw", etc.)
    pub tx_type: String,
    /// Raw transaction data
    pub data: Vec<u8>,
    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

/// Transaction ID reference
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionId(pub String);

impl TransactionId {
    /// Create a new transaction ID
    pub fn new(id: String) -> Self {
        Self(id)
    }
    
    /// Get the transaction ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Transaction receipt information
#[derive(Debug, Clone)]
pub struct TransactionReceipt {
    /// Transaction ID
    pub tx_id: TransactionId,
    /// Domain ID where transaction was submitted
    pub domain_id: DomainId,
    /// Block height of the transaction
    pub block_height: Option<BlockHeight>,
    /// Block hash of the transaction
    pub block_hash: Option<BlockHash>,
    /// Transaction timestamp
    pub timestamp: Option<Timestamp>,
    /// Transaction status
    pub status: TransactionStatus,
    /// Error message if transaction failed
    pub error: Option<String>,
    /// Gas or computation units used
    pub gas_used: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Domain type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainType {
    /// Ethereum Virtual Machine
    EVM,
    /// Solana
    Solana,
    /// Cosmos
    Cosmos,
    /// Layer-2 Solution
    Layer2,
    /// Cross-chain
    CrossChain,
    /// Custom domain
    Custom,
}

impl ToString for DomainType {
    fn to_string(&self) -> String {
        match self {
            DomainType::EVM => "EVM".to_string(),
            DomainType::Solana => "Solana".to_string(),
            DomainType::Cosmos => "Cosmos".to_string(),
            DomainType::Layer2 => "Layer2".to_string(),
            DomainType::CrossChain => "CrossChain".to_string(),
            DomainType::Custom => "Custom".to_string(),
        }
    }
}

/// Domain status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainStatus {
    /// Domain is active
    Active,
    /// Domain is inactive
    Inactive,
    /// Domain is in maintenance
    Maintenance,
    /// Domain is deprecated
    Deprecated,
}

impl ToString for DomainStatus {
    fn to_string(&self) -> String {
        match self {
            DomainStatus::Active => "Active".to_string(),
            DomainStatus::Inactive => "Inactive".to_string(),
            DomainStatus::Maintenance => "Maintenance".to_string(),
            DomainStatus::Deprecated => "Deprecated".to_string(),
        }
    }
}

/// Domain information structure
#[derive(Debug, Clone)]
pub struct DomainInfo {
    /// Domain ID
    pub id: DomainId,
    /// Domain type
    pub domain_type: DomainType,
    /// Domain name
    pub name: String,
    /// Domain description
    pub description: Option<String>,
    /// RPC URL
    pub rpc_url: Option<String>,
    /// Explorer URL
    pub explorer_url: Option<String>,
    /// Chain ID
    pub chain_id: Option<u64>,
    /// Native currency symbol
    pub native_currency: Option<String>,
    /// Domain status
    pub status: DomainStatus,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Fact query parameters for observing domain facts
#[derive(Debug, Clone)]
pub struct FactQuery {
    /// Domain ID to query
    pub domain_id: DomainId,
    /// Type of fact (e.g., "balance", "block", "transaction", etc.)
    pub fact_type: String,
    /// Query parameters (key-value pairs)
    pub parameters: HashMap<String, String>,
    /// Optional block height
    pub block_height: Option<BlockHeight>,
    /// Optional block hash
    pub block_hash: Option<BlockHash>,
    /// Optional timestamp
    pub timestamp: Option<Timestamp>,
}

/// Time map entry for domain time synchronization
#[derive(Debug, Clone)]
pub struct TimeMapEntry {
    /// Domain ID
    pub domain_id: DomainId,
    /// Block height
    pub height: BlockHeight,
    /// Block hash
    pub hash: BlockHash,
    /// Timestamp
    pub timestamp: Timestamp,
    /// Confidence level
    pub confidence: f64,
    /// Whether the entry is verified
    pub verified: bool,
    /// Source of the time information
    pub source: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl TimeMapEntry {
    /// Create a new time map entry
    pub fn new(domain_id: DomainId, height: BlockHeight, hash: BlockHash, timestamp: Timestamp) -> Self {
        Self {
            domain_id,
            height,
            hash,
            timestamp,
            confidence: 0.0,
            verified: false,
            source: "unknown".to_string(),
            metadata: HashMap::new(),
        }
    }
    
    /// Set confidence level
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
    
    /// Set verification status
    pub fn with_verification(mut self, verified: bool) -> Self {
        self.verified = verified;
        self
    }
    
    /// Set source
    pub fn with_source<S: ToString>(mut self, source: S) -> Self {
        self.source = source.to_string();
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Domain adapter trait for interacting with external domains
#[async_trait]
pub trait DomainAdapter: Debug + Send + Sync {
    /// Get domain ID
    fn domain_id(&self) -> &DomainId;
    
    /// Get domain information
    async fn domain_info(&self) -> Result<DomainInfo>;
    
    /// Get current block height
    async fn current_height(&self) -> Result<BlockHeight>;
    
    /// Get current block hash
    async fn current_hash(&self) -> Result<BlockHash>;
    
    /// Get current timestamp
    async fn current_timestamp(&self) -> Result<Timestamp>;
    
    /// Observe a fact in this domain
    ///
    /// This method retrieves a fact from the domain based on the query parameters.
    /// This method has been updated to return FactType directly,
    /// which represents the standardized fact system.
    async fn observe_fact(&self, query: FactQuery) -> Result<FactType>;
    
    /// Submit a transaction to this domain
    async fn submit_transaction(&self, transaction: Transaction) -> Result<TransactionId>;
    
    /// Get a transaction receipt
    async fn get_transaction_receipt(&self, tx_id: &TransactionId) -> Result<TransactionReceipt>;
    
    /// Get time mapping information
    async fn get_time_map(&self) -> Result<TimeMapEntry>;
    
    /// Verify a block hash at a given height
    async fn verify_block(&self, height: BlockHeight, hash: &BlockHash) -> Result<bool>;
    
    /// Check connectivity to the domain
    async fn check_connectivity(&self) -> Result<bool>;
} 