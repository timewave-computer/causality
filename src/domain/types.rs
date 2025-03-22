// Domain Types
//
// This module defines common types used by domain adapters and the domain registry.

use std::collections::HashMap;
use std::fmt;
use serde::{Serialize, Deserialize};

use crate::types::{DomainId, BlockHeight, BlockHash, Timestamp};
use crate::fact::{FactId, FactSelector};

/// Domain type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainType {
    /// EVM-compatible chain 
    EVM,
    /// CosmWasm-compatible chain
    CosmWasm,
    /// Substrate-compatible chain
    Substrate,
    /// Local memory domain (for testing)
    Memory,
    /// Bitcoin or Bitcoin-like chain
    Bitcoin,
    /// Unknown domain type
    Unknown,
}

impl fmt::Display for DomainType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainType::EVM => write!(f, "EVM"),
            DomainType::CosmWasm => write!(f, "CosmWasm"),
            DomainType::Substrate => write!(f, "Substrate"),
            DomainType::Memory => write!(f, "Memory"),
            DomainType::Bitcoin => write!(f, "Bitcoin"),
            DomainType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Domain status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainStatus {
    /// Domain is active and fully operational
    Active,
    /// Domain is being initialized
    Initializing,
    /// Domain is in maintenance mode
    Maintenance,
    /// Domain connection has errors
    Error,
    /// Domain is offline
    Offline,
    /// Domain status is unknown
    Unknown,
}

impl fmt::Display for DomainStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainStatus::Active => write!(f, "Active"),
            DomainStatus::Initializing => write!(f, "Initializing"),
            DomainStatus::Maintenance => write!(f, "Maintenance"),
            DomainStatus::Error => write!(f, "Error"),
            DomainStatus::Offline => write!(f, "Offline"),
            DomainStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Currency information for a domain
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrencyInfo {
    /// Symbol (e.g., ETH, BTC)
    pub symbol: String,
    /// Name (e.g., Ethereum, Bitcoin)
    pub name: String,
    /// Decimal places (e.g., 18 for ETH, 8 for BTC)
    pub decimals: u8,
}

/// Domain information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Chain ID (if applicable)
    pub chain_id: Option<String>,
    /// Native currency information
    pub native_currency: Option<CurrencyInfo>,
    /// Domain status
    pub status: DomainStatus,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Time map entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeMapEntry {
    /// Block height
    pub height: BlockHeight,
    /// Block hash
    pub hash: BlockHash,
    /// Timestamp
    pub timestamp: Timestamp,
}

/// Transaction data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction type
    pub tx_type: String,
    /// Sender address (if applicable)
    pub sender: Option<String>,
    /// Recipient address (if applicable)
    pub recipient: Option<String>,
    /// Transaction data
    pub data: Vec<u8>,
    /// Transaction value (if applicable)
    pub value: Option<String>,
    /// Gas limit (if applicable)
    pub gas_limit: Option<u64>,
    /// Gas price (if applicable)
    pub gas_price: Option<u64>,
    /// Nonce (if applicable)
    pub nonce: Option<u64>,
    /// Transaction metadata
    pub metadata: HashMap<String, String>,
}

/// Transaction ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId(pub String);

impl TransactionId {
    /// Create a new transaction ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Get the transaction ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Transaction status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction is pending
    Pending,
    /// Transaction is included in a block
    Included,
    /// Transaction is confirmed
    Confirmed,
    /// Transaction failed
    Failed,
    /// Transaction is rejected
    Rejected,
    /// Transaction status is unknown
    Unknown,
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "Pending"),
            TransactionStatus::Included => write!(f, "Included"),
            TransactionStatus::Confirmed => write!(f, "Confirmed"),
            TransactionStatus::Failed => write!(f, "Failed"),
            TransactionStatus::Rejected => write!(f, "Rejected"),
            TransactionStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Transaction receipt
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Transaction ID
    pub tx_id: TransactionId,
    /// Block height
    pub block_height: Option<BlockHeight>,
    /// Block hash
    pub block_hash: Option<BlockHash>,
    /// Transaction status
    pub status: TransactionStatus,
    /// Gas used (if applicable)
    pub gas_used: Option<u64>,
    /// Effective gas price (if applicable)
    pub effective_gas_price: Option<u64>,
    /// Logs or events emitted by the transaction
    pub logs: Vec<Vec<u8>>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Transaction metadata
    pub metadata: HashMap<String, String>,
}

/// Fact query for observing facts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactQuery {
    /// Fact ID (if querying a specific fact)
    pub fact_id: Option<FactId>,
    /// Block height range
    pub height_range: Option<(BlockHeight, BlockHeight)>,
    /// Time range
    pub time_range: Option<(Timestamp, Timestamp)>,
    /// Fact selectors (for filtering facts)
    pub selectors: Vec<FactSelector>,
    /// Maximum number of facts to return
    pub limit: Option<usize>,
    /// Query metadata
    pub metadata: HashMap<String, String>,
}

impl FactQuery {
    /// Create a new fact query
    pub fn new() -> Self {
        Self {
            fact_id: None,
            height_range: None,
            time_range: None,
            selectors: Vec::new(),
            limit: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Set the fact ID
    pub fn with_fact_id(mut self, fact_id: FactId) -> Self {
        self.fact_id = Some(fact_id);
        self
    }
    
    /// Set the height range
    pub fn with_height_range(mut self, start: BlockHeight, end: BlockHeight) -> Self {
        self.height_range = Some((start, end));
        self
    }
    
    /// Set the time range
    pub fn with_time_range(mut self, start: Timestamp, end: Timestamp) -> Self {
        self.time_range = Some((start, end));
        self
    }
    
    /// Add a fact selector
    pub fn with_selector(mut self, selector: FactSelector) -> Self {
        self.selectors.push(selector);
        self
    }
    
    /// Set the limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
} 